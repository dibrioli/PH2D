//! **O QUADRO** (MiroClone) — as abas de documento da barra de cima e o que o quadro diz.
//!
//! ⚠️ O nome de um quadro é do ARTISTA depois de nascer: `board.tab.default_name` só redige o
//! primeiro nome (`{n}` = o número da aba), e o que fica gravado é a frase já redigida.

/// A tradução de uma chave `board.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.tab.scene" => "Scene",
        "board.tab.default_name" => "Board {n}",
        "board.tab.new" => "New board",
        _ => return None,
    })
}
