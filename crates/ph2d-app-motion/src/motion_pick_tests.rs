//! O acerto do clique nas cópias do Motion: a caixa local pela inversa da pose, a de cima primeiro.

use ph2d_vector::Rect;

use super::{Copia, acertos};

const ID: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

fn copia(dono: u64, pos: [f32; 2], geometry: u32) -> Copia {
    Copia {
        dono,
        pos,
        size: [1.0, 1.0],
        basis: ID,
        anchor: [0.0, 0.0],
        geometry,
    }
}

fn caixa(_: u32) -> Option<Rect> {
    Some(Rect::new(-1.0, -1.0, 1.0, 1.0))
}

#[test]
fn a_copia_de_cima_ganha_e_cada_dono_aparece_uma_vez() {
    let c = [
        copia(10, [0.0, 0.0], 7),
        copia(20, [0.5, 0.0], 7),
        copia(10, [5.0, 0.0], 7),
    ];
    assert_eq!(acertos(&c, [0.2, 0.0], caixa), vec![20, 10]);
    assert_eq!(acertos(&c, [5.0, 0.5], caixa), vec![10]);
    assert!(acertos(&c, [3.0, 0.0], caixa).is_empty());
}

#[test]
fn a_pose_roda_escala_e_o_pivo_entram_antes_do_teste() {
    // Rodada 90°, size 2×1, pivô (1, 0) m: o ponto local q vai a P + R·(anchor + q·size).
    let c = Copia {
        dono: 3,
        pos: [10.0, 0.0],
        size: [2.0, 1.0],
        basis: [0.0, 1.0, -1.0, 0.0],
        anchor: [1.0, 0.0],
        geometry: 0,
    };
    // Imagem = quad unitário: anchor + q·size dá x ∈ [0, 2] ⇒ mundo y ∈ [0, 2]; y ∈ [−0,5, 0,5] ⇒ x ∈ [9,5, 10,5].
    assert_eq!(acertos(&[c], [10.0, 1.9], caixa), vec![3]);
    assert!(acertos(&[c], [10.0, -0.1], caixa).is_empty());
    assert!(acertos(&[c], [10.6, 1.0], caixa).is_empty());
    // Size zero não acerta (pose singular).
    let zero = Copia {
        size: [0.0, 0.0],
        ..c
    };
    assert!(acertos(&[zero], [10.0, 0.0], caixa).is_empty());
}

#[test]
fn geometria_sem_caixa_nao_acerta() {
    let c = [copia(1, [0.0, 0.0], 9)];
    assert!(acertos(&c, [0.0, 0.0], |_| None).is_empty());
}

fn forma(pos: [f32; 2], geometry_id: u32) -> ph2d_eval_motion::VectorInstance {
    ph2d_eval_motion::VectorInstance {
        geometry_id,
        world_pos: pos,
        size: [1.0, 1.0],
        basis: ID,
        tint: [1.0; 4],
        texture_id: 0,
        atlas_uv: [0.0; 4],
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        sampling: 0,
        blend_linha: 0,
        mistura: Default::default(),
    }
}

fn imagem(pos: [f32; 2]) -> ph2d_render::RenderInstance {
    ph2d_render::RenderInstance {
        world_pos: pos,
        size: [1.0, 1.0],
        basis: ID,
        tint: [1.0; 4],
        ..bytemuck::Zeroable::zeroed()
    }
}

/// ⭐ **Na CPU, o dono de cada cópia é o objecto do sink dela; o escondido não pega; a forma
/// (desenhada depois) fica por cima da imagem; a cópia sem tinta não pega.** (Mutação: `da_cpu`
/// sem o `filter(visible)` ⇒ o 9 aparece em `[9.0, 0]` — RED; formas antes das imagens ⇒ `[8, 7]` — RED.)
#[test]
fn na_cpu_o_clique_pega_o_dono_do_sink_e_respeita_o_olho() {
    use crate::motion_objeto::{Dono, IDENTIDADE};
    let mut m = crate::motion_state::MotionState::new();
    let (a, b, c) = (
        m.doc.graph.add_node("motion.output"),
        m.doc.graph.add_node("motion.output"),
        m.doc.graph.add_node("motion.output"),
    );
    for (no, e, visible) in [(a, 7, true), (b, 8, true), (c, 9, false)] {
        m.doc.set_owner(no, e);
        m.objeto.donos.insert(
            e,
            Dono {
                entity: e,
                visible,
                afim: IDENTIDADE,
            },
        );
    }
    let mut sem_tinta = forma([20.0, 0.0], 5);
    sem_tinta.tint[3] = 0.0;
    m.pump.vector_instances = vec![forma([0.0, 0.0], 5), forma([9.0, 0.0], 5), sem_tinta];
    m.pump.instances = vec![imagem([0.0, 0.0])];
    let faixa = |sink, instancias, vetores| ph2d_eval_motion::FaixaDoSink {
        sink,
        instancias,
        vetores,
        ladrilhos: 0..0,
    };
    m.pump.faixas = vec![
        faixa(b, 0..1, 0..0),
        faixa(a, 1..1, 0..1),
        faixa(c, 1..1, 1..2),
        faixa(a, 1..1, 2..3),
    ];
    let copias = super::da_cpu(&m);
    assert_eq!(acertos(&copias, [0.0, 0.0], caixa), vec![7, 8]);
    assert!(acertos(&copias, [9.0, 0.0], caixa).is_empty());
    assert!(acertos(&copias, [20.0, 0.0], caixa).is_empty());
}
