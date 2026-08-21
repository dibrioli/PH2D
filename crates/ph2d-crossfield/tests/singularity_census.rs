//! **SONDA — a contagem de singularidades depende da MALHA?**
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-crossfield --release \
//!     --test singularity_census -- --ignored --nocapture
//! ```
//!
//! ⭐ **A pergunta que o F5 levantou e que a SOMA não podia responder.** A soma
//! dos índices é `4·χ` por Poincaré–Hopf — **ela é forçada pela topologia** e vale
//! `8` numa esfera qualquer que seja o campo, inclusive um péssimo. *Um gate sobre
//! a soma não pode detetar um par `+1 / −1` espúrio: ele cancela-se.*
//!
//! A régua honesta é a **CONTAGEM**: uma esfera admite **8** singularidades, e
//! tudo acima disso é o campo a inventar pares que se anulam.

use ph2d_crossfield::{Dual, singularities, solve_miq};
use ph2d_mesh::{Mesh, shapes};

fn census(name: &str, mut mesh: Mesh) {
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, rep) = solve_miq(&dual);
    let (n, sum) = singularities(&mesh, &dual, &field);
    println!(
        "{name:<30} v {:<7} SING {:<5} soma {sum:<4} | resolucoes {:<5} inteiros {:<6} \
         NAO-CONVERGIU {:<5} pior residuo {:.2e}",
        mesh.vert_count(),
        n,
        rep.solves,
        rep.free_integers,
        rep.cg_capped,
        rep.cg_worst_residual
    );
}

#[test]
#[ignore = "sonda -- a contagem contra a malha, sem nada a jusante"]
fn does_the_count_depend_on_the_mesh() {
    println!("── esferas ESTRUTURADAS (grade uv, valencia quase toda 6) ──");
    for (r, s) in [(24, 36), (48, 72), (72, 108), (96, 144)] {
        census(&format!("uv_sphere({r},{s})"), shapes::uv_sphere(r, s, 1.0));
    }
    println!("── a MESMA esfera, remalhada isotropicamente (conectividade irregular) ──");
    for alpha in [0.02f32, 0.017, 0.014, 0.012, 0.010] {
        let mut m = shapes::uv_sphere(96, 144, 1.0);
        ph2d_remesh_iso::remesh_isotropic(&mut m, alpha);
        census(&format!("uv_sphere iso a={alpha}"), m);
    }
    // ⭐ **O experimento decisivo.** As duas primeiras hipóteses caíram (o passe
    // do F1 converge em todos os alvos; e a esfera ESTRUTURADA tem o pior resíduo
    // de CG da tabela e sai com 7). Sobra a terceira: o passe reprojeta sobre a
    // malha de ENTRADA, que é um poliedro **facetado** — e quando a saída se
    // aproxima da resolução da entrada, os vértices passam a pousar DENTRO das
    // facetas, onde a curvatura discreta é zero, com toda a curvatura concentrada
    // nas arestas da faceta. *O campo passa a seguir a triangulação da entrada,
    // não a forma.*
    //
    // A régua é a RAZÃO entre as duas resoluções, e não o tamanho de nenhuma.
    println!("── a MESMA saida (~10 k), com referencias de finura CRESCENTE ──");
    for (r, s) in [(96, 144), (144, 216), (192, 288), (256, 384)] {
        let mut m = shapes::uv_sphere(r, s, 1.0);
        let before = m.vert_count();
        ph2d_remesh_iso::remesh_isotropic(&mut m, 0.010);
        census(&format!("ref {before} -> iso a=0.010"), m);
    }
    println!("── e o inverso: saida GROSSA sobre referencia grossa ──");
    for (r, s) in [(48, 72), (32, 48)] {
        let mut m = shapes::uv_sphere(r, s, 1.0);
        let before = m.vert_count();
        ph2d_remesh_iso::remesh_isotropic(&mut m, 0.020);
        census(&format!("ref {before} -> iso a=0.020"), m);
    }
    println!("── toros ──");
    for (a, b) in [(32, 16), (64, 32)] {
        census(&format!("torus({a},{b})"), shapes::torus(a, b, 1.0, 0.35));
    }
    let mut m = shapes::torus(64, 32, 1.0, 0.35);
    ph2d_remesh_iso::remesh_isotropic(&mut m, ph2d_remesh_iso::ALPHA);
    census("torus iso a=0.02", m);
}
