//! ⭐⭐ **UMA FIGURA DE DUAS PRIMITIVAS É UMA FIGURA** — a meia-lua (um arco e a sua corda) nos TRÊS
//! leitores (2026-09-16).
//!
//! ⛔ O `Profile::with_arcs` pedia três primitivas — a lei do polígono — e recusava a meia-lua que a
//! caneta desenha com dois pontos. Curada a porta, sobra a pergunta que só a avaliação responde: com
//! duas primitivas a corda do arco e a recta de volta são o MESMO segmento percorrido nos dois
//! sentidos, os enrolamentos delas **cancelam-se exactamente**, e o sinal inteiro sai da correcção da
//! meia-lua. É o caso em que essa correcção trabalha sozinha — e o gate compara-a com uma distância
//! analítica que não passa por perfil, árvore ou índice nenhum.

use ph2d_field::{FillRule, Profile};

/// Raio do círculo e abertura do arco.
const R: f32 = 0.5;
const ABERTURA: f32 = 1.4;

/// A meia-lua `{|p| < R} ∩ {x > c}`, com `c = R·cos(θ/2)`: o arco à DIREITA de `a→b` (`bulge < 0`),
/// no sentido anti-horário. A polilinha ao lado é a do arco achatado a `1e-4`.
fn meia_lua(fill: FillRule) -> Profile {
    let meio = f64::from(ABERTURA) * 0.5;
    let r = f64::from(R);
    #[allow(clippy::cast_possible_truncation)]
    let ponto = |ang: f64| [(r * ang.cos()) as f32, (r * ang.sin()) as f32];
    let (a, b) = (ponto(-meio), ponto(meio));
    #[allow(clippy::cast_possible_truncation)]
    let bulge = -((meio * 0.5).tan()) as f32;
    let passos = 64;
    let poli: Vec<[f32; 2]> = (0..=passos)
        .map(|k| ponto(-meio + 2.0 * meio * f64::from(k) / f64::from(passos)))
        .collect();
    Profile::with_arcs(vec![(poli, vec![(a, bulge), (b, 0.0)])], fill, 1e-4)
        .expect("a meia-lua é uma figura")
}

/// A distância com sinal ANALÍTICA da meia-lua.
fn oraculo(x: f32, y: f32) -> f32 {
    let (x, y) = (f64::from(x), f64::from(y));
    let (r, meio) = (f64::from(R), f64::from(ABERTURA) * 0.5);
    let (c, w) = (r * meio.cos(), r * meio.sin());
    let (a, b) = ((c, -w), (c, w));
    let d_ponta = (x - a.0).hypot(y - a.1).min((x - b.0).hypot(y - b.1));
    let ang = y.atan2(x);
    let d_arco = if ang.abs() <= meio {
        (x.hypot(y) - r).abs()
    } else {
        d_ponta
    };
    let d_corda = if y.abs() <= w { (x - c).abs() } else { d_ponta };
    let d = d_arco.min(d_corda);
    #[allow(clippy::cast_possible_truncation)]
    if x.hypot(y) < r && x > c {
        -d as f32
    } else {
        d as f32
    }
}

fn avalia(t: fidget::context::Tree, xs: &[f32], ys: &[f32]) -> Vec<f32> {
    use fidget::shape::EzShape;
    let shape = crate::Engine::from(t);
    let tape = shape.ez_float_slice_tape();
    let mut e = crate::Engine::new_float_slice_eval();
    let zs = vec![0.0f32; xs.len()];
    e.eval(&tape, xs, ys, &zs).expect("avalia").to_vec()
}

#[test]
fn a_meia_lua_de_duas_primitivas_bate_o_oraculo_nos_tres_leitores() {
    use fidget::context::Tree;
    for fill in [FillRule::NonZero, FillRule::EvenOdd] {
        let p = meia_lua(fill);
        let idx = crate::profile_index::ProfileIndex::build(&p);
        // Uma grelha que não cai sobre o bordo (o passo e a origem são incomensuráveis com `c`), em
        // 4×4 regiões — cada uma com a sua árvore especializada.
        let (lo, hi, n) = (0.05f32, 0.62f32, 24usize);
        let passo = (hi - lo) / n as f32;
        let (mut pior, mut dentro) = (0.0f32, 0usize);
        for ri in 0..4 {
            for rj in 0..4 {
                let (mut xs, mut ys) = (Vec::new(), Vec::new());
                for i in 0..n / 4 {
                    for j in 0..n / 4 {
                        #[allow(clippy::cast_precision_loss)]
                        {
                            xs.push(lo + passo * ((ri * n / 4 + i) as f32 + 0.37));
                            ys.push(-0.31 + passo * ((rj * n / 4 + j) as f32 + 0.41));
                        }
                    }
                }
                let caixa_lo = [
                    xs.iter().copied().fold(f32::INFINITY, f32::min) - passo,
                    ys.iter().copied().fold(f32::INFINITY, f32::min) - passo,
                ];
                let caixa_hi = [
                    xs.iter().copied().fold(f32::NEG_INFINITY, f32::max) + passo,
                    ys.iter().copied().fold(f32::NEG_INFINITY, f32::max) + passo,
                ];
                let global = avalia(
                    crate::profile::sd_profile(&p, &Tree::x(), &Tree::y()),
                    &xs,
                    &ys,
                );
                let regiao = avalia(
                    crate::profile::sd_profile_in_region(
                        &p,
                        &idx,
                        &Tree::x(),
                        &Tree::y(),
                        caixa_lo,
                        caixa_hi,
                        false,
                        None,
                    ),
                    &xs,
                    &ys,
                );
                let mut indice = Vec::new();
                idx.sd_batch(&xs, &ys, &mut indice);
                for k in 0..xs.len() {
                    let certo = oraculo(xs[k], ys[k]);
                    if certo < 0.0 {
                        dentro += 1;
                    }
                    for (nome, v) in [
                        ("global", global[k]),
                        ("região", regiao[k]),
                        ("índice", indice[k]),
                    ] {
                        assert!(
                            v.signum() == certo.signum(),
                            "⛔ [{fill:?}] ({}, {}) está a {certo:+.5} e o leitor `{nome}` diz \
                             {v:+.5} — o SINAL de uma figura de duas primitivas",
                            xs[k],
                            ys[k]
                        );
                        pior = pior.max((v - certo).abs());
                    }
                }
            }
        }
        // ⚠️ O PISO: uma grelha que não entrasse na meia-lua leria o sinal certo por construção.
        assert!(
            dentro > 40,
            "[{fill:?}] só {dentro} pontos caíram dentro da meia-lua"
        );
        assert!(
            pior < 2e-6,
            "[{fill:?}] o pior valor afasta-se {pior:e} do oráculo analítico"
        );
    }
}

/// ⚠️ **E duas RECTAS entre os mesmos dois pontos não são figura nenhuma** — a outra metade da porta.
/// Aceitar duas primitivas não pode aceitar a que não fecha área: o enrolamento seria zero em todo o
/// plano e a peça sairia vazia em silêncio.
#[test]
fn duas_rectas_nao_fecham_area_e_a_porta_recusa() {
    let poli = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 0.5]];
    let recusa = Profile::with_arcs(
        vec![(poli, vec![([0.0, 0.0], 0.0), ([1.0, 0.0], 0.0)])],
        FillRule::NonZero,
        1e-4,
    );
    assert!(
        matches!(
            recusa,
            Err(ph2d_field::ProfileError::BulgeMismatch { points: 2, .. })
        ),
        "duas rectas foram aceites como figura: {recusa:?}"
    );
}
