//! C1 spike — Luau hello world foundation.
//!
//! Wraps `mlua` (feature `luau`) into a minimal `ScriptRuntime` capable of
//! loading a Luau source, calling top-level globals, and surviving a fixed
//! number of frames without panic. Surface intentionally small; will grow
//! during spike S2 (hot reload) and S3 (LLM-centric APIs).

use mlua::Lua;

pub struct ScriptRuntime {
    lua: Lua,
}

impl ScriptRuntime {
    pub fn new() -> mlua::Result<Self> {
        Ok(Self { lua: Lua::new() })
    }

    pub fn eval(&self, source: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()
    }

    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}
