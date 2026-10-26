//! As cenas NOSSAS da W4 (`docs/BlenderConstraints/fixtures/w4_cenas`, geradas pela bancada) traduzidas para paths
//! e regras por `(path, VertId)`, e a régua de cumprimento da bancada (`bancada_solvers/src/main.rs` `violacao`),
//! escrita aqui à parte do solver de propósito: quem mede não é quem resolve.

use ph2d_vec_constraints::{Folha, Solucao, orienta_tangente};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};
use ph2d_vec_scene::{VecPath, VecPathId, VecVertex, Xform, ellipse};
use serde_json::Value;

pub const TOL_PX: f64 = 1e-4;
const PATH: VecPathId = 1;
const CIRCULO0: VecPathId = 10;

pub fn le(nome: &str) -> Value {
    let p = format!(
        "{}/../../docs/BlenderConstraints/fixtures/w4_cenas/{nome}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&p).expect(&p)).expect("a cena é JSON")
}

pub fn xy(v: &Value) -> [f64; 2] {
    [v[0].as_f64().expect("x"), v[1].as_f64().expect("y")]
}

fn u(v: &Value, k: &str) -> usize {
    usize::try_from(v[k].as_u64().expect(k)).expect(k)
}

/// Uma cena da bancada: os paths (um com os pontos soltos, um por círculo), o endereço de cada índice e as regras.
pub struct Cena {
    pub json: Value,
    pub paths: Vec<VecPath>,
    pub ponto: Vec<PontoCad>,
    pub regras: Vec<RegraCad>,
    pub raios: Vec<f64>,
    pub circulo_de: Vec<usize>,
}

impl Cena {
    pub fn folha(&self) -> Folha<'_> {
        Folha::new(self.paths.iter().map(|p| (p, Xform::IDENTITY)))
    }

    /// `partida` ausente = parte de `pontos` (a convenção da bancada).
    pub fn pose(&self, chave: &str) -> Vec<[f64; 2]> {
        let v = if self.json[chave].is_null() {
            &self.json["pontos"]
        } else {
            &self.json[chave]
        };
        v.as_array().expect(chave).iter().map(xy).collect()
    }

    /// Os paths na pose dada (índice → posição), ids intactos.
    pub fn na_pose(&self, pose: &[[f64; 2]]) -> Vec<VecPath> {
        constroi_paths(pose, &self.circulo_de, &self.raios)
    }

    pub fn posicoes(&self, s: &Solucao) -> Vec<[f64; 2]> {
        self.ponto
            .iter()
            .map(|p| s.ponto(*p).expect("todo ponto da cena é variável"))
            .collect()
    }
}

fn constroi_paths(pose: &[[f64; 2]], circulo_de: &[usize], raios: &[f64]) -> Vec<VecPath> {
    let mut solto = VecPath {
        id: PATH,
        ..VecPath::default()
    };
    for (i, p) in pose.iter().enumerate() {
        if !circulo_de.contains(&i) {
            solto.verts.push(VecVertex::corner(*p));
        }
    }
    solto.normaliza_ids();
    let mut paths = vec![solto];
    for (k, (&c, &r)) in circulo_de.iter().zip(raios).enumerate() {
        let mut p = ellipse(pose[c], r, r);
        p.id = CIRCULO0 + k as u64;
        p.normaliza_ids();
        paths.push(p);
    }
    paths
}

pub fn cena(nome: &str) -> Cena {
    let json = le(nome);
    let pontos: Vec<[f64; 2]> = json["pontos"]
        .as_array()
        .expect("pontos")
        .iter()
        .map(xy)
        .collect();
    let circs = json["circulos"].as_array().expect("circulos");
    let circulo_de: Vec<usize> = circs.iter().map(|c| u(c, "centro")).collect();
    let raios: Vec<f64> = circs
        .iter()
        .map(|c| c["raio"].as_f64().expect("raio"))
        .collect();
    let paths = constroi_paths(&pontos, &circulo_de, &raios);
    let mut ponto = Vec::with_capacity(pontos.len());
    let mut solto = paths[0].verts.iter();
    for i in 0..pontos.len() {
        ponto.push(match circulo_de.iter().position(|&c| c == i) {
            Some(k) => PontoCad::Centro(CIRCULO0 + k as u64),
            None => PontoCad::Vert(PATH, solto.next().expect("vértice").id),
        });
    }
    let folha = Folha::new(paths.iter().map(|p| (p, Xform::IDENTITY)));
    let seg = |a: usize, b: usize| SegCad(ponto[a], ponto[b]);
    let mut regras: Vec<RegraCad> = Vec::new();
    for r in json["regras"].as_array().expect("regras") {
        let n = |k: &str| u(r, k);
        let lei = match r["t"].as_str().expect("t") {
            "h" => Lei::Horizontal(seg(n("a"), n("b"))),
            "v" => Lei::Vertical(seg(n("a"), n("b"))),
            "paralelo" => Lei::Paralelo(seg(n("a"), n("b")), seg(n("c"), n("d"))),
            "perpendicular" => Lei::Perpendicular(seg(n("a"), n("b")), seg(n("c"), n("d"))),
            "distancia" => {
                Lei::Distancia(ponto[n("a")], ponto[n("b")], r["d"].as_f64().expect("d"))
            }
            "igual" => Lei::Igual(seg(n("a"), n("b")), seg(n("c"), n("d"))),
            "sobre_linha" => Lei::SobreRecta(ponto[n("p")], seg(n("a"), n("b"))),
            "simetrico" => Lei::Simetrico(ponto[n("p")], ponto[n("q")], seg(n("a"), n("b"))),
            "tangente" => {
                let c = CIRCULO0 + n("circulo") as u64;
                Lei::Tangente(
                    orienta_tangente(&folha, seg(n("a"), n("b")), c)
                        .expect("a tangente cumpre na pose"),
                    c,
                )
            }
            "coincidente" => Lei::Coincidente(ponto[n("a")], ponto[n("b")]),
            t => panic!("regra {t} sem tradução"),
        };
        regras.push(RegraCad::nova(lei));
    }
    for f in json["fixos"].as_array().expect("fixos") {
        let i = usize::try_from(f.as_u64().expect("fixo")).expect("fixo");
        regras.push(RegraCad::nova(Lei::Fixo(ponto[i], pontos[i])));
    }
    for (k, &r) in raios.iter().enumerate() {
        regras.push(RegraCad::nova(Lei::Raio(CIRCULO0 + k as u64, r)));
    }
    Cena {
        json,
        paths,
        ponto,
        regras,
        raios,
        circulo_de,
    }
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
fn sub(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn cruz(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[1] - a[1] * b[0]
}
fn escalar(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn dist_ponto_recta(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    cruz(sub(b, a), sub(p, a)).abs() / dist(a, b)
}

/// A régua da bancada: violação de cada regra da cena em px (ângulos: desvio ao longo do 2.º segmento).
pub fn violacao(c: &Cena, x: &[[f64; 2]], r: &Value) -> f64 {
    let n = |k: &str| u(r, k);
    match r["t"].as_str().expect("t") {
        "h" => (x[n("a")][1] - x[n("b")][1]).abs(),
        "v" => (x[n("a")][0] - x[n("b")][0]).abs(),
        "paralelo" => {
            cruz(sub(x[n("b")], x[n("a")]), sub(x[n("d")], x[n("c")])).abs()
                / dist(x[n("c")], x[n("d")])
        }
        "perpendicular" => {
            escalar(sub(x[n("b")], x[n("a")]), sub(x[n("d")], x[n("c")])).abs()
                / dist(x[n("c")], x[n("d")])
        }
        "distancia" => (dist(x[n("a")], x[n("b")]) - r["d"].as_f64().expect("d")).abs(),
        "igual" => (dist(x[n("a")], x[n("b")]) - dist(x[n("c")], x[n("d")])).abs(),
        "sobre_linha" => dist_ponto_recta(x[n("p")], x[n("a")], x[n("b")]),
        "simetrico" => {
            let (a, b, p, q) = (x[n("a")], x[n("b")], x[n("p")], x[n("q")]);
            let (d, l) = (sub(b, a), dist(a, b));
            let t = escalar(sub(p, a), d) / (l * l);
            let pe = [a[0] + t * d[0], a[1] + t * d[1]];
            dist([2.0 * pe[0] - p[0], 2.0 * pe[1] - p[1]], q)
        }
        "tangente" => {
            let k = n("circulo");
            (dist_ponto_recta(x[c.circulo_de[k]], x[n("a")], x[n("b")]) - c.raios[k]).abs()
        }
        "coincidente" => dist(x[n("a")], x[n("b")]),
        t => panic!("regra {t} sem régua"),
    }
}

/// A pior violação na pose `x`: regras, pontos fixos fora do sítio e o arrastado fora do alvo.
pub fn pior(c: &Cena, x: &[[f64; 2]], alvo: Option<(usize, [f64; 2])>) -> f64 {
    let pontos = c.pose("pontos");
    let fixos = c.json["fixos"]
        .as_array()
        .expect("fixos")
        .iter()
        .map(|f| usize::try_from(f.as_u64().expect("fixo")).expect("fixo"))
        .map(|i| dist(x[i], pontos[i]))
        .fold(0.0, f64::max);
    let arr = alvo.map_or(0.0, |(i, a)| dist(x[i], a));
    c.json["regras"]
        .as_array()
        .expect("regras")
        .iter()
        .map(|r| violacao(c, x, r))
        .fold(fixos.max(arr), f64::max)
}
