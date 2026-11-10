use super::*;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_host::PointerButton;

fn gesto(phase: GesturePhase, x: f32, y: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::ResizeEdge { edges: 0 },
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods::default(),
    }
}

const CARTAO: Rect = Rect::new(100.0, 500.0, 800.0, 240.0);

/// O topo NA ORIGEM é a costura da banda; fora dela, ou por outra borda, é o cartão.
#[test]
fn o_topo_na_origem_e_a_costura_da_banda_e_o_resto_e_o_cartao() {
    let mut st = TimelinePanelState::default();
    let mut store = WidgetStore::default();
    let b = gesto(GesturePhase::Begin, 400.0, 500.0);
    assert!(!apply_resize(
        &mut st,
        &mut store,
        CARTAO,
        TIMELINE_EDGE_T,
        b
    ));
    assert!(st.resize.is_none(), "a banda é de `resize::apply_resize`");

    assert!(apply_resize(
        &mut st,
        &mut store,
        CARTAO,
        TIMELINE_EDGE_L,
        b
    ));
    let u = gesto(GesturePhase::Update, 460.0, 500.0);
    assert!(apply_resize(
        &mut st,
        &mut store,
        CARTAO,
        TIMELINE_EDGE_L,
        u
    ));
    assert_eq!(
        store.floating_card_offsets(PAINEL),
        ((60.0, 0.0), (-60.0, 0.0))
    );
    let e = gesto(GesturePhase::End, 460.0, 500.0);
    assert!(apply_resize(
        &mut st,
        &mut store,
        CARTAO,
        TIMELINE_EDGE_L,
        e
    ));
    assert!(st.resize.is_none());
    assert!(store.take_section_prefs_dirty(), "o lugar novo grava-se");
}

/// Os cantos registam-se DEPOIS das bordas (o último registado ganha onde se tocam).
#[test]
fn os_cantos_ganham_as_bordas() {
    let a = alcas(CARTAO);
    let (bordas, cantos) = a.split_at(4);
    assert!(bordas.iter().all(|(_, e, _)| e.count_ones() == 1));
    assert!(cantos.iter().all(|(_, e, _)| e.count_ones() == 2));
}
