//! **A lista da janela do Input Map rola como a de qualquer painel** (rolagem única, W5 —
//! `docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md`).
//!
//! ⛔ Até 2026-09-29 a janela tinha rolagem própria: um campo no store, um caso especial da roda
//! na shell, e uma barra que era pintada e registada **sem dono no despacho** — ela fazia hover e
//! **não se agarrava**. Desde a porta, a janela publica as mesmas tabelas de todo painel com a chave
//! `ids::INPUT_MAP_SURFACE`, e estes gates entram pelo despacho REAL, como o dedo entra.
//!
//! ⚠️ A 1.ª asserção de cada gate é o CONTROLO: a fixtura TRANSBORDA. Numa janela em que a lista
//! coubesse, «não rolou» seria a resposta certa e o gate passaria por vácuo.

use bumpalo::Bump;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore, dispatch_pointer, dispatch_wheel};
use ph2d_editor_core::screens::hero::chrome;
use ph2d_editor_core::widget::INPUT_MAP_SCROLLBAR_ID;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource, WheelEvent};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Baixa de propósito: é a altura que faz a lista não caber.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 360.0,
};

fn ev(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: t,
    }
}

/// Abre a janela com uma lista longa, pinta-a pelo pintor REAL e publica o que ele devolve — a
/// mesma costura que o quadro do hero faz.
fn painted() -> (WidgetStore, HitIndex, Rect) {
    let mut map = ph2d_input::InputMap::with_player_defaults();
    for i in 0..24 {
        map.create(format!("accao_{i}"));
    }
    let mut store = WidgetStore::with_capacity(256);
    store.open_input_map(VIEWPORT.x + 48.0, VIEWPORT.y + 8.0);
    chrome::sync_input_map_rows(&mut store, &map);
    let mut hit = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let (card, pending) = chrome::paint_input_map_window(
        &mut scene,
        &mut text,
        Theme::Forge,
        &mut hit,
        &store,
        &map,
        VIEWPORT,
    )
    .expect("a janela está aberta");
    pending.publish(&mut store);
    store.set_panel_rect(ids::INPUT_MAP_SURFACE, card);
    (store, hit, card)
}

fn overflows(store: &WidgetStore) -> bool {
    let content = store.panel_content_h(ids::INPUT_MAP_SURFACE).unwrap_or(0.0);
    let visible = store
        .panel_visible_h(ids::INPUT_MAP_SURFACE)
        .unwrap_or(f32::MAX);
    content > visible
}

/// A barra do Input Map **arrasta** — o gesto que ela prometia e não cumpria.
#[test]
fn the_input_map_scrollbar_drags_the_list() {
    let (mut store, hit, _) = painted();
    assert!(
        overflows(&store),
        "a fixtura não transborda — o gate não mediria nada"
    );
    let track = hit
        .iter_registrations()
        .find(|(id, _)| *id == INPUT_MAP_SCROLLBAR_ID)
        .map(|(_, r)| r)
        .expect("com a lista a transbordar a trilha tem de estar registada");
    let x = track.x + track.w * 0.5;
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hit,
        ev(PointerKind::Down, x, track.y + 2.0, 0),
        &arena,
    );
    assert!(
        store.scrollbar_drag().is_some(),
        "carregar na barra do Input Map não armou o arrasto — ela não tem dono no despacho"
    );
    let _ = dispatch_pointer(
        &mut store,
        &hit,
        ev(PointerKind::Move, x, track.y + track.h * 0.5, 16_000_000),
        &arena,
    );
    assert!(
        store.panel_scroll_target(ids::INPUT_MAP_SURFACE) > 0.0,
        "arrastar a barra do Input Map não rolou a lista"
    );
}

/// A roda sobre o cartão rola a lista pelo caminho de todo painel (o `panel_at` apanha o cartão).
#[test]
fn the_wheel_over_the_input_map_scrolls_its_list() {
    let (mut store, _, card) = painted();
    assert!(
        overflows(&store),
        "a fixtura não transborda — o gate não mediria nada"
    );
    assert_eq!(
        store.panel_at(card.x + card.w * 0.5, card.y + card.h * 0.5),
        Some(ids::INPUT_MAP_SURFACE),
        "o cartão não é achado como painel — a roda sobre ele iria ao canvas"
    );
    let arena = Bump::new();
    let _ = dispatch_wheel(
        &mut store,
        WheelEvent {
            x: card.x + card.w * 0.5,
            y: card.y + card.h * 0.5,
            delta_x: 0.0,
            delta_y: -60.0,
            modifiers: ph2d_host::Modifiers::default(),
            timestamp_ns: 0,
        },
        &arena,
    );
    assert!(
        store.panel_scroll_target(ids::INPUT_MAP_SURFACE) > 0.0,
        "a roda sobre o Input Map não rolou a lista"
    );
}
