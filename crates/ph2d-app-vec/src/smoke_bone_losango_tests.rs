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
