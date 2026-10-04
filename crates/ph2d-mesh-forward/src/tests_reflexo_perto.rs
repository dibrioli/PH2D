//! ⭐⭐⭐ **OS REFLEXOS DE PERTO** — o report do dono de 04/10: com o cromo a encher o ecrã, o reflexo das
//! vizinhas tinha bordas em DEGRAUS, MORDIDAS e cunhas escuras; o oráculo de longe
//! ([`crate::tests_reflexo`]) não o via (o contorno de cada vizinha refletida era `1 px`).
//!
//! O oráculo (`docs/3DModeling/ferramentas/oraculo_reflexo_perto_blender.py` →
//! `fixtures/oraculo_reflexo_perto.csv.gz`): a arrumação da cena 42 (o cromo e quatro vizinhas foscas de
//! albedos diferentes), o cromo a `512 px`, nítido e a `0,05`. Só os pixels do cromo; o ponto e a normal
//! de cada um tiram-se da esfera e da câmara.

use std::io::Read;

use crate::tests::{ID, ambiente, cena, esfera};
use crate::tests_chao_tapa::metal;
use crate::tests_contacto::{Peca, camera_do_blender, grades_de, linear, norm};
use crate::tests_sol::cubo;
use crate::{Forward, Instancia, Malha};

const ORACULO: &[u8] = include_bytes!("../fixtures/oraculo_reflexo_perto.csv.gz");
const LADO: u32 = 512;
pub(crate) const DE: [f32; 3] = [1.0, 0.45, 0.3];
const ALVO: [f32; 3] = [0.0, 0.3, 0.0];
const MEIA: f32 = 0.34;
/// As peças (o cromo primeiro) e o albedo das foscas.
pub(crate) const PECAS: [Peca; 5] = [
    ([0.0, 0.3, 0.0], 0.3, false),
    ([0.0, 0.2, -0.7], 0.2, false),
    ([0.0, 0.18, 0.72], 0.18, true),
    ([0.55, 0.17, -0.25], 0.17, false),
    ([-0.6, 0.15, 0.2], 0.15, true),
];
const ALBEDOS: [f32; 4] = [0.8, 0.15, 0.5, 0.3];

/// Um pixel do cromo: `(i, j)`, o ponto, a normal e `[viz, viz05, solo, solo05]` do Cycles.
pub(crate) struct Px {
    pub(crate) i: u32,
    pub(crate) j: u32,
    pub(crate) p: [f32; 3],
    pub(crate) n: [f32; 3],
    pub(crate) col: [f32; 4],
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// O raio da câmara do oráculo no centro do pixel `(i, j)`: origem e direcção.
fn raio(i: u32, j: u32) -> ([f32; 3], [f32; 3]) {
    let f = norm(DE.map(|c| -c));
    let up0 = [0.0, 1.0, 0.0];
    let up = norm([0, 1, 2].map(|e| up0[e] - dot(up0, f) * f[e]));
    let r = [
        f[1] * up[2] - f[2] * up[1],
        f[2] * up[0] - f[0] * up[2],
        f[0] * up[1] - f[1] * up[0],
    ];
    let x = (i as f32 + 0.5) / LADO as f32 * 2.0 - 1.0;
    let y = 1.0 - (j as f32 + 0.5) / LADO as f32 * 2.0;
    let o = [0, 1, 2].map(|e| ALVO[e] + MEIA * (x * r[e] + y * up[e]) - 10.0 * f[e]);
    (o, f)
}

pub(crate) fn oraculo() -> Vec<Px> {
    let mut texto = String::new();
    flate2::read::GzDecoder::new(ORACULO)
        .read_to_string(&mut texto)
        .expect("o oráculo descomprime");
    let (c, r0, _) = PECAS[0];
    texto
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let v: Vec<f32> = l.split(',').map(|x| x.parse().expect("número")).collect();
            let (i, j) = (v[0] as u32, v[1] as u32);
            let (o, d) = raio(i, j);
            let q = [0, 1, 2].map(|e| o[e] - c[e]);
            let b = dot(q, d);
            let t = -b - (b * b - (dot(q, q) - r0 * r0)).max(0.0).sqrt();
            let p = [0, 1, 2].map(|e| o[e] + t * d[e]);
            let n = norm([0, 1, 2].map(|e| p[e] - c[e]));
            Px {
                i,
                j,
                p,
                n,
                col: [v[2], v[3], v[4], v[5]],
            }
        })
        .collect()
}

fn fosca(a: f32) -> [f32; ph2d_material::wgsl::PACKED] {
    let s = ph2d_material::OpenPbr {
        base_color: [a; 3],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare();
    ph2d_material::wgsl::pack(&s, ph2d_material::wgsl::EnvLobe::of(&s))
}

pub(crate) fn desenhista() -> Option<Forward> {
    let mut fw = Forward::no_aparelho(&ambiente())?;
    let g = grades_de(&PECAS);
    for (k, (_, r, caixa)) in PECAS.iter().enumerate() {
        let (p, n, idx) = if *caixa { cubo(2.0 * r) } else { esfera(*r) };
        let ao = vec![1.0; p.len()];
        let mat = vec![k as u32; p.len()];
        fw.sobe(
            k as u64 + 1,
            &Malha {
                posicoes: &p,
                normais: &n,
                ao: &ao,
                material: &mat,
                indices: &idx,
            },
        );
        fw.sobe_contacto(k as u64 + 1, &g[k]);
    }
    Some(fw)
}

pub(crate) fn desenha(
    fw: &mut Forward,
    cromo: [f32; ph2d_material::wgsl::PACKED],
    todas: bool,
) -> Vec<u8> {
    let quais: &[usize] = if todas { &[0, 1, 2, 3, 4] } else { &[0] };
    let objs: Vec<Instancia> = quais
        .iter()
        .map(|&k| {
            let mut m = ID;
            m[3][..3].copy_from_slice(&PECAS[k].0);
            Instancia {
                malha: k as u64 + 1,
                modelo: m,
            }
        })
        .collect();
    let mats = [
        cromo,
        fosca(ALBEDOS[0]),
        fosca(ALBEDOS[1]),
        fosca(ALBEDOS[2]),
        fosca(ALBEDOS[3]),
    ];
    let mut c = cena(&objs, &mats, camera_do_blender(DE, ALVO, MEIA));
    c.tamanho = (LADO, LADO);
    if todas {
        c.chao = Some(0.0);
        c.caixa_tan = Some(0.47);
    }
    fw.quadro(&c).expect("quadro")
}

/// O raio reflectido em `p` acerta uma vizinha? (conta analítica sobre a geometria do oráculo)
fn reflete_vizinha(p: &Px) -> bool {
    let f = norm(DE.map(|c| -c));
    let fn_ = dot(f, p.n);
    let r: [f32; 3] = std::array::from_fn(|e| f[e] - 2.0 * fn_ * p.n[e]);
    PECAS[1..]
        .iter()
        .any(|q| crate::tests_sonda_cpu::acerta_em(*q, p.p, r).is_some())
}

/// `(px, |Δ| médio, px no miolo do reflexo de uma vizinha, |Δ| ali, máx ali, px na faixa do contorno,
/// |Δ| ali)` — a razão `viz/solo` contra a do Cycles; e a foto `nossa | Cycles | 4×|Δ|` se pedida.
fn mede(
    px: &[Px],
    (viz, solo): (&[u8], &[u8]),
    (cv, cs): (usize, usize),
    foto: Option<&str>,
) -> [f32; 8] {
    let l = LADO as usize;
    let mut cromo = vec![None; l * l];
    for p in px {
        cromo[p.j as usize * l + p.i as usize] = Some(reflete_vizinha(p));
    }
    let vizinho = |p: &Px, raio: i32, f: &dyn Fn(Option<bool>) -> bool| {
        (-raio..=raio).any(|dy| {
            (-raio..=raio).any(|dx| {
                let (x, y) = (p.i as i32 + dx, p.j as i32 + dy);
                !(0..LADO as i32).contains(&x)
                    || !(0..LADO as i32).contains(&y)
                    || f(cromo[y as usize * l + x as usize])
            })
        })
    };
    let mut img = vec![0u8; 3 * l * l];
    let (mut n, mut s, mut nv, mut sv, mut pior, mut nb, mut sb) =
        (0usize, 0.0f32, 0usize, 0.0f32, 0.0f32, 0usize, 0.0f32);
    let mut grosseiros = 0usize;
    for p in px {
        // Longe da silhueta do cromo e de quem o tapa na câmara.
        if vizinho(p, 2, &|c| c.is_none()) {
            continue;
        }
        let k = p.j as usize * l + p.i as usize;
        let (lv, ls) = (linear(viz[k * 4 + 1]), linear(solo[k * 4 + 1]));
        if ls < 0.05 || p.col[cs] < 0.05 {
            continue;
        }
        let (nosso, ciclos) = (lv / ls, p.col[cv] / p.col[cs]);
        let e = (nosso - ciclos).abs();
        let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
        img[p.j as usize * 3 * l + p.i as usize] = b(nosso);
        img[p.j as usize * 3 * l + l + p.i as usize] = b(ciclos);
        img[p.j as usize * 3 * l + 2 * l + p.i as usize] = b(4.0 * e);
        s += e;
        n += 1;
        grosseiros += usize::from(e > 0.2);
        let c = cromo[k];
        if vizinho(p, 2, &|o| o.is_some() && o != c) {
            sb += e;
            nb += 1;
        } else if c == Some(true) {
            sv += e;
            nv += 1;
            pior = pior.max(e);
        }
    }
    if let (Some(nome), Ok(pasta)) = (foto, std::env::var("PH2D_REFLEXO_FOTOS")) {
        let mut f = format!("P5 {} {l} 255\n", 3 * l).into_bytes();
        f.extend_from_slice(&img);
        let _ = std::fs::write(format!("{pasta}/{nome}.pgm"), f);
    }
    let m = |a: f32, b: usize| a / b.max(1) as f32;
    [
        n as f32,
        m(s, n),
        nv as f32,
        m(sv, nv),
        pior,
        nb as f32,
        m(sb, nb),
        grosseiros as f32,
    ]
}

/// ⭐⭐⭐ **De perto, o cromo mostra as vizinhas como no Cycles** — nítido e a `0,05`. No miolo do
/// reflexo de uma vizinha (a mais de `2 px` do contorno dela) os degraus e as mordidas do report são
/// erro; a faixa do contorno imprime-se (o Cycles do oráculo é pontual).
#[test]
#[ignore = "precisa de aparelho"]
fn de_perto_o_cromo_mostra_as_vizinhas_como_no_cycles() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let px = oraculo();
    let mut falhas = Vec::new();
    for (nome, rug, cols) in [("nítido", 0.0f32, (0usize, 2usize)), ("0,05", 0.05, (1, 3))] {
        let viz = desenha(&mut fw, metal(rug), true);
        let solo = desenha(&mut fw, metal(rug), false);
        let m = mede(&px, (&viz, &solo), cols, Some(&format!("perto_{rug}")));
        eprintln!(
            "cromo {nome}: {} px · |Δ| médio {:.4} · no miolo do reflexo de uma vizinha ({} px) {:.4} · máx \
             ali {:.3} · na faixa do contorno ({} px) {:.4} · GROSSEIROS (|Δ| > 0,2) {}",
            m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7]
        );
        assert!(m[0] > 50_000.0 && m[2] > 3000.0, "a fixtura encolheu");
        if !(m[1] < BARRAS.0 && m[3] < BARRAS.1 && m[7] < BARRAS.2) {
            falhas.push(nome);
        }
    }
    assert!(
        falhas.is_empty(),
        "de perto o reflexo afastou-se do Cycles: {falhas:?}"
    );
}

/// As barras: `(|Δ| médio, |Δ| médio no miolo do reflexo de uma vizinha, px GROSSEIROS)`. Medido
/// (04/10): a lei de 03/10 (ponto fixo, octaedro 256) `0,0080 / 0,0185 / 1 237` e `0,0089 / 0,0271 /
/// 1 141`; a busca a 256 `… / 0,0162 / 1 114`; a busca a 512 `0,0072 / 0,0126 / 919` e `0,0078 / 0,0220
/// / 831`. ⚠️ Os grosseiros são sobretudo a faixa de `1–2 px` de TODO contorno (o Cycles pontual): a
/// régua não separa uma borda serrilhada de uma lisa — a FOTO (`PH2D_REFLEXO_FOTOS`) é o juiz disso.
const BARRAS: (f32, f32, f32) = (0.012, 0.025, 1000.0);
