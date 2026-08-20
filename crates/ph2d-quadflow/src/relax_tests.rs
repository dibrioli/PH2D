//! **OS GATES DA RELAXAÇÃO** — as duas metades, e o que cada uma sozinha faz.

use ph2d_mesh::{Mesh, shapes};

use super::{RELAX_PASSES, edge_length_spread, relax};
use crate::extract::extract;
use crate::scale::{ScaleField, mean_edge};

fn extracted(mesh: &Mesh) -> Mesh {
    let scale = ScaleField::uniform(mesh, 3.0 * mean_edge(mesh));
    let (o, p) = crate::solve::solve_fields(mesh, &scale);
    extract(mesh, &o, &p, &scale).expect("extraiu").mesh
}

/// ⭐ **A GRADE FICA MAIS REGULAR** — é para isto que o passe existe.
#[test]
fn the_relaxation_makes_the_grid_more_even() {
    for (name, mesh) in [
        ("esfera", shapes::uv_sphere(48, 64, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let mut out = extracted(&mesh);
        let before = edge_length_spread(&out);
        relax(&mut out, &mesh, RELAX_PASSES);
        let after = edge_length_spread(&out);
        eprintln!("[quadflow] {name}: desvio de aresta {before:.3} -> {after:.3}");
        assert!(
            after < before,
            "{name}: o relaxamento deixou a grade MAIS irregular ({before:.3} -> {after:.3})"
        );
    }
}

/// ⭐ **E ELA NÃO SAI DA FORMA** — a metade que a projeção compra.
///
/// ⚠️ **A régua é o HAUSDORFF em unidades de quad, e não o volume** — a mesma da
/// A4, e pela mesma razão. O volume **cai uma vez** no primeiro passe (medido no
/// toro: 2,266 → 2,171) e depois fica parado, porque a relaxação põe cada nó
/// **exatamente sobre** a superfície de entrada, e um poliedro inscrito tem menos
/// volume que a superfície lisa. Isso é a projeção a funcionar, não a falhar — e
/// um gate de volume o leria como encolhimento.
///
/// Medido a 2 passadas: esfera `0,091 → 0,098`, toro `0,204 → 0,202`, amassada
/// `0,507 → 0,465`. A barra é **15 % pior que o não-relaxado** (2× a folga do
/// pior caso) e **abaixo de um quad** (a barra da A4).
#[test]
fn the_relaxation_does_not_leave_the_surface() {
    for (name, mesh) in [
        ("esfera", shapes::uv_sphere(48, 64, 1.0)),
        ("toro", shapes::torus(64, 32, 1.0, 0.35)),
    ] {
        let edge = 3.0 * mean_edge(&mesh);
        let mut out = extracted(&mesh);
        let before = hausdorff(&out, &mesh) / edge;
        relax(&mut out, &mesh, RELAX_PASSES);
        let after = hausdorff(&out, &mesh) / edge;
        eprintln!("[quadflow] {name}: forma {before:.3} -> {after:.3} quads");
        assert!(
            after < 1.0,
            "{name}: depois de relaxar a forma anda {after:.3} vezes o lado do quad (barra: um quad)"
        );
        assert!(
            after <= before * 1.15,
            "{name}: relaxar AFASTOU a malha da superficie ({before:.3} -> {after:.3} quads) -- a \
             projecao de volta a' entrada nao esta' a segurar o Laplaciano"
        );
    }
}

/// A distância de Hausdorff bilateral entre duas malhas, em unidades de objeto.
fn hausdorff(a: &Mesh, b: &Mesh) -> f32 {
    one_sided(a, b).max(one_sided(b, a))
}

/// A maior distância de um vértice de `from` à superfície de `to`.
fn one_sided(from: &Mesh, to: &Mesh) -> f32 {
    let mut worst = 0.0f32;
    for &p in from.positions() {
        let q = super::project_onto(to, p, 0.05);
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        worst = worst.max(d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt());
    }
    worst
}

/// ⭐ **A7 — DETERMINÍSTICO** (HR-5): Jacobi, e a ordem dos índices não conta.
#[test]
fn the_relaxation_is_bit_reproducible() {
    let mesh = shapes::torus(64, 32, 1.0, 0.35);
    let (mut a, mut b) = (extracted(&mesh), extracted(&mesh));
    relax(&mut a, &mesh, RELAX_PASSES);
    relax(&mut b, &mesh, RELAX_PASSES);
    assert_eq!(
        a.positions(),
        b.positions(),
        "duas relaxacoes deram vertices diferentes"
    );
}

/// **ZERO PASSADAS É UM NO-OP AO BIT** — o controle.
#[test]
fn zero_passes_is_a_bit_identical_noop() {
    let mesh = shapes::uv_sphere(48, 64, 1.0);
    let out = extracted(&mesh);
    let mut same = out.clone();
    relax(&mut same, &mesh, 0);
    assert_eq!(
        out.positions(),
        same.positions(),
        "zero passadas mexeu na malha"
    );
}
