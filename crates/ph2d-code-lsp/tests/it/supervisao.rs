//! Supervision: a dead server is an ERROR on the next call (never a hang or a silent no-op), and
//! `restart` brings it back with the open documents re-sent.

use std::time::Duration;

use ph2d_code_lsp::{LspError, Position, Solver};

use crate::support::{WAIT, ph2d_definitions, start, uri};

#[test]
fn a_killed_server_surfaces_as_an_error_and_restart_recovers() {
    let mut c = start(&ph2d_definitions(), Solver::Old);
    let u = uri("vivo.luau");
    c.did_open(&u, "local _x: number = \"a\"\n").expect("didOpen");
    c.wait_diagnostics(&u, WAIT).expect("first diagnostics");

    let pid = c.pid();
    let killed = std::process::Command::new("kill").args(["-9", &pid.to_string()]).status();
    assert!(killed.map(|s| s.success()).unwrap_or(false), "kill -9 {pid}");
    let t0 = std::time::Instant::now();
    while c.is_alive() && t0.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!c.is_alive(), "client still reports a killed server alive");
    match c.hover(&u, Position::new(0, 7)) {
        Err(LspError::ServerDied { .. }) => {}
        other => panic!("expected ServerDied, got {other:?}"),
    }

    c.restart().expect("restart");
    assert!(c.is_alive());
    let d = c.wait_diagnostics(&u, WAIT).expect("the re-opened document is checked again");
    assert_eq!(d.diagnostics.len(), 1, "{:?}", d.diagnostics);
    assert_eq!(c.text(&u), Some("local _x: number = \"a\"\n"));
    c.shutdown().expect("shutdown");
}
