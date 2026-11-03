//! ⭐ **CartoesUI C4: o cadeado do cartão Objeto do Inspector** (maquete C: ícone · nome · olho ·
//! cadeado). É o MESMO id do cadeado da linha da Hierarquia (`hier_lock_companion(linha)`) — um
//! comando, um id —, e o clique chega ao mesmo pedido (`HierRequest::ToggleLock`), no hero montado.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorTransformInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const BITS: u64 = 0x0000_0001_0000_0007;
const LINHA: NodeId = NodeId(100_007);

fn pinta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..8 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
    }
}

fn clique(h: &mut HeroScreen, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

fn hero_com_um_objeto(trancado: bool) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    ph2d_panel_inspector::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: BITS,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
    let mut linha = ph2d_editor_core::screens::hero::fixture::hierarchy()
        .into_iter()
        .next()
        .expect("a fixtura tem uma linha");
    linha.name = "Path 0".into();
    linha.entity = Some(BITS);
    linha.locked = trancado;
    ph2d_panel_hierarchy::sync_from_hierarchy(
        &mut h.store,
        &[LINHA],
        std::iter::once((LINHA, linha)).collect(),
    );
    pinta(&mut h);
    h
}

/// O cadeado aparece no cartão Objeto com o id da linha, e o clique pede `ToggleLock` dessa linha.
#[test]
fn the_object_card_lock_toggles_the_hierarchy_row_lock() {
    let mut h = hero_com_um_objeto(false);
    let id = ph2d_editor_core::ids::hier_lock_companion(LINHA);
    let r = h
        .hit_index
        .rect_for(id)
        .expect("o cartão Objeto não pintou o cadeado");
    let insp = h
        .store
        .panel_rect(ph2d_editor_core::ids::INSP_PANEL)
        .expect("o Inspector publicou o rect");
    assert!(
        insp.x <= r.x && r.x + r.w <= insp.x + insp.w,
        "o cadeado {r:?} não está no Inspector {insp:?}"
    );
    h.bus.drain().for_each(drop);
    clique(&mut h, r.x + r.w * 0.5, r.y + r.h * 0.5);
    let pedidos: Vec<EditorAction> = h.bus.drain().collect();
    assert!(
        pedidos.iter().any(|a| matches!(
            a,
            EditorAction::Hierarchy(HierRequest::ToggleLock { row }) if *row == LINHA
        )),
        "o clique no cadeado não pediu ToggleLock da linha: {pedidos:?}"
    );
}

/// Sem linha na Hierarquia para o objecto, não há cadeado (um cadeado que não tranca é mentira).
#[test]
fn without_a_hierarchy_row_there_is_no_lock() {
    let mut h = hero_com_um_objeto(true);
    h.store.set_hierarchy_entity_rows(Default::default());
    pinta(&mut h);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::hier_lock_companion(LINHA))
            .is_none(),
        "cadeado pintado sem linha"
    );
}
