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
