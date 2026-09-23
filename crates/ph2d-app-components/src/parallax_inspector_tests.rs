//! Os gates do instantâneo e do dreno da secção PARALLAX (plano 24, W7).

use super::*;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::parallax_edits::ParallaxQueixa;

fn camada(sim: &mut SimWorld, k: [f32; 2]) -> u64 {
    sim.world_mut()
        .spawn((ScrollFactor { k }, Transform::default()))
        .id()
        .to_bits()
}

/// ⭐ **Sem `ScrollFactor` não há secção** (ADR-0166) — e os três blocos só existem com o componente.
#[test]
fn a_seccao_existe_so_com_o_factor_e_os_blocos_so_com_o_componente() {
    let mut sim = SimWorld::default();
    let nada = sim.world_mut().spawn(Transform::default()).id().to_bits();
    assert!(build_parallax_info(sim.world(), nada, 1).is_none());
    let b = camada(&mut sim, [0.5, 0.5]);
    let i = build_parallax_info(sim.world(), b, 1).expect("secção");
    assert_eq!((i.repeat, i.motion, i.limits), (None, None, None));
    let e = Entity::from_bits(b);
    sim.world_mut().entity_mut(e).insert((
        ScrollRepeat { tile: [7.0, 0.0] },
        ScrollMotion {
            velocity: [0.3, 0.0],
        },
        ScrollLimits {
            min: [-1.0, -2.0],
            max: [3.0, 4.0],
        },
    ));
    let i = build_parallax_info(sim.world(), b, 1).expect("secção");
    assert_eq!(i.repeat, Some([7.0, 0.0]));
    assert_eq!(i.motion, Some([0.3, 0.0]));
    assert_eq!(i.limits, Some(([-1.0, -2.0], [3.0, 4.0])));
}

/// ⭐⭐⭐ **As duas queixas, pela ORDEM da recusa** — sem câmera fala-se da câmera, mesmo numa
/// camada neutra; com câmera fala-se do neutro; e uma camada viva com câmera cala-se.
///
/// ⚠️⚠️ **E esta é a metade que prova a leitura do MUNDO:** o controlo positivo (`SemCamera` →
/// silêncio) só passa se o construtor ACHAR a câmera. A mesma varredura por `EntityRef` que a cena
/// usava não achava um `Name` que uma query acha — medido nesta wave — e aqui ela diria *«não há
/// câmera»* para sempre.
#[test]
fn as_queixas_seguem_a_ordem_da_recusa_e_a_camera_e_achada() {
    let mut sim = SimWorld::default();
    let neutra = camada(&mut sim, [1.0, 1.0]);
    let viva = camada(&mut sim, [0.3, 0.3]);
    let q = |sim: &SimWorld, b| {
        build_parallax_info(sim.world(), b, 1)
            .expect("secção")
            .queixa()
    };
    assert_eq!(q(&sim, neutra), Some(ParallaxQueixa::SemCamera));
    assert_eq!(q(&sim, viva), Some(ParallaxQueixa::SemCamera));
    sim.world_mut()
        .spawn((GameCamera::default(), Transform::default()));
    assert_eq!(q(&sim, neutra), Some(ParallaxQueixa::Neutra));
    assert_eq!(q(&sim, viva), None);
}

/// ⭐⭐ **O dreno escreve o PAR, e nunca anexa um componente que o artista não pediu.**
#[test]
fn o_dreno_escreve_o_par_e_nao_anexa_nada() {
    let mut sim = SimWorld::default();
    let b = camada(&mut sim, [1.0, 1.0]);
    let e = Entity::from_bits(b);
    assert!(apply_parallax_edit(
        sim.world_mut(),
        b,
        &ParallaxFieldEdit::Factor([0.2, 0.7])
    ));
    assert_eq!(sim.world().get::<ScrollFactor>(e).expect("k").k, [0.2, 0.7]);
    for edit in [
        ParallaxFieldEdit::Repeat([5.0, 0.0]),
        ParallaxFieldEdit::Motion([1.0, 0.0]),
        ParallaxFieldEdit::LimitsMin([0.0, 0.0]),
        ParallaxFieldEdit::LimitsMax([9.0, 9.0]),
    ] {
        assert!(
            !apply_parallax_edit(sim.world_mut(), b, &edit),
            "{edit:?} anexou-se"
        );
    }
    assert!(sim.world().get::<ScrollRepeat>(e).is_none());
    assert!(sim.world().get::<ScrollMotion>(e).is_none());
    assert!(sim.world().get::<ScrollLimits>(e).is_none());
    sim.world_mut().entity_mut(e).insert((
        ScrollRepeat::default(),
        ScrollMotion::default(),
        ScrollLimits::default(),
    ));
    for edit in [
        ParallaxFieldEdit::Repeat([5.0, 1.0]),
        ParallaxFieldEdit::Motion([1.0, 2.0]),
        ParallaxFieldEdit::LimitsMin([-3.0, -4.0]),
        ParallaxFieldEdit::LimitsMax([9.0, 8.0]),
    ] {
        assert!(apply_parallax_edit(sim.world_mut(), b, &edit));
    }
    let w = sim.world();
    assert_eq!(w.get::<ScrollRepeat>(e).expect("r").tile, [5.0, 1.0]);
    assert_eq!(w.get::<ScrollMotion>(e).expect("m").velocity, [1.0, 2.0]);
    let l = w.get::<ScrollLimits>(e).expect("l");
    assert_eq!((l.min, l.max), ([-3.0, -4.0], [9.0, 8.0]));
}
