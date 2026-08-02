//! Geometria de teste — a fixture que os gates E as sondas compartilham.
//!
//! ⚠️ **A esfera UV cobre o ramo tri/quad, e SÓ ele:** ela tem quads no corpo e
//! triângulos nas calotas, então todo caminho que se ramifica em `Face::is_tri`
//! é exercitado pela MESMA malha. Uma fixture só-triângulo — que é o que se
//! escreve sem pensar — deixaria metade desses caminhos sem cobertura e os gates
//! verdes.
//!
//! ⚠️ **E o [`octahedron`] existe porque a esfera UV NÃO isola o ramo de
//! triângulo:** ela é mista, então um gate escrito sobre ela não distingue a
//! regra de Loop da de Catmull-Clark. Ele é a única fixture daqui que é fechada
//! e **só-triângulo**.
//!
//! ⚠️ **E o que ela NÃO cobre está medido, então não confie neste arquivo para
//! mais que aquilo.** Varridas as 18 dimensões que os gates usam, de `(5,8)` a
//! `(96,144)`: **borda = 0, valência ≤ 2 = 0, face degenerada = 0** — em todas,
//! nos dois cubos e no octaedro. Não é acidente de tamanho: as funções daqui só sabem fazer
//! malha fechada e bem formada. Quem precisa de um desses fenômenos usa a
//! [`crate::shapes_open`], e a frase anterior deste cabeçalho — que se lia como
//! cobertura geral — era exatamente a forma de nota que deixa uma classe de
//! defeito invisível com a suíte verde.
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

/// Um octaedro de "raio" `size` — **8 triângulos**, 6 vértices, valência 4 em
/// todo vértice.
///
/// ⚠️ **A fixture que faltava: fechada e SÓ-TRIÂNGULO.** A esfera UV tem quads
/// no corpo e o cubo é todo quad, então até aqui nenhuma malha fechada isolava o
/// ramo de **Loop** — o de Catmull-Clark cobria os dois e um gate escrito sobre
/// ela não distinguiria as duas regras. Valência 4 de propósito: a valência 3
/// cairia nos pesos de Warren, que são um caso especial dentro do caso.
#[must_use]
pub fn octahedron(size: f32) -> Mesh {
    let s = size;
    let positions = vec![
        [s, 0.0, 0.0],  // 0  +X
        [-s, 0.0, 0.0], // 1  -X
        [0.0, s, 0.0],  // 2  +Y
        [0.0, -s, 0.0], // 3  -Y
        [0.0, 0.0, s],  // 4  +Z
        [0.0, 0.0, -s], // 5  -Z
    ];
    // Winding CCW visto de fora, o mesmo critério do [`cube`].
    let faces = vec![
        Face::tri(0, 2, 4),
        Face::tri(2, 1, 4),
        Face::tri(1, 3, 4),
        Face::tri(3, 0, 4),
        Face::tri(2, 0, 5),
        Face::tri(1, 2, 5),
        Face::tri(3, 1, 5),
        Face::tri(0, 3, 5),
    ];
    Mesh::from_parts(positions, faces).expect("o octaedro é construído aqui e é válido")
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

/// **A mesma malha, com os vértices EMBARALHADOS** — o que um OBJ importado é.
///
/// ⚠️ **Ela existe porque sem ela as fixtures da reversão NÃO CONTINHAM o
/// fenômeno, e isso foi medido.** A [`crate::subdivide`] numera os vértices
/// originais PRIMEIRO, então reverter `subdivide(cube)` devolve a permutação
/// IDENTIDADE — e toda a maquinaria de renumeração, inclusive a cascata que
/// renumera os níveis acima, fica invisível a qualquer gate escrito sobre ela.
/// Duas mutações sobreviveram por isto antes de esta função existir.
///
/// Um arquivo de terceiro não tem obrigação nenhuma de ordenar vértices, e é
/// exatamente esse arquivo que a reversão serve. Fisher-Yates com um LCG:
/// **determinístico** dado o `seed`, e sem dependência nova.
#[must_use]
pub fn shuffled(mesh: &Mesh, seed: u64) -> Mesh {
    let n = mesh.vert_count();
    let mut order: Vec<u32> = (0..n as u32).collect();
    let mut s = seed | 1;
    for i in (1..n).rev() {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (s >> 33) as usize % (i + 1);
        order.swap(i, j);
    }
    let mut inv = vec![0u32; n];
    for (j, &o) in order.iter().enumerate() {
        inv[o as usize] = u32::try_from(j).unwrap_or(u32::MAX);
    }
    let positions: Vec<[f32; 3]> = order
        .iter()
        .map(|&o| mesh.positions()[o as usize])
        .collect();
    let faces: Vec<Face> = mesh
        .faces()
        .iter()
        .map(|f| {
            let mut g = *f;
            for k in 0..f.vert_count() {
                g.0[k] = inv[g.0[k] as usize];
            }
            g
        })
        .collect();
    let mut out = Mesh::from_parts(positions, faces).expect("embaralhar não inventa índice");
    if let Some(c) = mesh.colors() {
        let dst = out.colors_mut();
        for (j, &o) in order.iter().enumerate() {
            dst[j] = c[o as usize];
        }
    }
    if let Some(m) = mesh.masks() {
        let dst = out.masks_mut();
        for (j, &o) in order.iter().enumerate() {
            dst[j] = m[o as usize];
        }
    }
    out
}

/// Um CILINDRO em pé no eixo Y: raio `radius`, altura `height`, `segments`
/// fatias — **quads no corpo, leques nas tampas**.
///
/// ⚠️ **As tampas têm um POLO cada, como a esfera UV, e é uma escolha.** A
/// alternativa (uma grade de quads no disco) evita a valência alta mas inventa
/// uma tesselação interna que a silhueta não pede, e ela some no primeiro
/// `subdivide`. O polo aqui é o mesmo fenômeno que a `uv_sphere` já traz — e o
/// corpo, que é onde um escultor trabalha, é quad puro.
#[must_use]
pub fn cylinder(segments: usize, radius: f32, height: f32) -> Mesh {
    let segments = segments.max(3);
    let h = height * 0.5;
    // 0 = centro do topo, 1 = centro da base; os anéis vêm depois.
    let mut positions = vec![[0.0, h, 0.0], [0.0, -h, 0.0]];
    for j in 0..segments {
        let theta = core::f32::consts::TAU * (j as f32) / (segments as f32);
        let (st, ct) = theta.sin_cos();
        positions.push([radius * ct, h, radius * st]);
    }
    for j in 0..segments {
        let theta = core::f32::consts::TAU * (j as f32) / (segments as f32);
        let (st, ct) = theta.sin_cos();
        positions.push([radius * ct, -h, radius * st]);
    }
    let top = |j: usize| -> u32 { (2 + j % segments) as u32 };
    let bot = |j: usize| -> u32 { (2 + segments + j % segments) as u32 };

    let mut faces = Vec::new();
    for j in 0..segments {
        faces.push(Face::tri(0, top(j + 1), top(j)));
        faces.push(Face::tri(1, bot(j), bot(j + 1)));
        faces.push(Face::quad(top(j), top(j + 1), bot(j + 1), bot(j)));
    }
    Mesh::from_parts(positions, faces).expect("o cilindro é construído aqui e é válido")
}

/// Um TORO no plano XZ: raio maior `major`, raio menor `minor`, `major_segments`
/// voltas e `minor_segments` por volta.
///
/// ⚠️ **É a única primitiva daqui sem POLO nenhum** — quad puro, valência 4 em
/// todo vértice. É por isso que ela é a fixture certa para qualquer coisa que
/// discuta *topologia regular*, e é também a forma que um escultor recebe de
/// melhor grado: ela subdivide sem estrela nenhuma para o alisamento contornar.
#[must_use]
pub fn torus(major_segments: usize, minor_segments: usize, major: f32, minor: f32) -> Mesh {
    let (mj, mn) = (major_segments.max(3), minor_segments.max(3));
    let mut positions = Vec::with_capacity(mj * mn);
    for i in 0..mj {
        let u = core::f32::consts::TAU * (i as f32) / (mj as f32);
        let (su, cu) = u.sin_cos();
        for k in 0..mn {
            let v = core::f32::consts::TAU * (k as f32) / (mn as f32);
            let (sv, cv) = v.sin_cos();
            let r = major + minor * cv;
            positions.push([r * cu, minor * sv, r * su]);
        }
    }
    let at = |i: usize, k: usize| -> u32 { ((i % mj) * mn + (k % mn)) as u32 };
    let mut faces = Vec::with_capacity(mj * mn);
    for i in 0..mj {
        for k in 0..mn {
            faces.push(Face::quad(
                at(i, k),
                at(i, k + 1),
                at(i + 1, k + 1),
                at(i + 1, k),
            ));
        }
    }
    Mesh::from_parts(positions, faces).expect("o toro é construído aqui e é válido")
}

#[cfg(test)]
#[path = "shapes_tests.rs"]
mod tests;
