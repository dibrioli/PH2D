//! Os gates da topologia dinâmica.
//!
//! ⚠️ **O oráculo central é a AUSÊNCIA DE RACHADURA**, e ele não é sobre
//! contagem: uma malha fechada refinada continua fechada, e isso se verifica
//! contando as faces de cada aresta — `2` no interior. Um refino que esquecesse
//! a vizinha deixaria arestas de valência `1` no meio da superfície, que é
//! precisamente o T-vértice, e a contagem de vértices estaria *certa*.

use super::*;
use crate::{Face, Mesh, shapes};

/// Uma esfera triangulada — a fixture do produto (o `uv_sphere` nasce em quads,
/// e é por isso que entrar em dyntopo triangula).
fn tri_sphere(rings: usize, segs: usize) -> Mesh {
    let mut m = shapes::uv_sphere(rings, segs, 1.0);
    m.triangulate();
    m
}

/// Quantas arestas do interior ficaram com valência ≠ 2 — **zero é o contrato**.
fn cracks(m: &Mesh) -> usize {
    let e = m.edges();
    (0..e.len() as u32).filter(|&i| e.valence(i) != 2).count()
}

#[test]
fn the_refined_mesh_has_no_cracks() {
    let mut m = tri_sphere(12, 18);
    assert_eq!(cracks(&m), 0, "o controle: a esfera nasce fechada");

    let target = edge_target(0.6, 1.0);
    let r = refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.6, target);
    assert!(
        matches!(r, Refine::Done { .. }),
        "algo tem de ser partido: {r:?}"
    );
    assert_eq!(
        cracks(&m),
        0,
        "toda face que toca uma aresta partida aprendeu o vértice do meio"
    );
}

/// **O PADRÃO É TOTAL** — e este é o gate que de fato prova a ausência de
/// rachadura, depois de a mutação ter mostrado que a varredura não era ela.
///
/// Para cada um dos **oito** subconjuntos de arestas partidas, os triângulos
/// emitidos têm de formar exatamente o mesmo pedaço de superfície: toda aresta
/// interior aparece **duas vezes, em sentidos opostos** (é isso que uma malha
/// sem rachadura É, localmente) e a fronteira é o perímetro original com os
/// meios inseridos. Uma face girada errado inverte um par e o multiconjunto
/// acusa.
#[test]
fn every_split_pattern_tiles_the_triangle_without_a_crack() {
    let v = [0u32, 1, 2];
    let mids = [3u32, 4, 5]; // meio de (0,1), de (1,2), de (2,0)
    for bits in 0u8..8 {
        let mid = [
            (bits & 1 != 0).then_some(mids[0]),
            (bits & 2 != 0).then_some(mids[1]),
            (bits & 4 != 0).then_some(mids[2]),
        ];
        let mut out = Vec::new();
        emit_triangle(&mut out, v, mid);
        let n = mid.iter().filter(|m| m.is_some()).count();
        assert_eq!(
            out.len(),
            [1, 2, 3, 4][n],
            "padrão {bits:03b}: número de faces"
        );

        // O multiconjunto de arestas DIRIGIDAS.
        let mut dir: Vec<(u32, u32)> = Vec::new();
        for f in &out {
            assert!(f.is_tri(), "padrão {bits:03b}: só triângulos");
            let t = f.verts();
            dir.push((t[0], t[1]));
            dir.push((t[1], t[2]));
            dir.push((t[2], t[0]));
        }
        // Fronteira esperada: o perímetro 0→1→2→0 com os meios inseridos.
        let mut border: Vec<(u32, u32)> = Vec::new();
        for k in 0..3 {
            let (a, b) = (v[k], v[(k + 1) % 3]);
            match mid[k] {
                Some(m) => border.extend([(a, m), (m, b)]),
                None => border.push((a, b)),
            }
        }
        for e in &border {
            assert!(
                dir.contains(e),
                "padrão {bits:03b}: a fronteira {e:?} tem de estar na saída"
            );
        }
        // Toda aresta que não é da fronteira aparece nos DOIS sentidos.
        for &(a, b) in &dir {
            if border.contains(&(a, b)) {
                continue;
            }
            assert!(
                dir.contains(&(b, a)),
                "padrão {bits:03b}: a aresta interior ({a},{b}) não tem par — \
                 é exatamente a forma de uma rachadura, ou de uma face invertida"
            );
        }
        // E nenhum meio dado fica de fora: um padrão que ignore um meio deixa o
        // T-vértice de volta, com a contagem de faces certa.
        for m in mid.into_iter().flatten() {
            assert!(
                dir.iter().any(|&(a, b)| a == m || b == m),
                "padrão {bits:03b}: o meio {m} não foi usado"
            );
        }
    }
}

#[test]
fn the_refinement_stays_inside_the_brush() {
    let mut m = tri_sphere(12, 18);
    let before = m.vert_count();
    // O dab no polo NORTE não pode adensar o polo sul.
    let centre = [0.0, 0.0, 1.0];
    let radius = 0.5f32;
    refine_in_sphere(&mut m, centre, radius, edge_target(radius, 1.0));

    // ⚠️ O oráculo é a POSIÇÃO dos vértices novos, não a contagem: contar
    // provaria que cresceu, e o que está em julgamento é ONDE.
    let far = m.positions()[before..]
        .iter()
        .filter(|p| {
            let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > radius * 1.35
        })
        .count();
    assert_eq!(far, 0, "nenhum vértice novo nasceu longe do pincel");
    assert!(m.vert_count() > before, "e alguma coisa nasceu");
}

#[test]
fn refining_twice_is_deterministic() {
    let run = || {
        let mut m = tri_sphere(10, 14);
        refine_in_sphere(&mut m, [0.3, 0.2, 0.9], 0.7, edge_target(0.7, 0.8));
        m
    };
    let (a, b) = (run(), run());
    assert_eq!(a.vert_count(), b.vert_count());
    assert_eq!(
        a.faces(),
        b.faces(),
        "os índices são os MESMOS, não só a forma"
    );
    assert_eq!(a.positions(), b.positions());
}

#[test]
fn the_edge_target_is_the_references_formula() {
    // Os dois extremos, conferidos contra a aritmética do `SculptBase.js`.
    let r = 2.0f32;
    assert!((edge_target(r, 0.0) - r * 0.22f32.sqrt()).abs() < 1e-6);
    assert!((edge_target(r, 1.0) - r * 0.02f32.sqrt()).abs() < 1e-6);
    // Mais detalhe ⇒ aresta MENOR. A direção é metade do contrato: invertê-la
    // deixa o slider vivo e o significado ao contrário.
    assert!(edge_target(r, 1.0) < edge_target(r, 0.5));
    assert!(edge_target(r, 0.5) < edge_target(r, 0.0));
    // Fora da faixa não estoura — o clamp é da porta, não do chamador.
    assert_eq!(edge_target(r, 2.0), edge_target(r, 1.0));
    assert_eq!(edge_target(r, -1.0), edge_target(r, 0.0));
}

#[test]
fn refining_shortens_the_edges_it_was_asked_about() {
    let mut m = tri_sphere(8, 12);
    let (centre, radius) = ([0.0, 0.0, 1.0], 0.6f32);
    let target = edge_target(radius, 1.0);

    let long_before = long_edges_near(&m, centre, radius, target);
    assert!(long_before > 0, "a fixture TEM de conter o fenômeno");

    refine_in_sphere(&mut m, centre, radius, target);
    let long_after = long_edges_near(&m, centre, radius, target);
    assert!(
        long_after < long_before,
        "o refino encurtou as arestas do dab: {long_before} -> {long_after}"
    );
}

/// Arestas mais longas que `target` cujo meio cai na esfera — a mesma pergunta
/// que o motor faz, para o gate medir o que ele decide.
fn long_edges_near(m: &Mesh, c: [f32; 3], r: f32, target: f32) -> usize {
    let mut n = 0;
    for (fi, f) in m.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let Some(e) = m.edges().face_edge(fi, k) else {
                continue;
            };
            // Cada aresta é vista por duas faces; conta só uma vez.
            let _ = e;
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if a > b {
                continue;
            }
            let (pa, pb) = (m.positions()[a as usize], m.positions()[b as usize]);
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let mid = [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ];
            let dm = [mid[0] - c[0], mid[1] - c[1], mid[2] - c[2]];
            if len > target && (dm[0] * dm[0] + dm[1] * dm[1] + dm[2] * dm[2]).sqrt() <= r {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn a_quad_mesh_is_refused_instead_of_mangled() {
    let mut m = shapes::uv_sphere(8, 12, 1.0);
    assert!(
        m.faces().iter().any(|f| !f.is_tri()),
        "o controle: há quads"
    );
    let r = refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.6, 0.05);
    assert_eq!(r, Refine::NotTriangles);
    assert_eq!(m.face_count(), 8 * 12, "e a malha não foi tocada");
}

#[test]
fn triangulating_is_idempotent_and_moves_no_vertex() {
    let mut m = shapes::uv_sphere(6, 8, 1.0);
    let pos = m.positions().to_vec();
    let added = m.triangulate();
    assert!(added > 0);
    assert_eq!(m.positions(), &pos[..], "triangular não move um vértice");
    assert!(m.faces().iter().all(Face::is_tri));
    assert_eq!(m.triangulate(), 0, "a segunda chamada é um no-op");
}

#[test]
fn the_new_vertices_carry_colour_and_mask() {
    let mut m = tri_sphere(8, 12);
    // Pinta o hemisfério norte e mascara-o pela metade: os canais têm de
    // interpolar, não de nascer no default.
    let n = m.vert_count();
    m.put_masks(vec![0.25; n]);
    for (i, c) in m.colors_mut().iter_mut().enumerate() {
        *c = if m_pos_z(i) {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
    }
    let before = m.vert_count();
    refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.6, edge_target(0.6, 1.0));
    assert!(m.vert_count() > before);

    let masks = m.masks().expect("o plano sobrevive ao refino");
    assert_eq!(masks.len(), m.vert_count(), "e mede a malha nova");
    assert!(
        masks[before..].iter().all(|&v| (v - 0.25).abs() < 1e-5),
        "o meio de dois 0,25 é 0,25 — um vértice novo em zero apagaria a máscara ali"
    );
    let colors = m.colors().expect("a cor também");
    assert_eq!(colors.len(), m.vert_count());
}

/// Um `i` par para a fixture de cor — só precisa ser determinístico.
fn m_pos_z(i: usize) -> bool {
    i.is_multiple_of(2)
}

#[test]
fn an_empty_dab_changes_nothing() {
    let mut m = tri_sphere(8, 12);
    let before = (m.vert_count(), m.face_count());
    // Longe da malha.
    assert_eq!(
        refine_in_sphere(&mut m, [10.0, 10.0, 10.0], 0.5, 0.01),
        Refine::Enough
    );
    // Alvo maior que qualquer aresta.
    assert_eq!(
        refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.5, 100.0),
        Refine::Enough
    );
    // Alvo degenerado: recusa em vez de pedir infinitos vértices.
    assert_eq!(
        refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.5, 0.0),
        Refine::Enough
    );
    assert_eq!((m.vert_count(), m.face_count()), before);
}

#[test]
fn the_midpoint_follows_the_curve_instead_of_flattening_it() {
    // Numa esfera de raio 1 o meio geométrico de uma aresta cai DENTRO; o
    // deslocamento pela normal o traz de volta para perto da superfície.
    let mut m = tri_sphere(8, 12);
    let before = m.vert_count();
    refine_in_sphere(&mut m, [0.0, 0.0, 1.0], 0.8, edge_target(0.8, 1.0));

    let radii: Vec<f32> = m.positions()[before..]
        .iter()
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .collect();
    assert!(!radii.is_empty());
    let worst = radii.iter().fold(1.0f32, |acc, &r| acc.min(r));
    assert!(
        worst > 0.985,
        "o pior vértice novo ficou a {worst} do centro — o refino está achatando a esfera"
    );
}
