//! ⭐⭐⭐ **OS GATES DA LEI DOS ATRIBUTOS** ([`crate::attr_law`]) — filho do arnês do refinamento
//! para herdar a `malha()` dele.
//!
//! O report que os trouxe (dono, 2026-09-16): *«Smooth parece ter resultado discretamente inferior,
//! gerando micro irregularidades»*. A tabela do mecanismo está no cabeçalho do `attr_law.rs`.
//!
//! ⚠️ **Nenhuma fixtura aqui é alinhada aos eixos quando a pergunta é uma DIRECÇÃO** — uma aresta
//! horizontal deixa o termo transversal a zero dos dois lados, e a metade do gradiente que a lei
//! promete não teria como reprovar.

use super::*;
use crate::attr_law::midpoint;
use crate::{AttrLaw, RefineLaw, hermite_attrs, recover_gradients, refine_posed_with};

/// As opções de um caso, com a lei escolhida.
fn opts(tolerance_px: f64, max_pieces: usize, adaptativo: bool) -> RefineOptions {
    RefineOptions {
        tolerance_px,
        max_pieces,
        adaptativo,
    }
}

/// Uma quadrática da arte (`320×96`) com termo cruzado — e o gradiente EXACTO dela.
fn quadratica(p: [f64; 2]) -> (f64, [f64; 2]) {
    let (u, v) = ((p[0] - 160.0) / 160.0, (p[1] - 48.0) / 48.0);
    let w = 0.2 + 0.5 * u * u + 0.3 * u * v - 0.1 * v * v;
    let g = [(u + 0.3 * v) / 160.0, (0.3 * u - 0.2 * v) / 48.0];
    (w, g)
}

/// ⭐ **Um campo de PELE**: roda cada ponto em torno do centro por um ângulo que é o PESO `0` — é o
/// que torna a deformação função dos atributos, como no produto.
fn pele(t: f64) -> impl FnMut([f64; 2], &[f64]) -> [f64; 2] {
    move |p: [f64; 2], w: &[f64]| {
        let s = w.first().copied().unwrap_or(0.0) * t;
        let (sin, cos) = (s.sin(), s.cos());
        let (x, y) = (p[0] - 160.0, p[1] - 48.0);
        [
            (x * cos - y * sin) + 160.0,
            (x * sin).mul_add(1.0, y * cos) + 48.0,
        ]
    }
}

/// ⭐ **O gradiente recuperado é EXACTO num campo linear, em TODO vértice** — interior e bordo.
///
/// É a propriedade que faz a lei de Hermite não mexer num campo que o P1 já representava bem.
#[test]
fn recovered_gradients_are_exact_on_a_linear_field() {
    let m = malha();
    let valores: Vec<f64> = m
        .rest
        .iter()
        .flat_map(|p| [0.3 + 0.002 * p[0] - 0.005 * p[1], -0.7 * p[0] + 0.4 * p[1]])
        .collect();
    let g = recover_gradients(&m, &valores, 2);
    assert_eq!(g.len(), m.rest.len() * 4);
    let mut pior = 0.0_f64;
    for v in 0..m.rest.len() {
        for (k, esperado) in [[0.002, -0.005], [-0.7, 0.4]].iter().enumerate() {
            for eixo in 0..2 {
                pior = pior.max((g[(v * 2 + k) * 2 + eixo] - esperado[eixo]).abs());
            }
        }
    }
    assert!(m.rest.len() > 100, "a malha encolheu ({})", m.rest.len());
    assert!(
        pior < 1e-12,
        "o gradiente de um campo linear saiu errado ({pior:.3e})"
    );
}

/// ⭐ **O meio de uma aresta de Hermite é EXACTO num campo linear** — valor e gradiente, numa
/// aresta oblíqua.
#[test]
fn the_hermite_midpoint_is_exact_on_a_linear_field() {
    let g = [0.013, -0.041];
    let f = |p: [f64; 2]| 0.25 + g[0] * p[0] + g[1] * p[1];
    let (ra, rb) = ([3.0, 7.0], [41.0, -19.0]);
    let a = [f(ra), g[0], g[1]];
    let b = [f(rb), g[0], g[1]];
    let mut out = [0.0; 3];
    midpoint(AttrLaw::Hermite { values: 1 }, ra, rb, &a, &b, &mut out);
    let rm = [f64::midpoint(ra[0], rb[0]), f64::midpoint(ra[1], rb[1])];
    assert!(
        (out[0] - f(rm)).abs() < 1e-14,
        "valor {} contra {}",
        out[0],
        f(rm)
    );
    assert!(
        (out[1] - g[0]).abs() < 1e-14 && (out[2] - g[1]).abs() < 1e-14,
        "{out:?}"
    );
}

/// ⭐⭐⭐ **Uma CÚBICA ao longo da aresta sai exacta TRÊS níveis abaixo** — e é o segundo nível que
/// prova a correcção do declive: o vértice novo tem de nascer com a derivada que a cúbica tem ali,
/// senão o neto já herda uma tangente errada.
///
/// (Mutações: apagar o `(d_a − d_b)/8` ⇒ RED no 1.º nível; apagar a correcção do declive do
/// gradiente ⇒ RED no 2.º.)
#[test]
fn a_cubic_along_the_edge_survives_three_generations() {
    // Direcção OBLÍQUA de propósito — ver o cabeçalho do ficheiro.
    let (ra, dir) = ([5.0, -2.0], [0.6, 0.8]);
    let f = |s: f64| 0.1 + 0.4 * s - 0.03 * s * s + 0.002 * s * s * s;
    let df = |s: f64| 0.4 - 0.06 * s + 0.006 * s * s;
    let no = |s: f64| -> ([f64; 2], [f64; 3]) {
        let p = [ra[0] + dir[0] * s, ra[1] + dir[1] * s];
        (p, [f(s), df(s) * dir[0], df(s) * dir[1]])
    };
    let lei = AttrLaw::Hermite { values: 1 };
    // Bissecta sempre a metade ESQUERDA: 0–L, 0–L/2, 0–L/4.
    let (a_s, mut b_s) = (0.0_f64, 20.0_f64);
    let (pa, aa) = no(a_s);
    let (mut pb, mut ab) = no(b_s);
    for nivel in 1..=3 {
        let mut out = [0.0; 3];
        midpoint(lei, pa, pb, &aa, &ab, &mut out);
        let m_s = f64::midpoint(a_s, b_s);
        let (pm, exacto) = no(m_s);
        for c in 0..3 {
            assert!(
                (out[c] - exacto[c]).abs() < 1e-12,
                "nivel {nivel}, componente {c}: {} contra {}",
                out[c],
                exacto[c]
            );
        }
        // O filho seguinte nasce da ponta esquerda e do vértice que a lei ACABOU de inventar.
        (b_s, pb, ab) = (m_s, pm, out);
    }
}

/// ⭐⭐⭐ **A PARTIÇÃO DA UNIDADE SOBREVIVE AO REFINAMENTO, sem renormalizar.**
///
/// ⚠️ Pesos que somam `1` em cada vértice do bind; a lei é linear nos valores e nos gradientes, e
/// os gradientes recuperados somam `0` — logo cada geração soma `1` até ao arredondamento. ⛔ Por
/// isso a lei não corta nem renormaliza (um `max(0, ·)` poria de volta os vincos).
#[test]
fn the_partition_of_unity_survives_the_hermite_refinement() {
    let m = malha();
    let focos = [[40.0, 20.0], [160.0, 70.0], [290.0, 40.0]];
    let pesos: Vec<f64> = m
        .rest
        .iter()
        .flat_map(|p| {
            let e: Vec<f64> = focos
                .iter()
                .map(|c| (-((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)) / 6000.0).exp())
                .collect();
            let soma: f64 = e.iter().sum();
            e.into_iter().map(move |x| x / soma)
        })
        .collect();
    let attrs = hermite_attrs(&m, &pesos, 3);
    let lei = AttrLaw::Hermite { values: 3 };
    let mut campo = pele(1.5);
    let (r, _p, saida, rel) = refine_posed_with(
        &m,
        &attrs,
        9,
        lei,
        &mut campo,
        opts(0.05, m.tris.len() * 40, true),
    );
    assert!(
        matches!(rel.lei, RefineLaw::Adaptive { rondas } if rondas > 100),
        "o caso nao refinou o bastante para testar geracoes: {:?}",
        rel.lei
    );
    let (mut pior_soma, mut pior_grad, mut menor) = (0.0_f64, 0.0_f64, f64::MAX);
    for v in 0..r.rest.len() {
        let a = &saida[v * 9..(v + 1) * 9];
        pior_soma = pior_soma.max((a[0] + a[1] + a[2] - 1.0).abs());
        pior_grad = pior_grad.max((a[3] + a[5] + a[7]).abs().max((a[4] + a[6] + a[8]).abs()));
        menor = menor.min(a[0].min(a[1]).min(a[2]));
    }
    println!(
        "{} -> {} vertices | |soma - 1| {pior_soma:.2e} | |soma dos gradientes| {pior_grad:.2e} | \
         menor peso {menor:.2e}",
        m.rest.len(),
        r.rest.len()
    );
    assert!(
        pior_soma < 1e-13,
        "a particao da unidade derivou ({pior_soma:.3e})"
    );
    assert!(
        pior_grad < 1e-15,
        "os gradientes deixaram de somar zero ({pior_grad:.3e})"
    );
}

/// ⭐⭐⭐⭐ **A PORTA ADAPTATIVA LÊ A LEI QUE RECEBE** — numa quadrática com gradientes exactos, a
/// lei de Hermite dá o valor exacto em TODO vértice inventado, a qualquer profundidade; a linear
/// erra pela curvatura.
///
/// ⚠️ **Exacta a qualquer profundidade** porque numa quadrática o gradiente é LINEAR: a média das
/// pontas é o gradiente do meio, e a correcção do declive é zero.
///
/// (Mutação: o despachante ou a obra ignorarem `law` ⇒ RED.)
#[test]
fn the_adaptive_door_reads_the_law_it_is_given() {
    let m = malha();
    let mut hermite = Vec::new();
    let mut linear = Vec::new();
    for &p in &m.rest {
        let (w, g) = quadratica(p);
        hermite.extend_from_slice(&[w, 1.0 - w, g[0], g[1], -g[0], -g[1]]);
        linear.extend_from_slice(&[w, 1.0 - w]);
    }
    let o = opts(0.05, m.tris.len() * 40, true);
    let erro = |r: &Mesh2d, saida: &[f64], stride: usize| -> f64 {
        r.rest
            .iter()
            .enumerate()
            .map(|(v, &q)| (saida[v * stride] - quadratica(q).0).abs())
            .fold(0.0_f64, f64::max)
    };

    let mut campo = pele(1.5);
    let lei = AttrLaw::Hermite { values: 2 };
    let (rh, _, sh, relh) = refine_posed_with(&m, &hermite, 6, lei, &mut campo, o);
    let mut campo = pele(1.5);
    let (rl, _, sl, _) = refine_posed_with(&m, &linear, 2, AttrLaw::Linear, &mut campo, o);
    let (eh, el) = (erro(&rh, &sh, 6), erro(&rl, &sl, 2));
    println!(
        "Hermite: {} vertices, erro {eh:.2e} ({:?}) | linear: {} vertices, erro {el:.2e}",
        rh.rest.len(),
        relh.lei,
        rl.rest.len()
    );
    assert!(rh.rest.len() > m.rest.len(), "o caso nao refinou");
    assert!(
        eh < 1e-12,
        "a lei de Hermite errou uma quadratica ({eh:.3e})"
    );
    // ⛔ O CONTROLO: a linear erra — senão a fixtura não distingue as duas leis.
    assert!(
        el > 1e-3,
        "a lei linear acertou a quadratica ({el:.3e}) — a fixtura e' cega"
    );
}

/// ⛔⛔ **A LEI UNIFORME É O CAMINHO DE ANTES, com qualquer lei de atributos** — declarado na
/// [`refine_posed_with`]. Uma porta de bissecção tem de reproduzir o produto antigo AO BIT.
#[test]
fn a_lei_uniforme_e_o_caminho_de_antes_com_qualquer_lei_de_atributos() {
    let m = malha();
    let pesos: Vec<f64> = m
        .rest
        .iter()
        .flat_map(|p| {
            let w = quadratica(*p).0;
            [w, 1.0 - w]
        })
        .collect();
    let attrs = hermite_attrs(&m, &pesos, 2);
    let o = opts(0.05, m.tris.len() * 36, false);
    let mut campo = pele(1.5);
    let (r1, p1, s1, rel1) = refine_posed_with(&m, &pesos, 2, AttrLaw::Linear, &mut campo, o);
    let mut campo = pele(1.5);
    let lei = AttrLaw::Hermite { values: 2 };
    let (r2, p2, s2, rel2) = refine_posed_with(&m, &attrs, 6, lei, &mut campo, o);
    assert!(
        matches!(rel1.lei, RefineLaw::Uniform { k } if k > 1),
        "a lei uniforme nao refinou: {:?}",
        rel1.lei
    );
    assert_eq!(rel1.lei, rel2.lei);
    assert_eq!(r1.tris, r2.tris);
    assert_eq!(p1, p2, "as posicoes mudaram com a lei dos atributos");
    let valores: Vec<f64> = s2.chunks(6).flat_map(|c| [c[0], c[1]]).collect();
    assert_eq!(
        s1, valores,
        "os pesos da lei uniforme mudaram com a lei dos atributos"
    );
}

/// ⭐⭐ **A RÉGUA LÊ A MESMA LEI QUE O REFINADOR** — o que a lei adaptativa entrega sem travar no
/// orçamento está dentro da tolerância **medido pela [`crate::deviation_with`] com a mesma lei**.
///
/// (Mutação: a régua ignorar `law` ⇒ RED — ela passaria a medir o campo P1, que a malha não segue.)
#[test]
fn the_deviation_ruler_reads_the_law_the_refinement_used() {
    let m = malha();
    let focos = [[60.0, 48.0], [260.0, 48.0]];
    let pesos: Vec<f64> = m
        .rest
        .iter()
        .flat_map(|p| {
            let e: Vec<f64> = focos
                .iter()
                .map(|c| (-((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2)) / 3000.0).exp())
                .collect();
            let soma: f64 = e.iter().sum();
            e.into_iter().map(move |x| x / soma)
        })
        .collect();
    let attrs = hermite_attrs(&m, &pesos, 2);
    let lei = AttrLaw::Hermite { values: 2 };
    let tol = 0.05;
    let mut campo = pele(2.0);
    let (r, p, s, rel) = refine_posed_with(
        &m,
        &attrs,
        6,
        lei,
        &mut campo,
        opts(tol, m.tris.len() * 80, true),
    );
    assert!(!rel.travado_pelo_orcamento, "o caso travou no orcamento");
    let mut campo = pele(2.0);
    let com_a_lei = crate::deviation_with(&r, &p, &s, 6, lei, &mut campo);
    let mut campo = pele(2.0);
    let linear = crate::deviation_with(&r, &p, &s, 6, AttrLaw::Linear, &mut campo);
    println!(
        "{} pecas | desvio com a lei {com_a_lei:.4} | com a linear {linear:.4}",
        r.tris.len()
    );
    assert!(
        com_a_lei <= tol + 1e-12,
        "a regua da lei le {com_a_lei:.4} > {tol}"
    );
    // ⛔ O CONTROLO: a régua linear lê OUTRO campo nesta malha.
    assert!(
        linear > tol,
        "a regua linear tambem cabe ({linear:.4}) — a fixtura e' cega"
    );
}
