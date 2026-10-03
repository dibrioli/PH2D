//! Os gates da lei na CPU.

use crate::{Grade, Volume};

const ORACULO: &str = include_str!("../fixtures/oraculo_contacto.csv");

/// As peças do oráculo: `(centro, meia-aresta da caixa ou raio da esfera, é caixa)`.
const PECAS: [([f32; 3], f32, bool); 4] = [
    ([0.0, 0.2, 0.0], 0.2, true),
    ([0.0, 0.6, 0.0], 0.2, false),
    ([0.45, 0.25, 0.0], 0.25, false),
    ([0.0, 0.2, -0.37], 0.15, false),
];

fn sdf(peca: usize, p: [f32; 3]) -> f32 {
    let (c, r, caixa) = PECAS[peca];
    let q: [f32; 3] = std::array::from_fn(|e| p[e] - c[e]);
    if caixa {
        let d = q.map(|x| x.abs() - r);
        let fora = d.map(|x| x.max(0.0));
        (fora[0] * fora[0] + fora[1] * fora[1] + fora[2] * fora[2]).sqrt()
            + d[0].max(d[1]).max(d[2]).min(0.0)
    } else {
        (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - r
    }
}

/// A grelha de cada peça, a partir de um volume de `48³` sobre a caixa dela (+ dois passos).
pub(crate) fn grades() -> Vec<Grade> {
    (0..PECAS.len())
        .map(|k| {
            let (c, r, caixa) = PECAS[k];
            let m = r * (1.0 + 4.0 / 47.0);
            let vol = Volume::de(c.map(|x| x - m), c.map(|x| x + m), 48, |pts| {
                pts.iter().map(|p| sdf(k, *p)).collect()
            });
            let bola = if caixa { r * 3.0f32.sqrt() } else { r };
            Grade::constroi(&vol, c, bola)
        })
        .collect()
}

pub(crate) struct Ponto {
    pub obj: usize,
    pub p: [f32; 3],
    pub n: [f32; 3],
    pub vis: f32,
}

pub(crate) fn oraculo() -> Vec<Ponto> {
    ORACULO
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .map(|l| {
            let c: Vec<f32> = l.split(',').map(|x| x.parse().expect("número")).collect();
            let n = [c[6], c[7], c[8]];
            let ln = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            Ponto {
                obj: c[2] as usize - 1,
                p: [c[3], c[4], c[5]],
                n: n.map(|x| x / ln),
                vis: c[9],
            }
        })
        .collect()
}

/// `(|Δ| médio, médio onde o Cycles < 0,9, máximo, viés)`.
fn erros(pontos: &[Ponto], est: impl Fn(&Ponto) -> f32) -> (f32, f32, f32, f32) {
    let (mut s, mut sp, mut np, mut m, mut v) = (0.0f32, 0.0f32, 0usize, 0.0f32, 0.0f32);
    for p in pontos {
        let e = est(p) - p.vis;
        s += e.abs();
        v += e;
        m = m.max(e.abs());
        if p.vis < 0.9 {
            sp += e.abs();
            np += 1;
        }
    }
    let n = pontos.len() as f32;
    (s / n, sp / np as f32, m, v / n)
}

/// ⭐⭐⭐ **A oclusão entre peças é a do Cycles** — nos pontos e normais que o próprio Cycles deu,
/// cada peça lê as grelhas das OUTRAS (o produto das visibilidades). Controlo: sem a lei, o erro é o
/// próprio escurecimento.
///
/// Medido (03/10, `11 242` pontos): `|Δ|` médio `0,0114`, onde o Cycles `< 0,9` `0,0160`, máximo
/// `0,175` (o mesmo de `512` raios exactos: é a silhueta), viés `−0,0027`; sem a lei `0,218` / `0,370`.
/// Antes de escolher (protótipo nos mesmos pontos): a oclusão de Quilez que a casa assa por vértice
/// erra `0,162` (clara demais); `9` cones de `α = 0,4` `0,018` a `~90` leituras por pixel; harmónicos
/// de ordem `1` `0,028`; os de ordem `2` exactos no ponto `0,0061`.
#[test]
fn a_oclusao_entre_pecas_e_a_do_cycles() {
    let cena = ORACULO
        .lines()
        .find(|l| l.starts_with("# CENA"))
        .expect("a linha da cena");
    assert!(
        cena.contains(
            "caixa=((0.0, 0.2, 0.0), 0.4) esferas=[('em_cima', (0.0, 0.6, 0.0), 0.2), \
             ('ao_lado', (0.45, 0.25, 0.0), 0.25), ('perto', (0.0, 0.2, -0.37), 0.15)]"
        ),
        "o CSV é de outra cena: {cena}"
    );
    let pontos = oraculo();
    let g: Vec<Grade> = grades().iter().map(Grade::em_f16).collect();
    let lei = |p: &Ponto| {
        (0..g.len())
            .filter(|&b| b != p.obj)
            .map(|b| 1.0 - g[b].oclusao(p.p, p.n))
            .product::<f32>()
    };
    let (medio, perto, maximo, vies) = erros(&pontos, lei);
    let (c_medio, c_perto, _, _) = erros(&pontos, |_| 1.0);
    eprintln!(
        "{} pontos · |Δ| médio {medio:.4} · perto {perto:.4} · máx {maximo:.3} · viés {vies:+.4} \
         (sem a lei: {c_medio:.4} · perto {c_perto:.4})",
        pontos.len()
    );
    assert!(
        c_perto > 0.2,
        "CONTROLO: a fixtura tem de ter contacto ({c_perto})"
    );
    assert!(
        medio < 0.013 && perto < 0.018 && maximo < 0.19 && vies.abs() < 0.005,
        "a lei afastou-se do Cycles: médio {medio} · perto {perto} · máx {maximo} · viés {vies}"
    );
}

const PROPRIA: &str = include_str!("../fixtures/oraculo_oclusao_propria.csv");

fn caixa(p: [f32; 3], c: [f32; 3], h: [f32; 3]) -> f32 {
    let d: [f32; 3] = std::array::from_fn(|e| (p[e] - c[e]).abs() - h[e]);
    let f = d.map(|x| x.max(0.0));
    (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt() + d[0].max(d[1]).max(d[2]).min(0.0)
}

/// As peças do oráculo da oclusão própria (o cabeçalho do script).
fn sdf_propria(peca: &str, p: [f32; 3]) -> f32 {
    match peca {
        "L" => caixa(p, [0.0, 0.1, 0.0], [0.3, 0.1, 0.2]).min(caixa(
            p,
            [-0.2, 0.35, 0.0],
            [0.1, 0.15, 0.2],
        )),
        "bola" => {
            let q = [p[0] - 0.05, p[1] - 0.3, p[2] - 0.05];
            caixa(p, [0.0, 0.15, 0.0], [0.3, 0.15, 0.3])
                .min((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt() - 0.18)
        }
        _ => {
            let q = [p[0], p[1] - 0.1, p[2]];
            let r = (q[0] * q[0] + q[2] * q[2]).sqrt() - 0.3;
            (r * r + q[1] * q[1]).sqrt() - 0.1
        }
    }
}

/// ⭐⭐⭐ **A oclusão própria é a do Cycles** — peças fundidas (um L de duas caixas, uma bola meio
/// enterrada numa caixa, um toro), cada uma sozinha, no volume `64³` da caixa dela, nos pontos e
/// normais do Cycles. Controlo: a oclusão de Quilez que a casa assava (`5` passos em `0,01..0,16`).
///
/// Medido (03/10, `48` cones de `0,2` rad, volume `64³`, `14 712` pontos), médio / onde o Cycles
/// `< 0,9`: L `0,012` / `0,016` · bola `0,006` / `0,008` · toro `0,011` / `0,031` (o toro sai um pouco
/// CLARO, viés `+0,013`: os cones moles subestimam um tubo fino do outro lado do furo — `64` cones dão
/// `0,029`). Quilez: `0,115` / `0,209` · `0,051` / `0,142` · `0,073` / `0,218`. ⛔ Recusado (medido): o limite inferior
/// de FORA da caixa do volume nos cones (escurecia os que saem rasantes, viés `−0,025`).
#[test]
fn a_oclusao_propria_e_a_do_cycles() {
    let mut por =
        std::collections::BTreeMap::<String, Vec<(usize, [f32; 3], [f32; 3], f32)>>::new();
    for (k, l) in PROPRIA
        .lines()
        .filter(|l| !l.starts_with('#'))
        .skip(1)
        .enumerate()
    {
        let c: Vec<&str> = l.split(',').collect();
        let f = |i: usize| c[i].parse::<f32>().expect("número");
        let n = [f(6), f(7), f(8)];
        let ln = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        por.entry(c[0].to_string()).or_default().push((
            k,
            [f(3), f(4), f(5)],
            n.map(|x| x / ln),
            f(9),
        ));
    }
    let caixas = [
        ("L", [-0.32, -0.02, -0.22], [0.32, 0.52, 0.22]),
        ("bola", [-0.32, -0.02, -0.32], [0.32, 0.5, 0.32]),
        ("toro", [-0.42, -0.02, -0.42], [0.42, 0.22, 0.42]),
    ];
    let mut falhas = Vec::new();
    for (peca, lo, hi) in caixas {
        let pts = &por[peca];
        let vol = Volume::de(lo, hi, 64, |q| {
            q.iter().map(|p| sdf_propria(peca, *p)).collect()
        });
        let pos: Vec<[f32; 3]> = pts.iter().map(|p| p.1).collect();
        let nrm: Vec<[f32; 3]> = pts.iter().map(|p| p.2).collect();
        let diag = (0..3).map(|e| (hi[e] - lo[e]).powi(2)).sum::<f32>().sqrt();
        let lei = crate::visibilidade_propria(&vol, &pos, &nrm, diag);
        let quilez: Vec<f32> = pts
            .iter()
            .map(|(_, p, n, _)| {
                let (mut occ, mut w) = (0.0, 0.0);
                for (i, h) in [0.01f32, 0.02, 0.04, 0.08, 0.16].iter().enumerate() {
                    let q = [0, 1, 2].map(|e| p[e] + n[e] * h);
                    let pe = 0.5f32.powi(i as i32);
                    occ += pe * ((h - sdf_propria(peca, q)) / h).clamp(0.0, 1.0);
                    w += pe;
                }
                1.0 - occ / w
            })
            .collect();
        let mede = |est: &[f32]| {
            let (mut s, mut sp, mut np, mut m) = (0.0f32, 0.0f32, 0usize, 0.0f32);
            for (e, p) in est.iter().zip(pts) {
                let d = (e - p.3).abs();
                s += d;
                m = m.max(d);
                if p.3 < 0.9 {
                    sp += d;
                    np += 1;
                }
            }
            (s / pts.len() as f32, sp / np.max(1) as f32, m)
        };
        let ((m, p, x), (qm, qp, _)) = (mede(&lei), mede(&quilez));
        eprintln!(
            "{peca}: {} pontos · |Δ| médio {m:.4} · perto {p:.4} · máx {x:.3} (Quilez {qm:.4} · perto {qp:.4})",
            pts.len()
        );
        if !(m < 0.02 && p < 0.035 && qp > 4.0 * p) {
            falhas.push(format!(
                "{peca}: médio {m:.4} · perto {p:.4} · máx {x:.3} · Quilez perto {qp:.4}"
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "a oclusão própria afastou-se do Cycles: {falhas:?}"
    );
}
