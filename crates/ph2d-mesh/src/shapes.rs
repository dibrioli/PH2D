//! Geometria de teste — a fixture que os gates E as sondas compartilham.
//!
//! ⚠️ **A esfera UV é a fixture certa e não é escolha estética:** ela tem quads
//! no corpo e triângulos nas calotas, então todo caminho que se ramifica em
//! `Face::is_tri` é exercitado pela MESMA malha. Uma fixture só-triângulo — que
//! é o que se escreve sem pensar — deixaria metade do código sem cobertura e os
//! gates verdes, que é exatamente a forma de fixture que este repositório já
//! pagou para aprender.
//!
//! ⚠️ Estas funções usam `sin`/`cos` da `std`, que **não** são pinadas bit a bit
//! entre sistemas operacionais. É seguro porque os gates daqui afirmam
//! propriedades estruturais (contagens, adjacência, orientação), nunca literais
//! de ponto flutuante. Um gate futuro que queira um valor exato precisa gerar a
//! fixture de outro jeito, e este parágrafo existe para ele não descobrir isso
//! num CI vermelho de outro OS.

use crate::face::Face;
use crate::mesh::Mesh;

/// Um cubo de aresta `size` centrado na origem — **6 quads**, 8 vértices.
///
/// A malha mais barata que ainda é fechada e orientável: é ela que responde
/// "a normal aponta para fora?" sem que a resposta dependa de tesselação.
#[must_use]
pub fn cube(size: f32) -> Mesh {
    let h = size * 0.5;
    let positions = vec![
        [-h, -h, -h], // 0
        [h, -h, -h],  // 1
        [h, h, -h],   // 2
        [-h, h, -h],  // 3
        [-h, -h, h],  // 4
        [h, -h, h],   // 5
        [h, h, h],    // 6
        [-h, h, h],   // 7
    ];
    // Winding CCW visto de FORA em cada face (regra da mão direita ⇒ a normal
    // de Newell sai apontando para fora, que é o que o gate de orientação afirma).
    let faces = vec![
        Face::quad(4, 5, 6, 7), // +Z
        Face::quad(1, 0, 3, 2), // -Z
        Face::quad(5, 1, 2, 6), // +X
        Face::quad(0, 4, 7, 3), // -X
        Face::quad(3, 7, 6, 2), // +Y
        Face::quad(0, 1, 5, 4), // -Y
    ];
    Mesh::from_parts(positions, faces).expect("o cubo é construído aqui e é válido")
}

/// Uma esfera UV de raio `radius`, com `rings` anéis e `segments` segmentos.
///
/// `rings >= 2` e `segments >= 3` são impostos por clamp e não por pânico: uma
/// fixture degenerada vinda de um laço de varredura tem de virar a menor esfera
/// legítima, não derrubar a sonda no meio da tabela.
#[must_use]
pub fn uv_sphere(rings: usize, segments: usize, radius: f32) -> Mesh {
    let rings = rings.max(2);
    let segments = segments.max(3);

    let mut positions = Vec::with_capacity(2 + (rings - 1) * segments);
    positions.push([0.0, radius, 0.0]); // polo norte = 0
    for i in 1..rings {
        let phi = core::f32::consts::PI * (i as f32) / (rings as f32);
        let (sp, cp) = phi.sin_cos();
        for j in 0..segments {
            let theta = core::f32::consts::TAU * (j as f32) / (segments as f32);
            let (st, ct) = theta.sin_cos();
            positions.push([radius * sp * ct, radius * cp, radius * sp * st]);
        }
    }
    let south = positions.len() as u32;
    positions.push([0.0, -radius, 0.0]);

    // Índice do vértice no anel `i` (1-based), segmento `j`.
    let at = |i: usize, j: usize| -> u32 { (1 + (i - 1) * segments + (j % segments)) as u32 };

    let mut faces = Vec::new();
    for j in 0..segments {
        faces.push(Face::tri(0, at(1, j + 1), at(1, j)));
    }
    for i in 1..rings - 1 {
        for j in 0..segments {
            faces.push(Face::quad(
                at(i, j),
                at(i, j + 1),
                at(i + 1, j + 1),
                at(i + 1, j),
            ));
        }
    }
    for j in 0..segments {
        faces.push(Face::tri(south, at(rings - 1, j), at(rings - 1, j + 1)));
    }

    Mesh::from_parts(positions, faces).expect("a esfera é construída aqui e é válida")
}

/// A esfera com aproximadamente `target` triângulos — a porta que as sondas
/// usam para varrer escala.
///
/// Uma esfera `r × r` tem ~`2r²` triângulos, então `r = sqrt(target / 2)`. O
/// número devolvido é *aproximado de propósito*: a sonda imprime a contagem
/// REAL (`triangle_count`), porque uma tabela que reporta o alvo em vez do
/// medido é uma tabela que mente com uma casa decimal.
#[must_use]
pub fn sphere_with_triangles(target: usize, radius: f32) -> Mesh {
    let r = ((target as f64 / 2.0).sqrt().round() as usize).max(3);
    uv_sphere(r, r, radius)
}
