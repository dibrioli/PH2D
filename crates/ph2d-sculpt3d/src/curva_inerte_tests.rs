//! Os gates da [`super::CurvaInerte`] — e o valor deles está nas metades
//! NEGATIVAS: *uma porta que respondesse «inerte» sempre esconderia a razão
//! certa atrás de ruído que o artista aprende a ignorar.*

use super::CurvaInerte;
use crate::{Brush, PoseDeformacao, Verb};

/// ⭐⭐⭐ **AS TRÊS RAZÕES, e os controlos que as separam.**
#[test]
fn a_curva_so_e_inerte_onde_a_medicao_diz() {
    let com = |verb: Verb| Brush {
        verb,
        ..Brush::default()
    };

    // (1) O canal de máscara tem a SEGUNDA curva da referência.
    assert_eq!(
        com(Verb::Mask).curva_inerte(),
        Some(CurvaInerte::OCanalTemCurvaPropria)
    );
    // (2) Quem não tem lei por-vértice não tem onde a aplicar.
    assert_eq!(
        com(Verb::Density).curva_inerte(),
        Some(CurvaInerte::SemLeiPorVertice)
    );
    // (3) ⭐ O CONTROLO POSITIVO: os verbos comuns leem-na.
    for v in [Verb::Draw, Verb::Clay, Verb::Smooth, Verb::Inflate] {
        assert_eq!(
            com(v).curva_inerte(),
            None,
            "o {} lê a curva e a porta disse que não",
            v.label()
        );
    }
}

/// ⭐⭐⭐ **A POSE PEDE AS DUAS CONDIÇÕES, e nenhuma se adivinha da outra.**
///
/// ⚠️ A tabela medida (sonda `diag_a_pose_por_deformacao`, cinco deformações ×
/// `1/2/4/8` segmentos): só a **torção** move o barro ao trocar de curva, e
/// mesmo ela lê `0,000e0` com **um** segmento — *com um segmento não há nada
/// para a curva repartir*.
#[test]
fn a_pose_le_a_curva_so_na_torcao_e_com_mais_de_um_segmento() {
    let pose = |d: PoseDeformacao, segmentos: u32| Brush {
        verb: Verb::Pose,
        pose: crate::PoseControlos {
            deformacao: d,
            segmentos,
            ..crate::PoseControlos::default()
        },
        ..Brush::default()
    };
    // ⭐ O único par que lê.
    assert_eq!(pose(PoseDeformacao::Torcer, 2).curva_inerte(), None);
    assert_eq!(pose(PoseDeformacao::Torcer, 8).curva_inerte(), None);
    // (a) A torção com UM segmento não lê — e é o valor de FÁBRICA.
    assert_eq!(
        pose(PoseDeformacao::Torcer, 1).curva_inerte(),
        Some(CurvaInerte::APoseSoNaTorcaoComSegmentos),
        "com um segmento não há o que repartir, e é o valor de fábrica"
    );
    // (b) As outras quatro não leem, com quantos segmentos for.
    for d in PoseDeformacao::ALL {
        if d == PoseDeformacao::Torcer {
            continue;
        }
        for segmentos in [1u32, 2, 8] {
            assert_eq!(
                pose(d, segmentos).curva_inerte(),
                Some(CurvaInerte::APoseSoNaTorcaoComSegmentos),
                "o {} com {segmentos} segmento(s) não lê a curva",
                d.label()
            );
        }
    }
}
