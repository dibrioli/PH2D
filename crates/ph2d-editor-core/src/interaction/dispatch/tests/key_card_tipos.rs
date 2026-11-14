//! Os tipos de alça no editor da curva Custom (dono 13/11: «os tipos de alça como no módulo
//! vector»): agarrar um ponto escolhe-o; Corner · Smooth · Symmetric trocam o dele num passo de
//! undo; sem ponto escolhido não fazem nada; Alt + agarrar uma alça quebra o ponto em Corner.

use super::*;
use crate::ids as c;
use crate::interaction::state::{BezAlvo, BezArea};
use crate::interaction::{ContextMenuKind, ContextMenuRequest, TimelineInterpScope};
use ph2d_anim::{EaseSpline, HandleKind, SplineSide};
use ph2d_host::PointerButton;

const SCOPE: TimelineInterpScope = TimelineInterpScope::Key { target: 9, key: 2 };
const AREA: BezArea = BezArea {
    rect: Rect {
        x: 100.0,
        y: 50.0,
        w: 200.0,
        h: 160.0,
    },
    y_min: -0.5,
    y_max: 1.5,
};
const FORA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 10.0,
    h: 10.0,
};

/// O cartão aberto com a curva publicada: um ponto em `u = 0,5`.
fn cartao() -> (WidgetStore, EaseSpline) {
    let mut store = WidgetStore::with_capacity(4);
    store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineSegment { scope: SCOPE },
    });
    let mut curva = EaseSpline::from_bezier(0.42, 0.0, 0.58, 1.0);
    curva.insert(0.5);
    store.set_key_card_curve(Some(curva));
    store.set_key_card_bez_area(Some(AREA));
    (store, curva)
}

fn down(store: &mut WidgetStore, id: NodeId, (x, y): (f32, f32), button: PointerButton) -> bool {
    let ev = PointerEvent {
        button,
        ..pointer(PointerKind::Down, x, y)
    };
    crate::interaction::dispatch::key_card_drag::pega(store, Some((id, FORA)), ev)
}

fn tela(q: [f32; 2]) -> (f32, f32) {
    AREA.tela(q[0], q[1])
}

/// Drena o lote e publica a curva dele de volta, como a timeline faz a cada quadro.
fn drena(store: &mut WidgetStore) -> Option<crate::interaction::TimelineBezierEdit> {
    let e = store.take_key_card_bez_edit()?;
    if e.end {
        store.set_key_card_curve(Some(e.curve));
    }
    Some(e)
}

#[test]
fn the_handle_kind_buttons_change_the_chosen_point_in_one_undo_step() {
    let (mut store, curva) = cartao();
    for (id, ..) in crate::interaction::key_card_alvo::TIPOS {
        assert!(crate::interaction::key_card_alvo::e_alvo_do_cartao(id));
    }
    // Sem ponto escolhido: o botão consome o clique e não muda nada.
    assert!(down(
        &mut store,
        c::CTX_MENU_TL_KEY_HANDLE_CORNER,
        (0.0, 0.0),
        PointerButton::Primary
    ));
    assert_eq!(store.key_card_bez_escolhido(), None);
    assert!(drena(&mut store).is_none());

    // Agarrar o ponto escolhe-o (e soltar mantém a escolha).
    assert!(down(
        &mut store,
        c::CTX_MENU_TL_KEY_BEZ_AREA,
        tela(curva.node(1).p),
        PointerButton::Primary
    ));
    assert_eq!(store.key_card_bez_alvo(), Some(BezAlvo::Ponto(1)));
    store.end_key_card_bez();
    drena(&mut store);
    assert_eq!(store.key_card_bez_escolhido(), Some(1));

    for (id, tipo) in [
        (c::CTX_MENU_TL_KEY_HANDLE_SYMM, HandleKind::Symmetric),
        (c::CTX_MENU_TL_KEY_HANDLE_CORNER, HandleKind::Corner),
        (c::CTX_MENU_TL_KEY_HANDLE_SMOOTH, HandleKind::Smooth),
    ] {
        assert!(down(&mut store, id, (0.0, 0.0), PointerButton::Primary));
        let e = drena(&mut store).expect("um lote");
        assert!(e.begin && e.end, "um clique = um passo de undo");
        assert_eq!((e.scope, e.curve.kind(1)), (SCOPE, Some(tipo)));
        assert_eq!(store.key_card_curve().unwrap().kind(1), Some(tipo));
    }
    // O tipo que já é: nada a desfazer.
    assert!(down(
        &mut store,
        c::CTX_MENU_TL_KEY_HANDLE_SMOOTH,
        (0.0, 0.0),
        PointerButton::Primary
    ));
    assert!(drena(&mut store).is_none());

    // Apagar o ponto larga a escolha.
    let p = store.key_card_curve().unwrap().node(1).p;
    down(
        &mut store,
        c::CTX_MENU_TL_KEY_BEZ_AREA,
        tela(p),
        PointerButton::Secondary,
    );
    drena(&mut store);
    assert_eq!(store.key_card_bez_escolhido(), None);
}

#[test]
fn alt_grabbing_a_handle_breaks_the_point_into_a_corner() {
    let (mut store, curva) = cartao();
    let h = curva.node(1).h_out;
    // Sem Alt: o ponto continua Smooth.
    down(
        &mut store,
        c::CTX_MENU_TL_KEY_BEZ_AREA,
        tela(h),
        PointerButton::Primary,
    );
    assert_eq!(
        store.key_card_bez_alvo(),
        Some(BezAlvo::Alca(1, SplineSide::Out))
    );
    assert_eq!(
        store.key_card_bez_escolhido(),
        Some(1),
        "a alça escolhe o dono"
    );
    assert_eq!(
        store.key_card_curve().unwrap().kind(1),
        Some(HandleKind::Smooth)
    );
    store.end_key_card_bez();
    drena(&mut store);
    // Com Alt: Corner, e a outra alça fica onde estava ao arrastar.
    store.set_alt_held(true);
    down(
        &mut store,
        c::CTX_MENU_TL_KEY_BEZ_AREA,
        tela(h),
        PointerButton::Primary,
    );
    assert_eq!(
        store.key_card_curve().unwrap().kind(1),
        Some(HandleKind::Corner)
    );
    let h_in = store.key_card_curve().unwrap().node(1).h_in;
    store.move_key_card_bez((0.7, 0.2));
    assert_eq!(store.key_card_curve().unwrap().node(1).h_in, h_in);
    store.end_key_card_bez();
    let e = drena(&mut store).unwrap();
    assert_eq!(e.curve.kind(1), Some(HandleKind::Corner));
}
