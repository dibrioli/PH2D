//! Fixtures **malformadas** — cada uma existe para conter UM fenômeno que a
//! [`crate::shapes`] não sabe produzir.
//!
//! ⚠️ **A razão de existirem é um número medido.** Varridas as 18 dimensões de
//! `uv_sphere` que os gates usam — de `(5,8)` a `(96,144)` —, mais
//! `sphere_with_triangles` e os dois cubos: **borda = 0, valência ≤ 2 = 0, face
//! degenerada = 0 em TODAS**. Não é acidente de tamanho; a `shapes.rs` só sabe
//! fazer malha fechada e bem formada. Uma classe inteira de defeito — a regra de
//! borda do laplaciano, o congelamento de valência baixa, a normal fabricada de
//! face degenerada — era **invisível a qualquer gate**, não porque ninguém a
//! testou, mas porque nenhuma fixture podia contê-la.
//!
//! ⚠️ **Nenhuma delas tem parâmetro, de propósito.** Um `open_tube(rings)` é um
//! convite a chamá-lo com `rings = 2`, e aí a fixture deixa de conter o fenômeno
//! sem que nada falhe (ver a nota do [`open_tube3`]). Cada função aqui é UMA
//! malha, com o censo dela pinado no gate irmão.
//!
//! ⚠️ **`shapes_open` e não `tests/fixtures/`.** O precedente de arquivo-fixture
//! neste repo (`ph2d-ecs`, `ph2d-render`, `ph2d-imageio-psd`, `ph2d-audio-ml`) é
//! para **byte-goldens de serialização**; um `.obj` naquele diretório é
//! inalcançável da `ph2d-sculpt3d` e da `ph2d-mesh-render`, que é exatamente
//! onde moram os gates de Smooth e de sombreamento. É a razão que o
//! [`crate::shapes`] já escreveu para si mesma, um nível acima.

use crate::face::Face;
use crate::mesh::Mesh;

/// Um tubo aberto de **três** anéis — a fixture da BORDA.
///
/// V=18, F=12 quads, 12 vértices de borda e **6 de interior**.
///
/// ⚠️ **Os três anéis são o ponto inteiro, e dois não servem.** Se *todo*
/// vértice é de borda, *"promediar só com vizinhos também de borda"* e
/// *"promediar com o anel inteiro"* são a **mesma instrução** — medido nas duas
/// regras sobre um tubo de 2 anéis: `|Δy| = 0,333333`, idênticas. A regra de
/// borda só morde onde a borda **encosta em interior**, então uma fixture de
/// borda sem interior é uma fixture que não contém o fenômeno.
#[must_use]
pub fn open_tube3() -> Mesh {
    const SEGMENTS: usize = 6;
    const RINGS: usize = 3;

    let mut positions = Vec::with_capacity(RINGS * SEGMENTS);
    for i in 0..RINGS {
        let y = i as f32 - 1.0;
        for j in 0..SEGMENTS {
            let theta = core::f32::consts::TAU * (j as f32) / (SEGMENTS as f32);
            let (st, ct) = theta.sin_cos();
            positions.push([ct, y, st]);
        }
    }

    let at = |i: usize, j: usize| ((i * SEGMENTS) + (j % SEGMENTS)) as u32;
    // Winding CCW visto de FORA (a normal de Newell sai radial para longe do
    // eixo) — a mesma convenção do `cube`, e a que o gate de orientação afirma.
    let mut faces = Vec::with_capacity((RINGS - 1) * SEGMENTS);
    for i in 0..RINGS - 1 {
        for j in 0..SEGMENTS {
            faces.push(Face::quad(
                at(i, j),
                at(i + 1, j),
                at(i + 1, j + 1),
                at(i, j + 1),
            ));
        }
    }

    Mesh::from_parts(positions, faces).expect("o tubo é construído aqui e é válido")
}

/// Um disco PLANO de três anéis, com o anel de fora **desigualmente espaçado** —
/// a fixture da BORDA CURVA.
///
/// V=19, F=24 triângulos, 12 vértices de borda e 7 de interior; todos em
/// `z = 0`, logo a normal da superfície é `±z` em toda parte.
///
/// ⚠️ **Ela existe porque o [`open_tube3`] NÃO contém o fenômeno, e isso foi
/// MEDIDO antes de o gate existir** (sonda `measure_slide_relax`): o
/// `SlideRelax` troca, numa beira, a normal do vértice pela **bissetriz das
/// arestas de borda**, e as duas só divergem onde a curva de borda **não** é
/// perpendicular à superfície. Num tubo as duas são radiais — o ângulo entre
/// elas mede **0,015°** nos 12 vértices de beira —, então um gate escrito ali
/// teria passado com a bissetriz trocada pela normal e não diria nada.
///
/// Num disco plano elas são **ortogonais**: a normal é `+z` e a bissetriz aponta
/// para o centro. Com a normal errada, a média dos dois vizinhos de beira (que é
/// o ponto médio da corda, dentro do círculo) sobrevive INTEIRA à subtração e a
/// borda é sugada para dentro dab a dab; com a bissetriz, o que sobra é só o
/// deslize AO LONGO da beira, que é redistribuir sem encolher.
///
/// ⚠️ **O anel de fora é desigual DE PROPÓSITO** (as pás alternam
/// `±ANGULAR_SKEW`): num polígono regular cada vértice já está no ponto de menor
/// energia do próprio par, então a componente tangencial é zero por construção e
/// a fixture voltaria a não conter o fenômeno — o mesmo defeito que a `uv_sphere`
/// tem para o miolo, e que a sonda também mediu (`cv` de arestas movendo 0,016%).
///
/// ⚠️ **Três anéis, e dois não servem** — a mesma razão que o [`open_tube3`]
/// escreve: a regra de borda só morde onde a beira **encosta em interior**.
#[must_use]
pub fn open_disc() -> Mesh {
    const SEGMENTS: usize = 12;
    /// Quanto cada pá alternada foge do espaçamento regular, em fração de setor.
    const ANGULAR_SKEW: f32 = 0.3;

    let mut positions = vec![[0.0, 0.0, 0.0]];
    let sector = core::f32::consts::TAU / SEGMENTS as f32;
    // O anel de dentro tem metade das pás — é ele que dá INTERIOR à fixture.
    for j in 0..SEGMENTS / 2 {
        let theta = sector * 2.0 * (j as f32);
        let (st, ct) = theta.sin_cos();
        positions.push([ct * 0.5, st * 0.5, 0.0]);
    }
    for j in 0..SEGMENTS {
        let skew = if j % 2 == 0 { ANGULAR_SKEW } else { 0.0 };
        let theta = sector * (j as f32 + skew);
        let (st, ct) = theta.sin_cos();
        positions.push([ct, st, 0.0]);
    }

    let inner = |j: usize| (1 + (j % (SEGMENTS / 2))) as u32;
    let outer = |j: usize| (1 + SEGMENTS / 2 + (j % SEGMENTS)) as u32;

    let mut faces = Vec::with_capacity(SEGMENTS / 2 + SEGMENTS);
    // O leque do centro.
    for j in 0..SEGMENTS / 2 {
        faces.push(Face::tri(0, inner(j), inner(j + 1)));
    }
    // A coroa: cada pá de dentro cobre DUAS de fora.
    for j in 0..SEGMENTS / 2 {
        faces.push(Face::tri(inner(j), outer(2 * j), outer(2 * j + 1)));
        faces.push(Face::tri(inner(j), outer(2 * j + 1), outer(2 * j + 2)));
        faces.push(Face::tri(inner(j), outer(2 * j + 2), inner(j + 1)));
    }

    Mesh::from_parts(positions, faces).expect("o disco é construído aqui e é válido")
}

/// Duas faces sobre os MESMOS três vértices, com winding oposto — a fixture da
/// VALÊNCIA BAIXA.
///
/// V=3, F=2, **zero** vértices de borda e **três** de valência 2. O SculptGL
/// congela esse vértice no Smooth (`if (vcount <= 2) continue`); nós só
/// protegemos o anel vazio, e com dois vizinhos devolvemos o ponto médio deles
/// ⇒ uma ponta de tira **escorrega para a corda** (§livro-razão B).
///
/// ⚠️ **Ela não é de borda, e é isso que a torna necessária.** Pela regra do
/// original (`nº de faces != nº de vizinhos únicos`) cada vértice aqui tem 2 e 2
/// ⇒ *interior*. Uma fixture que confundisse os dois fenômenos deixaria a
/// correção de um passar pela do outro.
///
/// O winding oposto é deliberado: é o que faz dela uma superfície **fechada de
/// volume zero** em vez de duas faces soltas coincidentes — e por isso ela
/// **não** entra no gate de orientação, que só fala de sólido convexo.
#[must_use]
pub fn pillow() -> Mesh {
    let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
    let faces = vec![Face::tri(0, 1, 2), Face::tri(0, 2, 1)];
    Mesh::from_parts(positions, faces).expect("o pillow é construído aqui e é válido")
}

/// Um tetraedro com dois vértices COINCIDENTES — a fixture da FACE DEGENERADA.
///
/// V=4, F=4, das quais **as faces 1 e 3 têm área exatamente zero** (as duas que
/// contêm o par colapsado `v2 == v3`). O `normalize` das normais devolve `+Y`
/// unitário para elas, e esse vetor fabricado entra na média dos vértices **com
/// peso cheio**; no original a face degenerada contribui o vetor ZERO
/// (§livro-razão E ⛔).
///
/// ⚠️ **O vértice que DISCRIMINA não é o v0.** Com `v2 == v3` a forma colapsa
/// num triângulo duplo, então as faces 0 e 2 viram exatamente opostas e se
/// cancelam — em v0 e v1 o resultado é `[0,1,0]` **com ou sem** a cura, e um
/// gate ancorado ali fica verde nos dois mundos. Quem separa é **v2/v3**, cujo
/// anel tem uma face boa e duas degeneradas.
#[must_use]
pub fn collapsed_tetra() -> Mesh {
    // Um tetraedro regular centrado na origem, com o v3 pousado EM CIMA do v2.
    let positions = vec![
        [1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, 1.0, -1.0], // colapsado sobre o vértice 2
    ];
    // As quatro faces do tetraedro na orientação de FORA (conferido contra o
    // quarto vértice de cada uma). As que contêm o par colapsado — 1 e 3 —
    // são as degeneradas, e a numeração é o que os gates citam.
    let faces = vec![
        Face::tri(0, 1, 2),
        Face::tri(0, 2, 3),
        Face::tri(0, 3, 1),
        Face::tri(1, 3, 2),
    ];
    Mesh::from_parts(positions, faces).expect("o tetra colapsado é construído aqui e é válido")
}

/// Uma bipirâmide de anel MUITO irregular — a fixture do LEQUE COM SLIVER.
///
/// V=6, F=8. O anel equatorial tem quatro vértices em `0° · 10° · 20° · 190°`:
/// dois arcos de 10° e dois de 170°, então cada ápice é um leque de **dois
/// triângulos finíssimos e dois enormes**.
///
/// É onde a divergência §livro-razão E — *nós somamos normais UNITÁRIAS, o
/// original soma o Newell CRU e portanto pondera por ÁREA* — deixa de ser
/// desprezível: **37,99° aqui contra 1,40° na `uv_sphere(32,48)`**, medido, o
/// controle e a fixture lado a lado no gate irmão.
///
/// ⚠️ O handoff da W4 previa 20,06° para esta fixture; o anel que ficou é mais
/// extremo que o prototipado e mede quase o dobro. O controle bate no dígito, o
/// que é o sinal de que as duas medições falam da mesma grandeza.
///
/// ⚠️ **Os arcos são todos < 180° de propósito:** é o que mantém o centro
/// ESTRITAMENTE dentro do quadrilátero e, portanto, a bipirâmide **convexa** —
/// sem isso ela não poderia entrar no gate de orientação, que é justamente onde
/// uma fixture nova mal-enrolada é pega.
#[must_use]
pub fn sliver_bipyramid() -> Mesh {
    const RING_DEG: [f32; 4] = [0.0, 10.0, 20.0, 190.0];

    let mut positions = Vec::with_capacity(RING_DEG.len() + 2);
    positions.push([0.0, 1.0, 0.0]); // 0 = ápice norte
    for d in RING_DEG {
        let (s, c) = d.to_radians().sin_cos();
        positions.push([c, 0.0, s]); // 1..=4 = o anel
    }
    let south = positions.len() as u32;
    positions.push([0.0, -1.0, 0.0]);

    let ring = |j: usize| (1 + j % RING_DEG.len()) as u32;
    let mut faces = Vec::with_capacity(RING_DEG.len() * 2);
    for j in 0..RING_DEG.len() {
        faces.push(Face::tri(0, ring(j + 1), ring(j)));
        faces.push(Face::tri(south, ring(j), ring(j + 1)));
    }

    Mesh::from_parts(positions, faces).expect("a bipirâmide é construída aqui e é válida")
}

#[cfg(test)]
#[path = "shapes_open_tests.rs"]
mod tests;
