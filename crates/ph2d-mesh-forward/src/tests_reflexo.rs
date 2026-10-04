//! ⭐⭐⭐ **AS CAPTURAS DE REFLEXO** — uma peça espelhada mostra as VIZINHAS (e as sombras delas no
//! chão), como no Cycles.
//!
//! O oráculo (`docs/3DModeling/ferramentas/oraculo_reflexo_vizinhas_blender.py` →
//! `fixtures/oraculo_reflexo_vizinhas.csv`): as peças do oráculo do chão que tapa, sob o céu uniforme e
//! sobre o chão branco; uma delas metal branco, as outras foscas e escuras (albedo [`ALBEDO`]). A régua
//! que AFIRMA é a razão `viz/solo` nos pixels do espelho: o reflexo da cena inteira sobre o do céu
//! sozinho (o metal de cada peça a ver só o céu), a mesma conta dos dois lados.

use crate::tests::{ID, ambiente, cena};
use crate::tests_chao_tapa::{PECAS, metal};
use crate::tests_contacto::{
    LADO, Peca, Ponto, bordas, camera_do_blender, grades_de, linear, norm,
};
use crate::tests_sol::cubo;
use crate::{Forward, Instancia, Malha};

const ORACULO: &str = include_str!("../fixtures/oraculo_reflexo_vizinhas.csv");
pub(crate) const DE: [f32; 3] = [0.6, 0.15, 1.0];
const ALVO: [f32; 3] = [0.0, 0.2, 0.15];
const MEIA: f32 = 0.8;
/// O albedo das peças foscas do oráculo.
const ALBEDO: f32 = 0.3;

/// Uma linha do oráculo: `viz_espelho`, `viz_aspero`, `viz_caixa`, `solo_espelho`, `solo_aspero`.
pub(crate) struct Linha {
    pub(crate) col: [f32; 5],
}

pub(crate) fn oraculo() -> (Vec<Ponto>, Vec<Linha>) {
    ORACULO
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<f32> = l.split(',').map(|x| x.parse().expect("número")).collect();
            let p = Ponto {
                i: c[0] as u32,
                j: c[1] as u32,
                obj: c[2] as usize - 1,
                p: [c[3], c[4], c[5]],
                n: norm([c[6], c[7], c[8]]),
                vis: 1.0,
            };
            (
                p,
                Linha {
                    col: std::array::from_fn(|k| c[9 + k]),
                },
            )
        })
        .unzip()
}

/// A fosca do oráculo: difusa SÓ, de albedo [`ALBEDO`].
pub(crate) fn fosca() -> [f32; ph2d_material::wgsl::PACKED] {
    let s = ph2d_material::OpenPbr {
        base_color: [ALBEDO; 3],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare();
    ph2d_material::wgsl::pack(&s, ph2d_material::wgsl::EnvLobe::of(&s))
}

/// O desenhista com as peças do oráculo: a malha `k + 1` usa o material `k` e tem a grelha do contacto.
pub(crate) fn desenhista() -> Option<Forward> {
    desenhista_em(wgpu::Backends::all())
}

/// O mesmo, num backend escolhido.
pub(crate) fn desenhista_em(backends: wgpu::Backends) -> Option<Forward> {
    let mut fw = Forward::no_backend(backends, &ambiente())?;
    let g = grades_de(&PECAS);
    for (k, (_, r, caixa)) in PECAS.iter().enumerate() {
        let (p, n, idx) = if *caixa {
            cubo(2.0 * r)
        } else {
            crate::tests::esfera(*r)
        };
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

/// As peças `quais` do oráculo com os materiais `mats` (um por peça), da câmara do oráculo; com o
/// chão (e a caixa de sombra) ou sem nada.
pub(crate) fn desenha(
    fw: &mut Forward,
    quais: &[usize],
    mats: &[[f32; ph2d_material::wgsl::PACKED]],
    chao: bool,
) -> Vec<u8> {
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
    let mut c = cena(&objs, mats, camera_do_blender(DE, ALVO, MEIA));
    c.tamanho = (LADO, LADO);
    if chao {
        c.chao = Some(0.0);
        c.caixa_tan = Some(0.47);
    }
    fw.quadro(&c).expect("quadro")
}

/// O raio `p + t·r` acerta a peça `q`? (`t > 0`)
fn acerta((c, r, caixa): Peca, p: [f32; 3], d: [f32; 3]) -> bool {
    let o = [0, 1, 2].map(|e| p[e] - c[e]);
    if caixa {
        let (mut t0, mut t1) = (1.0e-4f32, f32::INFINITY);
        for e in 0..3 {
            if d[e].abs() < 1.0e-9 {
                if o[e].abs() > r {
                    return false;
                }
                continue;
            }
            let (a, b) = ((-r - o[e]) / d[e], (r - o[e]) / d[e]);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
        return t0 <= t1;
    }
    let b = o[0] * d[0] + o[1] * d[1] + o[2] * d[2];
    let cc = o[0] * o[0] + o[1] * o[1] + o[2] * o[2] - r * r;
    let disc = b * b - cc;
    disc >= 0.0 && -b + disc.sqrt() > 1.0e-4
}

/// A reflectida da câmara (ortográfica, a olhar ao longo de `−de`) em `p` acerta uma VIZINHA de `p.obj`?
fn reflete_vizinha(p: &Ponto) -> bool {
    let f = norm(DE.map(|c| -c));
    let fn_ = f[0] * p.n[0] + f[1] * p.n[1] + f[2] * p.n[2];
    let r: [f32; 3] = std::array::from_fn(|e| f[e] - 2.0 * fn_ * p.n[e]);
    PECAS
        .iter()
        .enumerate()
        .any(|(k, q)| k != p.obj && acerta(*q, p.p, r))
}

/// O que a régua de um espelho mede: todos os pixels; os do MIOLO do reflexo de uma vizinha; e a FAIXA
/// do contorno dela (um pixel cujo vizinho de `3 × 3` cai do outro lado). ⚠️ No contorno o Cycles do
/// oráculo é PONTUAL (filtro `0,01 px`) e o nosso quadro filtrado: um anel de `1 px` lê até `0,8` sem
/// nada fora do sítio — a faixa imprime-se e só a média dela tem barra (o reflexo no sítio errado
/// leva-a; sem a paralaxe a média geral da caixa vai a `0,27`).
struct Medida {
    n: usize,
    media: f32,
    nv: usize,
    media_viz: f32,
    pior_viz: f32,
    nb: usize,
    media_borda: f32,
}

/// A régua sobre os pixels da peça `espelho`: a razão `viz/solo` contra a do Cycles (colunas `cv`, `cs`).
fn mede(
    pontos: &[Ponto],
    linhas: &[Linha],
    borda: &[bool],
    espelho: usize,
    (viz, solo): (&[u8], &[u8]),
    (cv, cs): (usize, usize),
) -> Medida {
    let l = LADO as usize;
    let mut reflete = vec![None; l * l];
    for p in pontos.iter().filter(|p| p.obj == espelho) {
        reflete[(p.j * LADO + p.i) as usize] = Some(reflete_vizinha(p));
    }
    let contorno = |p: &Ponto| {
        let c = reflete[(p.j * LADO + p.i) as usize];
        (-1i32..=1).any(|dy| {
            (-1i32..=1).any(|dx| {
                let (x, y) = (p.i as i32 + dx, p.j as i32 + dy);
                (0..LADO as i32).contains(&x)
                    && (0..LADO as i32).contains(&y)
                    && reflete[y as usize * l + x as usize].is_some_and(|o| Some(o) != c)
            })
        })
    };
    let (mut n, mut s, mut nv, mut sv, mut pior) = (0usize, 0.0f32, 0usize, 0.0f32, 0.0f32);
    let (mut nb, mut sb) = (0usize, 0.0f32);
    for (p, l) in pontos.iter().zip(linhas) {
        let k = (p.j * LADO + p.i) as usize;
        if p.obj != espelho || borda[k] {
            continue;
        }
        let (lv, ls) = (linear(viz[k * 4 + 1]), linear(solo[k * 4 + 1]));
        if ls < 0.05 || l.col[cs] < 0.05 {
            continue;
        }
        let e = (lv / ls - l.col[cv] / l.col[cs]).abs();
        s += e;
        n += 1;
        if contorno(p) {
            sb += e;
            nb += 1;
        } else if reflete_vizinha(p) {
            sv += e;
            nv += 1;
            pior = pior.max(e);
        }
    }
    Medida {
        n,
        media: s / n.max(1) as f32,
        nv,
        media_viz: sv / nv.max(1) as f32,
        pior_viz: pior,
        nb,
        media_borda: sb / nb.max(1) as f32,
    }
}

/// A sonda: com `PH2D_REFLEXO_FOTOS=<pasta>`, grava `<nome>.pgm` — a razão nossa, a do Cycles e
/// `4 ×` a diferença, lado a lado, nos pixels do espelho.
fn fotografa(
    nome: &str,
    pontos: &[Ponto],
    linhas: &[Linha],
    espelho: usize,
    (viz, solo): (&[u8], &[u8]),
    (cv, cs): (usize, usize),
) {
    let Ok(pasta) = std::env::var("PH2D_REFLEXO_FOTOS") else {
        return;
    };
    let l = LADO as usize;
    let mut img = vec![0u8; 3 * l * l];
    for (p, li) in pontos.iter().zip(linhas) {
        if p.obj != espelho {
            continue;
        }
        let k = (p.j * LADO + p.i) as usize;
        let (lv, ls) = (linear(viz[k * 4 + 1]), linear(solo[k * 4 + 1]));
        let nosso = lv / ls.max(1.0e-3);
        let ciclos = li.col[cv] / li.col[cs].max(1.0e-3);
        let y = p.j as usize * 3 * l;
        let b = |x: f32| (x.clamp(0.0, 1.0) * 255.0) as u8;
        img[y + p.i as usize] = b(nosso);
        img[y + l + p.i as usize] = b(ciclos);
        img[y + 2 * l + p.i as usize] = b(4.0 * (nosso - ciclos).abs());
    }
    let mut f = format!("P5 {} {l} 255\n", 3 * l).into_bytes();
    f.extend_from_slice(&img);
    let _ = std::fs::write(format!("{pasta}/{nome}.pgm"), f);
}

/// ⭐⭐⭐ **O espelho mostra as vizinhas como no Cycles** — a esfera pousada (nítida e áspera) e a caixa
/// (faces planas: a paralaxe), cada uma contra o Cycles nos pixels dela; à parte, os que refletem uma
/// vizinha (o raio reflectido acerta-a, conta analítica sobre a geometria do oráculo).
#[test]
#[ignore = "precisa de aparelho"]
fn o_espelho_mostra_as_vizinhas_como_no_cycles() {
    let Some(mut fw) = desenhista() else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let (pontos, linhas) = oraculo();
    let (f, m0, m5) = (fosca(), metal(0.0), metal(0.5));
    let base = desenha(&mut fw, &[0, 1, 2], &[f, f, f], false);
    let borda = bordas(&pontos, &base);
    let mut falhas = Vec::new();
    for (nome, espelho, mats, m_solo, cols) in [
        ("esfera nítida", 1usize, [f, m0, f], m0, (0usize, 3usize)),
        ("esfera áspera", 1, [f, m5, f], m5, (1, 4)),
        ("caixa nítida", 0, [m0, f, f], m0, (2, 3)),
    ] {
        let viz = desenha(&mut fw, &[0, 1, 2], &mats, true);
        let mut so = [f, f, f];
        so[espelho] = m_solo;
        let solo = desenha(&mut fw, &[espelho], &so, false);
        let m = mede(&pontos, &linhas, &borda, espelho, (&viz, &solo), cols);
        fotografa(nome, &pontos, &linhas, espelho, (&viz, &solo), cols);
        eprintln!(
            "{nome}: {} px · |Δ| médio {:.4} · no miolo do reflexo de uma vizinha ({} px) {:.4} · máx \
             ali {:.3} · na faixa do contorno ({} px) {:.4}",
            m.n, m.media, m.nv, m.media_viz, m.pior_viz, m.nb, m.media_borda
        );
        assert!(
            m.n > 1500 && m.nv > 50 && m.nb > 50,
            "a fixtura encolheu: {} / {} / {} px",
            m.n,
            m.nv,
            m.nb
        );
        if !(m.media < 0.03 && m.media_viz < 0.05) {
            falhas.push(nome);
        }
    }
    assert!(
        falhas.is_empty(),
        "o reflexo das vizinhas afastou-se do Cycles: {falhas:?}"
    );
}
