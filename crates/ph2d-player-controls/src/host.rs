//! **O que a shell pergunta à sessão num evento do Play** — a tecla, o clique e o movimento da tela «Controls», e o relógio
//! que eles pedem. A shell só traduz o evento e toca no relógio do jogo (saiu dela pela catraca
//! `the_shell_only_shrinks`, R10a).

use ph2d_editor_core::screens::hero::{HeroScreen, transport_bar};
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

/// O botão «Controls» da barra: a tecla que abre a tela (a do mapa), premida e solta pelo MESMO [`key`] — abre,
/// fecha (ou larga a escuta) exactamente como ela. Conta como jogo à vista: o botão só age no Play.
#[must_use]
pub fn toggle(s: &mut PlaySession, map: &InputMap, playing: bool) -> Clock {
    let open_key = Some(map.controls().open_key);
    let (_, clock) = key(s, open_key, (true, false), true, (Some(map), playing));
    let _ = key(s, open_key, (false, false), true, (Some(map), playing));
    clock
}

/// O clique: aberta, a tela é dona do rato (`.0`, qualquer botão). Só o botão PRINCIPAL (`primary`) age: o Down em
/// `at` (lógico) escolhe nela; o Up dispara o botão premido se cai dentro dele (fora cancela).
#[must_use]
pub fn click(
    s: &mut PlaySession,
    (down, primary): (bool, bool),
    at: Option<((f32, f32), &InputMap)>,
    playing: bool,
) -> (bool, Clock) {
    if !s.screen().is_open() {
        return (false, None);
    }
    let Some(((x, y), map)) = at.filter(|_| primary) else {
        return (true, None);
    };
    let out = if down {
        s.click(x, y, map)
    } else {
        s.release(x, y, map)
    };
    (true, s.clock_after(out, playing))
}

/// O movimento do ponteiro em `at` (lógico; `None` = saiu da janela). O ponto guarda-se sempre (a tela que abre
/// com o ponteiro parado acende o que está sob ele); aberta, a tela é dona dele. `(dela, repinta)` — `repinta` = o
/// alvo sob o ponteiro mudou (o hover acende no quadro, D32).
#[must_use]
pub fn pointer_move(s: &mut PlaySession, at: Option<(f32, f32)>) -> (bool, bool) {
    let repinta = s.pointer_move(at);
    (s.screen().is_open(), repinta)
}

/// ⭐ Com a tela aberta o botão «Controls» da barra acende sob o ponteiro (dono 10/11, D32): sobre ele o movimento é
/// da barra (`false`: a shell passa-o ao hero) e a tela solta o hover dela; fora dele é o [`pointer_move`], e o botão
/// apaga-se ([`transport_bar::movimento_com_a_tela`]).
#[must_use]
pub fn pointer_move_com_a_barra(
    s: &mut PlaySession,
    hero: Option<&mut HeroScreen>,
    at: Option<(f32, f32)>,
) -> (bool, bool) {
    let barra = |(h, (x, y)): (&mut HeroScreen, _)| transport_bar::movimento_com_a_tela(h, x, y);
    if s.screen().is_open() && hero.zip(at).is_some_and(barra) {
        return (false, s.pointer_move(None));
    }
    pointer_move(s, at)
}
