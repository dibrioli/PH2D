//! **A TABELA QUE O `BATCH_FRACTION` EXIGIA** — qualidade × relógio.
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-crossfield --release \
//!     --test rounding_policy -- --ignored --nocapture
//! ```
//!
//! ⭐ **O comentário do `BATCH_FRACTION` dizia, desde que nasceu:** *"é uma
//! divergência declarada da referência, e ela tem preço por medir — ⛔ não a mexa
//! sem a tabela de qualidade × relógio ao lado."* Esta sonda é essa tabela.
//!
//! A régua é a **CONTAGEM** de singularidades, nunca a soma: a soma é `4·χ` por
//! Poincaré–Hopf e vale `8` numa esfera qualquer que seja o campo — um par
//! `+1 / −1` espúrio cancela-se e ela não pisca.

use ph2d_crossfield::{Dual, Rounding, singularities, solve_miq_with};
use ph2d_mesh::{Mesh, shapes};

fn measure(label: &str, mesh: &Mesh, dual: &Dual, policy: Rounding) {
    let t = std::time::Instant::now();
    let (field, rep) = solve_miq_with(dual, policy);
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let (n, sum) = singularities(mesh, dual, &field);
    println!(
        "  {label:<28} SING {n:<5} soma {sum:<4} resolucoes {:<6} {ms:>9.0} ms",
        rep.solves
    );
}

fn sweep(name: &str, mesh: Mesh) {
    let mut mesh = mesh;
    mesh.triangulate();
    let dual = Dual::build(&mesh);
    println!("── {name} ({} vertices) ──", mesh.vert_count());
    for fraction in [8usize, 16, 32, 64] {
        measure(
            &format!("1/{fraction} por rodada"),
            &mesh,
            &dual,
            Rounding {
                fraction,
                max_deviation: 0.5,
            },
        );
    }
    // ⭐ A outra porta: recusar quem está genuinamente fracionário. O lote fica
    // largo (1/8) e é a CONFIANÇA que trava — se o preço vier daqui, esta linha é
    // a barata, porque não multiplica o número de resoluções.
    for max_deviation in [0.25f32, 0.15, 0.08, 0.04] {
        measure(
            &format!("1/8, so' se |d| <= {max_deviation}"),
            &mesh,
            &dual,
            Rounding {
                fraction: 8,
                max_deviation,
            },
        );
    }
}

#[test]
#[ignore = "sonda -- a tabela de qualidade x relogio do arredondamento"]
fn what_does_the_batch_cost() {
    let mut coarse = shapes::uv_sphere(96, 144, 1.0);
    ph2d_remesh_iso::remesh_isotropic(&mut coarse, 0.020);
    sweep("esfera iso a=0.020 (o caso que ja' esta' bom)", coarse);

    let mut fine = shapes::uv_sphere(96, 144, 1.0);
    ph2d_remesh_iso::remesh_isotropic(&mut fine, 0.010);
    sweep("esfera iso a=0.010 (o caso que sai com 194)", fine);
}

#[test]
#[ignore = "sonda LENTA (~2 min) -- a cauda da curva: o lote e' alavanca ou CAUSA?"]
fn where_does_the_batch_curve_flatten() {
    // ⭐ **A pergunta que decide o que fazer a seguir.** Se a curva descer até `8`,
    // o lote é a causa inteira e a cura é escolher um ponto de compromisso; se ela
    // achatar acima disso, há um SEGUNDO mecanismo e a tabela sozinha não fecha o
    // assunto.
    //
    // ⚠️ **A lei da referência — um inteiro por rodada — foi orçada e NÃO corrida.**
    // São `E − F + 1 ≈ 10 250` resoluções, e as primeiras têm quase todos os
    // inteiros livres (dimensão ~30 700, até 600 iterações de CG cada): a conta dá
    // **mais de uma hora**, contra os 11 s de `1/64`. *Estes dois pontos respondem
    // à mesma pergunta por 1/100 do relógio.*
    let mut fine = shapes::uv_sphere(96, 144, 1.0);
    ph2d_remesh_iso::remesh_isotropic(&mut fine, 0.010);
    fine.triangulate();
    let dual = Dual::build(&fine);
    println!("── esfera iso a=0.010 ({} vertices) ──", fine.vert_count());
    for fraction in [128usize, 256] {
        measure(
            &format!("1/{fraction} por rodada"),
            &fine,
            &dual,
            Rounding {
                fraction,
                max_deviation: 0.5,
            },
        );
    }
}
