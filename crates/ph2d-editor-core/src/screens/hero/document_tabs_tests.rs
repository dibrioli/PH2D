//! As abas de documento: o clique REAL (Down + Up pelo despacho, sobre o que o pintor registou), a
//! geometria e o carregamento.

use super::*;
use crate::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use bumpalo::Bump;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

pub(super) fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
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

pub(super) fn painted() -> (HeroScreen, TextSystem) {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    let mut text = TextSystem::without_system_fonts();
    repaint(&mut hero, &mut text);
    (hero, text)
}

pub(super) fn repaint(hero: &mut HeroScreen, text: &mut TextSystem) {
    let mut scene = VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, text);
}

/// O centro do alvo `id` na barra de cima, achado pelo `HitIndex` que o pintor encheu.
pub(super) fn find(hero: &HeroScreen, id: NodeId) -> Option<(f32, f32)> {
    let bar = hero.last_layout.expect("pintado").top_bar;
    let (mut xs, y) = (Vec::new(), bar.y + bar.h / 2.0);
    let mut x = bar.x;
    while x < bar.x + bar.w {
        if hero.hit_index.hit(x, y) == Some(id) {
            xs.push(x);
        }
        x += 1.0;
    }
    Some(((*xs.first()? + *xs.last()?) / 2.0, y))
}

/// Down + Up pelo despacho real, e cada evento que ele emitir pelo `apply_event` — o caminho da shell.
pub(super) fn click(hero: &mut HeroScreen, id: NodeId) {
    let (x, y) = find(hero, id).unwrap_or_else(|| panic!("{id:?} não foi pintado na barra"));
    let arena = Bump::new();
    let mut events = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    events.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    let clicked = events
        .iter()
        .any(|e| matches!(e, WidgetEvent::Click(i) | WidgetEvent::DoubleClick(i) if *i == id));
    assert!(clicked, "o clique em {id:?} não emitiu Click: {events:?}");
    for e in events {
        hero.apply_event(e);
    }
}

#[test]
fn clicking_plus_creates_a_board_opens_it_and_its_tab_is_clickable() {
    let (mut hero, mut text) = painted();
    assert!(
        find(&hero, ids::DOC_TAB_SCENE).is_some(),
        "a aba Scene não está na barra"
    );
    click(&mut hero, ids::DOC_TAB_NEW);
    let first = hero.documents.boards().boards()[0].id;
    assert_eq!(hero.documents.active(), Some(first));
    assert_eq!(hero.documents.boards().boards()[0].name, "Board 1");
    repaint(&mut hero, &mut text);
    click(&mut hero, ids::DOC_TAB_NEW);
    assert_eq!(hero.documents.boards().boards().len(), 2);
    repaint(&mut hero, &mut text);
    click(&mut hero, ids::DOC_TAB_SCENE);
    assert_eq!(
        hero.documents.active(),
        None,
        "a aba Scene não devolveu a cena"
    );
    repaint(&mut hero, &mut text);
    click(&mut hero, tab_node_id(first));
    assert_eq!(
        hero.documents.active(),
        Some(first),
        "a aba do 1.º quadro não o abriu"
    );
}

#[test]
fn a_board_owns_the_drawing_area_and_takes_no_ruler_inset() {
    let (mut hero, mut text) = painted();
    click(&mut hero, ids::DOC_TAB_NEW);
    repaint(&mut hero, &mut text);
    assert_eq!(
        hero.last_content, hero.last_canvas,
        "a área do quadro perdeu a faixa das réguas"
    );
    // E cobre a faixa da fila de ferramentas: vazia num quadro, ela mostraria a cena por baixo.
    let l = hero.last_layout.expect("pintado");
    assert!(
        l.tool_bar.h > 0.0,
        "controlo: a fixture tem fila de ferramentas"
    );
    assert!(
        hero.last_canvas.y <= l.tool_bar.y,
        "a faixa da fila ficou fora do quadro"
    );
}

/// ⛔ Num quadro, os chips da cena (mover, girar, desfazer da cena) não se pintam nem se clicam.
#[test]
fn the_scene_tool_row_is_not_on_top_of_a_board() {
    let (mut hero, mut text) = painted();
    assert!(
        find_anywhere(&hero, ids::TOOL_TRANSLATE),
        "controlo: na Scene o MOVE está lá"
    );
    click(&mut hero, ids::DOC_TAB_NEW);
    repaint(&mut hero, &mut text);
    assert!(
        !find_anywhere(&hero, ids::TOOL_TRANSLATE),
        "o MOVE da cena está por cima do quadro"
    );
}

/// ⛔ Os overlays da cena (réguas, gizmos, selecção) mudaram-se para `paint_canvas_overlays.rs`, e
/// os gates deles leem esse ficheiro — este é o que prova que o `paint_hero_screen` ainda o CHAMA,
/// e só no ramo da cena (um quadro activo pinta-se no lugar dele).
#[test]
fn the_scene_overlays_are_still_painted_and_only_without_a_board() {
    const PAINT: &str = include_str!("paint.rs");
    let board = PAINT
        .find("hero.documents.active_parts()")
        .expect("o ramo do quadro sumiu do paint");
    let call = PAINT
        .find("paint_canvas_overlays::paint_canvas_overlays(")
        .expect("o paint deixou de chamar os overlays da cena");
    let between = &PAINT[board..call];
    assert!(
        between.contains("} else {"),
        "os overlays correm fora do `else` do quadro"
    );
}

/// O alvo `id` está em algum ponto do ecrã (passo de 4 px)?
pub(super) fn find_anywhere(hero: &HeroScreen, id: NodeId) -> bool {
    let vp = hero.last_viewport;
    let mut y = vp.y;
    while y < vp.y + vp.h {
        let mut x = vp.x;
        while x < vp.x + vp.w {
            if hero.hit_index.hit(x, y) == Some(id) {
                return true;
            }
            x += 4.0;
        }
        y += 4.0;
    }
    false
}

#[test]
fn tabs_sit_between_start_and_end_without_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let bar = Rect::new(0.0, 0.0, 1366.0, 28.0);
    let mut set = BoardSet::default();
    for i in 0..6 {
        set.create(format!("A board with a long name {i}"));
    }
    let tabs = tab_rects(bar, 260.0, 900.0, &set, &[], None, Target::Scene, &mut text);
    assert_eq!(tabs.len(), 8, "Scene + 6 quadros + New");
    assert_eq!(tabs[0].0, Target::Scene);
    assert_eq!(tabs[7].0, Target::New);
    assert!(tabs[0].1.x >= 260.0);
    let last = tabs[7].1;
    assert!(
        last.x + last.w <= 900.0 + 0.001,
        "a última aba passou do fim: {last:?}"
    );
    for w in tabs.windows(2) {
        assert!(
            w[0].1.x + w[0].1.w <= w[1].1.x + 0.001,
            "abas sobrepostas: {w:?}"
        );
    }
    assert!(
        tab_rects(bar, 260.0, 270.0, &set, &[], None, Target::Scene, &mut text).is_empty(),
        "sem espaço não se pinta nada"
    );
}

#[test]
fn board_tab_ids_do_not_meet_the_layout_or_panel_tab_ids() {
    for b in 1..1000 {
        let id = tab_node_id(BoardId(b));
        assert!(super::super::layout_tabs::layout_for_tab(id).is_none());
        assert_ne!(id, ids::DOC_TAB_SCENE);
        assert_ne!(id, ids::DOC_TAB_NEW);
    }
}

#[test]
fn loading_registers_every_tab_and_returns_to_the_scene() {
    let (mut hero, mut text) = painted();
    let mut set = BoardSet::default();
    let a = set.create("Retro".into());
    let b = set.create("Ideas".into());
    hero.documents.activate(None);
    load(&mut hero, set);
    assert_eq!(hero.documents.active(), None);
    repaint(&mut hero, &mut text);
    click(&mut hero, tab_node_id(b));
    assert_eq!(hero.documents.active(), Some(b));
    repaint(&mut hero, &mut text);
    click(&mut hero, tab_node_id(a));
    assert_eq!(hero.documents.active(), Some(a));
}

/// ⭐ W23 B5 — com quadros a mais para a fila, nenhuma aba fica abaixo do piso (quatro letras): a fila
/// mostra uma janela com a ACTIVA dentro e ‹ › nas pontas; sem transbordo as setas não existem.
#[test]
fn too_many_boards_show_a_window_with_the_active_one_and_arrows() {
    let mut text = TextSystem::without_system_fonts();
    let bar = Rect::new(0.0, 0.0, 1366.0, 28.0);
    let mut set = BoardSet::default();
    let ids: Vec<BoardId> = (0..40).map(|i| set.create(format!("Board {i}"))).collect();
    let floor = rect_for_label(4.0 * TypeToken::Sm.px());
    for &active in &[ids[0], ids[20], ids[39]] {
        let tabs = tab_rects(
            bar,
            260.0,
            900.0,
            &set,
            &[],
            None,
            Target::Board(active),
            &mut text,
        );
        assert_eq!(tabs[1].0, Target::Prev);
        assert_eq!(tabs[tabs.len() - 2].0, Target::Next);
        assert!(
            tabs.iter().any(|(t, _)| *t == Target::Board(active)),
            "a activa fora da fila"
        );
        for (t, r) in &tabs {
            if let Target::Board(b) = t {
                let name = &set.get(*b).unwrap().name;
                let natural = rect_for_label(text.prefix_width(name, TypeToken::Sm.px()));
                let want = floor.min(natural);
                assert!(r.w + 0.001 >= want, "{t:?} com {} < {want}", r.w);
            }
        }
        let last = tabs[tabs.len() - 1].1;
        assert!(last.x + last.w <= 900.0 + 0.001);
    }
    let few = BoardSet::default();
    let tabs = tab_rects(bar, 260.0, 900.0, &few, &[], None, Target::Scene, &mut text);
    assert!(
        !tabs
            .iter()
            .any(|(t, _)| matches!(t, Target::Prev | Target::Next))
    );
}

/// ⭐ W23 B5 — com 40 quadros a «›» pintada abre o quadro seguinte e a «‹» o anterior (dá a volta),
/// pelo clique real na barra; a aba aberta continua à vista.
#[test]
fn the_tab_arrows_open_the_next_and_previous_board() {
    let (mut hero, mut text) = painted();
    let ids: Vec<BoardId> = (0..40)
        .map(|i| hero.documents.create(format!("Board {i}")))
        .collect();
    for id in &ids {
        register_board(&mut hero.store, *id);
    }
    hero.documents.activate(Some(ids[0]));
    repaint(&mut hero, &mut text);
    click(&mut hero, ids::DOC_TAB_NEXT);
    repaint(&mut hero, &mut text);
    assert_eq!(hero.documents.active(), Some(ids[1]));
    click(&mut hero, ids::DOC_TAB_PREV);
    click(&mut hero, ids::DOC_TAB_PREV);
    repaint(&mut hero, &mut text);
    assert_eq!(hero.documents.active(), Some(ids[39]), "dá a volta");
    assert!(
        find(&hero, tab_node_id(ids[39])).is_some(),
        "a aberta está na fila"
    );
}
