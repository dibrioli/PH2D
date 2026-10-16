//! ⭐⭐ **A fila ② do vetor é a barra de controle do Illustrator** (F16, dono 15/10 — plano
//! `docs/UI_New_and_Simple/vetor_nova_ui/26_plano_F16_barra_de_controle.md`).
//!
//! Dois portões no herói REAL, com todos os painéis registados: (1) com o vetor na mão, a fila é só
//! a barra dele — nenhum chip genérico (gizmo, espaço, vista, desfazer) nem pulldown de área; (2) o
//! «Stroke ▾» abre, a um ponteiro real, o painel do traço POR BAIXO da fila, cujos controlos
//! respondem e que um clique fora fecha.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{AreaMenu, InteractiveState};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_tool_vector::ids::VECTOR_CAP_ROUND;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

/// Os chips genéricos da fila sem módulo: o gizmo, o espaço, a vista e o desfazer.
const GENERICOS: [NodeId; 8] = [
    ids::TOOL_TRANSLATE,
    ids::TOOL_ROTATE,
    ids::TOOL_SCALE,
    ids::TOOL_PIVOT,
    ids::TOOL_SPACE,
    ids::TOOL_HOME,
    ids::TOOL_UNDO,
    ids::TOOL_REDO,
];

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

/// Down+Up de verdade no ponto, e os eventos levantados entregues ao herói (como o app faz).
fn clica_em(hero: &mut HeroScreen, x: f32, y: f32) {
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        hero.apply_event(e);
    }
}

fn clica(hero: &mut HeroScreen, r: Rect) {
    clica_em(hero, r.x + r.w * 0.5, r.y + r.h * 0.5);
}

/// Um seletor de modo publicado — o 1.º pulldown da fila quando há objecto activo.
fn seletor_de_modo() -> AreaMenu {
    AreaMenu {
        label: "Mode".to_owned(),
        face: "Edit Mode".to_owned(),
        faces: Vec::new(),
        rows: vec![ToolRailEntry::icon(
            ids::TOOL_UNDO,
            "Undo",
            ph2d_editor_core::icons::IconId::Reset,
        )],
    }
}

/// Publica (ou não) as barras do vetor e o seletor de modo, e pinta — o que o app faz por quadro.
fn quadro(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, vetor: bool) {
    quadro_em(hero, ts, vetor, HERO_VIEWPORT_H);
}

/// O [`quadro`] numa janela de altura `altura`.
fn quadro_em(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, vetor: bool, altura: f32) {
    if vetor {
        ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
            mode: DrawMode::Select,
            ..VectorStyleSnapshot::default()
        }));
        ph2d_panel_vector::set_current_selection_count(1);
        ph2d_panel_vector::set_current_vertex_count(0);
        let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, None, Vec::new());
        hero.store.set_tool_bars(Some(bars));
    } else {
        hero.store.set_tool_bars(None);
    }
    hero.store.set_area_commands(Vec::new(), Vec::new());
    hero.store.publish_mode_menu(Some(seletor_de_modo()));
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, altura);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

/// Os ids da fila ② pintados neste quadro (o rect cai dentro da faixa da fila).
fn na_fila(hero: &HeroScreen) -> Vec<NodeId> {
    let bar = hero.last_layout.expect("layout").tool_bar;
    hero.hit_index
        .iter_registrations()
        .filter(|(_, r)| r.y >= bar.y - 0.5 && r.y + r.h <= bar.y + bar.h + 0.5)
        .map(|(id, _)| id)
        .collect()
}

/// ⭐⭐ **Com o vetor na mão, a fila é SÓ a barra de controle** (F16, dono 15/10: *«os botões que
/// estão na barra do topo devem sair»*): nenhum chip do gizmo, do espaço, da vista, do desfazer, e
/// nenhum pulldown de área (o seletor de modo incluído). O CONTROLO: sem módulo, os mesmos chips e
/// o seletor estão lá — senão a ausência passaria sobre uma fila que nunca os teve.
#[test]
fn with_the_vector_in_hand_the_row_is_only_the_control_bar() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let menus: Vec<NodeId> = (0..ids::MAX_AREA_MENUS)
        .map(ids::area_menu_button)
        .collect();

    let mut sem = HeroScreen::new(NodeId(1));
    quadro(&mut sem, &mut ts, false);
    quadro(&mut sem, &mut ts, false);
    let fila = na_fila(&sem);
    for id in GENERICOS {
        assert!(
            fila.contains(&id),
            "controlo: sem módulo, {id:?} devia estar na fila"
        );
    }
    assert!(
        fila.contains(&ids::area_menu_button(0)),
        "controlo: sem módulo, o seletor de modo devia estar na fila"
    );

    let mut com = HeroScreen::new(NodeId(1));
    quadro(&mut com, &mut ts, true);
    quadro(&mut com, &mut ts, true);
    let fila = na_fila(&com);
    assert!(
        fila.contains(&ph2d_panel_vector::ids::VECTOR_BAR_STROKE),
        "controlo: a barra de controle não foi pintada"
    );
    let over: Vec<NodeId> = com
        .store
        .tool_overflow()
        .iter()
        .filter_map(ToolRailEntry::node_id)
        .collect();
    for id in GENERICOS.iter().chain(&menus) {
        assert!(
            !fila.contains(id) && !over.contains(id),
            "{id:?} continua na fila com o vetor na mão"
        );
    }
}

/// ⭐⭐⭐ **O «Stroke ▾» abre o painel do traço POR BAIXO da fila** (F16, dono 15/10: *«a barra de
/// ajustes no topo como no illustrator, principalmente com os ajustes do stroke»*) — a um ponteiro
/// real: o cartão nasce sob a faixa ②, os controlos dele são registados UMA vez (o painel docado
/// pinta só a dica enquanto ele está aberto), uma ponta clicada chega ao barramento SEM o fechar,
/// e um clique no vazio fecha-o — o quadro seguinte já não pinta a ponta fora do painel.
#[test]
fn the_stroke_chip_opens_the_stroke_panel_under_the_bar() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ph2d_panel_vector::set_stroke_paint_kind(Some(ph2d_panel_vector::StrokePaintKind::Solid));
    let mut hero = HeroScreen::new(NodeId(1));
    <HeroScreen as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
        &mut hero, "vector", true,
    );
    quadro(&mut hero, &mut ts, true);
    quadro(&mut hero, &mut ts, true);
    let painel = hero.store.panel_rect(ids::VECTOR_PANEL);
    let dentro_do_painel =
        |r: Rect| painel.is_some_and(|p| r.x >= p.x - 0.5 && r.x + r.w <= p.x + p.w + 0.5);
    // O CONTROLO: fechado, a ponta mora no painel docado (e só lá).
    let antes = hero.hit_index.rect_for(VECTOR_CAP_ROUND);
    assert!(
        antes.is_some_and(dentro_do_painel),
        "controlo: com o painel do traço fechado a ponta devia estar no painel ({antes:?} × {painel:?})"
    );

    let chip = hero
        .hit_index
        .rect_for(ph2d_panel_vector::ids::VECTOR_BAR_STROKE)
        .expect("o «Stroke ▾» está pintado");
    clica(&mut hero, chip);
    assert!(
        matches!(
            hero.store.get(ph2d_panel_vector::ids::VECTOR_BAR_STROKE),
            Some(InteractiveState::Dropdown { open: true, .. })
        ),
        "o clique real não abriu o painel do traço"
    );
    quadro(&mut hero, &mut ts, true);
    quadro(&mut hero, &mut ts, true);
    let bar = hero.last_layout.expect("layout").tool_bar;
    let cap = hero
        .hit_index
        .rect_for(VECTOR_CAP_ROUND)
        .expect("aberto, o painel do traço não pinta as pontas");
    assert!(
        cap.y >= bar.y + bar.h,
        "a ponta não está POR BAIXO da fila ({cap:?} × {bar:?})"
    );
    assert!(
        !dentro_do_painel(cap),
        "a ponta ficou no painel docado: {cap:?}"
    );
    let vezes = hero
        .hit_index
        .iter_registrations()
        .filter(|(id, _)| *id == VECTOR_CAP_ROUND)
        .count();
    assert_eq!(
        vezes, 1,
        "a ponta registada {vezes} vezes com o painel do traço aberto"
    );

    let _ = hero.bus.drain().count();
    clica(&mut hero, cap);
    let chegou = hero.bus.drain().any(|a| {
        matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if i == VECTOR_CAP_ROUND)
    });
    assert!(
        chegou,
        "a ponta clicada no painel do traço não chegou à ferramenta"
    );
    assert!(
        matches!(
            hero.store.get(ph2d_panel_vector::ids::VECTOR_BAR_STROKE),
            Some(InteractiveState::Dropdown { open: true, .. })
        ),
        "um clique DENTRO do painel do traço fechou-o (o light-dismiss não o conhece)"
    );

    quadro(&mut hero, &mut ts, true);
    let area = hero.last_content;
    clica_em(&mut hero, area.x + area.w * 0.9, area.y + area.h * 0.9);
    assert!(
        matches!(
            hero.store.get(ph2d_panel_vector::ids::VECTOR_BAR_STROKE),
            Some(InteractiveState::Dropdown { open: false, .. })
        ),
        "o clique no vazio não fechou o painel do traço"
    );
    quadro(&mut hero, &mut ts, true);
    let depois = hero.hit_index.rect_for(VECTOR_CAP_ROUND);
    assert!(
        depois.is_none_or(dentro_do_painel),
        "fechado, a ponta continua pintada fora do painel: {depois:?}"
    );
    ph2d_panel_vector::set_stroke_paint_kind(None);
}

/// O painel do traço aberto numa janela de altura `altura`: o herói, o rect publicado e os ids que
/// ele regista DENTRO de si.
fn painel_do_traco_aberto(
    ts: &mut ph2d_text::TextSystem,
    altura: f32,
) -> (HeroScreen, Rect, Vec<NodeId>) {
    let mut hero = HeroScreen::new(NodeId(1));
    <HeroScreen as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
        &mut hero, "vector", true,
    );
    quadro_em(&mut hero, ts, true, altura);
    quadro_em(&mut hero, ts, true, altura);
    let chip = hero
        .hit_index
        .rect_for(ph2d_panel_vector::ids::VECTOR_BAR_STROKE)
        .expect("o «Stroke ▾» está pintado");
    clica(&mut hero, chip);
    quadro_em(&mut hero, ts, true, altura);
    quadro_em(&mut hero, ts, true, altura);
    let (id, pop) = hero
        .store
        .dropdown_popover()
        .expect("o painel do traço publicou-se");
    assert_eq!(id, ph2d_panel_vector::ids::VECTOR_BAR_STROKE);
    let dentro = linhas_dentro(&hero, pop);
    (hero, pop, dentro)
}

fn linhas_dentro(hero: &HeroScreen, pop: Rect) -> Vec<NodeId> {
    hero.hit_index
        .iter_registrations()
        .filter(|(id, r)| {
            *id != ids::RAIL_BACKDROP
                && r.x >= pop.x - 0.5
                && r.x + r.w <= pop.x + pop.w + 0.5
                && r.y >= pop.y - 0.5
                && r.y + r.h <= pop.y + pop.h + 0.5
        })
        .map(|(id, _)| id)
        .collect()
}

/// ⭐ **Numa janela baixa o painel do traço ROLA dentro da região** (aberto do handoff F16c §5.1:
/// a altura era cortada à região, mas as linhas pintavam e clicavam por baixo dela, por cima da
/// arte). Cada linha que a janela alta mostra inteira no painel, na baixa ou está DENTRO dele ou
/// não está registada; a roda leva-o até à última. O CONTROLO: a janela baixa transborda mesmo.
#[test]
fn on_a_short_window_the_stroke_panel_scrolls_inside_the_region() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    ph2d_panel_vector::set_stroke_paint_kind(Some(ph2d_panel_vector::StrokePaintKind::Solid));
    let (_, _, todas) = painel_do_traco_aberto(&mut ts, HERO_VIEWPORT_H);
    assert!(
        todas.contains(&VECTOR_CAP_ROUND),
        "controlo: a ponta mora no painel do traço"
    );

    let (mut hero, pop, _) = painel_do_traco_aberto(&mut ts, HERO_VIEWPORT_H * 0.5);
    let id = ph2d_panel_vector::ids::VECTOR_BAR_STROKE;
    let region = hero.last_layout.expect("layout").popover_region();
    assert!(
        pop.y >= region.y - 0.5 && pop.y + pop.h <= region.y + region.h + 0.5,
        "o painel do traço sai da região: {pop:?} × {region:?}"
    );
    let (conteudo, visivel) = (
        hero.store.panel_content_h(id).unwrap_or(0.0),
        hero.store.panel_visible_h(id).unwrap_or(0.0),
    );
    assert!(
        conteudo > visivel + 1.0,
        "controlo: a janela baixa não transborda ({conteudo} ≤ {visivel})"
    );
    let fora_do_painel = |hero: &HeroScreen, pop: Rect| -> Vec<NodeId> {
        let dentro = linhas_dentro(hero, pop);
        todas
            .iter()
            .copied()
            .filter(|t| !dentro.contains(t) && hero.hit_index.rect_for(*t).is_some())
            .collect()
    };
    let fora = fora_do_painel(&hero, pop);
    assert!(
        fora.is_empty(),
        "linhas do painel do traço clicáveis FORA dele: {fora:?}"
    );

    let arena = Bump::new();
    let _ = hero.handle_wheel(
        ph2d_host::WheelEvent {
            x: pop.x + pop.w * 0.5,
            y: pop.y + pop.h * 0.5,
            delta_x: 0.0,
            delta_y: -10_000.0,
            modifiers: ph2d_host::Modifiers::default(),
            timestamp_ns: 0,
        },
        &arena,
    );
    quadro_em(&mut hero, &mut ts, true, HERO_VIEWPORT_H * 0.5);
    let rolado = hero.store.panel_scroll(id);
    assert!(
        (rolado - (conteudo - visivel)).abs() < 1.0,
        "a roda não levou o painel até ao fim: {rolado} de {}",
        conteudo - visivel
    );
    let fora = fora_do_painel(&hero, pop);
    assert!(
        fora.is_empty(),
        "rolado, linhas clicáveis FORA do painel: {fora:?}"
    );
    let cap = hero.hit_index.rect_for(VECTOR_CAP_ROUND);
    assert!(
        cap.is_none_or(|c| c.y >= pop.y - 0.5),
        "rolado, a ponta saiu por cima do painel e continua clicável: {cap:?}"
    );
    ph2d_panel_vector::set_stroke_paint_kind(None);
}
