//! ⭐⭐⭐ **O CHANFRO DA ESTRELA SAI PARA O MIOLO** (report do Enio, 08/09, três fotos) — a medição
//! do defeito, antes de qualquer cura.
//!
//! > *«o Chamfer múltiplo acaba criando um padrão complexo que afeta até o miolo da estrela. Não
//! > fica apenas nas bordas externas e cria um padrão estranho. Ao usar fillet sobre Chamfer é ainda
//! > pior.»*
//!
//! ⭐ **A régua é a ALTURA DA FACE DE CIMA, ponto a ponto.** Numa chapa, a tampa é plana: acima de
//! todo ponto interior a superfície está exactamente em `z = half_height`. O chanfro **de aro**
//! encolhe o contorno da tampa, e a tampa que sobra **continua plana**. ⇒ *qualquer variação de
//! altura sobre a face de cima é uma faceta que não devia estar lá*, e ela mede-se sem olhar.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

const OUTER: f32 = 0.45;
const INNER: f32 = 0.18;
const H: f32 = 0.25;

fn estrela(round: f32, chamfer: f32) -> Field {
    estrela_com(round, chamfer, 0.0)
}

/// A mesma, com o chanfro das PONTAS (W143).
fn estrela_com(round: f32, chamfer: f32, corner_chamfer: f32) -> Field {
    Field::new(
        &FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Star {
                    points: 5,
                    outer: OUTER,
                    inner: INNER,
                    half_height: H,
                    round,
                    chamfer,
                    corner_chamfer,
                }),
            )],
            NodeId(0),
        )
        .expect("a estrela"),
    )
}

/// A altura da superfície de cima sobre `(x, y)` — `None` se ali não há peça nenhuma.
fn topo(f: &Field, x: f64, y: f64) -> Option<f64> {
    let alto = f64::from(H) * 2.0;
    if f.at(x, y, alto) < 0.0 {
        return None;
    }
    if f.at(x, y, 0.0) >= 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0_f64, alto);
    for _ in 0..50 {
        let m = f64::midpoint(lo, hi);
        if f.at(x, y, m) < 0.0 {
            lo = m;
        } else {
            hi = m;
        }
    }
    Some(lo)
}

/// ⭐⭐⭐ **A FACE DE CIMA É PLANA?** — a tabela que o report pede.
#[test]
#[ignore = "sonda: o chanfro da estrela sai para o miolo?"]
fn probe_whether_the_top_face_stays_flat() {
    println!(
        "  round | chamfer | amostras com peça | altura MAX | altura MIN | espalhamento | % fora do plano"
    );
    for (round, chamfer) in [
        (0.00_f32, 0.00_f32),
        (0.00, 0.02),
        (0.00, 0.04),
        (0.00, 0.06),
        (0.03, 0.00),
        (0.02, 0.02),
        (0.03, 0.03),
    ] {
        let f = estrela(round, chamfer);
        let (mut n, mut maxi, mut mini) = (0_u32, f64::NEG_INFINITY, f64::INFINITY);
        let mut alturas = Vec::new();
        const M: usize = 90;
        for i in 0..M {
            for j in 0..M {
                let c =
                    |t: usize| f64::from(OUTER) * 1.1 * (2.0 * (t as f64) / (M - 1) as f64 - 1.0);
                let (x, y) = (c(i), c(j));
                // ⚠️ Só o MIOLO: fora do raio do vale a tampa acaba mesmo, e o que lá se mede é a
                // ponta, que tem aresta por construção.
                if x.hypot(y) > f64::from(INNER) * 0.92 {
                    continue;
                }
                if let Some(z) = topo(&f, x, y) {
                    n += 1;
                    maxi = maxi.max(z);
                    mini = mini.min(z);
                    alturas.push(z);
                }
            }
        }
        let fora = alturas
            .iter()
            .filter(|z| (**z - maxi).abs() > 1.0e-4)
            .count();
        println!(
            "  {round:5.2} | {chamfer:7.2} | {n:17} | {maxi:10.5} | {mini:10.5} | {:12.5} | {:14.1}",
            maxi - mini,
            100.0 * fora as f64 / n.max(1) as f64
        );
    }
}

/// ⭐⭐⭐ **O MAPA da tampa** — onde ela afunda, e não só quanto.
///
/// ⚠️ *Uma percentagem diz que há defeito e esconde a FORMA dele* — um anel, cinco raios e uma
/// mancha central leem-se todos como «30 %».
#[test]
#[ignore = "sonda: desenhar onde a tampa afunda"]
fn probe_where_the_top_face_dips() {
    for (round, chamfer) in [(0.00_f32, 0.06_f32), (0.03, 0.03)] {
        let f = estrela(round, chamfer);
        println!(
            "\n  round {round} · chamfer {chamfer}  (`.` = plano · dígito = décimos de mm de afundamento · ' ' = sem peça)"
        );
        const M: usize = 49;
        for j in (0..M).rev() {
            let mut linha = String::new();
            for i in 0..M {
                let c =
                    |t: usize| f64::from(OUTER) * 1.05 * (2.0 * (t as f64) / (M - 1) as f64 - 1.0);
                let (x, y) = (c(i), c(j));
                linha.push(match topo(&f, x, y) {
                    None => ' ',
                    Some(z) => {
                        let d = (f64::from(H) - z) * 1000.0;
                        if d < 0.05 {
                            '.'
                        } else {
                            char::from_digit(((d / 3.0) as u32).min(9), 10).unwrap_or('9')
                        }
                    }
                });
            }
            println!("  {linha}");
        }
    }
}

/// ⭐⭐⭐ **O CAMPO DAS PAREDES, visto de dentro** — a pergunta que o mapa da tampa não responde.
///
/// ⚠️ **`f(x, y, 0)` É o campo das paredes** numa chapa sem acabamento: ali a laje vale `−h` (muito
/// negativa) e a intersecção dura devolve as paredes. ⇒ mede-se o insumo do aro **sem** o aro no
/// meio.
///
/// O aro (filete ou chanfro) mistura onde este campo está **perto de zero**. Numa peça honesta isso
/// só acontece junto da fronteira; num campo com costuras interiores acontece **dentro**.
#[test]
#[ignore = "sonda: onde o campo das paredes mente"]
fn probe_where_the_wall_field_is_shallow() {
    let f = estrela(0.0, 0.0);
    println!("\n  |paredes| no MIOLO (r < inner·0,92) — `.` = fundo (< −0,06) · dígito = décimos");
    const M: usize = 49;
    let (mut pior, mut onde) = (f64::NEG_INFINITY, [0.0; 2]);
    for j in (0..M).rev() {
        let mut linha = String::new();
        for i in 0..M {
            let c = |t: usize| f64::from(INNER) * (2.0 * (t as f64) / (M - 1) as f64 - 1.0);
            let (x, y) = (c(i), c(j));
            if x.hypot(y) > f64::from(INNER) * 0.92 {
                linha.push(' ');
                continue;
            }
            let w = f.at(x, y, 0.0);
            if w > pior {
                pior = w;
                onde = [x, y];
            }
            linha.push(if w < -0.06 {
                '.'
            } else {
                char::from_digit(((-w * 100.0) as u32).min(9), 10).unwrap_or('0')
            });
        }
        println!("  {linha}");
    }
    println!(
        "\n  ⇒ o campo das paredes chega a {pior:.5} dentro do miolo, em ({:.3}, {:.3})",
        onde[0], onde[1]
    );
}

/// **SONDA (W142)** — o campo das PAREDES tem uma crista na bissectriz da ponta?
///
/// ⭐ O plano do chanfro do aro é `(tampa + paredes + c)·√½`: ele **soma** o campo das paredes, logo
/// herda toda crista que esse campo tenha. Uma estrela **alta** (a laje longe) dá o campo das
/// paredes sozinho — e a pergunta é se a direcção do gradiente vira ao atravessar a bissectriz da
/// ponta, a várias PROFUNDIDADES.
#[test]
#[ignore = "sonda: imprime a tabela"]
fn probe_whether_the_wall_field_has_a_ridge_at_the_tip() {
    let limite = ph2d_field::round_limit(&Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    })
    .expect("tem filete");
    println!("  limite do filete = {limite:.4}");
    for r in [0.0_f32, limite * 0.5, limite] {
        // ⚠️ Estrela ALTA: a `z = 0` a laje não participa, e o que se lê é só a parede.
        let f = Field::new(
            &FieldDoc::new(
                vec![Node::new(
                    Xform::IDENTITY,
                    NodeKind::Leaf(Primitive::Star {
                        points: 5,
                        outer: OUTER,
                        inner: INNER,
                        half_height: 5.0,
                        round: r,
                        chamfer: 0.0,
                        corner_chamfer: 0.0,
                    }),
                )],
                NodeId(0),
            )
            .expect("a estrela alta"),
        );
        println!("\n  round = {r:.4}");
        println!("   profundidade |  giro do gradiente atravessando a bissectriz");
        // A ponta está em φ = 0, raio OUTER. Andamos para dentro pela bissectriz.
        for prof in [0.02_f64, 0.05, 0.08, 0.12, 0.16, 0.20] {
            let x = f64::from(OUTER) - prof;
            let h = 1.0e-4;
            let grad = |x: f64, y: f64| {
                let gx = f.at(x + h, y, 0.0) - f.at(x - h, y, 0.0);
                let gy = f.at(x, y + h, 0.0) - f.at(x, y - h, 0.0);
                let l = gx.hypot(gy).max(1.0e-15);
                (gx / l, gy / l)
            };
            let (ax, ay) = grad(x, -3.0 * h);
            let (bx, by) = grad(x, 3.0 * h);
            let giro = (ax * bx + ay * by).clamp(-1.0, 1.0).acos().to_degrees();
            println!(
                "        {prof:.3}    |  {giro:>6.1}°   (f = {:+.5})",
                f.at(x, 0.0, 0.0)
            );
        }
    }
}

/// **SONDA (W142)** — o ALCANCE da ponta na tampa: a cura não pode estar a cortar mais fundo.
#[test]
#[ignore = "sonda: imprime o alcance"]
fn probe_the_tip_reach_on_the_cap() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let limite = ph2d_field::round_limit(&base).expect("tem filete");
    let c = limite * 0.5;
    for (rot, r, ch) in [
        ("vivo", 0.0, 0.0),
        ("so' chanfro", 0.0, c),
        ("par", c * 0.5, c),
        ("so' filete", c * 0.5, 0.0),
    ] {
        let f = estrela(r, ch);
        // O alcance ao longo da bissectriz da ponta (φ = 0), na PAREDE e na TAMPA.
        let alcance = |z: f64| {
            let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
            for _ in 0..60 {
                let m = f64::midpoint(lo, hi);
                if f.at(m, 0.0, z) <= 0.0 {
                    lo = m
                } else {
                    hi = m
                }
            }
            lo
        };
        println!(
            "  {rot:<12} (r={r:.5} c={ch:.5}): parede {:.5} | tampa {:.5}",
            alcance(0.0),
            alcance(f64::from(H) - 1.0e-4)
        );
    }
}

/// **SONDA (W142)** — o PREÇO da cura, em passos de marcha (régua imune ao relógio).
#[test]
#[ignore = "sonda: imprime os passos"]
fn probe_what_the_second_profile_costs_the_march() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let limite = ph2d_field::round_limit(&base).expect("tem filete");
    let c = limite * 0.5;
    for (rot, r, ch) in [
        ("vivo", 0.0, 0.0),
        ("so' filete", c, 0.0),
        ("so' chanfro", 0.0, c),
        ("par (trabalho)", c * 0.5, c),
        ("par (saturacao)", c, c),
    ] {
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Star {
                    points: 5,
                    outer: OUTER,
                    inner: INNER,
                    half_height: H,
                    round: r,
                    chamfer: ch,
                    corner_chamfer: 0.0,
                }),
            )],
            NodeId(0),
        )
        .expect("a estrela");
        let f = Field::new(&doc);
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        // Um raio RASANTE, de longe, apontado a um ponto da tampa perto de uma ponta.
        let de = [2.0_f64, 0.0, 1.2];
        let alvo = [0.30_f64, 0.0, 0.25];
        let v = [alvo[0] - de[0], alvo[1] - de[1], alvo[2] - de[2]];
        let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let dir = [v[0] / m, v[1] / m, v[2] / m];
        let (mut p, mut andou, mut passos) = (de, 0.0_f64, 0_u32);
        while passos < 20_000 && andou < 8.0 {
            let d = f.at(p[0], p[1], p[2]);
            if d < 1.0e-4 {
                break;
            }
            let a = d * passo;
            for k in 0..3 {
                p[k] += dir[k] * a;
            }
            andou += a;
            passos += 1;
        }
        // ⭐ **O tamanho da ÁRVORE**, que é o custo por amostra — a outra metade do preço, e também
        // imune ao relógio.
        let mut ctx = fidget::context::Context::new();
        let raiz = ctx.import(&ph2d_field_eval::compile(&doc));
        let nos = ctx.len();
        let _ = raiz;
        println!(
            "  {rot:<16} (r={r:.5} c={ch:.5}): {passos} passos, {nos} nos, passo seguro {passo:.4}"
        );
    }
}

/// **SONDA (W143)** — a LARGURA DA FACETA do chanfro em cada família de aresta.
///
/// ⭐ Uma faceta de chanfro é **plana**: na secção `z = 0`, junto de uma ponta em `φ = 0`, a
/// fronteira tem `x` constante ao longo dela. Um filete é um arco, e `x` varia. ⇒ a largura da
/// faceta mede-se perguntando **até que altura `x` não se move**.
#[test]
#[ignore = "sonda: imprime a largura da faceta"]
fn probe_the_facet_width_of_each_edge_family() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let limite = ph2d_field::round_limit(&base).expect("tem filete");
    println!("  limite dos sliders = {limite:.5}");
    println!("\n  chanfro | filete | faceta PONTA | recuo FLANCO | recuo PONTA | razao");
    for cf in [0.25_f32, 0.5, 0.75] {
        for rf in [0.0_f32, 0.25, 0.5, 0.75] {
            let (c, r) = (limite * cf, limite * rf);
            let f = estrela(r, c);
            // ── a PONTA: secção z = 0, fronteira x(y) junto de φ = 0 ──
            let bordo = |y: f64| -> f64 {
                let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                for _ in 0..60 {
                    let m = f64::midpoint(lo, hi);
                    if f.at(m, y, 0.0) <= 0.0 {
                        lo = m;
                    } else {
                        hi = m;
                    }
                }
                lo
            };
            let x0 = bordo(0.0);
            let mut faceta = 0.0_f64;
            for i in 1..=400 {
                let y = f64::from(OUTER) * f64::from(i) / 400.0;
                if (bordo(y) - x0).abs() > 1.0e-4 {
                    break;
                }
                faceta = 2.0 * y;
            }
            // ── o ARO: corte vertical no meio de um flanco recto, na direcção da normal ──
            let meio = {
                let b = std::f64::consts::PI / 5.0;
                let (tip, val) = (
                    [f64::from(OUTER), 0.0],
                    [f64::from(INNER) * b.cos(), f64::from(INNER) * b.sin()],
                );
                [(tip[0] + val[0]) * 0.5, (tip[1] + val[1]) * 0.5]
            };
            let nlen = (meio[0] * meio[0] + meio[1] * meio[1]).sqrt();
            let dir = [meio[0] / nlen, meio[1] / nlen];
            // A normal EXTERIOR do flanco (ponta -> vale, rodada) e o cosseno contra a radial.
            let cos_radial_flanco = {
                let b = std::f64::consts::PI / 5.0;
                let (tip, val) = (
                    [f64::from(OUTER), 0.0],
                    [f64::from(INNER) * b.cos(), f64::from(INNER) * b.sin()],
                );
                let (dx, dy) = (val[0] - tip[0], val[1] - tip[1]);
                let l = dx.hypot(dy);
                let (nx, ny) = (dy / l, -dx / l);
                (dir[0] * nx + dir[1] * ny).abs()
            };
            // ⭐⭐⭐ **O RECUO DA FACE DE CIMA — a grandeza que o artista VÊ**, e a única que se
            // compara entre as duas famílias sem supor nada.
            //
            // ⛔ Três réguas de «largura da faceta» no aro foram construídas e deitadas fora antes
            // desta: uma supunha o declive `45°` (o `s` é radial, não normal ao flanco: declive
            // medido `2,05`), outra procurava o trecho recto **a partir do topo**, que é onde o
            // filete mistura, e a terceira lia a PAREDE como faceta. *A faceta é difícil de medir;
            // o recuo não é.*
            let topo = f64::from(H) - 1.0e-4;
            let recuo_no_flanco = {
                let vivo = |z: f64| -> f64 {
                    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                    for _ in 0..60 {
                        let m = f64::midpoint(lo, hi);
                        if f.at(dir[0] * m, dir[1] * m, z) <= 0.0 {
                            lo = m;
                        } else {
                            hi = m;
                        }
                    }
                    lo
                };
                // ⚠️ **Recuo PERPENDICULAR ao flanco**, e não ao longo do raio: o `s` mede na
                // direcção radial, e o factor `cos θ` entre as duas é o que a 1.ª régua ignorou.
                (vivo(0.0) - vivo(topo)) * cos_radial_flanco
            };
            let recuo_na_ponta = {
                let ponta = |z: f64| -> f64 {
                    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                    for _ in 0..60 {
                        let m = f64::midpoint(lo, hi);
                        if f.at(m, 0.0, z) <= 0.0 {
                            lo = m;
                        } else {
                            hi = m;
                        }
                    }
                    lo
                };
                ponta(0.0) - ponta(topo)
            };
            println!(
                "   {cf:.2}   |  {rf:.2}  |   {faceta:.5}   |  {recuo_no_flanco:.5}  |                   {recuo_na_ponta:.5}  | {:.2}x",
                recuo_na_ponta / recuo_no_flanco.max(1.0e-9)
            );
        }
    }
}

/// **SONDA (W143)** — até onde o chanfro da PONTA pode ir antes de a estrela deixar de ser estrela.
///
/// ⚠️ **A régua é o raio na direcção da ponta contra o raio no vale**, e não a largura de uma
/// faceta: uma faceta grande é o que se pede, e o que não se pode é a ponta desaparecer.
#[test]
#[ignore = "sonda: imprime o tecto"]
fn probe_how_far_the_corner_chamfer_can_go() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let limite = ph2d_field::round_limit(&base).expect("tem filete");
    let beta = std::f64::consts::PI / 5.0;
    let lado = (f64::from(OUTER).powi(2) + f64::from(INNER).powi(2)
        - 2.0 * f64::from(OUTER) * f64::from(INNER) * beta.cos())
    .sqrt();
    println!(
        "  aresta do flanco = {lado:.5}   chanfro = 0,5 x limite = {:.5}",
        limite * 0.5
    );
    println!("\n  mult | recuo na ponta | raio no VALE | ainda e' estrela?");
    let c = limite * 0.5;
    let f = estrela(0.0, c);
    let raio = |ang: f64| -> f64 {
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            if f.at(m * ang.cos(), m * ang.sin(), 0.0) <= 0.0 {
                lo = m;
            } else {
                hi = m;
            }
        }
        lo
    };
    let (r_ponta, r_vale) = (raio(0.0), raio(beta));
    println!(
        "   {:>4} | {r_ponta:.5} | {r_vale:.5} | {}",
        std::env::var("PH2D_TIPCHAM").unwrap_or_else(|_| "1".into()),
        if r_ponta > r_vale * 1.02 {
            "SIM"
        } else {
            "NAO"
        }
    );
}

/// **SONDA (W143)** — o controlo NOVO: a faceta da ponta ao longo da faixa PRÓPRIA dela.
#[test]
#[ignore = "sonda: imprime a faixa do controlo novo"]
fn probe_the_corner_chamfer_control() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let lim_faces = ph2d_field::round_limit(&base).expect("tem filete");
    let c = lim_faces * 0.5;
    let lim_pontas = ph2d_field::star_corner_chamfer_limit(5, OUTER, INNER);
    println!("  chanfro das FACES: tecto {lim_faces:.5}");
    println!(
        "  chanfro das PONTAS: tecto {lim_pontas:.5}  ({:.2}x mais curso)",
        lim_pontas / lim_faces
    );
    println!("\n  tip (fraccao do tecto dele) | valor | faceta na ponta | raio da ponta");
    for fr in [0.0_f32, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99] {
        let t = lim_pontas * fr;
        let f = estrela_com(c * 0.5, c, t);
        let bordo = |y: f64| -> f64 {
            let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
            for _ in 0..60 {
                let m = f64::midpoint(lo, hi);
                if f.at(m, y, 0.0) <= 0.0 {
                    lo = m
                } else {
                    hi = m
                }
            }
            lo
        };
        let x0 = bordo(0.0);
        let mut faceta = 0.0_f64;
        for i in 1..=800 {
            let y = f64::from(OUTER) * f64::from(i) / 800.0;
            if (bordo(y) - x0).abs() > 1.0e-4 {
                break;
            }
            faceta = 2.0 * y;
        }
        println!("        {fr:.2}                | {t:.5} |     {faceta:.5}     |   {x0:.5}");
    }
}

/// **SONDA (W143)** — o pior giro da normal com as DUAS arestas tratadas.
#[test]
#[ignore = "sonda: o pior giro por configuracao"]
fn probe_the_worst_turn_with_both_chamfers() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let lim_f = ph2d_field::round_limit(&base).expect("filete");
    let c = lim_f * 0.5;
    let lim_t = ph2d_field::star_corner_chamfer_limit(5, OUTER, INNER);
    println!("  tecto faces {lim_f:.5} | tecto pontas {lim_t:.5}");
    for (rot, r, ch, tip) in [
        ("so' chanfro faces", 0.0, c, 0.0),
        ("faces + pontas 0,25", 0.0, c, lim_t * 0.25),
        ("faces + pontas 0,50", 0.0, c, lim_t * 0.5),
        ("par: faces+pontas+filete", c * 0.5, c, lim_t * 0.5),
        ("par com pontas a 0,25", c * 0.5, c, lim_t * 0.25),
    ] {
        // A malha de amostragem: pontos na superfície e o giro da normal à volta de cada um.
        let f = estrela_com(r, ch, tip);
        let h = 1.0e-4_f64;
        let n = |p: [f64; 3]| {
            let g = [
                f.at(p[0] + h, p[1], p[2]) - f.at(p[0] - h, p[1], p[2]),
                f.at(p[0], p[1] + h, p[2]) - f.at(p[0], p[1] - h, p[2]),
                f.at(p[0], p[1], p[2] + h) - f.at(p[0], p[1], p[2] - h),
            ];
            let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt().max(1e-15);
            [g[0] / l, g[1] / l, g[2] / l]
        };
        // Varre um anel de direcções à volta da ponta, na superfície, e mede o maior salto.
        let mut pior = 0.0_f64;
        for i in 0..400 {
            let ang = -0.6 + 1.2 * f64::from(i) / 400.0;
            let z = 0.0;
            let raio = |a: f64| -> f64 {
                let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                for _ in 0..50 {
                    let m = f64::midpoint(lo, hi);
                    if f.at(m * a.cos(), m * a.sin(), z) <= 0.0 {
                        lo = m
                    } else {
                        hi = m
                    }
                }
                lo
            };
            let (a0, a1) = (ang, ang + 0.003);
            let (r0, r1) = (raio(a0), raio(a1));
            let p0 = [r0 * a0.cos(), r0 * a0.sin(), z];
            let p1 = [r1 * a1.cos(), r1 * a1.sin(), z];
            let (u, v) = (n(p0), n(p1));
            let d = (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0);
            pior = pior.max(d.acos().to_degrees());
        }
        println!("  {rot:<26} tip={tip:.5}  pior giro na secção z=0: {pior:6.1}°");
    }
}

/// **SONDA (W143)** — o pior giro da SUPERFÍCIE (o instrumento do gate) por configuração.
#[test]
#[ignore = "sonda"]
fn probe_the_surface_worst_turn() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let lim_f = ph2d_field::round_limit(&base).expect("filete");
    let c = lim_f * 0.5;
    let lim_t = ph2d_field::star_corner_chamfer_limit(5, OUTER, INNER);
    for (rot, r, ch, tip) in [
        ("faces=c, pontas=0", 0.0, c, 0.0),
        ("faces=c, pontas=c  (o de sempre)", 0.0, c, c),
        ("faces=c, pontas=0,25 do tecto", 0.0, c, lim_t * 0.25),
        ("faces=c, pontas=0,50 do tecto", 0.0, c, lim_t * 0.5),
        ("+ filete c/2, pontas=c", c * 0.5, c, c),
        ("+ filete c/2, pontas=0,5 tecto", c * 0.5, c, lim_t * 0.5),
    ] {
        let doc = FieldDoc::new(
            vec![Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Star {
                    points: 5,
                    outer: OUTER,
                    inner: INNER,
                    half_height: H,
                    round: r,
                    chamfer: ch,
                    corner_chamfer: tip,
                }),
            )],
            NodeId(0),
        )
        .expect("a estrela");
        let f = Field::new(&doc);
        let h = 1.0e-4_f64;
        let normal = |p: [f64; 3]| {
            let g = [
                f.at(p[0] + h, p[1], p[2]) - f.at(p[0] - h, p[1], p[2]),
                f.at(p[0], p[1] + h, p[2]) - f.at(p[0], p[1] - h, p[2]),
                f.at(p[0], p[1], p[2] + h) - f.at(p[0], p[1], p[2] - h),
            ];
            let l = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt().max(1e-15);
            [g[0] / l, g[1] / l, g[2] / l]
        };
        // Projecta uma nuvem de sementes na superfície e mede o giro num anel de 0,004.
        let mut pior = 0.0_f64;
        for i in 0..6000 {
            let z = 1.0 - 2.0 * (f64::from(i) + 0.5) / 6000.0;
            let sr = (1.0 - z * z).max(0.0).sqrt();
            let a = 2.399_963_229_728_653 * f64::from(i);
            let mut q = [sr * a.cos() * 0.7, sr * a.sin() * 0.7, z * 0.7];
            for _ in 0..24 {
                let d = f.at(q[0], q[1], q[2]);
                let n = normal(q);
                for k in 0..3 {
                    q[k] -= d * n[k];
                }
            }
            if f.at(q[0], q[1], q[2]).abs() > 1.0e-3 {
                continue;
            }
            let n0 = normal(q);
            let t = if n0[2].abs() < 0.9 {
                [-n0[1], n0[0], 0.0]
            } else {
                [1.0, 0.0, 0.0]
            };
            let tl = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            let t = [t[0] / tl, t[1] / tl, t[2] / tl];
            let t2 = [
                n0[1] * t[2] - n0[2] * t[1],
                n0[2] * t[0] - n0[0] * t[2],
                n0[0] * t[1] - n0[1] * t[0],
            ];
            for k in 0..6 {
                let ang = std::f64::consts::TAU * f64::from(k) / 6.0;
                let mut w = [
                    q[0] + 0.004 * (t[0] * ang.cos() + t2[0] * ang.sin()),
                    q[1] + 0.004 * (t[1] * ang.cos() + t2[1] * ang.sin()),
                    q[2] + 0.004 * (t[2] * ang.cos() + t2[2] * ang.sin()),
                ];
                for _ in 0..24 {
                    let d = f.at(w[0], w[1], w[2]);
                    let n = normal(w);
                    for j in 0..3 {
                        w[j] -= d * n[j];
                    }
                }
                let v = normal(w);
                let d = (n0[0] * v[0] + n0[1] * v[1] + n0[2] * v[2]).clamp(-1.0, 1.0);
                pior = pior.max(d.acos().to_degrees());
            }
        }
        println!("  {rot:<34} pior giro {pior:6.1}°");
    }
}

/// **SONDA (W144)** — onde a estrela de facto parte quando as DUAS quinas recuam com o mesmo número.
#[test]
#[ignore = "sonda: varre a fronteira"]
fn probe_where_the_corner_chamfer_actually_breaks() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let tecto = ph2d_field::star_corner_chamfer_limit(5, OUTER, INNER);
    let beta = std::f64::consts::PI / 5.0;
    println!("  tecto = {tecto:.5}");
    println!("\n  fraccao |  valor  | raio ponta | raio vale | ponta > vale?");
    for fr in [0.5_f32, 0.7, 0.8, 0.85, 0.9, 0.95, 0.98, 0.999] {
        let t = tecto * fr;
        let f = estrela_com(0.0, 0.0, t);
        let raio = |a: f64| -> f64 {
            let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
            for _ in 0..60 {
                let m = f64::midpoint(lo, hi);
                if f.at(m * a.cos(), m * a.sin(), 0.0) <= 0.0 {
                    lo = m
                } else {
                    hi = m
                }
            }
            lo
        };
        let (rp, rv) = (raio(0.0), raio(beta));
        println!(
            "   {fr:.3}  | {t:.5} |  {rp:.5}   |  {rv:.5}  | {}",
            if rp > rv * 1.001 { "SIM" } else { "NAO" }
        );
    }
    let _ = base;
}

/// **SONDA (W144)** — a fronteira MEDIDA em várias estrelas, contra a conta analítica.
#[test]
#[ignore = "sonda: a tabela que escolhe o tecto"]
fn probe_the_measured_corner_ceiling_over_many_stars() {
    println!("\n  n | outer | inner | tecto analitico | fronteira MEDIDA | razao");
    let mut corpus: Vec<(u32, f32, f32)> = Vec::new();
    for n in 3_u32..=16 {
        for k in [0.2_f32, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
            corpus.push((n, 0.45, 0.45 * k));
        }
    }
    let mut pior = (1.0_f32, 0_u32, 0.0_f32);
    for (n, outer, inner) in corpus {
        let tecto = ph2d_field::star_corner_chamfer_limit(n, outer, inner);
        let beta = std::f64::consts::PI / f64::from(n);
        let ainda_estrela = |t: f32| -> bool {
            let doc = FieldDoc::new(
                vec![Node::new(
                    Xform::IDENTITY,
                    NodeKind::Leaf(Primitive::Star {
                        points: n,
                        outer,
                        inner,
                        half_height: H,
                        round: 0.0,
                        chamfer: 0.0,
                        corner_chamfer: t,
                    }),
                )],
                NodeId(0),
            );
            let Ok(doc) = doc else { return false };
            let f = Field::new(&doc);
            let raio = |a: f64| -> f64 {
                let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                for _ in 0..50 {
                    let m = f64::midpoint(lo, hi);
                    if f.at(m * a.cos(), m * a.sin(), 0.0) <= 0.0 {
                        lo = m
                    } else {
                        hi = m
                    }
                }
                lo
            };
            // ⭐⭐⭐ **A RÉGUA CERTA: o BRAÇO tem de dominar o VALE.**
            //
            // ⛔ A 1.ª comparava `raio(0)` com `raio(β)` e supunha que o mínimo está no vale. Com
            // uma FACETA na ponta o raio **cresce** ao sair da bissectriz (numa recta `x = const`,
            // `√(x²+y²)` sobe com `|y|`), e com o vale chanfrado o mínimo muda de sítio. *Uma régua
            // que supõe onde está o mínimo mede a suposição.*
            let arco = |de: f64, ate: f64| -> f64 {
                (0..=16)
                    .map(|i| raio(de + (ate - de) * f64::from(i) / 16.0))
                    .fold(0.0_f64, f64::max)
            };
            arco(0.0, beta * 0.5) > raio(beta) * 1.001
        };
        let (mut lo, mut hi) = (0.0_f32, tecto);
        for _ in 0..30 {
            let m = 0.5 * (lo + hi);
            if ainda_estrela(m) { lo = m } else { hi = m }
        }
        let razao = lo / tecto;
        if razao < pior.0 {
            pior = (razao, n, inner);
        }
        if razao < 0.95 {
            println!(
                "  {n:>2} | {outer:.3} | {inner:.3} |    {tecto:.5}      |     {lo:.5}      | {razao:.3}"
            );
        }
    }
    println!(
        "\n  ⇒ PIOR do corpus: {:.4} (n = {}, inner = {:.3})",
        pior.0, pior.1, pior.2
    );
}

/// **SONDA (W144)** — o contorno inteiro de uma estrela perto do tecto, para ver ONDE ele parte.
#[test]
#[ignore = "sonda"]
fn probe_the_outline_near_the_ceiling() {
    for (n, inner) in [(5_u32, 0.18_f32), (6, 0.20)] {
        let tecto = ph2d_field::star_corner_chamfer_limit(n, OUTER, inner);
        for fr in [0.5_f32, 0.85] {
            let t = tecto * fr;
            let doc = FieldDoc::new(
                vec![Node::new(
                    Xform::IDENTITY,
                    NodeKind::Leaf(Primitive::Star {
                        points: n,
                        outer: OUTER,
                        inner,
                        half_height: H,
                        round: 0.0,
                        chamfer: 0.0,
                        corner_chamfer: t,
                    }),
                )],
                NodeId(0),
            )
            .expect("valida");
            let f = Field::new(&doc);
            let beta = std::f64::consts::PI / f64::from(n);
            print!("  n={n} fr={fr:.2} t={t:.5}  raios de 0 a beta:");
            for i in 0..=8 {
                let a = beta * f64::from(i) / 8.0;
                let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
                for _ in 0..50 {
                    let m = f64::midpoint(lo, hi);
                    if f.at(m * a.cos(), m * a.sin(), 0.0) <= 0.0 {
                        lo = m
                    } else {
                        hi = m
                    }
                }
                print!(" {lo:.4}");
            }
            println!();
        }
    }
}

/// **SONDA (W144)** — o que o VALE entrega: recuo do chanfro e avanço do filete, contra a conta.
#[test]
#[ignore = "sonda: o preco da rota n-aria"]
fn probe_what_the_valley_delivers() {
    let (outer, inner, n) = (0.45_f64, 0.18_f64, 5_u32);
    let beta = std::f64::consts::PI / f64::from(n);
    let lado = (outer * outer + inner * inner - 2.0 * outer * inner * beta.cos()).sqrt();
    let alfa = (inner * beta.sin() / lado).asin();
    let av = alfa + beta;
    println!(
        "  meia-abertura do vale = {:.2}°  sin = {:.5}",
        av.to_degrees(),
        av.sin()
    );
    let vale = |round: f32, cham: f32| -> f64 {
        let f = estrela_com(round, 0.0, cham);
        let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            if f.at(m * beta.cos(), m * beta.sin(), 0.0) <= 0.0 {
                lo = m
            } else {
                hi = m
            }
        }
        lo
    };
    let base = vale(0.0, 0.0);
    println!("  vale sem tratamento: {base:.5} (autorado {inner:.5})");
    for c in [0.02_f32, 0.04, 0.06] {
        let medido = vale(0.0, c) - base;
        let conta = f64::from(c) * av.cos();
        println!(
            "  chanfro {c:.3}: o vale avanca {medido:.5}, a conta honesta pede {conta:.5} ({:.3}x)",
            medido / conta
        );
    }
    for r in [0.02_f32, 0.04, 0.06] {
        let medido = vale(r, 0.0) - base;
        let conta = f64::from(r) * (1.0 / av.sin() - 1.0);
        println!(
            "  filete  {r:.3}: o vale avanca {medido:.5}, o arco exacto pede {conta:.5} ({:.3}x)",
            medido / conta
        );
    }
}

/// **SONDA (W144)** — o chanfro do VALE era load-bearing para o miolo?
#[test]
#[ignore = "sonda: A/B do vale contra o miolo"]
fn probe_whether_the_valley_chamfer_protects_the_middle() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let tecto = ph2d_field::round_limit(&base).expect("filete");
    // A fracção do MIOLO da tampa fora do plano — a régua da W141.
    let miolo = |ch: f32, cc: f32| -> f64 {
        let f = estrela_com(0.0, ch, cc);
        let (mut fora, mut total) = (0usize, 0usize);
        let n = 90;
        for i in 0..n {
            for j in 0..n {
                let x = -0.30 + 0.60 * f64::from(i) / f64::from(n - 1);
                let y = -0.30 + 0.60 * f64::from(j) / f64::from(n - 1);
                // só o MIOLO: dentro do disco dos vales
                if x.hypot(y) > f64::from(INNER) * 0.85 {
                    continue;
                }
                let Some(z) = topo(&f, x, y) else { continue };
                total += 1;
                if (z - f64::from(H)).abs() > 1.0e-4 {
                    fora += 1;
                }
            }
        }
        if total == 0 {
            f64::NAN
        } else {
            100.0 * fora as f64 / total as f64
        }
    };
    println!("\n  chanfro | vale a ZERO | vale = chanfro");
    for fr in [0.2_f32, 0.3, 0.4, 0.5, 0.667] {
        let ch = tecto * fr;
        println!(
            "   {fr:.3}  |   {:>6.2} %   |   {:>6.2} %",
            miolo(ch, 0.0),
            miolo(ch, ch)
        );
    }
}

/// **SONDA (W144)** — a PROFUNDIDADE do campo ao longo da costura entre pipas, contra o alcance do
/// chanfro do aro.
#[test]
#[ignore = "sonda: a profundidade da costura"]
fn probe_how_deep_the_seam_is() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let tecto = ph2d_field::round_limit(&base).expect("filete");
    let beta = std::f64::consts::PI / 5.0;
    // ⚠️ Estrela ALTA: a `z = 0` a laje não participa e o que se lê é o campo das PAREDES.
    let paredes = |cc: f32| -> Field {
        Field::new(
            &FieldDoc::new(
                vec![Node::new(
                    Xform::IDENTITY,
                    NodeKind::Leaf(Primitive::Star {
                        points: 5,
                        outer: OUTER,
                        inner: INNER,
                        half_height: 5.0,
                        round: 0.0,
                        chamfer: 0.0,
                        corner_chamfer: cc,
                    }),
                )],
                NodeId(0),
            )
            .expect("alta"),
        )
    };
    for cc in [0.0_f32, tecto * 0.667] {
        let f = paredes(cc);
        print!("  vale={cc:.5}  campo na COSTURA (r de 0,02 a inner):");
        for i in 1..=9 {
            let r = f64::from(INNER) * f64::from(i) / 9.0;
            print!(" {:.4}", f.at(r * beta.cos(), r * beta.sin(), 0.0));
        }
        println!();
    }
    println!(
        "\n  alcance do chanfro do aro a 0,667 do tecto: -{:.4}",
        tecto * 0.667
    );
}

/// **SONDA (W144)** — o que sobra do «miolo fora do plano» quando se tira o que o chanfro do VALE
/// legitimamente remove.
///
/// ⭐ Num canto CÔNCAVO, um chanfro de recuo `c` remove um disco de raio `c` à volta do vértice.
/// ⇒ os pontos a menos de `c` de um vale **têm** de sair do plano; contá-los é contar a geometria.
#[test]
#[ignore = "sonda"]
fn probe_the_middle_minus_what_the_valley_legitimately_eats() {
    let base = Primitive::Star {
        points: 5,
        outer: OUTER,
        inner: INNER,
        half_height: H,
        round: 0.0,
        chamfer: 0.0,
        corner_chamfer: 0.0,
    };
    let tecto = ph2d_field::round_limit(&base).expect("filete");
    let vales: Vec<[f64; 2]> = (0..5)
        .map(|k| {
            let a = std::f64::consts::PI / 5.0 + std::f64::consts::TAU * f64::from(k) / 5.0;
            [f64::from(INNER) * a.cos(), f64::from(INNER) * a.sin()]
        })
        .collect();
    println!("\n  chanfro | miolo cru | miolo SEM a coroa do vale");
    for fr in [0.4_f32, 0.5, 0.667] {
        let ch = tecto * fr;
        let f = estrela_com(0.0, ch, 0.0);
        let (mut cru, mut tot_cru, mut lim, mut tot_lim) = (0usize, 0usize, 0usize, 0usize);
        let n = 120;
        for i in 0..n {
            for j in 0..n {
                let x = -0.30 + 0.60 * f64::from(i) / f64::from(n - 1);
                let y = -0.30 + 0.60 * f64::from(j) / f64::from(n - 1);
                if x.hypot(y) > f64::from(INNER) * 0.85 {
                    continue;
                }
                let Some(z) = topo(&f, x, y) else { continue };
                let fora = (z - f64::from(H)).abs() > 1.0e-4;
                tot_cru += 1;
                if fora {
                    cru += 1;
                }
                // ⚠️ Fora da coroa que o chanfro do vale come, por construção.
                let perto = vales
                    .iter()
                    .any(|v| (x - v[0]).hypot(y - v[1]) < f64::from(ch));
                if !perto {
                    tot_lim += 1;
                    if fora {
                        lim += 1;
                    }
                }
            }
        }
        println!(
            "   {fr:.3}  |  {:>5.2} %  |        {:>5.2} %",
            100.0 * cru as f64 / tot_cru.max(1) as f64,
            100.0 * lim as f64 / tot_lim.max(1) as f64
        );
    }
}
