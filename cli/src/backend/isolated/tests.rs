use std::path::Path;
use std::time::Duration;

use super::*;
use crate::backend::genai::{Device, NpuTuning};

fn options(dir: &Path) -> GenaiOptions {
    GenaiOptions {
        model_dir: dir.join("model"),
        device: Device::Gpu,
        cache_dir: dir.join("cache"),
        max_prompt_tokens: 1024,
        max_new_tokens: 128,
        npu: NpuTuning::default(),
    }
}

fn prompt() -> Prompt {
    Prompt {
        system: "s".to_owned(),
        user: "u".to_owned(),
    }
}

const READY: &str = r#"read -r _; echo '@pionus {"Ready":{"description":"fake"}}'; "#;
const REPLY: &str =
    r#"read -r _; echo 'noise'; echo '@pionus {"Reply":{"text":"feat: x","metrics":null}}'; "#;

/// A worker that runs `script`; `$M` is a file that outlives the worker.
fn backend(
    dir: &Path,
    script: &str,
) -> Result<IsolatedBackend<impl Fn() -> Command + use<>>, BackendError> {
    let script = script.to_owned();
    let marker = dir.join("marker");
    IsolatedBackend::start(&options(dir), move || {
        let mut command = Command::new("sh");
        command.arg("-c").arg(&script).env("M", &marker);
        command
    })
}

#[test]
fn a_reply_comes_back_and_stdout_noise_is_skipped() {
    let dir = tempfile::tempdir().unwrap();
    let backend = backend(dir.path(), &format!("{READY}{REPLY}")).unwrap();
    assert_eq!(backend.describe(), "fake");
    assert_eq!(
        backend
            .generate(&prompt(), &GenerationConfig::default())
            .unwrap(),
        "feat: x"
    );
}

#[test]
fn a_crash_while_generating_loads_again_and_asks_again() {
    let dir = tempfile::tempdir().unwrap();
    // Aborts the first time it is asked, answers the second.
    let script = format!(
        "{READY}read -r _; if [ -e \"$M\" ]; then echo '@pionus {{\"Reply\":{{\"text\":\"fix: y\",\"metrics\":null}}}}'; else touch \"$M\"; kill -ABRT $$; fi"
    );
    let backend = backend(dir.path(), &script).unwrap();
    assert_eq!(
        backend
            .generate(&prompt(), &GenerationConfig::default())
            .unwrap(),
        "fix: y"
    );
}

#[test]
fn a_worker_that_keeps_crashing_is_an_error_not_an_abort() {
    let dir = tempfile::tempdir().unwrap();
    let script = format!("{READY}read -r _; echo 'CL_OUT_OF_RESOURCES' >&2; kill -ABRT $$");
    let backend = backend(dir.path(), &script).unwrap();
    let err = backend
        .generate(&prompt(), &GenerationConfig::default())
        .unwrap_err();
    let BackendError::Generation(text) = err else {
        panic!("{err:?}")
    };
    assert!(text.contains("crashed"), "{text}");
    assert!(text.contains("CL_OUT_OF_RESOURCES"), "{text}");
}

#[test]
fn a_crash_while_loading_is_an_error_so_the_next_device_can_be_tried() {
    let dir = tempfile::tempdir().unwrap();
    let err = backend(dir.path(), "read -r _; kill -ABRT $$")
        .err()
        .unwrap();
    assert!(matches!(err, BackendError::Generation(t) if t.contains("crashed")));
}

#[test]
fn a_load_error_is_passed_on_as_it_is() {
    let dir = tempfile::tempdir().unwrap();
    let script = r#"read -r _; echo '@pionus {"Failed":{"NotInstalled":"no model"}}'"#;
    let err = backend(dir.path(), script).err().unwrap();
    assert_eq!(err, BackendError::NotInstalled("no model".to_owned()));
}

/// Like `backend`, with timeouts short enough to wait for.
fn impatient(
    dir: &Path,
    script: &str,
) -> Result<IsolatedBackend<impl Fn() -> Command + use<>>, BackendError> {
    let script = script.to_owned();
    IsolatedBackend::start_with(
        &options(dir),
        move || {
            let mut command = Command::new("sh");
            command.arg("-c").arg(&script);
            command
        },
        Duration::from_secs(5),
        Duration::from_millis(300),
    )
}

#[test]
fn a_worker_that_hangs_is_killed_and_reported() {
    let dir = tempfile::tempdir().unwrap();
    let backend = impatient(dir.path(), &format!("{READY}read -r _; exec sleep 30")).unwrap();
    let err = backend
        .generate(&prompt(), &GenerationConfig::default())
        .unwrap_err();
    let BackendError::Generation(text) = err else {
        panic!("{err:?}")
    };
    assert!(text.contains("did not answer"), "{text}");
}

#[test]
fn a_worker_that_hangs_while_loading_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let err = IsolatedBackend::start_with(
        &options(dir.path()),
        || {
            let mut command = Command::new("sh");
            command.arg("-c").arg("read -r _; exec sleep 30");
            command
        },
        Duration::from_millis(300),
        Duration::from_millis(300),
    )
    .err()
    .unwrap();
    assert!(matches!(err, BackendError::Generation(t) if t.contains("did not answer")));
}

#[test]
fn a_tag_glued_to_unterminated_output_is_still_read() {
    let dir = tempfile::tempdir().unwrap();
    let script =
        r#"read -r _; printf 'loading...'; echo '@pionus {"Ready":{"description":"fake"}}'"#;
    let backend = backend(dir.path(), script).unwrap();
    assert_eq!(backend.describe(), "fake");
}

#[test]
fn a_replaced_binary_is_found_at_its_path() {
    assert_eq!(
        without_deleted(PathBuf::from("/usr/bin/pionus (deleted)")),
        PathBuf::from("/usr/bin/pionus")
    );
    assert_eq!(
        without_deleted(PathBuf::from("/usr/bin/pionus")),
        PathBuf::from("/usr/bin/pionus")
    );
}
