//! Os gates da cena `=140`: monta, as duas galáxias misturam em grupo (o fundo não), e vai à PLACA —
//! imagens e formas — pela porta do quadro do produto.

use super::*;
use crate::motion_state::MotionState;

fn cena() -> (MotionState, [NodeId; 3]) {
    let mut m = MotionState::new();
    let s = monta(&mut m.doc, &m.registry).expect("a cena monta e valida");
    m.sinks = s.to_vec();
    (m, s)
}

/// A cena monta, com o fundo em `Normal` e as duas galáxias em grupo (`Screen` e `Add`, `Copies`).
#[test]
fn a_cena_140_monta_com_duas_galaxias_em_grupo_sobre_um_fundo() {
    let (m, [fundo, esquerda, direita]) = cena();
    let em_grupo = |s| crate::motion_bridge::gpu::sink_mistura_em_grupo(&m.doc.graph, s, false);
    assert!(!em_grupo(fundo), "o fundo não mistura");
    assert!(
        em_grupo(esquerda) && em_grupo(direita),
        "as galáxias misturam em grupo"
    );
    for (s, tag) in [(esquerda, MISTURAS[0]), (direita, MISTURAS[1])] {
        assert_eq!(ph2d_eval_motion::sink_style(&m.doc.graph, s).blend, tag);
        assert_eq!(
            ph2d_eval_motion::sink_blend_with(&m.doc.graph, s),
            BlendWith::Copies
        );
    }
}

/// ⭐ **A cena vai à PLACA**: pela porta do quadro do produto, a rota é o dispositivo desde o 1.º
/// quadro, as imagens das galáxias chegam ao buffer da placa (e nenhuma fica visível nas sprites), e a
/// decisão da placa toma o quadro.
#[test]
#[ignore = "placa: precisa de GPU"]
fn a_cena_140_vai_a_placa() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a rota do device");
    };
    let (mut m, _) = cena();
    let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
    for q in 1..=4u32 {
        playhead.seek(f64::from(q) / 60.0);
        crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, 1.0 / 60.0);
        assert!(
            m.gpu_live && m.route_said.is_some_and(|r| r.starts_with("device")),
            "quadro {q}: {:?}",
            m.route_said
        );
    }
    let imagens = m
        .gpu_cook
        .imagens_do_grupo()
        .expect("as imagens foram à placa");
    let linhas = ph2d_gpu_cook::read_imagens_do_grupo(&gpu, imagens);
    let visiveis = linhas.iter().filter(|r| r.size != [0.0, 0.0]).count();
    let galaxia = (super::super::ritmo_demo::LADO * super::super::ritmo_demo::LADO) as usize;
    assert_eq!(
        visiveis,
        2 * galaxia,
        "um quadrado de imagem por ponto, nas duas galáxias"
    );
    let sprites = ph2d_gpu_cook::read_instances(&gpu, m.gpu_cook.instances().expect("sprites"));
    assert!(
        sprites
            .iter()
            .all(|r| r.size == [0.0, 0.0] || r.opacity == 0.0),
        "nenhuma imagem do grupo nas sprites"
    );
    let mut placa = crate::motion_shape_placa::PlacaDeFormas::default();
    assert!(crate::motion_shape_placa::decide_o_quadro(
        &mut placa,
        &gpu,
        &mut m,
        (true, true),
        crate::motion_shape_placa::gpu_tests::camara(),
    ));
    assert!(
        !placa.corridas_do_quadro().is_empty(),
        "as corridas levam a mistura"
    );
}
