//! C1 spike — Luau hello world runner.
//!
//! Loads `hello.luau`, runs its top-level once, then invokes the global
//! `update(frame)` 60 times. Pass criterion: 0 panics, 0 mlua errors.

use mlua::Function;
use ph2d_script::ScriptRuntime;

const SCRIPT: &str = include_str!("../../hello.luau");
const FRAMES: u32 = 60;

fn main() -> mlua::Result<()> {
    let runtime = ScriptRuntime::new()?;
    runtime.eval(SCRIPT)?;

    let update: Function = runtime.lua().globals().get("update")?;
    for frame in 0..FRAMES {
        update.call::<()>(frame)?;
    }

    println!("C1 PASS — {FRAMES} frames executed without panic");
    Ok(())
}
