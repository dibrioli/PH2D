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

/// **Bytes de TEXTO guardados por ler** (todas as linhas). ⚠️ **De que recurso é:** da MEMÓRIA,
/// como o de cima — que só conta linhas e foi medido com textos de 8 B: `print(string.rep("x",
/// 100000))` em cada passo encheria as 65 536 linhas com 6,5 GB. Este tecto é o do console cheio de
/// linhas curtas medido acima (~10,5 MB) arredondado para cima: o `print` normal continua limitado
/// pelas LINHAS, e o longo nunca passa daqui. A tabela: `docs/CodeEditor/02_onda1.md` §Medições do
/// fecho.
pub const CONSOLE_BYTES_MAX: usize = 16 << 20;

/// **Bytes de texto de UMA linha**; o resto corta-se (por fronteira de carácter) e a linha acaba
/// em `…`. ⚠️ **De que recurso é:** do QUADRO — o console pinta cada linha à vista inteira (o
/// `paint_text` molda o texto todo para o elidir). Medido (`docs/CodeEditor/02_onda1.md` §Medições
/// do fecho, A): o console aberto com todas as linhas no tecto custa 0.26 ms; a 16 KiB 0.59, a 64 KiB
/// 1.5 ms.
pub const CONSOLE_LINE_BYTES_MAX: usize = 4 << 10;

/// O texto de uma linha cortado ao tecto [`CONSOLE_LINE_BYTES_MAX`] (com `…` no fim).
#[must_use]
pub fn clip_console_text(mut text: String) -> String {
    if text.len() > CONSOLE_LINE_BYTES_MAX {
        let mut cut = CONSOLE_LINE_BYTES_MAX;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
        text.push('…');
    }
    text
}

/// **De que espécie é uma linha.**
#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleKind {
    /// `print(...)` do script.
    Print,
    /// Um gancho falhou a correr — o objecto parou.
    RuntimeError(ScriptDiagnostic),
    /// O texto novo não compilou — a versão anterior (se havia) continua.
    ReloadRefused(ScriptDiagnostic),
    /// Uma função da API recusou um pedido e disse porquê — o objecto CONTINUA a correr.
    Warning(ScriptDiagnostic),
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
    /// Os bytes de texto em `lines` (o tecto [`CONSOLE_BYTES_MAX`]).
    bytes: usize,
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

    /// Empurra uma linha (cortada ao tecto por linha); cheio, em linhas ou em bytes ⇒ as mais
    /// antigas saem.
    pub(crate) fn push(&self, mut line: ConsoleLine) {
        line.text = clip_console_text(line.text);
        let mut g = self.lock();
        while !g.lines.is_empty()
            && (g.lines.len() >= CONSOLE_LINES_MAX || g.bytes + line.text.len() > CONSOLE_BYTES_MAX)
        {
            if let Some(old) = g.lines.pop_front() {
                g.bytes -= old.text.len();
            }
            g.dropped += 1;
        }
        g.bytes += line.text.len();
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

    /// ⭐ Uma recusa em voz de uma função da API, na linha do script que a chamou (o primeiro nível
    /// da pilha que é Luau e não o `=ph2d` das funções da casa).
    pub(crate) fn warning(&self, lua: &Lua, msg: &str) {
        let line = (1..32).find_map(|level| {
            lua.inspect_stack(level, |d| {
                let src = d.source();
                let ours = src.source.as_deref() == Some("=ph2d") || src.what == "C";
                (!ours).then(|| d.current_line())
            })?
        });
        let d = ScriptDiagnostic {
            line: line.flatten().and_then(|n| u32::try_from(n).ok()),
            ..ScriptDiagnostic::runtime(msg)
        };
        let (script, entity) = self.lock().speaker;
        self.push(ConsoleLine {
            script,
            entity,
            text: d.to_string(),
            kind: ConsoleKind::Warning(d),
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
        let mut g = self.lock();
        g.bytes = 0;
        g.lines.drain(..).collect()
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
            // Passado o tecto por linha o resto nem se copia (o `push` corta e põe o `…`).
            if text.len() > CONSOLE_LINE_BYTES_MAX {
                break;
            }
            if i > 0 {
                text.push('\t');
            }
            // Um `integer` de 64 bits voltaria ao `tostring` como `number` e arredondaria acima de 2^53
            // (`print(integer.maxsigned)`, medido 20/10): os dígitos dele são o que o `tostring` dá.
            let room = |text: &String, n: usize| (CONSOLE_LINE_BYTES_MAX + 4 - text.len()).min(n);
            if let mlua::Value::Integer(i) = v {
                let digits = i.to_string();
                let n = room(&text, digits.len());
                text.push_str(&digits[..n]);
                continue;
            }
            let s: mlua::LuaString = tostring.call(v)?;
            let bytes = s.as_bytes();
            let n = room(&text, bytes.len());
            text.push_str(&String::from_utf8_lossy(&bytes[..n]));
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
