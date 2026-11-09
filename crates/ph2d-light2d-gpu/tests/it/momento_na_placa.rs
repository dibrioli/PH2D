//! ⭐ P14 «L» — o momento `M` na PLACA contra o gabarito (`docs/PixelArt/P14_plano.md` §7): o receptor de
//! normal `n` lê `(F + n·M)/2π`; aqui `F`, `Mx` e `My` lidos da placa, a conta do `receber.wgsl` na CPU,
//! contra `ph2d_light2d_ref::Gabarito::luz_em_n` no centro de cada texel.

use ph2d_light2d_ecs::{CenaDeLuz, Emissor};
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
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
    rodar_com(gpu, cena, Config::padrao(grade(128, 0.04)), 1)
}

/// ⭐ P14c-placa 5 — os modos: as cascatas clássicas e a holográfica (a do produto).
fn modos() -> [(&'static str, Config); 2] {
    let c = Config::padrao(grade(128, 0.04));
    [
        ("clássica", c.clone()),
        (
            "holográfica",
            Config {
                holografica: Some(HOLOGRAFICA_PADRAO),
                ..c
            },
        ),
    ]
}

/// [`rodar`] com `cfg`, `quadros` quadros (o rebote assenta).
fn rodar_com(gpu: &ph2d_gpu::GpuContext, cena: &CenaDeLuz, cfg: Config, quadros: u32) -> Lido {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    for _ in 0..quadros {
        s.quadro(gpu, cena);
    }
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
/// ⭐ P14c-placa 5 — a holográfica (09/11, RADV): `n = 0` 0,0148 · `(1, 0)` 0,0331 · `(0, 1)` 0,0168 ·
/// `(−0,6, −0,8)` 0,0215.
const TETO_HOLO: f64 = 0.045;

#[test]
fn a_placa_e_o_gabarito_concordam_com_normal() {
    let Some(gpu) = gpu() else { return };
    let cena = cena();
    let gab = Gabarito::novo(&cena);
    let mut falhas = Vec::new();
    for ((modo, cfg), teto) in modos().into_iter().zip([TETO, TETO_HOLO]) {
        let l = rodar_com(&gpu, &cena, cfg, 1);
        for (n, e) in concordam(&l, |p, n| gab.luz_em_n(p, n), modo) {
            if e > teto {
                falhas.push(format!("{modo} n = {n:?}: {e:.4} > {teto}"));
            }
        }
    }
    assert!(falhas.is_empty(), "placa × gabarito: {falhas:#?}");
}

/// O rmse relativo de `luz_em_n` contra o gabarito `r` em cada normal (fora do miolo do disco e do 1/r).
fn concordam(
    l: &Lido,
    r: impl Fn([f64; 2], [f64; 2]) -> [f64; 3] + Sync,
    modo: &str,
) -> Vec<([f64; 2], f64)> {
    let g = l.g;
    // Fora: o miolo do disco e 3 texels em volta da lâmpada pontual (o 1/r).
    let longe = |k: usize| {
        let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
        let d = |c: [f32; 2]| (p[0] - f64::from(c[0])).hypot(p[1] - f64::from(c[1]));
        d(LAMPADA) > 3.0 * g.texel_m && d([1.0, 1.0]) > 0.15 + g.texel_m
    };
    let mut erros = Vec::new();
    for n in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [-0.6, -0.8]] {
        let r = gabarito_por(&g, |p| r(p, n));
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
        eprintln!("{modo} n = {n:?}: rmse relativo {e:.4} · erro máximo absoluto {pior:.4}");
        erros.push((n, e));
    }
    erros
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
    for (modo, cfg) in modos() {
        let (a, b) = (
            rodar_com(&gpu, &sem, cfg.clone(), 1),
            rodar_com(&gpu, &com, cfg, 1),
        );
        eprintln!(
            "{modo} · bytes do momento: sem {} · com {}",
            a.bytes, b.bytes
        );
        assert!(a.m.is_none() && a.bytes == 0, "{modo}");
        assert!(b.m.is_some() && b.bytes > 0, "{modo}");
        assert!(a.f == b.f, "{modo}: o momento mexeu no F");
    }
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
    let gab = Gabarito::novo(&cena);
    for ((modo, cfg), teto) in modos().into_iter().zip([TETO_JUNTO, TETO_JUNTO_HOLO]) {
        let pior = junto(
            &rodar_com(&gpu, &cena, cfg, 1),
            &gab,
            [x0, x1, y0, y1],
            modo,
        );
        assert!(
            pior <= teto,
            "{modo} junto ao opaco: placa × gabarito {pior:.4} > {teto}"
        );
    }
}

/// O pior rmse relativo nos texels de fora a ≤ 4 texels da parede `[x0, x1, y0, y1]`.
fn junto(l: &Lido, gab: &Gabarito, [x0, x1, y0, y1]: [f64; 4], modo: &str) -> f64 {
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
        eprintln!("{modo} junto ao opaco, n = {n:?}: rmse relativo {e:.4}");
        pior = pior.max(e);
    }
    pior
}

/// O MEDIDO + folga (09/11, RADV, rmse relativo): `n = 0` 0,0489 · `(1, 0)` 0,0616 · `(−1, 0)` 0,0877 ·
/// `(0, 1)` 0,0554 · `(0, −1)` 0,0629. Visto vermelho sem a c0 de `Mx` (0,4123 · 0,5276).
const TETO_JUNTO: f64 = 0.10;
/// ⭐ P14c-placa 5 — a holográfica (09/11, RADV): `n = 0` 0,0285 · `(1, 0)` 0,1298 · `(−1, 0)` 0,1551 · `(0, 1)`
/// 0,0515 · `(0, −1)` 0,0585. O `±x` (a parede é vertical) era o cone de 90° do nível 0: a metade tapada do cone
/// de `±y` ficava com `ω̂_q`. ⭐ P14c-placa 6 — curado pelas METADES do cone 0 (`hrc_metades_momento`, a diferença
/// `R₊ − R₋` × `ω̂_q⊥·sin(π/8)·sinc(π/8)`): 0,0285 · 0,0555 · 0,0621 · 0,0378 · 0,0373 (RADV). Visto vermelho com
/// `METADES_DO_MOMENTO = false`: os números da placa 5, exatos.
const TETO_JUNTO_HOLO: f64 = 0.075;

/// ⭐ P14c-placa 5 — COM O REBOTE (a sala do G9: chão e teto brancos, parede vermelha à esquerda, verde à
/// direita; o degrau Baixo): o `M` da luz que volta das paredes, nos quatro pontos do G9 e nas quatro normais de
/// eixo, contra o `Rebote::luz_em_n` do gabarito. O erro é relativo à luz do plano (`n = 0`) do gabarito: a face
/// de costas para a luz é quase zero.
#[test]
fn com_o_rebote_a_placa_e_o_gabarito_concordam_com_normal() {
    use crate::g9_rebote::{luz_na_placa, pontos, sala};
    use ph2d_light2d_ref::{OpcoesDoRebote, Rebote};
    let Some(gpu) = gpu() else { return };
    let cena = CenaDeLuz {
        com_normal: true,
        ..sala()
    };
    let serie = Rebote::resolver(
        &cena,
        OpcoesDoRebote {
            passo_m: 0.01,
            rebotes: 300,
        },
    );
    let (_, lado, texel) = crate::cenas::DEGRAUS[2];
    let mut falhas = Vec::new();
    for ((modo, cfg), teto) in modos().into_iter().zip(TETO_REBOTE) {
        let cfg = Config {
            rebote: true,
            grade: grade(lado, texel),
            ..cfg
        };
        let l = rodar_com(&gpu, &cena, cfg, 60);
        let [mx, my] = l.m.as_ref().expect("o momento");
        let mut pior = 0.0_f64;
        for p in pontos() {
            let plano = serie.luz_em(p);
            let (f, x, y) = (
                luz_na_placa(&l.f, &l.g, p),
                luz_na_placa(mx, &l.g, p),
                luz_na_placa(my, &l.g, p),
            );
            for n in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
                let r = serie.luz_em_n(p, n);
                let e = (0..3)
                    .map(|c| (f[c] + n[0] * x[c] + n[1] * y[c] - r[c]).abs() / plano[c])
                    .fold(0.0, f64::max);
                eprintln!("{modo} rebote {p:?} n = {n:?}: {e:.4}");
                pior = pior.max(e);
            }
        }
        eprintln!("{modo} rebote: pior {pior:.4}");
        if pior > teto {
            falhas.push(format!("{modo}: {pior:.4} > {teto}"));
        }
    }
    assert!(falhas.is_empty(), "com o rebote: {falhas:#?}");
}

/// ⭐ P14c-placa 5 — o MEDIDO + folga (09/11, RADV, degrau Baixo, o pior dos 16): clássica 0,1832 (o F sozinho
/// tem 0,131 no G9 deste degrau) · holográfica 0,1329 (junto às paredes, `±x`: o cone de 90° do nível 0; no meio
/// da sala ≤ 0,07). Visto vermelho com `ω̂ → −ω̂` só na holográfica. ⭐ P14c-placa 6 — com as METADES do cone 0
/// (`hrc_metades_momento`) a holográfica dá 0,0407; vista vermelha (0,1329) com `METADES_DO_MOMENTO = false`.
const TETO_REBOTE: [f64; 2] = [0.20, 0.05];

/// ⭐ P14c-placa 6 — DENTRO DE UM OPACO (onde o Pixel Sprite que bloqueia recebe): uma lâmpada à esquerda e o
/// céu, um corpo opaco largo; o `luz_em_n` dos texels de dentro (a mais de meio texel da borda) contra o
/// gabarito, que difunde `[F; M]` no miolo (`Miolo<9>`). Prende a difusão de `M` na pirâmide do miolo.
#[test]
fn dentro_do_opaco_o_momento_e_o_do_gabarito() {
    let Some(gpu) = gpu() else { return };
    let (x0, x1, y0, y1) = (2.4_f64, 3.2, 1.6, 3.4);
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Ponto {
            centro: [1.2, 2.3],
            intensidade: [1.0, 0.8, 0.6],
            alcance_m: 4.0,
        }],
        oclusores: vec![oclusor(
            vec![
                [x0 as f32, y0 as f32],
                [x1 as f32, y0 as f32],
                [x1 as f32, y1 as f32],
                [x0 as f32, y1 as f32],
            ],
            1.0,
        )],
        ceu: [0.05, 0.05, 0.08],
        com_normal: true,
        ..Default::default()
    };
    let gab = Gabarito::novo(&cena);
    for ((modo, cfg), teto) in modos().into_iter().zip([TETO_DENTRO, TETO_DENTRO_HOLO]) {
        let l = rodar_com(&gpu, &cena, cfg, 1);
        let g = l.g;
        let dentro = |k: usize| {
            let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            let d = (p[0] - x0).min(x1 - p[0]).min(p[1] - y0).min(y1 - p[1]);
            d > 0.5 * g.texel_m
        };
        let mut pior = 0.0_f64;
        for n in [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            let r = gabarito_por(&g, |p| gab.luz_em_n(p, n));
            let e = rmse_rel(&l.luz_em_n(n), &r, dentro);
            eprintln!("{modo} dentro do opaco, n = {n:?}: rmse relativo {e:.4}");
            pior = pior.max(e);
        }
        assert!(
            pior <= teto,
            "{modo} dentro do opaco: placa × gabarito {pior:.4} > {teto}"
        );
    }
}

/// O MEDIDO + folga (09/11, RADV, rmse relativo; `n = 0` é o miolo de `F`, o mesmo antes e depois):
/// `n = 0` 0,0645 · `(1, 0)` 0,1779 · `(−1, 0)` 0,0716 · `(0, 1)` 0,0941 · `(0, −1)` 0,0728. Antes (o bilinear
/// cru, e visto vermelho com o miolo de `M` desligado): 0,0645 · 0,8590 · 0,2770 · 0,1795 · 0,1527. O `(1, 0)`
/// é a face de COSTAS para a lâmpada (quase só o céu: o relativo de um número pequeno).
const TETO_DENTRO: f64 = 0.20;
/// ⭐ A holográfica (09/11, RADV, com as metades do cone 0): 0,0546 · 0,1703 · 0,0678 · 0,0826 · 0,0715 (sem
/// as metades 0,2137 · 0,0766 · 0,0923 · 0,0800 nos quatro de normal).
const TETO_DENTRO_HOLO: f64 = 0.19;

/// ⭐ P14c-placa 6 — O CANTO no passe próprio (`Canto::passe_proprio`, a `c3_fina_cantos`): a fina de `F` adia os
/// texels de duas faces, a de `M` (`c3_momento`) calcula-os em linha com o MESMO código do canto (as constantes
/// `comuns`, sem `CANTO_ADIA`). ⇒ `M` é o mesmo bit com e sem o passe próprio, e nos texels de dentro junto aos
/// quatro vértices do corpo o `luz_em_n` com o passe próprio fica no gabarito.
#[test]
fn com_o_canto_adiado_o_momento_e_o_do_canto_em_linha() {
    let Some(gpu) = gpu() else { return };
    let (x0, x1, y0, y1) = (2.4_f64, 3.2, 1.6, 3.4);
    let cena = CenaDeLuz {
        emissores: vec![Emissor::Ponto {
            centro: [1.2, 2.3],
            intensidade: [1.0, 0.8, 0.6],
            alcance_m: 4.0,
        }],
        oclusores: vec![oclusor(
            vec![
                [x0 as f32, y0 as f32],
                [x1 as f32, y0 as f32],
                [x1 as f32, y1 as f32],
                [x0 as f32, y1 as f32],
            ],
            1.0,
        )],
        ceu: [0.05, 0.05, 0.08],
        com_normal: true,
        ..Default::default()
    };
    let gab = Gabarito::novo(&cena);
    for ((modo, cfg), teto) in modos().into_iter().zip([TETO_CANTO, TETO_CANTO_HOLO]) {
        // A fina com 2 texels por célula (a visibilidade medida: só com ela o canto se adia).
        let cfg = Config { fina: 2, ..cfg };
        let adiado = Config {
            canto: ph2d_light2d_gpu::Canto {
                passe_proprio: true,
                ..cfg.canto
            },
            ..cfg.clone()
        };
        assert!(adiado.adia_o_canto() && !cfg.adia_o_canto(), "{modo}");
        let (a, b) = (
            rodar_com(&gpu, &cena, cfg, 1),
            rodar_com(&gpu, &cena, adiado, 1),
        );
        assert!(a.m == b.m, "{modo}: o passe próprio do canto mexeu no M");
        let g = b.g;
        // Dentro, a ≤ 3 texels das DUAS faces de um vértice.
        let no_canto = |k: usize| {
            let p = g.centro(k as u32 % g.largura, k as u32 / g.largura);
            let (dx, dy) = ((p[0] - x0).min(x1 - p[0]), (p[1] - y0).min(y1 - p[1]));
            dx > 0.0 && dy > 0.0 && dx.max(dy) <= 3.0 * g.texel_m
        };
        let mut pior = 0.0_f64;
        for n in [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
            let r = gabarito_por(&g, |p| gab.luz_em_n(p, n));
            let e = rmse_rel(&b.luz_em_n(n), &r, no_canto);
            eprintln!("{modo} canto adiado, n = {n:?}: rmse relativo {e:.4}");
            pior = pior.max(e);
        }
        assert!(pior <= teto, "{modo} canto adiado: {pior:.4} > {teto}");
    }
}

/// O MEDIDO + folga (09/11, RADV, fina 2, o passe próprio): clássica `n = 0` 0,3013 · `(1, 0)` 0,2966 · `(−1, 0)`
/// 0,3427 · `(0, 1)` 0,3616 · `(0, −1)` 0,3314; holográfica 0,3102 · 0,3169 · 0,3458 · 0,3649 · 0,3443. O grosso é o
/// do `F` (`n = 0`) no vértice; o `M` acrescenta ≤ 0,06. Visto vermelho (o `M` muda) com a `c3_momento` compilada
/// com `CANTO_ADIA` quando o canto se adia.
const TETO_CANTO: f64 = 0.38;
const TETO_CANTO_HOLO: f64 = 0.38;
