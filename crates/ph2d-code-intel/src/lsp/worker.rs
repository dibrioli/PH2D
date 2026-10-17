//! O worker: dono do [`LspClient`], acordado pela caixa de entrada (pedidos do quadro, respostas
//! entregues pelo leitor do cliente, notificações). Nunca há relógio a girar: ele dorme em `recv`.

use super::answer::{self, Answer, Diag};
use super::sem::{self, Legend};
use super::{Connect, Event, Inbox, Req, ServerState, Ticket};
use crate::pos::{position_of, range_of};
use ph2d_code_buffer::Text;
use ph2d_code_lsp::client::Wake;
use ph2d_code_lsp::{LspClient, LspError, Notification};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};

struct Doc {
    version: u64,
    text: Text,
    /// Os textos das versões enviadas e ainda não diagnosticadas (o servidor ecoa a versão).
    sent: BTreeMap<u64, Text>,
}

struct Flight {
    id: i64,
    uri: String,
    version: u64,
    text: Text,
    req: Req,
}

struct Worker {
    connect: Connect,
    me: Sender<Inbox>,
    tx: Sender<Event>,
    client: Option<LspClient>,
    docs: BTreeMap<String, Doc>,
    flights: BTreeMap<Ticket, Flight>,
    /// O servidor respondeu alguma coisa desde o último arranque: uma morte agora repõe-no; uma morte
    /// antes disso é um arranque partido e fica em [`ServerState::Failed`].
    healthy: bool,
    /// A legenda dos tokens semânticos do servidor deste arranque.
    legend: Legend,
}

pub(super) fn run(connect: Connect, me: Sender<Inbox>, rx: Receiver<Inbox>, tx: Sender<Event>) {
    let mut w = Worker {
        connect,
        me,
        tx,
        client: None,
        docs: BTreeMap::new(),
        flights: BTreeMap::new(),
        healthy: false,
        legend: Legend::default(),
    };
    w.emit(Event::State(ServerState::Starting));
    w.boot();
    while let Ok(msg) = rx.recv() {
        if matches!(msg, Inbox::Shutdown) {
            break;
        }
        w.handle(msg);
    }
    if let Some(c) = w.client.take() {
        let _ = c.shutdown();
    }
}

fn params_at(uri: &str, text: &Text, byte: usize) -> Value {
    let p = position_of(text, byte);
    json!({ "textDocument": { "uri": uri },
            "position": { "line": p.line, "character": p.character } })
}

fn method_of(req: &Req) -> &'static str {
    match req {
        Req::Completion(_) => "textDocument/completion",
        Req::Hover(_) => "textDocument/hover",
        Req::Signature(_) => "textDocument/signatureHelp",
        Req::Definition(_) => "textDocument/definition",
        Req::Tokens => "textDocument/semanticTokens/full",
    }
}

fn params_of(uri: &str, text: &Text, req: &Req) -> Value {
    match req {
        Req::Completion(b) | Req::Hover(b) | Req::Signature(b) | Req::Definition(b) => {
            params_at(uri, text, *b)
        }
        Req::Tokens => json!({ "textDocument": { "uri": uri } }),
    }
}

impl Worker {
    /// A resposta `v` ao pedido `f`, em bytes do texto da versão do pedido.
    fn parse(&self, f: &Flight, v: &Value) -> Answer {
        let text = &f.text;
        match &f.req {
            Req::Completion(_) => Answer::Completion(answer::completion(text, v)),
            Req::Hover(_) => Answer::Hover(answer::hover(text, v)),
            Req::Signature(_) => Answer::Signature(answer::signature(v)),
            Req::Definition(_) => Answer::Definition(sem::definition(
                |uri| {
                    if uri == f.uri {
                        Some(text.clone())
                    } else {
                        self.docs.get(uri).map(|d| d.text.clone())
                    }
                },
                v,
            )),
            Req::Tokens => Answer::Tokens(sem::tokens(text, &self.legend, v)),
        }
    }
}

impl Worker {
    fn emit(&self, e: Event) {
        let _ = self.tx.send(e);
    }

    fn wake(&self) -> Wake {
        let me = self.me.clone();
        Arc::new(move || {
            let _ = me.send(Inbox::Wake);
        })
    }

    /// Arranca o servidor e reabre todos os documentos na versão de agora.
    fn boot(&mut self) {
        self.healthy = false;
        let wake = self.wake();
        match (self.connect)(wake) {
            Ok(mut c) => {
                self.legend = Legend::of(c.capabilities());
                let mut err = None;
                for (uri, d) in &mut self.docs {
                    d.sent.clear();
                    d.sent.insert(d.version, d.text.clone());
                    if let Err(e) = c.did_open_at(uri, d.version as i32, &d.text.to_string()) {
                        err = Some(e);
                        break;
                    }
                }
                self.client = Some(c);
                match err {
                    None => self.emit(Event::State(ServerState::Ready)),
                    Some(e) => self.died(&e),
                }
            }
            Err(e) => {
                self.client = None;
                self.emit(Event::State(ServerState::Failed(e.to_string())));
            }
        }
    }

    /// O servidor foi-se: os pedidos em voo falham com a razão, e ele volta se já tinha respondido.
    fn died(&mut self, e: &LspError) {
        self.client = None;
        for (ticket, f) in std::mem::take(&mut self.flights) {
            self.emit(Event::Answer {
                ticket,
                uri: f.uri,
                version: f.version,
                answer: Answer::Failed(e.to_string()),
            });
        }
        if self.healthy {
            self.emit(Event::State(ServerState::Restarting));
            self.boot();
        } else {
            self.emit(Event::State(ServerState::Failed(e.to_string())));
        }
    }

    /// Corre `f` no cliente vivo; um erro de morte leva a [`Self::died`].
    fn with_client<T>(
        &mut self,
        f: impl FnOnce(&mut LspClient) -> Result<T, LspError>,
    ) -> Option<T> {
        let c = self.client.as_mut()?;
        match f(c) {
            Ok(v) => Some(v),
            Err(e @ LspError::ServerDied { .. }) => {
                self.died(&e);
                None
            }
            Err(_) => None,
        }
    }

    fn handle(&mut self, msg: Inbox) {
        match msg {
            Inbox::Open { uri, version, text } => {
                let s = text.to_string();
                let mut sent = BTreeMap::new();
                sent.insert(version, text.clone());
                self.docs.insert(
                    uri.clone(),
                    Doc {
                        version,
                        text,
                        sent,
                    },
                );
                self.with_client(|c| c.did_open_at(&uri, version as i32, &s));
            }
            Inbox::Change {
                uri,
                version,
                text,
                dirty,
            } => {
                let Some(d) = self.docs.get_mut(&uri) else {
                    return;
                };
                let range = range_of(&d.text, dirty.old.clone());
                let insert = text.slice(dirty.new.clone());
                d.sent.insert(version, text.clone());
                d.text = text;
                d.version = version;
                self.with_client(|c| c.did_change_at(&uri, version as i32, range, &insert));
            }
            Inbox::Close(uri) => {
                self.docs.remove(&uri);
                self.with_client(|c| c.did_close(&uri));
            }
            Inbox::Ask { ticket, uri, req } => self.ask(ticket, uri, req),
            Inbox::Cancel(ticket) => {
                if let Some(f) = self.flights.remove(&ticket) {
                    self.with_client(|c| c.cancel(f.id));
                }
            }
            Inbox::Reply { ticket, result } => {
                let Some(f) = self.flights.remove(&ticket) else {
                    return;
                };
                let answer = match &result {
                    Ok(v) => {
                        self.healthy = true;
                        self.parse(&f, v)
                    }
                    Err(e) => Answer::Failed(e.to_string()),
                };
                self.emit(Event::Answer {
                    ticket,
                    uri: f.uri,
                    version: f.version,
                    answer,
                });
            }
            Inbox::Wake => self.pump(),
            Inbox::Retry => {
                if self.client.is_none() {
                    self.emit(Event::State(ServerState::Starting));
                    self.boot();
                }
            }
            Inbox::Shutdown => {}
        }
    }

    fn ask(&mut self, ticket: Ticket, uri: String, req: Req) {
        let stale: Vec<Ticket> = self
            .flights
            .iter()
            .filter(|(_, f)| f.uri == uri && f.req.kind() == req.kind())
            .map(|(t, _)| *t)
            .collect();
        for t in stale {
            if let Some(f) = self.flights.remove(&t) {
                self.with_client(|c| c.cancel(f.id));
            }
        }
        let fail = |w: &Self, why: &str| {
            w.emit(Event::Answer {
                ticket,
                uri: uri.clone(),
                version: 0,
                answer: Answer::Failed(why.to_owned()),
            });
        };
        let Some(d) = self.docs.get(&uri) else {
            return fail(self, "document not open");
        };
        if self.client.is_none() {
            return fail(self, "luau-lsp is not running");
        }
        let (version, text) = (d.version, d.text.clone());
        let params = params_of(&uri, &text, &req);
        let me = self.me.clone();
        let id = self.with_client(|c| {
            c.request_async(method_of(&req), params, move |result| {
                let _ = me.send(Inbox::Reply { ticket, result });
            })
        });
        if let Some(id) = id {
            self.flights.insert(
                ticket,
                Flight {
                    id,
                    uri,
                    version,
                    text,
                    req,
                },
            );
        }
    }

    fn pump(&mut self) {
        let Some(c) = self.client.as_mut() else {
            return;
        };
        let mut gone = false;
        for n in c.pump() {
            match n {
                Notification::Diagnostics(p) => {
                    let Some(d) = self.docs.get_mut(&p.uri) else {
                        continue;
                    };
                    self.healthy = true;
                    let version = p.version.map_or(d.version, |v| v as u64);
                    let Some(text) = d.sent.get(&version).cloned() else {
                        continue;
                    };
                    d.sent.retain(|v, _| *v >= version);
                    let list = p.diagnostics.iter().map(|x| Diag::of(&text, x)).collect();
                    let _ = self.tx.send(Event::Diagnostics {
                        uri: p.uri,
                        version,
                        list,
                    });
                }
                Notification::Exited => gone = true,
                _ => {}
            }
        }
        if gone {
            let e =
                self.client
                    .as_mut()
                    .map_or(LspError::Protocol("luau-lsp went away".into()), |c| {
                        LspError::ServerDied {
                            status: None,
                            stderr_tail: c.stderr_tail().join("\n"),
                        }
                    });
            self.died(&e);
        }
    }
}
