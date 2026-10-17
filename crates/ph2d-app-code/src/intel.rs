//! ⭐ **O assistente de código na família** (CodeEditor Onda 2, I2, `docs/CodeEditor/03_onda2.md`): a
//! sessão do `luau-lsp` ([`LspSession`]) e o ESPELHO de cada script do projeto nela.
//!
//! Lei: o servidor vê o texto que o editor MOSTRA — o da aba aberta, o da aba fechada com ●, senão o
//! da biblioteca — de TODOS os scripts do projeto (cada script é um ambiente, I0b §6). O quadro só
//! envia instantâneos (um `Arc` da rope) e drena; a tecla nunca espera o servidor. Arranca no 1.º
//! documento aberto. Sem binário, ou morto sem volta, o editor edita na mesma e a barra di-lo.

use crate::doc::CodeDoc;
use crate::docs::Parked;
use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_intel::lsp::Connect;
use ph2d_code_intel::{Answer, Diag, Dirty, Event, LspSession, Req, ServerState, uri_of};
use ph2d_code_lsp::{LspClient, ServerConfig, resolve_binary};
use ph2d_panel_code::Assist;
use ph2d_script::ScriptId;
use ph2d_script::api_table::{DEFINITIONS_PACKAGE, luau_definitions, luau_docs};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Que servidor a família liga.
pub enum IntelServer {
    /// O `luau-lsp` fixado (`resolve_binary`), com as definições e a documentação geradas da API.
    Pinned,
    /// Este binário, com as mesmas definições (`PH2D_LUAU_LSP` sem a variável de ambiente).
    Binary(PathBuf),
    /// Qualquer servidor (os falsos dos testes); cada chamada liga um.
    Custom(Connect),
    /// Sem assistente: a barra cala-se e nada se envia.
    Off,
}

/// O que o servidor tem de um script: o texto da última versão enviada e de onde veio.
struct Mirror {
    uri: String,
    version: u64,
    text: Text,
    src: Src,
}

/// De onde vem o texto de um script. Duas fontes iguais = o mesmo texto, sem olhar para ele.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Src {
    /// Um documento (aberto ou estacionado): o seu número e a revisão do `Buffer`.
    Doc { serial: u64, revision: u64 },
    /// A biblioteca, na revisão dela.
    Library(u64),
}

/// As edições que um documento fez desde a última vez que o espelho o leu (o `didChange`
/// incremental): `from` = a revisão do `Buffer` em que começam.
#[derive(Default)]
pub(crate) struct Pending {
    pub from: u64,
    pub edits: Vec<TextEdit>,
}

pub(crate) struct Intel {
    server: Option<IntelServer>,
    session: Option<LspSession>,
    /// `None` = desligado, ou ainda sem documento aberto.
    state: Option<ServerState>,
    /// O binário fixado não se encontrou (a razão crua: os caminhos tentados).
    missing: Option<String>,
    mirrors: BTreeMap<ScriptId, Mirror>,
    diags: BTreeMap<ScriptId, (u64, Vec<Diag>)>,
    /// O uso de cada nome nos scripts do projeto, na revisão da biblioteca em que se contou.
    usage: Option<(u64, BTreeMap<String, u32>)>,
}

impl Default for Intel {
    fn default() -> Self {
        Self::new(IntelServer::Pinned)
    }
}

impl Intel {
    pub fn new(server: IntelServer) -> Self {
        Self {
            server: Some(server),
            session: None,
            state: None,
            missing: None,
            mirrors: BTreeMap::new(),
            diags: BTreeMap::new(),
            usage: None,
        }
    }

    fn off(&self) -> bool {
        matches!(self.server, Some(IntelServer::Off))
    }

    /// O que a barra diz (`None` = pronto, desligado, ou ainda não arrancou).
    pub fn assist(&self) -> Option<Assist> {
        if self.missing.is_some() {
            return Some(Assist::Missing);
        }
        match self.state.as_ref()? {
            ServerState::Starting => Some(Assist::Starting),
            ServerState::Ready => None,
            ServerState::Restarting => Some(Assist::Restarting),
            ServerState::Failed(_) => Some(Assist::Stopped),
        }
    }

    /// A razão crua do «não está pronto» (os caminhos tentados, a saída do servidor).
    pub fn reason(&self) -> Option<&str> {
        self.missing.as_deref().or(match &self.state {
            Some(ServerState::Failed(r)) => Some(r.as_str()),
            _ => None,
        })
    }

    /// O «Restart Code Assistant»: procura o binário outra vez, ou pede à sessão que volte.
    pub fn retry(&mut self) {
        if self.missing.take().is_some() {
            self.state = None;
            return; // o próximo `sync` arranca de novo
        }
        if let Some(s) = &self.session {
            s.retry();
        }
    }

    /// Os diagnósticos do script `id` para o texto que o servidor TEM agora; os de uma versão velha
    /// nunca saem daqui (pintariam no sítio errado).
    pub fn diagnostics(&self, id: ScriptId) -> Option<&[Diag]> {
        let (v, list) = self.diags.get(&id)?;
        (self.mirrors.get(&id)?.version == *v).then_some(list.as_slice())
    }

    /// A versão do script `id` no servidor (`None` = não aberto).
    pub fn version(&self, id: ScriptId) -> Option<u64> {
        self.mirrors.get(&id).map(|m| m.version)
    }

    /// Cada aba segura a última resposta do servidor ao texto DELA (I3): adopta-a quando chega, sobre
    /// as edições que ainda não foram, e larga tudo se o servidor parou de vez.
    fn hand_over(&self, docs: &mut [CodeDoc]) {
        let stopped = matches!(self.state, Some(ServerState::Failed(_)));
        for d in docs.iter_mut() {
            if stopped {
                d.assist.clear();
                continue;
            }
            let Some(m) = self.mirrors.get(&d.script) else {
                continue;
            };
            let ours = Src::Doc {
                serial: d.serial,
                revision: d.lsp.from,
            };
            if let Some((v, list)) = self.diags.get(&d.script)
                && m.src == ours
                && m.version == *v
                && d.assist.version != Some(*v)
            {
                d.assist.adopt(*v, list, &d.lsp.edits);
            }
        }
    }

    fn start(&mut self) {
        let connect = match self.server.take() {
            Some(IntelServer::Pinned) => match resolve_binary() {
                Ok((bin, _)) => pinned(bin),
                Err(e) => {
                    self.missing = Some(e.to_string());
                    self.server = Some(IntelServer::Pinned);
                    return;
                }
            },
            Some(IntelServer::Binary(bin)) => pinned(bin),
            Some(IntelServer::Custom(c)) => c,
            other => {
                self.server = other;
                return;
            }
        };
        self.state = Some(ServerState::Starting);
        self.session = Some(LspSession::with_connect(connect));
    }

    /// Um quadro: arranca (no 1.º documento aberto), traz o que o servidor disse, e envia o que
    /// mudou em cada script desde o último quadro.
    pub fn sync(
        &mut self,
        docs: &mut [CodeDoc],
        parked: &BTreeMap<ScriptId, Parked>,
        lib: &ph2d_script::ScriptLibrary,
        now_ns: u128,
    ) {
        if self.session.is_none() && !self.off() && self.missing.is_none() && !docs.is_empty() {
            self.start();
        }
        let Some(session) = &self.session else {
            for d in docs.iter_mut() {
                d.lsp.edits.clear();
                d.assist.clear();
                d.complete.close();
            }
            return;
        };
        for e in session.drain() {
            match e {
                Event::State(s) => self.state = Some(s),
                Event::Diagnostics { uri, version, list } => {
                    let owner = self.mirrors.iter().find(|(_, m)| m.uri == uri);
                    if let Some((id, m)) = owner
                        && m.version == version
                    {
                        self.diags.insert(*id, (version, list));
                    }
                }
                Event::Answer { ticket, answer, .. } => {
                    let asked = |d: &&mut CodeDoc| {
                        d.complete
                            .session
                            .as_ref()
                            .is_some_and(|s| s.ticket == Some(ticket))
                    };
                    let Some(d) = docs.iter_mut().find(asked) else {
                        // O balão ou a assinatura de uma aba (os bilhetes são únicos na sessão).
                        if let Some(d) = docs.iter_mut().find(|d| d.hints.owns(ticket)) {
                            d.hints.answer(ticket, answer);
                        }
                        continue;
                    };
                    match answer {
                        Answer::Completion(items) => {
                            let props = d.syntax.declared_properties(d.buffer.text());
                            let usage = usage_of(&mut self.usage, lib);
                            d.complete.answer(ticket, items, &props, usage);
                        }
                        _ => d.complete.close(),
                    }
                }
            }
        }
        let gone: Vec<ScriptId> = self
            .mirrors
            .keys()
            .copied()
            .filter(|id| lib.get(*id).is_none())
            .collect();
        for id in gone {
            if let Some(m) = self.mirrors.remove(&id) {
                session.close(&m.uri);
            }
            self.diags.remove(&id);
        }
        self.hand_over(docs);
        for (id, s) in lib.iter() {
            let uri = uri_of(id.0, &s.name);
            let (src, text, pending) = if let Some(d) = docs.iter_mut().find(|d| d.script == id) {
                let src = Src::Doc {
                    serial: d.serial,
                    revision: d.buffer.revision(),
                };
                let from = Src::Doc {
                    serial: d.serial,
                    revision: d.lsp.from,
                };
                let p = std::mem::take(&mut d.lsp.edits);
                d.lsp.from = d.buffer.revision();
                (src, None, Some((from, p, d.buffer.text().clone())))
            } else if let Some(p) = parked.get(&id) {
                let src = Src::Doc {
                    serial: p.serial,
                    revision: p.buffer.revision(),
                };
                (src, Some(p.buffer.text().clone()), None)
            } else {
                (Src::Library(lib.revision()), None, None)
            };
            let m = self.mirrors.get_mut(&id);
            if let Some(m) = &m
                && m.uri == uri
                && m.src == src
            {
                continue;
            }
            let text = match (text, &pending) {
                (Some(t), _) => t,
                (None, Some((_, _, t))) => t.clone(),
                (None, None) => Text::new(&s.text),
            };
            match m {
                Some(m) if m.uri == uri => {
                    // As edições do documento desde o último envio dizem a troca sem olhar o
                    // texto; outra fonte (ou um comprimento que não bate) compara os dois.
                    let by_edits = match pending {
                        Some((from, edits, _)) if m.src == from => {
                            match Dirty::of(&edits, m.text.len()) {
                                None if text.len() == m.text.len() => Some(None),
                                Some(d)
                                    if m.text.len() - d.old.len() + d.new.len() == text.len() =>
                                {
                                    Some(Some(d))
                                }
                                _ => {
                                    debug_assert!(
                                        false,
                                        "uma edição do buffer não passou por `edited`"
                                    );
                                    None
                                }
                            }
                        }
                        _ => None,
                    };
                    let dirty = by_edits.unwrap_or_else(|| Dirty::between(&m.text, &text));
                    m.src = src;
                    if let Some(dirty) = dirty {
                        m.version += 1;
                        session.change(&uri, m.version, &text, dirty);
                        m.text = text;
                    }
                }
                Some(m) => {
                    session.close(&m.uri);
                    m.version += 1;
                    session.open(&uri, m.version, &text);
                    *m = Mirror {
                        uri,
                        version: m.version,
                        text,
                        src,
                    };
                }
                None => {
                    session.open(&uri, 1, &text);
                    self.mirrors.insert(
                        id,
                        Mirror {
                            uri,
                            version: 1,
                            text,
                            src,
                        },
                    );
                }
            }
        }
        self.ask_completions(docs, now_ns);
        self.ask_hints(docs, now_ns);
    }

    /// O balão cuja pausa passou e a assinatura cujo texto ou cursor mudou perguntam sobre o texto
    /// que o servidor acabou de receber. Sem servidor que responda, fecham.
    fn ask_hints(&mut self, docs: &mut [CodeDoc], now_ns: u128) {
        let waiting = matches!(
            self.state,
            Some(ServerState::Starting | ServerState::Restarting)
        );
        let ready = matches!(self.state, Some(ServerState::Ready));
        let Some(session) = self.session.as_mut() else {
            return;
        };
        for d in docs.iter_mut() {
            if d.hints.hover.is_none() && d.hints.sig.is_none() {
                continue;
            }
            if !ready {
                if !waiting {
                    d.hints.close();
                }
                continue;
            }
            let current = Src::Doc {
                serial: d.serial,
                revision: d.buffer.revision(),
            };
            let Some(m) = self.mirrors.get(&d.script).filter(|m| m.src == current) else {
                continue;
            };
            if let Some(h) = d.hints.hover.as_mut()
                && h.ticket.is_none()
                && h.due_ns <= now_ns
            {
                h.ticket = Some(session.ask(&m.uri, Req::Hover(h.word.start)));
            }
            let at = (d.buffer.revision(), d.buffer.selection().primary().head);
            if let Some(s) = d.hints.sig.as_mut()
                && s.asked != Some(at)
            {
                s.asked = Some(at);
                s.ticket = Some(session.ask(&m.uri, Req::Signature(at.1)));
            }
        }
    }

    /// A pausa da lista passou: pergunta ao servidor sobre o texto que ele acabou de receber. Sem
    /// servidor que responda (parado, em falta), a lista não abre.
    fn ask_completions(&mut self, docs: &mut [CodeDoc], now_ns: u128) {
        let waiting = matches!(
            self.state,
            Some(ServerState::Starting | ServerState::Restarting)
        );
        let ready = matches!(self.state, Some(ServerState::Ready));
        let Some(session) = self.session.as_mut() else {
            return;
        };
        for d in docs.iter_mut() {
            let Some(s) = d.complete.session.as_mut() else {
                continue;
            };
            if s.ticket.is_some() || s.due_ns > now_ns {
                continue;
            }
            if !ready {
                if !waiting {
                    d.complete.close();
                }
                continue;
            }
            let current = Src::Doc {
                serial: d.serial,
                revision: d.buffer.revision(),
            };
            let Some(m) = self.mirrors.get(&d.script).filter(|m| m.src == current) else {
                continue;
            };
            let head = d.buffer.selection().primary().head;
            s.ticket = Some(session.ask(&m.uri, Req::Completion(head)));
        }
    }
}

/// O uso de cada nome no projeto, contado de novo só quando a biblioteca muda.
fn usage_of<'a>(
    cache: &'a mut Option<(u64, BTreeMap<String, u32>)>,
    lib: &ph2d_script::ScriptLibrary,
) -> &'a BTreeMap<String, u32> {
    if cache.as_ref().is_none_or(|c| c.0 != lib.revision()) {
        let u = crate::complete::usage(lib.iter().map(|(_, s)| s.text.as_str()));
        *cache = Some((lib.revision(), u));
    }
    &cache.as_ref().expect("acabado de contar").1
}

/// O `luau-lsp` `bin` com as definições e a documentação geradas da API (ficheiros por conteúdo na
/// cache: duas instâncias do PH2D partilham-nos sem se pisarem).
fn pinned(bin: PathBuf) -> Connect {
    Box::new(move |wake| {
        let (defs, docs) = support_files().map_err(ph2d_code_lsp::LspError::Spawn)?;
        let mut cfg = ServerConfig::new(bin.clone()).with_definitions(DEFINITIONS_PACKAGE, defs);
        cfg.docs = Some(docs);
        LspClient::start_with_wake(cfg, Some(wake))
    })
}

/// As definições (`--definitions`) e a base de documentação (`--docs`) da API, escritas uma vez por
/// conteúdo em `<cache>/ph2d/luau-lsp/<versão>/api/`.
pub(crate) fn support_files() -> std::io::Result<(PathBuf, PathBuf)> {
    let dir = ph2d_code_lsp::resolve::cache_dir()
        .unwrap_or_else(|| std::env::temp_dir().join("ph2d-luau-lsp"))
        .join("api");
    std::fs::create_dir_all(&dir)?;
    let put = |stem: &str, ext: &str, body: &str| -> std::io::Result<PathBuf> {
        let p = dir.join(format!("{stem}-{:016x}.{ext}", fnv64(body.as_bytes())));
        if std::fs::read(&p).ok().as_deref() != Some(body.as_bytes()) {
            let tmp = dir.join(format!("{stem}-{}.tmp", std::process::id()));
            std::fs::write(&tmp, body)?;
            std::fs::rename(&tmp, &p)?;
        }
        Ok(p)
    };
    Ok((
        put("ph2d", "d.luau", &luau_definitions())?,
        put("ph2d-docs", "json", &luau_docs())?,
    ))
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}
