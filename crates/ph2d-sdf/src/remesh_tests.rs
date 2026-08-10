//! Gates do botão inteiro — a porta que o shell chama.

use super::*;
use ph2d_mesh::{shapes, shapes_open};

#[test]
fn remeshing_a_sphere_keeps_the_sphere_and_reports_what_it_did() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let (out, report) = remesh(&m, 40).expect("remesh");

    assert_eq!(report.verts.0, m.vert_count());
    assert_eq!(report.verts.1, out.vert_count());
    assert_eq!(report.holes_filled, 0, "a esfera já é fechada");
    assert!(report.cells > 0);

    let mut worst = 0.0f32;
    for p in out.positions() {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        worst = worst.max((r - 1.0).abs());
    }
    assert!(worst < 0.06, "o raio erra por {worst}");
}

/// ⚠️ Sem tapar o buraco **não haveria dentro**, e o remesh devolveria uma malha
/// vazia. Este é o gate que prende o passo 1 ao resultado.
#[test]
fn an_open_mesh_still_comes_back_as_a_body() {
    let m = shapes_open::open_tube3();
    let (out, report) = remesh(&m, 24).expect("remesh");
    assert!(report.holes_filled > 0, "não achou buraco no tubo aberto");
    assert!(out.vert_count() > 0, "voltou vazio");

    let edges = out.edges();
    let borders = (0..edges.len() as u32)
        .filter(|e| edges.valence(*e) == 1)
        .count();
    assert_eq!(borders, 0, "a saída tem {borders} arestas de beira");
}

/// ⚠️ **A entrada NÃO é modificada.** Tapar buracos é exigência do algoritmo, e
/// se ela vazasse para a malha do artista o remesh estaria editando geometria
/// que ninguém pediu — e o Ctrl+Z de fora não saberia disso.
#[test]
fn the_input_mesh_is_left_alone() {
    let m = shapes_open::open_tube3();
    let before = (m.vert_count(), m.face_count());
    let _ = remesh(&m, 16).expect("remesh");
    assert_eq!((m.vert_count(), m.face_count()), before);
}

/// Duas peças separadas entram, duas peças separadas saem — a voxelização é a
/// união, não um casco.
#[test]
fn two_separate_bodies_stay_separate() {
    let mut positions = shapes::cube(1.0).positions().to_vec();
    let faces_a = shapes::cube(1.0).faces().to_vec();
    let n = positions.len() as u32;
    let mut faces = faces_a.clone();
    for p in shapes::cube(1.0).positions() {
        positions.push([p[0] + 2.5, p[1], p[2]]);
    }
    for f in &faces_a {
        let v = f.0;
        faces.push(ph2d_mesh::Face::quad(
            v[0] + n,
            v[1] + n,
            v[2] + n,
            v[3] + n,
        ));
    }
    let both = Mesh::from_parts(positions, faces).expect("dois cubos");

    let (out, _) = remesh(&both, 40).expect("remesh");
    // Os dois cubos ficam a 2.5 de distância com lado 1: se a grade os tivesse
    // fundido, não haveria vértice nenhum na faixa entre eles.
    let left = out.positions().iter().filter(|p| p[0] < 0.8).count();
    let right = out.positions().iter().filter(|p| p[0] > 1.7).count();
    let between = out
        .positions()
        .iter()
        .filter(|p| (0.8..=1.7).contains(&p[0]))
        .count();
    assert!(left > 0 && right > 0, "esquerda {left}, direita {right}");
    assert_eq!(between, 0, "{between} vértices no vão entre os dois cubos");
}

/// **Um remesh nunca reporta SUCESSO com uma malha vazia.**
///
/// ⚠️ **As resoluções são MEDIDAS, não escolhidas.** Entre 100 e 200 há ONZE em
/// que o flood fill vaza para dentro e o campo sai sem interior — `112, 151,
/// 160, 161, 168, 180, 181, 193, 194, 196, 197` — e o default que shipa é
/// **150**, a UMA unidade da primeira. O que decide não é a resolução e sim o
/// alinhamento da grade contra os triângulos, então outra malha (outra caixa,
/// outro `step`) vaza noutros números: o 150 não é seguro, é sortudo.
///
/// ⚠️ **E o irmão [`an_open_mesh_still_comes_back_as_a_body`] já afirmava esta
/// mesma propriedade** — na resolução 24 de um tubo aberto, onde ela passa. A
/// fixture dele não continha o fenômeno; este é o mesmo `assert` com uma que
/// contém.
///
/// A afirmação é sobre o RESULTADO, não sobre o vazamento: curar o flood fill
/// faz `remesh` devolver uma malha de verdade nestas resoluções e o gate segue
/// verde — ele não pode ser silenciado pelo conserto, só pela regressão.
#[test]
fn a_remesh_never_reports_success_with_an_empty_mesh() {
    let m = shapes::uv_sphere(96, 144, 1.0);
    for res in [151u32, 320] {
        if let Ok((out, report)) = remesh(&m, res) {
            assert!(
                out.vert_count() > 0,
                "resolução {res}: `Ok` com ZERO vértices — o chamador instala isto \
                 e a escultura do artista SOME da tela com log de sucesso ({report:?})"
            );
        }
    }
}

#[test]
fn the_default_resolution_is_the_references() {
    assert_eq!(DEFAULT_RESOLUTION, 150);
    // E o atalho é o mesmo botão: uma segunda porta com um default próprio é
    // como os dois passam a reconstruir malhas diferentes.
    let m = shapes::cube(1.0);
    let (a, _) = remesh_default(&m).expect("default");
    let (b, _) = remesh(&m, DEFAULT_RESOLUTION).expect("explícito");
    assert_eq!(a.vert_count(), b.vert_count());
}
