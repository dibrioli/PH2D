//! Os gates do [`crate::morph_live`] — o Morph vivo (o `t` animável).
//!
//! O que eles medem é o que o **olho** vê e o que o **relógio** paga:
//!
//! 1. a forma É o caminho (`t=0` é A, `t=1` é B, e o meio está no meio);
//! 2. mexer no `t` **não** re-busca a correspondência (ela é função do par, não do `t`);
//! 3. mexer numa FONTE re-busca (senão o morph descreve a forma onde ela estava);
//! 4. o morph vive na identidade — a geometria é mundo, e uma pose por cima a deslocaria;
//! 5. uma fonte apagada **congela**, não some;
//! 6. o buraco de uma rosquinha atravessa o morph.

use super::*;
use ph2d_vec_scene::{Contour, FillRule, ShapeKind, contains_point, cook};

/// Uma forma do catálogo, centrada em `c`.
fn shape(kind: ShapeKind, c: [f64; 2], half: [f64; 2]) -> VecPath {
    cook(
        kind,
        [c[0] - half[0], c[1] - half[1]],
        [c[0] + half[0], c[1] + half[1]],
        &[],
    )
}

fn circle(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Ellipse, c, [r, r])
}

fn square(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Rectangle, c, [r, r])
}

/// Uma cena com um morph entre `a` e `b`. Devolve `(sim, scene, map, morph_id, [fonte_a, fonte_b])`.
fn scene_with_morph(
    a: VecPath,
    b: VecPath,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId, [VecPathId; 2]) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let ia = scene.push_path(a);
    let ib = scene.push_path(b);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let (id, morph) = create(&mut scene, ia, ib);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map); // dá entidade ao morph
    assert!(attach(&mut sim, &map, id, &morph));
    (sim, scene, map, id, [ia, ib])
}

/// Roda um frame de recook e devolve a forma morfada.
fn frame(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    plans: &mut MorphPlans,
    id: VecPathId,
) -> VecPath {
    let xf = crate::vec_transform::build(sim, map);
    recook(sim, scene, map, &xf, plans);
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("o morph")
        .clone()
}

/// Põe o `t` do morph.
fn set_t(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, t: f32) {
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().get_mut::<VecMorph>(e).expect("morph").t = t;
}

/// O centro da bbox das âncoras do contorno primário — "onde a forma está".
fn center(p: &VecPath) -> [f64; 2] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
}

/// **A FORMA É O CAMINHO.** `t=0` devolve A, `t=1` devolve B, e o meio está no meio.
///
/// Sem isto nada mais importa: o morph seria uma forma qualquer que responde a um slider.
#[test]
fn the_morph_is_the_shape_between_the_two_sources() {
    let (mut sim, mut scene, map, id, _) =
        scene_with_morph(circle([0.0, 0.0], 1.0), circle([10.0, 0.0], 1.0));
    let mut plans = MorphPlans::new();

    for (t, want_x) in [(0.0, 0.0), (0.5, 5.0), (1.0, 10.0)] {
        set_t(&mut sim, &map, id, t);
        let got = center(&frame(&mut sim, &mut scene, &map, &mut plans, id));
        assert!(
            (got[0] - want_x).abs() < 0.05,
            "t={t}: a forma devia estar em x={want_x}, está em {got:?}"
        );
    }
}

/// **MEXER NO `t` NÃO RE-BUSCA A CORRESPONDÊNCIA.**
///
/// É a razão de o cache existir, e o gate **conta** em vez de cronometrar (um cronômetro mede a
/// máquina, não o código). A correspondência é função do PAR; o `t` só a avalia.
///
/// Sem o cache, uma animação de 60 fps faria 60 buscas de fase (256×256) por segundo — os 5,9 ms
/// por busca que o `Plan` foi inventado para matar, agora dentro do orçamento de quadro, num
/// caminho que roda **enquanto a timeline toca**.
#[test]
fn scrubbing_t_does_not_research_the_correspondence() {
    let (mut sim, mut scene, map, id, _) =
        scene_with_morph(square([0.0, 0.0], 1.0), circle([10.0, 0.0], 1.0));
    let mut plans = MorphPlans::new();

    for k in 0..=20 {
        set_t(&mut sim, &map, id, k as f32 / 20.0);
        frame(&mut sim, &mut scene, &map, &mut plans, id);
    }
    assert_eq!(
        plans.builds, 1,
        "21 frames de scrub custaram {} planos — a correspondência não depende do `t`, e \
         re-buscá-la por frame é o trabalho que o `Plan` existe para não fazer",
        plans.builds
    );
}

/// **MEXER NUMA FONTE RE-BUSCA** — e é por isso que a chave do cache é a geometria em MUNDO.
///
/// O gate move a fonte pela POSE (o `Transform`, que é como o artista a move), não pelos vértices:
/// uma chave que olhasse só a geometria local ficaria feliz, e o morph continuaria a descrever a
/// forma onde ela **estava**.
#[test]
fn moving_a_source_rebuilds_the_plan_and_reflows_the_morph() {
    let (mut sim, mut scene, map, id, src) =
        scene_with_morph(circle([0.0, 0.0], 1.0), circle([10.0, 0.0], 1.0));
    let mut plans = MorphPlans::new();
    set_t(&mut sim, &map, id, 0.5);

    let before = center(&frame(&mut sim, &mut scene, &map, &mut plans, id));
    assert_eq!(plans.builds, 1);

    // A fonte A sobe 8 unidades — pela POSE, como o gizmo faz.
    let e = Entity::from_bits(map[&src[0]]);
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .translation
        .y += 8.0;

    let after = center(&frame(&mut sim, &mut scene, &map, &mut plans, id));
    assert_eq!(
        plans.builds, 2,
        "a fonte MOVEU e o plano não foi refeito: o cache está a olhar a geometria LOCAL, e o \
         morph descreve a forma onde ela estava"
    );
    assert!(
        (after[1] - before[1]) > 3.0,
        "o morph não acompanhou a fonte: {before:?} -> {after:?}"
    );
}

/// **O morph vive na IDENTIDADE.** A geometria é MUNDO; uma pose por cima a deslocaria — e o
/// gizmo, que é inócuo sobre ele, deixaria de o ser.
#[test]
fn the_morph_lives_at_identity() {
    let (mut sim, mut scene, map, id, _) =
        scene_with_morph(circle([0.0, 0.0], 1.0), circle([10.0, 0.0], 1.0));
    let mut plans = MorphPlans::new();

    let e = Entity::from_bits(map[&id]);
    sim.world_mut()
        .get_mut::<Transform>(e)
        .expect("Transform")
        .translation = ph2d_core::Vec2::new(3.0, 4.0);

    frame(&mut sim, &mut scene, &map, &mut plans, id);
    assert_eq!(
        *sim.world().get::<Transform>(e).expect("Transform"),
        Transform::IDENTITY,
        "uma pose sobreviveu no morph — ela somaria com a geometria de mundo e o deslocaria"
    );
}

/// **UMA FONTE APAGADA CONGELA a forma, não a apaga.**
///
/// Apagar o morph junto destruiria trabalho; cozê-lo com uma fonte só o faria sumir da tela. É a
/// escolha do conector com uma ponta perdida — a única que preserva o desenho.
#[test]
fn a_dead_source_freezes_the_morph_instead_of_vanishing_it() {
    let (mut sim, mut scene, map, id, src) =
        scene_with_morph(circle([0.0, 0.0], 1.0), circle([10.0, 0.0], 1.0));
    let mut plans = MorphPlans::new();
    set_t(&mut sim, &map, id, 0.5);
    let alive = frame(&mut sim, &mut scene, &map, &mut plans, id);
    assert!(!alive.verts.is_empty(), "o morph nasceu vazio");

    scene.remove_path(src[1]);
    let frozen = frame(&mut sim, &mut scene, &map, &mut plans, id);
    assert_eq!(
        frozen.verts, alive.verts,
        "a fonte sumiu e o morph mudou: ele tinha de CONGELAR onde estava"
    );
}

/// **O BURACO ATRAVESSA O MORPH.** O irmão vivo do `the_hole_survives_the_morph` do motor: a
/// rosquinha é a saída típica da booleana, e um morph animado dela não pode ser um disco animado.
#[test]
fn the_hole_survives_into_the_live_morph() {
    let donut = |c: [f64; 2]| VecPath {
        verts: circle(c, 2.0).verts,
        closed: true,
        subpaths: vec![Contour::new_closed(circle(c, 1.0).verts)],
        fill_rule: FillRule::EvenOdd,
        ..VecPath::default()
    };
    let (mut sim, mut scene, map, id, _) = scene_with_morph(donut([0.0, 0.0]), donut([10.0, 0.0]));
    let mut plans = MorphPlans::new();
    set_t(&mut sim, &map, id, 0.5);

    let mid = frame(&mut sim, &mut scene, &map, &mut plans, id);
    assert!(contains_point(&mid, [5.0, 1.5]), "a parede sumiu");
    assert!(
        !contains_point(&mid, [5.0, 0.0]),
        "o morph vivo de duas rosquinhas é um DISCO"
    );
}
