//! Os ENVIOS da sessão ao vivo (cortado do `live.rs`, tecto de LOC): cada mensagem pelo transporte, e
//! se alguma passou do tecto dele (`MAX_FRAME` — uma imagem de mais de 256 MiB, W14) quem está no
//! quadro sabe (nunca uma falha calada: os outros nunca a veriam).

use ph2d_board_net::{self as net, SendError};
use ph2d_board_sync::{Msg, To};
use ph2d_editor_core::screens::HeroScreen;
use ph2d_i18n::tr_with;

fn too_big(r: Result<(), SendError>) -> bool {
    matches!(r, Err(SendError::TooBig { .. }))
}

/// O anfitrião manda `out`; `true` = alguma passou do tecto.
pub(super) fn host(net: &net::Host, out: Vec<(To, Msg)>) -> bool {
    let mut big = false;
    for (to, m) in out {
        let bytes = m.encode();
        big |= too_big(match to {
            To::One(c) => net.send(c, bytes),
            To::All => net.broadcast(bytes, None),
            To::AllBut(c) => net.broadcast(bytes, Some(c)),
        });
    }
    big
}

/// O convidado manda `out`; `true` = alguma passou do tecto.
pub(super) fn guest(net: &net::Guest, out: Vec<Msg>) -> bool {
    out.into_iter()
        .fold(false, |big, m| big | too_big(net.send(m.encode())))
}

/// Diz no quadro que uma mensagem (uma imagem) não coube no transporte.
pub(super) fn tell_too_big(hero: &mut HeroScreen, big: bool) {
    if big {
        let max = (net::MAX_FRAME >> 20).to_string();
        hero.documents
            .tell(tr_with("board.live.too_big", &[("max", &max)]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Auditoria W14 — só o «grande demais» se diz no quadro, com o tecto em MB; nada a dizer, nada.
    #[test]
    fn only_a_too_big_frame_is_told_on_the_board() {
        assert!(too_big(Err(SendError::TooBig { len: 1, max: 0 })));
        assert!(!too_big(Err(SendError::Gone)) && !too_big(Ok(())));
        let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
        tell_too_big(&mut h, false);
        assert_eq!(h.documents.told(), None);
        tell_too_big(&mut h, true);
        assert_eq!(
            h.documents.told(),
            Some("A picture is too big to share live (over 256 MB)")
        );
    }
}
