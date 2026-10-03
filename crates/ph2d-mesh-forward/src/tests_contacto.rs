//! ⭐⭐⭐ **O CONTACTO ENTRE PEÇAS na placa** — a cena do oráculo
//! (`docs/3DModeling/ferramentas/oraculo_contacto_blender.py` → `ph2d-contacto/fixtures/oraculo_contacto.csv`)
//! desenhada da MESMA câmara, duas vezes: com as grelhas e sem elas. Sob o céu chapado, sem chão nem
//! caixa, a razão entre as duas é a visibilidade que o contacto dá a cada pixel; compara-se com o
//! Cycles e com a lei da CPU ([`ph2d_contacto::Grade::oclusao`]) nos pontos e normais do Cycles.

use crate::tests::{ID, ambiente, cena, esfera, material_cinza};
use crate::tests_sol::cubo;
use crate::{Camera, Forward, Instancia, Malha};
use ph2d_contacto::{Grade, Volume};

const ORACULO: &str = include_str!("../../ph2d-contacto/fixtures/oraculo_contacto.csv");
const LADO: u32 = 256;
const DE: [f32; 3] = [1.0, 0.8, -1.3];
const ALVO: [f32; 3] = [0.15, 0.3, -0.1];
const MEIA: f32 = 0.75;
/// As peças: `(centro, meia-aresta da caixa ou raio, é caixa)` — as do oráculo.
const PECAS: [([f32; 3], f32, bool); 4] = [
    ([0.0, 0.2, 0.0], 0.2, true),
    ([0.0, 0.6, 0.0], 0.2, false),
    ([0.45, 0.25, 0.0], 0.25, false),
    ([0.0, 0.2, -0.37], 0.15, false),
];

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|c| c / l)
}

fn cruz(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A câmara ortográfica do Blender (`-Z` para o alvo, `Y` para cima) no nosso recorte.
fn camera() -> Camera {
    let f = norm(DE.map(|c| -c));
    let up0 = [0.0, 1.0, 0.0];
    let up = norm([0, 1, 2].map(|e| up0[e] - dot(up0, f) * f[e]));
    let r = cruz(f, up);
    let mut vp = [[0.0f32; 4]; 4];
    for c in 0..3 {
        vp[c] = [r[c] / MEIA, up[c] / MEIA, 0.1 * f[c], 0.0];
    }
    vp[3] = [
        -dot(ALVO, r) / MEIA,
        -dot(ALVO, up) / MEIA,
        0.5 - 0.1 * dot(ALVO, f),
        1.0,
    ];
    Camera {
        view_proj: vp,
        olho: [0.0; 3],
        perspectiva: false,
        dir_vista: f,
    }
}

fn sdf_local(k: usize, q: [f32; 3]) -> f32 {
    let (_, r, caixa) = PECAS[k];
    if caixa {
        let d = q.map(|x| x.abs() - r);
        let fora = d.map(|x| x.max(0.0));
        dot(fora, fora).sqrt() + d[0].max(d[1]).max(d[2]).min(0.0)
    } else {
        dot(q, q).sqrt() - r
    }
}

/// A grelha de cada peça, no referencial da MALHA (centrada na origem).
fn grades() -> Vec<Grade> {
    (0..PECAS.len())
        .map(|k| {
            let (_, r, caixa) = PECAS[k];
            let m = r * (1.0 + 4.0 / 47.0);
            let vol = Volume::de([-m; 3], [m; 3], 48, |pts| {
                pts.iter().map(|p| sdf_local(k, *p)).collect()
            });
            Grade::constroi(&vol, [0.0; 3], if caixa { r * 3.0f32.sqrt() } else { r })
        })
        .collect()
}

/// A pose de cada peça: no centro do oráculo, e as ESFERAS rodadas (a geometria é a mesma; a grelha
/// tem de ser lida no referencial delas — ponto e normal).
fn pose(k: usize, c: [f32; 3]) -> [[f32; 4]; 4] {
    let mut m = ID;
    if !PECAS[k].2 {
        let a = 0.7 + k as f32;
        let (co, si) = (a.cos(), a.sin());
        // Em torno de (1, 1, 0)/√2 (Rodrigues), por colunas.
        let u = [
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ];
        for col in 0..3 {
            for lin in 0..3 {
                let id = if col == lin { 1.0 } else { 0.0 };
                let cr = [[0.0, u[2], -u[1]], [-u[2], 0.0, u[0]], [u[1], -u[0], 0.0]][col][lin];
                m[col][lin] = co * id + si * cr + (1.0 - co) * u[col] * u[lin];
            }
        }
    }
    m[3][..3].copy_from_slice(&c);
    m
}

fn desenha(fw: &mut Forward) -> Vec<u8> {
    desenha_com(fw, |k, c| pose(k, c))
}

fn desenha_com(fw: &mut Forward, poe: impl Fn(usize, [f32; 3]) -> [[f32; 4]; 4]) -> Vec<u8> {
    let objs: Vec<Instancia> = PECAS
        .iter()
        .enumerate()
        .map(|(k, (c, _, _))| Instancia {
            malha: k as u64 + 1,
            modelo: poe(k, *c),
        })
        .collect();
    let mats = [material_cinza()];
    let mut c = cena(&objs, &mats, camera());
    c.tamanho = (LADO, LADO);
    fw.quadro(&c).expect("quadro")
}

fn desenhista(com_grades: bool) -> Option<Forward> {
    let mut fw = Forward::no_aparelho(&ambiente())?;
    let g = grades();
    for (k, (_, r, caixa)) in PECAS.iter().enumerate() {
        let (p, n, idx) = if *caixa { cubo(2.0 * r) } else { esfera(*r) };
        let ao = vec![1.0; p.len()];
        let mat = vec![0u32; p.len()];
        let id = k as u64 + 1;
        fw.sobe(
            id,
            &Malha {
                posicoes: &p,
                normais: &n,
                ao: &ao,
                material: &mat,
                indices: &idx,
            },
        );
        if com_grades {
            fw.sobe_contacto(id, &g[k]);
        }
    }
    Some(fw)
}

fn linear(b: u8) -> f32 {
    let x = f32::from(b) / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

struct Ponto {
    i: u32,
    j: u32,
    obj: usize,
    p: [f32; 3],
    n: [f32; 3],
    vis: f32,
}

fn oraculo() -> Vec<Ponto> {
    ORACULO
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<f32> = l.split(',').map(|x| x.parse().expect("número")).collect();
            Ponto {
                i: c[0] as u32,
                j: c[1] as u32,
                obj: c[2] as usize - 1,
                p: [c[3], c[4], c[5]],
                n: norm([c[6], c[7], c[8]]),
                vis: c[9],
            }
        })
        .collect()
}

/// Os pixels que não se comparam: perto da silhueta, de outra peça ou de uma ARESTA da caixa (a
/// normal salta, e o pixel de um lado cai na face vizinha do outro). `fundo` = um quadro qualquer.
fn bordas(pontos: &[Ponto], fundo: &[u8]) -> Vec<bool> {
    let mut dono = vec![usize::MAX; (LADO * LADO) as usize];
    let mut normal = vec![[0.0f32; 3]; (LADO * LADO) as usize];
    for p in pontos {
        dono[(p.j * LADO + p.i) as usize] = p.obj;
        normal[(p.j * LADO + p.i) as usize] = p.n;
    }
    let mut borda = vec![true; (LADO * LADO) as usize];
    for p in pontos {
        let n0 = p.n;
        borda[(p.j * LADO + p.i) as usize] = (-2i32..=2).any(|dy| {
            (-2i32..=2).any(|dx| {
                let (x, y) = (p.i as i32 + dx, p.j as i32 + dy);
                if x < 0 || y < 0 || x >= LADO as i32 || y >= LADO as i32 {
                    return true;
                }
                let k = (y as u32 * LADO + x as u32) as usize;
                fundo[k * 4 + 3] == 0
                    || (dono[k] != usize::MAX && (dono[k] != p.obj || dot(normal[k], n0) < 0.9))
            })
        });
    }
    borda
}

/// ⭐⭐⭐ **O contacto na placa é o do Cycles e o da lei da CPU** — pixel a pixel, longe das silhuetas
/// e das arestas (onde a malha facetada e a do Blender não cobrem o mesmo ponto). Controlo: sem as grelhas a razão
/// é `1` e o erro é o próprio escurecimento.
#[test]
#[ignore = "precisa de aparelho"]
fn o_contacto_na_placa_e_o_do_cycles() {
    let (Some(mut com), Some(mut sem)) = (desenhista(true), desenhista(false)) else {
        eprintln!("sem aparelho — o gate não corre aqui");
        return;
    };
    let (a, b) = (desenha(&mut com), desenha(&mut sem));
    let pontos = oraculo();
    let borda = bordas(&pontos, &b);
    let g: Vec<Grade> = grades().iter().map(Grade::em_f16).collect();
    let (mut n, mut s_c, mut s_l, mut s_sem, mut pior_c, mut pior_l) =
        (0usize, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let (mut np, mut sp) = (0usize, 0.0f32);
    for p in &pontos {
        if borda[(p.j * LADO + p.i) as usize] {
            continue;
        }
        let k = ((p.j * LADO + p.i) * 4 + 1) as usize;
        let (la, lb) = (linear(a[k]), linear(b[k]));
        if lb < 0.05 {
            continue;
        }
        let nosso = la / lb;
        // A lei da CPU no ponto do Cycles (as grelhas no referencial da malha).
        let cpu: f32 = (0..PECAS.len())
            .filter(|&o| o != p.obj)
            .map(|o| {
                // No referencial da peça: a pose é rígida, a inversa é a transposta.
                let m = pose(o, PECAS[o].0);
                let d = [0, 1, 2].map(|e| p.p[e] - m[3][e]);
                let local = |v: [f32; 3]| [0, 1, 2].map(|c| (0..3).map(|r| m[c][r] * v[r]).sum());
                1.0 - g[o].oclusao(local(d), local(p.n))
            })
            .product();
        let (dc, dl) = ((nosso - p.vis).abs(), (nosso - cpu).abs());
        s_c += dc;
        s_l += dl;
        s_sem += (1.0 - p.vis).abs();
        pior_c = pior_c.max(dc);
        pior_l = pior_l.max(dl);
        n += 1;
        if p.vis < 0.9 {
            sp += dc;
            np += 1;
        }
    }
    let nf = n as f32;
    eprintln!(
        "{n} px · contra o Cycles |Δ| médio {:.4} · perto {:.4} · máx {pior_c:.3} · contra a CPU médio {:.4} · \
         máx {pior_l:.3} · sem a lei {:.4}",
        s_c / nf,
        sp / np as f32,
        s_l / nf,
        s_sem / nf
    );
    assert!(n > 5000, "a fixtura encolheu: {n} px");
    assert!(
        s_sem / nf > 0.1,
        "CONTROLO: sem a lei o erro tem de ser o escurecimento"
    );
    // Medido (03/10, `8 695` px, as esferas rodadas): contra o Cycles médio `0,0088`, perto
    // `0,0126`, máx `0,063`; contra a CPU médio `0,0031`, máx `0,010`; sem a lei `0,181`.
    assert!(
        s_c / nf < 0.013 && sp / (np as f32) < 0.02 && pior_c < 0.12,
        "a placa afastou-se do Cycles"
    );
    assert!(
        s_l / nf < 0.005 && pior_l < 0.02,
        "a placa afastou-se da lei da CPU"
    );
}

/// ⭐⭐ **O escurecimento anda com a peça** — a grelha vive no referencial dela, logo mover a esfera
/// encostada à face `+x` da caixa para longe tira-lhe a sombra de contacto no MESMO desenhista (nada
/// se refaz, só a afim do quadro). Controlo: antes de a mover, a face escurece.
#[test]
#[ignore = "precisa de aparelho"]
fn o_contacto_anda_com_a_peca() {
    let (Some(mut com), Some(mut sem)) = (desenhista(true), desenhista(false)) else {
        return;
    };
    let longe = |k: usize, c: [f32; 3]| pose(k, if k == 2 { [3.0, 0.25, 0.0] } else { c });
    let antes = desenha(&mut com);
    let (depois, base) = (desenha_com(&mut com, longe), desenha_com(&mut sem, longe));
    let referencia = desenha(&mut sem);
    // Os pixels da caixa que o Cycles escurece pela esfera do lado (perto da face +x, longe das outras).
    let (mut n, mut antes_min, mut depois_min) = (0usize, 1.0f32, 1.0f32);
    let pontos = oraculo();
    let borda = bordas(&pontos, &referencia);
    for p in &pontos {
        if p.obj != 0 || p.n[0] < 0.9 || p.vis > 0.7 || borda[(p.j * LADO + p.i) as usize] {
            continue;
        }
        let k = ((p.j * LADO + p.i) * 4 + 1) as usize;
        let r = |a: &[u8], b: &[u8]| linear(a[k]) / linear(b[k]).max(1.0e-4);
        antes_min = antes_min.min(r(&antes, &referencia));
        depois_min = depois_min.min(r(&depois, &base));
        n += 1;
    }
    eprintln!(
        "{n} px da face +x · antes {antes_min:.3} · depois de afastar a esfera {depois_min:.3}"
    );
    assert!(
        n > 50 && antes_min < 0.6,
        "CONTROLO: a esfera encostada escurece a face"
    );
    assert!(
        depois_min > 0.97,
        "a sombra ficou onde a esfera estava: {depois_min}"
    );
}
