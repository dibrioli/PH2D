//! O CENSO das fixtures malformadas.
//!
//! ⚠️ **Estes gates não testam um algoritmo: eles testam que a FIXTURE contém o
//! fenômeno.** É a inversão que este módulo existe para pagar — as três curas da
//! W4 (borda do laplaciano, valência congelada, normal fabricada) são
//! indistinguíveis de não-cura sobre uma malha bem formada, então o gate que
//! pode falhar depende de a fixture estar certa. Se alguém editar uma destas
//! malhas e ela deixar de conter o defeito, é AQUI que aparece — e não três
//! waves adiante, num gate que passou a ser verde por vácuo.
//!
//! ⚠️ **A regra de borda é a do original** (`Mesh.js`, dentro do build do anel):
//! `nº de faces do vértice != nº de vizinhos únicos`. Ela é computada AQUI, no
//! gate, e não como API de produto: o `vertOnEdge` é da W6.0, e antecipá-lo por
//! conveniência de teste seria construir metade de uma wave sem o consumidor.

use super::*;
use crate::shapes;

/// A regra de borda do SculptGL, computada a partir dos dois CSRs que já temos.
fn is_border(m: &Mesh, v: usize) -> bool {
    let faces = m.adjacency().vert_faces.neighbours(v).len();
    let verts = m.adjacency().vert_verts.neighbours(v).len();
    faces != verts
}

fn border_count(m: &Mesh) -> usize {
    (0..m.vert_count()).filter(|&v| is_border(m, v)).count()
}

/// Vértices que o original **congela** no Smooth (`vcount <= 2`), ignorando o
/// vértice solto — que é outro fenômeno, com outra cura.
fn low_valence_count(m: &Mesh) -> usize {
    (0..m.vert_count())
        .filter(|&v| {
            let n = m.adjacency().vert_verts.neighbours(v).len();
            n > 0 && n <= 2
        })
        .count()
}

/// O Newell **CRU** de uma face: a direção é a normal, o comprimento é
/// proporcional à ÁREA.
///
/// Não é uma segunda cópia do [`crate::normals::face_normal`] — é a grandeza que
/// aquela função descarta ao normalizar, e é justamente a que separa a nossa
/// média das normais da média ponderada por área do original.
fn raw_newell(positions: &[[f32; 3]], face: Face) -> [f32; 3] {
    let vs = face.verts();
    let n = vs.len();
    let mut acc = [0.0f32; 3];
    for i in 0..n {
        let a = positions[vs[i] as usize];
        let b = positions[vs[(i + 1) % n] as usize];
        acc[0] += (a[1] - b[1]) * (a[2] + b[2]);
        acc[1] += (a[2] - b[2]) * (a[0] + b[0]);
        acc[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    acc
}

fn len2(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// As faces cuja área é zero — o MESMO limiar que o `normalize` usa para
/// decidir fabricar `+Y`, para o censo e o defeito falarem do mesmo conjunto.
fn degenerate_faces(m: &Mesh) -> Vec<usize> {
    (0..m.face_count())
        .filter(|&f| len2(raw_newell(m.positions(), m.faces()[f])) <= f32::MIN_POSITIVE)
        .collect()
}

/// A normal do vértice **ponderada por área** — o que o original computa por
/// somar o Newell cru e deixar o shader normalizar.
fn area_weighted_normal(m: &Mesh, v: usize) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    for &fi in m.adjacency().vert_faces.neighbours(v) {
        let n = raw_newell(m.positions(), m.faces()[fi as usize]);
        acc[0] += n[0];
        acc[1] += n[1];
        acc[2] += n[2];
    }
    let inv = 1.0 / len2(acc).sqrt();
    [acc[0] * inv, acc[1] * inv, acc[2] * inv]
}

fn angle_deg(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    d.acos().to_degrees()
}

/// O maior desvio entre a nossa normal e a ponderada por área, sobre a malha.
fn worst_weighting_divergence(m: &Mesh) -> f32 {
    (0..m.vert_count())
        .map(|v| angle_deg(m.normals()[v], area_weighted_normal(m, v)))
        .fold(0.0f32, f32::max)
}

// ---------------------------------------------------------------------------
// O CONTROLE — sem ele o censo não prova nada
// ---------------------------------------------------------------------------

/// ⚠️ **A metade que torna o resto um achado.** Se as malhas bem formadas também
/// contassem borda ou face degenerada, os contadores acima estariam medindo
/// outra coisa — e as fixtures novas seriam decoração.
#[test]
fn the_well_formed_shapes_contain_none_of_these_phenomena() {
    let sizes = [(5, 8), (12, 16), (32, 48)];
    let mut meshes = vec![shapes::cube(1.0), shapes::cube(2.0)];
    for (r, s) in sizes {
        meshes.push(shapes::uv_sphere(r, s, 1.0));
    }
    for (i, m) in meshes.iter().enumerate() {
        assert_eq!(border_count(m), 0, "malha {i}: apareceu vértice de borda");
        assert_eq!(low_valence_count(m), 0, "malha {i}: valência <= 2");
        assert_eq!(
            degenerate_faces(m).len(),
            0,
            "malha {i}: apareceu face degenerada"
        );
    }
}

// ---------------------------------------------------------------------------
// O CENSO, uma fixture por fenômeno
// ---------------------------------------------------------------------------

#[test]
fn the_open_tube_has_a_border_that_touches_an_interior() {
    let m = open_tube3();
    assert_eq!(m.vert_count(), 18);
    assert_eq!(m.face_count(), 12);
    assert_eq!(border_count(&m), 12, "os dois anéis das pontas");
    assert_eq!(
        m.vert_count() - border_count(&m),
        6,
        "o anel do MEIO é o interior, e é ele que faz a regra de borda morder"
    );
    assert_eq!(
        low_valence_count(&m),
        0,
        "o tubo não é a fixture de valência"
    );
    assert_eq!(degenerate_faces(&m).len(), 0);
}

/// **A fixture da BORDA CURVA** — o censo dela, e o número que a torna
/// necessária.
///
/// ⚠️ **A última asserção é a razão de a fixture existir:** ela mede o ângulo
/// entre a normal da SUPERFÍCIE e a **bissetriz das arestas de beira**, que é a
/// troca que o `SlideRelax` faz numa borda. No [`open_tube3`] esse ângulo é
/// **0,015°** (medido) — as duas coincidem e um gate ali passaria com a
/// bissetriz apagada. Aqui elas são ORTOGONAIS.
#[test]
fn the_open_disc_has_a_curved_border_whose_bisector_is_not_the_surface_normal() {
    let m = open_disc();
    assert_eq!(m.vert_count(), 19);
    assert_eq!(m.face_count(), 24);
    assert_eq!(border_count(&m), 12, "só o anel de fora é beira");
    assert_eq!(
        m.vert_count() - border_count(&m),
        7,
        "o centro e o anel do meio são o interior que faz a regra de borda morder"
    );
    assert_eq!(
        low_valence_count(&m),
        0,
        "o disco não é a fixture de valência"
    );
    assert_eq!(degenerate_faces(&m).len(), 0);

    let adj = m.adjacency();
    let mut worst_deg: f64 = 0.0;
    for v in 0..m.vert_count() {
        if !adj.is_border(v) {
            continue;
        }
        let at = m.positions()[v];
        let mut acc = [0.0f64; 3];
        let mut n = 0;
        for &nb in adj.vert_verts.neighbours(v) {
            if !adj.is_border(nb as usize) {
                continue;
            }
            let p = m.positions()[nb as usize];
            let d = [
                f64::from(p[0] - at[0]),
                f64::from(p[1] - at[1]),
                f64::from(p[2] - at[2]),
            ];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            for k in 0..3 {
                acc[k] += d[k] / l;
            }
            n += 1;
        }
        assert_eq!(
            n, 2,
            "beira manifold tem exactamente dois vizinhos de beira"
        );
        let l = (acc[0] * acc[0] + acc[1] * acc[1] + acc[2] * acc[2]).sqrt();
        assert!(
            l > 1e-3,
            "beira CURVA: a bissetriz não pode degenerar (|s| = {l})"
        );
        let sn = m.normals()[v];
        let dot = (acc[0] / l) * f64::from(sn[0])
            + (acc[1] / l) * f64::from(sn[1])
            + (acc[2] / l) * f64::from(sn[2]);
        worst_deg = worst_deg.max(dot.abs());
    }
    assert!(
        worst_deg < 1e-4,
        "a bissetriz é ORTOGONAL à normal num disco plano; pior |cos| = {worst_deg}"
    );
}

#[test]
fn the_pillow_has_low_valence_and_no_border() {
    let m = pillow();
    assert_eq!(m.vert_count(), 3);
    assert_eq!(m.face_count(), 2);
    assert_eq!(
        low_valence_count(&m),
        3,
        "os três vértices têm dois vizinhos — o que o original congela"
    );
    assert_eq!(
        border_count(&m),
        0,
        "valência baixa NÃO é borda: 2 faces e 2 vizinhos únicos em cada vértice"
    );
}

#[test]
fn the_collapsed_tetra_has_two_zero_area_faces() {
    let m = collapsed_tetra();
    assert_eq!(m.vert_count(), 4);
    assert_eq!(
        degenerate_faces(&m),
        vec![1, 3],
        "são as duas faces que contêm o par colapsado, e a numeração é a que os gates citam"
    );
}

/// Medido: **sliver 37,99° · `uv_sphere(32,48)` 1,40°**.
///
/// ⚠️ **A barra que decide é a RAZÃO, não o valor absoluto.** Um número absoluto
/// sozinho não diz se a fixture separa as duas ponderações ou se a malha inteira
/// é ruim; o controle bem formado ao lado é o que transforma isto num oráculo.
#[test]
fn the_sliver_fan_separates_area_weighting_from_plain_averaging() {
    let sliver = worst_weighting_divergence(&sliver_bipyramid());
    let control = worst_weighting_divergence(&shapes::uv_sphere(32, 48, 1.0));
    println!("sliver = {sliver:.2} deg · uv_sphere(32,48) = {control:.2} deg");
    assert!(
        control < 3.0,
        "o controle bem formado não separa as duas ponderações ({control:.2} deg)"
    );
    assert!(
        sliver > 20.0,
        "o leque com sliver tinha de separar, e mediu {sliver:.2} deg"
    );
    assert!(
        sliver > control * 5.0,
        "a fixture só vale se a diferença for de outra ordem: {sliver:.2} contra {control:.2}"
    );
}
