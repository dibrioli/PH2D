//! Testes do [`crate::blend_live`] — módulo irmão (teto de 600 LOC por arquivo da shell).
//!
//! O que eles provam, em ordem de importância:
//!
//! 1. **A transição SEGUE a forma** (`moving_a_source_reflows_the_steps`) — a feature inteira
//!    numa asserção: mover uma fonte re-coza os passos, sem re-clicar "Blend".
//! 2. **A cadeia é pairwise** (`chain_is_pairwise_across_sources`) — N fontes, `steps` por elo.
//! 3. O blend vive na IDENTIDADE e o `settle` o pula (senão o spine seria assentado e a transição
//!    sairia deslocada).
//! 4. Um elo morto é pulado; sem 2 fontes vivas, os passos somem.

use super::*;
use ph2d_vec_scene::{VecScene, rectangle};

/// Uma cena com `n` retângulos regularmente espaçados e um Blend Object vivo sobre eles.
/// Devolve `(sim, scene, map, spine_id, sources)`.
#[allow(clippy::type_complexity)]
fn scene_with_blend(
    n: usize,
    steps: u32,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // Retângulos 2×2, centros em x = 0, 4, 8, … (bem separados: a correspondência importa).
    let sources: Vec<VecPathId> = (0..n)
        .map(|i| {
            let x = i as f64 * 4.0;
            scene.push_path(rectangle([x - 1.0, -1.0], [x + 1.0, 1.0]))
        })
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let xf = crate::vec_transform::build(&sim, &map);
    let (spine_id, blend) = create(&mut scene, &xf, &sources, steps).expect("create");
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map); // dá entidade ao spine
    assert!(attach(&mut sim, &map, spine_id, &blend));
    (sim, scene, map, spine_id, sources)
}

/// O centro (média das âncoras) de um path de MUNDO.
fn centroid(p: &ph2d_vec_scene::VecPath) -> [f64; 2] {
    let n = p.verts.len().max(1) as f64;
    let (sx, sy) = p
        .verts
        .iter()
        .fold((0.0, 0.0), |(x, y), v| (x + v.anchor[0], y + v.anchor[1]));
    [sx / n, sy / n]
}

/// **O TESTE.** Move-se uma fonte; a transição se refaz.
///
/// É a feature inteira numa asserção: os passos são função pura das fontes cozidas no MUNDO, então
/// arrastar uma forma (o gizmo de sprite, ADR-0111) recoza os passos — sem re-clicar "Blend". Se o
/// recook não rodar, se ele ler a geometria local em vez da de mundo, ou se não re-cozer ao mover,
/// este teste fica vermelho.
#[test]
fn moving_a_source_reflows_the_steps() {
    let (mut sim, mut scene, map, _spine, sources) = scene_with_blend(2, 5);
    let mut out = Vec::new();

    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut out);
    assert_eq!(out.len(), 5, "2 fontes, 5 passos por elo");
    // O passo mais perto de B (o último, t≈1) — é o que mais anda quando B anda.
    let near_b_before = centroid(out.last().expect("passo"));

    // Move a 2ª fonte (B) por d — o gizmo de sprite faria exatamente isto.
    let d = [3.0_f32, 2.0_f32];
    let eb = Entity::from_bits(map[&sources[1]]);
    sim.world_mut()
        .get_mut::<Transform>(eb)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(d[0], d[1]);

    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut out);
    assert_eq!(out.len(), 5, "a contagem não muda");
    let near_b_after = centroid(out.last().expect("passo"));

    // O passo perto de B ANDOU na direção de B (não o delta exato — ele é uma interpolação —,
    // mas claramente para lá). Se o recook não seguisse a fonte, ficaria parado.
    assert!(
        near_b_after[0] > near_b_before[0] + 1.0 && near_b_after[1] > near_b_before[1] + 0.5,
        "o passo perto de B tinha de seguir B: {near_b_before:?} -> {near_b_after:?}"
    );
}

/// A cadeia é **pairwise**: N fontes ⇒ (N−1) elos ⇒ `steps·(N−1)` passos. É o Blend multi-forma
/// do Illustrator, a capacidade nova do ADR-0122.
#[test]
fn chain_is_pairwise_across_sources() {
    for (n, steps, want) in [(2, 5, 5), (3, 4, 8), (5, 2, 8)] {
        let (mut sim, mut scene, map, _s, _src) = scene_with_blend(n, steps);
        let mut out = Vec::new();
        let xf = crate::vec_transform::build(&sim, &map);
        recook(&mut sim, &mut scene, &map, &xf, &mut out);
        assert_eq!(out.len(), want, "{n} fontes × {steps} passos/elo = {want}");
    }
}

/// O blend vive na IDENTIDADE: o recook devolve o `Transform` da entidade dele à identidade,
/// mesmo que alguém (o gizmo) o tenha mexido. É o que o torna não-arrastável — o que se move são
/// as fontes.
#[test]
fn the_blend_object_lives_at_identity() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let es = Entity::from_bits(map[&spine]);
    // Alguém arrastou o blend (o gizmo). O recook tem de desfazer.
    sim.world_mut()
        .get_mut::<Transform>(es)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(5.0, 5.0);

    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut out);

    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        Some(Transform::IDENTITY),
        "o recook devolve o blend à identidade (a geometria dele é MUNDO)"
    );
}

/// O `settle_origins` PULA o blend. O spine tem geometria de MUNDO (centros das fontes, longe de
/// 0), então sem o pulo o `settle` o centraria e a transição sairia deslocada. Gate
/// mutation-testável: tire o `VecBlend.is_none()` do `settle` e este fica vermelho.
#[test]
fn settle_skips_the_blend_object() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let es = Entity::from_bits(map[&spine]);
    // O spine NÃO está centrado em 0 (os centros das fontes estão em x = 0 e 4).
    let before = sim.world().get::<Transform>(es).copied();

    crate::vec_transform::settle_origins(&mut sim, &mut scene, &map, &[]);

    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        before,
        "o settle não pode assentar o blend (ele vive na identidade)"
    );
    assert_eq!(
        sim.world().get::<Transform>(es).copied(),
        Some(Transform::IDENTITY),
    );
}

/// Um elo morto (fonte apagada) é PULADO — a cadeia não quebra. E sem 2 fontes vivas, os passos
/// somem (o spine fica vazio, nada é desenhado).
#[test]
fn a_dead_source_is_skipped_and_below_two_the_steps_vanish() {
    let (mut sim, mut scene, map, _spine, sources) = scene_with_blend(3, 4);
    let mut out = Vec::new();

    // Apaga a fonte do MEIO — restam 2, a cadeia vira 1 elo (4 passos).
    scene.remove_path(sources[1]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut out);
    assert_eq!(out.len(), 4, "3→2 fontes vivas: 1 elo, 4 passos");

    // Apaga mais uma — resta 1, não há transição.
    scene.remove_path(sources[0]);
    let xf = crate::vec_transform::build(&sim, &map);
    recook(&mut sim, &mut scene, &map, &xf, &mut out);
    assert!(out.is_empty(), "menos de 2 fontes vivas: nenhum passo");
}
