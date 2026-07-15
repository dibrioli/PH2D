//! Testes do **SPINE editável** do [`crate::blend_live`] (ADR-0122 Fase C2) — irmão de
//! `blend_live_tests.rs`, separado pelo teto de 600 LOC. Reusa `scene_with_blend`/`centroid` de lá
//! (`super::tests`, `pub(super)`).
//!
//! O que provam: o spine editado puxa os passos pela curva (arco); o não-editado fica na reta; a
//! edição AUTORA o spine (e trava o auto-regen); as PONTAS seguem as fontes (só o interior edita).

use super::tests::{centroid, scene_with_blend};
use super::*;

/// Bota o spine `id` com os vértices `pts` (o que a edição no modo Node faria).
fn set_spine(scene: &mut VecScene, id: VecPathId, pts: &[[f64; 2]]) {
    let p = scene.path_mut(id).expect("spine");
    p.verts = pts.iter().map(|&q| VecVertex::corner(q)).collect();
    p.closed = false;
}

/// **O spine EDITADO puxa os passos para a curva** (o coração do pedido do Enio). As fontes estão
/// em (0,0) e (4,0); um spine com pico em (2,3) leva o passo do meio (fração de arco ½) para o
/// topo. Prova o flow por comprimento de arco no caminho real do recook.
#[test]
fn an_authored_bent_spine_flows_the_steps_onto_the_curve() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 1); // 1 passo (no meio)
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]); // pico no meio

    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    // out = [passo, fonte_de_cima]. O passo (lerp em (2,0)) subiu para ~(2,3).
    let step = centroid(&out[0]);
    assert!(
        (step[0] - 2.0).abs() < 0.5 && step[1] > 2.0,
        "o passo tinha de subir para a curva do spine: {step:?}"
    );
}

/// **O spine NÃO-editado deixa os passos na reta do lerp** — o caminho automático é byte-idêntico
/// à Fase B (as fontes estão em y=0, então todo passo fica em y=0; um flow espúrio o tiraria).
#[test]
fn an_unedited_spine_leaves_the_steps_on_the_lerp() {
    let (mut sim, mut scene, map, _spine, _src) = scene_with_blend(2, 5);
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    for step in &out {
        let c = centroid(step);
        assert!(
            c[1].abs() < 1e-9,
            "não-autorado: passo na reta (y=0): {c:?}"
        );
    }
}

/// **Editar o spine o AUTORA — e o recook para de sobrescrevê-lo.** É a detecção (spine atual ≠
/// último auto): frame 1 escreve o auto e o memoriza; a mão adiciona um ponto INTERIOR; frame 2
/// detecta, marca `spine_authored` e o interior SOBREVIVE (o auto teria voltado a 2 vértices). Edita
/// o INTERIOR, não uma ponta — as pontas são pinadas aos centros (ver o gate de pinagem).
#[test]
fn editing_the_spine_authors_it_and_stops_the_auto_regen() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    let mut spines = BlendSpines::new();
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);

    // Frame 1: automático. O recook escreve o auto (2 vértices) e o memoriza.
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(
        !sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "ainda automático"
    );

    // A mão edita o spine (modo Node): ADICIONA um ponto interior e o sobe.
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 5.0], [4.0, 0.0]]);

    // Frame 2: detecta a edição → autora, e o INTERIOR sobrevive (auto teria voltado a 2 vértices).
    recook(&mut sim, &mut scene, &map, &xf, &mut spines, &mut out);
    assert!(
        sim.world()
            .get::<VecBlend>(e)
            .expect("blend")
            .spine_authored,
        "a edição tinha de autorar o spine"
    );
    let p = scene.paths().iter().find(|p| p.id == spine).expect("spine");
    assert_eq!(
        p.verts.len(),
        3,
        "o ponto interior não foi varrido pelo auto-regen"
    );
    assert!(
        p.verts[1].anchor[1] > 4.0,
        "o interior editado sobrevive: {:?}",
        p.verts[1].anchor
    );
}

/// **As PONTAS do spine seguem as fontes** — só o interior é editável (o Illustrator). O artista
/// tenta arrastar as pontas para longe; o recook as devolve aos centros das fontes; o interior fica.
#[test]
fn the_spine_endpoints_are_pinned_to_the_sources() {
    let (mut sim, mut scene, map, spine, _src) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 9.0], [2.0, 3.0], [4.0, 9.0]]); // pontas afastadas

    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    let v = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .clone();
    assert!(
        v[0].anchor[1].abs() < 1e-9 && v[2].anchor[1].abs() < 1e-9,
        "as pontas voltaram aos centros (y=0): {:?} / {:?}",
        v[0].anchor,
        v[2].anchor
    );
    assert!(
        (v[1].anchor[1] - 3.0).abs() < 1e-9,
        "o interior fica: {:?}",
        v[1].anchor
    );
}

/// Uma ponta fixada **segue a fonte** quando ela se move (a curva vai junto).
#[test]
fn a_pinned_spine_endpoint_follows_its_source() {
    let (mut sim, mut scene, map, spine, sources) = scene_with_blend(2, 3);
    let e = Entity::from_bits(map[&spine]);
    sim.world_mut()
        .get_mut::<VecBlend>(e)
        .expect("blend")
        .spine_authored = true;
    set_spine(&mut scene, spine, &[[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]);

    let e1 = Entity::from_bits(map[&sources[1]]);
    sim.world_mut()
        .get_mut::<Transform>(e1)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(0.0, 5.0);

    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );

    let last = scene
        .paths()
        .iter()
        .find(|p| p.id == spine)
        .expect("spine")
        .verts
        .last()
        .expect("ponta")
        .anchor;
    assert!(
        (last[1] - 5.0).abs() < 1e-6,
        "a ponta seguiu a fonte para y=5: {last:?}"
    );
}
