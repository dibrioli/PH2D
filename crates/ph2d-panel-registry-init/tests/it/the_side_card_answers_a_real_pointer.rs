//! ⭐⭐⭐ **O cartão lateral responde a um PONTEIRO real** (F10, pedido do dono de 11/10).
//!
//! Os campos sem alça vão a um cartão JUNTO da forma enquanto as alças estão à vista (modo Forma +
//! forma viva). Com todos os painéis registados, o produtor publica, o ecrã pinta, e um Down+Up na
//! ESCOLHA do cartão cicla o valor no store e sai no barramento pelo id do campo — o caminho
//! inteiro: *pintado ⇒ registado ⇒ despachado*. E o cartão some no Select e com o latch armado.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::float_bar::tool_bars::{placed, placed_card};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{RailAxis, entry_rects};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_tool_vector::ids::{vector_shape_choice_id, vector_shape_field_id};
use ph2d_tool_vector::shapes::{FieldUnit, SHAPES, next_choice};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

/// A primeira forma do catálogo com uma ESCOLHA que nenhuma alça move: `(índice da forma, campo)`.
fn forma_com_escolha_sem_alca() -> (usize, usize) {
    SHAPES
        .iter()
        .enumerate()
        .find_map(|(s, d)| {
            ph2d_tool_vector::shape_handles::fields_without_handle(d.kind)
                .find(|&i| matches!(d.fields[i].unit, FieldUnit::Choice(_)))
                .map(|i| (s, i))
        })
        .expect("controlo: nenhuma forma tem uma escolha sem alça")
}

/// Publica o quadro: o modo, a forma viva (`None` = latch armado / não é forma) e a seleção.
fn publica(hero: &mut HeroScreen, mode: DrawMode, live: Option<usize>, sel: Rect) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(1);
    ph2d_panel_vector::set_current_shape_focus(live.map(|s| SHAPES[s].kind));
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, Some(sel), Vec::new());
    hero.store.set_tool_bars(Some(bars));
}

/// O rect da escolha `id` no cartão deste quadro, pela MESMA conta que o pintor.
fn no_cartao(hero: &HeroScreen, ts: &mut ph2d_text::TextSystem, id: NodeId) -> Option<Rect> {
    let bars = hero.store.tool_bars()?.clone();
    let area = hero.last_content;
    let all = placed(&bars, area, false, false, false);
    let size = hero.store.rail_button_size();
    let card = placed_card(&bars, area, &all, size, ts)?;
    entry_rects(&card.rail, card.content, size, RailAxis::Horizontal)
        .into_iter()
        .find(|s| s.id == Some(id))
        .map(|s| s.rect)
}

#[test]
fn the_side_card_choice_cycles_the_value_under_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (s, i) = forma_com_escolha_sem_alca();
    let (choice, field) = (vector_shape_choice_id(i), vector_shape_field_id(i));
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    let area = hero.last_content;
    let sel = Rect::new(area.x + area.w * 0.4, area.y + area.h * 0.4, 120.0, 90.0);
    publica(&mut hero, DrawMode::Shape, Some(s), sel);
    pinta(&mut hero, &mut ts);
    let r = no_cartao(&hero, &mut ts, choice).expect("a escolha sem alça está no cartão");
    assert!(
        r.x >= sel.x + sel.w,
        "o cartão fica à direita da seleção ({r:?})"
    );
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        hero.hit_index.hit(x, y),
        Some(choice),
        "o rect do cartão não está registado"
    );
    let antes = hero.store.number_value(field).unwrap_or(0.0);
    let esperado = next_choice(SHAPES[s].kind, i, antes).expect("é uma escolha");
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == choice)),
        "o ponteiro no cartão não clica a escolha ({ev:?})"
    );
    let _ = hero.bus.drain().count();
    for e in ev {
        hero.apply_event(e);
    }
    let saiu = hero.bus.drain().any(|a| {
        matches!(a, EditorAction::ToolPanelEvent(PanelEvent::SetValue(f, v))
            if f == field && (v - esperado).abs() < 1e-9)
    });
    assert!(
        saiu,
        "a escolha do cartão não chegou à ferramenta pelo id do campo"
    );
    assert!(
        (hero.store.number_value(field).unwrap_or(f64::NAN) - esperado).abs() < 1e-9,
        "o store não ciclou ({antes} → {esperado})"
    );
}

#[test]
fn the_side_card_lives_only_while_the_handles_are_shown() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (s, i) = forma_com_escolha_sem_alca();
    let choice = vector_shape_choice_id(i);
    for (mode, live, ha) in [
        (DrawMode::Shape, Some(s), true),
        (DrawMode::Select, Some(s), false),
        (DrawMode::Shape, None, false),
        (DrawMode::Node, Some(s), false),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        pinta(&mut hero, &mut ts);
        let area = hero.last_content;
        let sel = Rect::new(area.x + area.w * 0.4, area.y + area.h * 0.4, 120.0, 90.0);
        publica(&mut hero, mode, live, sel);
        let card = !hero.store.tool_bars().expect("publicadas").card.is_empty();
        assert_eq!(
            card,
            ph2d_panel_vector::bars::shape_handles_shown(mode, live.map(|s| SHAPES[s].kind)),
            "{mode:?} live={live:?}: o cartão e as alças não nascem do mesmo predicado"
        );
        assert_eq!(card, ha, "{mode:?} live={live:?}");
        pinta(&mut hero, &mut ts);
        assert_eq!(
            no_cartao(&hero, &mut ts, choice).is_some(),
            ha,
            "{mode:?} live={live:?}: o cartão pintado não segue as alças"
        );
    }
}
