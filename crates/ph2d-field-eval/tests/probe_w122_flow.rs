//! ⭐ **A VARREDURA das cercas do fluxograma** (W122) — o instrumento que escolhe cada parede.
//!
//! ⚠️ **Ela arrasta cada grandeza PELA PORTA DO PAINEL** (`ph2d_field::set_dim`), e não constrói a
//! primitiva à mão: uma sonda que arma a forma de outra maneira mede outro programa.
//!
//! ⚠️ **E o filete é o do REPRESENTANTE**, não um número à mão — *uma sonda com o knob noutro ponto
//! mede outra peça* (a lição que o escudo pagou na W121).
//!
//! `cargo test -p ph2d-field-eval --test probe_w122_flow -- --ignored --nocapture`

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn marcha(p: &Primitive) -> Option<f64> {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
        NodeId(0),
    )
    .ok()?;
    let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
    let f = Field::new(&doc);
    let mut pior = 0.0f64;
    let mut varre = |e: f64, passos: usize, banda: Option<f64>| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / passos as f64;
        for i in 0..passos {
            for j in 0..passos {
                for k in 0..passos {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if banda.is_some_and(|b| f.at(x, y, z).abs() > b) {
                        continue;
                    }
                    let h = 1.0e-4;
                    let gx = (f.at(x + h, y, z) - f.at(x - h, y, z)) / (2.0 * h);
                    let gy = (f.at(x, y + h, z) - f.at(x, y - h, z)) / (2.0 * h);
                    let gz = (f.at(x, y, z + h) - f.at(x, y, z - h)) / (2.0 * h);
                    pior = pior.max((gx * gx + gy * gy + gz * gz).sqrt());
                }
            }
        }
    };
    varre(1.0, 22, None);
    varre(0.85, 70, Some(0.03));
    Some(passo * pior)
}

#[test]
#[ignore = "instrumento"]
fn probe_flow_fences() {
    println!("\n── PARALELOGRAMO: skew / half_span ──");
    for r in [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0] {
        let p = Primitive::Parallelogram {
            half_width: 0.38,
            half_span: 0.28,
            skew: 0.28 * r,
            half_height: 0.10,
            round: 0.04,
            chamfer: 0.0,
        };
        let mut q = p.clone();
        ph2d_field::clamp_round(&mut q);
        println!("  k = {r:.2}: {:?}", marcha(&q).map(|v| format!("{v:.4}")));
    }

    println!("\n── ATRASO: half_span / half_width ──");
    for r in [
        0.30, 0.70, 1.00, 1.30, 1.60, 1.90, 2.00, 2.20, 2.60, 3.00, 4.00,
    ] {
        let p = Primitive::Delay {
            half_width: 0.45,
            half_span: 0.45 * r,
            half_height: 0.10,
            round: 0.04,
            chamfer: 0.0,
        };
        let mut q = p.clone();
        ph2d_field::clamp_round(&mut q);
        println!(
            "  s/w = {r:.2}: {:?}",
            marcha(&q).map(|v| format!("{v:.4}"))
        );
    }

    for chamfer in [0.0f32, 0.02] {
        println!("\n[chanfro = {chamfer}]");
        println!("\n── MOSTRADOR: bico / (2w − s), com s/w = 0,58 ──");
        let (w, s) = (0.45f32, 0.26f32);
        for f in [0.0, 0.30, 0.70, 1.00, 1.20, 1.50, 2.00, 2.50] {
            let p = Primitive::Display {
                half_width: w,
                half_span: s,
                point: (2.0 * w - s) * f,
                half_height: 0.10,
                round: 0.04,
                chamfer,
            };
            let mut q = p.clone();
            ph2d_field::clamp_round(&mut q);
            println!("  f = {f:.2}: {:?}", marcha(&q).map(|v| format!("{v:.4}")));
        }

        println!("\n── CONECTOR: bico / (2 × half_span) ──");
        for f in [0.0, 0.30, 0.70, 1.00, 1.30, 1.60, 2.00, 2.50, 3.00] {
            let p = Primitive::OffPage {
                half_width: 0.36,
                half_span: 0.42,
                point: 0.84 * f,
                half_height: 0.10,
                round: 0.04,
                chamfer,
            };
            let mut q = p.clone();
            ph2d_field::clamp_round(&mut q);
            println!("  f = {f:.2}: {:?}", marcha(&q).map(|v| format!("{v:.4}")));
        }
    }
}
