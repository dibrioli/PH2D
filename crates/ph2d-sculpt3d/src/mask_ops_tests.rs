//! Gates das quatro operações de máscara.

use super::*;
use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// Uma esfera com uma calota mascarada **pela porta do produto** — um traço de
/// `Verb::Mask`, não um plano escrito à mão. Uma fixture escrita à mão teria uma
/// borda que o pincel não produz, e o blur mede justamente a borda.
fn masked() -> ph2d_mesh::Mesh {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(48, 72, 1.0);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &Brush {
            verb: Verb::Mask,
            radius: 0.5,
            strength: Verb::Mask.default_strength(),
            ..Brush::default()
        },
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    mesh
}

/// A largura da rampa entre livre e protegido, em vértices — **o que o artista
/// vê como *borda dura* ou *borda macia***.
fn ramp_verts(mesh: &ph2d_mesh::Mesh) -> usize {
    mesh.masks()
        .expect("mascarada")
        .iter()
        .filter(|&&m| (0.05..0.95).contains(&m))
        .count()
}

#[test]
fn clearing_removes_the_plane_instead_of_filling_it_with_zeros() {
    let mut mesh = masked();
    assert!(mesh.masks().is_some());
    assert!(clear(&mut mesh), "havia o que limpar");
    assert!(
        mesh.masks().is_none(),
        "limpar devolve a malha ao estado de quem nunca mascarou"
    );
    // E limpar de novo não aloca nem mente.
    assert!(!clear(&mut mesh), "não havia o que limpar");
    assert!(mesh.masks().is_none());
}

/// ⚠️ **A convenção invertida, no gate.** Aqui `0 = livre`, então inverter é
/// `1 − m`. No SculptGL é o contrário, e trocar o sinal por engano dá uma
/// ferramenta que protege exatamente o que o artista queria esculpir.
#[test]
fn inverting_swaps_free_and_protected() {
    let mesh = masked();
    let before: Vec<f32> = mesh.masks().expect("mascarada").to_vec();
    let mut after = mesh;
    invert(&mut after);
    let got = after.masks().expect("mascarada");
    for (i, (&b, &a)) in before.iter().zip(got).enumerate() {
        assert!((a - (1.0 - b)).abs() < 1e-6, "vértice {i}: {b} -> {a}");
    }
    // O gesto que ela existe para servir: mascarar um pedaço, inverter, esculpir
    // só ali. Numa malha SEM máscara, inverter protege tudo.
    let mut virgin = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    assert!(virgin.masks().is_none());
    invert(&mut virgin);
    assert!(
        virgin
            .masks()
            .expect("materializou")
            .iter()
            .all(|&m| m > 0.999),
        "o inverso de nada protegido é TUDO protegido"
    );
}

#[test]
fn blurring_softens_the_edge_and_sharpening_puts_it_back() {
    let mut mesh = masked();
    let sharp = ramp_verts(&mesh);
    assert!(sharp > 0, "a fixture tem de ter uma borda: {sharp}");

    blur(&mut mesh, 8);
    let soft = ramp_verts(&mesh);
    assert!(
        soft > sharp,
        "borrar tem de ALARGAR a rampa: {sharp} -> {soft}"
    );

    sharpen(&mut mesh, 8);
    let back = ramp_verts(&mesh);
    assert!(
        back < soft,
        "afiar tem de estreitá-la de volta: {soft} -> {back}"
    );
}

/// Zero passos é zero trabalho, e uma malha sem máscara não ganha uma por ser
/// borrada — senão *borrar nada* alocaria 4 B/vértice para escrever zeros.
#[test]
fn a_no_op_pass_allocates_nothing_and_changes_nothing() {
    let mut mesh = masked();
    let before: Vec<f32> = mesh.masks().expect("mascarada").to_vec();
    blur(&mut mesh, 0);
    assert_eq!(mesh.masks().expect("mascarada"), &before[..]);

    let mut virgin = ph2d_mesh::shapes::uv_sphere(16, 24, 1.0);
    blur(&mut virgin, 4);
    sharpen(&mut virgin, 4);
    assert!(
        virgin.masks().is_none(),
        "borrar nenhuma máscara não materializa um plano"
    );
}

/// ⚠️ **O passo lê o estado do INÍCIO do passo.** Com Gauss-Seidel (ler o que o
/// próprio laço acabou de escrever) o resultado passa a depender da ORDEM dos
/// vértices — que é a ordem do ARQUIVO — e a mesma máscara borrada num OBJ
/// reordenado sairia diferente. O oráculo é a simetria: uma calota centrada no
/// polo é simétrica sob a reflexão que troca os dois lados dela, e um passe
/// dependente de ordem quebra essa simetria.
#[test]
fn a_blur_pass_does_not_depend_on_vertex_order() {
    let mut mesh = masked();
    blur(&mut mesh, 6);
    let m = mesh.masks().expect("mascarada");
    let pos = mesh.positions();

    // Para cada vértice, o espelho em X é outro vértice da mesma esfera UV; a
    // máscara (uma calota em +Z) tem de valer o mesmo nos dois.
    let mut checked = 0;
    let mut worst = 0.0f32;
    for (i, p) in pos.iter().enumerate() {
        if p[0].abs() < 1e-4 {
            continue;
        }
        let mirror = [-p[0], p[1], p[2]];
        let Some(j) = pos.iter().position(|q| {
            (q[0] - mirror[0]).abs() < 1e-4
                && (q[1] - mirror[1]).abs() < 1e-4
                && (q[2] - mirror[2]).abs() < 1e-4
        }) else {
            continue;
        };
        worst = worst.max((m[i] - m[j]).abs());
        checked += 1;
    }
    assert!(
        checked > 200,
        "a fixture tem de ter pares espelhados: {checked}"
    );
    assert!(
        worst < 1e-5,
        "a máscara borrada perdeu a simetria da cena por {worst} — o passe está lendo o que ele mesmo escreveu"
    );
}
