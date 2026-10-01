//! ⭐⭐ **A ILHA QUE A BOLA FECHA** (F44, report do dono de 2026-10-01 com três fotos: *«quando uma
//! parte do membro se sobrepõe a outra formando uma ilha, nessa ilha as quinas ainda não estão
//! corretas»*).
//!
//! Quando um membro da pele se fecha sobre outro, a união deixa um BURACO — a ilha. A silhueta é o
//! FECHO da forma por uma bola de raio `r` ([`crate::bola`]), e o fecho de um buraco é a bola a rolar
//! por DENTRO dele ([`crate::bola::rola_a_bola_por_dentro`]). Este módulo responde à pergunta que
//! vem antes: **a bola CABE dentro da ilha?** Se não cabe em sítio nenhum, o fecho enche a ilha
//! inteira — ela desaparece, exactamente como um vinco mais estreito que a bola é cheio pelo arco
//! dela no contorno de fora.
//!
//! ⚠️ Medido na barra da cena (o braço dobrado em C): as ilhas com raio inscrito de `0,40 r` a
//! `0,79 r` ficavam com três bicos de `128°` depois de rolar a bola — ali NENHUMA bola vazia existe,
//! e a procura não tinha onde pousar. Elas são o caso de uma ilha a NASCER (o membro acabou de se
//! cruzar) ou a MORRER (a sobreposição a engoli-la).

use kurbo::{CubicBez, ParamCurve, Point};
use ph2d_vec_scene::VecVertex;

/// Amostras por segmento do contorno — `32`, as mesmas da bola.
const AMOSTRAS: usize = 32;

/// Pontos por lado da grelha onde se procura o centro de uma bola que caiba — `64`. Numa ilha à
/// beira de fechar (raio inscrito `~r`, largura `~3 r`) a célula vale `~0,05 r`, e o erro de pôr o
/// centro na célula errada é meia diagonal dela: `~0,03 r`.
const GRELHA: usize = 64;

fn pt(a: [f64; 2]) -> Point {
    Point::new(a[0], a[1])
}

/// O contorno fechado `verts` amostrado como polígono.
fn poligono(verts: &[VecVertex]) -> Vec<Point> {
    let n = verts.len();
    let mut out = Vec::with_capacity(n * AMOSTRAS);
    for j in 0..n {
        let (c, q) = (&verts[j], &verts[(j + 1) % n]);
        let cub = CubicBez::new(
            pt(c.anchor),
            pt(c.out_handle),
            pt(q.in_handle),
            pt(q.anchor),
        );
        for k in 0..AMOSTRAS {
            #[allow(clippy::cast_precision_loss)]
            out.push(cub.eval(k as f64 / AMOSTRAS as f64));
        }
    }
    out
}

/// `p` está dentro do polígono `poli`? — paridade de um raio horizontal.
fn dentro(poli: &[Point], p: Point) -> bool {
    let m = poli.len();
    let mut d = false;
    for i in 0..m {
        let (a, b) = (poli[i], poli[(i + 1) % m]);
        if (a.y > p.y) != (b.y > p.y) {
            let x = a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x);
            if p.x < x {
                d = !d;
            }
        }
    }
    d
}

/// A distância de `p` ao polígono `poli` — menor que `limite` devolve cedo.
fn longe_de_tudo(poli: &[Point], p: Point, limite: f64) -> bool {
    let m = poli.len();
    (0..m).all(|i| {
        let (a, b) = (poli[i], poli[(i + 1) % m]);
        let ab = b - a;
        let l2 = ab.hypot2();
        let t = if l2 > 0.0 {
            ((p - a).dot(ab) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (p - (a + ab * t)).hypot() >= limite
    })
}

/// ⭐⭐ **Uma bola de raio `raio` cabe DENTRO do contorno fechado `verts`?** — algum ponto de dentro
/// fica a pelo menos `raio` de todo o contorno. Um contorno degenerado (menos de três nós, raio
/// não finito) responde `true`: quem pergunta deixa-o como estava.
#[must_use]
pub fn a_bola_cabe_dentro(verts: &[VecVertex], raio: f64) -> bool {
    if verts.len() < 3 || !raio.is_finite() || raio <= 0.0 {
        return true;
    }
    let poli = poligono(verts);
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in &poli {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    // Uma caixa mais estreita que a bola não a contém.
    if x1 - x0 < 2.0 * raio || y1 - y0 < 2.0 * raio {
        return false;
    }
    // ⚠️ Do CENTRO para fora: numa ilha folgada o primeiro ponto já responde.
    let (cx, cy) = (0.5 * (x0 + x1), 0.5 * (y0 + y1));
    let mut pontos: Vec<Point> = (0..GRELHA)
        .flat_map(|i| {
            (0..GRELHA).map(move |k| {
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = (
                    (i as f64 + 0.5) / GRELHA as f64,
                    (k as f64 + 0.5) / GRELHA as f64,
                );
                Point::new(x0 + u * (x1 - x0), y0 + v * (y1 - y0))
            })
        })
        .collect();
    pontos.sort_by(|a, b| ((a.x - cx).hypot(a.y - cy)).total_cmp(&(b.x - cx).hypot(b.y - cy)));
    pontos
        .iter()
        .any(|&p| dentro(&poli, p) && longe_de_tudo(&poli, p, raio))
}

#[cfg(test)]
#[path = "ilha_tests.rs"]
mod tests;
