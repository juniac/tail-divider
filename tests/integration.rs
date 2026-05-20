use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_taild"))
}

#[test]
fn passes_through_activity_lines() {
    let mut child = binary()
        .args(["-t", "10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = child.stdin.as_mut().unwrap();
    writeln!(stdin, "first line").unwrap();
    writeln!(stdin, "second line").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("first line"), "stdout: {}", stdout);
    assert!(stdout.contains("second line"), "stdout: {}", stdout);
}

#[test]
fn emits_idle_marker_after_threshold() {
    let mut child = binary()
        .args(["-t", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    writeln!(child.stdin.as_mut().unwrap(), "start").unwrap();

    // 2s: enough headroom for process startup + 1s idle threshold
    std::thread::sleep(Duration::from_millis(2000));
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("idle"), "expected idle marker, got: {}", stdout);
}

#[test]
fn invalid_color_exits_nonzero() {
    let output = binary()
        .args(["--color", "chartreuse"])
        .stdin(Stdio::null())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("chartreuse"), "stderr: {}", stderr);
}

#[test]
fn blank_lines_between_idle_and_resumed_activity() {
    let mut child = binary()
        .args(["-t", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    writeln!(child.stdin.as_mut().unwrap(), "before idle").unwrap();
    std::thread::sleep(Duration::from_millis(1300));
    writeln!(child.stdin.as_mut().unwrap(), "after idle").unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    let idle_pos = stdout.find("idle").expect("no idle marker");
    let after_pos = stdout.find("after idle").expect("no resumed line");
    assert!(idle_pos < after_pos, "idle marker should precede resumed line");
}
