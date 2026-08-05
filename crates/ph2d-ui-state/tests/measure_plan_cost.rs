//! **A SONDA que decidiu o desenho desta crate** — ela imprime os números que o doc do
//! `lib.rs` cita, para que eles continuem reproduzíveis em vez de virarem folclore.
//!
//! `cargo test -p ph2d-ui-state --release --test measure_plan_cost -- --ignored --nocapture`
//!
//! ⚠️ `--release` não é preferência: o `Plan` é uma busca de fase 256×256, e em debug o número
//! mede o PERFIL do build e não o produto.

use ph2d_ui_state::{ObjectPose, Transition, UiState};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, ellipse, rectangle};
use std::time::Instant;

fn timed(label: &str, n: u32, mut f: impl FnMut()) -> f64 {
    let t = Instant::now();
    for _ in 0..n {
        f();
    }
    let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(n);
    println!("{label:<42} {ms:>9.4} ms");
    ms
}

fn state(name: &str, geom: VecPath) -> UiState {
    let mut s = UiState::new(name);
    s.objects = vec![ObjectPose {
        geometry: Some(geom),
        ..ObjectPose::new(1)
    }];
    s
}

#[test]
#[ignore]
fn measure_what_a_transition_costs() {
    let mut a_geom = rectangle([0.0, 0.0], [2.0, 1.0]);
    a_geom.id = 1;
    a_geom.fill = Some(Paint::Solid(Rgba8::new(30, 90, 200, 255)));
    let mut colour_only = a_geom.clone();
    colour_only.fill = Some(Paint::Solid(Rgba8::new(230, 200, 40, 255)));
    let mut other_shape = ellipse([1.0, 0.5], 1.0, 0.5);
    other_shape.id = 1;

    let a = state("idle", a_geom);
    let same = state("hover", colour_only);
    let diff = state("open", other_shape);

    let n_colour = timed("Transition::new (so' COR)", 200, || {
        std::hint::black_box(Transition::new(&a, &same));
    });
    let n_shape = timed("Transition::new (a FORMA muda)", 200, || {
        std::hint::black_box(Transition::new(&a, &diff));
    });
    let tr = Transition::new(&a, &diff);
    let at = timed("Transition::at (um passo)", 2000, || {
        std::hint::black_box(tr.at(0.5));
    });

    println!(
        "\nplans construidos:  so'-cor {}  |  forma {}",
        Transition::new(&a, &same).plans_built(),
        tr.plans_built()
    );
    println!(
        "razao new/at quando a FORMA muda:      {:.0}x",
        n_shape / at
    );
    println!(
        "razao forma/cor (o que o `Plan` custa): {:.0}x",
        n_shape / n_colour
    );
    println!(
        "20 objetos so'-de-cor: {:.2} ms por transicao (um quadro de 60 fps tem 16,7)",
        n_colour * 20.0
    );
}
