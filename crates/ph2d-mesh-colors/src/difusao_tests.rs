//! Os gates do [DESFOQUE NA SUPERFÍCIE](crate::difusao) — irmão (`#[path]`) do
//! `lib.rs`.
//!
//! ⚠️ As fixturas planas são RODADAS no espaço (uma rotação rígida que não
//! deixa nenhum eixo do plano num eixo do objecto) e a variância lê-se nos dois
//! eixos DO PLANO: com o plano alinhado, um peso trocado entre direcções leria
//! igual. Os valores esperados são EXACTOS no espaço do tipo — `σ²` de
//! variância, `Φ(1) = 0,8413` a um desvio de um degrau — e não relacionais.

use crate::difusao::Difusao;
use crate::{Tinta, cantos};

/// Uma rotação rígida: as colunas são a base do plano (`u`, `v`) e a normal.
/// (`37°` em torno de `(1, 2, 3)/√14`, calculada em `f64`.)
const R: [[f32; 3]; 3] = [
    [0.813_018_7, -0.453_759_14, 0.364_833_2],
    [0.511_291_85, 0.856_168_2, -0.074_542_76],
    [-0.278_534_13, 0.247_140_9, 0.928_084_1],
];

fn roda(x: f32, y: f32) -> [f32; 3] {
    [
        R[0][0] * x + R[0][1] * y,
        R[1][0] * x + R[1][1] * y,
        R[2][0] * x + R[2][1] * y,
    ]
}

/// As coordenadas DO PLANO de um ponto rodado (a inversa de [`roda`]).
fn no_plano(p: [f32; 3]) -> (f32, f32) {
    (
        R[0][0] * p[0] + R[1][0] * p[1] + R[2][0] * p[2],
        R[0][1] * p[0] + R[1][1] * p[1] + R[2][1] * p[2],
    )
}

/// `n × n` células de lado `1` centradas na origem: quads, ou dois triângulos
/// EQUILÁTEROS por célula (a grelha cortada a 60°).
fn grelha(n: u32, tri: bool) -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
    let v = |i: u32, j: u32| j * (n + 1) + i;
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let (a, b, c, d) = (v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1));
            if tri {
                faces.push(vec![a, b, d]);
                faces.push(vec![b, c, d]);
            } else {
                faces.push(vec![a, b, c, d]);
            }
        }
    }
    let h = n as f32 * 0.5;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            let (x, y) = (i as f32 - h, j as f32 - h);
            pos.push(if tri {
                roda(x + 0.5 * y, 0.866_025_4 * y)
            } else {
                roda(x, y)
            });
        }
    }
    (pos, faces)
}

/// A posição de cada amostra.
fn posicoes(t: &Tinta, faces: &[Vec<u32>], pos: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0; 3]; t.amostras().len()];
    for (f, c) in faces.iter().enumerate() {
        let c = &c[..cantos(c)];
        let l = t.lado_da_face(f);
        let p = |k: usize| pos[c[k] as usize];
        if c.len() == 3 {
            t.para_cada_amostra_tri(f, c, |i, ijk| {
                out[i as usize] = crate::amostragem::posicao_tri(p(0), p(1), p(2), l, ijk);
            });
        } else {
            t.para_cada_amostra_quad(f, c, |i, ij| {
                out[i as usize] = crate::amostragem::posicao_quad([p(0), p(1), p(2), p(3)], l, ij);
            });
        }
    }
    out
}

fn difusao(faces: &[Vec<u32>], pos: &[[f32; 3]], nivel: u8) -> (Tinta, Difusao) {
    let t = Tinta::nova(pos.len(), faces.iter().map(|f| &f[..]), nivel);
    let d = Difusao::nova(&t, |f| &faces[f][..], pos);
    (t, d)
}

/// ⭐⭐⭐ **Um campo constante fica constante AO BIT** — o passo é
/// `u + Σ c·(u_b − u_a)` e cada diferença é zero.
#[test]
fn um_campo_constante_fica_constante_ao_bit() {
    for tri in [false, true] {
        let (pos, faces) = grelha(4, tri);
        let (t, d) = difusao(&faces, &pos, 3);
        let c = [0.3f32, 0.6, 0.9, 0.75];
        let mut buf = vec![c; t.amostras().len()];
        d.desfoca(0.4, &mut buf);
        assert!(
            d.polinomio(0.4).len() > 4,
            "a fixtura tem de dar um polinómio de grau alto"
        );
        assert!(buf.iter().all(|&p| p == c), "tri={tri}: o constante andou");
    }
}

/// ⭐⭐⭐ **Raio zero é NO-OP ao bit** — nenhum passo.
#[test]
fn raio_zero_e_no_op_ao_bit() {
    let (pos, faces) = grelha(2, false);
    let (t, d) = difusao(&faces, &pos, 2);
    assert!(d.polinomio(0.0).is_empty() && d.polinomio(-1.0).is_empty());
    let antes: Vec<f32> = (0..t.amostras().len())
        .map(|i| (i as f32 * 0.37).sin())
        .collect();
    let mut buf = antes.clone();
    d.desfoca(0.0, &mut buf);
    assert_eq!(buf, antes);
}

/// ⭐⭐ **Nada sai do intervalo e a cor TOTAL conserva-se** (`Σ m·u`, pesada
/// pela área — aqui uniforme por construção da grelha interior).
#[test]
fn nada_sai_do_intervalo_da_entrada() {
    let (pos, faces) = grelha(4, true);
    let (t, d) = difusao(&faces, &pos, 3);
    let n = t.amostras().len();
    let antes: Vec<f32> = (0..n)
        .map(|i| 0.5 + 0.5 * ((i as f32 * 12.9898).sin() * 43_758.547).fract())
        .collect();
    let (lo, hi) = antes
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    let mut buf = antes.clone();
    d.desfoca(0.3, &mut buf);
    // O polinómio erra `< 1e-6` sobre o calor, que não sai do intervalo.
    let folga = 2e-6;
    assert!(
        buf.iter().all(|&v| v >= lo - folga && v <= hi + folga),
        "saiu de [{lo}, {hi}]"
    );
}

/// ⭐⭐⭐ **O polinómio É o calor**: o mesmo `e^{−tA}` por passos explícitos
/// pequenos (`(I − dt·A)^n`, que converge para ele) dá o mesmo a `1e-4`, nas
/// duas formas.
#[test]
fn o_polinomio_e_o_calor_por_passos() {
    for tri in [false, true] {
        let (pos, faces) = grelha(3, tri);
        let (t, d) = difusao(&faces, &pos, 3);
        let n = t.amostras().len();
        let antes: Vec<[f32; 4]> = (0..n)
            .map(|i| {
                let h = ((i as f32 * 12.9898).sin() * 43_758.547).fract().abs();
                [h, 1.0 - h, 0.5 * h, 1.0]
            })
            .collect();
        let (mut a, mut b) = (antes.clone(), antes);
        d.desfoca(0.35, &mut a);
        d.desfoca_por_passos(0.35, 40.0, &mut b);
        let pior = a
            .iter()
            .zip(&b)
            .flat_map(|(x, y)| (0..4).map(move |c| (x[c] - y[c]).abs()))
            .fold(0.0f32, f32::max);
        assert!(
            pior < 1e-4,
            "tri={tri}: o polinómio afasta-se do calor em {pior}"
        );
    }
}

/// O segundo momento DO PLANO de um impulso desfocado, à volta do sítio dele.
fn variancia(
    t: &Tinta,
    d: &Difusao,
    faces: &[Vec<u32>],
    pos: &[[f32; 3]],
    sigma: f32,
) -> (f32, f32, f32) {
    let xs = posicoes(t, faces, pos);
    // O impulso na amostra mais perto do centro.
    let a = (0..xs.len())
        .min_by(|&i, &j| {
            let (pi, pj) = (no_plano(xs[i]), no_plano(xs[j]));
            (pi.0.hypot(pi.1)).total_cmp(&pj.0.hypot(pj.1))
        })
        .expect("amostras");
    let mut buf = vec![0.0f32; xs.len()];
    buf[a] = 1.0;
    d.desfoca(sigma, &mut buf);
    // A massa de cada amostra é proporcional ao que a difusão conserva: numa
    // grelha regular interior é a mesma em todas, logo a média pesada é simples.
    let c = no_plano(xs[a]);
    let (mut s, mut uu, mut vv, mut uv) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for (i, &w) in buf.iter().enumerate() {
        let p = no_plano(xs[i]);
        let (du, dv) = (f64::from(p.0 - c.0), f64::from(p.1 - c.1));
        let w = f64::from(w);
        s += w;
        uu += w * du * du;
        vv += w * dv * dv;
        uv += w * du * dv;
    }
    ((uu / s) as f32, (vv / s) as f32, (uv / s) as f32)
}

/// ⭐⭐⭐⭐ **A variância é `σ²` EXACTO no MUNDO, nos dois eixos do plano, a
/// `8x` e a `32x`** — o raio é das unidades da peça e não das amostras.
///
/// ⛔ Um peso em AMOSTRAS (o mesmo número de passos em todo degrau) daria
/// `σ²/16` a `32x`; um peso trocado entre direcções daria `u ≠ v`.
#[test]
fn a_variancia_e_sigma_ao_quadrado_no_mundo_em_todo_degrau() {
    let sigma = 0.5f32;
    for tri in [false, true] {
        let (pos, faces) = grelha(8, tri);
        for nivel in [3u8, 5] {
            let (t, d) = difusao(&faces, &pos, nivel);
            let (uu, vv, uv) = variancia(&t, &d, &faces, &pos, sigma);
            let s2 = sigma * sigma;
            for (eixo, v) in [("u", uu), ("v", vv)] {
                assert!(
                    (v - s2).abs() < 2e-3,
                    "tri={tri} nível {nivel}: variância em {eixo} = {v}, quer-se σ² = {s2}"
                );
            }
            assert!(uv.abs() < 2e-3, "tri={tri} nível {nivel}: covariância {uv}");
        }
    }
}

/// `Φ(x)`, a normal acumulada (Abramowitz–Stegun 7.1.26, erro `< 1,5e-7`).
fn phi(x: f32) -> f32 {
    let z = f64::from(x) / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * z.abs());
    let y = 1.0
        - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t
            * (-z * z).exp();
    let erf = if z >= 0.0 { y } else { -y };
    (0.5 * (1.0 + erf)) as f32
}

/// ⭐⭐⭐⭐ **A COSTURA: o desfoque atravessa uma ARESTA DA MALHA dobrada a 90°
/// como se a superfície fosse plana.** Um cubo de lado `2`, a face de cima a
/// `1`, as outras a `0`: a meio de uma aresta de cima, a `d` da aresta medido
/// NA SUPERFÍCIE, o desfoque é `Φ(d/σ)` em cima e `Φ(−d/σ)` do lado — `0,5`
/// exacto na aresta — com `σ` cinco vezes menor que a distância aos cantos.
///
/// ⛔ O CONTROLO que dá direito ao gate: borrar pela ORDEM das amostras (o
/// atlas implícito) põe cor em amostras longe da aresta — o gate mede-o e
/// exige que a régua o veja.
#[test]
fn o_desfoque_atravessa_a_aresta_do_cubo_como_um_plano() {
    let pos: Vec<[f32; 3]> = (0..8u32)
        .map(|k| {
            let b = |s: u32| if k >> s & 1 == 1 { 1.0 } else { -1.0 };
            roda_3d([b(0), b(1), b(2)])
        })
        .collect();
    // Os seis quads, virados para fora.
    let faces: Vec<Vec<u32>> = vec![
        vec![4, 5, 7, 6], // z = +1 (cima)
        vec![0, 2, 3, 1], // z = −1
        vec![0, 1, 5, 4], // y = −1
        vec![2, 6, 7, 3], // y = +1
        vec![0, 4, 6, 2], // x = −1
        vec![1, 3, 7, 5], // x = +1
    ];
    let (t, d) = difusao(&faces, &pos, 5);
    let xs = posicoes(&t, &faces, &pos);
    let local: Vec<[f32; 3]> = xs.iter().map(|&p| desroda_3d(p)).collect();
    let mut buf: Vec<f32> = local
        .iter()
        .map(|p| {
            // ⚠️ A fila DA aresta é metade de cada face: a `1` ela poria o
            // degrau meia amostra para o lado (`Φ(h/2σ) − ½ = 0,062` medido).
            let borda = p[0].abs() > 1.0 - 1e-4 || p[1].abs() > 1.0 - 1e-4;
            match (p[2] > 1.0 - 1e-4, borda) {
                (true, true) => 0.5,
                (true, false) => 1.0,
                _ => 0.0,
            }
        })
        .collect();
    let original = buf.clone();
    let sigma = 0.2f32;
    d.desfoca(sigma, &mut buf);
    // O perfil a meio da aresta (x = 0) entre a de cima (z = 1) e a de y = −1.
    let mut pior = 0.0f32;
    let mut vistos = 0;
    for (i, p) in local.iter().enumerate() {
        if p[0].abs() > 1e-3 {
            continue;
        }
        // A distância NA SUPERFÍCIE à aresta (y = −1, z = 1), com sinal (+ em cima).
        let dist = if p[2] > 1.0 - 1e-4 && p[1] < -0.4 {
            p[1] + 1.0
        } else if p[1] < -1.0 + 1e-4 && p[2] > 0.4 {
            -(1.0 - p[2])
        } else {
            continue;
        };
        vistos += 1;
        pior = pior.max((buf[i] - phi(dist / sigma)).abs());
    }
    // `x = 0` a cada `1/16` em `0,6` de cada lado: `10 + 10` menos a aresta partilhada.
    assert!(vistos >= 19, "o perfil tem de ter amostras ({vistos})");
    assert!(
        pior < 0.02,
        "o perfil através da aresta afasta-se de Φ(d/σ) em {pior}"
    );
    // ⛔ O CONTROLO: o mesmo σ em AMOSTRAS ao longo da ordem do plano.
    let passos_1d = {
        let mut u = original.clone();
        let raio = (sigma / (2.0 / 32.0)).ceil() as usize;
        let k: Vec<f32> = (0..=3 * raio)
            .map(|x| (-(x as f32).powi(2) / (2.0 * (raio as f32).powi(2))).exp())
            .collect();
        let soma: f32 = k[0] + 2.0 * k[1..].iter().sum::<f32>();
        for (i, o) in u.iter_mut().enumerate() {
            let mut s = original[i] * k[0];
            for (x, w) in k.iter().enumerate().skip(1) {
                let a = original[i.saturating_sub(x)];
                let b = original[(i + x).min(original.len() - 1)];
                s += w * (a + b);
            }
            *o = s / soma;
        }
        u
    };
    // Cor longe da face de cima (a mais de 4σ dela, na superfície).
    let longe = |u: &[f32]| {
        local
            .iter()
            .zip(u)
            .filter(|(p, _)| p[2] < 1.0 - 4.0 * sigma && p[2] > -1.0 + 1e-4 || p[2] < -1.0 + 1e-4)
            .map(|(_, &v)| v)
            .fold(0.0f32, f32::max)
    };
    assert!(
        longe(&buf) < 1e-3,
        "o calor pôs cor longe da aresta: {}",
        longe(&buf)
    );
    assert!(
        longe(&passos_1d) > 0.05,
        "CONTROLO: o desfoque pela ordem das amostras tinha de espalhar cor longe ({})",
        longe(&passos_1d)
    );
}

/// O cubo rodado (sem nenhum eixo dele num eixo do objecto).
fn roda_3d(p: [f32; 3]) -> [f32; 3] {
    [
        R[0][0] * p[0] + R[0][1] * p[1] + R[0][2] * p[2],
        R[1][0] * p[0] + R[1][1] * p[1] + R[1][2] * p[2],
        R[2][0] * p[0] + R[2][1] * p[1] + R[2][2] * p[2],
    ]
}

fn desroda_3d(p: [f32; 3]) -> [f32; 3] {
    [
        R[0][0] * p[0] + R[1][0] * p[1] + R[2][0] * p[2],
        R[0][1] * p[0] + R[1][1] * p[1] + R[2][1] * p[2],
        R[0][2] * p[0] + R[1][2] * p[1] + R[2][2] * p[2],
    ]
}

/// ⭐⭐⭐ **O degrau num plano: `Φ(1) = 0,8413` a um desvio do degrau, a `8x` e
/// a `32x`** — o mesmo desfoque ao olho nos dois degraus, medido num ponto
/// exacto do mundo (interpolado pela leitura da própria retícula).
#[test]
fn um_degrau_desfocado_e_phi_no_mundo_a_8x_e_a_32x() {
    let sigma = 0.4f32;
    let (pos, faces) = grelha(8, false);
    for (nivel, tol) in [(3u8, 0.02f32), (5, 0.005)] {
        let (mut t, d) = difusao(&faces, &pos, nivel);
        let xs = posicoes(&t, &faces, &pos);
        // O degrau em u = 0.3 (não cai numa aresta da malha nem numa amostra).
        let mut buf: Vec<[f32; 4]> = xs
            .iter()
            .map(|&p| {
                if no_plano(p).0 > 0.3 {
                    [1.0; 4]
                } else {
                    [0.0; 4]
                }
            })
            .collect();
        d.desfoca(sigma, &mut buf);
        for (o, b) in t.amostras_mut().iter_mut().zip(&buf) {
            *o = [b[0], b[1], b[2]];
        }
        // Lê em u = 0.3 + σ e u = 0.3 − σ, em v = 0.5 (dentro de uma face).
        for (du, quer) in [(sigma, phi(1.0)), (-sigma, phi(-1.0)), (0.0, 0.5)] {
            let (u, v) = (0.3 + du, 0.5f32);
            let lido = le_no_plano(&t, &faces, u, v);
            assert!(
                (lido - quer).abs() < tol,
                "nível {nivel}: em u = {u} lido {lido}, quer-se {quer}"
            );
        }
    }
}

/// Lê a cor no ponto `(u, v)` do plano da [`grelha`] de quads (`8 × 8`).
fn le_no_plano(t: &Tinta, faces: &[Vec<u32>], u: f32, v: f32) -> f32 {
    let (x, y) = (u + 4.0, v + 4.0);
    let (i, j) = (x.floor() as usize, y.floor() as usize);
    let f = j * 8 + i;
    t.cor_quad(f, &faces[f], [x - i as f32, y - j as f32])[0]
}

/// ⭐⭐ **As threads não mudam um bit** — cada linha lê só o passo anterior.
#[test]
fn as_threads_nao_mudam_um_bit() {
    let (pos, faces) = grelha(8, true);
    let t = Tinta::nova(pos.len(), faces.iter().map(|f| &f[..]), 5);
    assert!(t.amostras().len() >= 1 << 14, "a fixtura tem de repartir");
    let um = Difusao::com_threads(&t, |f| &faces[f][..], &pos, 1);
    let muitas = Difusao::com_threads(&t, |f| &faces[f][..], &pos, 7);
    let antes: Vec<[f32; 4]> = (0..t.amostras().len())
        .map(|i| [(i as f32 * 0.013).sin().abs(), 0.2, 0.7, 1.0])
        .collect();
    let (mut a, mut b) = (antes.clone(), antes);
    um.desfoca(0.3, &mut a);
    muitas.desfoca(0.3, &mut b);
    assert_eq!(a, b);
}
