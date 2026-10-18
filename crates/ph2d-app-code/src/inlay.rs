//! ⭐ **I9 — as dicas de tipo** (`docs/CodeEditor/03_onda2.md`; medição
//! `ferramentas/medicoes_onda2/I9_I12_I14.md` §I9).
//!
//! - **Desligadas por omissão**; o comando «Show Type Hints» liga-as em todas as abas (o VS Code não
//!   tem comando de fábrica: é uma definição).
//! - **Quais:** a dica que o Alt+Enter inseriria (a lei do I7, [`crate::actions::insertable`]) — no
//!   `bob.luau` 4 de 9: nem `any`, nem o `self` truncado, nem o genérico solto.
//! - **Quando:** na pausa das cores (I11), uma pergunta por versão; o servidor dá o ficheiro inteiro
//!   (ignora a faixa, I0b §6) e a pintura só põe as das filas à vista.
//! - **Edições:** a dica anda com o texto; escrever na posição dela empurra-a para depois do escrito;
//!   apagada a posição (dentro de uma faixa removida), sai até a pausa a refazer.

use crate::doc::map_byte;
use ph2d_code_buffer::TextEdit;
use ph2d_code_intel::{Answer, Ticket};
use ph2d_panel_code::Inlay;

/// As dicas que a aba segura.
#[derive(Default)]
pub(crate) struct InlayHeld {
    /// Por ordem de `at`.
    pub list: Vec<Inlay>,
    /// O (arranque do servidor, versão) da última pergunta.
    pub asked: Option<(u64, u64)>,
    pub ticket: Option<Ticket>,
}

impl InlayHeld {
    pub fn edited(&mut self, edits: &[TextEdit]) {
        if edits.is_empty() {
            return;
        }
        self.ticket = None;
        for e in edits {
            self.list
                .retain(|h| !(e.start_byte < h.at && h.at < e.old_end_byte));
            // Na posição dela, `map_byte` leva-a para depois do escrito.
            for h in &mut self.list {
                h.at = map_byte(h.at, e);
            }
        }
    }

    /// A resposta `ticket`: as dicas novas (um erro deixa as que havia).
    pub fn answer(&mut self, ticket: Ticket, answer: Answer) {
        if self.ticket != Some(ticket) {
            return;
        }
        self.ticket = None;
        if let Answer::TypeHints(hints) = answer {
            let mut list: Vec<Inlay> = hints
                .into_iter()
                .filter(|h| {
                    h.label
                        .trim_start()
                        .strip_prefix(':')
                        .is_some_and(|ty| crate::actions::insertable(ty.trim()))
                })
                .map(|h| Inlay {
                    at: h.at,
                    label: h.label,
                })
                .collect();
            list.sort_by_key(|h| h.at);
            self.list = list;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
