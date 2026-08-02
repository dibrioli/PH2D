//! **Surface Nets** — a superfície de nível zero do campo, em quads.
//!
//! Adaptado de `src/editing/SurfaceNets.js` do SculptGL (MIT), que por sua vez
//! é o `isosurface` de Mikola Lysenko sobre *S. F. Gibson, "Constrained Elastic
//! Surface Nets" (1998)*. Licença em `LICENSES/sculptgl-MIT.txt`.
//!
//! # Por que quads, e por que isto e não marching cubes
//!
//! Surface Nets põe **um vértice por célula** que a superfície cruza e liga os
//! vizinhos — então a saída é uma grade deformada, com valência 4 quase em toda
//! parte. É a topologia que um escultor quer receber de volta: o marching cubes
//! devolve triângulos finos e valências irregulares, e a malha resultante
//! subdivide mal.
//!
//! O preço é que a saída **não é garantidamente manifold** em configurações
//! ambíguas de célula. O marching cubes da referência existe justamente para
//! quem precisa dessa garantia, e é wave própria.

use ph2d_mesh::{Face, Mesh, MeshError};

use crate::field::VoxelField;

/// Os 12 pares de cantos que formam as arestas de um cubo, na ordem em que o
/// `edge_table` os enumera.
fn cube_edges() -> [usize; 24] {
    let mut out = [0usize; 24];
    let mut k = 0;
    for i in 0..8usize {
        let mut j = 1usize;
        while j <= 4 {
            let p = i ^ j;
            if i <= p {
                out[k] = i;
                out[k + 1] = p;
                k += 2;
            }
            j <<= 1;
        }
    }
    out
}

/// Para cada uma das 256 configurações de sinal dos 8 cantos, quais das 12
/// arestas cruzam o nível zero.
fn edge_table(edges: &[usize; 24]) -> [u32; 256] {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut em = 0u32;
        for j in (0..24).step_by(2) {
            let a = i & (1 << edges[j]) != 0;
            let b = i & (1 << edges[j + 1]) != 0;
            if a != b {
                em |= 1 << (j >> 1);
            }
        }
        *slot = em;
    }
    table
}

/// Extrai a superfície do campo.
///
/// O campo já tem de ter passado pelo [`VoxelField::flood_fill`] — sem sinal não
/// há nível zero a cruzar, e a saída sai vazia.
pub fn surface_nets(field: &VoxelField) -> Result<Mesh, MeshError> {
    let dims = field.dims();
    let (rx, ry, rz) = (dims[0], dims[1], dims[2]);
    let data = field.distances();
    let step = field.step();
    let origin = field.origin();

    let edges = cube_edges();
    let table = edge_table(&edges);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();

    let rxy = rx * ry;
    // A régua do buffer rolante: duas fatias de `(rx+1) * (ry+1)` índices, e a
    // terceira componente TROCA DE SINAL a cada fatia — é assim que ler "a
    // célula equivalente na fatia anterior" vira uma soma, sem copiar nada.
    let r: [isize; 2] = [1, (rx + 1) as isize];
    let slice = ((rx + 1) * (ry + 1)) as isize;
    let mut rz2 = slice;
    let mut buffer = vec![0u32; (slice * 2) as usize];
    let mut nb_buf = 1isize;

    let mut grid = [0f32; 8];

    for z in 0..rz.saturating_sub(1) {
        let mut m = 1 + (rx as isize + 1) * (1 + nb_buf * (ry as isize + 1));

        for y in 0..ry.saturating_sub(1) {
            for x in 0..rx.saturating_sub(1) {
                let n = x + y * rx + z * rxy;

                // Os 8 cantos da célula, e a máscara de quem está DENTRO.
                let mut mask = 0usize;
                let mut g = 0;
                for k in 0..2 {
                    for j in 0..2 {
                        for i in 0..2 {
                            let p = data[n + i + j * rx + k * rxy];
                            grid[g] = p;
                            if p < 0.0 {
                                mask |= 1 << g;
                            }
                            g += 1;
                        }
                    }
                }

                if mask == 0 || mask == 0xff {
                    m += 1;
                    continue;
                }

                let edge_mask = table[mask];
                buffer[m as usize] = positions.len() as u32;
                positions.push(interpolate(
                    edge_mask,
                    &edges,
                    &grid,
                    [x, y, z],
                    origin,
                    step,
                ));

                // As faces: para cada um dos 3 eixos cuja aresta-base cruza, o
                // quad formado com os três vizinhos já visitados.
                for i in 0..3usize {
                    if edge_mask & (1 << i) == 0 {
                        continue;
                    }
                    let iu = (i + 1) % 3;
                    let iv = (i + 2) % 3;
                    let coord = [x, y, z];
                    // Na borda da grade os vizinhos não existem ainda.
                    if coord[iu] == 0 || coord[iv] == 0 {
                        continue;
                    }
                    let du = if iu == 2 { rz2 } else { r[iu] };
                    let dv = if iv == 2 { rz2 } else { r[iv] };
                    let at = |d: isize| buffer[(m + d) as usize];
                    // O sinal do canto 0 decide a orientação — sem isto metade
                    // dos quads sai com a normal para dentro.
                    let q = if mask & 1 != 0 {
                        Face::quad(at(0), at(-du), at(-du - dv), at(-dv))
                    } else {
                        Face::quad(at(0), at(-dv), at(-du - dv), at(-du))
                    };
                    faces.push(q);
                }

                m += 1;
            }
            m += 2;
        }

        nb_buf ^= 1;
        rz2 = -rz2;
    }

    Mesh::from_parts(positions, faces)
}

/// A posição do vértice da célula: a média dos pontos onde as arestas cruzam o
/// zero, que é o que faz a superfície *encolher* para dentro da célula em vez de
/// ficar em degraus.
fn interpolate(
    edge_mask: u32,
    edges: &[usize; 24],
    grid: &[f32; 8],
    coord: [usize; 3],
    origin: [f32; 3],
    step: f32,
) -> [f32; 3] {
    let mut acc = [0f32; 3];
    let mut count = 0f32;

    for i in 0..12usize {
        if edge_mask & (1 << i) == 0 {
            continue;
        }
        let e0 = edges[i << 1];
        let e1 = edges[(i << 1) + 1];
        let g0 = grid[e0];
        let g1 = grid[e1];
        let den = g0 - g1;
        // ⚠️ A referência só testa `|den| < 1e-7`, e isso deixa passar o caso em
        // que os dois lados são infinitos de sinais opostos: `inf / inf` é
        // **NaN**, e um NaN aqui envenena o vértice inteiro sem erro nenhum. A
        // finitude é conferida antes.
        if !den.is_finite() || !g0.is_finite() || den.abs() < 1e-7 {
            continue;
        }
        let t = g0 / den;
        count += 1.0;

        // Cada canto é um bit por eixo: o cruzamento anda `t` no eixo em que os
        // dois cantos diferem e fica no canto nos outros dois.
        for (j, k) in (0..3).map(|j| (j, 1usize << j)) {
            let a = e0 & k;
            if a != (e1 & k) {
                acc[j] += if a != 0 { 1.0 - t } else { t };
            } else if a != 0 {
                acc[j] += 1.0;
            }
        }
    }

    if count == 0.0 {
        // Toda aresta degenerada: o centro da célula é a resposta honesta.
        count = 1.0;
        acc = [0.5, 0.5, 0.5];
    }

    let s = 1.0 / count;
    [
        origin[0] + (coord[0] as f32 + s * acc[0]) * step,
        origin[1] + (coord[1] as f32 + s * acc[1]) * step,
        origin[2] + (coord[2] as f32 + s * acc[2]) * step,
    ]
}

#[cfg(test)]
#[path = "surface_nets_tests.rs"]
mod tests;
