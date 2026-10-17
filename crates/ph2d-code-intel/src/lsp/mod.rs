//! A sessão assíncrona do `luau-lsp`: o quadro NUNCA espera o servidor.
//!
//! Lei (`docs/CodeEditor/03_onda2.md` I2): o lado do quadro só ENVIA (instantâneos da rope — um
//! `Arc` — e pedidos etiquetados) e só DRENA (`try_recv`). Um worker é dono do [`LspClient`]: converte
//! bytes ↔ UTF-16 sobre o instantâneo da VERSÃO de cada pedido, manda `didChange` incremental e
//! devolve respostas com a versão a que respondem. Quem pinta decide o que fazer com uma resposta
//! velha (o servidor ecoa a versão em `publishDiagnostics`, medido 16/10).

mod answer;
mod fix;
mod sem;
mod worker;

pub use answer::{Answer, Completion, Diag, Hover, Signature};
pub use fix::{Edit, Fix, TypeHint};
pub use sem::{Sem, SemToken, Target};

use crate::pos::Dirty;
use ph2d_code_buffer::Text;
use ph2d_code_lsp::{LspClient, LspError, ServerConfig};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// O endereço de um documento no servidor (os scripts do projeto não estão em disco): o id faz-no
/// único (dois nomes que se escrevem igual depois de codificados nunca colidem), o nome lê-se nas
/// mensagens, e a extensão `.luau` é a que diz ao servidor que é Luau.
pub fn uri_of(script: u64, name: &str) -> String {
    let mut out = format!("file:///ph2d-project/{script}/");
    for b in name.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    if !name.ends_with(".luau") && !name.ends_with(".lua") {
        out.push_str(".luau");
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticket(pub u64);

/// O que se pergunta, em BYTES do texto da versão do pedido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Req {
    Completion(usize),
    Hover(usize),
    Signature(usize),
    /// Onde o nome em `usize` está definido (I6).
    Definition(usize),
    /// Os tokens semânticos do documento inteiro (I11; o `luau-lsp` não tem `range`).
    Tokens,
    /// As correcções do servidor na faixa, com os diagnósticos que a aba MOSTRA ali (I7).
    Fixes(std::ops::Range<usize>, Vec<Diag>),
    /// A lista de completação em `usize` como candidatas do «você quis dizer» (I7): outro género
    /// que a da lista, para que as duas nunca se cancelem.
    Candidates(usize),
    /// As dicas de tipo do documento (I7, «adicionar tipo»).
    TypeHints,
}

impl Req {
    /// Dois pedidos do mesmo género sobre o mesmo documento: o novo torna o velho inútil.
    fn kind(&self) -> u8 {
        match self {
            Self::Completion(_) => 0,
            Self::Hover(_) => 1,
            Self::Signature(_) => 2,
            Self::Definition(_) => 3,
            Self::Tokens => 4,
            Self::Fixes(..) => 5,
            Self::Candidates(_) => 6,
            Self::TypeHints => 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerState {
    Starting,
    Ready,
    /// Morreu e está a ser reposto (os documentos abertos voltam a ser enviados).
    Restarting,
    /// Não arrancou, ou morreu outra vez antes de responder: só um [`LspSession::retry`] tenta de novo.
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Diagnostics {
        uri: String,
        version: u64,
        list: Vec<Diag>,
    },
    Answer {
        ticket: Ticket,
        uri: String,
        version: u64,
        answer: Answer,
    },
    State(ServerState),
}

pub(crate) enum Inbox {
    Open {
        uri: String,
        version: u64,
        text: Text,
    },
    Change {
        uri: String,
        version: u64,
        text: Text,
        dirty: Dirty,
    },
    Close(String),
    Ask {
        ticket: Ticket,
        uri: String,
        req: Req,
    },
    Cancel(Ticket),
    Reply {
        ticket: Ticket,
        result: Result<serde_json::Value, LspError>,
    },
    /// Uma notificação chegou ao cliente (ou o servidor foi-se): é hora de `pump`.
    Wake,
    Retry,
    Shutdown,
}

/// Arranca (ou re-arranca) o servidor; a sessão dá-lhe a função que acorda o worker.
pub type Connect =
    Box<dyn FnMut(ph2d_code_lsp::client::Wake) -> Result<LspClient, LspError> + Send>;

/// O lado do quadro.
pub struct LspSession {
    inbox: Sender<Inbox>,
    events: Receiver<Event>,
    next: u64,
    worker: Option<thread::JoinHandle<()>>,
}

impl LspSession {
    /// O `luau-lsp` fixado com `config`.
    pub fn start(config: ServerConfig) -> Self {
        Self::with_connect(Box::new(move |wake| {
            LspClient::start_with_wake(config.clone(), Some(wake))
        }))
    }

    /// Qualquer servidor (os falsos dos testes: lento, que morre).
    pub fn with_connect(connect: Connect) -> Self {
        let (inbox, rx) = mpsc::channel();
        let (tx, events) = mpsc::channel();
        let me = inbox.clone();
        let worker = thread::Builder::new()
            .name("ph2d-code-intel-lsp".into())
            .spawn(move || worker::run(connect, me, rx, tx))
            .expect("thread do worker do luau-lsp");
        Self {
            inbox,
            events,
            next: 0,
            worker: Some(worker),
        }
    }

    pub fn open(&self, uri: &str, version: u64, text: &Text) {
        let _ = self.inbox.send(Inbox::Open {
            uri: uri.to_owned(),
            version,
            text: text.clone(),
        });
    }

    /// `text` é o texto DEPOIS das edições do lote; `dirty` diz onde ele difere do anterior.
    pub fn change(&self, uri: &str, version: u64, text: &Text, dirty: Dirty) {
        let _ = self.inbox.send(Inbox::Change {
            uri: uri.to_owned(),
            version,
            text: text.clone(),
            dirty,
        });
    }

    pub fn close(&self, uri: &str) {
        let _ = self.inbox.send(Inbox::Close(uri.to_owned()));
    }

    /// Pergunta sobre o texto da ÚLTIMA versão enviada de `uri`. Um pedido do mesmo género ainda por
    /// responder sobre o mesmo documento é cancelado.
    pub fn ask(&mut self, uri: &str, req: Req) -> Ticket {
        self.next += 1;
        let ticket = Ticket(self.next);
        let _ = self.inbox.send(Inbox::Ask {
            ticket,
            uri: uri.to_owned(),
            req,
        });
        ticket
    }

    pub fn cancel(&self, ticket: Ticket) {
        let _ = self.inbox.send(Inbox::Cancel(ticket));
    }

    pub fn retry(&self) {
        let _ = self.inbox.send(Inbox::Retry);
    }

    /// Tudo o que chegou desde a última vez — nunca bloqueia.
    pub fn drain(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }

    /// Espera um evento (só para testes e sondas — o quadro usa [`Self::drain`]).
    pub fn wait(&self, timeout: std::time::Duration) -> Option<Event> {
        self.events.recv_timeout(timeout).ok()
    }
}

/// Quanto o fecho da sessão espera o worker: um servidor preso responderia ao `shutdown` só no
/// prazo do pedido (30 s) — fechar o editor nunca espera isso.
const DROP_WAIT: std::time::Duration = std::time::Duration::from_secs(1);

impl Drop for LspSession {
    /// Pede o fim e espera até [`DROP_WAIT`]; depois disso o worker acaba sozinho (o cliente mata o
    /// processo ao largar-se).
    fn drop(&mut self) {
        let _ = self.inbox.send(Inbox::Shutdown);
        let Some(w) = self.worker.take() else { return };
        let deadline = std::time::Instant::now() + DROP_WAIT;
        while !w.is_finished() && std::time::Instant::now() < deadline {
            thread::sleep(std::time::Duration::from_millis(2));
        }
        if w.is_finished() {
            let _ = w.join();
        }
    }
}
