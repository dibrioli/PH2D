//! ⭐⭐⭐ **A DERIVADA DA MEMBRANA, conferida contra a ENERGIA.**
//!
//! ⚠️ **O oráculo é a energia, e é por isso que ele vale.** Um gate que
//! reescrevesse a fórmula do gradiente seria um espelho — ele ficaria verde sobre
//! a mesma álgebra errada. A energia é inequívoca (é a definição do material) e a
//! derivada é exatamente onde um sinal trocado passa despercebido: com diferenças
//! finitas da energia, um sinal errado **sangra**.

use crate::{ClothMaterial, ClothRest, ClothTopology, V3, energy, fixtures, membrane};

fn mat() -> ClothMaterial {
    ClothMaterial {
        young: 500.0,
        poisson: 0.31,
        bending: 0.0,
        ..ClothMaterial::default()
    }
}

/// A energia da malha inteira, para as diferenças finitas atravessarem a PORTA
/// pública — e não uma cópia da lei escrita no teste.
fn e_of(topo: &ClothTopology, rest: &ClothRest, x: &[V3]) -> f64 {
    energy(topo, rest, &mat(), x)
}

/// ⭐⭐⭐ **GATE — o repouso não tem energia nem gradiente.**
///
/// ⛔ É o controle positivo de toda a suíte: se a membrana empurrasse no repouso,
/// o pincel mexeria na peça só por encostar. Com `G = 0` a conta é zero **ao
/// bit**, e é por isso que a barra aqui é `0.0` e não um epsilon.
#[test]
fn o_repouso_nao_tem_energia_nem_gradiente() {
    let (x, t) = fixtures::triangle();
    let topo = fixtures::region(&x, &[t]);
    let rest = ClothRest::measure(&topo, &x, &mat());
    assert_eq!(e_of(&topo, &rest, &x), 0.0);
    let (mu, lambda) = mat().lame();
    for slot in 0..3 {
        let (g, _) = membrane::accumulate(&x, t, &rest.tri[0], mu, lambda, slot);
        assert_eq!(g, [0.0; 3], "slot {slot} empurra no repouso");
    }
}

/// ⭐⭐⭐ **GATE — o gradiente É a derivada da energia.**
#[test]
fn o_gradiente_bate_com_a_diferenca_finita_da_energia() {
    let (x0, t) = fixtures::triangle();
    let topo = fixtures::region(&x0, &[t]);
    let rest = ClothRest::measure(&topo, &x0, &mat());
    // Deformada de verdade: estica, cisalha e sai do plano.
    let mut x = x0.clone();
    x[1] = [1.5, 0.3, 0.1];
    x[2] = [0.2, 1.4, 0.9];
    let (mu, lambda) = mat().lame();

    let h = 1e-6;
    let mut pior = 0.0f64;
    for slot in 0..3 {
        let (g, _) = membrane::accumulate(&x, t, &rest.tri[0], mu, lambda, slot);
        for c in 0..3 {
            let (mut a, mut b) = (x.clone(), x.clone());
            a[slot][c] += h;
            b[slot][c] -= h;
            let fd = (e_of(&topo, &rest, &a) - e_of(&topo, &rest, &b)) / (2.0 * h);
            pior = pior.max((fd - g[c]).abs() / fd.abs().max(1.0));
        }
    }
    assert!(pior < 1e-6, "gradiente contra diferenca finita: {pior:.3e}");
}

/// ⭐⭐⭐ **GATE — a Hessiana É a derivada do gradiente.**
///
/// ⚠️ Ela é a métrica que orienta o passo do VBD; errada, o solver não fica
/// errado — fica **lento**, e lento num pincel lê-se como *"o tecido não responde"*.
#[test]
fn a_hessiana_bate_com_a_diferenca_finita_do_gradiente() {
    let (x0, t) = fixtures::triangle();
    let topo = fixtures::region(&x0, &[t]);
    let rest = ClothRest::measure(&topo, &x0, &mat());
    let mut x = x0.clone();
    x[1] = [1.4, 0.25, 0.05];
    x[2] = [0.3, 1.3, 0.7];
    let (mu, lambda) = mat().lame();

    let h = 1e-6;
    let mut pior = 0.0f64;
    for slot in 0..3 {
        let (_, hess) = membrane::accumulate(&x, t, &rest.tri[0], mu, lambda, slot);
        for c in 0..3 {
            let (mut a, mut b) = (x.clone(), x.clone());
            a[slot][c] += h;
            b[slot][c] -= h;
            let ga = membrane::accumulate(&a, t, &rest.tri[0], mu, lambda, slot).0;
            let gb = membrane::accumulate(&b, t, &rest.tri[0], mu, lambda, slot).0;
            for r in 0..3 {
                let fd = (ga[r] - gb[r]) / (2.0 * h);
                pior = pior.max((fd - hess[r][c]).abs() / fd.abs().max(1.0));
            }
        }
    }
    assert!(pior < 1e-5, "hessiana contra diferenca finita: {pior:.3e}");
}

/// ⭐⭐⭐ **GATE — a energia não sabe onde a peça está nem para onde ela aponta.**
///
/// ⛔ Sem isto, mover o modelo na cena mudaria como o tecido se comporta — o
/// defeito que a `line/quadextract` mediu no remesh e que custou uma jornada.
#[test]
fn a_energia_e_invariante_a_pose() {
    let (x0, t) = fixtures::triangle();
    let topo = fixtures::region(&x0, &[t]);
    let rest = ClothRest::measure(&topo, &x0, &mat());
    let mut x = x0.clone();
    x[1] = [1.5, 0.3, 0.1];
    let base = e_of(&topo, &rest, &x);
    assert!(base > 0.0, "a fixtura nao contem o fenomeno: energia zero");

    // Roda 40° em torno de (1,1,1) normalizado, e translada.
    let (s, c) = 0.698_131_7f64.sin_cos();
    let k = 1.0 / 3.0f64.sqrt();
    let posed: Vec<V3> = x
        .iter()
        .map(|p| {
            let a = [p[0], p[1], p[2]];
            let kd = k * (a[0] + a[1] + a[2]);
            let cr = [k * (a[1] - a[2]), k * (a[2] - a[0]), k * (a[0] - a[1])];
            [
                a[0] * c + cr[0] * s + k * kd * (1.0 - c) + 7.5,
                a[1] * c + cr[1] * s + k * kd * (1.0 - c) - 3.25,
                a[2] * c + cr[2] * s + k * kd * (1.0 - c) + 11.0,
            ]
        })
        .collect();
    let moved = e_of(&topo, &rest, &posed);
    assert!(
        (moved - base).abs() / base < 1e-12,
        "a energia mudou com a pose: {base:.9e} contra {moved:.9e}"
    );
}

/// ⭐⭐⭐ **GATE — a Hessiana da membrana é SEMI-DEFINIDA POSITIVA, inclusive em
/// COMPRESSÃO.**
///
/// ⛔⛔⛔ **O buraco de simetria que este gate fecha, e o que ele custou.** A dobra
/// tinha [`a_hessiana_da_dobra_e_semi_definida_positiva`](super::bending_tests)
/// desde que existe; a membrana **não tinha o gate irmão**, e o gate que ela
/// tinha (`a_hessiana_bate_com_a_diferenca_finita_do_gradiente`) prova que ela
/// está **CERTA**. *Uma Hessiana indefinida CORRETA é precisamente o defeito:
/// nenhuma régua desta crate perguntava se ela era **utilizável**.*
///
/// Medido em 2026-09-05, sem a projeção: a `35 %` de compressão a energia de um
/// retalho ia de `6,4e1` a **`5,26e8`** num sub-passo e um vértice andava `20×` a
/// peça — e refinar a amostragem da compressão `10×` fazia o pico **não
/// convergir** (`0,9 → 4,9e5`), a assinatura de `det H` a cruzar o zero.
///
/// ⚠️ **A forma quadrática é amostrada, e não os autovalores.** Para um bloco
/// `3×3` simétrico, `vᵀHv ≥ 0` numa rede densa de direções é a definição
/// operacional, e não pede um decompositor no caminho de um teste. As direções
/// saem de uma espiral de Fibonacci, que é **determinística** (a lei do
/// `BTreeMap` desta casa vale para réguas também).
#[test]
fn a_hessiana_da_membrana_e_semi_definida_positiva() {
    let (x0, t) = fixtures::triangle();
    let rest = membrane::rest_of(&x0, t);
    let (mu, lambda) = mat().lame();

    // 512 direções unitárias por espiral de Fibonacci.
    let dirs: Vec<V3> = (0..512)
        .map(|k| {
            let z = 1.0 - 2.0 * (f64::from(k) + 0.5) / 512.0;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let a = std::f64::consts::PI * (3.0 - 5.0f64.sqrt()) * f64::from(k);
            [r * a.cos(), r * a.sin(), z]
        })
        .collect();

    let mut pior = 0.0f64;
    let mut escala = 0.0f64;
    // ⚠️ **O CONTROLE: a varredura tem de ALCANÇAR o regime não-convexo.** Sem
    // ele, um `c` que nunca comprime deixaria a asserção verde por vácuo — e o
    // regime é nomeado pelo 2.º Piola-Kirchhoff a ficar negativo.
    let mut viu_compressao = false;

    for k in 0..=80 {
        let c = 1.0 - 0.01 * f64::from(k); // 1,00 … 0,20
        let x: Vec<V3> = x0.iter().map(|p| [p[0] * c, p[1] * c, p[2] * c]).collect();
        let f = membrane::deform(&x, t, &rest.dm_inv);
        let (_, s) = membrane::strain(&f, &rest.metric, mu, lambda);
        if s[0][0] < 0.0 || s[1][1] < 0.0 {
            viu_compressao = true;
        }
        for slot in 0..3 {
            let (_, h) = membrane::accumulate(&x, t, &rest, mu, lambda, slot);
            escala = escala.max(h.iter().flatten().fold(0.0f64, |m, v| m.max(v.abs())));
            for d in &dirs {
                let q: f64 = (0..3)
                    .map(|r| d[r] * (0..3).map(|c2| h[r][c2] * d[c2]).sum::<f64>())
                    .sum();
                pior = pior.min(q);
            }
        }
    }

    assert!(
        viu_compressao,
        "a varredura nunca comprimiu o triangulo -- este gate estaria verde por vacuo"
    );
    assert!(
        pior >= -1e-9 * escala.max(1.0),
        "a Hessiana da membrana e' INDEFINIDA: a forma quadratica desce a {pior:.4e} \
         (escala do bloco {escala:.4e}). Sem a projecao de `wsw`, o passo de Newton \
         e' um POLO em compressao"
    );
}
