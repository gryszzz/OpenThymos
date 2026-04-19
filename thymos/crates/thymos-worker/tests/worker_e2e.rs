use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::json;
use thymos_tools::{ToolWorkerRequest, ToolWorkerResponse};

#[test]
fn worker_executes_shell_request() {
    let worker = env!("CARGO_BIN_EXE_thymos-worker");
    let mut child = Command::new(worker)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn worker");

    let request = ToolWorkerRequest::Shell {
        command: "printf worker-ok".into(),
        cwd: None,
        timeout_secs: 5,
        purpose: Some("worker test".into()),
        capability_profile: "inspect".into(),
        restricted_env: true,
        env: Default::default(),
        max_output_bytes: 4096,
        blocked_patterns: vec![],
        wrapper: vec![],
        allowed_roots: vec![std::env::current_dir().expect("cwd").display().to_string()],
        isolate_home: true,
    };

    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(serde_json::to_vec(&request).expect("serialize").as_slice())
        .expect("write request");

    let output = child.wait_with_output().expect("wait worker");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response: ToolWorkerResponse =
        serde_json::from_slice(&output.stdout).expect("parse worker response");
    assert_eq!(response.kind, "shell");
    assert_eq!(response.output["stdout"], json!("worker-ok"));
    assert_eq!(response.output["exit_code"], json!(0));
    assert_eq!(
        response.output["receipt"]["capability_profile"],
        json!("inspect")
    );
}
