//! A JUNÇÃO da malha por blocos: só passagens lineares sobre o que as camadas guardaram (numerar os
//! vértices, traduzir os anéis e as ligações, paredes e cantos, vértice→polígonos, ilhas pela união
//! das componentes, a caixa e as grelhas dos blocos).

use std::collections::BTreeMap;

use super::{Chave, Lig, MalhaPorBlocos, Vref, vizinho};
use crate::geom::V2;
use crate::grelha::{GrelhaDoBloco, Localizador};
use crate::mesh::{MeshError, NavMesh, polys_de_cada_vertice};

fn raiz(pai: &mut [u32], mut c: u32) -> u32 {
    while pai[c as usize] != c {
        pai[c as usize] = pai[pai[c as usize] as usize];
        c = pai[c as usize];
    }
    c
}

pub(super) fn junta(m: &MalhaPorBlocos) -> Result<NavMesh, MeshError> {
    let ordem: Vec<(Chave, &super::Bloco)> = m.blocos.iter().map(|(&k, b)| (k, b)).collect();
    let n = ordem.len();
    let idx: BTreeMap<Chave, usize> = ordem.iter().enumerate().map(|(i, (k, _))| (*k, i)).collect();
    let viz: Vec<[usize; 9]> = ordem
        .iter()
        .map(|(k, _)| std::array::from_fn(|d| idx.get(&vizinho(*k, d as u8)).copied().unwrap_or(usize::MAX)))
        .collect();
    let (mut mbase, mut pbase, mut cbase) = (vec![0usize; n + 1], vec![0u32; n + 1], vec![0u32; n + 1]);
    for (b, (_, bl)) in ordem.iter().enumerate() {
        let np = bl.peca.ring_off.len() - 1;
        if bl.peca.area.len() != np {
            return Err(MeshError::AreaCount {
                polys: np,
                areas: bl.peca.area.len(),
            });
        }
        mbase[b + 1] = mbase[b] + bl.peca.verts.len();
        pbase[b + 1] = pbase[b] + np as u32;
        cbase[b + 1] = cbase[b] + bl.l2.ncomp;
    }

    // Os vértices: cada um no 1.º bloco que o tem; os outros apontam para lá (já numerado).
    let mut mapa = vec![u32::MAX; mbase[n]];
    let mut verts: Vec<V2> = Vec::with_capacity(mbase[n]);
    for (b, (_, bl)) in ordem.iter().enumerate() {
        for (i, r) in bl.l2.slots[..bl.peca.verts.len()].iter().enumerate() {
            let o = viz[b][r.dir() as usize];
            let j = r.em(&ordem[o].1.l1);
            mapa[mbase[b] + i] = if o == b && j == i as u32 {
                verts.push(bl.peca.verts[i]);
                (verts.len() - 1) as u32
            } else {
                mapa[mbase[o] + j as usize]
            };
        }
    }
    let gv = |b: usize, r: Vref| {
        let o = viz[b][r.dir() as usize];
        mapa[mbase[o] + r.em(&ordem[o].1.l1) as usize]
    };
    for (b, (_, bl)) in ordem.iter().enumerate() {
        if let Some(e) = &bl.l2.erro {
            let nl = bl.peca.verts.len();
            let slot = |s: u32| {
                if (s as usize) < nl {
                    mapa[mbase[b] + s as usize]
                } else {
                    gv(b, bl.l2.slots[s as usize])
                }
            };
            return Err(match *e {
                MeshError::NonManifold { a, b } => MeshError::NonManifold {
                    a: slot(a),
                    b: slot(b),
                },
                _ => global(e, pbase[b]),
            });
        }
    }

    // Os anéis, a vizinhança, as paredes e os cantos (pela ordem dos polígonos e das arestas, a de
    // `from_rings`), as áreas — e a união das componentes pelas costuras.
    let nv = verts.len();
    let mut corner = vec![false; nv];
    let mut walls = Vec::new();
    let (tr, tp) = (
        ordem.iter().map(|(_, bl)| bl.l2.s_ring.len()).sum::<usize>(),
        pbase[n] as usize,
    );
    let mut ring_off: Vec<u32> = Vec::with_capacity(tp + 1);
    ring_off.push(0);
    let mut ring: Vec<u32> = Vec::with_capacity(tr);
    let mut nbrs: Vec<Option<u32>> = Vec::with_capacity(tr);
    let mut twin: Vec<u32> = Vec::with_capacity(tr);
    let mut area: Vec<u16> = Vec::with_capacity(tp);
    let mut pai: Vec<u32> = (0..cbase[n]).collect();
    let mut gslot: Vec<u32> = Vec::new();
    for (b, (_, bl)) in ordem.iter().enumerate() {
        let l = &bl.l2;
        let nl = bl.peca.verts.len();
        gslot.clear();
        gslot.extend(l.slots.iter().enumerate().map(|(s, &r)| {
            if s < nl {
                mapa[mbase[b] + s]
            } else {
                gv(b, r)
            }
        }));
        for (pi, w) in l.s_off.windows(2).enumerate() {
            let (o0, o1) = (w[0] as usize, w[1] as usize);
            for pos in o0..o1 {
                let u = gslot[l.s_ring[pos] as usize];
                ring.push(u);
                let (nb, tw) = match l.lig[pos] {
                    Lig::Parede => {
                        let v = gslot[l.s_ring[if pos + 1 == o1 { o0 } else { pos + 1 }] as usize];
                        corner[u as usize] = true;
                        corner[v as usize] = true;
                        walls.push((u, v));
                        (None, u32::MAX)
                    }
                    Lig::Dentro { q, e } => (Some(pbase[b] + q), e),
                    Lig::Fora { d, q, e } => {
                        let o = viz[b][d as usize];
                        let (ca, cb) = (
                            raiz(&mut pai, cbase[b] + l.comp[pi]),
                            raiz(&mut pai, cbase[o] + ordem[o].1.l2.comp[q as usize]),
                        );
                        pai[ca.max(cb) as usize] = ca.min(cb);
                        (Some(pbase[o] + q), e)
                    }
                };
                nbrs.push(nb);
                twin.push(tw);
            }
            ring_off.push(ring.len() as u32);
        }
        area.extend_from_slice(&bl.peca.area);
    }

    let (vp_off, vert_polys) = polys_de_cada_vertice(nv, &ring_off, &ring);

    // As ilhas pela ordem do 1.º polígono = a ordem das componentes (cada bloco numera as suas pela
    // ordem do 1.º polígono, e os blocos vão por ordem).
    let mut ilha_da_raiz = vec![u32::MAX; cbase[n] as usize];
    let mut ilha_da_comp = vec![0u32; cbase[n] as usize];
    let mut islands = 0;
    for c in 0..cbase[n] {
        let r = raiz(&mut pai, c) as usize;
        if ilha_da_raiz[r] == u32::MAX {
            ilha_da_raiz[r] = islands;
            islands += 1;
        }
        ilha_da_comp[c as usize] = ilha_da_raiz[r];
    }
    let mut island = Vec::with_capacity(tp);
    for (b, (_, bl)) in ordem.iter().enumerate() {
        island.extend(bl.l2.comp.iter().map(|&c| ilha_da_comp[(cbase[b] + c) as usize]));
    }

    // A caixa e a localização: uma grelha por bloco, numa grelha regular de blocos.
    let (mut min, mut max) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for (_, bl) in &ordem {
        if let Some((lo, hi)) = bl.l1.caixa {
            min = [min[0].min(lo[0]), min[1].min(lo[1])];
            max = [max[0].max(hi[0]), max[1].max(hi[1])];
        }
        xs.extend([bl.peca.lo[0], bl.peca.hi[0]]);
        ys.extend([bl.peca.lo[1], bl.peca.hi[1]]);
    }
    if ring.is_empty() {
        (min, max) = ([0.0; 2], [0.0; 2]);
    }
    for v in [&mut xs, &mut ys] {
        v.sort_by(f64::total_cmp);
        v.dedup();
    }
    let nx = xs.len().saturating_sub(1);
    let mut grelhas = vec![None; nx * ys.len().saturating_sub(1)];
    for (b, (_, bl)) in ordem.iter().enumerate() {
        if let Some(g) = &bl.l1.grid {
            let ix = xs.partition_point(|&f| f < bl.peca.lo[0]);
            let iy = ys.partition_point(|&f| f < bl.peca.lo[1]);
            grelhas[iy * nx + ix] = Some(GrelhaDoBloco {
                base: pbase[b],
                grid: g.clone(),
            });
        }
    }
    Ok(NavMesh {
        verts,
        ring_off,
        ring,
        nbrs,
        twin,
        corner,
        vp_off,
        vert_polys,
        island,
        islands,
        area,
        walls,
        grid: Localizador::Blocos {
            xs,
            ys,
            blocos: grelhas,
        },
        min,
        max,
    })
}

/// O erro de um bloco com o índice do polígono na malha montada.
fn global(e: &MeshError, base: u32) -> MeshError {
    let b = base as usize;
    match e.clone() {
        MeshError::Degenerate { poly } => MeshError::Degenerate { poly: poly + b },
        MeshError::BadIndex { poly, index } => MeshError::BadIndex {
            poly: poly + b,
            index,
        },
        MeshError::NotCcw { poly, area2 } => MeshError::NotCcw {
            poly: poly + b,
            area2,
        },
        MeshError::NotConvex { poly, at } => MeshError::NotConvex { poly: poly + b, at },
        outro => outro,
    }
}
