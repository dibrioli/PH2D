//! ⭐⭐⭐ **A COMPOSIÇÃO JÁ EXPRIME ESTAS DUAS?** (W138) — a pergunta que o `CLAUDE.md` §5.0 manda
//! fazer **antes** de construir um item de lista aberta.
//!
//! O plano ([doc 09](../../../docs/3DModeling/09_plano_das_dez_que_faltam.md) lote 12) diz que a
//! *Death Star* e o *Vesica Segment* são «composição com a distância certa no encontro», e que a
//! nossa subtracção **não** dá a distância exacta na cratera. ⚠️ **Isso é uma afirmação sobre um
//! número que ninguém mediu** — e o módulo nunca precisou da distância exacta: precisa de um
//! MINORANTE 1-Lipschitz (doc 06 §124).
//!
//! ⇒ esta sonda mede as três coisas que decidem: o gradiente, quanto o campo subestima, e o preço
//! em passos de marcha — com uma **esfera** ao lado, que é o campo exacto de referência.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;

/// A distância EXACTA do ponto à fronteira da peça, por varredura densa das duas calotes.
///
/// ⚠️ Ela não partilha uma linha com o produto: sai da DEFINIÇÃO do conjunto — cada ponto de uma
/// esfera é fronteira se o teste da outra o mantiver.
fn dist_a_fronteira(p: [f64; 3], a: ([f64; 3], f64), b: ([f64; 3], f64), corta: bool) -> f64 {
    let n = 900_usize;
    let mut melhor = f64::INFINITY;
    for (centro, raio, outro, r_outro) in [(a.0, a.1, b.0, b.1), (b.0, b.1, a.0, a.1)] {
        for i in 0..=n {
            for j in 0..(n * 2) {
                let th = std::f64::consts::PI * i as f64 / n as f64;
                let ph = std::f64::consts::TAU * j as f64 / (n * 2) as f64;
                let q = [
                    centro[0] + raio * th.sin() * ph.cos(),
                    centro[1] + raio * th.sin() * ph.sin(),
                    centro[2] + raio * th.cos(),
                ];
                let d_outro = ((q[0] - outro[0]).powi(2)
                    + (q[1] - outro[1]).powi(2)
                    + (q[2] - outro[2]).powi(2))
                .sqrt()
                    - r_outro;
                // Na SUBTRACÇÃO a fronteira de `a` sobrevive fora de `b`, e a de `b` dentro de `a`;
                // na INTERSECÇÃO as duas sobrevivem dentro da outra.
                let vive = if corta {
                    if centro == a.0 {
                        d_outro >= 0.0
                    } else {
                        d_outro <= 0.0
                    }
                } else {
                    d_outro <= 0.0
                };
                if vive {
                    let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                        .sqrt();
                    melhor = melhor.min(d);
                }
            }
        }
    }
    melhor
}

fn composta(op: Op, a: ([f64; 3], f64), b: ([f64; 3], f64)) -> FieldDoc {
    let mut mover = |c: [f64; 3]| {
        let mut x = Xform::IDENTITY;
        x.translation = [c[0] as f32, c[1] as f32, c[2] as f32];
        x
    };
    // ⚠️ **Os filhos vêm ANTES do pai na arena** — o documento recusa uma referência para a frente
    // (`ForwardReference`), que é o que torna a árvore acíclica por construção em vez de por teste.
    FieldDoc::new(
        vec![
            Node::new(
                mover(a.0),
                NodeKind::Leaf(Primitive::Sphere { radius: a.1 as f32 }),
            ),
            Node::new(
                mover(b.0),
                NodeKind::Leaf(Primitive::Sphere { radius: b.1 as f32 }),
            ),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op,
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a peça composta tem de ser aceite")
}

fn pior_gradiente(f: &Field, alcance: f64, n: usize) -> f64 {
    let eps = 1.0e-4;
    let mut pior = 0.0_f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let c = |t: usize| -alcance + 2.0 * alcance * t as f64 / (n - 1) as f64;
                let (x, y, z) = (c(i), c(j), c(k));
                let g = [
                    (f.at(x + eps, y, z) - f.at(x - eps, y, z)) / (2.0 * eps),
                    (f.at(x, y + eps, z) - f.at(x, y - eps, z)) / (2.0 * eps),
                    (f.at(x, y, z + eps) - f.at(x, y, z - eps)) / (2.0 * eps),
                ];
                pior = pior.max(g[0].mul_add(g[0], g[1].mul_add(g[1], g[2] * g[2])).sqrt());
            }
        }
    }
    pior
}

fn passos(f: &Field, doc: &FieldDoc, de: [f64; 3]) -> u32 {
    let passo = f64::from(ph2d_field_eval::safe_march_step(doc));
    let alvo = [0.0, 0.30, 0.0];
    let v = [alvo[0] - de[0], alvo[1] - de[1], alvo[2] - de[2]];
    let m = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let dir = [v[0] / m, v[1] / m, v[2] / m];
    let n = (de[0] * de[0] + de[1] * de[1] + de[2] * de[2]).sqrt();
    let (mut p, mut andou, mut c) = (de, 0.0, 0_u32);
    while c < 20_000 && andou < 4.0 * n {
        let d = f.at(p[0], p[1], p[2]);
        if d < 1.0e-4 {
            break;
        }
        for t in 0..3 {
            p[t] += dir[t] * d * passo;
        }
        andou += d * passo;
        c += 1;
    }
    c
}

#[test]
#[ignore = "sonda: a composição já exprime a Death Star e o Vesica Segment?"]
fn probe_the_composition_already_expresses_them() {
    let esfera = FieldDoc::new(
        vec![Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: 0.45 }),
        )],
        NodeId(0),
    )
    .expect("a esfera");
    let casos: Vec<(
        &str,
        FieldDoc,
        Option<(([f64; 3], f64), ([f64; 3], f64), bool)>,
    )> = vec![
        ("esfera (controlo)", esfera, None),
        (
            "death star (A menos B)",
            composta(
                Op::Difference(Blend::Sharp),
                ([0.0, 0.0, 0.0], 0.45),
                ([0.50, 0.0, 0.0], 0.35),
            ),
            Some((([0.0, 0.0, 0.0], 0.45), ([0.50, 0.0, 0.0], 0.35), true)),
        ),
        (
            "vesica (A com B)",
            composta(
                Op::Intersection(Blend::Sharp),
                ([-0.18, 0.0, 0.0], 0.40),
                ([0.18, 0.0, 0.0], 0.40),
            ),
            Some((([-0.18, 0.0, 0.0], 0.40), ([0.18, 0.0, 0.0], 0.40), false)),
        ),
    ];
    println!("  peca                     | grad  | pior campo/verdade | passos 1,0 | 4,0");
    for (nome, doc, oraculo) in casos {
        let f = Field::new(&doc);
        let g = pior_gradiente(&f, 1.0, 22);
        let razao = oraculo.map_or(1.0, |(a, b, corta)| {
            let mut pior = 1.0_f64;
            for i in 0..24 {
                for j in 0..24 {
                    for k in 0..12 {
                        let c = |t: usize, n: usize| -1.0 + 2.0 * t as f64 / (n - 1) as f64;
                        let p = [c(i, 24), c(j, 24), c(k, 12)];
                        let lido = f.at(p[0], p[1], p[2]);
                        if lido <= 0.02 {
                            continue;
                        }
                        let verdade = dist_a_fronteira(p, a, b, corta);
                        pior = pior.min(lido / verdade);
                    }
                }
            }
            pior
        });
        println!(
            "  {nome:24} | {g:5.3} | {razao:18.4} | {:10} | {}",
            passos(&f, &doc, [1.0, 0.0, 0.0]),
            passos(&f, &doc, [4.0, 0.0, 0.0])
        );
    }
}
