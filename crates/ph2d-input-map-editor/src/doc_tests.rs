//! O ficheiro do jogador da aba sai da MESMA lei da tela do Play (`player_file_of`).

use super::{InputMapDoc, View};

/// `game_id` 0 não tem ficheiro (partilharia o `0000000000000000.txt` entre projectos); sem base, também não.
#[test]
fn game_id_zero_means_no_player_file() {
    let base = std::path::PathBuf::from("/nowhere/.ph2d");
    let mut d = InputMapDoc::new(View::List);
    d.tryit.base = Some(base.clone());
    assert_eq!(d.player_file(), None, "game_id 0");
    d.snap.game_id = 0x2A;
    let want = ph2d_player_controls::player_file(&base, 0x2A);
    assert_eq!(d.player_file(), Some(want));
    d.tryit.base = None;
    assert_eq!(d.player_file(), None, "sem base");
}
