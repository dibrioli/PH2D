//! I10: os pedidos de rename e de referências (`crate::rename`, `crate::refs`) sobre o texto que o
//! servidor acabou de receber. Sem servidor que responda, o rename di-lo na caixinha.

use super::{Intel, Src};
use crate::doc::CodeDoc;
use ph2d_code_intel::{Req, ServerState};

impl Intel {
    /// Quantas vezes o servidor arrancou (ou foi reposto) nesta sessão.
    #[must_use]
    pub fn boots(&self) -> u64 {
        self.boots
    }

    pub(super) fn ask_names(&mut self, docs: &mut [CodeDoc]) {
        let waiting = matches!(
            self.state,
            Some(ServerState::Starting | ServerState::Restarting)
        );
        let ready = matches!(self.state, Some(ServerState::Ready));
        let Some(session) = self.session.as_mut() else {
            return;
        };
        for d in docs.iter_mut() {
            if !ready {
                if !waiting {
                    d.rename_offline();
                    d.occ.want = None;
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
            if let Some(b) = d.occ.want.take() {
                d.occ.ticket = Some(session.ask(&m.uri, Req::References(b)));
            }
            if let Some(r) = d.rename.as_mut()
                && let Some((name, t @ None)) = r.sent.as_mut()
            {
                *t = Some(session.ask(&m.uri, Req::Rename(r.word.start, name.clone())));
            }
        }
    }
}
