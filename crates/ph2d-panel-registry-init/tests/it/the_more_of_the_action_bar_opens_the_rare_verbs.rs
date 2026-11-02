//! ⭐ **F50 — o «⋯» da barra de ações ④** (dono 02/11: «os mais comuns na barra de ações e "..."
//! na barra de ações para as menos comuns»): um ponteiro real abre o menu por baixo do «⋯», cada
//! linha tem nome, escolher uma chega à porta do painel dono e fecha o menu; o 2.º clique no «⋯»
//! fecha-o.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn clica(hero: &mut HeroScreen, r: Rect) {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let p = |kind| PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let mut ev = hero.handle_pointer(p(PointerKind::Down), &arena).to_vec();
    ev.extend_from_slice(hero.handle_pointer(p(PointerKind::Up), &arena));
    for e in ev {
        hero.apply_event(e);
    }
}

fn publica(hero: &mut HeroScreen, sel: usize) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode: DrawMode::Select,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(sel);
    let area = hero.last_content;
    let at = Rect::new(area.x + area.w * 0.4, area.y + area.h * 0.6, 120.0, 90.0);
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, Some(at), Vec::new());
    hero.store.set_tool_bars(Some(bars));
    hero.store.set_area_commands(Vec::new(), Vec::new());
}

#[test]
fn the_dots_open_the_rare_verbs_and_a_pick_reaches_the_vector() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    publica(&mut hero, 2);
    pinta(&mut hero, &mut ts);
    let dots = hero
        .hit_index
        .rect_for(ids::SELECTION_ACTIONS_MORE)
        .expect("o «⋯» está pintado na ④");
    clica(&mut hero, dots);
    assert_eq!(
        hero.store.context_menu().map(|m| m.kind),
        Some(ContextMenuKind::SelectionActionsMore),
        "o «⋯» não abriu o menu"
    );
    pinta(&mut hero, &mut ts);
    let trim = hero
        .hit_index
        .rect_for(ph2d_tool_vector::ids::VECTOR_BOOL_TRIM)
        .expect("o Trim está no menu");
    assert!(trim.y >= dots.y + dots.h, "o menu abre POR BAIXO do «⋯»");
    let _ = hero.bus.drain().count();
    clica(&mut hero, trim);
    assert!(
        hero.bus.drain().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if i == ph2d_tool_vector::ids::VECTOR_BOOL_TRIM
        )),
        "o Trim foi escolhido e não chegou ao vetor"
    );
    assert!(hero.store.context_menu().is_none(), "o menu ficou aberto");

    // O 2.º clique no «⋯» fecha o menu que o 1.º abriu.
    hero.apply_event(WidgetEvent::Click(ids::SELECTION_ACTIONS_MORE));
    assert!(hero.store.context_menu().is_some());
    hero.apply_event(WidgetEvent::Click(ids::SELECTION_ACTIONS_MORE));
    assert!(
        hero.store.context_menu().is_none(),
        "o 2.º clique não fechou"
    );
}

/// Sem nada de raro para oferecer, não há «⋯» — um botão que abre um menu vazio é pior que nenhum.
#[test]
fn no_rare_verbs_no_dots() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    publica(&mut hero, 2);
    let mut bars = hero.store.tool_bars().cloned().unwrap();
    bars.actions_more.clear();
    hero.store.set_tool_bars(Some(bars));
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index
            .rect_for(ids::SELECTION_ACTIONS_PIN)
            .is_some(),
        "controlo: a ④ está pintada"
    );
    assert!(
        hero.hit_index
            .rect_for(ids::SELECTION_ACTIONS_MORE)
            .is_none()
    );
}
