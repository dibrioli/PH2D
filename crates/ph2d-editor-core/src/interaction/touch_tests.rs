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
    assert_eq!(t.tick(LONG_PRESS_NS - 1), None, "abriu antes da hora");
    assert_eq!(
        t.tick(LONG_PRESS_NS),
        Some(TouchOut::ContextMenu([70.0, 80.0]))
    );
    assert_eq!(t.tick(LONG_PRESS_NS * 2), None, "abriu duas vezes");
    assert!(
        t.feed(1, TouchPhase::End, [70.0, 80.0], LONG_PRESS_NS * 2, None)
            .is_empty()
    );
    let mut c = g();
    c.feed(1, TouchPhase::Start, [70.0, 80.0], 0, None);
    c.feed(1, TouchPhase::Move, [120.0, 80.0], MS, None);
    assert_eq!(
        c.tick(LONG_PRESS_NS),
        None,
        "o controlo: um traço abriu o menu"
    );
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
