//! ⭐⭐⭐ **O CAMPO COM ARCOS É O MESMO CAMPO** — o gate que decide a wave de 2026-09-16.
//!
//! A decomposição exacta existe para a marcha avaliar `22` primitivas onde avaliava `94`. Ela só
//! vale se o **campo não mudar**: a distância e — sobretudo — o **SINAL**.
//!
//! ⚠️ **O sinal é a metade que engana.** A distância a um arco é fácil de acertar e de conferir; o
//! enrolamento não. Entre a corda e o arco há uma **meia-lua** onde o polígono das cordas responde
//! *dentro* e a figura verdadeira responde *fora* (ou o contrário), e um erro ali é uma casca fina
//! de sinal trocado colada a cada quina — invisível numa amostragem grosseira e visível como um
//! risco no render. ⇒ a grelha é fina e a barra do sinal é **zero** longe do bordo.
//!
//! ⚠️ **O perfil é construído AQUI, à mão, e não vem do cozedor**: esta crate não depende dele, e
//! fazê-la depender para um teste poria uma aresta nova no grafo por causa de uma fixtura. O
//! caminho inteiro — desenho → cozedura → arcos → fita → placa — tem a sonda dele na
//! `ph2d-app-field3d`.

use ph2d_field::{FillRule, Profile};

/// Um quadrado de lado `2·half` com as quatro quinas arredondadas a `r`, nas DUAS descrições.
///
/// A quina é um quarto de círculo ⇒ `bulge = tan(90°/4) = tan(22,5°)`. O sinal sai da orientação:
/// com os vértices em sentido anti-horário, o arco de uma quina convexa curva para a **direita** de
/// `a→b`, logo o bulge é negativo na convenção desta casa (positivo = esquerda).
fn quadrado_redondo(half: f64, r: f64, lados: usize) -> (Profile, Profile) {
    let bulge = -(std::f64::consts::FRAC_PI_8).tan();
    // Os oito vértices da decomposição: entrada e saída de cada quina, em sentido anti-horário.
    let cantos = [
        (half, half, 0.0_f64),   // canto +x +y
        (-half, half, 1.0),      // −x +y
        (-half, -half, 2.0),     // −x −y
        (half, -half, 3.0),      // +x −y
    ];
    let mut exacto: Vec<([f32; 2], f32)> = Vec::new();
    let mut denso: Vec<[f32; 2]> = Vec::new();
    for (cx, cy, q) in cantos {
        // O centro do arco desta quina, e os ângulos de entrada/saída.
        let (ox, oy) = (cx - r * cx.signum(), cy - r * cy.signum());
        // ângulo inicial do arco desta quina, em múltiplos de 90°
        let a0 = std::f64::consts::FRAC_PI_2 * q;
        let a1 = a0 + std::f64::consts::FRAC_PI_2;
        #[allow(clippy::cast_possible_truncation)]
        let ponto = |a: f64| [(ox + r * a.cos()) as f32, (oy + r * a.sin()) as f32];
        // A recta que CHEGA a esta quina termina no início do arco.
        exacto.push((ponto(a0), bulge as f32));
        denso.push(ponto(a0));
        for k in 1..lados {
            let a = a0 + (a1 - a0) * f64::from(u32::try_from(k).unwrap_or(1)) / lados as f64;
            denso.push(ponto(a));
        }
        // O fim do arco é o início da recta seguinte — entra como vértice de recta.
        exacto.push((ponto(a1), 0.0));
        denso.push(ponto(a1));
    }
    let poli: Vec<[f32; 2]> = exacto.iter().map(|(p, _)| *p).collect();
    let com = Profile::with_arcs(vec![(poli, exacto)], FillRule::NonZero, 1e-4)
        .expect("o quadrado redondo é válido");
    let sem = Profile::new(vec![denso], FillRule::NonZero, 1e-4).expect("a polilinha é válida");
    (com, sem)
}

fn avalia(p: &Profile, xs: &[f32], ys: &[f32]) -> Vec<f32> {
    use fidget::context::Tree;
    use fidget::shape::EzShape;
    let tree = crate::profile::sd_profile(p, &Tree::x(), &Tree::y());
    let shape = crate::Engine::from(tree);
    let mut eval = crate::Engine::new_float_slice_eval();
    let tape = shape.ez_float_slice_tape();
    let zs = vec![0.0f32; xs.len()];
    eval.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
}

#[test]
fn um_arco_da_o_mesmo_campo_que_a_polilinha_que_ele_substitui() {
    // `192` lados por quarto de volta: a polilinha de controlo é MUITO mais fina que a tolerância,
    // para que a discordância medida seja do ARCO e não da referência.
    let (com, sem) = quadrado_redondo(0.5, 0.15, 192);
    assert_eq!(com.arc_count(), 4, "as quatro quinas têm de ser arcos");
    assert_eq!(sem.arc_count(), 0, "o controlo não pode ter arcos");
    assert_eq!(com.prim_count(), 8, "a decomposição exacta tem 8 primitivas");
    assert!(
        sem.segment_count() > 700,
        "a referência tem de ser densa (deu {})",
        sem.segment_count()
    );

    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    const N: i32 = 260;
    for i in 0..=N {
        for j in 0..=N {
            #[allow(clippy::cast_possible_truncation)]
            {
                xs.push((-0.8 + 1.6 * f64::from(i) / f64::from(N)) as f32);
                ys.push((-0.8 + 1.6 * f64::from(j) / f64::from(N)) as f32);
            }
        }
    }
    let a = avalia(&com, &xs, &ys);
    let b = avalia(&sem, &xs, &ys);

    // A referência é uma polilinha de 768 lados sobre arcos de raio `0,15`: a flecha dela é
    // `r·(1−cos(π/384)) ≈ 5e-6`. A barra é uma ordem de grandeza acima disso, e NÃO a tolerância
    // declarada do perfil — *a barra sai do erro da RÉGUA, não de um número redondo*.
    const BARRA: f32 = 5e-5;
    let mut pior = 0.0f32;
    let mut trocas = 0usize;
    let mut pior_troca = 0.0f32;
    for ((va, vb), (x, y)) in a.iter().zip(&b).zip(xs.iter().zip(&ys)) {
        pior = pior.max((va - vb).abs());
        if va.signum() != vb.signum() && va.abs() > BARRA && vb.abs() > BARRA {
            trocas += 1;
            if vb.abs() > pior_troca {
                pior_troca = vb.abs();
                let _ = (x, y);
            }
        }
    }
    assert_eq!(
        trocas, 0,
        "⛔ o SINAL trocou em {trocas} pontos longe do bordo (pior |f| = {pior_troca:.6}) — \
         é a meia-lua entre a corda e o arco"
    );
    assert!(
        pior <= BARRA,
        "os dois campos afastam-se {pior:.3e} (barra {BARRA:.3e})"
    );
}

/// ⭐ **E um perfil SEM arcos emite a fita de sempre** — a metade que prova que esta wave não tocou
/// no caminho que já existia.
#[test]
fn um_perfil_sem_arcos_da_o_campo_de_sempre() {
    let quadrado = Profile::new(
        vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]],
        FillRule::NonZero,
        1e-4,
    )
    .expect("o quadrado é válido");
    assert_eq!(quadrado.arc_count(), 0);
    let xs = [0.5f32, 0.5, -0.5, 0.25];
    let ys = [0.5f32, 1.5, 0.5, 0.5];
    let esperado = [-0.5f32, 0.5, 0.5, -0.25];
    let v = avalia(&quadrado, &xs, &ys);
    for (i, (got, want)) in v.iter().zip(&esperado).enumerate() {
        assert!(
            (got - want).abs() < 1e-6,
            "ponto {i}: {got} em vez de {want}"
        );
    }
}
