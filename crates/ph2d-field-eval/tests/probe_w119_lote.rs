//! ⭐ **A SONDA DO LOTE DA SETA** (W119) — gradiente e extensão por eixo das seis formas novas, numa
//! tabela.
//!
//! ⚠️ **`#[ignore]`, e é um INSTRUMENTO, não um gate**: quem reprova é o censo
//! (`the_census_of_every_primitive`), que corre as `34` derivadas de `PrimitiveKind::ALL`. Esta
//! existe porque o censo custa `145 s` e uma iteração de fórmula precisa de segundos.
//!
//! `cargo test -p ph2d-field-eval --test probe_w119_lote -- --ignored --nocapture`

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn as_seis() -> Vec<(&'static str, Primitive)> {
    vec![
        (
            "arrow",
            Primitive::Arrow {
                heads: 1,
                half_length: 0.45,
                shaft: 0.09,
                head: 0.24,
                head_length: 0.26,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        (
            "arrow(2)",
            Primitive::Arrow {
                heads: 2,
                half_length: 0.45,
                shaft: 0.09,
                head: 0.24,
                head_length: 0.26,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        (
            "chevron",
            Primitive::Chevron {
                half_length: 0.40,
                half_span: 0.30,
                thickness: 0.09,
                half_height: 0.10,
                round: 0.02,
                chamfer: 0.0,
            },
        ),
        (
            "bent_arrow",
            Primitive::BentArrow {
                run: 0.42,
                rise: 0.34,
                shaft: 0.08,
                head: 0.18,
                head_length: 0.20,
                half_height: 0.10,
                round: 0.02,
                chamfer: 0.0,
            },
        ),
        (
            "rhombus",
            Primitive::Rhombus {
                half_width: 0.45,
                half_span: 0.26,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        (
            "tube",
            Primitive::Tube {
                outer: 0.45,
                inner: 0.26,
                angle: 1.1,
                half_height: 0.12,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        (
            "circle_segment",
            Primitive::CircleSegment {
                radius: 0.45,
                cut: 0.16,
                half_height: 0.10,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        // ⛔ **OS CONTROLES** — três formas que já shipam, medidas pelo MESMO instrumento. Sem elas
        // não há como saber se um número desta tabela é da wave ou da casa.
        (
            "~cross",
            Primitive::Cross {
                arm: 0.45,
                width: 0.14,
                half_height: 0.12,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
        (
            "~gear",
            Primitive::Gear {
                teeth: 7,
                root: 0.32,
                outer: 0.45,
                tooth: 0.45,
                half_height: 0.15,
                round: 0.02,
                chamfer: 0.0,
            },
        ),
        (
            "~pie",
            Primitive::Pie {
                radius: 0.45,
                angle: 1.0,
                half_height: 0.12,
                round: 0.03,
                chamfer: 0.0,
            },
        ),
    ]
}

/// Onde exactamente está o ponto mais afastado num eixo — a pergunta que a tabela não responde.
#[test]
#[ignore = "instrumento"]
fn probe_w119_onde() {
    let p = Primitive::BentArrow {
        run: 0.42,
        rise: 0.34,
        shaft: 0.08,
        head: 0.18,
        head_length: 0.20,
        half_height: 0.10,
        round: 0.02,
        chamfer: 0.0,
    };
    let p2 = p.clone();
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("peça");
    let f = Field::new(&doc);
    let n = 260usize;
    let at = |t: usize| -0.7 + 1.4 * (t as f64 + 0.5) / n as f64;
    let mut melhor = (0.0f64, [0.0f64; 3]);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let (x, y, z) = (at(i), at(j), at(k));
                if f.at(x, y, z) < 0.0 && y > melhor.0 {
                    melhor = (y, [x, y, z]);
                }
            }
        }
    }
    println!("o ponto mais alto (grelha): {:?}  (rise = 0,34)", melhor.1);
    // Varredura FINA só na banda que interessa.
    let m = 400usize;
    let mut alto = (0.0f64, [0.0f64; 3]);
    for i in 0..m {
        for j in 0..m {
            for k in 0..60 {
                let x = -0.7 + 1.4 * (i as f64 + 0.5) / m as f64;
                let y = 0.32 + 0.05 * (j as f64 + 0.5) / m as f64;
                let z = -0.12 + 0.24 * (k as f64 + 0.5) / 60.0;
                if f.at(x, y, z) < 0.0 && y > alto.0 {
                    alto = (y, [x, y, z]);
                }
            }
        }
    }
    println!("o ponto mais alto (banda fina): {:?}", alto.1);
    // ⭐ A MESMA varredura por raios do censo — para achar a direcção em que as duas réguas
    // discordam, e depois PERGUNTAR AO CAMPO quem tem razão.
    let r = f64::from(ph2d_field::bounding_radius(&p2)) * 1.001;
    const DIRS: usize = 96;
    const AMOSTRAS: usize = 256;
    let far = r * 4.0;
    let mut pior = (0.0f64, [0.0f64; 3], 0.0f64);
    for i in 0..DIRS {
        for j in 0..(DIRS * 2) {
            let theta = std::f64::consts::PI * (i as f64 + 0.5) / DIRS as f64;
            let phi = std::f64::consts::TAU * (j as f64 + 0.5) / (DIRS * 2) as f64;
            let d = [
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            ];
            let at = |t: f64| f.at(d[0] * t, d[1] * t, d[2] * t);
            let mut dentro: Option<f64> = None;
            for n in (1..=AMOSTRAS).rev() {
                let t = far * n as f64 / AMOSTRAS as f64;
                if at(t) < 0.0 {
                    dentro = Some(t);
                    break;
                }
            }
            let Some(mut lo) = dentro else { continue };
            let mut hi = (lo + far / AMOSTRAS as f64).min(far * 1.001);
            for _ in 0..40 {
                let mid = 0.5 * (lo + hi);
                if at(mid) < 0.0 { lo = mid } else { hi = mid }
            }
            let y = (d[1] * hi).abs();
            if y > pior.0 {
                pior = (y, [d[0] * hi, d[1] * hi, d[2] * hi], at(lo));
            }
        }
    }
    println!(
        "o raio mais alto: y = {:.4} em {:?}, e o campo ali vale {:.6}",
        pior.0, pior.1, pior.2
    );
    println!(
        "   campo no ponto: {:.6}",
        f.at(pior.1[0], pior.1[1], pior.1[2])
    );
}

#[test]
#[ignore = "instrumento: imprime a tabela, o veredito é do censo"]
fn probe_w119_lote() {
    println!(
        "{:16} {:>8} {:>10}   {:^24}  {:^24}",
        "forma", "‖∇f‖", "passo·‖∇f‖", "extensão medida", "caixa declarada"
    );
    for (nome, p) in as_seis() {
        for chanfro in [0.0_f32, 0.5] {
            let mut peca = p.clone();
            if chanfro > 0.0 {
                let Some(limite) = ph2d_field::round_limit(&peca) else {
                    continue;
                };
                let linha = ph2d_field::dims(&peca)
                    .iter()
                    .position(|d| d.key == "field.dim.chamfer")
                    .expect("chanfro");
                let _ = ph2d_field::set_dim(&mut peca, 0, linha, limite * chanfro);
            }
            let doc = FieldDoc::new(
                vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(peca.clone()))],
                NodeId(0),
            )
            .expect("peça");
            let f = Field::new(&doc);
            let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
            let (mut g, mut ext) = (0.0f64, [0.0f64; 3]);
            let n = 90usize;
            let at = |t: usize| -0.8 + 1.6 * (t as f64 + 0.5) / n as f64;
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        let (x, y, z) = (at(i), at(j), at(k));
                        let v = f.at(x, y, z);
                        if v < 0.0 {
                            for (eixo, c) in [x, y, z].iter().enumerate() {
                                ext[eixo] = ext[eixo].max(c.abs());
                            }
                        }
                        if v.abs() < 0.04 {
                            let gg = f.gradient_norm(x, y, z, 1.0e-4);
                            if gg.is_finite() {
                                g = g.max(gg);
                            }
                        }
                    }
                }
            }
            let c = ph2d_field::bounding_half_extents(&peca);
            let rotulo = if chanfro > 0.0 {
                format!("{nome}+chanfro")
            } else {
                nome.to_string()
            };
            println!(
                "{rotulo:16} {g:8.4} {:10.4}   {:6.4} {:6.4} {:6.4}   {:6.4} {:6.4} {:6.4}",
                passo * g,
                ext[0],
                ext[1],
                ext[2],
                c[0],
                c[1],
                c[2]
            );
        }
    }
}
