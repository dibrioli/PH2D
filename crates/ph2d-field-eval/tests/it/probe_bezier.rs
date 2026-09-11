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

// ─────────────────────────── W137 ───────────────────────────

/// A distância EXACTA do ponto ao sólido da onda, por varredura densa do corpo.
///
/// ⚠️ Ele não partilha uma linha com o produto: sai da DEFINIÇÃO do conjunto
/// `{ |ρ − R(φ)| ≤ t, |z| ≤ h }`, varrendo `φ`, as duas paredes em `ρ` e as duas tampas.
fn dist_a_onda(
    p: [f64; 3],
    radius: f64,
    amplitude: f64,
    lobes: u32,
    thickness: f64,
    half_height: f64,
) -> f64 {
    let n = 4_000_usize;
    let mut melhor = f64::INFINITY;
    for i in 0..n {
        let phi = std::f64::consts::TAU * i as f64 / n as f64;
        let r = radius + amplitude * (f64::from(lobes) * phi).sin();
        for k in 0..=40 {
            let rr = r - thickness + 2.0 * thickness * f64::from(k) / 40.0;
            for m in 0..=8 {
                let z = -half_height + 2.0 * half_height * f64::from(m) / 8.0;
                let d = (p[0] - rr * phi.cos())
                    .hypot(p[1] - rr * phi.sin())
                    .hypot(p[2] - z);
                melhor = melhor.min(d);
            }
        }
    }
    melhor
}

/// ⭐⭐⭐ **O CAMPO LONGE DA PEÇA** — os dois relatos do dono numa régua só.
///
/// *"afeta/deforma tudo que está na sua direção em x mesmo se estiver distante"* e *"tem performance
/// ruim"* são o MESMO número: quanto o campo devolve num ponto afastado. A junta mistura por
/// diferença de campo, e a marcha ANDA o campo — um campo que satura num valor pequeno agarra
/// vizinhos longínquos **e** obriga a marcha a dar centenas de passos no vazio.
#[test]
#[ignore = "sonda"]
fn probe_wave_far_field() {
    let (radius, amplitude, lobes, half_height) = (0.35_f64, 0.08_f64, 6_u32, 0.10_f64);
    let thickness = 0.30 * 0.90 * (radius - amplitude);
    let p = ph2d_field::Primitive::CircleWave {
        radius: radius as f32,
        amplitude: amplitude as f32,
        lobes,
        thickness: thickness as f32,
        half_height: half_height as f32,
        round: 0.0,
        chamfer: 0.0,
    };
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field::Node::new(
            ph2d_field::Xform::IDENTITY,
            ph2d_field::NodeKind::Leaf(p),
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a peca tem de ser aceite");
    let f = Field::new(&doc);
    println!("  ponto                    |   campo   |  verdade  |  campo/verdade");
    for (nome, q) in [
        ("no aro (controlo)", [radius + thickness + 0.02, 0.0, 0.0]),
        ("+x  0,5", [0.5, 0.0, 0.0]),
        ("+x  1,0", [1.0, 0.0, 0.0]),
        ("+x  2,0", [2.0, 0.0, 0.0]),
        ("+y  2,0", [0.0, 2.0, 0.0]),
        ("diagonal 2,0", [1.414, 1.414, 0.0]),
        ("+z  2,0", [0.0, 0.0, 2.0]),
        ("no eixo (controlo)", [0.0, 0.0, 0.0]),
    ] {
        let lido = f.at(q[0], q[1], q[2]);
        let verdade = dist_a_onda(q, radius, amplitude, lobes, thickness, half_height);
        println!(
            "  {nome:24} | {lido:9.5} | {verdade:9.5} | {:9.4}",
            lido / verdade
        );
    }
}

/// Quantos passos a marcha de esferas dá desde `de` até tocar a peça, na direcção da origem.
///
/// ⭐ **É a régua do PREÇO que não é um relógio** — a contagem é determinística, logo vale sob
/// qualquer carga da máquina (§5.0). Um campo que satura longe não muda a FORMA: ele multiplica
/// esta contagem, e é isso que o dono vê como *«performance ruim»*.
fn passos_da_marcha(f: &Field, doc: &ph2d_field::FieldDoc, de: [f64; 3]) -> u32 {
    let passo = f64::from(ph2d_field_eval::safe_march_step(doc));
    // ⚠️ **O raio aponta de RASPÃO, e não ao centro.** Apontado à origem, uma esfera resolve-se num
    // passo — o campo exacto entrega a distância certa e o primeiro salto aterra na superfície. Um
    // controlo que lê `1` não distingue uma régua boa de uma partida; de raspão ele lê a dezena que
    // a marcha de esferas de facto custa.
    let alvo = [0.0, 0.30, 0.0];
    let v = [alvo[0] - de[0], alvo[1] - de[1], alvo[2] - de[2]];
    let n = (de[0] * de[0] + de[1] * de[1] + de[2] * de[2]).sqrt();
    let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let dir = [v[0] / m, v[1] / m, v[2] / m];
    let (mut p, mut andou, mut passos) = (de, 0.0_f64, 0_u32);
    while passos < 20_000 && andou < 4.0 * n {
        let d = f.at(p[0], p[1], p[2]);
        if d < 1.0e-4 {
            break;
        }
        let avanco = d * passo;
        for k in 0..3 {
            p[k] += dir[k] * avanco;
        }
        andou += avanco;
        passos += 1;
    }
    passos
}

/// ⭐⭐⭐ **O PREÇO DA ONDA MEDIDO EM PASSOS** — o segundo relato do dono (*«tem performance ruim»*).
#[test]
#[ignore = "sonda"]
fn probe_wave_march_steps() {
    println!("  peca            | de 1,0 | de 2,0 | de 4,0");
    for (nome, p) in [
        (
            "esfera (controlo)",
            ph2d_field::Primitive::Sphere { radius: 0.35 },
        ),
        (
            "tubo (controlo)",
            ph2d_field::Primitive::Tube {
                outer: 0.43,
                inner: 0.27,
                angle: std::f32::consts::PI,
                half_height: 0.10,
                round: 0.0,
                chamfer: 0.0,
            },
        ),
        (
            "onda 6 lobulos",
            ph2d_field::Primitive::CircleWave {
                radius: 0.35,
                amplitude: 0.08,
                lobes: 6,
                thickness: 0.30 * 0.90 * (0.35 - 0.08),
                half_height: 0.10,
                round: 0.0,
                chamfer: 0.0,
            },
        ),
        (
            "onda 12 lobulos",
            ph2d_field::Primitive::CircleWave {
                radius: 0.35,
                amplitude: 0.08,
                lobes: 12,
                thickness: 0.30 * 0.90 * (0.35 - 0.08),
                half_height: 0.10,
                round: 0.0,
                chamfer: 0.0,
            },
        ),
    ] {
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field::Node::new(
                ph2d_field::Xform::IDENTITY,
                ph2d_field::NodeKind::Leaf(p),
            )],
            ph2d_field::NodeId(0),
        )
        .expect("a peca tem de ser aceite");
        let f = Field::new(&doc);
        let a = passos_da_marcha(&f, &doc, [1.0, 0.0, 0.0]);
        let b = passos_da_marcha(&f, &doc, [2.0, 0.0, 0.0]);
        let c = passos_da_marcha(&f, &doc, [4.0, 0.0, 0.0]);
        println!("  {nome:15} | {a:6} | {b:6} | {c:6}");
    }
}
