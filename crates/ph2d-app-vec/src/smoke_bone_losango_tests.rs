//! O gate da cena do losango (`=8`).

use super::*;

/// ⭐⭐ O *Length* tem as três chaves no osso do MEIO (autorado · 1,5× · autorado), o foco é ele, e
/// ele tem segmentos para as alças aparecerem. Controlo: nenhum outro osso tem a track.
#[test]
fn a_cena_do_losango_anima_o_length_do_osso_do_meio() {
    let mut scene = VecScene::new();
    let mut sim = SimWorld::default();
    let mut doc = TimelineDoc::default();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    let raiz = st
        .bone_smoke_pend
        .as_ref()
        .and_then(|p| p[0].1)
        .expect("raiz");
    let meio = crate::smoke_bone::osso_do_meio(&sim, raiz).expect("meio");
    let l0 = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(meio)
        .expect("osso")
        .length;
    bind(&mut scene, &mut sim, &mut doc, &mut st);
    assert_eq!(
        st.bone_smoke_foco,
        Some(meio.to_bits()),
        "o foco é o osso do meio"
    );
    let osso = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(meio)
        .expect("osso");
    assert_eq!(osso.segments, SEGMENTOS_DO_MEIO);
    let alvo = doc
        .binding_for(meio.to_bits(), canal_comprimento())
        .expect("track")
        .target;
    let chaves: Vec<(i64, f32)> = doc
        .active_clip()
        .track(alvo)
        .expect("track")
        .keys()
        .iter()
        .map(|k| {
            let AnimValue::Float(v) = k.value else {
                panic!("float")
            };
            #[expect(clippy::cast_possible_truncation, reason = "quadros pequenos")]
            let q = (k.t.to_seconds() * doc.fps_display).round() as i64;
            (q, v)
        })
        .collect();
    #[expect(clippy::cast_possible_truncation, reason = "o canal é f32")]
    let esperado = vec![(0, l0 as f32), (24, (l0 * CRESCE) as f32), (48, l0 as f32)];
    assert_eq!(chaves, esperado);
    for b in ph2d_skeleton_live::esqueletos::ossos_desde(&sim, raiz) {
        if b != meio {
            assert!(doc.binding_for(b.to_bits(), canal_comprimento()).is_none());
        }
    }
}

/// O ponto local `p` de `e`, em mundo.
fn mundo(sim: &SimWorld, e: Entity, p: [f64; 2]) -> [f64; 2] {
    ph2d_vec_entities::transform::xform_of_transform(ph2d_vec_entities::transform::world_transform(
        sim, e,
    ))
    .apply(p)
}

/// A largura da barra em mundo (o maior `x` menos o menor).
fn largura(scene: &VecScene) -> f64 {
    let xs: Vec<f64> = (scene.paths().iter())
        .flat_map(|p| p.verts.iter().map(|v| v.anchor[0]))
        .collect();
    xs.iter().copied().fold(f64::MIN, f64::max) - xs.iter().copied().fold(f64::MAX, f64::min)
}

/// ⭐⭐ **No Play o *Length* EMPURRA o osso seguinte e ESTICA a barra** (dono 30/10): nos três
/// quadros das chaves a raiz do filho é a ponta do meio, e a barra é mais larga no quadro 24.
/// ⛔ CONTROLO: sem o empurrão (só o `Bone.length`, a lei velha) a raiz do filho NÃO está na ponta.
#[test]
fn no_play_o_length_empurra_o_filho_e_estica_a_barra() {
    let mut scene = VecScene::new();
    let mut sim = SimWorld::default();
    let mut doc = TimelineDoc::default();
    let mut st = crate::state::VecState::default();
    build(&mut scene, &mut sim, &mut st);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut st.entities);
    bind(&mut scene, &mut sim, &mut doc, &mut st);
    let meio = Entity::from_bits(st.bone_smoke_foco.expect("foco"));
    let filho = *(sim.world().get::<ph2d_ecs::Children>(meio))
        .and_then(|f| {
            f.iter()
                .find(|&&c| sim.world().get::<ph2d_skeleton_ecs::Bone>(c).is_some())
        })
        .expect("o osso seguinte");
    let comp = |sim: &SimWorld| {
        sim.world()
            .get::<ph2d_skeleton_ecs::Bone>(meio)
            .expect("osso")
            .length
    };
    let l0 = comp(&sim);
    let junta = |sim: &SimWorld| {
        let (p, c) = (
            mundo(sim, meio, [comp(sim), 0.0]),
            mundo(sim, filho, [0.0, 0.0]),
        );
        (p[0] - c[0]).hypot(p[1] - c[1])
    };
    let poe = |sim: &mut SimWorld, l: f64, empurra: bool| {
        let antes = comp(sim);
        sim.world_mut()
            .get_mut::<ph2d_skeleton_ecs::Bone>(meio)
            .expect("osso")
            .length = l;
        if empurra {
            ph2d_skeleton_live::comprimento::empurra_filhos(sim, meio, antes, l);
        }
    };
    let mut larguras = Vec::new();
    for l in [l0, l0 * CRESCE, l0] {
        poe(&mut sim, l, true);
        assert!(
            junta(&sim) < 1e-4,
            "a raiz do filho saiu da ponta: {}",
            junta(&sim)
        );
        ph2d_skeleton_live::skin_live::recook(&sim, &mut scene);
        larguras.push(largura(&scene));
    }
    assert!(
        larguras[1] > larguras[0] + 0.1,
        "a barra não esticou: {larguras:?}"
    );
    assert!(
        (larguras[2] - larguras[0]).abs() < 1e-6,
        "a barra não voltou: {larguras:?}"
    );
    poe(&mut sim, l0 * CRESCE, false);
    assert!(junta(&sim) > 0.1, "controlo: sem o empurrão a junta abre");
}
