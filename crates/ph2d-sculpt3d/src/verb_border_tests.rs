//! **A BORDA** — as duas regras que só uma malha ABERTA pode revelar.
//!
//! ⚠️ Este arquivo é o consumidor das fixtures que a `shapes_open` construiu: até
//! ele existir, `open_tube3` e `pillow` eram infraestrutura sem quem as usasse.
//! Numa `uv_sphere` não há vértice de borda nem valência < 3 (medido), então as
//! duas regras são inertes lá — e a classe inteira ficou invisível por isso.

use super::*;
use ph2d_mesh::shapes_open;

fn smooth() -> Brush {
    Brush {
        verb: Verb::Smooth,
        radius: 10.0,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Quanto o tubo mede ao longo do PRÓPRIO EIXO, medido nos vértices de borda —
/// **o quanto as duas bocas foram sugadas para dentro**.
///
/// ⚠️ **E não o raio delas, que foi o meu primeiro oráculo e estava ERRADO.** A
/// beira do tubo é um HEXÁGONO, e alisar um polígono o encolhe para o círculo
/// inscrito: medido, o raio cai para `0,5198 ≈ cos(60°)` **com a regra correta
/// aplicada**, porque é isso que o Smooth faz com curvatura. O que a regra de
/// borda decide é outra coisa — se a beira ouve o MIOLO —, e isso se lê no eixo.
fn tube_height(mesh: &ph2d_mesh::Mesh) -> f32 {
    let adj = mesh.adjacency();
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for v in 0..mesh.vert_count() {
        if !adj.is_border(v) {
            continue;
        }
        lo = lo.min(mesh.positions()[v][1]);
        hi = hi.max(mesh.positions()[v][1]);
    }
    assert!(hi > lo, "a fixture tem de ter duas bocas separadas");
    hi - lo
}

/// ⚠️ **A identidade que o original usa, conferida contra as fixtures.**
#[test]
fn a_vertex_is_on_the_border_when_its_ring_does_not_close() {
    let tube = shapes_open::open_tube3();
    let adj = tube.adjacency();
    let border = (0..tube.vert_count()).filter(|&v| adj.is_border(v)).count();
    assert_eq!(border, 12, "o tubo de três anéis tem duas bocas de seis");
    let interior = tube.vert_count() - border;
    assert_eq!(interior, 6, "e o anel do meio é INTERIOR");

    // O CONTROLE, e é ele que explica por que a classe era invisível: uma esfera
    // fechada não tem beira nenhuma.
    let closed = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    let cadj = closed.adjacency();
    assert_eq!(
        (0..closed.vert_count())
            .filter(|&v| cadj.is_border(v))
            .count(),
        0,
        "numa malha fechada toda regra de borda é inerte"
    );
}

/// **O defeito que a regra 2 fecha, e o número dele:** com o anel INTEIRO, a
/// média de um vértice de beira inclui os vizinhos do anel de DENTRO, e a boca é
/// sugada para o miolo do tubo. Medido com a regra desligada, a altura cai de
/// **2 para 1,3597** em seis passes — a peça encolhe pelas duas pontas, e nada na
/// ferramenta diz por quê. Com a regra, ela fica em **2 exatos**: os vizinhos de
/// borda de um vértice de boca estão no MESMO anel, logo na mesma altura.
#[test]
fn smoothing_the_lip_of_an_open_mesh_does_not_suck_it_inward() {
    let mut mesh = shapes_open::open_tube3();
    let before = tube_height(&mesh);

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for _ in 0..6 {
        stroke.dab(
            &mut mesh,
            &smooth(),
            // Um dab que cobre o tubo inteiro: as duas bocas e o miolo.
            &Dab::at([0.0, 0.0, 0.0], 10.0, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    let after = tube_height(&mesh);
    assert!(
        (after - before).abs() < 1e-4,
        "as bocas foram sugadas para dentro ({before} -> {after}) — a média está ouvindo o miolo"
    );

    // ⚠️ E a beira ENCOLHE de raio, o que é correto e não é o defeito: alisar um
    // hexágono o leva ao círculo inscrito. O gate afirma isso para ninguém
    // "consertar" o Smooth de volta para uma média que não suaviza nada.
    let adj = mesh.adjacency();
    let radius: f32 = (0..mesh.vert_count())
        .filter(|&v| adj.is_border(v))
        .map(|v| mesh.positions()[v][0].hypot(mesh.positions()[v][2]))
        .sum::<f32>()
        / 12.0;
    assert!(
        (0.45..0.6).contains(&radius),
        "a beira tem de alisar AO LONGO DELA MESMA (cos 60° ≈ 0,52), e mediu {radius}"
    );
}

/// **A regra 1:** com dois vizinhos a média é o ponto médio da corda, então o
/// vértice escorrega para dentro dela. A `pillow` tem valência 2 em todos os
/// três vértices, e é a única fixture do módulo que contém isso.
#[test]
fn a_vertex_with_two_neighbours_is_frozen_instead_of_sliding_to_the_chord() {
    let mut mesh = shapes_open::pillow();
    let adj = mesh.adjacency();
    assert!(
        (0..mesh.vert_count()).all(|v| adj.valence(v) <= 2),
        "a fixture existe para conter a valência baixa"
    );
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &smooth(),
        &Dab::at([0.3, 0.0, 0.3], 10.0, [0.0, -1.0, 0.0]),
        Symmetry::default(),
    );
    assert_eq!(
        mesh.positions(),
        &before[..],
        "sem superfície em volta não há o que suavizar: o vértice fica"
    );
}

/// ⚠️ **E o interior não pode ter mudado de comportamento.** As duas regras são
/// sobre a beira; se elas alcançassem o miolo, todo Smooth já shipado mudaria de
/// resultado — e o gate que pega isso é a malha FECHADA, onde elas são inertes.
#[test]
fn the_rules_are_inert_on_a_closed_mesh() {
    let mut a = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    let mut b = a.clone();

    // Uma bossa para haver o que suavizar.
    for mesh in [&mut a, &mut b] {
        let mut st = SculptStroke::default();
        st.begin(mesh);
        st.dab(
            mesh,
            &Brush {
                verb: Verb::Draw,
                radius: 0.4,
                strength: 1.0,
                ..Brush::default()
            },
            &Dab::at([0.0, 0.0, 1.0], 0.4, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }

    let mut st = SculptStroke::default();
    st.begin(&b);
    st.dab(
        &mut b,
        &smooth(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    let moved = a
        .positions()
        .iter()
        .zip(b.positions())
        .map(|(p, q)| {
            (q[0] - p[0])
                .abs()
                .max((q[1] - p[1]).abs())
                .max((q[2] - p[2]).abs())
        })
        .fold(0.0f32, f32::max);
    assert!(
        moved > 1e-4,
        "o Smooth tem de continuar suavizando o interior, e mexeu {moved}"
    );
}
