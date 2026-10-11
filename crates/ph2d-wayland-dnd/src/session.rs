//! O gesto de largar ficheiros visto pelo DESTINO, sem compositor: o que pedir e o que entregar.
//!
//! A lista é lida UMA vez por oferta, logo na entrada (o aviso de «largar» mostra os nomes), e o
//! largar usa essa leitura. Um largar sem lista ainda chegada fica «a aterrar» até ela chegar —
//! e um `leave` depois do `drop` não o cancela (o KWin manda-o antes de a lista chegar; Blender
//! rB00aa57594c74).

use std::path::PathBuf;

use crate::uri_list::{self, UriList};

/// O que a app recebe. Posições em pixels LÓGICOS da superfície (o `wl_fixed` do compositor).
#[derive(Debug, Clone, PartialEq)]
pub enum DndEvent {
    /// O arrasto está sobre a janela com estes ficheiros.
    Hover { paths: Vec<PathBuf>, at: (f64, f64) },
    /// O arrasto mexeu-se (só depois de um `Hover`).
    Motion { at: (f64, f64) },
    /// O arrasto saiu sem largar (só depois de um `Hover`).
    Leave,
    /// Largou em `at` (a última posição vista: o `drop` do Wayland não traz posição).
    Drop { list: UriList, at: (f64, f64) },
}

/// Identifica uma oferta entre o pedido de leitura e a chegada da lista.
pub type Token = u64;

/// O que a camada do compositor tem de fazer.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Emit(DndEvent),
    /// Largou e a lista chegou: `finish` + destruir a oferta.
    Finish(Token),
    /// Saiu sem largar: esquecer a leitura e a oferta (o toolkit já a destruiu).
    Cancel(Token),
}

#[derive(Debug)]
struct Drag {
    token: Token,
    at: (f64, f64),
    list: Option<UriList>,
    hovered: bool,
}

#[derive(Debug, Default)]
pub struct Session {
    next: Token,
    current: Option<Drag>,
    landing: Vec<Drag>,
}

impl Session {
    /// Entrou um arrasto. `Some(token)`: aceite a `text/uri-list` e leia-a com este token;
    /// `None`: recuse (outra superfície, ou nada que seja lista de ficheiros).
    pub fn enter(
        &mut self,
        ours: bool,
        mimes: &[String],
        at: (f64, f64),
    ) -> (Option<Token>, Vec<Effect>) {
        let fx = self.abandon_current();
        if !ours || !mimes.iter().any(|m| m == crate::URI_LIST_MIME) {
            return (None, fx);
        }
        let token = self.next;
        self.next += 1;
        self.current = Some(Drag {
            token,
            at,
            list: None,
            hovered: false,
        });
        (Some(token), fx)
    }

    pub fn motion(&mut self, at: (f64, f64)) -> Vec<Effect> {
        match self.current.as_mut() {
            Some(d) => {
                d.at = at;
                if d.hovered {
                    vec![Effect::Emit(DndEvent::Motion { at })]
                } else {
                    Vec::new()
                }
            }
            None => Vec::new(),
        }
    }

    pub fn leave(&mut self) -> Vec<Effect> {
        self.abandon_current()
    }

    pub fn drop_performed(&mut self) -> Vec<Effect> {
        let Some(mut d) = self.current.take() else {
            return Vec::new();
        };
        match d.list.take() {
            Some(list) => land(d.token, list, d.at),
            None => {
                self.landing.push(d);
                Vec::new()
            }
        }
    }

    /// A lista da oferta `token` chegou (ou a leitura falhou: `bytes` vazio).
    pub fn data_ready(&mut self, token: Token, bytes: &[u8]) -> Vec<Effect> {
        let list = uri_list::parse(bytes);
        if let Some(i) = self.landing.iter().position(|d| d.token == token) {
            let d = self.landing.remove(i);
            return land(token, list, d.at);
        }
        match self.current.as_mut() {
            Some(d) if d.token == token => {
                let fx = if list.paths.is_empty() {
                    Vec::new()
                } else {
                    d.hovered = true;
                    vec![Effect::Emit(DndEvent::Hover {
                        paths: list.paths.clone(),
                        at: d.at,
                    })]
                };
                d.list = Some(list);
                fx
            }
            _ => Vec::new(),
        }
    }

    fn abandon_current(&mut self) -> Vec<Effect> {
        let Some(d) = self.current.take() else {
            return Vec::new();
        };
        let mut fx = Vec::with_capacity(2);
        if d.hovered {
            fx.push(Effect::Emit(DndEvent::Leave));
        }
        fx.push(Effect::Cancel(d.token));
        fx
    }
}

fn land(token: Token, list: UriList, at: (f64, f64)) -> Vec<Effect> {
    vec![
        Effect::Emit(DndEvent::Drop { list, at }),
        Effect::Finish(token),
    ]
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
