//! Sonda de bancada da W123 — a espiral e o documento, ANTES de serem ligados ao documento.
//!
//! `cargo test -p ph2d-field-eval --test probe_w123_curves -- --ignored --nocapture`

use ph2d_field_eval::Field;

/// O maior `‖∇f‖` numa casca fina em volta da superfície.
fn pior_gradiente(f: &Field, e: f64, passos: usize) -> f64 {
    let mut pior: f64 = 0.0;
    let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / passos as f64;
    for i in 0..passos {
        for j in 0..passos {
            for k in 0..passos {
                let (x, y, z) = (at(i), at(j), at(k));
                if f.at(x, y, z).abs() > 0.03 {
                    continue;
                }
                pior = pior.max(f.gradient_norm(x, y, z, 1.0e-4));
            }
        }
    }
    pior
}

#[test]
#[ignore = "instrumento"]
fn probe_spiral_and_document() {
    println!("\n── ESPIRAL: raio 0,10 · pitch 0,12 · voltas 3 · espessura 0,03 ──");
    let esp = Field::from_tree(&ph2d_field_eval::ops_spiral::sd_spiral(
        0.10, 0.12, 3.0, 0.03, 0.08, 0.0, 0.0,
    ));
    // Na fita: o raio da volta k no ângulo 0 é r0 + pitch*k.
    for (nome, x, y) in [
        ("centro da 1.ª volta (r=0,10)", 0.10, 0.0),
        ("centro da 2.ª volta (r=0,22)", 0.22, 0.0),
        ("centro da 3.ª volta (r=0,34)", 0.34, 0.0),
        ("vale entre a 1.ª e a 2.ª", 0.16, 0.0),
        ("dentro do olho (r=0,04)", 0.04, 0.0),
        ("fora do fim (r=0,50)", 0.50, 0.0),
    ] {
        println!("  {nome:32} f = {:+.4}", esp.at(x, y, 0.0));
    }
    println!(
        "  pior ‖∇f‖ na casca = {:.4}",
        pior_gradiente(&esp, 0.55, 60)
    );

    println!("\n── DOCUMENTO: w 0,40 · s 0,25 · onda 0,08 ──");
    let doc = Field::from_tree(&ph2d_field_eval::ops_flowchart::sd_document(
        0.40, 0.25, 0.08, 0.08, 0.0, 0.0,
    ));
    for (nome, x, y) in [
        ("o meio", 0.0, 0.0),
        ("o topo", 0.0, 0.25),
        ("o flanco direito", 0.40, 0.0),
        ("a base no meio (y = −0,25)", 0.0, -0.25),
        ("o VALE da onda (x=−0,2)", -0.20, -0.33),
        ("a CRISTA da onda (x=+0,2)", 0.20, -0.17),
        ("abaixo do vale", -0.20, -0.36),
        ("sob a crista, onde já não há peça", 0.20, -0.22),
    ] {
        println!("  {nome:34} f = {:+.4}", doc.at(x, y, 0.0));
    }
    println!(
        "  pior ‖∇f‖ na casca = {:.4}",
        pior_gradiente(&doc, 0.5, 60)
    );
}

/// A varredura que escolhe as três paredes da W123.
#[test]
#[ignore = "instrumento"]
fn probe_w123_fences() {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
    let marcha = |p: &Primitive| -> Option<String> {
        let mut q = p.clone();
        ph2d_field::clamp_round(&mut q);
        let doc = FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(q))],
            NodeId(0),
        )
        .ok()?;
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        let f = Field::new(&doc);
        Some(format!("{:.4}", passo * pior_gradiente(&f, 1.0, 70)))
    };
    println!("\n── ESPIRAL: VOLTAS ──");
    for t in [1.0, 2.0, 4.0, 8.0, 12.0, 20.0, 32.0] {
        let p = Primitive::Spiral {
            radius: 0.09,
            pitch: 0.15,
            turns: t,
            thickness: 0.04,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        };
        println!("  voltas {t:5.1}: {:?}", marcha(&p));
    }
    println!("\n── ESPIRAL: 2·espessura / passo ──");
    for f in [0.2, 0.4, 0.6, 0.8, 0.9, 0.95, 0.99] {
        let p = Primitive::Spiral {
            radius: 0.09,
            pitch: 0.15,
            turns: 3.0,
            thickness: 0.15 * f * 0.5,
            half_height: 0.10,
            round: 0.01,
            chamfer: 0.0,
        };
        println!("  enchimento {f:4.2}: {:?}", marcha(&p));
    }
    println!("\n── DOCUMENTO: onda / half_span ──");
    for f in [0.0, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0] {
        let p = Primitive::Document {
            half_width: 0.42,
            half_span: 0.26,
            wave: 0.26 * f,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        };
        println!("  onda {f:4.2}: {:?}", marcha(&p));
    }
}
