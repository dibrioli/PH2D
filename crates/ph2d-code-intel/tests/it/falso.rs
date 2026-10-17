//! Um servidor LSP FALSO no próprio processo (canos do SO): responde ao `initialize` e, a cada
//! pedido, depois de `atraso`; grava o que recebeu; pode morrer depois de N respostas.

use ph2d_code_intel::lsp::Connect;
use ph2d_code_lsp::framing::{read_message, write_message};
use ph2d_code_lsp::{LspClient, ServerConfig};
use serde_json::{Value, json};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Default)]
pub struct Gravado {
    /// Cada mensagem recebida, por arranque (0 = o primeiro).
    pub por_arranque: Arc<Mutex<Vec<Vec<Value>>>>,
}

impl Gravado {
    pub fn metodos(&self, arranque: usize) -> Vec<String> {
        self.por_arranque.lock().unwrap()[arranque]
            .iter()
            .filter_map(|m| m.get("method").and_then(Value::as_str).map(str::to_owned))
            .collect()
    }

    pub fn arranques(&self) -> usize {
        self.por_arranque.lock().unwrap().len()
    }
}

#[derive(Clone, Copy)]
pub struct Plano {
    pub atraso: Duration,
    /// Morre (fecha o cano) depois de responder a este número de pedidos que não o `initialize`.
    pub morre_depois_de: Option<usize>,
    /// Morre logo depois do `initialize` (um arranque partido).
    pub morre_no_arranque: bool,
}

impl Plano {
    pub fn lento(atraso: Duration) -> Self {
        Self {
            atraso,
            morre_depois_de: None,
            morre_no_arranque: false,
        }
    }
}

fn servir(
    from_client: std::io::PipeReader,
    mut to_client: std::io::PipeWriter,
    plano: Plano,
    gravado: Gravado,
    arranque: usize,
) {
    let mut r = BufReader::new(from_client);
    let mut respondidos = 0usize;
    while let Ok(Some(msg)) = read_message(&mut r) {
        gravado.por_arranque.lock().unwrap()[arranque].push(msg.clone());
        let (Some(id), Some(method)) = (msg.get("id"), msg.get("method").and_then(Value::as_str))
        else {
            continue;
        };
        let result = match method {
            "initialize" => json!({ "capabilities": {} }),
            "shutdown" => Value::Null,
            "textDocument/completion" => {
                thread::sleep(plano.atraso);
                json!({ "items": [{ "label": "amplitude" }] })
            }
            _ => {
                thread::sleep(plano.atraso);
                Value::Null
            }
        };
        if write_message(
            &mut to_client,
            &json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        )
        .is_err()
        {
            return;
        }
        if method == "initialize" {
            if plano.morre_no_arranque {
                return;
            }
            continue;
        }
        respondidos += 1;
        if plano.morre_depois_de == Some(respondidos) {
            return;
        }
    }
}

/// A função de arranque da sessão: cada chamada liga um servidor falso novo.
pub fn ligar(plano: Plano, gravado: Gravado) -> Connect {
    Box::new(move |wake| {
        let (srv_in, cli_out) = std::io::pipe().expect("cano");
        let (cli_in, srv_out) = std::io::pipe().expect("cano");
        let arranque = {
            let mut v = gravado.por_arranque.lock().unwrap();
            v.push(Vec::new());
            v.len() - 1
        };
        let g = gravado.clone();
        thread::spawn(move || servir(srv_in, srv_out, plano, g, arranque));
        LspClient::connect(
            ServerConfig::new(PathBuf::from("falso")),
            Box::new(cli_out),
            Box::new(cli_in),
            Some(wake),
        )
    })
}
