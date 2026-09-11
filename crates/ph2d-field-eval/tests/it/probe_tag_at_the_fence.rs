//! Sonda (`#[ignore]`): a `tag` estreita lê `1,0232` contra uma barra de `1,02` — é a FORMA ou é a
//! RÉGUA? Mede onde está o pior ponto e como ele responde à densidade da grelha.
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn doc(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça")
}

/// Devolve `(pior, onde, f_ali)` numa grelha de `steps³` sobre `[-e, e]³`, opcionalmente só na
/// casca de meia-espessura `banda`.
fn pior_em(f: &Field, e: f64, steps: usize, banda: Option<f64>) -> (f64, [f64; 3], f64) {
    let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
    let mut pior = 0.0f64;
    let mut onde = [0.0; 3];
    let mut vali = 0.0;
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                let (x, y, z) = (at(i), at(j), at(k));
                let v = f.at(x, y, z);
                if banda.is_some_and(|b| v.abs() > b) {
                    continue;
                }
                let g = f.gradient_norm(x, y, z, 1.0e-4);
                if g.is_finite() && g > pior {
                    pior = g;
                    onde = [x, y, z];
                    vali = v;
                }
            }
        }
    }
    (pior, onde, vali)
}

fn tag_estreita() -> Primitive {
    let mut p = Primitive::Tag {
        half_width: 0.45,
        half_span: 0.26,
        point: 0.24,
        hole: 0.07,
        half_height: 0.10,
        round: 0.03,
        chamfer: 0.0,
    };
    ph2d_field::set_dim(&mut p, 0, 0, 0.225).expect("a largura");
    p
}

#[test]
#[ignore]
fn probe_tag_at_the_fence() {
    let p = tag_estreita();
    let d = doc(p.clone());
    let passo = f64::from(ph2d_field_eval::safe_march_step(&d));
    let f = Field::new(&d);
    println!("peça: {p:?}\npasso da marcha = {passo:.4}  (barra do censo: 1,02)\n");

    let (g, o, v) = pior_em(&f, 1.0, 20, None);
    println!(
        "grossa 20³ ±1,0 (a do censo): ‖∇f‖ {g:.4} -> {:.4}  em {o:?}, f = {v:.5}",
        passo * g
    );
    let (g, o, v) = pior_em(&f, 0.85, 78, Some(0.03));
    println!(
        "fina 78³ ±0,85 casca 0,03  : ‖∇f‖ {g:.4} -> {:.4}  em {o:?}, f = {v:.5}\n",
        passo * g
    );

    println!("── a MESMA caixa, adensando a grelha grossa ──");
    for n in [20_usize, 30, 40, 60, 80, 120] {
        let (g, o, v) = pior_em(&f, 1.0, n, None);
        println!(
            "  {n:>3}³ ±1,0: ‖∇f‖ {g:.4} -> {:.4}  em [{:.3}, {:.3}, {:.3}], f = {v:.5}",
            passo * g,
            o[0],
            o[1],
            o[2]
        );
    }
    println!("\n── e a casca, adensando ──");
    for n in [78_usize, 120, 160] {
        let (g, _, _) = pior_em(&f, 0.85, n, Some(0.03));
        println!("  {n:>3}³ casca 0,03: ‖∇f‖ {g:.4} -> {:.4}", passo * g);
    }

    println!("\n── LARGURA × (ponta/largura), régua GROSSA do censo (barra 1,02) ──");
    print!("        ponta/w:");
    for pf in [0.25_f32, 0.4, 0.55, 0.7, 0.85, 0.999] {
        print!("  {pf:>6.3}");
    }
    println!();
    for largura in [0.9_f32, 0.6, 0.4, 0.3, 0.225, 0.15] {
        print!("  w = {largura:.3}:  ");
        for pf in [0.25_f32, 0.4, 0.55, 0.7, 0.85, 0.999] {
            let q = Primitive::Tag {
                half_width: largura * 0.5,
                half_span: 0.26,
                point: largura * pf,
                hole: largura * 0.15 * 0.999,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            };
            match std::panic::catch_unwind(|| {
                let dd = doc(q.clone());
                let pp = f64::from(ph2d_field_eval::safe_march_step(&dd));
                let (gg, _, _) = pior_em(&Field::new(&dd), 1.0, 20, None);
                pp * gg
            }) {
                Ok(v) => print!("  {v:>6.3}"),
                Err(_) => print!("       -"),
            }
        }
        println!();
    }

    println!("\n── e o VÃO (span) a variar, largura 0,225, ponta a 0,999·w ──");
    for vao in [0.52_f32, 0.4, 0.3, 0.22, 0.15, 0.10] {
        let q = Primitive::Tag {
            half_width: 0.1125,
            half_span: vao * 0.5,
            point: 0.224775,
            hole: 0.03371625,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        };
        let dd = doc(q.clone());
        let pp = f64::from(ph2d_field_eval::safe_march_step(&dd));
        let (gg, _, _) = pior_em(&Field::new(&dd), 1.0, 20, None);
        println!(
            "  vão {vao:.3} (vão/largura = {:.2}): {:.4}",
            vao / 0.225,
            pp * gg
        );
    }
}
