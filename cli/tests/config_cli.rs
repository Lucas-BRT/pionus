//! `pionus config` and how settings reach generation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use tempfile::TempDir;

mod common;
use common::{TestRepo, pionus};

/// A global config home of its own for each test.
struct Env {
    config_home: TempDir,
}

impl Env {
    fn new() -> Self {
        Self {
            config_home: TempDir::new().unwrap(),
        }
    }

    fn global_file(&self) -> PathBuf {
        self.config_home.path().join("pionus/config.toml")
    }

    fn run(&self, args: &[&str], dir: &Path) -> Output {
        self.run_with(args, dir, &[])
    }

    fn run_with(&self, args: &[&str], dir: &Path, extra: &[(&str, &str)]) -> Output {
        let mut env = vec![("XDG_CONFIG_HOME", self.config_home.path().to_str().unwrap())];
        env.extend_from_slice(extra);
        pionus(args, dir, &env)
    }

    /// Runs and expects success, returning stdout.
    fn ok(&self, args: &[&str], dir: &Path) -> String {
        let output = self.run(args, dir);
        assert!(output.status.success(), "{args:?}: {}", stderr(&output));
        String::from_utf8(output.stdout).unwrap()
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn staged_repo() -> TestRepo {
    let repo = TestRepo::new();
    repo.commit("feat(git): add history");
    repo.commit("fix(git): handle detached HEAD");
    repo.write("main.rs", "fn main() {}\n");
    repo.git(&["add", "main.rs"]);
    repo
}

#[test]
fn init_writes_a_template_once() {
    let env = Env::new();
    let dir = TempDir::new().unwrap();
    let out = env.ok(&["config", "init"], dir.path());
    assert_eq!(out, format!("wrote {}\n", env.global_file().display()));
    let template = fs::read_to_string(env.global_file()).unwrap();
    assert!(template.contains("# device = \"auto\""), "{template}");
    assert!(template.contains("# Environment: PIONUS_DEVICE."));

    let again = env.run(&["config", "init"], dir.path());
    assert_eq!(again.status.code(), Some(1));
    assert!(stderr(&again).contains("use --force"));
    env.ok(&["config", "init", "--force"], dir.path());

    // A file of comments only changes nothing.
    let out = env.ok(&["config", "list", "--show-origin"], dir.path());
    assert!(out.lines().all(|l| l.ends_with("(default)")), "{out}");
}

#[test]
fn repository_template_has_only_shared_settings() {
    let env = Env::new();
    let repo = TestRepo::new();
    env.ok(&["config", "init", "--repo"], repo.path());
    let template = fs::read_to_string(repo.path().join(".pionus.toml")).unwrap();
    assert!(template.contains("# convention = \"auto\""));
    assert!(!template.contains("device"), "{template}");
}

#[test]
fn set_get_unset_in_the_global_config() {
    let env = Env::new();
    let dir = TempDir::new().unwrap();
    assert_eq!(
        env.ok(&["config", "get", "model.device"], dir.path()),
        "auto\n"
    );

    let out = env.ok(&["config", "set", "model.device", "cpu"], dir.path());
    assert_eq!(
        out,
        format!(
            "model.device = cpu  (global {})\n",
            env.global_file().display()
        )
    );
    assert_eq!(
        env.ok(&["config", "get", "model.device"], dir.path()),
        "cpu\n"
    );
    let list = env.ok(&["config", "list", "--show-origin"], dir.path());
    assert!(list.contains(&format!(
        "model.device = cpu  (global {})",
        env.global_file().display()
    )));

    env.ok(&["config", "unset", "model.device"], dir.path());
    assert_eq!(
        env.ok(&["config", "get", "model.device"], dir.path()),
        "auto\n"
    );
    assert_eq!(
        env.ok(&["config", "path"], dir.path()),
        format!("{}\n", env.global_file().display())
    );
}

#[test]
fn invalid_settings_are_refused_and_nothing_is_written() {
    let env = Env::new();
    let dir = TempDir::new().unwrap();
    for (args, problem) in [
        (
            &["config", "set", "model.device", "tpu"][..],
            "expected one of auto, npu, intel-gpu, vulkan, cpu, openvino-cpu",
        ),
        (
            &["config", "set", "model.devise", "cpu"],
            "unknown key `model.devise`",
        ),
        (
            &["config", "set", "style.history_depth", "-3"],
            "expected a whole number",
        ),
        (&["config", "get", "nope"], "unknown key `nope`"),
    ] {
        let output = env.run(args, dir.path());
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(
            stderr(&output).contains(problem),
            "{args:?}: {}",
            stderr(&output)
        );
    }
    assert!(!env.global_file().exists());
}

#[test]
fn repository_settings_override_global_ones_inside_it() {
    let env = Env::new();
    let repo = TestRepo::new();
    let elsewhere = TempDir::new().unwrap();
    env.ok(&["config", "set", "style.convention", "free"], repo.path());
    let out = env.ok(
        &["config", "set", "--repo", "style.convention", "gitmoji"],
        repo.path(),
    );
    assert!(out.contains("(repository "), "{out}");
    assert!(repo.path().join(".pionus.toml").exists());

    assert_eq!(
        env.ok(&["config", "get", "style.convention"], repo.path()),
        "gitmoji\n"
    );
    assert_eq!(
        env.ok(&["config", "get", "style.convention"], elsewhere.path()),
        "free\n"
    );
    let path = repo.path().to_str().unwrap();
    assert_eq!(
        env.ok(
            &["config", "get", "style.convention", "-C", path],
            elsewhere.path()
        ),
        "gitmoji\n"
    );

    // A global change the repository overrides is pointed out.
    let output = env.run(&["config", "set", "style.convention", "free"], repo.path());
    assert!(
        stderr(&output).contains("note: style.convention is still gitmoji, from the repository"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn a_file_with_the_language_keys_of_an_earlier_version_still_works() {
    let env = Env::new();
    let repo = TestRepo::new();
    std::fs::create_dir_all(env.global_file().parent().unwrap()).unwrap();
    std::fs::write(
        env.global_file(),
        "[style]\nlanguage = \"portuguese\"\ndefault_language = \"english\"\n",
    )
    .unwrap();
    assert_eq!(
        env.ok(&["config", "get", "style.convention"], repo.path()),
        "auto\n"
    );
    let listing = env.ok(&["config", "list"], repo.path());
    assert!(!listing.contains("language"), "{listing}");
    // The keys are gone: they cannot be set again.
    let output = env.run(&["config", "set", "style.language", "english"], repo.path());
    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("unknown key `style.language`"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn the_subject_limit_is_set_globally_or_for_a_repository() {
    let env = Env::new();
    let repo = staged_repo();
    let prompt = |env: &Env| env.ok(&["prompt"], repo.path());
    assert!(prompt(&env).contains("at most 72 characters"));

    env.ok(&["config", "set", "style.subject_max", "90"], repo.path());
    assert!(prompt(&env).contains("at most 90 characters"));
    env.ok(
        &["config", "set", "--repo", "style.subject_max", "100"],
        repo.path(),
    );
    assert!(prompt(&env).contains("at most 100 characters"));
    assert!(
        std::fs::read_to_string(repo.path().join(".pionus.toml"))
            .unwrap()
            .contains("subject_max = 100")
    );
    for bad in ["10", "500", "wide"] {
        let output = env.run(&["config", "set", "style.subject_max", bad], repo.path());
        assert!(!output.status.success(), "{bad} was accepted");
    }
    env.ok(
        &["config", "unset", "--repo", "style.subject_max"],
        repo.path(),
    );
    assert!(prompt(&env).contains("at most 90 characters"));
}

#[test]
fn machine_settings_cannot_go_in_the_repository() {
    let env = Env::new();
    let repo = TestRepo::new();
    let output = env.run(
        &["config", "set", "--repo", "model.dir", "/tmp/m"],
        repo.path(),
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("can only be set in the global config"));

    // Nor by editing the file by hand.
    repo.write(".pionus.toml", "backend = \"mock\"\n");
    let output = env.run(&["config", "list"], repo.path());
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("`backend` can only be set in the global config"));
}

#[test]
fn environment_overrides_files_and_is_shown() {
    let env = Env::new();
    let dir = TempDir::new().unwrap();
    env.ok(&["config", "set", "model.device", "gpu"], dir.path());
    let device = [("PIONUS_DEVICE", "cpu")];
    let output = env.run_with(&["config", "list", "--show-origin"], dir.path(), &device);
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("model.device = cpu  (environment PIONUS_DEVICE)")
    );
    let output = env.run_with(
        &["config", "set", "model.device", "npu"],
        dir.path(),
        &device,
    );
    assert!(stderr(&output).contains("note: model.device is still cpu, from the environment"));
}

#[test]
fn a_mode_shows_what_it_chose_and_a_written_setting_beats_it() {
    let env = Env::new();
    let dir = TempDir::new().unwrap();
    env.ok(&["config", "set", "model.mode", "fast"], dir.path());
    let listed = env.ok(&["config", "list", "--show-origin"], dir.path());
    assert!(
        listed.contains("model.name = pionus-0.5b-v0.3  (mode fast)"),
        "{listed}"
    );
    assert!(
        listed.contains("model.max_prompt_tokens = 1024  (default)"),
        "{listed}"
    );
    let models = env.ok(&["model", "list"], dir.path());
    assert!(
        models.contains("In use: pionus-0.5b-v0.3 (model.name, mode fast)."),
        "{models}"
    );
    // The environment is written explicitly: it beats the mode for the model
    // and leaves the prompt to it, which fast does not set.
    let output = env.run_with(
        &["config", "list", "--show-origin"],
        dir.path(),
        &[("PIONUS_MODEL", "qwen2.5-coder-7b")],
    );
    let listed = String::from_utf8_lossy(&output.stdout);
    assert!(
        listed.contains("model.name = qwen2.5-coder-7b  (environment PIONUS_MODEL)"),
        "{listed}"
    );
    assert!(
        listed.contains("model.max_prompt_tokens = 1024  (default)"),
        "{listed}"
    );
    // `quality` asks for a longer prompt on the llama.cpp devices only, so the
    // setting itself stays what it was.
    env.ok(&["config", "set", "model.mode", "quality"], dir.path());
    let listed = env.ok(&["config", "list", "--show-origin"], dir.path());
    assert!(
        listed.contains("model.max_prompt_tokens = 1024  (default)"),
        "{listed}"
    );
    assert!(
        listed.contains("model.name = pionus-1.5b-v0.3  (mode quality)"),
        "{listed}"
    );
    // A mode of the repository would reach every machine that opens it.
    let repo = staged_repo();
    let output = env.run(
        &["config", "set", "--repo", "model.mode", "fast"],
        repo.path(),
    );
    assert!(!output.status.success());
}

#[test]
fn a_trace_records_the_mode_asked_for_and_only_then() {
    let env = Env::new();
    let repo = staged_repo();
    let dir = TempDir::new().unwrap();
    let trace = |mode: Option<&str>| -> String {
        let path = dir.path().join("trace.json");
        let path = path.to_str().unwrap();
        let args = [
            "--backend",
            "mock",
            "--mock-response",
            "feat: add main",
            "--trace",
            path,
        ];
        let extra: Vec<(&str, &str)> = mode.map(|m| ("PIONUS_MODE", m)).into_iter().collect();
        let output = env.run_with(&args, repo.path(), &extra);
        assert!(output.status.success(), "{}", stderr(&output));
        fs::read_to_string(path).unwrap()
    };
    assert!(trace(Some("fast")).contains("\"mode\": \"fast\""));
    assert!(trace(Some("quality")).contains("\"mode\": \"quality\""));
    assert!(!trace(None).contains("\"mode\""));
    assert!(!trace(Some("off")).contains("\"mode\""));
    // `fast` also says it asked for one reply.
    assert!(trace(Some("fast")).contains("\"max_attempts\": 1"));
}

#[test]
fn global_backend_setting_reaches_generation() {
    let env = Env::new();
    let repo = staged_repo();
    // Without --backend, the default would try to load the real model.
    env.ok(&["config", "set", "backend", "mock"], repo.path());
    let out = env.ok(&["--mock-response", "feat: add main"], repo.path());
    assert_eq!(out, "feat: add main\n");
    // A flag still wins.
    let output = env.run(&["-v", "--backend", "openvino"], repo.path());
    assert!(
        stderr(&output).contains("no model in"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn model_dir_setting_is_used_and_relative_flags_are_resolved() {
    let env = Env::new();
    let repo = staged_repo();
    let models = TempDir::new().unwrap();
    let dir = models.path().join("mine");
    env.ok(
        &["config", "set", "model.dir", dir.to_str().unwrap()],
        repo.path(),
    );
    // OpenVINO's devices, whatever hardware the machine running the tests has.
    let output = env.run(&["-v", "--backend", "openvino"], repo.path());
    assert!(
        stderr(&output).contains(&format!("no model in {};", dir.display())),
        "{}",
        stderr(&output)
    );
    let output = env.run(
        &[
            "-v",
            "--backend",
            "openvino",
            "--model-dir",
            "relative/model",
        ],
        repo.path(),
    );
    let expected = std::path::absolute(repo.path().join("relative/model")).unwrap();
    assert!(
        stderr(&output).contains(&format!("no model in {};", expected.display())),
        "{}",
        stderr(&output)
    );
}

#[test]
fn forced_convention_reaches_the_prompt_and_the_trace() {
    let env = Env::new();
    let repo = staged_repo();
    env.ok(
        &["config", "set", "--repo", "style.convention", "gitmoji"],
        repo.path(),
    );

    let prompt = env.ok(&["prompt"], repo.path());
    assert!(
        prompt.contains("Start the subject with one gitmoji"),
        "{prompt}"
    );
    assert!(
        !prompt.contains("feat(git): add history"),
        "contradicting examples are dropped"
    );

    let trace = repo.path().join("trace.json");
    let out = env.ok(
        &[
            "--backend",
            "mock",
            "--mock-response",
            "✨ add main",
            "--trace",
            trace.to_str().unwrap(),
        ],
        repo.path(),
    );
    assert_eq!(out, "✨ add main\n");
    let json = fs::read_to_string(&trace).unwrap();
    assert!(json.contains("\"convention\": \"gitmoji\""), "{json}");
    // The trace carries the settings: it replays without the repository.
    let elsewhere = TempDir::new().unwrap();
    env.ok(&["replay", trace.to_str().unwrap()], elsewhere.path());
}

#[test]
fn history_depth_setting_reaches_the_context() {
    let env = Env::new();
    let repo = staged_repo();
    env.ok(
        &["config", "set", "--repo", "style.history_depth", "1"],
        repo.path(),
    );
    let out = env.ok(&["context"], repo.path());
    assert!(out.contains("(from 1 commits)"), "{out}");
}

#[test]
fn broken_config_files_are_reported_with_their_path() {
    let env = Env::new();
    let repo = staged_repo();
    repo.write(".pionus.toml", "[style\n");
    let output = env.run(&["--backend", "mock"], repo.path());
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("cannot parse") && stderr(&output).contains(".pionus.toml"),
        "{}",
        stderr(&output)
    );
}
