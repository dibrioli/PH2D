//! ⭐⭐⭐ **As barras do vetor respondem a um PONTEIRO real** (D10, 07/10).
//!
//! Com todos os painéis registados (a porta do quadro), o produtor do vetor publica as barras, o
//! ecrã pinta-as, e um Down+Up sobre cada botão pintado tem de virar `Click` e chegar ao barramento
//! pela porta do painel — o caminho inteiro, de ponta a ponta: *pintado ⇒ registado ⇒ despachado*.
//!
//! E o ORÇAMENTO dos tablets (F0 de `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md` §4): a fila ②
//! do caso mais longo (Edit + Node) fica numa linha nos três iPads, e o que vai para o `⋯` é a
//! CAUDA — nunca uma opção do gesto.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarFace, WidgetEvent};
use ph2d_editor_core::screens::hero::float_bar::tool_bars::placed;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::screens::hero::{menu_bar, tool_bar};
use ph2d_editor_core::screens::layout::{
    CenterSplit, ChromeBands, DockSide, DockSides, HeroLayout,
};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::{RailButtonSize, ToolRailEntry, horizontal_lines};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
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

/// O retrato que a crate do vetor publica no painel num quadro.
fn retrato(mode: DrawMode, sel: usize, verts: usize) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(sel);
    ph2d_panel_vector::set_current_vertex_count(verts);
}

fn publica(hero: &mut HeroScreen, mode: DrawMode, sel: usize, verts: usize, at: Option<Rect>) {
    retrato(mode, sel, verts);
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

/// Down+Up no centro de `r`: devolve os eventos que o despacho levantou.
fn clica(hero: &mut HeroScreen, r: Rect) -> Vec<WidgetEvent> {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    ev
}

/// ⭐⭐⭐ Cada botão de ① e ④, em cada ferramenta, responde a um ponteiro real e chega ao barramento.
#[test]
fn every_painted_column_and_action_button_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut medidos = 0usize;
    for &mode in DrawMode::ALL {
        let mut hero = HeroScreen::new(NodeId(1));
        pinta(&mut hero, &mut ts);
        let area = hero.last_content;
        let sel = Rect::new(area.x + area.w * 0.5, area.y + area.h * 0.5, 120.0, 90.0);
        publica(&mut hero, mode, 2, 0, Some(sel));
        pinta(&mut hero, &mut ts);
        let bars = hero.store.tool_bars().cloned().expect("publicadas");
        for (b, r) in placed(&bars, hero.last_content, false, false) {
            if hero.store.is_picker_swatch(b.id) {
                // A amostra abre o seletor no Down — o despacho genérico da cor.
                let _ = clica(&mut hero, r);
                assert_eq!(
                    hero.store.picker_target(),
                    Some(b.id),
                    "{mode:?}: a amostra não abriu a cor"
                );
                hero.store.set_picker_target(None);
                continue;
            }
            let ev = clica(&mut hero, r);
            assert!(
                ev.iter()
                    .any(|e| matches!(e, WidgetEvent::Click(i) if *i == b.id)),
                "{mode:?}: o botão «{}» está pintado e o ponteiro não o clica ({ev:?})",
                b.label
            );
            let _ = hero.bus.drain().count();
            for e in ev {
                hero.apply_event(e);
            }
            let chegou = hero.bus.drain().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if i == b.id),
            ) || hero.store.picker_target().is_some()
                || b.id == ph2d_editor_core::ids::SELECTION_ACTIONS_PIN;
            assert!(
                chegou,
                "{mode:?}: «{}» clicou e não chegou à ferramenta",
                b.label
            );
            hero.store.set_picker_target(None);
            hero.store.set_selection_actions_pinned(false);
            medidos += 1;
        }
    }
    assert!(medidos >= 17 * 20, "controlo: só {medidos} botões medidos");
}

/// ⭐ **Fixar** (D10 decisão 4): fixada, a barra deixa de seguir a seleção e vai ao topo; o 2.º
/// clique devolve-a à seleção. ⛔ E **nada a esconde** (dono 15/10): ela é a única casa do Arrange e
/// do Alinhar — nenhum botão dela tem a cara de fechar.
#[test]
fn the_action_bar_pins_and_never_hides() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = HeroScreen::new(NodeId(1));
    pinta(&mut hero, &mut ts);
    let area = hero.last_content;
    let sel = Rect::new(area.x + area.w * 0.4, area.y + area.h * 0.7, 120.0, 90.0);
    publica(&mut hero, DrawMode::Select, 2, 0, Some(sel));
    pinta(&mut hero, &mut ts);
    let bars = hero.store.tool_bars().cloned().expect("publicadas");
    let fecha = placed(&bars, area, false, false)
        .into_iter()
        .filter(|(b, _)| matches!(b.face, BarFace::Icon(IconId::Close)))
        .count();
    assert_eq!(fecha, 0, "a ④ voltou a ter um botão de esconder");
    let pin = hero
        .hit_index
        .rect_for(ph2d_editor_core::ids::SELECTION_ACTIONS_PIN)
        .expect("o fixar está pintado");
    assert!(pin.y < sel.y, "a barra flutua por CIMA da seleção");
    for e in clica(&mut hero, pin) {
        hero.apply_event(e);
    }
    assert!(hero.store.selection_actions_pinned());
    pinta(&mut hero, &mut ts);
    let fixa = hero
        .hit_index
        .rect_for(ph2d_editor_core::ids::SELECTION_ACTIONS_PIN)
        .expect("fixada continua lá");
    assert!(
        fixa.y < area.y + 3.0 * fixa.h,
        "fixada mora no topo da área ({} contra {})",
        fixa.y,
        area.y
    );
    // ⚠️ O 2.º clique pelo despacho, e não pelo ponteiro: no mesmo sítio, logo a seguir, o
    // ponteiro lê-o como DUPLO clique.
    hero.apply_event(WidgetEvent::Click(
        ph2d_editor_core::ids::SELECTION_ACTIONS_PIN,
    ));
    assert!(!hero.store.selection_actions_pinned(), "o 2.º clique não a soltou");
}

/// Os três tablets (os mesmos do `the_chrome_never_eats_more_of_a_tablet_than_this`).
const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

fn area_w(w: f32, h: f32) -> f32 {
    let bands = ChromeBands {
        rail_w: 0.0,
        top_bar_h: menu_bar::MENU_BAR_H,
        tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
        left_dock_w: ChromeBands::default_dock_w(DockSide::Left, w),
        right_dock_w: ChromeBands::default_dock_w(DockSide::Right, w),
        ..ChromeBands::DEFAULT
    };
    HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, w, h),
        false,
        bands,
        CenterSplit::None,
        DockSides::BOTH,
    )
    .draw_area
    .w
}

/// ⭐⭐ **O orçamento dos tablets vira teste** (F0): as filas mais longas — Edit + Node com dois
/// caminhos e nós escolhidos, e o Lápis com a simetria radial ligada — ficam numa LINHA nos três
/// iPads, e o que vai para o `⋯` é só o que lá FUNCIONA: nenhum CAMPO (lá não se edita) e nenhuma
/// AMOSTRA (lá não abre a cor). A ordem é a prioridade (`bars.rs`).
#[test]
fn the_longest_vector_rows_fit_one_line_and_only_operable_entries_overflow() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut transbordou_algures = false;
    for (caso, mode, sym) in [
        ("Edit + Node", DrawMode::Node, false),
        ("Lápis + simetria radial", DrawMode::Pencil, true),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        publica(&mut hero, mode, 2, 2, None);
        if sym {
            ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
                mode,
                symmetry: ph2d_symmetry::SymmetryStyle {
                    on: true,
                    kind: ph2d_symmetry::SymmetryKind::Radial,
                    ..ph2d_symmetry::SymmetryStyle::default()
                },
                ..VectorStyleSnapshot::default()
            }));
            let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, None, Vec::new());
            hero.store.set_tool_bars(Some(bars));
        }
        let n = hero
            .store
            .tool_bars()
            .map_or(0, |b| b.row_lead.len() + b.row_tail.len());
        assert!(n >= 10, "controlo: {caso} tem só {n} entradas");
        for (name, w, h) in TABLETS {
            let aw = area_w(w, h);
            let (rail, over) = tool_bar::bar_split(&hero.store, &mut ts, false, false, aw);
            // A largura em que os chips CORREM é a do produto (`content_rect`), nunca um `aw − 16`
            // escrito à mão: o respiro é `2 × Spacing::Xs`, e o `Xs` não é `8`.
            let content = tool_bar::content_rect(Rect::new(0.0, 0.0, aw, h)).w;
            let lines = horizontal_lines(&rail, content, RailButtonSize::Small);
            println!(
                "{caso:24} {name:10} área {aw:6.1} · na fila {} · atrás do ⋯ {}",
                rail.entries.len(),
                over.len()
            );
            assert_eq!(
                lines, 1,
                "{caso} · {name}: a fila quebrou em {lines} linhas"
            );
            let mortas: Vec<String> = over
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        ToolRailEntry::Number { .. } | ToolRailEntry::Swatch { .. }
                    )
                })
                .filter_map(|e| e.label().map(str::to_owned))
                .collect();
            assert!(
                mortas.is_empty(),
                "{caso} · {name}: foram para o ⋯ e lá não funcionam: {mortas:?}"
            );
            transbordou_algures |= !over.is_empty();
        }
    }
    assert!(
        transbordou_algures,
        "controlo: nada transbordou — a prioridade não foi medida"
    );
}
