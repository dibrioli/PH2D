//! ⭐⭐⭐ **AUDITORIA DAS JUNTAS DA W145 na configuração do PRODUTO** (report do Enio, 09/09, três
//! fotos: *«groove não parece correto»*).
//!
//! # ⛔⛔ O que a bancada da W145 não podia ver
//!
//! Tudo o que autorizou aquela wave foi medido num **canto de 90° feito de dois SEMIESPAÇOS**. Ali o
//! conjunto `{a = b}` — o lugar onde as duas peças estão à mesma distância — é um **plano** que toca
//! a superfície numa **recta**, e uma decoração sobre ele lê-se como uma decoração sobre a costura.
//!
//! ⚠️ **Numa peça a sério isso é falso.** Um saliente sobre uma chapa tem duas faces quase
//! coincidentes por baixo dele, e ali `a ≈ b` sobre uma **REGIÃO INTEIRA** — não sobre uma curva. Uma
//! decoração que ACRESCENTA não se nota (pôr matéria dentro de matéria é invisível); uma que
//! **ESCAVA** come a região toda e **solta as duas peças**.
//!
//! *É a mesma classe da «almofada» do remalhador de quads: a régua media o resultado certo e nenhuma
//! contava as PEÇAS.*
//!
//! Corre com
//! `cargo test --release -p ph2d-field-eval --test audit_the_new_junctions -- --ignored --nocapture`.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;

const R: f32 = 0.06;

/// A peça da cena 32, com a junta pedida: um saliente sobre uma chapa.
fn peca(b: Blend) -> FieldDoc {
    let caixa = |half: [f32; 3], z: f32| {
        Node::new(
            Xform {
                translation: [0.0, 0.0, z],
                ..Xform::IDENTITY
            },
            NodeKind::Leaf(Primitive::Box {
                half,
                round: 0.0,
                chamfer: 0.0,
            }),
        )
    };
    FieldDoc::new(
        vec![
            caixa([0.26, 0.26, 0.11], -0.11),
            caixa([0.12, 0.12, 0.28], 0.18),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(b),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a peça")
}

/// A grelha da auditoria — a caixa que contém a peça inteira, com folga.
const NX: usize = 61;
const NZ: usize = 73;
fn px(i: usize) -> f64 {
    -0.34 + 0.68 * (i as f64 + 0.5) / NX as f64
}
fn pz(k: usize) -> f64 {
    -0.20 + 0.72 * (k as f64 + 0.5) / NZ as f64
}

/// ⭐⭐⭐ **QUANTAS PEÇAS SOLTAS** — a régua que faltava, e a única que vê o defeito da foto.
///
/// ⚠️ **Nem o volume, nem a área, nem o `‖∇f‖`, nem «mudou de lado» a veem:** uma peça partida em
/// duas tem o mesmo campo válido, o mesmo gradiente e uma variação de volume perfeitamente
/// plausível. O que muda é a **contagem de componentes ligadas**.
fn pecas_soltas(f: &Field) -> usize {
    let dentro = |i: usize, j: usize, k: usize| f.at(px(i), px(j), pz(k)) < 0.0;
    let mut visto = vec![false; NX * NX * NZ];
    let id = |i: usize, j: usize, k: usize| (k * NX + j) * NX + i;
    let mut n = 0;
    for k in 0..NZ {
        for j in 0..NX {
            for i in 0..NX {
                if visto[id(i, j, k)] || !dentro(i, j, k) {
                    continue;
                }
                n += 1;
                let mut pilha = vec![(i, j, k)];
                visto[id(i, j, k)] = true;
                while let Some((a, b, c)) = pilha.pop() {
                    let viz = [
                        (a.wrapping_sub(1), b, c),
                        (a + 1, b, c),
                        (a, b.wrapping_sub(1), c),
                        (a, b + 1, c),
                        (a, b, c.wrapping_sub(1)),
                        (a, b, c + 1),
                    ];
                    for (x, y, z) in viz {
                        if x < NX && y < NX && z < NZ && !visto[id(x, y, z)] && dentro(x, y, z) {
                            visto[id(x, y, z)] = true;
                            pilha.push((x, y, z));
                        }
                    }
                }
            }
        }
    }
    n
}

/// A secção `y = 0` em ASCII — o que a foto do Enio mostra.
fn seccao(f: &Field, nome: &str) {
    println!("  ── {nome} — secção x–z em y = 0 ──");
    for k in (0..NZ).rev().step_by(2) {
        let linha: String = (0..NX)
            .map(|i| {
                if f.at(px(i), 0.0, pz(k)) < 0.0 {
                    '#'
                } else {
                    '.'
                }
            })
            .collect();
        println!("  {linha}");
    }
}

/// **Que fracção da CHAPA a junta tocou** — a régua da localização.
///
/// ⚠️ Uma decoração de costura tem de ficar numa **banda** à volta dela. Se ela toca metade da chapa,
/// ela deixou de ser uma decoração de costura e passou a ser uma operação sobre a peça toda.
fn fraccao_da_chapa_tocada(f: &Field, viva: &Field) -> f64 {
    let (mut tocou, mut total) = (0usize, 0usize);
    // Uma fatia logo abaixo do topo da chapa, onde a decoração de facto aparece.
    for k in 0..NZ {
        let z = pz(k);
        if !(-0.14..=0.0).contains(&z) {
            continue;
        }
        for j in 0..NX {
            for i in 0..NX {
                let (x, y) = (px(i), px(j));
                if x.abs() > 0.26 || y.abs() > 0.26 {
                    continue;
                }
                total += 1;
                if (f.at(x, y, z) < 0.0) != (viva.at(x, y, z) < 0.0) {
                    tocou += 1;
                }
            }
        }
    }
    tocou as f64 / total as f64
}

/// ⭐⭐⭐ **NENHUMA JUNTA PARTE A PEÇA EM DUAS** — a régua que faltava, e a única que via o report.
///
/// ⚠️ **Ela é um GATE e não uma sonda**, porque o defeito que apanha é silencioso: o campo continua
/// válido, o gradiente continua no balde, o volume muda de forma plausível e a fileira de chips fica
/// igual. *O que muda é a contagem de componentes ligadas, e nada mais no repositório a media.*
#[test]
fn every_new_character_leaves_the_piece_in_one_piece() {
    let largura = R * Blend::SEAM_WIDTH_RATIO;
    for (nome, b) in [
        ("Soft", Blend::Soft { radius: R }),
        ("Bead", Blend::Bead { radius: R }),
        ("Groove", Blend::Groove { radius: R }),
        (
            "Ridge",
            Blend::Ridge {
                radius: R,
                width: largura,
            },
        ),
        (
            "Bevel",
            Blend::Bevel {
                radius: R,
                bias: 3.0,
            },
        ),
    ] {
        let n = pecas_soltas(&Field::new(&peca(b)));
        assert_eq!(
            n, 1,
            "o caracter {nome} deixou a peca em {n} pedacos — o saliente soltou-se da chapa, que e' \
             exactamente o report de 09/09, e NENHUMA outra regua deste repositorio o ve"
        );
    }
}

#[test]
#[ignore = "auditoria: as quatro juntas na peça do produto"]
fn audit_the_new_junctions_on_a_boss_over_a_plate() {
    let viva = Field::new(&peca(Blend::Sharp));
    let base = pecas_soltas(&viva);
    println!("\n⭐ AUDITORIA — saliente sobre chapa, R = {R}\n");
    println!("  {:>10} | {:>6} | {:>14}", "junta", "peças", "% da chapa");
    println!("  {}", "-".repeat(40));
    println!("  {:>10} | {base:6} | {:13.1} %", "viva", 0.0);
    let largura = R * Blend::SEAM_WIDTH_RATIO;
    let corpus = [
        ("Fillet", Blend::Exact { radius: R }),
        ("Soft", Blend::Soft { radius: R }),
        ("Bead", Blend::Bead { radius: R }),
        ("Groove", Blend::Groove { radius: R }),
        (
            "Ridge",
            Blend::Ridge {
                radius: R,
                width: largura,
            },
        ),
        (
            "Bevel",
            Blend::Bevel {
                radius: R,
                bias: 3.0,
            },
        ),
    ];
    for (nome, b) in corpus {
        let f = Field::new(&peca(b));
        let n = pecas_soltas(&f);
        let frac = fraccao_da_chapa_tocada(&f, &viva);
        println!("  {nome:>10} | {n:6} | {:13.1} %", frac * 100.0);
    }
    println!();
    for (nome, b) in corpus {
        seccao(&Field::new(&peca(b)), nome);
    }
}
