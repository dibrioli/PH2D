//! Gates da malha.

use super::*;
use crate::shapes;

#[test]
fn a_cube_is_six_quads_and_twelve_triangles() {
    let m = shapes::cube(2.0);
    assert_eq!(m.vert_count(), 8);
    assert_eq!(m.face_count(), 6);
    assert_eq!(
        m.triangle_count(),
        12,
        "a contagem de TRIÂNGULOS não é a de faces quando há quads — é ela que os tetos por tier falam"
    );
    let mut tris = Vec::new();
    m.triangle_indices(&mut tris);
    assert_eq!(tris.len(), 12);
}

/// Num sólido convexo centrado na origem, a normal de cada vértice aponta para
/// fora ⇔ `dot(normal, posição) > 0`. Este é o gate da CONVENÇÃO de winding, e
/// ele falha se a fórmula de Newell inverter o sinal — o que a renderização
/// mostraria como um objeto iluminado por dentro.
#[test]
fn the_normals_of_a_convex_solid_point_outward() {
    for m in [shapes::cube(2.0), shapes::uv_sphere(12, 16, 1.0)] {
        for (v, n) in m.normals().iter().enumerate() {
            let p = m.positions()[v];
            let dot = p[0] * n[0] + p[1] * n[1] + p[2] * n[2];
            assert!(
                dot > 0.0,
                "a normal do vértice {v} aponta para dentro ({dot})"
            );
        }
        for (f, n) in m.face_normals().iter().enumerate() {
            let c = crate::normals::face_normal(m.positions(), m.faces()[f]);
            assert_eq!(*n, c, "a normal guardada divergiu da porta que a calcula");
        }
    }
}

/// As normais de uma esfera unitária são a própria posição normalizada. É o
/// oráculo ANALÍTICO — independente do código, ao contrário de comparar a saída
/// com ela mesma.
#[test]
fn a_unit_spheres_normals_are_its_own_positions() {
    let m = shapes::uv_sphere(24, 32, 1.0);
    for (v, n) in m.normals().iter().enumerate() {
        let p = m.positions()[v];
        for k in 0..3 {
            assert!(
                (n[k] - p[k]).abs() < 0.02,
                "vértice {v}: normal {n:?} contra posição {p:?}"
            );
        }
    }
}

/// `rebuild` é idempotente — chamá-lo duas vezes dá o mesmo resultado. Sem
/// isso, um dab que reconstrói derivaria a cada movimento do mouse.
#[test]
fn rebuild_is_idempotent() {
    let mut m = shapes::uv_sphere(10, 14, 1.0);
    let n0 = m.normals().to_vec();
    let a0 = m.adjacency().clone();
    let nodes0 = m.octree().node_count();
    m.rebuild();
    assert_eq!(m.normals(), &n0[..]);
    assert_eq!(m.adjacency(), &a0);
    assert_eq!(m.octree().node_count(), nodes0);
}

/// A consulta de esfera contra a **força bruta**, que é um oráculo
/// independente: ela não sabe que existe um octree.
#[test]
fn the_sphere_query_agrees_with_brute_force() {
    let m = shapes::uv_sphere(18, 24, 1.0);
    let mut scratch = QueryScratch::default();
    let mut got = Vec::new();
    for (center, radius) in [
        ([0.0, 1.0, 0.0], 0.5),  // o polo
        ([1.0, 0.0, 0.0], 0.3),  // o equador
        ([0.7, 0.7, 0.0], 0.25), // entre os dois
        ([0.0, 0.0, 0.0], 2.0),  // engole tudo
        ([5.0, 5.0, 5.0], 0.1),  // não pega nada
    ] {
        m.verts_in_sphere(center, radius, &mut scratch, &mut got);
        got.sort_unstable();

        let r2 = radius * radius;
        let mut want: Vec<u32> = (0..m.vert_count() as u32)
            .filter(|&v| {
                let p = m.positions()[v as usize];
                let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
                d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= r2
            })
            .collect();
        want.sort_unstable();

        assert_eq!(got, want, "consulta em {center:?} r={radius}");
    }
}

/// O scratch é reusado entre consultas, e a época tem de isolar uma da outra.
/// Uma época que não avança faz a segunda consulta devolver vazio.
#[test]
fn a_reused_scratch_does_not_leak_between_queries() {
    let m = shapes::uv_sphere(10, 12, 1.0);
    let mut scratch = QueryScratch::default();
    let mut a = Vec::new();
    let mut b = Vec::new();
    m.verts_in_sphere([0.0, 1.0, 0.0], 0.6, &mut scratch, &mut a);
    m.verts_in_sphere([0.0, 1.0, 0.0], 0.6, &mut scratch, &mut b);
    assert!(!a.is_empty());
    assert_eq!(a, b, "a mesma consulta duas vezes tem de dar o mesmo");
    assert!(scratch.capacity_bytes() > 0);
}

/// ⚠️ **Limite MEDIDO e documentado, não um bug:** a consulta acha vértices
/// através das FACES, então um vértice sem face nenhuma é invisível para ela.
/// É a resposta certa para escultura (não há superfície a mover), e está aqui
/// para ninguém "consertar" isso e pagar uma varredura linear por dab.
#[test]
fn a_faceless_vertex_is_invisible_to_the_sphere_query() {
    let m = Mesh::from_parts(
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.1, 0.1, 0.0],
        ],
        vec![Face::tri(0, 1, 2)],
    )
    .unwrap();
    let mut scratch = QueryScratch::default();
    let mut out = Vec::new();
    m.verts_in_sphere([0.1, 0.1, 0.0], 0.05, &mut scratch, &mut out);
    assert!(
        out.is_empty(),
        "o vértice solto 3 não devia aparecer: {out:?}"
    );
}

/// Cor e máscara são preguiçosas: não existem até alguém escrever nelas.
#[test]
fn colour_and_mask_are_not_allocated_until_touched() {
    let mut m = shapes::cube(1.0);
    assert!(m.colors().is_none());
    assert!(m.masks().is_none());
    m.colors_mut()[0] = [1.0, 0.0, 0.0];
    assert_eq!(m.colors().unwrap().len(), m.vert_count());
    assert!(
        m.masks().is_none(),
        "tocar a cor não pode materializar a máscara"
    );
    m.masks_mut()[1] = 1.0;
    assert_eq!(m.masks().unwrap()[1], 1.0);
    assert_eq!(m.masks().unwrap()[0], DEFAULT_MASK);
}

/// Um índice fora de alcance é recusado na porta. Sem isto, ele vira leitura
/// errada em cada kernel, e o sintoma aparece a três waves de distância.
#[test]
fn an_out_of_range_index_is_refused_at_the_door() {
    let e = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 9)],
    )
    .unwrap_err();
    assert_eq!(
        e,
        MeshError::VertexOutOfRange {
            face: 0,
            vertex: 9,
            vert_count: 3
        }
    );
}

/// O sentinela de triângulo NÃO conta como índice fora de alcance — se
/// contasse, nenhuma malha de triângulos poderia ser construída.
#[test]
fn the_triangle_sentinel_is_not_mistaken_for_an_index() {
    let m = Mesh::from_parts(
        vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![Face::tri(0, 1, 2)],
    );
    assert!(m.is_ok());
}
