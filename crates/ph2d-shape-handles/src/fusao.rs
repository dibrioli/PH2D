//! **«Merge by distance»** dos pontos do «Outline» (idioma do Blender; dono 09/11, «fundir pontos por proximidade»).
//!
//! Em cada anel, andando na ordem (com a volta do fim ao começo), um grupo cresce enquanto o ponto seguinte está a
//! no máximo `d` do PRIMEIRO do grupo — nunca em cadeia, que comeria um arco denso inteiro. Cada grupo de 2 ou mais
//! vira um ponto na média. Com seleção, só pontos selecionados entram num grupo. O resultado tem de ser
//! [`validos`]: se fundir tudo de uma vez não for, cada grupo entra só se o anel continuar válido (`retidos` conta os
//! que ficaram). Medição do padrão: `ph2d-physics-scene`, `a_distancia_padrao_do_merge_limpa_os_cachos_da_rock_sem_comer_as_reentrancias`.

use crate::pontos::{Ponto, validos};
use ph2d_physics_ecs::OutlineRing as Ring;

/// A distância por omissão, em px da imagem (medida na Rock em Detail 2: 1,5–4 px tiram só o cacho; 5 px já come reentrâncias).
pub const DISTANCIA_PADRAO_PX: f32 = 3.0; // LITERAL-PX-OK: px da imagem, medido na Rock

/// O que a fusão fez.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fusao {
    pub rings: Vec<Ring>,
    /// Quantos pontos deixaram de existir.
    pub fundidos: usize,
    /// Grupos que ficaram por fundir para o anel continuar válido.
    pub retidos: usize,
    /// A seleção nos anéis novos: um ponto fundido de pontos selecionados continua selecionado.
    pub selecionados: Vec<Ponto>,
}

fn perto(a: [i32; 2], b: [i32; 2], d2: i64) -> bool {
    let (x, y) = (i64::from(a[0] - b[0]), i64::from(a[1] - b[1]));
    x * x + y * y <= d2
}

/// Os grupos de um anel, em ordem a partir de uma fronteira (os índices de cada grupo seguidos, com a volta).
fn grupos(r: &Ring, entra: &dyn Fn(usize) -> bool, d2: i64) -> Vec<Vec<usize>> {
    let n = r.len();
    let liga = |a: usize, b: usize| entra(a) && entra(b) && perto(r[a], r[b], d2);
    let s = (0..n).find(|&i| !liga((i + n - 1) % n, i)).unwrap_or(0);
    let mut out: Vec<Vec<usize>> = Vec::new();
    for j in 0..n {
        let i = (s + j) % n;
        match out.last_mut() {
            Some(g) if liga(g[0], i) => g.push(i),
            _ => out.push(vec![i]),
        }
    }
    out
}

fn media(r: &Ring, g: &[usize]) -> [i32; 2] {
    let n = g.len() as f64;
    let s = g.iter().fold([0.0f64; 2], |a, &i| {
        [a[0] + f64::from(r[i][0]), a[1] + f64::from(r[i][1])]
    });
    [(s[0] / n).round() as i32, (s[1] / n).round() as i32]
}

/// Os anéis com os grupos `aceites[k][g]` fundidos, e a seleção remapeada.
fn monta(
    rings: &[Ring],
    gs: &[Vec<Vec<usize>>],
    aceites: &[Vec<bool>],
    sel: &[Ponto],
) -> (Vec<Ring>, Vec<Ponto>) {
    let mut novos = Vec::with_capacity(rings.len());
    let mut nova_sel = Vec::new();
    for (k, r) in rings.iter().enumerate() {
        let mut v: Ring = Vec::with_capacity(r.len());
        let marcado = |i: usize| sel.contains(&Ponto { anel: k, i });
        for (g, sim) in gs[k].iter().zip(&aceites[k]) {
            if *sim {
                if g.iter().any(|&i| marcado(i)) {
                    nova_sel.push(Ponto {
                        anel: k,
                        i: v.len(),
                    });
                }
                v.push(media(r, g));
            } else {
                for &i in g {
                    if marcado(i) {
                        nova_sel.push(Ponto {
                            anel: k,
                            i: v.len(),
                        });
                    }
                    v.push(r[i]);
                }
            }
        }
        novos.push(v);
    }
    (novos, nova_sel)
}

/// **Funde os pontos a no máximo `d_q4`** (unidades Q4 da imagem: px × 16). Sem seleção, todos os pontos entram.
/// `None` = nenhum grupo de 2 ou mais (nada a fundir).
#[must_use]
pub fn fundidos(rings: &[Ring], sel: &[Ponto], d_q4: f32) -> Option<Fusao> {
    let d = f64::from(d_q4.max(0.0));
    let d2 = (d * d).floor() as i64;
    let gs: Vec<Vec<Vec<usize>>> = rings
        .iter()
        .enumerate()
        .map(|(k, r)| {
            let entra = |i: usize| sel.is_empty() || sel.contains(&Ponto { anel: k, i });
            grupos(r, &entra, d2)
        })
        .collect();
    let multi = |gs: &[Vec<Vec<usize>>]| -> Vec<Vec<bool>> {
        gs.iter()
            .map(|a| a.iter().map(|g| g.len() > 1).collect())
            .collect()
    };
    let todos = multi(&gs);
    let candidatos = todos.iter().flatten().filter(|b| **b).count();
    if candidatos == 0 {
        return None;
    }
    let antes: usize = rings.iter().map(Vec::len).sum();
    let (r, s) = monta(rings, &gs, &todos, sel);
    let (r, s, retidos) = if validos(rings, &r) {
        (r, s, 0)
    } else {
        // Um grupo de cada vez: entra só se tudo continuar válido.
        let mut aceites: Vec<Vec<bool>> = gs.iter().map(|a| vec![false; a.len()]).collect();
        let mut retidos = 0;
        for (k, a) in todos.iter().enumerate() {
            for (g, &e) in a.iter().enumerate() {
                if !e {
                    continue;
                }
                aceites[k][g] = true;
                if !validos(rings, &monta(rings, &gs, &aceites, sel).0) {
                    aceites[k][g] = false;
                    retidos += 1;
                }
            }
        }
        let (r, s) = monta(rings, &gs, &aceites, sel);
        (r, s, retidos)
    };
    let depois: usize = r.iter().map(Vec::len).sum();
    Some(Fusao {
        rings: r,
        fundidos: antes - depois,
        retidos,
        selecionados: s,
    })
}

#[cfg(test)]
#[path = "fusao_tests.rs"]
mod tests;
