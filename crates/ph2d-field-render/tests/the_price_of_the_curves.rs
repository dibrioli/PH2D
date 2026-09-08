//! ⭐ **SONDA da W136** — quanto custam as duas curvas com espessura, medido no **QUADRO**.
//!
//! ⚠️ **A régua tem de ser o quadro** (doc 06 §130.1): a árvore é especializada por ladrilho × fatia,
//! e uma medição por amostra responde a outra pergunta. A Bezier lia **`6,8×` um círculo** por
//! amostra — aqui é que se sabe o que isso vale.
//!
//! ⭐ E os dois mecanismos de preço são diferentes: a **Bezier** paga a CÚBICA (dois ramos, duas
//! raízes cúbicas e um `acos`, todos sempre); a **onda** paga o DIVISOR, que cresce com os lóbulos e
//! encurta o passo da marcha. É daqui que sai o [`ph2d_field::MAX_WAVE_LOBES`].

use ph2d_field::{FieldDoc, NodeId, Primitive, Xform, wave_thickness_ceiling};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{Orbit, trace};

fn cronometra(nome: &str, p: Primitive) -> f64 {
    let doc =
        FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("a peça");
    let reg = Registry::default();
    let cam = Orbit::default();
    let _ = trace(&doc, &reg, &cam, 320, 180);
    let t0 = std::time::Instant::now();
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("  {nome:<34} {ms:>8.1} ms");
    ms
}

#[test]
#[ignore = "sonda: o preço de um quadro com as duas curvas"]
fn the_price_of_the_curves() {
    println!(
        "\n  /proc/loadavg = {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("\n── um quadro a 640×360, uma peça só ──");
    let esfera = cronometra("esfera", Primitive::Sphere { radius: 0.5 });
    let tubo = cronometra(
        "tubo (o anel que a onda seria)",
        Primitive::Tube {
            outer: 0.50,
            inner: 0.38,
            angle: std::f32::consts::PI,
            half_height: 0.12,
            round: 0.0,
            chamfer: 0.0,
        },
    );
    println!("\n── a BEZIER, que paga a CÚBICA ──");
    for (nome, a, b, c) in [
        (
            "arco",
            [-0.35_f32, -0.20_f32],
            [0.0_f32, 0.45_f32],
            [0.35_f32, -0.20_f32],
        ),
        ("S deitado", [-0.40, 0.10], [0.10, -0.40], [0.40, 0.30]),
        (
            "recta (ramo do host)",
            [-0.40, 0.00],
            [0.00, 0.00],
            [0.40, 0.00],
        ),
    ] {
        cronometra(
            &format!("bezier — {nome}"),
            Primitive::Bezier {
                a,
                b,
                c,
                thickness: 0.06,
                half_height: 0.12,
                round: 0.02,
                chamfer: 0.0,
            },
        );
    }
    println!("\n── a ONDA, lóbulo a lóbulo (o divisor é que paga) ──");
    let mut linhas = Vec::new();
    for lobes in [1_u32, 2, 4, 8, 12, 16, 24, 32, 48] {
        let (radius, amplitude) = (0.50_f32, 0.11_f32);
        let ms = cronometra(
            &format!("onda ({lobes} lóbulos)"),
            Primitive::CircleWave {
                radius,
                amplitude,
                lobes,
                thickness: wave_thickness_ceiling(radius, amplitude) * 0.30,
                half_height: 0.12,
                round: 0.01,
                chamfer: 0.0,
            },
        );
        linhas.push((lobes, ms));
    }
    println!("\n  esfera {esfera:.1} ms · tubo {tubo:.1} ms");
    println!("  ⇒ a onda custa, por lóbulo:");
    for (lobes, ms) in linhas {
        println!(
            "     {lobes:>2} lóbulos   {:.2}× a esfera   {:.2}× o tubo",
            ms / esfera,
            ms / tubo
        );
    }
    println!(
        "\n  /proc/loadavg no fim = {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
