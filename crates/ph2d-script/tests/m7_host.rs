//! M7 visual gate: Luau script controls 100 entities via `ph2d.set`,
//! `Lua::sandbox(true)` blocks unsafe globals, hot reload via
//! reset+restore actually rebuilds the VM (HR-8/9/16).

use ph2d_script::{EntityWrite, ScriptHost};

#[test]
fn script_writes_one_hundred_entities_in_one_tick() {
    let mut host = ScriptHost::new().unwrap();
    host.load_script(
        r#"
        for i = 0, 99 do
            ph2d.set(i, "x", i * 0.1)
            ph2d.set(i, "y", i * 0.2)
        end
        "#,
    )
    .unwrap();

    let writes = host.drain_writes();
    assert_eq!(writes.len(), 200, "100 entities × 2 fields = 200 writes");

    // Spot-check a few writes for correctness.
    let entity_0_x = writes.iter().find(|w| w.entity == 0 && w.field == "x");
    assert_eq!(
        entity_0_x,
        Some(&EntityWrite {
            entity: 0,
            field: "x".into(),
            value: 0.0
        })
    );

    let entity_50_y = writes.iter().find(|w| w.entity == 50 && w.field == "y");
    assert_eq!(
        entity_50_y,
        Some(&EntityWrite {
            entity: 50,
            field: "y".into(),
            value: 10.0
        })
    );
}

#[test]
fn ph2d_get_resolves_from_read_snapshot() {
    let mut host = ScriptHost::new().unwrap();
    host.provide_read(42, "hp", 75.0);

    host.load_script(
        r#"
        local hp = ph2d.get(42, "hp")
        if hp ~= nil then
            ph2d.set(42, "hp", hp - 10)
        end
        "#,
    )
    .unwrap();

    let writes = host.drain_writes();
    assert_eq!(
        writes,
        vec![EntityWrite {
            entity: 42,
            field: "hp".into(),
            value: 65.0
        }]
    );
}

#[test]
fn ph2d_get_returns_nil_for_unknown_field() {
    let mut host = ScriptHost::new().unwrap();
    host.load_script(
        r#"
        local v = ph2d.get(999, "missing")
        if v == nil then
            ph2d.set(0, "missing_was_nil", 1)
        end
        "#,
    )
    .unwrap();

    let writes = host.drain_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].field, "missing_was_nil");
}

#[test]
fn sandbox_blocks_io_open() {
    let mut host = ScriptHost::new().unwrap();
    // `io` is not a Luau-baseline global; sandbox(true) doesn't
    // re-introduce it either. Calling `io.open` should fail with
    // "attempt to index nil with 'open'" (io itself is nil).
    let err = host
        .load_script(r#" return io.open("/etc/passwd", "r") "#)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("nil") && msg.contains("open"),
        "sandbox should reject io.open — got: {msg}"
    );
}

#[test]
fn sandbox_blocks_os_execute() {
    let mut host = ScriptHost::new().unwrap();
    // Similar story: `os.execute` is not exposed. Either `os` is nil
    // or `os.execute` is nil — both surface as "attempt to call/index
    // a nil value".
    let err = host
        .load_script(r#" return os.execute("rm -rf /") "#)
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("nil"),
        "sandbox should reject os.execute — got: {msg}"
    );
}

#[test]
fn hot_reload_resets_vm_only_when_source_changes() {
    let mut host = ScriptHost::new().unwrap();
    let src_v1 = r#" ph2d.set(1, "v", 100) "#;
    let src_v2 = r#" ph2d.set(1, "v", 200) "#;

    let changed = host.load_script(src_v1).unwrap();
    assert!(changed, "first load is always a reset");
    assert_eq!(host.reset_count(), 1);
    let writes = host.drain_writes();
    assert_eq!(writes[0].value, 100.0);

    // Re-load identical source → no reset.
    let changed = host.load_script(src_v1).unwrap();
    assert!(!changed, "identical source must not rebuild VM");
    assert_eq!(host.reset_count(), 1);
    // …and no new writes (script didn't re-run).
    assert!(host.drain_writes().is_empty());

    // Different source → reset + re-execute.
    let changed = host.load_script(src_v2).unwrap();
    assert!(changed, "changed source must rebuild VM");
    assert_eq!(host.reset_count(), 2);
    let writes = host.drain_writes();
    assert_eq!(writes[0].value, 200.0);
}

#[test]
fn reset_clears_old_globals() {
    let mut host = ScriptHost::new().unwrap();
    host.load_script(r#" my_global_thing = 42 "#).unwrap();

    // After reset, `my_global_thing` should not exist (proves the VM
    // is genuinely fresh, not just re-executed in the same VM).
    host.load_script(
        r#"
        if my_global_thing == nil then
            ph2d.set(0, "old_global_gone", 1)
        end
        "#,
    )
    .unwrap();

    let writes = host.drain_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].field, "old_global_gone");
}

#[test]
fn read_snapshot_clear_removes_stale_entries() {
    let mut host = ScriptHost::new().unwrap();
    host.provide_read(0, "x", 1.0);
    host.clear_reads();
    host.load_script(
        r#"
        local v = ph2d.get(0, "x")
        if v == nil then ph2d.set(0, "cleared", 1) end
        "#,
    )
    .unwrap();
    let writes = host.drain_writes();
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].field, "cleared");
}
