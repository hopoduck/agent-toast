use std::fs;

fn run_with_home(home: &std::path::Path, args: &[&str]) -> std::process::ExitStatus {
    let exe = env!("CARGO_BIN_EXE_agent-toast-send");
    std::process::Command::new(exe)
        .args(args)
        .env("HOME", home)
        .env("USERPROFILE", home) // Windows
        .env_remove("XDG_CONFIG_HOME")
        .status()
        .unwrap()
}

#[test]
fn init_creates_settings_with_stop_and_notification_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    let status = run_with_home(tmp.path(), &["init", "--url", "http://desktop:8787"]);
    assert!(status.success());

    let settings = tmp.path().join(".claude/settings.json");
    let content = fs::read_to_string(&settings).expect("settings.json should exist");
    let v: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(v["hooks"]["Stop"].is_array(), "Stop 훅이 배열이어야 함");
    assert!(v["hooks"]["Notification"].is_array());
    let stop_cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    assert!(stop_cmd.contains("agent-toast-send"));
    assert!(stop_cmd.contains("http://desktop:8787"));
    assert!(stop_cmd.contains("--event task_complete"));
}

#[test]
fn init_preserves_existing_non_agent_toast_hooks() {
    let tmp = tempfile::tempdir().unwrap();
    let settings_path = tmp.path().join(".claude/settings.json");
    fs::create_dir_all(settings_path.parent().unwrap()).unwrap();
    fs::write(
        &settings_path,
        r#"{
        "hooks": {
            "Stop": [{"hooks":[{"type":"command","command":"/usr/bin/my-custom-hook"}]}],
            "SessionStart": [{"hooks":[{"type":"command","command":"other-tool"}]}]
        }
    }"#,
    )
    .unwrap();

    let status = run_with_home(tmp.path(), &["init", "--url", "http://desktop:8787"]);
    assert!(status.success());

    let content = fs::read_to_string(&settings_path).unwrap();
    assert!(content.contains("my-custom-hook"), "기존 Stop 훅 보존");
    assert!(content.contains("other-tool"), "SessionStart 훅 보존");
    assert!(content.contains("agent-toast-send"), "agent-toast 훅 추가");
}

#[test]
fn uninstall_removes_only_agent_toast_entries() {
    let tmp = tempfile::tempdir().unwrap();
    run_with_home(tmp.path(), &["init", "--url", "http://desktop:8787"]);

    // Add a custom hook manually
    let settings_path = tmp.path().join(".claude/settings.json");
    let mut v: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).unwrap()).unwrap();
    v["hooks"]["Stop"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "hooks":[{"type":"command","command":"/usr/bin/custom"}]
        }));
    fs::write(&settings_path, serde_json::to_string(&v).unwrap()).unwrap();

    let status = run_with_home(tmp.path(), &["uninstall"]);
    assert!(status.success());

    let content = fs::read_to_string(&settings_path).unwrap();
    assert!(
        !content.contains("agent-toast-send"),
        "agent-toast 항목 제거되어야 함"
    );
    assert!(content.contains("/usr/bin/custom"), "커스텀 훅 보존");
}
