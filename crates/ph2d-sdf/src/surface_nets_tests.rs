//! Gates da extração.
//!
//! O oráculo é a **forma**: uma esfera voxelizada e re-extraída continua sendo
//! uma esfera, e é isso que um artista julga. Contagem de vértices não separa
//! uma superfície correta de uma explodida.

use super::*;
use ph2d_mesh::shapes;

fn sphere_surface(res: u32) -> Mesh {
    let m = shapes::uv_sphere(24, 32, 1.0);
    let mut f = VoxelField::for_bounds(m.bounds(), res);
    f.voxelize(&m);
    f.flood_fill();
    surface_nets(&f).expect("a extração devolveu índices fora da malha")
}

#[test]
fn a_sphere_comes_back_as_a_sphere() {
    let out = sphere_surface(40);
    assert!(
        out.vert_count() > 500,
        "saiu com {} vértices",
        out.vert_count()
    );

    let mut worst = 0.0f32;
    for p in out.positions() {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        worst = worst.max((r - 1.0).abs());
    }
    // Um passo de grade a 40 células no maior lado vale ~0.05; a superfície de
    // nível zero fica dentro de meio passo do lugar certo.
    assert!(worst < 0.06, "o raio erra por {worst}");
}

/// ⚠️ O gate que importa mais que a forma: uma malha de escultura **fechada**.
/// Aresta de valência 1 é beira, e uma beira num remesh é o pincel atravessando
/// o barro por um buraco que ninguém desenhou.
#[test]
fn the_extracted_surface_is_closed() {
    let out = sphere_surface(32);
    let edges = out.edges();
    let borders = (0..edges.len() as u32)
        .filter(|e| edges.valence(*e) == 1)
        .count();
    assert_eq!(borders, 0, "{borders} arestas de beira");
}

/// A orientação é o que separa uma malha de uma malha do avesso, e nenhum gate
/// de contagem a enxerga — a sombra do matcap sai errada e mais nada.
#[test]
fn every_face_looks_outward() {
    let out = sphere_surface(32);
    let pos = out.positions();
    let mut inverted = 0usize;
    for (f, face) in out.faces().iter().enumerate() {
        let n = out.face_normals()[f];
        let vs = face.verts();
        let mut c = [0.0f32; 3];
        for v in vs {
            let p = pos[*v as usize];
            for a in 0..3 {
                c[a] += p[a] / vs.len() as f32;
            }
        }
        // Numa esfera centrada na origem, "para fora" é "na direção do próprio
        // centroide" — um oráculo que não conhece a convenção do extrator.
        if c[0] * n[0] + c[1] * n[1] + c[2] * n[2] < 0.0 {
            inverted += 1;
        }
    }
    assert_eq!(
        inverted,
        0,
        "{inverted} faces de {} viradas",
        out.face_count()
    );
}

/// A saída é uma grade deformada: quads em quase toda parte é o que faz esta
/// malha subdividir bem, e é o motivo de o Surface Nets vir antes do marching
/// cubes.
#[test]
fn the_output_is_made_of_quads() {
    let out = sphere_surface(32);
    let tris = out.faces().iter().filter(|f| f.is_tri()).count();
    assert_eq!(tris, 0, "{tris} triângulos numa saída que devia ser quads");
}

#[test]
fn a_finer_grid_gives_a_denser_mesh() {
    let coarse = sphere_surface(16).vert_count();
    let fine = sphere_surface(32).vert_count();
    // Dobrar a resolução quadruplica a superfície, então a contagem cresce com o
    // QUADRADO — pedir só "mais" deixaria passar um extrator que ignora a grade.
    let ratio = fine as f32 / coarse as f32;
    assert!(
        (2.5..=5.5).contains(&ratio),
        "razão {ratio} entre {fine} e {coarse}"
    );
}
