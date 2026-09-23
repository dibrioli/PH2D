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
    assert!(build_parallax_info(&mut sim, nada, 1, true, false).is_none());
    let b = camada(&mut sim, [0.5, 0.5]);
    let i = build_parallax_info(&mut sim, b, 1, true, false).expect("secção");
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
    let i = build_parallax_info(&mut sim, b, 1, true, false).expect("secção");
    assert_eq!(i.repeat, Some([7.0, 0.0]));
    assert_eq!(i.motion, Some([0.3, 0.0]));
    assert_eq!(i.limits, Some(([-1.0, -2.0], [3.0, 4.0])));
}

/// ⭐⭐⭐ **As queixas, pela ORDEM da recusa** — sem câmera fala-se da câmera, mesmo numa camada
/// neutra; com câmera DESLIGADA fala-se da caixa `Active` (auditoria 26, §2.4 — a 1.ª redacção
/// contava a desligada como «há câmera» e calava-se); com câmera activa fala-se do neutro; e uma
/// camada viva com câmera cala-se.
///
/// ⚠️⚠️ **E esta é a metade que prova a leitura do MUNDO:** o controlo positivo (`SemCamera` →
/// silêncio) só passa se o construtor ACHAR a câmera activa, pela MESMA porta que a fase usa.
#[test]
fn as_queixas_seguem_a_ordem_da_recusa_e_a_camera_e_achada() {
    let mut sim = SimWorld::default();
    let neutra = camada(&mut sim, [1.0, 1.0]);
    let viva = camada(&mut sim, [0.3, 0.3]);
    let q = |sim: &mut SimWorld, b| {
        build_parallax_info(sim, b, 1, true, false)
            .expect("secção")
            .queixa()
    };
    assert_eq!(q(&mut sim, neutra), Some(ParallaxQueixa::SemCamera));
    assert_eq!(q(&mut sim, viva), Some(ParallaxQueixa::SemCamera));
    let cam = sim
        .world_mut()
        .spawn((
            GameCamera {
                active: false,
                ..GameCamera::default()
            },
            ph2d_ecs::StableId(9),
            Transform::default(),
        ))
        .id();
    assert_eq!(q(&mut sim, viva), Some(ParallaxQueixa::CameraDesligada));
    sim.world_mut()
        .get_mut::<GameCamera>(cam)
        .expect("camera")
        .active = true;
    assert_eq!(q(&mut sim, neutra), Some(ParallaxQueixa::Neutra));
    assert_eq!(q(&mut sim, viva), None);
}

/// ⛔⛔ **O neutro COM deriva não é neutro** (auditoria 26, §2.4): o passe conduz uma nuvem de
/// `k = 1` que anda sozinha, e a queixa mandava baixar o factor. As duas respostas vêm das portas do
/// passe.
#[test]
fn o_neutro_com_deriva_nao_se_queixa() {
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        GameCamera::default(),
        ph2d_ecs::StableId(9),
        Transform::default(),
    ));
    let b = camada(&mut sim, [1.0, 1.0]);
    sim.world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(ScrollMotion {
            velocity: [0.4, 0.0],
        });
    let i = build_parallax_info(&mut sim, b, 1, true, false).expect("secção");
    assert!(!i.e_neutra);
    assert_eq!(i.queixa(), None);
    // O CONTROLO: a deriva inerte volta a ser o neutro.
    sim.world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(ScrollMotion::default());
    let i = build_parallax_info(&mut sim, b, 1, true, false).expect("secção");
    assert_eq!(i.queixa(), Some(ParallaxQueixa::Neutra));
}

/// ⛔⛔ **O dolly que ATRAVESSA a camada diz-se** — e o outro motor também (auditoria 26, §2.4 e
/// §2.7). A resposta do dolly é a MESMA porta com que a ponte larga a camada.
#[test]
fn o_dolly_que_atravessa_e_o_outro_motor_dizem_se() {
    let mut sim = SimWorld::default();
    let cam = sim
        .world_mut()
        .spawn((
            GameCamera {
                dolly: 0.6,
                ..GameCamera::default()
            },
            ph2d_ecs::StableId(9),
            Transform::default(),
        ))
        .id();
    let frente = camada(&mut sim, [2.0, 2.0]);
    let i = build_parallax_info(&mut sim, frente, 1, true, false).expect("secção");
    assert_eq!(i.queixa(), Some(ParallaxQueixa::Atravessa));
    sim.world_mut()
        .get_mut::<GameCamera>(cam)
        .expect("camera")
        .dolly = 0.3;
    let i = build_parallax_info(&mut sim, frente, 1, true, false).expect("secção");
    assert_eq!(
        i.queixa(),
        None,
        "o controlo: aquém da camada não há queixa"
    );
    let i = build_parallax_info(&mut sim, frente, 1, true, true).expect("secção");
    assert_eq!(i.queixa(), Some(ParallaxQueixa::OutroMotor));
}

/// ⭐ **As duas NOTAS** — a pré-visualização desligada (a lei corre contra uma vista que o artista não
/// vê) e a cerca de fábrica sem eixo activo.
#[test]
fn as_notas_da_pre_visualizacao_e_da_cerca_inerte() {
    let mut sim = SimWorld::default();
    sim.world_mut().spawn((
        GameCamera::default(),
        ph2d_ecs::StableId(9),
        Transform::default(),
    ));
    let b = camada(&mut sim, [0.5, 0.5]);
    assert!(
        build_parallax_info(&mut sim, b, 1, false, false)
            .expect("secção")
            .nota_pre_visualizacao()
    );
    assert!(
        !build_parallax_info(&mut sim, b, 1, true, false)
            .expect("secção")
            .nota_pre_visualizacao()
    );
    sim.world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(ScrollLimits::default());
    assert!(
        build_parallax_info(&mut sim, b, 1, true, false)
            .expect("secção")
            .limites_inertes()
    );
    sim.world_mut()
        .entity_mut(Entity::from_bits(b))
        .insert(ScrollLimits {
            min: [0.0, 0.0],
            max: [0.0, 5.0],
        });
    assert!(
        !build_parallax_info(&mut sim, b, 1, true, false)
            .expect("secção")
            .limites_inertes(),
        "um eixo activo basta"
    );
}

/// ⛔⛔ **O applier trava ao DOMÍNIO DA LEI** (auditoria 26, §2.6) — um ladrilho digitado negativo
/// é «não repete», e um valor não-finito é recusado inteiro.
#[test]
fn o_applier_trava_ao_dominio_da_lei() {
    let mut sim = SimWorld::default();
    let b = camada(&mut sim, [0.5, 0.5]);
    let e = Entity::from_bits(b);
    sim.world_mut()
        .entity_mut(e)
        .insert((ScrollRepeat::default(), ScrollMotion::default()));
    assert!(apply_parallax_edit(
        sim.world_mut(),
        b,
        &ParallaxFieldEdit::Repeat([-5.0, 3.0])
    ));
    assert_eq!(
        sim.world().get::<ScrollRepeat>(e).expect("r").tile,
        [0.0, 3.0]
    );
    assert!(!apply_parallax_edit(
        sim.world_mut(),
        b,
        &ParallaxFieldEdit::Factor([f32::NAN, 0.5])
    ));
    assert_eq!(sim.world().get::<ScrollFactor>(e).expect("k").k, [0.5, 0.5]);
    assert!(!apply_parallax_edit(
        sim.world_mut(),
        b,
        &ParallaxFieldEdit::Motion([f32::INFINITY, 0.0])
    ));
    // ⚠️ O domínio e não a pista: um `k = 5` digitado é uma camada legítima.
    assert!(apply_parallax_edit(
        sim.world_mut(),
        b,
        &ParallaxFieldEdit::Factor([5.0, 5.0])
    ));
    assert_eq!(sim.world().get::<ScrollFactor>(e).expect("k").k, [5.0, 5.0]);
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
