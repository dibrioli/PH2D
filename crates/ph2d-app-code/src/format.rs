//! ⭐ **I13 — formatar ao salvar** com o StyLua fixado (decisão 6 do dono; `ph2d-code-format`).
//!
//! O StyLua não cabe num quadro acima de ~800 linhas (I0d), então a lei é UMA para todo tamanho: o
//! Ctrl+S aplica o texto e dá o selo no MESMO quadro; o StyLua corre fora do fio da UI; a resposta
//! entra como UM passo de desfazer **só se o texto não mudou desde o pedido** (a revisão do buffer), e
//! então a aba salva-se outra vez (a cena recarrega, o projecto com ficheiro grava-se) — «salvar =
//! guardado». As edições são as mínimas (linhas, depois caracteres): o cursor, as dobras e as marcas
//! seguem-nas como a qualquer outra edição. Erro de sintaxe ⇒ fica o texto (a recusa do Luau já o
//! diz no selo); o `--verify` que recusa ⇒ uma nota no console.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Change, ChangeSet, EditKind};
use ph2d_code_format::{Done, FormatFn, Outcome, Program, Refusal, Worker};
use ph2d_editor_core::Toast;
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::Tone;
use ph2d_script::ScriptId;
use std::path::PathBuf;
use std::sync::Arc;

/// Que formatador a família usa.
#[derive(Clone)]
pub enum FormatServer {
    /// O StyLua fixado (`ph2d_code_format::STYLUA`).
    Pinned,
    /// Este binário do StyLua, com as nossas regras.
    Binary(PathBuf),
    /// Qualquer função (os falsos dos testes).
    Custom(FormatFn),
    /// Sem formatar.
    Off,
}

pub(crate) struct Format {
    server: FormatServer,
    /// Resolvido no 1.º salvar; `Err` = não há programa (o aviso sai uma vez).
    program: Option<Result<Arc<Program>, String>>,
    worker: Worker<ScriptId>,
    /// O interruptor do comando «Format on Save» (por sessão).
    pub on: bool,
    ticket: u64,
}

impl Default for Format {
    fn default() -> Self {
        Self::new(FormatServer::Pinned)
    }
}

/// O pedido de um documento à espera da resposta: o bilhete e a revisão do texto enviado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Pending {
    pub ticket: u64,
    pub revision: u64,
}

impl Format {
    pub fn new(server: FormatServer) -> Self {
        Self {
            on: !matches!(server, FormatServer::Off),
            server,
            program: None,
            worker: Worker::default(),
            ticket: 0,
        }
    }

    /// Pede a formatação do texto SALVO de `d`. Devolve o aviso do ecrã quando não há formatador
    /// (só da 1.ª vez).
    pub fn ask(&mut self, d: &mut CodeDoc) -> Option<Toast> {
        if !self.on {
            return None;
        }
        let first = self.program.is_none();
        let program = self.program.get_or_insert_with(|| match &self.server {
            FormatServer::Pinned => Program::pinned().map(Arc::new),
            FormatServer::Binary(bin) if !bin.is_file() => {
                Err(format!("{} does not exist", bin.display()))
            }
            FormatServer::Binary(bin) => Program::stylua(bin.clone()).map(Arc::new),
            FormatServer::Custom(f) => Ok(Arc::new(Program::Custom(Arc::clone(f)))),
            FormatServer::Off => Err(String::new()),
        });
        match program {
            Ok(p) => {
                self.ticket += 1;
                d.formatting = Some(Pending {
                    ticket: self.ticket,
                    revision: d.buffer.revision(),
                });
                self.worker
                    .submit(p, d.script, self.ticket, d.buffer.text().to_string());
                None
            }
            Err(e) if first => Some(Toast::warning(tr_with(
                "code.format.missing",
                &[("err", e)],
            ))),
            Err(_) => None,
        }
    }

    pub fn arrived(&mut self) -> Vec<Done<ScriptId>> {
        self.worker.poll()
    }

    /// Espera as respostas em voo (a sonda dos testes); o quadro seguinte aplica-as.
    pub fn wait(&mut self, timeout: std::time::Duration) {
        self.worker.wait(timeout);
    }

    pub fn in_flight(&self) -> usize {
        self.worker.in_flight()
    }
}

/// O que uma resposta fez à aba.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Landed {
    /// O texto mudou: um passo de desfazer, e a aba salva-se outra vez.
    Formatted,
    /// Nada a fazer (já formatado, outra resposta, o texto andou, erro de sintaxe).
    Nothing,
    /// Uma nota para o console do script.
    Note(String),
}

impl CodeDoc {
    /// Aplica a resposta `ticket`, se ainda é a do texto que está na aba.
    pub(crate) fn format_landed(&mut self, ticket: u64, outcome: Outcome) -> Landed {
        let Some(p) = self.formatting.filter(|p| p.ticket == ticket) else {
            return Landed::Nothing;
        };
        self.formatting = None;
        if p.revision != self.buffer.revision() || self.preedit.is_some() {
            return Landed::Nothing;
        }
        match outcome {
            Outcome::Changes(e) if e.is_empty() => Landed::Nothing,
            Outcome::Changes(e) => {
                let changes = e
                    .into_iter()
                    .map(|e| Change {
                        range: e.range,
                        insert: e.insert,
                    })
                    .collect();
                let Some(cs) = ChangeSet::new(changes) else {
                    return Landed::Nothing;
                };
                let edits = self.buffer.apply(cs, None, EditKind::Other);
                self.edited(&edits, true);
                Landed::Formatted
            }
            Outcome::Refused(Refusal::Syntax(_)) => Landed::Nothing,
            Outcome::Refused(Refusal::WouldChangeCode) => {
                Landed::Note(tr("code.format.would_change").to_owned())
            }
            Outcome::Refused(Refusal::Failed(m)) => {
                Landed::Note(tr_with("code.format.failed", &[("err", &m)]))
            }
        }
    }
}

impl crate::docs::CodeDocs {
    /// Que formatador a família usa (por omissão o StyLua fixado).
    pub fn set_format_server(&mut self, server: FormatServer) {
        self.format = Format::new(server);
    }

    /// As respostas do formatador que chegaram: as que entram marcam a aba para salvar outra vez.
    pub(crate) fn format_arrivals(&mut self) {
        for d in self.format.arrived() {
            let Some(doc) = self.docs.iter_mut().find(|x| x.script == d.key) else {
                continue;
            };
            match doc.format_landed(d.ticket, d.outcome) {
                Landed::Formatted => doc.resave = true,
                Landed::Nothing => {}
                Landed::Note(text) => self
                    .consoles
                    .entry(d.key)
                    .or_default()
                    .note(text, Tone::Warning),
            }
        }
    }
}
