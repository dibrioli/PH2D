//! Os gates do **toque sem teclado** (F5).

use super::*;

const MS: u128 = 1_000_000;

fn g() -> TouchGestures {
    TouchGestures::default()
}

/// ⭐ **O dedo que desenha só vira `Down` ao andar, e o `Down` sai na ORIGEM.**
#[test]
fn the_drawing_finger_presses_at_its_origin_once_it_travels() {
    let mut t = g();
    assert!(
        t.feed(1, TouchPhase::Start, [100.0, 100.0], 0, None)
            .is_empty()
    );
    assert!(
        t.feed(1, TouchPhase::Move, [104.0, 100.0], MS, None)
            .is_empty(),
        "dentro da folga já desenhou"
    );
    let out = t.feed(1, TouchPhase::Move, [130.0, 100.0], 2 * MS, None);
    assert_eq!(
        out,
        vec![
            TouchOut::Press([100.0, 100.0]),
            TouchOut::Drag([130.0, 100.0])
        ]
    );
    let out = t.feed(1, TouchPhase::End, [140.0, 100.0], 3 * MS, None);
    assert_eq!(out, vec![TouchOut::Release([130.0, 100.0])]);
}

/// **Um toque curto é um clique.**
#[test]
fn a_short_tap_is_a_click() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [50.0, 60.0], 0, None);
    let out = t.feed(1, TouchPhase::End, [51.0, 60.0], 80 * MS, None);
    assert_eq!(
        out,
        vec![
            TouchOut::Press([50.0, 60.0]),
            TouchOut::Release([50.0, 60.0])
        ]
    );
}

/// ⭐ **+1 dedo = Shift, +2 = Shift+Alt, enquanto o primeiro arrasta — e soltá-los solta.**
///
/// ⚠️ Com o CONTROLO: o mesmo segundo dedo ANTES de o primeiro andar não é modificador (é gesto).
#[test]
fn extra_fingers_are_shift_then_shift_alt_while_drawing() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [0.0, 0.0], 0, None);
    t.feed(1, TouchPhase::Move, [40.0, 0.0], MS, None);
    assert_eq!(t.modifiers(), TouchMods::default());
    t.feed(2, TouchPhase::Start, [500.0, 500.0], 2 * MS, None);
    assert!(
        t.modifiers().shift && !t.modifiers().alt,
        "+1 dedo não é Shift"
    );
    t.feed(3, TouchPhase::Start, [520.0, 500.0], 3 * MS, None);
    assert!(
        t.modifiers().shift && t.modifiers().alt,
        "+2 dedos não é Shift+Alt"
    );
    assert_eq!(
        t.feed(1, TouchPhase::Move, [60.0, 0.0], 4 * MS, None),
        vec![TouchOut::Drag([60.0, 0.0])],
        "o traço parou com os dedos extra"
    );
    t.feed(3, TouchPhase::End, [520.0, 500.0], 5 * MS, None);
    assert!(t.modifiers().shift && !t.modifiers().alt);
    t.feed(2, TouchPhase::End, [500.0, 500.0], 6 * MS, None);
    assert_eq!(t.modifiers(), TouchMods::default());

    let mut c = g();
    c.feed(1, TouchPhase::Start, [0.0, 0.0], 0, None);
    c.feed(2, TouchPhase::Start, [30.0, 0.0], MS, None);
    assert_eq!(
        c.modifiers(),
        TouchMods::default(),
        "o controlo: virou Shift"
    );
}

/// ⭐ **Toque de 2 dedos desfaz, 3 refaz, 4 foca — e nenhum traço nasce.**
#[test]
fn two_three_four_finger_taps_undo_redo_focus() {
    for (n, want) in [
        (2u64, TouchOut::Undo),
        (3, TouchOut::Redo),
        (4, TouchOut::Focus),
    ] {
        let mut t = g();
        let mut out = Vec::new();
        for f in 0..n {
            #[allow(clippy::cast_precision_loss)]
            out.extend(t.feed(f, TouchPhase::Start, [f as f32 * 40.0, 0.0], 0, None));
        }
        for f in 0..n {
            #[allow(clippy::cast_precision_loss)]
            out.extend(t.feed(f, TouchPhase::End, [f as f32 * 40.0, 0.0], 90 * MS, None));
        }
        assert_eq!(out, vec![want], "{n} dedos");
    }
}

/// ⛔ **Dedos que ANDAM ou que ficam além do tocar-e-segurar não são toque.**
#[test]
fn a_moving_or_slow_multi_finger_contact_is_not_a_tap() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [0.0, 0.0], 0, None);
    t.feed(2, TouchPhase::Start, [40.0, 0.0], 0, None);
    t.feed(2, TouchPhase::Move, [90.0, 0.0], MS, None);
    t.feed(1, TouchPhase::End, [0.0, 0.0], 2 * MS, None);
    assert!(
        t.feed(2, TouchPhase::End, [90.0, 0.0], 3 * MS, None)
            .is_empty()
    );
    let mut t = g();
    t.feed(1, TouchPhase::Start, [0.0, 0.0], 0, None);
    t.feed(2, TouchPhase::Start, [40.0, 0.0], 0, None);
    t.feed(1, TouchPhase::End, [0.0, 0.0], LONG_PRESS_NS, None);
    assert!(
        t.feed(2, TouchPhase::End, [40.0, 0.0], LONG_PRESS_NS, None)
            .is_empty()
    );
}

/// ⭐ **Tocar e segurar abre o menu de contexto — no relógio, sem outro evento — e soltar depois
/// não clica.**
#[test]
fn touch_and_hold_opens_the_context_menu() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [70.0, 80.0], 0, None);
    assert!(t.tick(LONG_PRESS_NS - 1).is_empty(), "abriu antes da hora");
    assert_eq!(
        t.tick(LONG_PRESS_NS),
        vec![TouchOut::ContextMenu([70.0, 80.0])]
    );
    assert!(t.tick(LONG_PRESS_NS * 2).is_empty(), "abriu duas vezes");
    assert!(
        t.feed(1, TouchPhase::End, [70.0, 80.0], LONG_PRESS_NS * 2, None)
            .is_empty()
    );
    let mut c = g();
    c.feed(1, TouchPhase::Start, [70.0, 80.0], 0, None);
    c.feed(1, TouchPhase::Move, [120.0, 80.0], MS, None);
    assert!(
        c.tick(LONG_PRESS_NS).is_empty(),
        "o controlo: um traço abriu o menu"
    );
}

/// ⭐ **Dois dedos que ANDAM navegam: o ponto médio arrasta, o afastamento dá zoom — e não desfazem.**
///
/// Os dois lados da fronteira (a folga do UIKit) e o controlo: um contacto de dois dedos que não
/// andou além dela e levanta depressa desfaz; o que andou navega e levantar não desfaz.
#[test]
fn two_travelling_fingers_pan_and_pinch_and_never_undo() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [100.0, 100.0], 0, None);
    t.feed(2, TouchPhase::Start, [200.0, 100.0], MS, None);
    // Dentro da folga: nada anda.
    assert!(
        t.feed(1, TouchPhase::Move, [104.0, 100.0], 2 * MS, None)
            .is_empty(),
        "dentro da folga já navegou"
    );
    assert!(t.tick(2 * MS).is_empty(), "dentro da folga já navegou");
    // Além da folga: a referência semeia-se SEM salto no QUADRO — o botão do meio pousa, nada
    // anda ainda. (O evento em si nunca publica navegação: só o `tick`.)
    assert!(
        t.feed(1, TouchPhase::Move, [120.0, 100.0], 3 * MS, None)
            .is_empty(),
        "o evento publicou navegação; a lei é uma vez por quadro"
    );
    assert_eq!(t.tick(3 * MS), vec![TouchOut::NavBegin([160.0, 100.0])]);
    // Os DOIS dedos andam 40 pt no mesmo quadro: um pan puro de 40 — sem o zoom espúrio que
    // publicar a cada evento inventaria (cada dedo sozinho muda o afastamento).
    t.feed(2, TouchPhase::Move, [240.0, 100.0], 4 * MS, None);
    t.feed(1, TouchPhase::Move, [160.0, 100.0], 4 * MS, None);
    assert_eq!(t.tick(4 * MS), vec![TouchOut::Pan([40.0, 0.0])]);
    // Pinça: o dedo 2 afasta-se 40 pt — o meio anda 20 (pan) e o afastamento vai de 40 a 60 (o
    // factor é 40/60: aproxima), à volta do meio NOVO.
    t.feed(2, TouchPhase::Move, [280.0, 100.0], 5 * MS, None);
    assert_eq!(
        t.tick(5 * MS),
        vec![
            TouchOut::Pan([20.0, 0.0]),
            TouchOut::Zoom {
                factor: 40.0 / 60.0,
                about: [220.0, 100.0],
            }
        ]
    );
    // Levantar depressa NÃO desfaz: só o botão do meio levanta — e levanta no PRIMEIRO dedo a
    // sair (um dedo sozinho não navega; o botão do meio não fica pousado à espera).
    assert_eq!(
        t.feed(1, TouchPhase::End, [160.0, 100.0], 6 * MS, None),
        vec![TouchOut::NavEnd],
        "o botão do meio ficou pousado com um dedo só"
    );
    assert!(
        t.feed(2, TouchPhase::End, [280.0, 100.0], 7 * MS, None)
            .is_empty(),
        "um pan que levantou desfez"
    );
    assert!(t.tick(8 * MS).is_empty(), "navegou depois de levantar");

    // O CONTROLO, do outro lado da fronteira: o mesmo contacto, dentro da folga, desfaz.
    let mut c = g();
    c.feed(1, TouchPhase::Start, [100.0, 100.0], 0, None);
    c.feed(2, TouchPhase::Start, [200.0, 100.0], MS, None);
    assert!(
        c.feed(1, TouchPhase::Move, [106.0, 100.0], 2 * MS, None)
            .is_empty()
    );
    assert!(c.tick(2 * MS).is_empty());
    let mut out = c.feed(1, TouchPhase::End, [106.0, 100.0], 7 * MS, None);
    out.extend(c.feed(2, TouchPhase::End, [200.0, 100.0], 8 * MS, None));
    assert_eq!(
        out,
        vec![TouchOut::Undo],
        "um toque que não andou não desfez"
    );
}

/// ⚠️ **Um dedo a mais ou a menos a meio da navegação não salta**: a referência semeia-se outra vez.
#[test]
fn a_finger_joining_or_leaving_mid_navigation_does_not_jump() {
    let mut t = g();
    t.feed(1, TouchPhase::Start, [100.0, 100.0], 0, None);
    t.feed(2, TouchPhase::Start, [201.0, 100.0], MS, None);
    t.feed(1, TouchPhase::Move, [120.0, 100.0], 2 * MS, None);
    assert_eq!(t.tick(2 * MS), vec![TouchOut::NavBegin([160.5, 100.0])]);
    // O 3.º dedo pousa longe: o ponto médio muda sem ninguém andar.
    t.feed(3, TouchPhase::Start, [501.0, 500.0], 3 * MS, None);
    t.feed(3, TouchPhase::Move, [501.0, 500.0], 4 * MS, None);
    assert!(t.tick(4 * MS).is_empty(), "o 3.º dedo fez a vista saltar");
    // (120 + 201 + 501) / 3 = 274 → (120 + 201 + 531) / 3 = 284: exacto em f32.
    t.feed(3, TouchPhase::Move, [531.0, 500.0], 5 * MS, None);
    assert_eq!(t.tick(5 * MS).first(), Some(&TouchOut::Pan([10.0, 0.0])));
    // Um dedo sai, dois ficam: ainda a navegar, sem salto.
    assert!(
        t.feed(3, TouchPhase::End, [531.0, 500.0], 6 * MS, None)
            .is_empty(),
        "sair do 3.º dedo acabou a navegação com dois dedos no vidro"
    );
    t.feed(1, TouchPhase::Move, [120.0, 100.0], 7 * MS, None);
    assert!(
        t.tick(7 * MS).is_empty(),
        "o dedo que saiu fez a vista saltar"
    );
    t.feed(1, TouchPhase::Move, [130.0, 100.0], 8 * MS, None);
    assert_eq!(t.tick(8 * MS).first(), Some(&TouchOut::Pan([5.0, 0.0])));
}

/// **Os contactos sintéticos (a cena de smoke) saem pelo `tick`**, a porta por onde a shell os aplica.
#[test]
fn synthetic_contacts_are_delivered_by_the_tick() {
    let mut t = g();
    t.feed_synthetic(1, TouchPhase::Start, [100.0, 100.0], 0);
    t.feed_synthetic(2, TouchPhase::Start, [200.0, 100.0], MS);
    t.feed_synthetic(1, TouchPhase::Move, [120.0, 100.0], 2 * MS);
    t.feed_synthetic(1, TouchPhase::Move, [140.0, 100.0], 3 * MS);
    // O quadro semeia com os dedos onde ESTÃO (140 e 200): meio 170, sem salto.
    assert_eq!(t.tick(4 * MS), vec![TouchOut::NavBegin([170.0, 100.0])]);
    t.feed_synthetic(1, TouchPhase::Move, [150.0, 100.0], 5 * MS);
    assert_eq!(
        t.tick(5 * MS),
        vec![
            TouchOut::Pan([5.0, 0.0]),
            TouchOut::Zoom {
                factor: 30.0 / 25.0,
                about: [175.0, 100.0],
            }
        ]
    );
    assert!(t.tick(6 * MS).is_empty(), "o tick entregou duas vezes");
    // E os px viram pt pela escala anotada: a mesma posição, a 2 px/pt, é metade.
    let mut h = g();
    h.note_scale(2.0);
    h.feed_synthetic(1, TouchPhase::Start, [100.0, 100.0], 0);
    h.feed_synthetic(2, TouchPhase::Start, [200.0, 100.0], MS);
    h.feed_synthetic(1, TouchPhase::Move, [140.0, 100.0], 2 * MS);
    assert_eq!(h.tick(2 * MS), vec![TouchOut::NavBegin([85.0, 50.0])]);
}

/// ⭐ **O widget: segurar = segurar, tocar = travar — e o dedo nele não conta como dedo extra.**
#[test]
fn the_widget_holds_while_pressed_and_locks_on_a_tap() {
    let shift = crate::ids::TOUCH_MOD_SHIFT;
    let mut t = g();
    assert!(!t.widget_visible(), "o widget apareceu sem toque");
    t.feed(9, TouchPhase::Start, [5.0, 5.0], 0, Some(shift));
    assert!(t.modifiers().shift, "segurar o Shift não o segura");
    t.feed(9, TouchPhase::End, [5.0, 5.0], LONG_PRESS_NS, None);
    assert_eq!(t.locked(), TouchMods::default(), "segurar travou");
    assert!(!t.modifiers().shift);
    t.feed(9, TouchPhase::Start, [5.0, 5.0], 0, Some(shift));
    t.feed(9, TouchPhase::End, [5.0, 5.0], 90 * MS, None);
    assert!(t.locked().shift, "tocar não travou");
    // Com o Shift preso por um dedo, outro dedo desenha sozinho — não é gesto.
    t.feed(
        8,
        TouchPhase::Start,
        [5.0, 5.0],
        0,
        Some(crate::ids::TOUCH_MOD_ALT),
    );
    t.feed(1, TouchPhase::Start, [300.0, 300.0], 0, None);
    let out = t.feed(1, TouchPhase::Move, [340.0, 300.0], MS, None);
    assert_eq!(out.first(), Some(&TouchOut::Press([300.0, 300.0])));
    assert!(t.modifiers().shift && t.modifiers().alt);
    t.toggle_lock(shift);
    assert!(!t.locked().shift, "o clique não destravou");
}

/// A MÃO sintética da F8: dois dedos em volta de `(400, 300)`, afastados `d0` → `d1` pt e com o meio
/// a andar `andar`, em `dur` ms (um quadro a cada 16 ms, o `tick` em cada um); levantam no fim, o
/// segundo 8 ms depois do primeiro. Devolve tudo o que o reconhecedor pediu.
fn mao(dur: u128, d0: f32, d1: f32, andar: [f32; 2]) -> Vec<TouchOut> {
    let mut t = g();
    let mut out = Vec::new();
    let em = |k: f32| {
        let d = d0 + (d1 - d0) * k;
        let m = [400.0 + andar[0] * k, 300.0 + andar[1] * k];
        ([m[0] - d / 2.0, m[1]], [m[0] + d / 2.0, m[1]])
    };
    let (a, b) = em(0.0);
    out.extend(t.feed(1, TouchPhase::Start, a, 0, None));
    out.extend(t.feed(2, TouchPhase::Start, b, MS, None));
    let mut ms = 16;
    while ms <= dur {
        #[allow(clippy::cast_precision_loss)]
        let (a, b) = em(ms as f32 / dur as f32);
        out.extend(t.feed(1, TouchPhase::Move, a, ms * MS, None));
        out.extend(t.feed(2, TouchPhase::Move, b, ms * MS, None));
        out.extend(t.tick(ms * MS));
        ms += 16;
    }
    let (a, b) = em(1.0);
    out.extend(t.feed(1, TouchPhase::End, a, dur * MS, None));
    out.extend(t.feed(2, TouchPhase::End, b, (dur + 8) * MS, None));
    out.extend(t.tick((dur + 16) * MS));
    out
}

fn rapido(out: &[TouchOut]) -> Option<bool> {
    out.iter().find_map(|o| match o {
        TouchOut::QuickPinch { closing } => Some(*closing),
        _ => None,
    })
}

/// ⭐ **O beliscão RÁPIDO enquadra; a pinça LENTA continua a zoomar** (F8). O rápido é a MESMA pinça
/// (o zoom vivo acontece igual) com um pedido a mais no fim; juntar os dedos = *enquadrar tudo*,
/// afastá-los = a vista anterior. E nunca desfaz.
#[test]
fn a_quick_pinch_asks_for_the_frame_and_a_slow_one_only_zooms() {
    let r = mao(200, 200.0, 60.0, [0.0, 0.0]);
    assert_eq!(rapido(&r), Some(true), "{r:?}");
    assert!(r.iter().any(|o| matches!(o, TouchOut::Zoom { .. })), "a pinça viva sumiu");
    assert!(!r.contains(&TouchOut::Undo));
    assert_eq!(rapido(&mao(200, 60.0, 200.0, [0.0, 0.0])), Some(false));
    // A pinça da cena 11 (F6): 1 s — só zoom.
    let lenta = mao(1000, 200.0, 60.0, [0.0, 0.0]);
    assert_eq!(rapido(&lenta), None);
    assert!(lenta.iter().any(|o| matches!(o, TouchOut::Zoom { .. })));
}

/// Os inimigos do gesto novo: um pan rápido com um pouco de pinça (o meio andou mais do que o
/// afastamento mudou), uma pinça rápida que não passou da folga, e um contacto cancelado.
#[test]
fn a_quick_pan_or_a_pinch_inside_the_slop_is_not_a_quick_pinch() {
    assert_eq!(rapido(&mao(200, 100.0, 140.0, [200.0, 0.0])), None, "pan rápido");
    assert_eq!(rapido(&mao(200, 100.0, 115.0, [0.0, 0.0])), None, "dentro da folga");
    let mut t = g();
    t.feed(1, TouchPhase::Start, [300.0, 300.0], 0, None);
    t.feed(2, TouchPhase::Start, [500.0, 300.0], MS, None);
    t.feed(1, TouchPhase::Move, [360.0, 300.0], 16 * MS, None);
    t.feed(2, TouchPhase::Move, [440.0, 300.0], 16 * MS, None);
    t.tick(16 * MS);
    t.feed(1, TouchPhase::Move, [380.0, 300.0], 32 * MS, None);
    t.feed(2, TouchPhase::Move, [420.0, 300.0], 32 * MS, None);
    t.tick(32 * MS);
    t.feed(1, TouchPhase::Cancel, [380.0, 300.0], 40 * MS, None);
    let out = t.feed(2, TouchPhase::Cancel, [420.0, 300.0], 48 * MS, None);
    assert_eq!(rapido(&out), None, "um contacto cancelado enquadrou");
}

/// ⭐ **A fronteira rápido × lento é a do toque-e-segurar** (`LONG_PRESS_NS`, o `minimumPressDuration`
/// do UIKit — a mesma que já separa o toque de 2 dedos de um segurar): medida com a mão sintética,
/// varrendo a duração do gesto inteiro. Nenhum número novo.
#[test]
fn the_quick_slow_border_is_the_long_press() {
    let mut tabela = Vec::new();
    for dur in (100..=1200).step_by(50) {
        let q = rapido(&mao(dur, 200.0, 60.0, [0.0, 0.0])).is_some();
        tabela.push((dur, q));
        // O gesto acaba no 2.º dedo, 8 ms depois do `dur`.
        assert_eq!(q, (dur + 8) * MS < LONG_PRESS_NS, "{dur} ms: {tabela:?}");
    }
    assert!(tabela.iter().any(|&(_, q)| q) && tabela.iter().any(|&(_, q)| !q));
}
