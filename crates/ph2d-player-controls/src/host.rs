//! **O que a shell pergunta à sessão num evento do Play** — a tecla, o clique e o movimento da tela «Controls», e o relógio
//! que eles pedem. A shell só traduz o evento e toca no relógio do jogo (saiu dela pela catraca
//! `the_shell_only_shrinks`, R10a).

use ph2d_input::{InputMap, Key};

use crate::{Outcome, PlaySession};

/// O relógio do jogo depois de um gesto ([`PlaySession::clock_after`]): `Some(false)` pausa, `Some(true)` retoma.
pub type Clock = Option<bool>;

/// A tecla física, ANTES da tecla do jogo e de todo atalho. `.0` = da tela (aberta, ou a tecla que a abre);
/// com a tela fechada, só conta com o jogo à vista (`game_on_screen`). `key` `None` = tecla sem nome.
#[must_use]
pub fn key(
    s: &mut PlaySession,
    key: Option<Key>,
    (down, repeat): (bool, bool),
    game_on_screen: bool,
    (map, playing): (Option<&InputMap>, bool),
) -> (bool, Clock) {
    let open = s.screen().is_open();
    if !s.running() || (!open && !game_on_screen) {
        return (false, None);
    }
    let (Some(k), false) = (key, repeat) else {
        return (open, None);
    };
    let Some(map) = map else {
        return (false, None);
    };
    let out = s.key(k, down, map);
    (out != Outcome::Ignored, s.clock_after(out, playing))
}

/// O clique: aberta, a tela é dona do rato (`.0`). O Down em `at` (lógico) escolhe nela; o Up dispara o botão
/// premido se cai dentro dele (fora cancela).
#[must_use]
pub fn click(
    s: &mut PlaySession,
    down: bool,
    at: Option<((f32, f32), &InputMap)>,
    playing: bool,
) -> (bool, Clock) {
    if !s.screen().is_open() {
        return (false, None);
    }
    let Some(((x, y), map)) = at else {
        return (true, None);
    };
    let out = if down {
        s.click(x, y, map)
    } else {
        s.release(x, y, map)
    };
    (true, s.clock_after(out, playing))
}

/// O movimento do ponteiro em `at` (lógico): aberta, a tela é dona dele. `(dela, repinta)` — `repinta` = o alvo
/// sob o ponteiro mudou (o hover acende no quadro, D32).
#[must_use]
pub fn pointer_move(s: &mut PlaySession, at: Option<(f32, f32)>) -> (bool, bool) {
    if !s.screen().is_open() {
        return (false, false);
    }
    (true, at.is_some_and(|(x, y)| s.pointer_move(x, y)))
}
