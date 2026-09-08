//! Sonda (`#[ignore]`) da W136: a Bezier entrega a curva que promete? quanto custa a cúbica?
//! e a PARÁBOLA é mesmo a Bezier com os três pontos no sítio certo?
use ph2d_field_eval::{Field, ops_curve};

/// O oráculo HONESTO: a distância à curva paramétrica por varredura densa em `t`.
///
/// ⚠️ Ele não partilha uma linha com o produto — sai da definição da Bezier, não da cúbica.
fn dist_a_curva(px: f64, py: f64, a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let n = 200_000_usize;
    let mut melhor = f64::INFINITY;
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let u = 1.0 - t;
        let x = u * u * a[0] + 2.0 * t * u * b[0] + t * t * c[0];
        let y = u * u * a[1] + 2.0 * t * u * b[1] + t * t * c[1];
        melhor = melhor.min((px - x).hypot(py - y));
    }
    melhor
}

fn campo(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> Field {
    let d2 = ops_curve::bezier_dist2(
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
        a,
        b,
        c,
    );
    Field::from_tree(&d2.sqrt())
}

/// Um arranjo do corpus: o nome e os três pontos de controlo.
type Arranjo = (&'static str, [f64; 2], [f64; 2], [f64; 2]);

fn corpus() -> Vec<Arranjo> {
    vec![
        ("arco simples", [-0.35, -0.20], [0.0, 0.45], [0.35, -0.20]),
        ("parábola k=2", [-0.30, 0.18], [0.0, -0.18], [0.30, 0.18]),
        ("S deitado", [-0.40, 0.10], [0.10, -0.40], [0.40, 0.30]),
        ("quase recta", [-0.40, 0.00], [0.00, 0.02], [0.40, 0.00]),
        ("ponta fechada", [-0.20, -0.30], [0.00, 0.40], [0.20, -0.30]),
    ]
}

/// ⭐⭐⭐ **A CURVA que ela promete** — o campo contra a varredura densa, na caixa toda.
#[test]
#[ignore = "sonda"]
fn probe_bezier_shape() {
    println!("  forma            |  pior erro  |  erro medio  |  amostras");
    for (nome, a, b, c) in corpus() {
        let f = campo(a, b, c);
        let (mut pior, mut soma, mut n) = (0.0_f64, 0.0_f64, 0_u32);
        for i in 0..40 {
            for j in 0..40 {
                let x = -0.5 + 1.0 * f64::from(i) / 39.0;
                let y = -0.5 + 1.0 * f64::from(j) / 39.0;
                let lido = f.at(x, y, 0.0);
                let verdade = dist_a_curva(x, y, a, b, c);
                if !lido.is_finite() {
                    println!("  {nome:16} | NAO-FINITO em ({x:.3},{y:.3})");
                    return;
                }
                let erro = (lido - verdade).abs();
                pior = pior.max(erro);
                soma += erro;
                n += 1;
            }
        }
        println!(
            "  {nome:16} | {pior:10.6}  | {:11.6}  | {n}",
            soma / f64::from(n)
        );
    }
}

/// ⭐⭐ **A PARÁBOLA É A BEZIER** — `y = k·x²` em `[−w, w]` sai dos três pontos
/// `(−w, kw²) · (0, −kw²) · (w, kw²)`, porque o ponto de controlo é o encontro das duas tangentes.
#[test]
#[ignore = "sonda"]
fn probe_parabola_is_a_bezier() {
    println!("  k     w    |  pior desvio da curva y = k x^2");
    for (k, w) in [(1.0_f64, 0.4_f64), (2.0, 0.3), (0.5, 0.45), (4.0, 0.25)] {
        let (a, b, c) = ([-w, k * w * w], [0.0, -k * w * w], [w, k * w * w]);
        let mut pior = 0.0_f64;
        for i in 0..=400 {
            let t = f64::from(i) / 400.0;
            let u = 1.0 - t;
            let x = u * u * a[0] + 2.0 * t * u * b[0] + t * t * c[0];
            let y = u * u * a[1] + 2.0 * t * u * b[1] + t * t * c[1];
            pior = pior.max((y - k * x * x).abs());
        }
        println!("  {k:.1}   {w:.2}  |  {pior:.3e}");
    }
}

/// ⭐ **O PREÇO** — quantas amostras por segundo, contra uma forma que a casa já tem.
#[test]
#[ignore = "sonda"]
fn probe_bezier_price() {
    let medir = |nome: &str, f: &Field| {
        let t = std::time::Instant::now();
        let mut soma = 0.0;
        for i in 0..200_000 {
            let a = f64::from(i) * 0.000_013;
            soma += f.at(a.sin() * 0.4, a.cos() * 0.4, 0.0);
        }
        let ns = t.elapsed().as_secs_f64() * 1.0e9 / 200_000.0;
        println!("  {nome:22} {ns:7.1} ns/ponto   [{soma:.1}]");
        ns
    };
    println!(
        "  /proc/loadavg = {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let circulo = Field::from_tree(
        &(fidget::context::Tree::x().square() + fidget::context::Tree::y().square()).sqrt(),
    );
    let base = medir("circulo (referencia)", &circulo);
    let (_, a, b, c) = corpus()[0];
    let bez = medir("bezier quadratica", &campo(a, b, c));
    println!("  ⇒ a bezier custa {:.1}x um circulo", bez / base);
}

/// ⭐⭐⭐ **A FORMA DO FILETE NO ARO DA ONDA** — ele é um círculo ou uma ELIPSE?
///
/// A hipótese: o campo da onda leva um divisor constante `1/lip`, e a junta do aro, que supõe duas
/// distâncias honestas, devolve um arco esticado `lip` vezes na direcção da PAREDE.
///
/// ⚠️ **Mede-se o RECUO nas duas direcções** — em `z` (contra a tampa) e em `ρ` (contra a parede) —
/// e compara-se com o `round` pedido. *Uma teoria sobre a forma de um filete verifica-se medindo o
/// filete, não relendo o operador.*
#[test]
#[ignore = "sonda"]
fn probe_wave_fillet_is_round_or_oval() {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
    let (radius, amplitude, lobes, half_height) = (0.38_f64, 0.11_f64, 7_u32, 0.12_f64);
    let thickness = f64::from(ph2d_field::wave_thickness_ceiling(0.38, 0.11)) * 0.30;
    let dentro = radius - amplitude - thickness;
    let lip = (1.0 + (amplitude * f64::from(lobes) / dentro).powi(2)).sqrt();
    println!("  lip = {lip:.3}   (amplitude {amplitude} · lobes {lobes} · dentro {dentro:.3})");
    println!("  round |  recuo em Z  |  recuo em RHO |  razao rho/z");
    for frac in [0.0_f64, 0.25, 0.5] {
        #[allow(clippy::cast_possible_truncation)]
        let round = (thickness.min(half_height) * frac) as f32;
        let p = Primitive::CircleWave {
            radius: 0.38,
            amplitude: 0.11,
            lobes,
            #[allow(clippy::cast_possible_truncation)]
            thickness: thickness as f32,
            #[allow(clippy::cast_possible_truncation)]
            half_height: half_height as f32,
            round,
            chamfer: 0.0,
        };
        let doc = FieldDoc::new(
            vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
            NodeId(0),
        )
        .expect("a peça");
        let f = Field::new(&doc);
        // ⚠️ A CRISTA de um lóbulo, onde `R' = 0` — ali a onda é localmente um anel.
        let phi = std::f64::consts::FRAC_PI_2 / f64::from(lobes);
        let r_crista = radius + amplitude;
        // recuo em Z: onde a parede externa (rho = r_crista + thickness) deixa de existir
        let mut z_fim = 0.0;
        for i in 0..=4000 {
            let z = f64::from(i) / 4000.0 * half_height;
            let rr = r_crista + thickness - 1.0e-5;
            if f.at(rr * phi.cos(), rr * phi.sin(), z) <= 0.0 {
                z_fim = z;
            }
        }
        // recuo em RHO: onde a tampa (z = half_height) deixa de existir, indo para fora
        let mut rho_fim = r_crista;
        for i in 0..=4000 {
            let rr = r_crista + f64::from(i) / 4000.0 * thickness;
            if f.at(rr * phi.cos(), rr * phi.sin(), half_height - 1.0e-5) <= 0.0 {
                rho_fim = rr;
            }
        }
        let (rz, rrho) = (half_height - z_fim, r_crista + thickness - rho_fim);
        println!(
            "  {round:.4} |  {rz:10.5} |  {rrho:12.5} | {:11.2}",
            if rz > 1.0e-6 { rrho / rz } else { f64::NAN }
        );
    }
}

/// ⭐ **A FOLGA da onda** — com o divisor LOCAL o filete fica redondo e o minorante perde-se; este
/// varre a região permitida e devolve o pior `‖∇f‖`, que é de onde a folga sai.
#[test]
#[ignore = "sonda"]
fn probe_wave_gradient() {
    use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
    println!("  amplitude lobes  round |   grad caixa   grad pele   onde");
    let mut pior_global = 0.0_f64;
    for (amp, lob, raio) in [
        (0.11_f32, 7_u32, 0.38_f32),
        (0.11, 7, 0.80),
        (0.11, 7, 1.60),
        (0.11, 1, 0.38),
        (0.04, 2, 0.38),
    ] {
        for frac in [0.0_f32, 0.5, 0.99] {
            let t = ph2d_field::wave_thickness_ceiling(0.38, amp) * 0.30;
            let _ = raio;
            let round = t.min(0.12) * frac;
            let p = Primitive::CircleWave {
                radius: raio,
                amplitude: amp,
                lobes: lob,
                thickness: t,
                half_height: 0.12,
                round,
                chamfer: 0.0,
            };
            let Ok(doc) = FieldDoc::new(
                vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p.clone()))],
                NodeId(0),
            ) else {
                println!("  {amp:9.2} {lob:5}  {frac:.2}  |  o documento RECUSA");
                continue;
            };
            let f = Field::new(&doc);
            let (mut caixa, mut pele) = (0.0_f64, 0.0_f64);
            let mut onde = [0.0_f64; 3];
            let mut varre = |nn: usize, e: f64, so_pele: bool| {
                let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / nn as f64;
                for i in 0..nn {
                    for j in 0..nn {
                        for k in 0..nn {
                            let (x, y, z) = (at(i), at(j), at(k));
                            let v = f.at(x, y, z);
                            if !v.is_finite() || (so_pele && v.abs() > 0.01) {
                                continue;
                            }
                            let g = f.gradient_norm(x, y, z, 1.0e-5);
                            if !g.is_finite() {
                                continue;
                            }
                            if so_pele {
                                pele = pele.max(g);
                            }
                            if g > caixa {
                                caixa = g;
                                onde = [x, y, z];
                            }
                        }
                    }
                }
            };
            let e = f64::from(ph2d_field::bounding_radius(&p)) * 1.05;
            varre(38, e, false);
            varre(120, e * 0.98, true);
            pior_global = pior_global.max(caixa);
            println!(
                "  {amp:9.2} {lob:5}  {frac:.2}  |  {caixa:10.4}  {pele:10.4}   em ({:.3},{:.3},{:.3})  rho {:.3}",
                onde[0],
                onde[1],
                onde[2],
                onde[0].hypot(onde[1])
            );
        }
    }
    println!(
        "\n  ⇒ pior de toda a varredura: {pior_global:.4}   ⇒ folga = {:.4}",
        1.0 / pior_global
    );
}
