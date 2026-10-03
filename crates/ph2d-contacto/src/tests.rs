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
