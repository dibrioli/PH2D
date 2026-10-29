//! ⭐⭐ **As ferramentas do pintor moram na COLUNA do vetor** (dono 23/10: *«vamos colocar na barra lateral
//! esquerda, com a mesma decoração do módulo vector»*). Com o pintor na mão: as doze ferramentas do trilho
//! estão na coluna publicada e em nenhum chip da fila de cima; cada uma é pintada e registada pela MESMA
//! porta do vetor (`float_bar::tool_bars::placed`, que o pintor das barras lê); um ponteiro real em cada
//! uma troca a ferramenta e acende só a ela (a Máscara e a Seleção são dois botões); o flyout das formas abre na
//! coluna; e largar o pintor limpa a coluna.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::float_bar::tool_bars::placed;
use ph2d_editor_core::screens::hero::left_rail::no_trilho;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen, tool_bar,
};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

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

fn clica(hero: &mut HeroScreen, r: Rect) -> Vec<WidgetEvent> {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    ev
}

/// As ferramentas da coluna: as do trilho QUE ESTE MEIO OFERECE ([`no_trilho`]: as da água só com o Wet Paint, e
/// no lugar do Smear e do Blur do Digital — dono 25/10), com a Máscara e a Seleção em dois botões no lugar do grupo
/// delas (dono 24/10: «temos espaço para ambos»).
fn na_coluna_esperada(wet: bool) -> Vec<NodeId> {
    ids::PAINTER_RAIL_TOOL_IDS
        .into_iter()
        .filter(|i| *i != ids::PAINTER_RAIL_MASK_GROUP && no_trilho(*i, wet))
        .chain(ids::PAINTER_RAIL_MASK_SUB_IDS)
        .collect()
}

/// Os dois meios que mudam a coluna: o Digital (sem as da água) e o Wet Paint (com elas).
const MEIOS: [bool; 2] = [false, true];

fn pintor(wet: bool) -> (HeroScreen, ph2d_text::TextSystem) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.legacy_chrome = false;
    hero.image_edit.mode_on = true;
    hero.image_edit.active_tool_id = Some("painter");
    hero.store.set_painter_rail_wet(wet);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    pinta(&mut hero, &mut ts);
    pinta(&mut hero, &mut ts);
    (hero, ts)
}

/// O fio que a ferramenta recebeu pelo barramento.
fn fio(hero: &mut HeroScreen) -> Option<String> {
    hero.bus.drain().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SelectOption(id, v))
            if id == ids::PAINTER_PAINT_MODE =>
        {
            Some(v)
        }
        _ => None,
    })
}

#[test]
fn the_painter_tools_are_the_column_and_not_the_row() {
    for wet in MEIOS {
        a_coluna_e_nao_a_fila(wet);
    }
}

fn a_coluna_e_nao_a_fila(wet: bool) {
    let (hero, mut ts) = pintor(wet);
    let bars = hero
        .store
        .tool_bars()
        .cloned()
        .expect("o pintor publicou a coluna");
    assert!(bars.painter_rail);
    let na_coluna: Vec<NodeId> = bars.column.iter().flatten().map(|b| b.id).collect();
    for id in na_coluna_esperada(wet) {
        assert!(
            na_coluna.contains(&id),
            "{id:?} (wet {wet}): a ferramenta não está na coluna"
        );
    }
    assert_eq!(
        na_coluna.len(),
        na_coluna_esperada(wet).len(),
        "a coluna tem intrusos"
    );
    let fila = tool_bar::bar_rail(&hero.store, &mut ts, true, true);
    let na_fila: Vec<NodeId> = fila.entries.iter().filter_map(|e| e.node_id()).collect();
    for id in ids::PAINTER_RAIL_TOOL_IDS
        .into_iter()
        .chain(ids::PAINTER_RAIL_MASK_SUB_IDS)
    {
        assert!(
            !na_fila.contains(&id),
            "{id:?}: a ferramenta continua na fila de cima"
        );
    }
    // O que fica em cima: o espaço, a vista, desfazer e refazer (não são ferramentas).
    for id in [
        ids::TOOL_SPACE,
        ids::TOOL_HOME,
        ids::TOOL_UNDO,
        ids::TOOL_REDO,
    ] {
        assert!(na_fila.contains(&id), "{id:?} saiu da fila de cima");
    }
    // …e o seletor de modo (Object Mode / Paint Mode), que é por onde se sai do modo Paint.
    let mut hero = hero;
    hero.store
        .publish_mode_menu(Some(ph2d_editor_core::interaction::AreaMenu {
            label: "Mode".to_owned(),
            face: "Paint Mode".to_owned(),
            faces: Vec::new(),
            rows: Vec::new(),
        }));
    let fila = tool_bar::bar_rail(&hero.store, &mut ts, true, true);
    assert!(
        fila.entries
            .iter()
            .any(|e| e.node_id() == Some(ids::area_menu_button(0))),
        "o seletor de modo saiu da fila de cima com a coluna do pintor"
    );
}

/// A MESMA porta do vetor: cada botão do pintor é registado no rectângulo que o `placed` do vetor dá.
#[test]
fn the_painter_column_is_painted_by_the_vector_door() {
    for wet in MEIOS {
        let (hero, _) = pintor(wet);
        let bars = hero.store.tool_bars().cloned().expect("publicada");
        let postos = placed(&bars, hero.last_content, false, false);
        assert_eq!(postos.len(), na_coluna_esperada(wet).len(), "wet {wet}");
        a_porta_do_vetor(&hero, postos);
    }
}

fn a_porta_do_vetor(hero: &HeroScreen, postos: Vec<(ph2d_editor_core::interaction::BarButton, Rect)>) {
    for (b, r) in postos {
        assert_eq!(
            hero.hit_index.rect_for(b.id),
            Some(r),
            "«{}»: o registo não é o da porta do vetor",
            b.label
        );
    }
}

#[test]
fn every_painter_column_button_answers_a_real_pointer() {
    for wet in MEIOS {
        cada_botao_responde(wet);
    }
}

fn cada_botao_responde(wet: bool) {
    let (mut hero, mut ts) = pintor(wet);
    let mut medidos = 0;
    let bars = hero.store.tool_bars().cloned().expect("publicada");
    for (b, r) in placed(&bars, hero.last_content, false, false) {
        let _ = hero.bus.drain().count();
        let ev = clica(&mut hero, r);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(i) if *i == b.id))
                || hero.store.picker_target().is_some(),
            "«{}»: pintado e o ponteiro não o clica ({ev:?})",
            b.label
        );
        for e in ev {
            hero.apply_event(e);
        }
        pinta(&mut hero, &mut ts);
        if b.id == ids::PAINTER_RAIL_SHAPES {
            let aberto = hero.store.tool_bars().and_then(|t| t.flyout.clone());
            assert_eq!(
                aberto.map(|(a, _)| a),
                Some(b.id),
                "«{}»: o flyout não abriu na coluna",
                b.label
            );
        } else if b.id == ids::PAINTER_RAIL_FILL {
            // A amostra é também o seletor da cor.
        } else {
            assert!(
                fio(&mut hero).is_some(),
                "«{}»: o clique não trocou a ferramenta",
                b.label
            );
            assert_eq!(
                hero.store.button_state(b.id),
                Some(ButtonState::Pressed),
                "«{}»: o botão não acendeu",
                b.label
            );
            // …e a coluna PINTA-o aceso (o `on` do botão), só a ele.
            let acesos: Vec<NodeId> = hero
                .store
                .tool_bars()
                .map(|t| {
                    t.column
                        .iter()
                        .flatten()
                        .filter(|x| x.on)
                        .map(|x| x.id)
                        .collect()
                })
                .unwrap_or_default();
            assert_eq!(
                acesos,
                vec![b.id],
                "«{}»: a coluna não acende o botão da ferramenta",
                b.label
            );
        }
        hero.store.set_picker_target(None);
        medidos += 1;
    }
    assert_eq!(
        medidos,
        na_coluna_esperada(wet).len(),
        "controlo: botões medidos (wet {wet})"
    );
}

#[test]
fn a_flyout_item_answers_and_leaving_the_painter_clears_the_column() {
    let (mut hero, mut ts) = pintor(false);
    let bars = hero.store.tool_bars().cloned().expect("publicada");
    let (_, shapes) = placed(&bars, hero.last_content, false, false)
        .into_iter()
        .find(|(b, _)| b.id == ids::PAINTER_RAIL_SHAPES)
        .expect("Shapes na coluna");
    for e in clica(&mut hero, shapes) {
        hero.apply_event(e);
    }
    pinta(&mut hero, &mut ts);
    let bars = hero.store.tool_bars().cloned().expect("publicada");
    let (b, r) = placed(&bars, hero.last_content, false, false)
        .into_iter()
        .find(|(b, _)| b.id == ids::PAINTER_RAIL_SHAPE_LINE)
        .expect("o flyout das formas pinta a Linha");
    let ev = clica(&mut hero, r);
    assert!(
        ev.iter()
            .any(|e| matches!(e, WidgetEvent::Click(i) if *i == b.id)),
        "a Linha do flyout não responde"
    );
    hero.image_edit.active_tool_id = None;
    pinta(&mut hero, &mut ts);
    assert!(
        hero.store.tool_bars().is_none(),
        "largar o pintor deixou a coluna pintada"
    );
}
