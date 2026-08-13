//! **Gates da FICHA do arrasto no quadro real** — ela é desenhada, e só quando há algo a dizer.
//!
//! ⚠️ O oráculo é `Scene::encoding().n_paths` — *o que foi mesmo desenhado* —, e não uma região de
//! hit: a ficha **não regista hit nenhum** de propósito (um alvo a ~18 px do cursor roubaria o
//! pen-down exactamente onde a mão trabalha), então um gate que a procurasse no `hit_index` ficaria
//! verde sobre uma ficha que ninguém pinta. É o mesmo oráculo (e a mesma razão) da barra de
//! progresso: [[feedback_painted_is_not_populated_paint_gate]].

use super::paint::paint_hero_screen;
use super::*;
use crate::gizmo::{GizmoDragKind, GizmoDragState, GizmoTarget, TransformSnapshot};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H)
}

fn a_drag() -> GizmoDragState {
    GizmoDragState {
        kind: GizmoDragKind::Translate,
        entity_bits: 1,
        start_screen: (400.0, 300.0),
        cursor_screen: (500.0, 340.0),
        start_transform: TransformSnapshot::IDENTITY,
        pivot_world: [0.0, 0.0],
        start_cursor_world: [0.0, 0.0],
        sprite_half_intrinsic: [0.5, 0.5],
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    }
}

/// Quantos caminhos um quadro desenha com este estado de gizmo.
fn paths_drawn(readout: Option<&str>, drag: Option<GizmoDragState>) -> u32 {
    crate::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.gizmo.readout = readout.map(str::to_string);
    hero.gizmo.drag = drag;
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(&mut hero, viewport(), &mut scene, &mut text);
    scene.inner().encoding().n_paths
}

/// ⭐ **A ficha é DESENHADA quando há número e mão.**
///
/// *Mutação que sangra:* apagar o bloco da ficha do `paint_hero_screen` — o número volta a existir
/// só no `HeroScreen` e nunca na tela, que é o estado em que a feature inteira fica invisível com
/// os 11 gates do `gizmo::readout` verdes.
#[test]
fn the_chip_is_painted_when_there_is_a_number_and_a_hand() {
    let bare = paths_drawn(None, None);
    let with_chip = paths_drawn(Some("+120.0, -45.0 px"), Some(a_drag()));
    assert!(
        with_chip > bare,
        "nada foi desenhado a mais com a ficha armada: {bare} contra {with_chip} caminhos"
    );
}

/// **Sem número não há ficha** — nem quando há um arrasto aberto.
///
/// É esta metade que impede a ficha de piscar a cada clique de selecção: um pick de canvas abre um
/// arrasto de Translate, e quem decide o silêncio é o `None` publicado (ver
/// `GizmoReadout::is_idle`).
#[test]
fn a_drag_with_nothing_to_say_paints_no_chip() {
    assert_eq!(
        paths_drawn(None, Some(a_drag())),
        paths_drawn(None, None),
        "um arrasto sem número desenhou alguma coisa"
    );
}

/// **Sem MÃO não há ficha.** Um número sem arrasto não tem onde pousar — e pousá-lo num sítio
/// arbitrário seria pior do que não o mostrar.
#[test]
fn a_number_with_no_drag_paints_no_chip() {
    assert_eq!(
        paths_drawn(Some("+120.0, -45.0 px"), None),
        paths_drawn(None, None),
        "uma ficha sem mão a seguir foi desenhada algures"
    );
}

/// A ficha segue a MÃO: mover o cursor do arrasto muda o que é desenhado.
///
/// ⚠️ O oráculo aqui não pode ser a contagem de caminhos (a mesma ficha noutro sítio desenha o
/// mesmo número deles) — é o BYTE do encoding, que carrega as coordenadas.
#[test]
fn the_chip_follows_the_hand() {
    crate::test_support::ensure_panel_registry();
    let render = |cursor: (f32, f32)| {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.gizmo.readout = Some("+120.0, -45.0 px".to_string());
        hero.gizmo.drag = Some(GizmoDragState {
            cursor_screen: cursor,
            ..a_drag()
        });
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_hero_screen(&mut hero, viewport(), &mut scene, &mut text);
        scene.inner().encoding().path_data.clone()
    };
    assert_ne!(
        render((300.0, 300.0)),
        render((700.0, 500.0)),
        "a ficha desenhou a mesma geometria em dois sítios do cursor: ela não segue a mão"
    );
}
