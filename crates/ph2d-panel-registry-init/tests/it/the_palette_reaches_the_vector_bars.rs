//! ⭐⭐ **A paleta alcança as barras do vetor** (D10, F4 — 08/10).
//!
//! Com todos os painéis registados e as barras REAIS do vetor publicadas: todo comando que a paleta
//! oferece das barras, escolhido, chega à porta do painel (ou abre o seletor, se é amostra) E pisca
//! numa porta PINTADA neste quadro.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::float_bar::flash;
use ph2d_editor_core::screens::hero::global_palette::{self, global_palette_bars};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

fn publica(hero: &mut HeroScreen, mode: DrawMode, sel: usize, at: Option<Rect>) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(sel);
    ph2d_panel_vector::set_current_vertex_count(0);
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, at, Vec::new());
    hero.store.set_tool_bars(Some(bars));
    hero.store
        .set_area_commands(ph2d_panel_vector::bars::module_menus(), Vec::new());
}

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

/// Um herói com as barras do vetor de `mode` publicadas e pintadas, com uma seleção de duas.
fn com_barras(mode: DrawMode, ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, ts);
    let area = hero.last_content;
    let sel = Rect::new(area.x + area.w * 0.5, area.y + area.h * 0.5, 120.0, 90.0);
    publica(&mut hero, mode, 2, Some(sel));
    pinta(&mut hero, ts);
    hero
}

/// O que a ponte do vetor faz a um clique num interruptor do Snap: publica o estado novo — e a linha
/// do pulldown passa a ter o OUTRO id (o `ON` vira `OFF`). Sem isto o gate não veria o id a trocar.
fn ponte_do_snap(id: NodeId, pos: &mut (bool, bool)) {
    use ph2d_panel_vector::ids as v;
    match id {
        x if x == ids::VECTOR_SNAP_ON => ph2d_panel_vector::set_current_snap(true),
        x if x == v::VECTOR_SNAP_OFF => ph2d_panel_vector::set_current_snap(false),
        x if x == v::VECTOR_SNAP_PATH_ON => pos.0 = true,
        x if x == v::VECTOR_SNAP_PATH_OFF => pos.0 = false,
        x if x == v::VECTOR_SNAP_CROSS_ON => pos.1 = true,
        x if x == v::VECTOR_SNAP_CROSS_OFF => pos.1 = false,
        _ => {}
    }
    ph2d_panel_vector::set_current_snap_position(pos.0, pos.1);
}

/// As portas onde um comando pode morar e que este quadro pintou.
fn porta_acesa(hero: &HeroScreen, id: NodeId) -> bool {
    let mut slots = vec![id, ids::TOOL_BAR_OVERFLOW];
    slots.extend((0..ids::MAX_AREA_MENUS).map(ids::area_menu_button));
    slots.into_iter().any(|s| {
        hero.hit_index.rect_for(s).is_some() && flash::level_at(&hero.store, false, s).is_some()
    })
}

/// ⭐⭐ **TODO comando das barras na paleta: chega à porta do painel E pisca onde mora.**
///
/// ⚠️ Com o CONTROLO: os campos numéricos da fila NÃO estão na paleta (o clique sem Down não os
/// opera) — e há campos, senão a exclusão passaria sobre nada.
#[test]
fn every_bar_command_in_the_palette_lands_and_flashes_where_it_lives() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut medidos = 0usize;
    let mut campos = 0usize;
    for &mode in DrawMode::ALL {
        ph2d_panel_vector::set_current_snap(false);
        ph2d_panel_vector::set_current_snap_position(false, false);
        let hero = com_barras(mode, &mut ts);
        let bars = hero.store.tool_bars().cloned().expect("publicadas");
        let groups = global_palette_bars::groups(&hero.store);
        let items: Vec<(String, NodeId)> = groups
            .iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .map(|i| (i.label.clone(), i.id))
            .collect();
        for e in bars.row_lead.iter().chain(&bars.row_tail) {
            if let ph2d_editor_core::widget::ToolRailEntry::Number { id, .. } = e {
                campos += 1;
                assert!(
                    !items.iter().any(|(_, i)| i == id),
                    "{mode:?}: um campo numérico entrou na paleta como comando"
                );
            }
        }
        for (label, id) in items {
            let mut pos = (false, false);
            ph2d_panel_vector::set_current_snap(false);
            ph2d_panel_vector::set_current_snap_position(false, false);
            let mut hero = com_barras(mode, &mut ts);
            let model = global_palette::build_global_model(&hero);
            assert!(model.is_item(id), "{mode:?}: «{label}» fora do modelo");
            let _ = hero.bus.drain().count();
            let swatch = global_palette_bars::is_swatch(&hero.store, id);
            global_palette::route_global_pick(&mut hero, id);
            let chegou = if swatch {
                hero.store.picker_target() == Some(id)
            } else {
                // A mesma régua do teste de ponteiro real: o `X` abre o seletor em vez de ir ao
                // barramento, e fixar/desligar a ④ são do chrome.
                hero.bus.drain().any(
                    |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if i == id),
                ) || hero.store.picker_target().is_some()
                    || [ids::SELECTION_ACTIONS_PIN, ids::SELECTION_ACTIONS_OFF].contains(&id)
            };
            assert!(
                chegou,
                "{mode:?}: «{label}» escolhido na paleta não chegou ao painel"
            );
            // ⚠️ O quadro seguinte REPUBLICA as barras, como o app faz: uma linha-interruptor troca
            // de id ao ser executada, e o piscar não pode depender do id de antes (a foto de 08/10).
            hero.store.set_picker_target(None);
            ponte_do_snap(id, &mut pos);
            let area = hero.last_content;
            let sel = Rect::new(area.x + area.w * 0.5, area.y + area.h * 0.5, 120.0, 90.0);
            publica(&mut hero, mode, 2, Some(sel));
            pinta(&mut hero, &mut ts);
            assert!(
                porta_acesa(&hero, id),
                "{mode:?}: «{label}» foi escolhido e nada pintado pisca onde ele mora"
            );
            medidos += 1;
        }
    }
    assert!(
        medidos > 100,
        "só {medidos} comandos medidos — a paleta não viu as barras"
    );
    assert!(campos > 0, "o controlo: nenhuma fila tinha campo numérico");
}
