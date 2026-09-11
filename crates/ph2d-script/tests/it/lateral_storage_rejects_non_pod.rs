//! HR-16 conformance test: the lateral state setter rejects
//! `function`, `thread`, `userdata` and `lightuserdata` Lua values
//! with a runtime error — not a panic, not silent corruption.
//!
//! This is the gate that keeps save/replay/hot-reload working: only
//! POD values cross the boundary, so the snapshot bytes are always
//! serializable.

use ph2d_script::ScriptHost;

#[test]
fn function_value_is_rejected() {
    let host = ScriptHost::new().unwrap();
    let err = host
        .runtime()
        .eval(
            r#"
            local function f() return 1 end
            ph2d.state_set(1, "bad", f)
        "#,
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("HR-16") && msg.contains("function"),
        "expected HR-16 function rejection, got: {msg}"
    );
}

#[test]
fn coroutine_value_is_rejected() {
    let host = ScriptHost::new().unwrap();
    let err = host
        .runtime()
        .eval(
            r#"
            local co = coroutine.create(function() end)
            ph2d.state_set(1, "bad", co)
        "#,
        )
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("HR-16") && msg.contains("thread"),
        "expected HR-16 thread rejection, got: {msg}"
    );
}

#[test]
fn nested_table_is_accepted_within_depth_limit() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval(r#"ph2d.state_set(1, "nested", { x = 1, y = 2 })"#)
        .unwrap();
    // Round-trip the table back to Lua and confirm the value.
    let got: f64 = host
        .runtime()
        .lua()
        .load(r#"return ph2d.state_get(1, "nested").x"#)
        .eval()
        .unwrap();
    assert_eq!(got, 1.0);
}

#[test]
fn pod_scalars_round_trip() {
    let host = ScriptHost::new().unwrap();
    host.runtime()
        .eval(
            r#"
            ph2d.state_set(1, "n", 42.5)
            ph2d.state_set(1, "s", "hello")
            ph2d.state_set(1, "b", true)
            ph2d.state_set(1, "nil_val", nil)
        "#,
        )
        .unwrap();
    let n: f64 = host
        .runtime()
        .lua()
        .load(r#"return ph2d.state_get(1, "n")"#)
        .eval()
        .unwrap();
    assert_eq!(n, 42.5);
    let s: String = host
        .runtime()
        .lua()
        .load(r#"return ph2d.state_get(1, "s")"#)
        .eval()
        .unwrap();
    assert_eq!(s, "hello");
    let b: bool = host
        .runtime()
        .lua()
        .load(r#"return ph2d.state_get(1, "b")"#)
        .eval()
        .unwrap();
    assert!(b);
}
