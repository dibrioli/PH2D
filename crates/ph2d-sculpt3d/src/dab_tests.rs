//! Gates do dab.

use super::*;
use ph2d_mesh::shapes;

/// O falloff é C¹ na borda: valor **e** derivada zeram em `t = 1`. É isso que
/// impede um degrau na fronteira do pincel — o defeito que se vê como um anel.
#[test]
fn the_falloff_lands_on_zero_with_zero_slope() {
    assert_eq!(falloff(0.0), 1.0);
    assert_eq!(falloff(1.0), 0.0);
    assert_eq!(falloff(2.0), 0.0);
    // A inclinação perto da borda tem de ir a zero junto com o valor.
    let h = 1e-3;
    let slope = (falloff(1.0) - falloff(1.0 - h)) / h;
    assert!(
        slope.abs() < 0.01,
        "a borda tem degrau (inclinação {slope})"
    );
    // Monotônico do centro para fora — um falloff que sobe no meio do caminho
    // faz o pincel deixar um anel.
    let mut prev = f32::INFINITY;
    for i in 0..=100 {
        let w = falloff(i as f32 / 100.0);
        assert!(w <= prev + 1e-6, "subiu em t={}", i as f32 / 100.0);
        prev = w;
    }
}

/// Um dab empurra a superfície para FORA, mais no centro que na borda — e não
/// toca em nada além do raio.
#[test]
fn a_dab_pushes_the_surface_out_most_at_the_centre_and_nothing_beyond_the_radius() {
    let mut mesh = shapes::uv_sphere(40, 56, 1.0);
    let before = mesh.positions().to_vec();
    let mut scratch = DabScratch::default();

    let dab = Dab {
        center: [0.0, 1.0, 0.0], // o polo
        radius: 0.5,
        strength: 0.1,
    };
    let moved = apply_dab(&mut mesh, &dab, &mut scratch);
    assert!(moved > 20, "o dab tocou só {moved} vértices");

    let mut max_at_centre = 0.0f32;
    for (v, (&a, &b)) in before.iter().zip(mesh.positions()).enumerate() {
        let delta = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
        let d = ((a[0] - dab.center[0]).powi(2)
            + (a[1] - dab.center[1]).powi(2)
            + (a[2] - dab.center[2]).powi(2))
        .sqrt();
        if d > dab.radius {
            assert_eq!(delta, 0.0, "o vértice {v} está a {d} e mexeu {delta}");
        }
        if d < 0.05 {
            max_at_centre = max_at_centre.max(delta);
        }
        // O raio da esfera cresce: empurrar pela normal de uma esfera é para fora.
        let r_before = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
        let r_after = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
        assert!(r_after >= r_before - 1e-5, "o vértice {v} afundou");
    }
    assert!(
        (max_at_centre - dab.strength).abs() < 1e-3,
        "no centro o deslocamento tem de ser a força cheia, e foi {max_at_centre}"
    );
}

/// Força negativa CAVA. O mesmo kernel, o sinal do artista.
#[test]
fn a_negative_strength_digs_instead_of_pushing() {
    let mut mesh = shapes::uv_sphere(24, 32, 1.0);
    let before = mesh.positions()[0];
    let mut scratch = DabScratch::default();
    apply_dab(
        &mut mesh,
        &Dab {
            center: before,
            radius: 0.4,
            strength: -0.1,
        },
        &mut scratch,
    );
    let after = mesh.positions()[0];
    let r = |p: [f32; 3]| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    assert!(
        r(after) < r(before) - 0.05,
        "não cavou: {before:?} → {after:?}"
    );
}

/// ⚠️ **As normais são lidas ANTES de qualquer escrita.** Se o laço lesse a
/// normal já atualizada de um vizinho, a ordem de iteração vazaria para a
/// forma — o mesmo dab daria resultados diferentes conforme a numeração dos
/// vértices, e isso é invisível até alguém reordenar a malha.
///
/// O oráculo é a SIMETRIA: um dab no polo de uma esfera tem de produzir um
/// resultado com simetria de revolução, e o vazamento a quebra.
#[test]
fn the_dab_reads_every_normal_before_it_writes_any_position() {
    let mut mesh = shapes::uv_sphere(32, 48, 1.0);
    let mut scratch = DabScratch::default();
    apply_dab(
        &mut mesh,
        &Dab {
            center: [0.0, 1.0, 0.0],
            radius: 0.6,
            strength: 0.15,
        },
        &mut scratch,
    );
    // Todos os vértices de um mesmo anel estão à mesma distância do polo, logo
    // têm de terminar no mesmo raio.
    let ring: Vec<f32> = mesh
        .positions()
        .iter()
        .filter(|p| (p[1] - mesh.positions()[1][1]).abs() < 1e-4)
        .map(|p| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt())
        .collect();
    assert!(ring.len() > 8, "a fixture não achou o anel");
    let (lo, hi) = ring
        .iter()
        .fold((f32::MAX, 0.0f32), |(l, h), &r| (l.min(r), h.max(r)));
    assert!(
        hi - lo < 1e-4,
        "o anel perdeu a simetria ({lo} a {hi}) — a ordem de iteração vazou"
    );
}

/// Um dab fora da malha não faz nada e não derruba nada.
#[test]
fn a_dab_that_touches_nothing_is_a_no_op() {
    let mut mesh = shapes::cube(1.0);
    let before = mesh.positions().to_vec();
    let mut scratch = DabScratch::default();
    let moved = apply_dab(
        &mut mesh,
        &Dab {
            center: [50.0, 0.0, 0.0],
            radius: 1.0,
            strength: 1.0,
        },
        &mut scratch,
    );
    assert_eq!(moved, 0);
    assert_eq!(mesh.positions(), &before[..]);
    // Raio zero também: um pincel sem tamanho não é um pincel.
    assert_eq!(
        apply_dab(
            &mut mesh,
            &Dab {
                center: [0.0; 3],
                radius: 0.0,
                strength: 1.0
            },
            &mut scratch
        ),
        0
    );
}

/// As normais ficam CERTAS depois do dab — o mesmo que uma reconstrução
/// completa daria. Sem isto a superfície nova aparece com a luz da antiga.
#[test]
fn the_normals_are_correct_after_a_dab() {
    let mut mesh = shapes::uv_sphere(20, 28, 1.0);
    let mut scratch = DabScratch::default();
    apply_dab(
        &mut mesh,
        &Dab {
            center: [0.0, 1.0, 0.0],
            radius: 0.5,
            strength: 0.2,
        },
        &mut scratch,
    );
    let incremental = mesh.normals().to_vec();
    mesh.rebuild();
    assert_eq!(
        incremental,
        mesh.normals(),
        "as normais depois do dab divergiram da reconstrução completa"
    );
}
