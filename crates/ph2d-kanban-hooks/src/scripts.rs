//! Os scripts de Kanban carregados (um [`KanbanScripts`] por fio) e os dois canais com o editor de
//! scripts: os pedidos de correr ([`request_run`] → [`take_runs`], servidos por quem tem os
//! históricos) e o que eles disseram ([`take_lines`], para o console do script).
//!
//! ⭐ Um script É de Kanban quando o código usa `ph2d.kanban` (fora de comentários e de strings —
//! [`is_kanban_script`]): corre então na VM do Kanban, sobre o projecto. Na cena, a mesma tabela
//! responde com um erro legível (`ph2d_script::kanban_stub`). Sem cabeçalho a declarar o alvo: o
//! código já o diz, e um cabeçalho que mente seria um segundo sítio para a mesma verdade.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ph2d_kanban_model::{History, KanbanId, KanbanSet};
use ph2d_kanban_script::{Ctx, KanbanScripts, Report};

use crate::Who;

/// Um pedido de correr o script `key` (o id do projecto) com o texto salvo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunRequest {
    pub key: u64,
    pub name: String,
    pub source: String,
}

/// Uma linha para o console do script `key`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub key: u64,
    pub text: String,
    pub error: bool,
    /// A linha do erro neste script, quando o Luau a deu (`nome:linha: …` sai do texto).
    pub line: Option<u32>,
}

#[derive(Default)]
struct State {
    scripts: KanbanScripts,
    /// O nome com que cada script corre (o `chunk` das mensagens) → o id dele.
    keys: BTreeMap<String, u64>,
    runs: Vec<RunRequest>,
    lines: Vec<Line>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Os scripts carregados deste fio.
pub fn with_scripts<R>(f: impl FnOnce(&mut KanbanScripts) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut().scripts))
}

/// O editor pede para correr `key` (Ctrl+S / Run): quem tem os históricos serve-o ([`take_runs`]).
pub fn request_run(key: u64, name: &str, source: &str) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.runs.retain(|r| r.key != key);
        s.runs.push(RunRequest {
            key,
            name: name.to_owned(),
            source: source.to_owned(),
        });
    });
}

/// Os pedidos de correr por servir.
#[must_use]
pub fn take_runs() -> Vec<RunRequest> {
    STATE.with(|s| std::mem::take(&mut s.borrow_mut().runs))
}

/// Larga o script `key` e os handlers dele (deixou de usar `ph2d.kanban`, ou saiu do projecto).
pub fn unload(key: u64) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let names: Vec<String> = s
            .keys
            .iter()
            .filter(|(_, k)| **k == key)
            .map(|(n, _)| n.clone())
            .collect();
        for n in names {
            s.scripts.unload(&n);
            s.keys.remove(&n);
        }
    });
}

/// O que os scripts disseram desde a última leitura.
#[must_use]
pub fn take_lines() -> Vec<Line> {
    STATE.with(|s| std::mem::take(&mut s.borrow_mut().lines))
}

/// ⭐ Serve um pedido: corre o script sobre o projecto (um passo de desfazer) e deixa os handlers
/// `ph2d.kanban.on` dele activos para os gestos seguintes.
pub fn run(
    req: &RunRequest,
    set: &mut KanbanSet,
    histories: &mut BTreeMap<KanbanId, History>,
    who: &Who<'_>,
    at: u64,
) -> Report {
    unload(req.key);
    let cx = Ctx {
        now: who.now,
        zone: who.zone,
        me: who.me.clone(),
        at,
    };
    let report = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.keys.insert(req.name.clone(), req.key);
        s.scripts.run(&req.name, &req.source, set, histories, &cx)
    });
    log(&report);
    report
}

pub(crate) fn log(r: &Report) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let key = |s: &State, n: &str| s.keys.get(n).copied();
        let mut out = Vec::new();
        for (name, text) in &r.printed {
            if let Some(k) = key(&s, name) {
                out.push(Line {
                    key: k,
                    text: text.clone(),
                    error: false,
                    line: None,
                });
            }
        }
        for e in &r.errors {
            if let Some(k) = key(&s, &e.script) {
                let (line, text) = split_line(&e.script, &e.message);
                out.push(Line {
                    key: k,
                    text,
                    error: true,
                    line,
                });
            }
        }
        s.lines.extend(out);
    });
}

/// `nome:12: msg` → `(Some(12), "msg")`; sem o prefixo, a mensagem inteira.
fn split_line(script: &str, message: &str) -> (Option<u32>, String) {
    let pat = format!("{script}:");
    let parsed = message
        .find(&pat)
        .map(|i| &message[i + pat.len()..])
        .and_then(|r| {
            let (n, rest) = r.split_once(':')?;
            Some((n.trim().parse().ok()?, rest.trim_start().to_owned()))
        });
    match parsed {
        Some((n, rest)) => (Some(n), rest),
        None => (None, message.to_owned()),
    }
}

/// O código usa `ph2d.kanban` (comentários e strings não contam).
#[must_use]
pub fn is_kanban_script(src: &str) -> bool {
    code_only(src).contains("ph2d.kanban")
}

/// O texto sem comentários (`--`, `--[[ ]]`, `--[==[ ]==]`) nem strings (`'…'`, `"…"`, `[[…]]`).
fn code_only(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < b.len() {
        let rest = &src[i..];
        if rest.starts_with("--") {
            i += 2;
            if let Some(n) = long_open(&src[i..]) {
                i = long_close(src, i + n.0, n.1);
            } else {
                i = src[i..].find('\n').map_or(b.len(), |p| i + p);
            }
            out.push(' ');
        } else if let Some(n) = long_open(rest) {
            i = long_close(src, i + n.0, n.1);
            out.push(' ');
        } else if b[i] == b'"' || b[i] == b'\'' {
            let q = b[i];
            i += 1;
            while i < b.len() && b[i] != q && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
            out.push(' ');
        } else {
            let c = rest.chars().next().unwrap_or(' ');
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// `[[` ou `[==[` no início: (bytes da abertura, número de `=`).
fn long_open(s: &str) -> Option<(usize, usize)> {
    let r = s.strip_prefix('[')?;
    let eq = r.bytes().take_while(|c| *c == b'=').count();
    r[eq..].starts_with('[').then_some((eq + 2, eq))
}

fn long_close(src: &str, from: usize, eq: usize) -> usize {
    let close = format!("]{}]", "=".repeat(eq));
    src.get(from..)
        .and_then(|r| r.find(&close))
        .map_or(src.len(), |p| from + p + close.len())
}
