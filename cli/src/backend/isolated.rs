//! An OpenVINO model in a child process.
//!
//! OpenVINO reports some driver failures (`CL_OUT_OF_RESOURCES` on an Intel
//! GPU) as a C++ exception nobody catches, and the whole process aborts with
//! a core dump. So the model lives in `pionus genai-worker`, and a crash
//! there is an error here: the commit loads the model again in a fresh
//! worker, once, and only then gives up with an error.
//!
//! The protocol is one JSON line per message, each tagged [`TAG`] because
//! the libraries print to stdout too. The parent sends the
//! [`GenaiOptions`], the worker answers [`Response::Ready`] or
//! [`Response::Failed`]; then each [`Request`] gets a [`Response`].

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::backend::genai::{GenaiBackend, GenaiOptions, device_prompt_budget};
use crate::backend::{Backend, BackendError, GenerationConfig, Metrics};
use crate::prompt::Prompt;

#[cfg(test)]
mod tests;

/// The hidden subcommand that runs the worker.
pub const SUBCOMMAND: &str = "genai-worker";

const TAG: &str = "@pionus ";

/// How long the worker may take to load the model (the NPU compiles it on
/// the first run) and to answer one request before it counts as hung.
const LOAD_TIMEOUT: Duration = Duration::from_secs(600);
const REPLY_TIMEOUT: Duration = Duration::from_secs(180);

/// The running binary. Linux appends " (deleted)" to the path when
/// `pionus update` replaced the file under a running process; the new binary
/// is at the path without it.
pub fn current_exe() -> Result<PathBuf, BackendError> {
    let exe = std::env::current_exe().map_err(|e| BackendError::Unavailable(e.to_string()))?;
    Ok(without_deleted(exe))
}

fn without_deleted(exe: PathBuf) -> PathBuf {
    exe.to_str()
        .and_then(|path| path.strip_suffix(" (deleted)"))
        .map_or(exe.clone(), PathBuf::from)
}

/// A lock whose holder panicked still guards usable data here.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Debug, Serialize, Deserialize)]
struct Request {
    prompt: Prompt,
    config: GenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
enum Response {
    Ready {
        description: String,
    },
    Reply {
        text: String,
        metrics: Option<Metrics>,
    },
    Failed(BackendError),
}

/// The worker: loads the model of the first line, then answers each request.
/// Returns when the parent closes the pipe.
pub fn work(input: impl BufRead, mut output: impl Write) -> Result<(), BackendError> {
    let io = |e: std::io::Error| BackendError::Unavailable(e.to_string());
    let mut send = |response: &Response| -> Result<(), BackendError> {
        let line = serde_json::to_string(response).expect("response serializes");
        writeln!(output, "{TAG}{line}")
            .and_then(|()| output.flush())
            .map_err(io)
    };
    let mut lines = input.lines();
    let first = lines.next().transpose().map_err(io)?.unwrap_or_default();
    let options: GenaiOptions = serde_json::from_str(&first)
        .map_err(|e| BackendError::Unavailable(format!("bad options: {e}")))?;
    let backend = match GenaiBackend::load(&options) {
        Ok(backend) => backend,
        Err(err) => return send(&Response::Failed(err)),
    };
    send(&Response::Ready {
        description: backend.describe(),
    })?;
    for line in lines {
        let request: Request = serde_json::from_str(&line.map_err(io)?)
            .map_err(|e| BackendError::Generation(format!("bad request: {e}")))?;
        send(&match backend.generate(&request.prompt, &request.config) {
            Ok(text) => Response::Reply {
                text,
                metrics: backend.last_metrics(),
            },
            Err(err) => Response::Failed(err),
        })?;
    }
    Ok(())
}

/// A running worker.
struct Worker {
    child: Child,
    stdin: ChildStdin,
    /// The JSON of each tagged line the worker wrote to stdout.
    answers: Receiver<String>,
    /// The last line the worker wrote to stderr: why it died.
    last_error: Arc<Mutex<String>>,
    stderr: Option<JoinHandle<()>>,
    description: String,
}

impl Worker {
    fn start(
        options: &GenaiOptions,
        mut command: Command,
        load_timeout: Duration,
    ) -> Result<Self, BackendError> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| BackendError::Unavailable(format!("cannot start the worker: {e}")))?;
        let last_error = Arc::new(Mutex::new(String::new()));
        let stderr = child.stderr.take().expect("stderr is piped");
        let seen = Arc::clone(&last_error);
        let stderr = std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if !line.trim().is_empty() {
                    *lock(&seen) = line;
                }
            }
        });
        let stdout = child.stdout.take().expect("stdout is piped");
        let (sender, answers) = channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                // A library may write without a newline just before the tag.
                if let Some((_, json)) = line.split_once(TAG)
                    && sender.send(json.trim().to_owned()).is_err()
                {
                    break;
                }
            }
        });
        let mut worker = Self {
            stdin: child.stdin.take().expect("stdin is piped"),
            answers,
            child,
            last_error,
            stderr: Some(stderr),
            description: String::new(),
        };
        let options = serde_json::to_string(options).expect("options serialize");
        match worker.call(&options, load_timeout)? {
            Response::Ready { description } => worker.description = description,
            Response::Failed(err) => return Err(err),
            Response::Reply { .. } => {
                return Err(BackendError::Unavailable(
                    "the worker did not load".to_owned(),
                ));
            }
        }
        Ok(worker)
    }

    /// Sends `line` and waits for the answer. `Err` is a worker that died or
    /// hung (and was killed).
    fn call(&mut self, line: &str, timeout: Duration) -> Result<Response, BackendError> {
        let sent = writeln!(self.stdin, "{line}").and_then(|()| self.stdin.flush());
        let why = if sent.is_err() {
            "closed its input".to_owned()
        } else {
            match self.answers.recv_timeout(timeout) {
                Ok(json) => {
                    return serde_json::from_str(&json).map_err(|e| {
                        BackendError::Generation(format!("unreadable answer from the worker: {e}"))
                    });
                }
                Err(RecvTimeoutError::Timeout) => format!("did not answer in {timeout:?}"),
                Err(RecvTimeoutError::Disconnected) => "crashed".to_owned(),
            }
        };
        Err(self.crashed(&why))
    }

    fn crashed(&mut self, why: &str) -> BackendError {
        let _ = self.child.kill();
        let status = self
            .child
            .wait()
            .map_or_else(|e| e.to_string(), |s| s.to_string());
        // Not `join`: a grandchild holding the pipe open must not hang the error path.
        if let Some(stderr) = self.stderr.take() {
            for _ in 0..20 {
                if stderr.is_finished() {
                    let _ = stderr.join();
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        let last = lock(&self.last_error).clone();
        BackendError::Generation(format!(
            "the OpenVINO worker {why} ({status}): {}",
            if last.is_empty() { "no output" } else { &last }
        ))
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct IsolatedBackend<F: Fn() -> Command> {
    options: GenaiOptions,
    command: F,
    worker: Mutex<Worker>,
    last_metrics: Mutex<Option<Metrics>>,
    load_timeout: Duration,
    reply_timeout: Duration,
}

impl<F: Fn() -> Command> IsolatedBackend<F> {
    /// Starts a worker with `command` and waits until it has loaded the model.
    pub fn start(options: &GenaiOptions, command: F) -> Result<Self, BackendError> {
        Self::start_with(options, command, LOAD_TIMEOUT, REPLY_TIMEOUT)
    }

    fn start_with(
        options: &GenaiOptions,
        command: F,
        load_timeout: Duration,
        reply_timeout: Duration,
    ) -> Result<Self, BackendError> {
        let worker = Worker::start(options, command(), load_timeout)?;
        Ok(Self {
            options: options.clone(),
            command,
            worker: Mutex::new(worker),
            last_metrics: Mutex::new(None),
            load_timeout,
            reply_timeout,
        })
    }

    fn ask(&self, request: &str) -> Result<Response, BackendError> {
        let mut worker = lock(&self.worker);
        match worker.call(request, self.reply_timeout) {
            Err(crash) => {
                tracing::warn!("{crash}; loading the model again");
                *worker = Worker::start(&self.options, (self.command)(), self.load_timeout)?;
                worker.call(request, self.reply_timeout)
            }
            answer => answer,
        }
    }
}

impl<F: Fn() -> Command> Backend for IsolatedBackend<F> {
    fn describe(&self) -> String {
        lock(&self.worker).description.clone()
    }

    fn prompt_budget(&self, requested: usize) -> usize {
        device_prompt_budget(self.options.device, requested)
    }

    fn last_metrics(&self) -> Option<Metrics> {
        lock(&self.last_metrics).clone()
    }

    fn generate(&self, prompt: &Prompt, config: &GenerationConfig) -> Result<String, BackendError> {
        let request = serde_json::to_string(&Request {
            prompt: prompt.clone(),
            config: config.clone(),
        })
        .expect("request serializes");
        match self.ask(&request)? {
            Response::Reply { text, metrics } => {
                *lock(&self.last_metrics) = metrics;
                Ok(text)
            }
            Response::Failed(err) => Err(err),
            Response::Ready { .. } => Err(BackendError::Generation(
                "unexpected answer from the worker".to_owned(),
            )),
        }
    }
}
