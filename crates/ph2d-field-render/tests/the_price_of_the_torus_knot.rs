//! ⭐ **SONDA da W134** — quanto custa uma volta do nó de toro, medido no **QUADRO**.
//!
//! ⚠️ **A régua tem de ser o quadro, e não uma varredura de campo.** A W128 mediu `3,8×` uma esfera
//! **por amostra** e concluiu que o preço estava bem; a árvore é **especializada por ladrilho ×
//! fatia**, e o que viaja com ela é outra coisa (doc 06 §130.1). O tecto de `p` sai daqui.

use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_field_render::{Orbit, trace};

fn doc_de(p: Primitive) -> FieldDoc {
    FieldDoc::new(vec![ph2d_field_eval::leaf(p, Xform::IDENTITY)], NodeId(0)).expect("a peça")
}

fn cronometra(nome: &str, p: Primitive) -> f64 {
    let doc = doc_de(p);
    let reg = Registry::default();
    let cam = Orbit::default();
    let _ = trace(&doc, &reg, &cam, 320, 180);
    let t0 = std::time::Instant::now();
    let _ = trace(&doc, &reg, &cam, 640, 360);
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("  {nome:<26} {ms:>8.1} ms");
    ms
}

fn no(winds: u32, loops: u32) -> Primitive {
    Primitive::TorusKnot {
        radius: 0.55,
        tube: 0.24,
        cord: ph2d_field::knot_cord_ceiling(0.55, 0.24, winds, loops) * 0.55,
        winds,
        loops,
    }
}

#[test]
#[ignore = "sonda: o preço de um quadro com o nó de toro"]
fn the_price_of_the_torus_knot() {
    println!("\n── um quadro a 640×360, uma peça só ──");
    let esfera = cronometra("esfera", Primitive::Sphere { radius: 0.5 });
    let toro = cronometra(
        "toro",
        Primitive::Torus {
            major: 0.55,
            minor: 0.24,
        },
    );
    println!("\n── e o nó, volta a volta (q = 3) ──");
    let mut linhas = Vec::new();
    for p in [1_u32, 2, 3, 4, 6, 8, 10, 12] {
        linhas.push((p, cronometra(&format!("nó (p = {p}, q = 3)"), no(p, 3))));
    }
    println!("\n── e voltando ao tubo, no PIOR `q/p` que existe (p = 1) ──");
    // ⚠️ O tecto de `q` é `4·p`, então com `p = 1` a faixa acaba em `4` — pedir mais é uma peça que
    // o documento RECUSA, e a sonda estava a rebentar aí em silêncio.
    for (pp, qq) in [
        (1_u32, 1_u32),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 8),
        (3, 12),
        (12, 48),
    ] {
        cronometra(&format!("nó (p = {pp}, q = {qq})"), no(pp, qq));
    }
    println!("\n  esfera {esfera:.1} ms · toro {toro:.1} ms");
    println!("  ⇒ o nó custa, por volta:");
    for (p, ms) in linhas {
        println!(
            "     p = {p:>2}   {:.1}× a esfera   {:.1}× o toro",
            ms / esfera,
            ms / toro
        );
    }
}
