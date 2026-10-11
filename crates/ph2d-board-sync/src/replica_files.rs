//! W14 — os **ficheiros das imagens** ao vivo. Os bytes de uma imagem não mudam nunca (a chave é o
//! conteúdo): juntar os de dois pares é a união, sem carimbos que disputem. Cada ficheiro sai UMA vez
//! de cada par, numa escrita só dele — e numa MENSAGEM só dele ([`by_file`]): o tecto de uma mensagem é
//! o do transporte (`ph2d_board_net::MAX_FRAME`), e um quadro com muitas fotografias passava-o se elas
//! viajassem juntas (o retrato de quem entra vai sem elas, `BoardSet::board_bytes_lean`).

use super::{Replica, Write};
use crate::session::Msg;
use ph2d_board_model::Board;

impl Replica {
    /// Os ficheiros do quadro que ainda não saíram daqui (nem chegaram de fora), um por escrita.
    pub(super) fn new_files(&mut self, board: &Board) -> Vec<Write> {
        let new: Vec<_> = board
            .files
            .iter()
            .filter(|(k, _)| !self.files.contains(*k))
            .map(|(k, f)| (*k, f.clone()))
            .collect();
        new.into_iter()
            .map(|(k, file)| {
                self.files.insert(k);
                Write::File {
                    file,
                    at: self.tick(),
                }
            })
            .collect()
    }

    /// Uma escrita de ficheiro que chegou: entra no quadro (`true`); as outras não são daqui.
    pub(super) fn file_in(&mut self, board: &mut Board, w: &Write) -> bool {
        let Write::File { file, at } = w else {
            return false;
        };
        self.clock = self.clock.max(at.n);
        let id = file.id();
        self.files.insert(id);
        board.files.entry(id).or_insert_with(|| file.clone());
        true
    }

    /// ⭐ Os ficheiros que o quadro refere ([`Board::files_referenced`]), um por escrita — o que o
    /// anfitrião manda a quem entra, logo a seguir ao retrato magro.
    #[must_use]
    pub fn all_files(&mut self, board: &Board) -> Vec<Write> {
        let used = board.files_referenced();
        board
            .files
            .iter()
            .filter(|(k, _)| used.contains(*k))
            .map(|(_, f)| Write::File {
                file: f.clone(),
                at: self.tick(),
            })
            .collect()
    }
}

/// As mensagens de um lote de escritas: cada ficheiro na sua, o resto numa.
#[must_use]
pub fn by_file(ws: Vec<Write>) -> Vec<Msg> {
    let (files, rest): (Vec<Write>, Vec<Write>) =
        ws.into_iter().partition(|w| matches!(w, Write::File { .. }));
    let mut out: Vec<Msg> = files.into_iter().map(|w| Msg::Writes(vec![w])).collect();
    if !rest.is_empty() {
        out.insert(0, Msg::Writes(rest));
    }
    out
}
