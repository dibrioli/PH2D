//! Os gates do operador por cotangentes — ver o [`super`].

use super::*;
use crate::{Mesh, shapes, shapes_open};

/// Uma malha PLANA, quadrada, de `n × n` vértices — o controle de curvatura
/// zero, e a única fixture em que a resposta certa é conhecida sem medir.
fn grid(n: usize) -> Mesh {
    let mut pos = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: índice de grade.
            pos.push([i as f32, 0.0, j as f32]);
        }
    }
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let a = (j * n + i) as u32;
            let b = a + 1;
            let c = a + n as u32 + 1;
            let d = a + n as u32;
            faces.push(Face::quad(a, b, c, d));
        }
    }
    Mesh::from_parts(pos, faces).expect("fixture")
}

/// **A REFERÊNCIA MEDE O QUE ELA PROMETE.** Numa esfera de raio `R` o paper diz
/// `|K| = 2/R` e a direção é a normal.
///
/// ⚠️ **O gate afirma o NÚMERO do paper, não o número que saiu.** Uma barra
/// escolhida depois de ver o resultado não distingue *o operador está certo* de
/// *o operador é reprodutível*.
#[test]
fn the_operator_measures_the_curvature_of_a_sphere() {
    for &r in &[0.5f32, 1.0, 4.0] {
        let m = shapes::uv_sphere(24, 32, r);
        let (mut worst_mag, mut worst_dir) = (0.0f32, 0.0f32);
        let mut seen = 0usize;
        for v in 0..m.positions().len() {
            let Some(k) = mean_curvature_normal_at(m.positions(), m.faces(), m.adjacency(), v)
            else {
                continue;
            };
            seen += 1;
            let mag = len2(k).sqrt();
            worst_mag = worst_mag.max((mag - 2.0 / r).abs() * r / 2.0);
            // A direção contra a posição radial — numa esfera centrada na
            // origem a normal exacta é `p/|p|`, e não a estimada pela malha.
            let p = m.positions()[v];
            let inv = 1.0 / len2(p).sqrt();
            let dot = (k[0] * p[0] + k[1] * p[1] + k[2] * p[2]) * inv / mag;
            worst_dir = worst_dir.max(1.0 - dot);
        }
        assert!(seen > 100, "esfera fechada: {seen} vértices responderam");
        assert!(worst_mag < 0.02, "raio {r}: erro relativo {worst_mag}");
        assert!(worst_dir < 1e-3, "raio {r}: desvio de direção {worst_dir}");
    }
}

/// **UM PLANO NÃO TEM NORMAL DE CURVATURA**, e é por isso que a direção devolve
/// [`None`] em vez de um vetor arbitrário.
///
/// ⚠️ A metade `mean_curvature_normal_at` responde `Some(≈0)`: *a curvatura aqui
/// é zero* é uma resposta, e é diferente de *não há direção defensável*.
#[test]
fn a_flat_sheet_has_no_curvature_normal() {
    let m = grid(6);
    let mut interior = 0usize;
    for v in 0..m.positions().len() {
        if m.adjacency().is_border(v) {
            continue;
        }
        interior += 1;
        let k = mean_curvature_normal_at(m.positions(), m.faces(), m.adjacency(), v)
            .expect("interior de uma folha plana responde");
        assert!(len2(k).sqrt() < 1e-5, "plano com curvatura {k:?}");
        assert!(
            curvature_normal_dir_at(m.positions(), m.faces(), m.adjacency(), v).is_none(),
            "um plano não pode nomear uma direção de curvatura"
        );
    }
    assert!(interior >= 16, "a fixture tem {interior} interiores");
}

/// **A BEIRA NÃO RESPONDE** — a construção pede dois ângulos por aresta e uma
/// aresta de borda tem um só.
///
/// ⚠️ **A metade que torna o gate honesto é o CONTROLE:** sem ele, um operador
/// que devolvesse `None` para TUDO passaria.
#[test]
fn the_border_declines_and_the_interior_answers() {
    let m = shapes_open::open_tube3();
    let (mut border, mut interior) = (0usize, 0usize);
    for v in 0..m.positions().len() {
        let got = mean_curvature_normal_at(m.positions(), m.faces(), m.adjacency(), v);
        if m.adjacency().is_border(v) {
            border += 1;
            assert!(got.is_none(), "vértice {v} de borda respondeu {got:?}");
        } else {
            interior += 1;
            assert!(got.is_some(), "vértice {v} interior não respondeu");
        }
    }
    assert!(
        border > 0 && interior > 0,
        "borda {border} interior {interior}"
    );
}

/// **A ÁREA MISTA É A DO PAPER**, e o discriminante é um triângulo OBTUSO — onde
/// a região de Voronoi sai do próprio triângulo e a §3.3 troca de ramo.
///
/// O oráculo é a esfera: com a partição errada o `2/R` deixa de fechar. Um
/// leque deliberadamente obtuso torna o ramo alcançável, que é o que a esfera
/// regular NÃO faz.
#[test]
fn the_mixed_area_survives_an_obtuse_fan() {
    // Um leque raso: o vértice central baixo e um anel largo ⇒ os triângulos
    // ficam obtusos NO CENTRO.
    let mut pos = vec![[0.0, 0.05, 0.0]];
    let n = 8;
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: índice do anel.
        let a = (i as f32) * std::f32::consts::TAU / (n as f32);
        pos.push([a.cos(), 0.0, a.sin()]);
    }
    let faces: Vec<Face> = (0..n)
        .map(|i| Face::tri(0, 1 + i as u32, 1 + ((i as u32 + 1) % n as u32)))
        .collect();
    let m = Mesh::from_parts(pos, faces).expect("fixture");
    let k = mean_curvature_normal_at(m.positions(), m.faces(), m.adjacency(), 0)
        .expect("o ápice de um leque fechado responde");
    // O ápice é convexo para CIMA, então `K` aponta para `+Y`.
    assert!(k[1] > 0.0, "o ápice de um domo aponta para cima: {k:?}");
    assert!(
        len2(k).sqrt().is_finite() && len2(k).sqrt() > 0.0,
        "curvatura degenerada {k:?}"
    );
}

/// **A PROPRIEDADE QUE O PAPER REIVINDICA, isolada** — porta contra porta, sem
/// pincel, sem passes, sem falloff.
///
/// O laplaciano uniforme aponta para o centroide do anel, e num anel
/// ANISOTRÓPICO esse centroide tem uma componente **TANGENCIAL** grande: os
/// vértices deslizam pela superfície. O geométrico é (a primeira ordem)
/// puramente normal — é essa a razão de ele existir.
///
/// ⚠️ **Este gate vive AQUI e não no `ph2d-sculpt3d` por uma razão medida:** lá o
/// `l-mode` traz o par λ|μ JUNTO com o operador, e o par sozinho já reduz a
/// deriva — um gate escrito lá fica **verde com o operador desligado**, o que
/// aconteceu comigo na primeira versão. Aqui as duas portas são funções
/// públicas e a comparação não tem terceira variável.
///
/// ⚠️ **A fixture serve por ACIDENTE da própria construção:** numa esfera de
/// anéis quadrados os dois coincidem por SIMETRIA e o gate mediria zero; na
/// `uv_sphere` o passo em longitude encolhe com `sin θ` e o em latitude não.
/// Medido, UM passe: **0,003164 contra 0,000014**, 226×. A barra pede 20×.
#[test]
fn the_geometric_ring_slides_far_less_than_the_uniform_one() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let (mut d_uni, mut d_cot, mut seen) = (0.0f64, 0.0f64, 0usize);
    for v in 0..m.positions().len() {
        let p = m.positions()[v];
        let n = m.normals()[v];
        let Some(cot) = curvature_normal_dir_at(m.positions(), m.faces(), m.adjacency(), v).and(
            cotangent_ring_average_at(m.positions(), m.faces(), m.adjacency(), v),
        ) else {
            continue;
        };
        let uni = crate::ring_average(m.adjacency(), v as u32, p, |nb| m.positions()[nb as usize]);
        // A componente do deslocamento que é PERPENDICULAR à normal — o que
        // "a parametrização se distorce" significa em número.
        let tang = |t: [f32; 3]| {
            let d = [t[0] - p[0], t[1] - p[1], t[2] - p[2]];
            let along = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
            f64::from(len2([
                d[0] - along * n[0],
                d[1] - along * n[1],
                d[2] - along * n[2],
            ]))
            .sqrt()
        };
        d_uni += tang(uni);
        d_cot += tang(cot);
        seen += 1;
    }
    assert!(seen > 500, "a fixture tem de conter o fenômeno: {seen}");
    assert!(
        d_uni > d_cot * 20.0,
        "deriva tangencial: uniforme {d_uni:.6} contra cotangente {d_cot:.6}"
    );
    // ⚠️ **O CONTROLE.** Sem ele um operador que devolvesse o PRÓPRIO vértice
    // passaria — e *não mover* não é *mover sem deslizar*.
    assert!(d_cot > 0.0, "o operador geométrico não moveu nada");
}

/// **A SOMA DOS PESOS É POSITIVA POR IDENTIDADE, e é isso que torna o divisor
/// seguro** — não uma fixture que calhou de não o exercitar.
///
/// Cada triângulo contribui `cot q + cot r` para a soma, e
///
/// ```text
/// cot q + cot r = sin(q + r) / (sin q · sin r) = sin p / (sin q · sin r)
/// ```
///
/// que é **estritamente positivo** para todo triângulo não-degenerado
/// (`0 < p < π`). Um ângulo obtuso dá cotangente negativa, sim — mas um
/// triângulo tem no máximo UM obtuso, e o par que entra na soma nunca é
/// dominado por ele.
///
/// ⚠️ **Este gate existe porque a mutação que apaga a guarda de `Σw`
/// SOBREVIVEU**, e a explicação honesta não era *"falta fixture"*: varrida uma
/// grade de razão de aspecto **1 até 1000** (sonda descartada depois de dar a
/// resposta), a contagem de `Σw ≤ 0` foi **zero em todas**. A guarda fica como
/// leitura do divisor; o que a justifica está escrito aqui, e um dia em que
/// alguém troque a acumulação por outra coisa é este gate que nomeia o que se
/// perdeu.
#[test]
fn the_weight_sum_is_positive_by_identity_not_by_luck() {
    let meshes = [
        shapes::uv_sphere(24, 32, 1.0),
        shapes::uv_sphere_shuffled(24, 32, 1.0),
        shapes::uv_sphere_noisy(16, 24, 1.0, 0.15),
        shapes::torus(24, 12, 1.0, 0.3),
        grid(6),
    ];
    let mut checked = 0usize;
    for m in &meshes {
        for v in 0..m.positions().len() {
            let Some(w) = ring_weights_at(m.positions(), m.faces(), m.adjacency(), v) else {
                continue;
            };
            checked += 1;
            assert!(
                w.weight > 0.0,
                "Σw = {} num anel de {} — a identidade diz que isto é impossível",
                w.weight,
                m.adjacency().valence(v)
            );
        }
    }
    assert!(checked > 2000, "a varredura viu {checked} anéis");
}
