//! Os gates da [`crate::curvatura`] — a grandeza geométrica que a subsuperfície maciça lê.

use crate::curvatura;
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::{Hybrid, Registry};

fn esfera(raio: f32) -> FieldDoc {
    FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: raio },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("esfera")
}

/// Pontos na superfície de uma esfera centrada na origem, espalhados pelo hemisfério.
fn na_superficie(raio: f32, n: usize) -> Vec<[f32; 3]> {
    (0..n)
        .map(|i| {
            let t = (i as f32 + 0.5) / n as f32;
            let z = 1.0 - 2.0 * t;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_2 * i as f32; // o ângulo de ouro — espalha sem repetir meridiano
            [raio * r * a.cos(), raio * r * a.sin(), raio * z]
        })
        .collect()
}

/// ⭐⭐⭐ **Uma esfera de raio `R` lê curvatura `1/R`** — e a régua DISTINGUE raios, que é a metade
/// que prova que ela mede a peça e não devolve uma constante.
///
/// ⚠️ A barra é **relativa** porque a lei é uma diferença finita de segunda ordem: o erro dela é
/// `O(ε²·f⁗)` ao lado de um cancelamento catastrófico em `f32` (`Σf − 4f(p)` sobre números da ordem
/// de `ε²`). Medido, ela fecha bem dentro de `5 %` nos três raios.
#[test]
fn uma_esfera_de_raio_r_le_curvatura_um_sobre_r() {
    let mut lido = Vec::new();
    for raio in [0.4_f32, 1.0, 2.5] {
        let doc = esfera(raio);
        let reg = Registry::default();
        let mut eval = Hybrid::new(&doc, &reg);
        // ⚠️ O passo é o da SEGUNDA diferença, e não o da normal — ver [`curvatura::eps_para`].
        let eps = curvatura::eps_para(raio);
        let k = curvatura::curvaturas(&mut eval, &na_superficie(raio, 64), eps);
        let pior = k
            .iter()
            .map(|v| (v - 1.0 / raio).abs() / (1.0 / raio))
            .fold(0.0_f32, f32::max);
        let media = k.iter().sum::<f32>() / k.len() as f32;
        eprintln!(
            "raio {raio}: verdade {:.4} · média {media:.4} · pior relativo {pior:.4}",
            1.0 / raio
        );
        assert!(
            pior <= 0.05,
            "raio {raio}: a curvatura afasta-se {pior:.4} de 1/R (barra 0,05)"
        );
        lido.push(media);
    }
    // A metade que prova que a régua VÊ a peça: raios diferentes dão curvaturas diferentes, na
    // ordem certa. ⛔ Sem isto, uma lei que devolvesse uma constante passaria a metade de cima
    // escolhendo o número certo para um raio.
    assert!(
        lido[0] > lido[1] * 2.0 && lido[1] > lido[2] * 2.0,
        "as curvaturas {lido:?} não separam os raios 0,4 · 1,0 · 2,5"
    );
}

/// ⭐⭐ **Um PLANO lê curvatura ZERO** — o outro extremo da régua, e o valor que faz o piso do GLSL
/// (`max(κ, 0,01)`) entregar o raio de `100` que a referência usa para uma superfície plana.
#[test]
fn um_plano_le_curvatura_zero() {
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Box {
                half: [4.0, 0.5, 4.0],
                round: 0.0,
                chamfer: 0.0,
            },
            Xform::IDENTITY,
        )],
        NodeId(0),
    )
    .expect("caixa");
    let reg = Registry::default();
    let mut eval = Hybrid::new(&doc, &reg);
    let eps = curvatura::eps_para(4.0);
    // No meio da face de cima, longe das quinas.
    let pontos: Vec<[f32; 3]> = (0..16)
        .map(|i| [(i as f32 - 8.0) * 0.2, 0.5, 0.0])
        .collect();
    let k = curvatura::curvaturas(&mut eval, &pontos, eps);
    let pior = k.iter().copied().fold(0.0_f32, f32::max);
    eprintln!("plano: pior curvatura {pior:.6}");
    assert!(pior <= 0.01, "a face plana leu curvatura {pior:.6}");
}

/// ⭐ **Sem pontos não há amostras**, e um `eps` impossível devolve zeros em vez de `NaN`.
///
/// ⚠️ *Um `NaN` a atravessar o material lê-se como um pixel preto legítimo* — a guarda existe para
/// que a resposta de «não sei» seja a superfície plana, que é o que o piso do GLSL já significa.
#[test]
fn a_guarda_devolve_zeros_e_nunca_nan() {
    let doc = esfera(1.0);
    let reg = Registry::default();
    let mut eval = Hybrid::new(&doc, &reg);
    assert!(curvatura::curvaturas(&mut eval, &[], 1e-3).is_empty());
    let pontos = na_superficie(1.0, 4);
    for eps in [0.0_f32, -1.0, f32::NAN] {
        let k = curvatura::curvaturas(&mut eval, &pontos, eps);
        assert_eq!(k.len(), pontos.len());
        assert!(k.iter().all(|v| *v == 0.0), "eps {eps} devolveu {k:?}");
    }
}

/// ⏱️ **SONDA — que PASSO a segunda diferença precisa.**
#[test]
#[ignore = "sonda: imprime uma tabela"]
fn sonda_o_passo_da_segunda_diferenca() {
    for raio in [0.4_f32, 1.0, 2.5] {
        let doc = esfera(raio);
        let reg = Registry::default();
        let mut eval = Hybrid::new(&doc, &reg);
        let pts = na_superficie(raio, 32);
        eprintln!("--- raio {raio} (verdade {:.4}) ---", 1.0 / raio);
        for k in 0..14 {
            let eps = raio * 1.0e-4 * f32::powi(2.0, k);
            let c = curvatura::curvaturas(&mut eval, &pts, eps);
            let med = c.iter().sum::<f32>() / c.len() as f32;
            let pior = c
                .iter()
                .map(|v| (v - 1.0 / raio).abs() / (1.0 / raio))
                .fold(0.0_f32, f32::max);
            eprintln!(
                "  eps {:.6} ({:.5} do raio) → média {med:.4} · pior rel {pior:.4}",
                eps,
                eps / raio
            );
        }
    }
}
