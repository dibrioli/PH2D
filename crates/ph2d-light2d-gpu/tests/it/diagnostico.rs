//! Diagnóstico (à mão): onde o erro da penumbra mora — por faixa de distância à luz e por
//! visibilidade, com o VIÉS (média de gpu/ref − 1) ao lado do RMSE. Viés ≠ 0 é defeito; oscilação
//! com viés ~0 é o anel.
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-light2d-gpu --test it diagnostico -- --ignored --nocapture
//! ```

use ph2d_light2d_gpu::{Config, Fusao, Solver};

use crate::cenas::{DEGRAUS, LUZ, RAIO_LUZ, penumbra};
use crate::comum::{gabarito, gpu, grade, ler_fluencia};

#[test]
#[ignore = "diagnóstico: corre-se à mão"]
fn diagnostico_do_erro_da_penumbra() {
    let Some(gpu) = gpu() else { return };
    let (_, lado, texel) = DEGRAUS[0];
    let cena = penumbra();
    let a = f64::from(RAIO_LUZ);
    let faixas = [0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 1e9];
    let r = gabarito(&cena, &grade(lado, texel));
    for fusao in [
        Fusao::CorrecaoBilinear,
        Fusao::Classica,
        Fusao::VizinhaIntercalada,
    ] {
        let mut s = Solver::novo(
            &gpu,
            Config {
                fusao,
                ..Config::padrao(grade(lado, texel))
            },
        )
        .expect("solver");
        s.quadro(&gpu, &cena);
        let g = ler_fluencia(&gpu, &s);
        println!("\n{fusao:?}: faixa (d/a) | livre: n, viés, rmse_rel | sombra: n, viés, rmse_rel");
        for w in faixas.windows(2) {
            let mut acc = [[0.0_f64; 4]; 2]; // n, soma(g/r−1), soma (g−r)², soma r
            for (k, (gv, rv)) in g.iter().zip(&r).enumerate() {
                let p = [
                    (f64::from(k as u32 % lado) + 0.5) * texel,
                    (f64::from(k as u32 / lado) + 0.5) * texel,
                ];
                let d = (p[0] - f64::from(LUZ[0])).hypot(p[1] - f64::from(LUZ[1]));
                if d / a < w[0] || d / a >= w[1] {
                    continue;
                }
                let vazio = if d > a {
                    2.0 * (a / d).asin()
                } else {
                    std::f64::consts::TAU
                };
                let livre = (rv[0] - vazio).abs() <= 1e-9 * vazio;
                let e = &mut acc[usize::from(!livre)];
                e[0] += 1.0;
                if rv[0] > 1e-12 {
                    e[1] += f64::from(gv[0]) / rv[0] - 1.0;
                }
                e[2] += (f64::from(gv[0]) - rv[0]).powi(2);
                e[3] += rv[0];
            }
            let f = |e: [f64; 4]| {
                if e[0] == 0.0 {
                    "—".to_owned()
                } else {
                    format!(
                        "{:>6}, {:+.4}, {:.4}",
                        e[0],
                        e[1] / e[0],
                        (e[2] / e[0]).sqrt() / (e[3] / e[0]).max(1e-30)
                    )
                }
            };
            println!(
                "  [{:>4}, {:>4}) | {} | {}",
                w[0],
                w[1],
                f(acc[0]),
                f(acc[1])
            );
        }
        // Perfil radial para a ESQUERDA da luz (sem oclusor).
        let j = (f64::from(LUZ[1]) / texel) as u32;
        let i0 = (f64::from(LUZ[0]) / texel) as u32;
        print!("  perfil −x (d px: gpu/ref): ");
        for dpx in [2u32, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24, 32, 48, 64, 85, 100] {
            let k = (j * lado + i0 - dpx) as usize;
            print!("{dpx}:{:.3} ", f64::from(g[k][0]) / r[k][0]);
        }
        println!();
    }
}

/// A cena capturada lida de volta: emissão (rgb, 1/alcance), cobertura e τ dentro, por texel.
struct Capturada {
    lado: u32,
    emissao: Vec<[f32; 4]>,
    cobertura: Vec<f32>,
    tau: Vec<f32>,
}

fn ler_capturada(gpu: &ph2d_gpu::GpuContext, s: &Solver) -> Capturada {
    let [e, c, t] = s.cena_capturada();
    let f16 = |b: &[u8], k: usize| half::f16::from_le_bytes([b[2 * k], b[2 * k + 1]]).to_f32();
    let um = |tex| {
        crate::comum::ler_bytes(gpu, tex)
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| f16(b, 0))
            .collect()
    };
    Capturada {
        lado: e.width(),
        emissao: crate::comum::ler_bytes(gpu, e)
            .as_chunks::<8>()
            .0
            .iter()
            .map(|b| [f16(b, 0), f16(b, 1), f16(b, 2), f16(b, 3)])
            .collect(),
        cobertura: um(c),
        tau: um(t),
    }
}

/// O meio em `p` com a regra do `c2_rc.wgsl`: dentro ⟺ cobertura bilinear ≥ ½; o material é o do
/// vizinho mais coberto. Devolve `(τ, índice do texel)`.
fn meio(c: &Capturada, p: [f64; 2]) -> (f64, usize) {
    let lado = c.lado as i64;
    if p[0] < 0.0 || p[1] < 0.0 || p[0] >= lado as f64 || p[1] >= lado as f64 {
        return (1.0, 0);
    }
    let preso = |i: i64, j: i64| (j.clamp(0, lado - 1) * lado + i.clamp(0, lado - 1)) as usize;
    let (u, v) = (p[0] - 0.5, p[1] - 0.5);
    let (i0, j0) = (u.floor() as i64, v.floor() as i64);
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let viz = [
        preso(i0, j0),
        preso(i0 + 1, j0),
        preso(i0, j0 + 1),
        preso(i0 + 1, j0 + 1),
    ];
    let w = [
        (1.0 - fu) * (1.0 - fv),
        fu * (1.0 - fv),
        (1.0 - fu) * fv,
        fu * fv,
    ];
    let cx: f64 = viz
        .iter()
        .zip(w)
        .map(|(&k, w)| w * f64::from(c.cobertura[k]))
        .sum();
    if cx < 0.5 {
        return (1.0, 0);
    }
    let mut melhor = viz[0];
    for &k in &viz[1..] {
        if c.cobertura[k] > c.cobertura[melhor] {
            melhor = k;
        }
    }
    (f64::from(c.tau[melhor]), melhor)
}

/// O MODELO: a fluência da cena CAPTURADA por força bruta — `n` direções, cada raio marchado do
/// centro do texel com passo `h` até sair da grade, com a regra de entrada da placa.
fn forca_bruta(
    c: &Capturada,
    p: [f64; 2],
    n: u32,
    h: f64,
    texel_m: f64,
    ceu: [f32; 3],
) -> [f64; 3] {
    let lado = f64::from(c.lado);
    let mut f = [0.0; 3];
    for k in 0..n {
        let a = std::f64::consts::TAU * (f64::from(k) + 0.5) / f64::from(n);
        let d = [a.cos(), a.sin()];
        let (mut t, mut tau_ant, mut r) = (1.0_f64, 1.0_f64, [0.0_f64; 3]);
        let mut s = 0.0;
        loop {
            let x = [p[0] + d[0] * s, p[1] + d[1] * s];
            if x[0] < 0.0 || x[1] < 0.0 || x[0] >= lado || x[1] >= lado {
                for ch in 0..3 {
                    r[ch] += t * f64::from(ceu[ch]);
                }
                break;
            }
            let (tau, idx) = meio(c, x);
            if tau < tau_ant {
                let e = c.emissao[idx];
                let perda = 1.0 - tau / tau_ant;
                let dist = ((x[0] - p[0]).hypot(x[1] - p[1])) * texel_m;
                let q = (1.0 - (dist * f64::from(e[3])).powi(4))
                    .clamp(0.0, 1.0)
                    .powi(2);
                for ch in 0..3 {
                    r[ch] += t * perda * f64::from(e[ch]) * q;
                }
                t *= tau / tau_ant;
                if t <= 0.0 {
                    break;
                }
            }
            tau_ant = tau;
            s += h;
        }
        for ch in 0..3 {
            f[ch] += r[ch];
        }
    }
    f.map(|v| v * std::f64::consts::TAU / f64::from(n))
}

/// Separa os dois erros, por degrau e por cena: o da CAPTURA (força bruta na grade × gabarito) e
/// o das CASCATAS (RC × força bruta na mesma grade), numa amostra de 1 500 texels (LCG fixo).
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-light2d-gpu --profile smoke --test it modelo -- --ignored --nocapture
/// ```
#[test]
#[ignore = "diagnóstico: corre-se à mão, no perfil smoke"]
fn diagnostico_modelo_captura_contra_cascatas() {
    let Some(gpu) = gpu() else { return };
    for (nome, cena) in [
        ("penumbra", penumbra()),
        ("céu e vidro", crate::cenas::ceu_e_vidro()),
    ] {
        for (degrau, lado, texel) in DEGRAUS {
            let mut s = Solver::novo(&gpu, Config::padrao(grade(lado, texel))).expect("solver");
            s.quadro(&gpu, &cena);
            let rc = ler_fluencia(&gpu, &s);
            let cap = ler_capturada(&gpu, &s);
            let mut amostra: Vec<u32> = Vec::new();
            let mut x: u64 = 0x2545_F491_4F6C_DD1D;
            for _ in 0..1500 {
                x = x
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                amostra.push(((x >> 33) % u64::from(lado * lado)) as u32);
            }
            let g = grade(lado, texel);
            let pedaco = amostra.len().div_ceil(16);
            let mut linhas = vec![[[0.0_f64; 3]; 2]; amostra.len()];
            std::thread::scope(|sc| {
                for (k, fatia) in linhas.chunks_mut(pedaco).enumerate() {
                    let (amostra, cap, cena, g) = (&amostra, &cap, &cena, &g);
                    sc.spawn(move || {
                        for (o, v) in fatia.iter_mut().enumerate() {
                            let idx = amostra[k * pedaco + o];
                            let (i, j) = (idx % lado, idx / lado);
                            let p = [f64::from(i) + 0.5, f64::from(j) + 0.5];
                            *v = [
                                ph2d_light2d_ref::fluencia(cena, g.centro(i, j)),
                                forca_bruta(cap, p, 4096, 0.25, texel, cena.ceu),
                            ];
                        }
                    });
                }
            });
            let rmse = |par: &dyn Fn(usize) -> ([f64; 3], [f64; 3])| {
                let (mut se, mut sr) = (0.0, 0.0);
                for k in 0..amostra.len() {
                    let (a, b) = par(k);
                    for ch in 0..3 {
                        se += (a[ch] - b[ch]).powi(2);
                        sr += b[ch];
                    }
                }
                let n = 3.0 * amostra.len() as f64;
                (se / n).sqrt() / (sr / n)
            };
            let rcv = |k: usize| rc[amostra[k] as usize].map(f64::from);
            println!(
                "{nome} · {degrau}: RC × gabarito {:.4} · força bruta × gabarito {:.4} · RC × força bruta {:.4}",
                rmse(&|k| (rcv(k), linhas[k][0])),
                rmse(&|k| (linhas[k][1], linhas[k][0])),
                rmse(&|k| (rcv(k), linhas[k][1])),
            );
        }
    }
}
