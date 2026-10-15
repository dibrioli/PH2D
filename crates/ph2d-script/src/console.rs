//! ⭐ **O CONSOLE dos scripts** (CodeEditor Onda 1, E10): o `print` de um script, os erros de
//! runtime dos ganchos e as recusas de recarga vão para uma fila drenável — nunca para o stdout.
//!
//! Quem fala é o [`Console::as_speaker`] em curso: o executor da cena marca o script e o objecto
//! antes de cada chamada, e o `print` lê essa marca. Fora de qualquer chamada da cena (o `eval` do
//! `ScriptHost::load_script`) a marca é [`ScriptId::NONE`] sem objecto.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use mlua::{Function, Lua, MultiValue};
use ph2d_ecs::Entity;
use ph2d_script_doc::ScriptId;

use crate::diagnostic::ScriptDiagnostic;

/// **Linhas guardadas para o painel enquanto ninguém as drena.**
///
/// ⚠️ **De que recurso é:** da MEMÓRIA retida por linhas por ler. A cena pior que o plano nomeia
/// (1 000 objectos a fazer `print` em cada passo de 60 Hz) produz 60 000 linhas/s; este tecto é
/// ~1 s dela, ⇒ um painel que drena pelo menos uma vez por segundo nunca perde uma linha. Medido
/// (`tests/it/a_recarga_e_o_console.rs::mede_o_tecto_do_console`, 15/10): 60 passos × 1 000
/// objectos = 60 000 linhas; `size_of::<ConsoleLine>()` = 152 B + 8 B de texto ⇒ ~160 B/linha ⇒
/// ~10,5 MB cheio. A mais antiga sai primeiro e conta em [`Console::dropped`].
pub const CONSOLE_LINES_MAX: usize = 65_536;

/// **De que espécie é uma linha.**
#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleKind {
    /// `print(...)` do script.
    Print,
    /// Um gancho falhou a correr — o objecto parou.
    RuntimeError(ScriptDiagnostic),
    /// O texto novo não compilou — a versão anterior (se havia) continua.
    ReloadRefused(ScriptDiagnostic),
}

/// ⭐ **Uma linha do console.**
#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleLine {
    /// O script que falou ([`ScriptId::NONE`] fora da cena).
    pub script: ScriptId,
    /// O objecto em cujo gancho falou — `None` no topo do script.
    pub entity: Option<Entity>,
    /// Os argumentos do `print` pelo `tostring` do Luau, separados por tab; ou o diagnóstico.
    pub text: String,
    pub kind: ConsoleKind,
}

#[derive(Default)]
struct Inner {
    lines: VecDeque<ConsoleLine>,
    dropped: u64,
    speaker: (ScriptId, Option<Entity>),
}

/// A fila partilhada (sobrevive à reconstrução da VM, como as outras filas do host).
#[derive(Clone, Default)]
pub struct Console(Arc<Mutex<Inner>>);

impl Console {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.0.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Empurra uma linha; cheio ⇒ a mais antiga sai.
    pub(crate) fn push(&self, line: ConsoleLine) {
        let mut g = self.lock();
        if g.lines.len() >= CONSOLE_LINES_MAX {
            g.lines.pop_front();
            g.dropped += 1;
        }
        g.lines.push_back(line);
    }

    /// Um diagnóstico de `script`/`entity` como linha.
    pub(crate) fn diagnostic(
        &self,
        script: ScriptId,
        entity: Option<Entity>,
        d: &ScriptDiagnostic,
        refused: bool,
    ) {
        let kind = if refused {
            ConsoleKind::ReloadRefused(d.clone())
        } else {
            ConsoleKind::RuntimeError(d.clone())
        };
        self.push(ConsoleLine {
            script,
            entity,
            text: d.to_string(),
            kind,
        });
    }

    /// Corre `f` com `script`/`entity` como quem fala. ⚠️ O cadeado NÃO fica preso durante `f`.
    pub(crate) fn as_speaker<T>(
        &self,
        script: ScriptId,
        entity: Option<Entity>,
        f: impl FnOnce() -> T,
    ) -> T {
        self.lock().speaker = (script, entity);
        let out = f();
        self.lock().speaker = (ScriptId::NONE, None);
        out
    }

    /// Tira todas as linhas por ler, da mais antiga à mais nova.
    pub fn drain(&self) -> Vec<ConsoleLine> {
        self.lock().lines.drain(..).collect()
    }

    /// Quantas linhas o tecto já deitou fora (desde o arranque).
    #[must_use]
    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }
}

/// O console registado na VM (`None` numa VM que não é do `ScriptHost`).
pub(crate) fn of(lua: &Lua) -> Option<Console> {
    lua.app_data_ref::<Console>().map(|c| c.clone())
}

/// Corre `f` com `script`/`entity` como quem fala, se a VM tem console.
pub(crate) fn speaking<T>(
    lua: &Lua,
    script: ScriptId,
    entity: Option<Entity>,
    f: impl FnOnce() -> T,
) -> T {
    match of(lua) {
        Some(c) => c.as_speaker(script, entity, f),
        None => f(),
    }
}

/// ⭐ **Troca o `print` global** — instalada antes da sandbox. Formata como o do Luau: o
/// `tostring` de cada argumento (o ORIGINAL, apanhado aqui), separados por tab.
pub(crate) fn wire_print(lua: &Lua, console: &Console) -> mlua::Result<()> {
    lua.set_app_data(console.clone());
    let tostring: Function = lua.globals().get("tostring")?;
    let print = lua.create_function(move |lua, args: MultiValue| -> mlua::Result<()> {
        let mut text = String::new();
        for (i, v) in args.into_iter().enumerate() {
            if i > 0 {
                text.push('\t');
            }
            let s: mlua::LuaString = tostring.call(v)?;
            text.push_str(&s.to_string_lossy());
        }
        if let Some(c) = of(lua) {
            let (script, entity) = c.lock().speaker;
            c.push(ConsoleLine {
                script,
                entity,
                text,
                kind: ConsoleKind::Print,
            });
        }
        Ok(())
    })?;
    lua.globals().set("print", print)
}
