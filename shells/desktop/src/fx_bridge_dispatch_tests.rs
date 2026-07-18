//! Gates do dispatch da seção Effects — a tradução `NodeId` → intenção.
//!
//! Estes existem porque a costura é onde a feature morre em silêncio: um id que ninguém
//! classifica vira um botão pintado e inerte, e nenhum teste de unidade do motor o vê.

use super::*;
use ph2d_editor::ids as i;
use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::{VecPath, VecVertex};

fn scene_with_square() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    (scene, id)
}

/// **Todo id que o painel pode pintar é CLASSIFICADO.** Varre os tetos inteiros: um id que
/// caísse no `None` seria um controle desenhado que o dispatch ignora — o botão-morto.
#[test]
fn every_id_the_panel_can_paint_is_classified() {
    for k in 0..i::MAX_FX_KINDS {
        assert_eq!(
            classify_click(i::vector_fx_add_id(k)),
            Some(FxClick::Add(k)),
            "o Add do tipo {k} não é classificado"
        );
    }
    for r in 0..i::MAX_FX_ROWS {
        assert_eq!(
            classify_click(i::vector_fx_remove_id(r)),
            Some(FxClick::Row(r, FxRowAction::Remove))
        );
        assert_eq!(
            classify_click(i::vector_fx_up_id(r)),
            Some(FxClick::Row(r, FxRowAction::Up))
        );
        assert_eq!(
            classify_click(i::vector_fx_down_id(r)),
            Some(FxClick::Row(r, FxRowAction::Down))
        );
        assert_eq!(
            classify_click(i::vector_fx_hide_id(r)),
            Some(FxClick::Row(r, FxRowAction::Hide))
        );
        for p in 0..i::MAX_FX_ROW_PARAMS {
            assert_eq!(
                classify_param(i::vector_fx_param_id(r, p)),
                Some((r, p)),
                "o parâmetro ({r}, {p}) não é classificado"
            );
        }
    }
}

/// Um id de FORA da seção não é classificado — senão a seção roubaria cliques dos vizinhos.
#[test]
fn a_foreign_id_is_not_claimed() {
    assert_eq!(classify_click(i::VECTOR_ENVELOPE_RUN), None);
    assert_eq!(classify_param(i::VECTOR_ENVELOPE_BEND), None);
}

/// **As linhas e os tipos não se confundem**: o id da linha 1 não é o da linha 0.
#[test]
fn rows_and_kinds_do_not_collide() {
    // `BTreeSet` e não `HashSet`: o projeto proíbe o segundo (ordem de iteração não
    // determinística vira teste que flaka em CI e passa localmente).
    let mut seen = std::collections::BTreeSet::new();
    for k in 0..i::MAX_FX_KINDS {
        assert!(seen.insert(i::vector_fx_add_id(k)), "Add {k} colide");
    }
    for r in 0..i::MAX_FX_ROWS {
        assert!(seen.insert(i::vector_fx_remove_id(r)));
        assert!(seen.insert(i::vector_fx_up_id(r)));
        assert!(seen.insert(i::vector_fx_down_id(r)));
        assert!(seen.insert(i::vector_fx_hide_id(r)));
        assert!(seen.insert(i::vector_fx_card_id(r)));
        for p in 0..i::MAX_FX_ROW_PARAMS {
            assert!(seen.insert(i::vector_fx_param_id(r, p)), "param ({r},{p})");
            assert!(seen.insert(i::vector_fx_param_num_id(r, p)));
        }
    }
}

/// **O Add vem ANTES do resto**: a linha nova tem de existir antes de alguém mexer nela.
#[test]
fn the_add_is_applied_before_the_row_actions() {
    let (mut scene, id) = scene_with_square();
    // No MESMO frame: põe um efeito e já ajusta o parâmetro 0 dele.
    apply(&mut scene, id, Some(0), None, Some((0, 0, 1.0)));
    let rows = crate::fx_bridge::stack_view(&scene, id);
    assert_eq!(rows.len(), 1, "o efeito entrou");
    assert!(
        rows[0].params[0].value > 0.0,
        "o parâmetro foi ajustado no mesmo frame em que a linha nasceu"
    );
}

/// **Um Toggle num SLIDER é recusado.** O id é partilhado (o painel pinta um OU outro), então
/// um clique perdido não pode virar uma escrita silenciosa no parâmetro errado.
#[test]
fn a_toggle_click_on_a_slider_parameter_is_refused() {
    let (mut scene, id) = scene_with_square();
    apply(&mut scene, id, Some(0), None, None);
    let before = crate::fx_bridge::stack_view(&scene, id);
    // O parâmetro 0 do efeito 0 é um slider (não é caixinha) — confirmado pela declaração.
    assert!(!before[0].params[0].toggle);
    apply(
        &mut scene,
        id,
        None,
        Some((0, FxRowAction::Toggle(0))),
        None,
    );
    assert_eq!(
        crate::fx_bridge::stack_view(&scene, id),
        before,
        "um Toggle sobre um slider tem de ser inerte"
    );
}

/// Remove e reordena chegam à cena pela `apply` — o caminho que o frame de facto percorre.
#[test]
fn remove_and_reorder_reach_the_scene() {
    let (mut scene, id) = scene_with_square();
    apply(&mut scene, id, Some(0), None, None);
    apply(&mut scene, id, Some(1), None, None);
    let first = crate::fx_bridge::stack_view(&scene, id)[0].label;

    apply(&mut scene, id, None, Some((0, FxRowAction::Down)), None);
    assert_ne!(
        crate::fx_bridge::stack_view(&scene, id)[0].label,
        first,
        "o Down reordenou"
    );

    apply(&mut scene, id, None, Some((0, FxRowAction::Remove)), None);
    assert_eq!(crate::fx_bridge::stack_view(&scene, id).len(), 1);
}

/// **Uma caixinha ALTERNA a cada clique, mesmo clicada sempre no mesmo sítio.**
///
/// O id do slider é partilhado com a caixinha, então um press nela emite TAMBÉM o
/// `ValueChanged` do slider — com o track = a posição horizontal do cursor. Essa escrita corria
/// depois do flip e ganhava, então quem decidia o estado era o *onde* do clique: apertar
/// seguidamente no mesmo ponto dava sempre o mesmo resultado (Enio, 2026-07-18).
///
/// O oráculo é a ALTERNÂNCIA sob um track FIXO — é isso que o artista faz com o dedo parado.
///
/// ⚠️ **ESTE GATE FABRICA O CLIQUE**, e foi por isso que a 1ª correção passou por ele e chegou
/// morta ao produto: ele chama `apply` com o `Some(Toggle)` já pronto, então prova que o ramo
/// funciona *se* o Click chegar — nunca que ele chega. E não chegava: o id era partilhado com o
/// slider, e *um slider não emite `Click` no Up*. Quem responde a essa pergunta é o gate de seam
/// `every_effect_stack_button_reaches_the_bus_when_clicked`, que PINTA e CLICA a caixinha.
/// Registado ≠ despachado; este gate cobre a segunda metade, e só ela.
#[test]
fn a_toggle_alternates_even_when_clicked_at_the_same_spot() {
    let (mut scene, id) = scene_with_square();
    // O tipo que tem caixinha, achado pela tabela e não por nome.
    let Some((kind, param)) = (0..PathEffect::KINDS.len()).find_map(|k| {
        PathEffect::from_kind(k)?
            .params()
            .iter()
            .position(|d| d.toggle)
            .map(|p| (k, p))
    }) else {
        return; // nenhum efeito tem caixinha hoje — o gate dorme, não mente
    };
    crate::fx_bridge::add(&mut scene, id, kind);

    let read = |s: &VecScene| crate::fx_bridge::stack_view(s, id)[0].params[param].value >= 0.5;
    let start = read(&scene);
    // O MESMO clique, quatro vezes: Click (flip) + ValueChanged (o track do press). Um track
    // de 0.9 é o canto direito do botão — com o bug, ele fixava o estado em "ligado".
    for i in 1..=4 {
        apply(
            &mut scene,
            id,
            None,
            Some((0, FxRowAction::Toggle(param))),
            Some((0, param, 0.9)),
        );
        assert_eq!(
            read(&scene),
            (i % 2 == 1) != start,
            "clique {i} no mesmo sítio: a caixinha tem de ter alternado {i} vezes"
        );
    }
}

/// E o irmão: um parâmetro CONTÍNUO na mesma linha continua a receber o track. Sem este, a
/// recusa acima podia ser escrita larga demais e matar todos os sliders em silêncio.
#[test]
fn a_continuous_parameter_still_takes_the_track() {
    let (mut scene, id) = scene_with_square();
    let Some((kind, param)) = (0..PathEffect::KINDS.len()).find_map(|k| {
        PathEffect::from_kind(k)?
            .params()
            .iter()
            .position(|d| !d.toggle)
            .map(|p| (k, p))
    }) else {
        return;
    };
    crate::fx_bridge::add(&mut scene, id, kind);
    let before = crate::fx_bridge::stack_view(&scene, id)[0].params[param].value;
    apply(&mut scene, id, None, None, Some((0, param, 0.75)));
    let after = crate::fx_bridge::stack_view(&scene, id)[0].params[param].value;
    assert!(
        (after - before).abs() > 1e-9,
        "o slider parou de receber o track ({before} -> {after})"
    );
}
