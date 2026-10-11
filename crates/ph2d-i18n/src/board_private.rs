//! **O QUADRO — o que é de uma pessoa** (MiroClone W12, ADR-0182): o modo privado e a votação entre
//! pessoas numa sessão ao vivo — o painel, o selo das notas escondidas e os avisos curtos.

/// A tradução de uma chave do modo privado e da votação ao vivo, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── os avisos curtos (a família mostra-os) ──
        "board.private.only_starter" => {
            "Only the person who turned on private mode (or the host) can do that"
        }
        "board.private.reveal_mine_off" => "Revealing your own notes isn't turned on",
        "board.livevote.running" => "A vote is already running",
        "board.livevote.only_starter" => {
            "Only the person who started the vote (or the host) can end it"
        }
        _ => return None,
    })
}
