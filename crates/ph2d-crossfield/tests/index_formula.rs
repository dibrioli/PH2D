//! **SONDA — falta o DEFEITO ANGULAR na fórmula do índice?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-crossfield --release \
//!     --test index_formula -- --ignored --nocapture
//! ```
//!
//! ⭐ **A pista que a auditoria da régua deu.** O índice sai de
//! `round(Σ(κ + (π/2)·p) / (π/2))`, e o resíduo desse arredondamento é:
//!
//! | malha | pior resíduo | ambíguos |
//! |---|---|---|
//! | `uv_sphere(96,144)` | **0,001** | 0 |
//! | `uv_sphere(24,36)` | 0,004 | 0 |
//! | `torus(32,16)` | 0,049 | 0 |
//! | ⛔ `sphere_shuffled` | **0,500** | 1 468 |
//! | ⛔ `sphere_noisy` | **0,500** | 4 472 |
//!
//! ⚠️ **`0,500` é o máximo possível: um empate.** O `round` decide por sorteio, e
//! é por isso que a soma sai `−147` onde a topologia exige `+8`.
//!
//! E a coluna do resíduo tem a ORDEM DE GRANDEZA do **defeito angular**
//! `K_v = 2π − Σ(ângulos incidentes)`: numa esfera `uv` de 13 682 vértices ele vale
//! `4π/13682 ≈ 0,0009 rad`, que em quartos de volta é `0,0006` — exactamente o
//! resíduo medido. ⇒ **Hipótese: a fórmula esqueceu o `K_v`**, e ele só se esconde
//! porque numa malha bem distribuída é minúsculo.
//!
//! Esta sonda mede as três variantes e deixa o resíduo decidir. *Não se escolhe
//! uma fórmula por dedução quando três linhas de medição a nomeiam.*
//!
//! ✅ **Respondido: `total + K_v` dá `0,0000` em TODAS as fixturas**, incluindo
//! as duas com milhares de ambíguos. A sonda FICA porque ela é a única coisa que
//! volta a responder isto se alguém mexer no `κ` ou nas molduras — e a coluna
//! `Σ K_v / 2π = χ` é o controle de Gauss–Bonnet que prova que o defeito angular
//! desta sonda está certo antes de ela julgar seja o que for.

use ph2d_crossfield::{Dual, QUARTER, solve_miq};
use ph2d_mesh::{Mesh, shapes};

/// `K_v = 2π − Σ(ângulos incidentes)` — o defeito angular de cada vértice.
fn angle_defects(mesh: &Mesh) -> Vec<f64> {
    let pos = mesh.positions();
    let mut k = vec![f64::from(core::f32::consts::TAU); mesh.vert_count()];
    for f in mesh.faces() {
        let v = f.verts();
        for i in 0..v.len() {
            let (o, a, b) = (
                pos[v[i] as usize],
                pos[v[(i + 1) % v.len()] as usize],
                pos[v[(i + v.len() - 1) % v.len()] as usize],
            );
            let (u, w) = (sub(a, o), sub(b, o));
            let (lu, lw) = (norm(u), norm(w));
            if lu < 1e-12 || lw < 1e-12 {
                continue;
            }
            k[v[i] as usize] -= f64::from((dot(u, w) / (lu * lw)).clamp(-1.0, 1.0).acos());
        }
    }
    k
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}
fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// O pior resíduo e quantos ambíguos, para uma variante da fórmula.
fn residuals(totals: &[f64], defect: &[f64], sign: f64) -> (f64, usize) {
    let q = f64::from(QUARTER);
    let mut worst = 0.0f64;
    let mut ambiguous = 0usize;
    for (t, k) in totals.iter().zip(defect) {
        let x = (t + sign * k) / q;
        let r = (x - x.round()).abs();
        worst = worst.max(r);
        if r > 0.25 {
            ambiguous += 1;
        }
    }
    (worst, ambiguous)
}

fn probe(name: &str, mut mesh: Mesh) {
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    // O `total` cru de cada vértice — a soma transportada, sem nenhuma correção.
    let totals = ph2d_crossfield::ring_totals(&mesh, &dual, &field);
    let defect = angle_defects(&mesh);
    let checked: f64 = defect.iter().sum::<f64>() / f64::from(core::f32::consts::TAU);
    println!(
        "{name:<28} v {:<7} Σ K_v / 2π = {checked:.3} (= χ, o controle)",
        mesh.vert_count()
    );
    for (label, sign) in [
        ("so' o total (a lei ANTIGA)", 0.0),
        ("total − K_v", -1.0),
        ("total + K_v (a lei EM VIGOR)", 1.0),
    ] {
        let (worst, amb) = residuals(&totals, &defect, sign);
        println!("    {label:<26} pior-residuo {worst:.4}  ambiguos {amb}");
    }
}

#[test]
#[ignore = "sonda -- qual das tres variantes da' um INTEIRO"]
fn which_variant_lands_on_an_integer() {
    probe("uv_sphere(96,144)", shapes::uv_sphere(96, 144, 1.0));
    probe("uv_sphere(24,36)", shapes::uv_sphere(24, 36, 1.0));
    probe("torus(32,16)", shapes::torus(32, 16, 1.0, 0.35));
    probe("sphere_shuffled", shapes::uv_sphere_shuffled(96, 144, 1.0));
    probe("sphere_noisy", shapes::uv_sphere_noisy(96, 144, 1.0, 0.02));
    let mut iso = shapes::uv_sphere(96, 144, 1.0);
    ph2d_remesh_iso::remesh_isotropic(&mut iso, 0.010);
    probe("esfera iso a=0.010", iso);
}
