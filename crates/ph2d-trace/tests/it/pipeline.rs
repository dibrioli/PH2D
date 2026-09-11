//! **A SONDA DE PONTA A PONTA** — campo, traçado, patches, quantização.
//!
//! ⭐ **É a primeira vez que a cadeia inteira corre com código NOSSO.** Antes do
//! F3 o layout só existia porque o oráculo o exportava; aqui ele nasce da malha.
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-trace --release \
//!     --test pipeline -- --ignored --nocapture
//! ```

use ph2d_crossfield::{Dual, singularities, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quantize::quantize;
use ph2d_trace::trace_patches;

fn run(name: &str, mut mesh: Mesh, iso: bool) {
    if iso {
        let r = ph2d_remesh_iso::remesh_isotropic(&mut mesh, ph2d_remesh_iso::ALPHA);
        eprintln!("  [iso] {} -> {} vertices", r.verts_before, r.verts_after);
    }
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let (sing, sum) = singularities(&mesh, &dual, &field);
    let layout = trace_patches(&mesh, &dual, &field);
    let r = &layout.report;
    println!(
        "{name:<16} v={:<6} sing={sing:<3}(soma {sum:<3}) aneis-abertos={:<4} sep={:<4} soltas={:<4} patches={:<5} dissolv={:<3} nao-disco={:<4} arcos={:<5} val={:?}",
        mesh.vert_count(),
        r.open_rings,
        r.separatrices,
        r.dangling,
        r.patches,
        r.dissolved,
        r.non_disk,
        r.arcs,
        r.valence
    );
    match layout.to_layout(0.05) {
        Ok(l) => match quantize(&l) {
            Ok((_, rep)) => println!(
                "{:<16}   -> QUANTIZA: custo {:.2}, limite {:.2}, prova {}",
                "",
                rep.cost,
                rep.lower_bound,
                if rep.proved { "sim" } else { "NAO" }
            ),
            Err(e) => println!("{:<16}   -> o F4 recusou: {e:?}", ""),
        },
        Err(e) => println!("{:<16}   -> layout invalido: {e:?}", ""),
    }
}

#[test]
#[ignore = "sonda -- imprime a cadeia inteira, nao afirma um limite"]
fn the_whole_chain_runs_on_our_own_code() {
    run("cube", shapes::cube(1.0), true);
    run("sphere 24x36", shapes::uv_sphere(24, 36, 1.0), false);
    run("sphere 48x72", shapes::uv_sphere(48, 72, 1.0), false);
    run("torus 32x16", shapes::torus(32, 16, 1.0, 0.35), false);
    run("sphere iso", shapes::uv_sphere(48, 72, 1.0), true);
}
