#![forbid(unsafe_code)]
//! ph2d-script — Luau scripting runtime + the scene's scripts + messaging bus.
//!
//! Spike (S1-S3) skeleton: minimal `ScriptRuntime`, Defold-style `MessageBus`. ✓ The
//! `ph2d.wait` waits are the scene's (`scene.rs`, CodeEditor item 6).
//!
//! M7 layered the canonical engine ↔ Luau bridge on top:
//! - [`ScriptHost`] — public facade with reset+restore hot reload via
//!   blake3 source-content hashing (HR-16). VM rebuild costs one
//!   constructor call; ECS state stays untouched (canonical store).
//! - [`WriteQueue`] / [`ReadSnapshot`] — the only legal data path
//!   between Luau callbacks and ECS, per HR-8 ("scripts are pure
//!   data at the FFI boundary"). Same pattern Defold/Roblox use.
//! - `ph2d.set(entity, field, value)` and `ph2d.get(entity, field)`
//!   wired into the Lua globals; both go through the queues above.
//! - `Lua::sandbox(true)` activated so user scripts can't reach
//!   `os.execute`, `io.open`, `package.loadlib`, etc. (HR-9).
//!
//! M8 added a third FFI lane: [`InputSnapshot`] + the
//! `ph2d.input(key)` Lua binding. Same Arc-shared, survives-reset
//! pattern; populated by shells from `ph2d_input::InputState` per
//! frame.

/// O texto da página de ajuda de cada linha da `api_table`.
pub mod api_help;
/// A API Luau como tabela — a fonte única do `ph2d.d.luau`.
pub mod api_table;
pub mod component;
/// O console dos scripts: `print`, erros de runtime, recusas de recarga (CodeEditor E10).
pub mod console;
/// O diagnóstico estruturado de um script (linha, mensagem inteira, pilha).
pub mod diagnostic;
pub mod host;
pub mod io;
/// A pista do mundo: o `&mut World` emprestado à VM durante os ganchos (CodeEditor item 6).
pub mod lane;
pub mod lateral;
pub mod messaging;
pub mod module;
/// Ciclo 19: a tabela `motion` (as entradas do grafo do Motion).
pub mod motion_api;
/// O load de um projeto e os scripts: o degrau v2 → v3 e a importação dos caminhos (ADR-0188).
pub mod project_scripts;
pub mod props;
pub mod registry;
pub mod scene;
/// As curas da biblioteca padrão do Luau: a família do inteiro e o `loadstring` de cada script.
mod stdlib;
mod valores;

pub use component::{LibraryEpoch, LuauScript, ProjectScripts, ScriptEditRequests, ScriptSource};
pub use console::{
    CONSOLE_BYTES_MAX, CONSOLE_LINE_BYTES_MAX, CONSOLE_LINES_MAX, Console, ConsoleKind,
    ConsoleLine, clip_console_text,
};
pub use diagnostic::{DiagnosticKind, ScriptDiagnostic};
pub use host::ScriptHost;
pub use io::{Devices, Edge, EntityWrite, InputSnapshot, QueueFull, WriteQueue};
pub use lateral::{PodKey, PodValue, StateTable};
pub use messaging::{EntityId, Handler, Message, MessageBus, MessageId};
pub use module::{ScriptModule, load_module};
pub use ph2d_script_doc::{DocError, ScriptDoc, ScriptId, ScriptLibrary};
pub use props::{
    DeclError, Origin, Orphan, OrphanWhy, PROPS_MAX, PropDecl, PropHint, Resolution, Resolved,
    ScriptValue, ScriptValueKind,
};
pub use registry::register_script_components;
pub use lane::{NoDoors, SceneDoors};
pub use scene::{HOOK_BUDGET, ReloadOutcome, SceneReport, SceneScripts, ScriptInfo, ScriptReload};

use mlua::Lua;

pub struct ScriptRuntime {
    lua: Lua,
}

/// ⭐ **`ph2d.wait`** — cede a marca da espera e os segundos; quem retoma é o
/// [`scene::SceneScripts`] (cada gancho corre numa corrotina). Fora de um gancho diz porquê, em vez
/// do «attempt to yield across a C-call boundary» do Luau.
const WAIT_LUAU: &str = r#"
local token = ...
ph2d = ph2d or {}
function ph2d.wait(seconds)
  if seconds == nil then seconds = 0 end
  if type(seconds) ~= "number" then
    error("ph2d.wait: seconds must be a number", 2)
  end
  if not coroutine.isyieldable() then
    error("ph2d.wait: only inside init, update or on_signal while the scene runs", 2)
  end
  return coroutine.yield(token, seconds)
end
"#;

impl ScriptRuntime {
    pub fn new() -> mlua::Result<Self> {
        let lua = Lua::new();
        let token = lua.create_table()?;
        lua.set_named_registry_value(scene::WAIT_TOKEN, token.clone())?;
        lua.load(WAIT_LUAU).set_name("=ph2d").call::<()>(token)?;
        Ok(Self { lua })
    }

    pub fn eval(&self, source: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()
    }

    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}
