//! P14 «L» — o receptor com NORMAL (núcleo cardioide `k = (1 + n_xy·ω̂)/2π`):
//! `luz_em_n = (F + n_xy·M)/2π`, `M = ∫L(θ)(cos θ, sin θ) dθ` por canal. Plano
//! `docs/PixelArt/P14_plano.md` §4 «L».

use std::f64::consts::TAU;

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_ref::queda::queda;
use ph2d_light2d_ref::{
    Gabarito, OpcoesDoMiolo, OpcoesDoRebote, Rebote, fluencia, luz_em, luz_em_n, momento,
};

use crate::comum::{cena, disco};

fn caixa(x0: f32, y0: f32, x1: f32, y1: f32, opacidade: f32, albedo: f32) -> Oclusor {
    Oclusor {
        vertices: vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        opacidade,
        albedo: [albedo; 3],
    }
}

fn ponto(centro: [f32; 2], i: f32, alcance_m: f32) -> Emissor {
    Emissor::Ponto {
        centro,
        intensidade: [i, 0.5 * i, 2.0 * i],
        alcance_m,
    }
}

fn normais(n: usize, modulos: &[f64]) -> Vec<[f64; 2]> {
    let mut v = Vec::new();
    for &m in modulos {
        for k in 0..n {
            let a = TAU * k as f64 / n as f64 + 0.1;
            v.push([m * a.cos(), m * a.sin()]);
        }
    }
    v
}

/// A cena da quadratura: um disco de alcance finito e um oclusor translúcido que tapa parte dele.
fn cena_da_quadratura() -> CenaDeLuz {
    cena(
        vec![disco([2.2, 0.7], 0.5, [1.0, 0.5, 2.0], 4.0)],
        vec![caixa(1.0, 0.5, 1.2, 1.5, 0.6, 0.0)],
    )
}

fn cenas_variadas() -> Vec<(CenaDeLuz, Vec<[f64; 2]>)> {
    let mut com_ceu = cena(
        vec![disco([-2.0, 1.0], 0.3, [3.0, 1.0, 0.2], f32::INFINITY)],
        vec![caixa(0.5, -0.5, 1.5, 0.5, 1.0, 0.0)],
    );
    com_ceu.ceu = [0.2, 0.3, 0.4];
    let brilha = cena(
        vec![
            Emissor::Poligono {
                vertices: vec![[2.0, -1.0], [3.0, -1.0], [3.0, 1.0], [2.0, 1.0]],
                radiancia: [1.0, 2.0, 0.5],
                alcance_m: 5.0,
            },
            ponto([-1.0, 2.0], 4.0, 6.0),
        ],
        vec![caixa(0.8, 0.2, 1.2, 0.6, 0.5, 0.0)],
    );
    vec![
        (
            cena_da_quadratura(),
            vec![[0.0, 0.0], [0.5, 1.8], [3.0, -0.5]],
        ),
        (com_ceu, vec![[0.0, 0.0], [2.0, 0.0], [-1.0, -1.0]]),
        (brilha, vec![[0.0, 0.0], [1.5, 0.4], [-2.0, -2.0]]),
    ]
}

/// O FORNO BRANCO: num campo uniforme `∫k = 1` ⇒ a mesma luz para toda normal.
#[test]
fn forno_branco_vale_para_toda_normal() {
    let mut c = cena(vec![], vec![]);
    c.ceu = [0.25, 0.5, 1.75];
    for p in [[0.0, 0.0], [3.7, -12.0]] {
        let plano = luz_em(&c, p);
        for n in normais(8, &[0.5, 1.0]) {
            let e = luz_em_n(&c, p, n);
            for k in 0..3 {
                assert!(
                    (e[k] - plano[k]).abs() <= 1e-12,
                    "n = {n:?}, canal {k}: {} ≠ {}",
                    e[k],
                    plano[k]
                );
            }
        }
    }
}

/// `n_xy = 0` (a normal de frente) é o receptor de hoje — o mesmo bit.
#[test]
fn a_normal_de_frente_e_o_receptor_de_hoje_bit_a_bit() {
    for (c, pontos) in cenas_variadas() {
        let g = Gabarito::novo(&c);
        // Um ponto dentro do oclusor (o miolo) também.
        for p in pontos.into_iter().chain([[1.1, 0.4]]) {
            let a = luz_em(&c, p).map(f64::to_bits);
            assert_eq!(luz_em_n(&c, p, [0.0, 0.0]).map(f64::to_bits), a, "{p:?}");
            assert_eq!(g.luz_em_n(p, [0.0, 0.0]).map(f64::to_bits), a, "{p:?}");
        }
    }
}

/// Uma luz a `+x`: `n = (1, 0)` recebe `(F + M_x)/2π`. Para o ponto, `M = F·(1, 0)` (2× e 0);
/// para o disco de raio `r` a `d`, `F = 2L·asin(r/d)` e `M = (2L·r/d, 0)` — exatos.
#[test]
fn a_lampada_a_direita_acende_a_normal_a_direita() {
    let c = cena(vec![ponto([3.0, 0.0], 2.0, f32::INFINITY)], vec![]);
    let plano = luz_em(&c, [0.0, 0.0]);
    let casos = [([1.0, 0.0], 2.0), ([-1.0, 0.0], 0.0), ([0.0, 1.0], 1.0)];
    for (n, fator) in casos {
        let e = luz_em_n(&c, [0.0, 0.0], n);
        for k in 0..3 {
            assert!(
                (e[k] - fator * plano[k]).abs() <= 1e-12 * plano[k],
                "ponto, n = {n:?}, canal {k}: {} ≠ {}",
                e[k],
                fator * plano[k]
            );
        }
    }

    let (r, d, l) = (0.5_f64, 3.0_f64, 1.0_f64);
    let c = cena(
        vec![disco(
            [d as f32, 0.0],
            r as f32,
            [l as f32; 3],
            f32::INFINITY,
        )],
        vec![],
    );
    let f = 2.0 * l * (r / d).asin();
    let mx = 2.0 * l * r / d;
    let casos = [
        ([1.0, 0.0], (f + mx) / TAU),
        ([-1.0, 0.0], (f - mx) / TAU),
        ([0.0, 1.0], f / TAU),
        ([0.6, -0.8], (f + 0.6 * mx) / TAU),
    ];
    for (n, esperado) in casos {
        let e = luz_em_n(&c, [0.0, 0.0], n)[0];
        assert!(
            (e - esperado).abs() <= 1e-12,
            "disco, n = {n:?}: {e} ≠ {esperado}"
        );
    }
}

/// Entrada do raio `(cos θ, sin θ)` saindo da origem no disco `(c, r)`.
fn entra_no_disco(d: [f64; 2], c: [f64; 2], r: f64) -> Option<f64> {
    let b = d[0] * c[0] + d[1] * c[1];
    let disc = b * b - (c[0] * c[0] + c[1] * c[1] - r * r);
    (disc >= 0.0).then(|| b - disc.sqrt())
}

/// Entrada do raio no retângulo `[lo, hi]` (método das lajes).
fn entra_na_caixa(d: [f64; 2], lo: [f64; 2], hi: [f64; 2]) -> Option<f64> {
    let (mut t0, mut t1) = (0.0_f64, f64::INFINITY);
    for k in 0..2 {
        if d[k] == 0.0 {
            if lo[k] > 0.0 || hi[k] < 0.0 {
                return None;
            }
            continue;
        }
        let (a, b) = (lo[k] / d[k], hi[k] / d[k]);
        t0 = t0.max(a.min(b));
        t1 = t1.min(a.max(b));
    }
    (t0 <= t1).then_some(t0)
}

/// O momento contra uma quadratura de ponto médio INDEPENDENTE (≥ 200 000 ângulos), partida nos
/// ângulos onde o integrando salta (tangentes do disco, vértices da caixa).
#[test]
fn o_momento_e_o_da_quadratura_bruta() {
    let c = cena_da_quadratura();
    let Emissor::Disco {
        centro,
        raio,
        radiancia,
        alcance_m,
    } = c.emissores[0].clone()
    else {
        unreachable!()
    };
    let cc = [f64::from(centro[0]), f64::from(centro[1])];
    let (r, alcance) = (f64::from(raio), f64::from(alcance_m));
    let tau = 1.0 - f64::from(c.oclusores[0].opacidade);
    let v = &c.oclusores[0].vertices;
    let (lo, hi) = (
        [f64::from(v[0][0]), f64::from(v[0][1])],
        [f64::from(v[2][0]), f64::from(v[2][1])],
    );
    let base = cc[1].atan2(cc[0]);
    let delta = (r / cc[0].hypot(cc[1])).asin();
    let (a0, b0) = (base - delta, base + delta);
    let mut cortes = vec![a0, b0];
    for q in v {
        let t = f64::from(q[1]).atan2(f64::from(q[0]));
        if t > a0 && t < b0 {
            cortes.push(t);
        }
    }
    cortes.sort_by(f64::total_cmp);
    let total_n = 240_000.0;
    let (mut f, mut mx, mut my) = (0.0, 0.0, 0.0);
    for w in cortes.windows(2) {
        let n = ((w[1] - w[0]) / (b0 - a0) * total_n).ceil() as usize;
        let h = (w[1] - w[0]) / n as f64;
        for i in 0..n {
            let th = w[0] + (i as f64 + 0.5) * h;
            let d = [th.cos(), th.sin()];
            let Some(t) = entra_no_disco(d, cc, r) else {
                continue;
            };
            let tr = entra_na_caixa(d, lo, hi).map_or(1.0, |tc| if tc < t { tau } else { 1.0 });
            let g = queda(t, alcance) * tr * h;
            f += g;
            mx += g * d[0];
            my += g * d[1];
        }
    }
    let m = momento(&c, [0.0, 0.0]);
    let fl = fluencia(&c, [0.0, 0.0]);
    for k in 0..3 {
        let l = f64::from(radiancia[k]);
        let escala = l * mx.hypot(my);
        assert!(
            (m[0][k] - l * mx).abs() <= 1e-5 * escala,
            "Mx canal {k}: {} ≠ {}",
            m[0][k],
            l * mx
        );
        assert!(
            (m[1][k] - l * my).abs() <= 1e-5 * escala,
            "My canal {k}: {} ≠ {}",
            m[1][k],
            l * my
        );
        assert!(
            (fl[k] - l * f).abs() <= 1e-5 * l * f,
            "F canal {k}: {} ≠ {}",
            fl[k],
            l * f
        );
    }
}

/// `|n_xy| = 1` em 32 direções nunca dá luz negativa (`|M| ≤ F`), fora e dentro de um corpo.
#[test]
fn nunca_negativa() {
    for (c, pontos) in cenas_variadas() {
        for p in pontos.into_iter().chain([[1.1, 0.4]]) {
            for n in normais(32, &[1.0]) {
                let e = luz_em_n(&c, p, n);
                assert!(e.iter().all(|&x| x >= -1e-12), "{p:?} n = {n:?}: {e:?}");
            }
        }
    }
}

fn sala_do_rebote() -> CenaDeLuz {
    cena(
        vec![disco([-3.0, 0.0], 0.5, [1.0, 0.8, 0.6], f32::INFINITY)],
        vec![caixa(2.0, -1.0, 2.5, 1.0, 1.0, 0.8)],
    )
}

fn op(rebotes: u32) -> OpcoesDoRebote {
    OpcoesDoRebote {
        passo_m: 0.1,
        rebotes,
    }
}

#[test]
fn o_rebote_com_normal_de_frente_e_o_de_hoje() {
    let c = sala_do_rebote();
    let r = Rebote::resolver(&c, op(3));
    for p in [[0.0, 0.0], [1.0, 0.5], [2.2, 0.0]] {
        assert_eq!(
            r.luz_em_n(p, [0.0, 0.0]).map(f64::to_bits),
            r.luz_em(p).map(f64::to_bits),
            "{p:?}"
        );
    }
}

/// A luz que a parede (a `+x`, entre `±δ`) devolve chega de `+x`: o momento que o rebote soma
/// tem `ΔM_x ≥ cos δ·ΔF` e `|ΔM_y| ≤ sin δ·ΔF`.
#[test]
fn o_rebote_chega_pela_direcao_da_parede() {
    let c = sala_do_rebote();
    let (r0, r1) = (Rebote::resolver(&c, op(0)), Rebote::resolver(&c, op(3)));
    let p = [0.0, 0.0];
    let delta = (1.0_f64 / 2.0).atan();
    let (f0, f1) = (r0.fluencia(p), r1.fluencia(p));
    let (m0, m1) = (r0.momento(p), r1.momento(p));
    for k in 0..3 {
        let df = f1[k] - f0[k];
        assert!(df > 1e-3, "canal {k}: o rebote não chegou ({df})");
        let (dx, dy) = (m1[0][k] - m0[0][k], m1[1][k] - m0[1][k]);
        assert!(
            dx >= delta.cos() * df * (1.0 - 1e-9),
            "canal {k}: ΔMx {dx} × ΔF {df}"
        );
        assert!(
            dy.abs() <= delta.sin() * df,
            "canal {k}: ΔMy {dy} × ΔF {df}"
        );
    }
    // E o momento sem rebote é o do disco a −x: M = (−2L·r/d, 0).
    let mx = -2.0 * 0.5 / 3.0;
    assert!((m0[0][0] - mx).abs() <= 1e-12, "{} ≠ {mx}", m0[0][0]);
}

/// Dentro de um corpo com `suavidade = 0` a resposta (F e M) é a do exterior mais próximo.
#[test]
fn dentro_do_corpo_com_suavidade_zero_e_o_exterior_mais_proximo() {
    let c = cena(
        vec![disco([3.0, 1.5], 0.5, [1.0, 0.5, 2.0], f32::INFINITY)],
        vec![caixa(-1.0, -1.0, 1.0, 1.0, 1.0, 0.0)],
    );
    let g = Gabarito::com_miolo(
        &c,
        OpcoesDoMiolo {
            suavidade: 0.0,
            penetracao_m: f64::INFINITY,
        },
    );
    let (p, q) = ([0.9, 0.2], [1.0 + 1e-9, 0.2]);
    let (mp, mq) = (g.momento(p), g.momento(q));
    let (fp, fq) = (g.fluencia(p), g.fluencia(q));
    for k in 0..3 {
        assert!((fp[k] - fq[k]).abs() <= 1e-9 * fq[k], "F canal {k}");
        assert!(
            mq[0][k] > 0.5 * fq[k],
            "M de fora aponta para a lâmpada: {mq:?}"
        );
        for e in 0..2 {
            assert!(
                (mp[e][k] - mq[e][k]).abs() <= 1e-9 * fq[k],
                "M[{e}] canal {k}: {} ≠ {}",
                mp[e][k],
                mq[e][k]
            );
        }
    }
}

/// O miolo por omissão (a difusão) difunde M como difunde F: no centro de um corpo com a lâmpada a
/// `+x`, `M_x > 0` e `|M| ≤ F`.
#[test]
fn o_miolo_difunde_o_momento() {
    let c = cena(
        vec![disco([4.0, 0.0], 0.5, [1.0; 3], f32::INFINITY)],
        vec![caixa(-1.0, -1.0, 1.0, 1.0, 1.0, 0.0)],
    );
    let p = [0.0, 0.0];
    let (f, m) = (fluencia(&c, p), momento(&c, p));
    assert!(f[0] > 0.0, "{f:?}");
    assert!(m[0][0] > 0.1 * f[0], "M = {m:?}, F = {f:?}");
    assert!(m[0][0].hypot(m[1][0]) <= f[0], "M = {m:?}, F = {f:?}");
}
