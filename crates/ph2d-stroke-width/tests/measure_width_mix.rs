//! **Quanto a mistura de dois perfis de largura DESVIA nas pontas** (plano UI/UX W7).
//!
//! Rode com: `cargo test -p ph2d-stroke-width --test measure_width_mix -- --ignored --nocapture`
//!
//! A pergunta é uma só: `mix(a, b, 0)` é `a`? A representação é uma lista de paradas com
//! `smoothstep` entre elas, então uma parada nova **no meio de um vão** re-parte esse
//! `smoothstep` em dois — e o que se mede aqui é o tamanho disso.

use ph2d_stroke_width::{WidthProfile, WidthStops};

fn worst(a: &WidthStops, b: &WidthStops, t: f64) -> f64 {
    let m = a.mix(b, t);
    let want = |p: f64| {
        let (u, v) = (a.at(p), b.at(p));
        u + (v - u) * t
    };
    (0..=1000)
        .map(|i| {
            let p = f64::from(i) / 1000.0;
            (m.at(p) - want(p)).abs()
        })
        .fold(0.0_f64, f64::max)
}

#[test]
#[ignore = "sonda de medição — roda a pedido"]
fn how_far_the_union_sampling_drifts() {
    let uniform = WidthStops::default();
    let bulge = WidthProfile {
        start: 0.2,
        mid: 1.8,
        end: 0.2,
        position: 0.5,
    }
    .to_stops();
    let early = WidthProfile {
        start: 1.0,
        mid: 0.3,
        end: 1.4,
        position: 0.25,
    }
    .to_stops();

    println!("\n  par                                 t=0      t=0.5      t=1");
    println!("  ------------------------------------------------------------");
    for (name, a, b) in [
        ("uniforme x bulge (o par da UI)", &uniform, &bulge),
        ("bulge x bulge (joelhos iguais)", &bulge, &bulge),
        ("bulge x early (joelhos DIFERENTES)", &bulge, &early),
    ] {
        println!(
            "  {name:<35} {:.4}   {:.4}   {:.4}",
            worst(a, b, 0.0),
            worst(a, b, 0.5),
            worst(a, b, 1.0)
        );
    }
    println!();
}
