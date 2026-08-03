//! Os gates da PORTA de autoria das ÂNCORAS.
//!
//! O que só se pode afirmar aqui: que a tabela `par ↔ chip` responde igual nos dois sentidos, que
//! **armar captura a régua uma vez e ela SOBREVIVE à troca de chip**, que o neutro **destaca**, e
//! que o SUJEITO da seção é o filho — e só quando a moldura dele NÃO flui.

use super::*;
use ph2d_ecs::{LayoutDir, VecFrame, VecLayout};
use ph2d_vec_scene::rectangle;

/// Uma moldura de 100×40 com `n` filhos, já sincronizada. Devolve `(sim, scene, map, ids)` onde o
/// **primeiro** id é a moldura.
fn frame_with(n: usize) -> (SimWorld, VecScene, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [100.0, 40.0]));
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut()
        .entity_mut(frame)
        .insert(VecFrame { clip: false });
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(frame));
    }
    let mut ids = vec![frame_id];
    ids.extend(kids);
    (sim, scene, map, ids)
}

fn rule_of(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<VecAnchors> {
    sim.world()
        .get::<VecAnchors>(Entity::from_bits(map[&id]))
        .copied()
}

/// **Todo chip nomeia um par, e o par acende o chip de volta.**
///
/// As duas tabelas são percorridas inteiras: um chip novo entra numa lista só, e a ida-e-volta é o
/// que impede a metade que PINTA de divergir da que HONRA.
#[test]
fn every_chip_names_a_pair_and_the_pair_lights_it_back() {
    for (table, wrap) in [
        (H, (|p| AnchorEdit::H(p)) as fn([f64; 2]) -> AnchorEdit),
        (V, (|p| AnchorEdit::V(p)) as fn([f64; 2]) -> AnchorEdit),
    ] {
        for &(chip, pair) in table {
            assert_eq!(anchor_edit_for_id(chip), Some(wrap(pair)), "ida");
            assert_eq!(chip_of(table, pair), Some(chip), "volta");
        }
    }
}

/// **Um par que nenhum chip nomeia não acende chip nenhum** — a verdade, e não o vizinho mais
/// próximo.
#[test]
fn an_unnamed_pair_lights_nothing() {
    assert_eq!(chip_of(H, [0.25, 0.25]), None);
}

/// **Sem componente, a seção mostra o NEUTRO** — que é o que a ausência de facto produz.
#[test]
fn a_child_without_a_rule_shows_the_neutral_lit() {
    let (sim, _scene, map, ids) = frame_with(1);
    let a = selected_anchors(&sim, &map, &[ids[1]]).expect("o filho e' ancoravel");
    assert_eq!(a.h, Some(ids::VECTOR_ANCHOR_H_START));
    assert_eq!(a.v, Some(ids::VECTOR_ANCHOR_V_END), "Bottom, o Y-up");
}

/// **A seção NÃO é oferecida num fluxo** — a mesma porta que o passe usa para recusar.
#[test]
fn the_section_is_not_offered_inside_a_flow() {
    let (mut sim, _scene, map, ids) = frame_with(1);
    let frame = Entity::from_bits(map[&ids[0]]);
    sim.world_mut().entity_mut(frame).insert(VecLayout {
        dir: LayoutDir::Row,
        ..VecLayout::default()
    });
    assert!(selected_anchors(&sim, &map, &[ids[1]]).is_none());
}

/// **Armar captura a régua da moldura de AGORA.**
#[test]
fn arming_captures_the_frames_current_box_as_the_ruler() {
    let (mut sim, scene, map, ids) = frame_with(1);
    assert!(apply_anchor_edit(
        &mut sim,
        &scene,
        &map,
        &[ids[1]],
        AnchorEdit::H([1.0, 1.0]),
    ));
    let a = rule_of(&sim, &map, ids[1]).expect("a regra nasceu");
    assert_eq!(a.base, [0.0, 0.0, 100.0, 40.0]);
    assert_eq!(a.min, [1.0, 0.0], "so' o eixo X foi escrito");
}

/// **A régua SOBREVIVE à troca de chip**, e isso é a decisão de produto.
///
/// ⚠️ Re-capturar a cada clique faria a troca de `Right` por `Center` devolver o filho à posição
/// autorada — um salto para trás que o artista não pediu. Mantendo-a, ele vê *o que a regra nova
/// diz sobre o redimensionamento que já existe*.
///
/// ⚠️ **A fixture TEM de redimensionar a moldura entre os dois cliques.** Sem isso, *re-capturar*
/// e *manter* devolvem o mesmo número e o gate fica verde sobre as duas leis — foi assim que a
/// primeira versão dele sobreviveu à mutação.
#[test]
fn changing_the_chip_keeps_the_ruler_that_was_captured() {
    let (mut sim, mut scene, map, ids) = frame_with(1);
    apply_anchor_edit(&mut sim, &scene, &map, &[ids[1]], AnchorEdit::H([1.0, 1.0]));
    // A moldura CRESCE (o `W` do painel escala a geometria local em torno do canto mínimo) e só
    // então o artista troca de chip.
    assert!(scene.scale_path(ids[0], 1.6, 1.0, [0.0, 0.0]));
    apply_anchor_edit(&mut sim, &scene, &map, &[ids[1]], AnchorEdit::H([0.5, 0.5]));
    let a = rule_of(&sim, &map, ids[1]).expect("a regra continua");
    assert_eq!(
        a.base,
        [0.0, 0.0, 100.0, 40.0],
        "a regua foi RE-capturada: a troca de chip devolveu o filho a' posicao autorada"
    );
    assert_eq!(a.min, [0.5, 0.0]);
}

/// **O neutro DESTACA** — uma regra que não move nada não viaja no arquivo.
#[test]
fn returning_to_the_neutral_detaches_the_component() {
    let (mut sim, scene, map, ids) = frame_with(1);
    apply_anchor_edit(&mut sim, &scene, &map, &[ids[1]], AnchorEdit::H([1.0, 1.0]));
    assert!(rule_of(&sim, &map, ids[1]).is_some());
    assert!(apply_anchor_edit(
        &mut sim,
        &scene,
        &map,
        &[ids[1]],
        AnchorEdit::H([0.0, 0.0]),
    ));
    assert!(rule_of(&sim, &map, ids[1]).is_none(), "destacou");
}

/// **Clicar o chip que já está aceso é um no-op** — o `post_frame_undo` regista por diff, e um
/// passo de undo sem mudança nenhuma é lixo que o artista tem de desfazer.
#[test]
fn re_clicking_the_lit_chip_changes_nothing() {
    let (mut sim, scene, map, ids) = frame_with(1);
    assert!(!apply_anchor_edit(
        &mut sim,
        &scene,
        &map,
        &[ids[1]],
        AnchorEdit::H([0.0, 0.0]),
    ));
    assert!(rule_of(&sim, &map, ids[1]).is_none());
}
