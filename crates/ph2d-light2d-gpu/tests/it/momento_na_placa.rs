//! ⭐ P14 «L» — o momento `M` na PLACA contra o gabarito (`docs/PixelArt/P14_plano.md` §7): o receptor de
//! normal `n` lê `(F + n·M)/2π`; aqui `F`, `Mx` e `My` lidos da placa, a conta do `receber.wgsl` na CPU,
//! contra `ph2d_light2d_ref::Gabarito::luz_em_n` no centro de cada texel.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_light2d_ref::Gabarito;

use crate::comum::{disco, gabarito_por, gpu, grade, ler_bytes, ler_fluencia, oclusor, rmse_rel};

const TAU: f64 = std::f64::consts::TAU;
const LAMPADA: [f32; 2] = [4.0, 2.56];

/// Lâmpada pontual à direita (alcance finito), um disco com alcance finito, céu e um oclusor parcial.
fn cena() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![
            Emissor::Ponto {
                centro: LAMPADA,
                intensidade: [1.0, 0.8, 0.6],
                alcance_m: 3.0,
            },
            disco([1.0, 1.0], 0.15, [0.5, 0.5, 1.0], 2.5),
        ],
        oclusores: vec![oclusor(
            vec![[2.5, 2.0], [2.8, 2.0], [2.8, 3.2], [2.5, 3.2]],
            0.5,
        )],
        ceu: [0.05, 0.05, 0.08],
        com_normal: true,
        ..Default::default()
    }
}

fn f16s(b: &[u8]) -> Vec<[f32; 3]> {
    b.as_chunks::<8>()
        .0
        .iter()
        .map(|t| {
            let c = |k: usize| half::f16::from_le_bytes([t[2 * k], t[2 * k + 1]]).to_f32();
            [c(0), c(1), c(2)]
        })
        .collect()
}

/// `(F, Mx, My)` do quadro, texel a texel.
fn rodar(gpu: &ph2d_gpu::GpuContext, cena: &CenaDeLuz) -> Lido {
    let mut s = Solver::novo(gpu, Config::padrao(grade(128, 0.04))).expect("solver");
    s.quadro(gpu, cena);
    let f = ler_fluencia(gpu, &s);
    let m = s
        .momento_texturas()
        .map(|[x, y]| [f16s(&ler_bytes(gpu, x)), f16s(&ler_bytes(gpu, y))]);
    Lido {
        f,
        m,
        g: s.grade_da_fluencia(),
        bytes: s.bytes_do_momento(),
    }
}

struct Lido {
    f: Vec<[f32; 3]>,
    m: Option<[Vec<[f32; 3]>; 2]>,
    g: ph2d_light2d_gpu::Grade,
    bytes: u64,
}

impl Lido {
    /// A conta do `luz_em_n` com o que a placa escreveu.
    fn luz_em_n(&self, n: [f64; 2]) -> Vec<[f32; 3]> {
        let [mx, my] = self.m.as_ref().expect("o momento");
        (0..self.f.len())
            .map(|k| {
                std::array::from_fn(|c| {
                    ((f64::from(self.f[k][c])
                        + n[0] * f64::from(mx[k][c])
                        + n[1] * f64::from(my[k][c]))
                        / TAU) as f32
                })
            })
            .collect()
    }
}

/// O MEDIDO + folga (09/11, RADV, rmse relativo): `n = 0` (o F de hoje) 0,0151 · `(1, 0)` 0,0336 ·
/// `(0, 1)` 0,0164 · `(−0,6, −0,8)` 0,0225. Visto vermelho com `ω̂ → −ω̂`.
const TETO: f64 = 0.045;

#[test]
fn a_placa_e_o_gabarito_concordam_com_normal() {
    let Some(gpu) = gpu() else { return };
    let cena = cena();
    let l = rodar(&gpu, &cena);
    let gab = Gabarito::novo(&cena);
    let g = l.g;
    // Fora: o miolo do disco e 3 texels em volta da lâmpada pontual (o 1/r).
    let longe = |k: usize| {
        let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
        let d = |c: [f32; 2]| (p[0] - f64::from(c[0])).hypot(p[1] - f64::from(c[1]));
        d(LAMPADA) > 3.0 * g.texel_m && d([1.0, 1.0]) > 0.15 + g.texel_m
    };
    for n in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [-0.6, -0.8]] {
        let r = gabarito_por(&g, |p| gab.luz_em_n(p, n));
        let e = rmse_rel(&l.luz_em_n(n), &r, longe);
        let pior = l
            .luz_em_n(n)
            .iter()
            .zip(&r)
            .enumerate()
            .filter(|(k, _)| longe(*k))
            .map(|(_, (a, b))| {
                (0..3)
                    .map(|c| (f64::from(a[c]) - b[c]).abs())
                    .fold(0.0, f64::max)
            })
            .fold(0.0, f64::max);
        eprintln!("n = {n:?}: rmse relativo {e:.4} · erro máximo absoluto {pior:.4}");
        assert!(e <= TETO, "n = {n:?}: placa × gabarito {e:.4} > {TETO}");
    }
}

/// Só a lâmpada à direita, sem céu: num ponto à esquerda dela `n = (1, 0)` dá ~2× o plano e `n = (−1, 0)` ~0.
#[test]
fn a_lampada_a_direita_acende_a_normal_a_direita_na_placa() {
    let Some(gpu) = gpu() else { return };
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Ponto {
            centro: LAMPADA,
            intensidade: [1.0; 3],
            alcance_m: f32::INFINITY,
        }],
        com_normal: true,
        ..Default::default()
    };
    let l = rodar(&gpu, &cena);
    let g = l.g;
    let k = |p: [f64; 2]| {
        let i = ((p[0] - g.origem_m[0]) / g.texel_m) as u32;
        let j = ((p[1] - g.origem_m[1]) / g.texel_m) as u32;
        (j * g.largura + i) as usize
    };
    let alvo = k([2.5, 2.56]);
    let [claro, plano, escuro] =
        [[1.0, 0.0], [0.0, 0.0], [-1.0, 0.0]].map(|n| l.luz_em_n(n)[alvo][1]);
    eprintln!(
        "claro {claro:.5} · plano {plano:.5} · escuro {escuro:.5} · razão {:.4}",
        claro / plano
    );
    assert!(plano > 0.0);
    assert!(
        (claro / plano - 2.0).abs() < 0.02,
        "a face virada à lâmpada: {claro} vs 2×{plano}"
    );
    assert!(escuro.abs() < 0.02 * plano, "a face de costas: {escuro}");
}

/// Sem normal à vista o momento não existe (0 bytes) e o F é o mesmo bit a bit que com ele.
#[test]
fn sem_normal_nao_aloca_o_momento() {
    let Some(gpu) = gpu() else { return };
    let com = cena();
    let sem = CenaDeLuz {
        com_normal: false,
        ..com.clone()
    };
    let (a, b) = (rodar(&gpu, &sem), rodar(&gpu, &com));
    eprintln!("bytes do momento: sem {} · com {}", a.bytes, b.bytes);
    assert!(a.m.is_none() && a.bytes == 0);
    assert!(b.m.is_some() && b.bytes > 0);
    assert!(a.f == b.f, "o momento mexeu no F");
}

/// ⭐ P14c-placa 4 — JUNTO A UM OPACO, só com o céu (toda a luz é a c0): o `M` dos texels de fora a ≤ 4 texels
/// da parede contra o gabarito. Prende a c0 de `M` onde metade das direções está tapada.
#[test]
fn junto_ao_opaco_o_momento_e_o_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let (x0, x1, y0, y1) = (2.5_f64, 2.8, 1.5, 3.6);
    // Só o céu: junto à parede toda a luz é a c0 (sem lâmpada direta), e o `M` aponta para longe dela.
    let cena = CenaDeLuz {
        oclusores: vec![oclusor(
            vec![
                [x0 as f32, y0 as f32],
                [x1 as f32, y0 as f32],
                [x1 as f32, y1 as f32],
                [x0 as f32, y1 as f32],
            ],
            1.0,
        )],
        ceu: [0.5, 0.5, 0.6],
        com_normal: true,
        ..Default::default()
    };
    let l = rodar(&gpu, &cena);
    let gab = Gabarito::novo(&cena);
    let g = l.g;
    let colado = |k: usize| {
        let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
        let dx = (x0 - p[0]).max(p[0] - x1).max(0.0);
        let dy = (y0 - p[1]).max(p[1] - y1).max(0.0);
        let d = dx.hypot(dy);
        d > 0.0 && d <= 4.0 * g.texel_m
    };
    let mut pior = 0.0_f64;
    for n in [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
        let r = gabarito_por(&g, |p| gab.luz_em_n(p, n));
        let e = rmse_rel(&l.luz_em_n(n), &r, colado);
        eprintln!("junto ao opaco, n = {n:?}: rmse relativo {e:.4}");
        pior = pior.max(e);
    }
    assert!(
        pior <= TETO_JUNTO,
        "junto ao opaco: placa × gabarito {pior:.4} > {TETO_JUNTO}"
    );
}

/// O MEDIDO + folga (09/11, RADV, rmse relativo): `n = 0` 0,0489 · `(1, 0)` 0,0616 · `(−1, 0)` 0,0877 ·
/// `(0, 1)` 0,0554 · `(0, −1)` 0,0629. Visto vermelho sem a c0 de `Mx` (0,4123 · 0,5276).
const TETO_JUNTO: f64 = 0.10;
