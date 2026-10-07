//! O RECUO pelo raio do agente: cada obstáculo cresce, a região encolhe (plano 30 §2.3).
//!
//! # Redondo, não em esquadria — e a divergência é DECLARADA
//!
//! A folga exacta que um disco de raio `r` precisa à volta de um obstáculo é a soma de Minkowski com
//! o disco: lados recuados `r`, cantos ARREDONDADOS. O Godot recua em esquadria (medido na sonda da
//! pesquisa: o furo `(150,100)–(250,200)` a raio `10` sai com as quinas VIVAS em `(140,90)–(260,210)`)
//! — mais folga do que o disco precisa, e o caminho dá a volta mais larga. ⇒ [`Corner::Round`] é o
//! produto; [`Corner::Miter`] existe para a PARIDADE com o oráculo (com ele, a nossa construção tem de
//! dar a malha do Godot).
//!
//! O disco vira o polígono regular de `n` lados que o CIRCUNSCREVE ⇒ o obstáculo recuado CONTÉM a
//! soma verdadeira (nunca falta folga), e o excesso nos cantos é `r · (1/cos(π/n) − 1)` — `0,48 %` do
//! raio a `n = 32`, a escolha medida na W1.

use ph2d_nav::V2;
use ph2d_nav::geom::{dot, len, scale, sub};

use crate::lattice::{P, SCALE, disk_dirs, hull, to_lattice};

/// A forma de um obstáculo, em metros, no mundo.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// Um polígono CONVEXO (qualquer sentido) — uma caixa rodada, um polígono convexo de colisor.
    Convex(Vec<V2>),
    /// Um círculo.
    Circle { center: V2, radius: f64 },
    /// Uma cápsula: o segmento `a → b` engordado por `radius`.
    Capsule { a: V2, b: V2, radius: f64 },
}

/// Como os cantos de um obstáculo recuam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Corner {
    /// A soma de Minkowski com o disco — o produto.
    Round,
    /// As rectas recuadas a encontrarem-se (limite de esquadria `2`) — a paridade com o Godot.
    Miter,
}

/// O obstáculo recuado, como um anel CONVEXO da grelha (anti-horário). Vazio se a forma degenera.
pub fn inflate(shape: &Shape, r: f64, corner: Corner, n: u32) -> Vec<P> {
    let (dirs, sec) = disk_dirs(n);
    // ⭐ Uma unidade da grelha de margem: o arredondamento à grelha pode encolher meio passo, e o
    // recuo promete CONTER a soma verdadeira.
    let margin = 1.0 / SCALE;
    match shape {
        Shape::Circle { center, radius } => {
            sum_with_disk(&[*center], radius + r, &dirs, sec, margin)
        }
        Shape::Capsule { a, b, radius } => sum_with_disk(&[*a, *b], radius + r, &dirs, sec, margin),
        Shape::Convex(pts) => {
            if r <= 0.0 {
                return hull(pts.iter().map(|&p| to_lattice(p)).collect());
            }
            match corner {
                Corner::Round => sum_with_disk(pts, r, &dirs, sec, margin),
                Corner::Miter => miter(pts, r),
            }
        }
    }
}

/// ⭐ (W18) Uma ÁREA de custo pelo raio do agente: para FORA ([`inflate`]) ou, uma mais barata que o chão,
/// para DENTRO ([`erode`]) — a lei de [`crate::Area`].
pub fn recua(area: &crate::Area, r: f64, corner: Corner, n: u32) -> Vec<P> {
    if area.dentro {
        erode(&area.shape, r, n)
    } else {
        inflate(&area.shape, r, corner, n)
    }
}

/// ⭐ (W19, plano 30 §28.3) Uma área que recua para DENTRO e NÃO SOBRA na malha deste raio — nenhum corpo
/// cabe inteiro nela, logo ela não muda caminho nenhum (um controlo morto para quem tem este raio). A mesma
/// erosão, o mesmo raio e os mesmos lados que a construção ([`recua`]).
#[must_use]
pub fn some_na_malha(area: &crate::Area, params: &crate::Params) -> bool {
    area.dentro
        && erode(
            &area.shape,
            params.agent_radius.max(0.0),
            params.lados_do_disco(),
        )
        .is_empty()
}

/// ⭐ (W18) A forma ENCOLHIDA pelo raio: onde o CENTRO de um disco de raio `r` o deixa inteiro dentro dela.
/// CONTIDA na verdadeira (o polígono do disco é INSCRITO, e uma unidade da grelha de margem) — o desconto
/// de uma área mais barata nunca vale com o corpo a sair dela. Vazio se nenhum corpo cabe.
pub fn erode(shape: &Shape, r: f64, n: u32) -> Vec<P> {
    let (dirs, _) = disk_dirs(n);
    let margin = 1.0 / SCALE;
    let inscrito = |pts: &[V2], raio: f64| {
        if raio <= margin {
            return Vec::new();
        }
        let k = raio - margin;
        let anel: Vec<P> = pts
            .iter()
            .flat_map(|&p| {
                dirs.iter()
                    .map(move |&d| to_lattice([p[0] + d[0] * k, p[1] + d[1] * k]))
            })
            .collect();
        hull(anel)
    };
    let anel = match shape {
        Shape::Circle { center, radius } => inscrito(&[*center], radius - r),
        Shape::Capsule { a, b, radius } => inscrito(&[*a, *b], radius - r),
        Shape::Convex(pts) => inset_region(pts, r.max(0.0) + margin),
    };
    if anel.len() < 3 { Vec::new() } else { anel }
}

/// O fecho de `{pᵢ + R·uₖ}`: a soma de Minkowski de um convexo com o polígono circunscrito.
fn sum_with_disk(pts: &[V2], r: f64, dirs: &[V2], sec: f64, margin: f64) -> Vec<P> {
    if r <= 0.0 {
        return hull(pts.iter().map(|&p| to_lattice(p)).collect());
    }
    let big = r * sec + margin;
    let mut all = Vec::with_capacity(pts.len() * dirs.len());
    for &p in pts {
        for &d in dirs {
            all.push(to_lattice([p[0] + d[0] * big, p[1] + d[1] * big]));
        }
    }
    hull(all)
}

/// O recuo em esquadria de um convexo: cada aresta sobe `r` pela normal de fora e as vizinhas
/// encontram-se; acima do limite `2·r` o canto é chanfrado (as duas pontas recuadas).
fn miter(pts: &[V2], r: f64) -> Vec<P> {
    let ring = ccw(pts);
    let n = ring.len();
    if n < 3 {
        return hull(ring.iter().map(|&p| to_lattice(p)).collect());
    }
    // A normal de FORA de cada aresta (anti-horário ⇒ fora é a direita).
    let normals: Vec<V2> = (0..n)
        .map(|i| {
            let d = sub(ring[(i + 1) % n], ring[i]);
            let l = len(d);
            if l <= 0.0 {
                [0.0, 0.0]
            } else {
                [d[1] / l, -d[0] / l]
            }
        })
        .collect();
    let mut out = Vec::with_capacity(2 * n);
    for i in 0..n {
        let n0 = normals[(i + n - 1) % n];
        let n1 = normals[i];
        let p = ring[i];
        // O ponto da esquadria: p + r·(n0 + n1)/(1 + n0·n1).
        let k = 1.0 + dot(n0, n1);
        let m = if k > 1e-12 {
            scale([n0[0] + n1[0], n0[1] + n1[1]], r / k)
        } else {
            [0.0, 0.0]
        };
        if k > 1e-12 && len(m) <= 2.0 * r {
            out.push(to_lattice([p[0] + m[0], p[1] + m[1]]));
        } else {
            out.push(to_lattice([p[0] + n0[0] * r, p[1] + n0[1] * r]));
            out.push(to_lattice([p[0] + n1[0] * r, p[1] + n1[1] * r]));
        }
    }
    hull(out)
}

/// A região andável ENCOLHIDA `r`: a erosão de um convexo pelo disco é a intersecção dos semi-planos
/// recuados — sem arcos (os cantos de uma região convexa ficam vivos, e é a resposta exacta).
pub fn inset_region(region: &[V2], r: f64) -> Vec<P> {
    let mut poly = ccw(region);
    if r > 0.0 {
        let base = poly.clone();
        let n = base.len();
        for i in 0..n {
            let a = base[i];
            let b = base[(i + 1) % n];
            let d = sub(b, a);
            let l = len(d);
            if l <= 0.0 {
                continue;
            }
            // A normal de DENTRO (anti-horário ⇒ esquerda).
            let normal_dentro = [-d[1] / l, d[0] / l];
            poly = clip_half_plane(&poly, a, normal_dentro, r);
            if poly.is_empty() {
                return Vec::new();
            }
        }
    }
    hull(poly.iter().map(|&p| to_lattice(p)).collect())
}

/// Sutherland–Hodgman: fica com `dot(p − a, n) ≥ r`.
fn clip_half_plane(poly: &[V2], a: V2, n: V2, r: f64) -> Vec<V2> {
    let f = |p: V2| dot(sub(p, a), n) - r;
    let m = poly.len();
    let mut out = Vec::with_capacity(m + 1);
    for i in 0..m {
        let p = poly[i];
        let q = poly[(i + 1) % m];
        let (fp, fq) = (f(p), f(q));
        if fp >= 0.0 {
            out.push(p);
        }
        if (fp >= 0.0) != (fq >= 0.0) {
            let t = fp / (fp - fq);
            out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]);
        }
    }
    out
}

/// O mesmo anel, em sentido anti-horário.
fn ccw(pts: &[V2]) -> Vec<V2> {
    let n = pts.len();
    let mut a = 0.0;
    for i in 0..n {
        let p = pts[i];
        let q = pts[(i + 1) % n];
        a += p[0] * q[1] - q[0] * p[1];
    }
    let mut v = pts.to_vec();
    if a < 0.0 {
        v.reverse();
    }
    v
}
