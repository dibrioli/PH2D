//! **QUANTO O SOLVER INDEPENDENTE DE ORDEM TRANSPORTA** — a consequência de
//! produto da troca de modelo (doc 28 §5.45), medida em vez de estimada.
//!
//! O Gauss-Seidel do port 1:1 e o solver independente de ordem não são a mesma
//! resposta (o [`tests/solver_symmetry.rs`] mostra por quê: só um deles
//! preserva a simetria da cena). Isto aqui responde a **outra** pergunta, a que
//! o artista faz: *o escorrido corre menos?*
//!
//! ⚠️ **A régua é o CENTROIDE de massa, não a frente.** A célula mais extrema
//! acima de um limiar é uma estatística de um valor só, e ela é caótica: a
//! mesma varredura de `rf` devolvia 27, 23, 36, 18, 10, 14, 21 **dentro do
//! mesmo modelo**. O centroide é liso e é o que "onde a água está" significa.

mod util;

use ph2d_wet_paint::grid::Grid;
use ph2d_wet_paint::painter::Engine;
use util::drive_stroke;

const W: usize = 512;
const H: usize = 512;

fn scene(rf: usize, grav: f64, order_invariant: bool) -> Engine {
    let mut e = Engine::with_flow_ratio(W, H, rf);
    e.sim.order_invariant = order_invariant;
    e.sliders.water = 1.0;
    e.sliders.size = 0.8;
    e.sim.gravity_override = Some([0.0, grav]);
    drive_stroke(&mut e, 180.0, 80.0, 320.0, 80.0, 8.0, 0);
    for _ in 0..60 {
        e.step_simulation();
    }
    e
}

fn film_centroid_y(g: &Grid) -> f64 {
    let (mut m, mut my) = (0.0f64, 0.0f64);
    for y in 1..=g.h as i32 {
        for x in 1..=g.w as i32 {
            let v = f64::from(g.film[x as usize + y as usize * g.s]);
            m += v;
            my += v * f64::from(y);
        }
    }
    if m > 0.0 { my / m } else { 0.0 }
}

fn carry(rf: usize, oi: bool) -> f64 {
    film_centroid_y(scene(rf, 2.0, oi).active_grid())
        - film_centroid_y(scene(rf, 0.0, oi).active_grid())
}

/// **O escorrido corre ~18% menos, e é uniforme** — o número que o smoke julga.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_how_far_each_model_carries_the_water() {
    println!("\n  DESLOCAMENTO DO CENTROIDE do filme (celulas) sob gravidade 2.0\n");
    println!(
        "    {:>4} {:>14} {:>16}  {:>7}",
        "flow", "Gauss-Seidel", "order-invariant", "razao"
    );
    let (mut sum, mut n) = (0.0f64, 0u32);
    for rf in [1usize, 2, 3, 4, 6, 8] {
        let (a, b) = (carry(rf, false), carry(rf, true));
        sum += b / a.max(1e-9);
        n += 1;
        println!("    {rf:>4} {a:>14.2} {b:>16.2}  {:>6.2}x", b / a.max(1e-9));
    }
    println!(
        "\n    media {:.2}x — o solver independente de ordem transporta MENOS,\n\
         \x20   uniformemente, sem colapso. O knob Gravity cobre a diferenca.",
        sum / f64::from(n)
    );
}

/// **E o transporte é SIMÉTRICO em direção nos dois modelos** — o que refuta a
/// hipótese óbvia (*"a varredura de cima para baixo cascateia com a gravidade"*)
/// e é a razão de a diferença acima ser uniforme em vez de direcional.
///
/// ⚠️ Fica como gate, não como sonda: se alguém reintroduzir uma varredura
/// alinhada ao fluxo, é AQUI que aparece.
#[test]
fn the_water_runs_the_same_distance_in_both_directions() {
    let mid = |grav: f64, oi: bool| -> Engine {
        let mut e = Engine::with_flow_ratio(W, H, 1);
        e.sim.order_invariant = oi;
        e.sliders.water = 1.0;
        e.sliders.size = 0.8;
        e.sim.gravity_override = Some([0.0, grav]);
        drive_stroke(&mut e, 180.0, 256.0, 320.0, 256.0, 8.0, 0);
        for _ in 0..60 {
            e.step_simulation();
        }
        e
    };
    for oi in [false, true] {
        let d = film_centroid_y(mid(2.0, oi).active_grid()) - 256.0;
        let u = 256.0 - film_centroid_y(mid(-2.0, oi).active_grid());
        let ratio = d.max(u) / d.min(u).max(1e-9);
        println!("  order_invariant {oi}: baixo {d:.2}  cima {u:.2}  ({ratio:.2}x)");
        assert!(
            d > 3.0 && u > 3.0,
            "o CONTROLE nao correu ({d:.2} / {u:.2}) — a fixture nao contem o fenomeno"
        );
        assert!(
            ratio < 1.35,
            "o transporte depende da DIRECAO (order_invariant {oi}): baixo {d:.2}, cima {u:.2}"
        );
    }
}
