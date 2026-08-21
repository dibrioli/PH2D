//! **OS GATES DO TRAÇADO** (ADR-0161, F3).
//!
//! ⭐ **A régua principal é o FECHO DA CADEIA**: o layout que esta fase produz
//! tem de ser aceite pelo F4 e quantizado com **prova**. Nenhum número interno do
//! traçado prova tanto quanto o solver seguinte não recusar o que ele produziu.

use std::collections::BTreeMap;

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quantize::quantize;
use ph2d_trace::{PatchLayout, Ring, trace_patches};

/// Traça uma malha e devolve a decomposição.
fn decompose(mut mesh: Mesh) -> (Mesh, PatchLayout) {
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let out = trace_patches(&mesh, &dual, &field);
    (mesh, out)
}

/// Aresta dirigida -> face, como o traçado a constrói.
fn halves(mesh: &Mesh) -> BTreeMap<(u32, u32), u32> {
    let mut half = BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            half.insert((v[k], v[(k + 1) % v.len()]), u32::try_from(fi).unwrap());
        }
    }
    half
}

// ───────────────────────────── o anel ─────────────────────────────

#[test]
fn the_ring_of_a_closed_mesh_closes_at_every_vertex() {
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let half = halves(&mesh);
    for v in 0..mesh.vert_count() {
        let v = u32::try_from(v).unwrap();
        let ring =
            Ring::build(&mesh, &half, v).unwrap_or_else(|| panic!("o anel de {v} nao fecha"));
        assert_eq!(
            ring.verts.len(),
            ring.cum.len(),
            "vertice {v}: um acumulado por vizinho"
        );
        assert_eq!(ring.cum[0], 0.0, "o acumulado comeca em zero");
        assert!(
            ring.cum.windows(2).all(|w| w[1] > w[0]),
            "vertice {v}: o acumulado tem de crescer"
        );
        assert!(
            ring.total > ring.cum[ring.cum.len() - 1],
            "vertice {v}: o total inclui a ultima face"
        );
        // Numa esfera lisa a soma dos ângulos é quase a volta inteira.
        assert!(
            (ring.total - core::f32::consts::TAU).abs() < 0.35,
            "vertice {v}: total {} longe de 2π",
            ring.total
        );
    }
}

#[test]
fn a_singularity_emits_four_minus_its_index_separatrices() {
    // ⭐ **É a lei que decide quantas paredes nascem.** Medido em 2026-08-20:
    // emitir sempre QUATRO dava 23 patches numa esfera onde o oráculo dá 8 — a
    // quarta separatriz não tem para onde ir e vai bater no meio de outra.
    let mut mesh = shapes::uv_sphere(16, 24, 1.0);
    mesh.triangulate();
    let half = halves(&mesh);
    // Um vértice do meio, com anel farto.
    let v = (0..mesh.vert_count())
        .map(|v| u32::try_from(v).unwrap())
        .find(|&v| mesh.adjacency().valence(v as usize) >= 6)
        .expect("ha' vertice de valencia >= 6");
    let ring = Ring::build(&mesh, &half, v).expect("fecha");
    assert_eq!(ring.seeds(1, 0).len(), 3, "indice +1 emite tres");
    assert_eq!(ring.seeds(0, 0).len(), 4, "um regular teria quatro");
    assert_eq!(ring.seeds(-1, 0).len(), 5, "indice −1 emite cinco");
    // ⚠️ E elas são DISTINTAS: duas sementes no mesmo vizinho seriam uma parede
    // duplicada, que não separa nada.
    let s = ring.seeds(1, 0);
    assert!(s[0] != s[1] && s[1] != s[2] && s[0] != s[2], "{s:?}");
}

#[test]
fn an_index_at_or_above_four_emits_nothing() {
    // A lei `4 − k` deixa de fazer sentido em `k >= 4`; devolver uma lista vazia
    // é a única resposta honesta, e é melhor que emitir uma semente arbitrária.
    let mut mesh = shapes::uv_sphere(12, 16, 1.0);
    mesh.triangulate();
    let half = halves(&mesh);
    let ring = Ring::build(&mesh, &half, 5).expect("fecha");
    assert!(ring.seeds(4, 0).is_empty());
    assert!(ring.seeds(9, 0).is_empty());
}

// ──────────────────────── a decomposição ────────────────────────

#[test]
fn every_face_lands_in_exactly_one_patch() {
    let (mesh, out) = decompose(shapes::uv_sphere(24, 36, 1.0));
    assert_eq!(out.face_patch.len(), mesh.face_count());
    assert!(
        out.face_patch
            .iter()
            .all(|&p| (p as usize) < out.report.patches),
        "ha' face sem patch"
    );
    let mut seen = vec![false; out.report.patches];
    for &p in &out.face_patch {
        seen[p as usize] = true;
    }
    assert!(seen.iter().all(|&s| s), "ha' patch sem face nenhuma");
}

#[test]
fn every_arc_is_shared_by_exactly_two_patches() {
    // ⚠️ **É a cerca que apanha a chave de arco dependente do SENTIDO.** Os dois
    // patches percorrem o mesmo arco em sentidos opostos; se a identidade do arco
    // for a sequência e não o conjunto, cada um cria o seu e todos passam a ser
    // "de bordo". Medido em 2026-08-20: era exatamente isso, e o sintoma era o F4
    // a recusar com `ArcUse { uses: 1 }` numa superfície fechada.
    for (name, mesh) in [
        ("esfera", shapes::uv_sphere(24, 36, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (_, out) = decompose(mesh);
        let mut uses = vec![0usize; out.arc_length.len()];
        for sides in &out.sides {
            for side in sides {
                for &a in side {
                    uses[a as usize] += 1;
                }
            }
        }
        for (a, n) in uses.iter().enumerate() {
            assert_eq!(*n, 2, "{name}: o arco {a} e' usado {n} vezes, nao 2");
        }
    }
}

#[test]
fn the_cleanup_leaves_no_patch_with_fewer_than_three_sides() {
    // ⚠️ **Uma lasca de dois lados invalida o LAYOUT INTEIRO**, não só a si mesma:
    // a lei do F4 pede pelo menos três lados por patch, e ele recusa tudo por
    // causa de um. É por isso que a limpeza é obrigatória e não cosmética.
    for (name, mesh) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (_, out) = decompose(mesh);
        assert!(
            out.degenerate().is_empty(),
            "{name}: sobraram patches degenerados {:?} (valencias {:?})",
            out.degenerate(),
            out.report.valence
        );
    }
}

// ─────────────────── ⭐ o fecho da cadeia ───────────────────

#[test]
fn the_layout_we_produce_is_quantized_with_proof() {
    // ⭐ **O gate que diz que o F3 fechou.** Antes dele o layout só existia porque
    // o oráculo o exportava; aqui ele nasce da malha, e quem o valida é o solver
    // da fase seguinte — que já sabia recusar layout aberto, patch de menos de
    // três lados e arco de bordo.
    for (name, mesh) in [
        ("esfera 24x36", shapes::uv_sphere(24, 36, 1.0)),
        ("esfera 48x72", shapes::uv_sphere(48, 72, 1.0)),
        ("toro", shapes::torus(32, 16, 1.0, 0.35)),
    ] {
        let (_, out) = decompose(mesh);
        assert_eq!(out.report.open_rings, 0, "{name}: a malha chegou aberta");
        let layout = out
            .to_layout(0.05)
            .unwrap_or_else(|e| panic!("{name}: o F4 recusou o layout: {e:?}"));
        let (q, report) =
            quantize(&layout).unwrap_or_else(|e| panic!("{name}: o F4 nao quantizou: {e:?}"));
        assert!(report.proved, "{name}: sem prova de otimalidade");
        assert!(
            q.arc.iter().all(|&v| v >= 1),
            "{name}: um arco colapsou a zero"
        );
        assert_eq!(
            q.corners.len(),
            out.sides.len(),
            "{name}: um leque por patch"
        );
    }
}

#[test]
fn an_open_mesh_is_reported_not_silently_traced() {
    // ⚠️ **Uma malha com bordo não tem índice de singularidade definido** nos
    // vértices da beira, e o campo devolve `0` ali — logo a soma dos índices
    // deixa de bater `4·χ` sem que nada no campo esteja errado. Contar os anéis
    // abertos é o que separa *"o traçado é mau"* de *"a malha que chegou é má"*.
    // Medido em 2026-08-20: o cubo remalhado pelo F1 chega com **18** deles.
    // ⚠️ O `cylinder` das `shapes` tem TAMPAS — ele é fechado. Para ter bordo é
    // preciso arrancar faces, que é exatamente o que uma escultura rasgada tem.
    let mut sphere = shapes::uv_sphere(16, 24, 1.0);
    sphere.triangulate();
    let faces: Vec<_> = sphere.faces().iter().skip(6).copied().collect();
    let torn = Mesh::from_parts(sphere.positions().to_vec(), faces).expect("a malha rasgada monta");
    let (_, out) = decompose(torn);
    assert!(
        out.report.open_rings > 0,
        "uma malha rasgada tem de acusar aneis abertos"
    );
}
