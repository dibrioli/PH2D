//! ⭐⭐⭐ **TODO TRIO de modificadores** — o ponto cego que a auditoria de 2026-08-30 nomeou e que
//! ninguém tinha medido.
//!
//! # ⛔⛔ Dois defeitos que só o TRIO revelava, e os dois eram de ORDEM
//!
//! O gate dos pares varre `10 × 10`; nenhum media três. E três é **três cliques do nascimento**.
//! Medido na primeira corrida: `4` de `1 000` trios rasgavam, dois deles catastroficamente.
//!
//! **1. `deformado` só olhava para trás** (`[Shell, Array, Twist] = 2 224`). A repetição alarga a
//! janela de fatias quando um deformador já passou — mas a caixa de recorte é o envelope do **FIM**
//! da pilha, e um deformador **posterior** alarga-a na mesma. Os MESMOS modificadores com a ordem
//! trocada mediam `0,38`. *É a lei que o divisor já tinha aprendido, por não ter sido aplicada aqui.*
//!
//! **2. O piso do `k` da inclinação era um ÉPSILON** (`[Bend, Twist, Taper] = 2,21`). Com dois
//! deformadores antes dele o envelope cresce, o `k` vai a **negativo** dentro do recorte e cai no
//! `TAPER_FLOOR = 0,01`, onde `σ = 100` contra um divisor de `4,1`. Hoje o piso é o `k` no ápice da
//! **peça**, e dentro do material ele não muda um bit.
//!
//! ⇒ **`0` de `1 000` depois das duas curas**, e o gate dos pares subiu de `20³` para `40³` na mesma
//! jornada.
//!
//! ⚠️ **Ele é caro de propósito** (`1 000` docs × `20³` amostras, ~1 min). É um gate de **fecho**,
//! não do laço interno — a mesma natureza do irmão dos pares.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::Field;

/// ⭐ Um exemplar **VIVO** de cada natureza — e não o de nascimento.
///
/// ⛔ O `born` do `Offset` e do `Taper` nasce **neutro**, e uma sonda que os instancie assim mede o
/// modificador **desligado**. ⚠️ O `match` é exaustivo: uma natureza nova não compila até alguém
/// dizer com que valor ela se mede.
fn vivo(k: UnaryKind) -> Unary {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    match k {
        UnaryKind::Shell => Unary::Shell { thickness: 0.06 },
        UnaryKind::Offset => Unary::Offset { distance: 0.05 },
        UnaryKind::Mirror => Unary::Mirror,
        UnaryKind::MirrorY => Unary::MirrorY,
        UnaryKind::MirrorZ => Unary::MirrorZ,
        UnaryKind::Array => Unary::Array {
            count: 3,
            spacing: 0.5,
            joint: ph2d_field::Joint::SHARP,
            axis: ARRAY_AXIS,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            joint: ph2d_field::Joint::SHARP,
            axis: RADIAL_AXIS,
        },
        UnaryKind::Taper => Unary::Taper {
            slope: 0.6,
            axis: TAPER_AXIS,
        },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: TWIST_AXIS,
        },
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: BEND_AXIS,
        },
    }
}

fn peca(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.35, 0.35, 0.30],
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// `‖∇f‖` **dentro da caixa de recorte** — fora dela ninguém pergunta nada, e medir lá acusa código
/// correcto.
fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, &reg) else {
        return 0.0;
    };
    let (lo, hi_box) = bola.aabb();
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32, e: usize| {
                    let t = f64::from(n) / f64::from(steps);
                    f64::from(lo[e]) + t * f64::from(hi_box[e] - lo[e])
                };
                let g = f.gradient_norm(p(i, 0), p(j, 1), p(k, 2), 1.0e-5);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// ⭐⭐⭐ **OS MIL TRIOS** — derivado do [`UnaryKind::ALL`], nas três posições.
///
/// ⚠️ Uma lista escrita à mão seria a terceira cópia da mesma pergunta e envelheceria no
/// modificador seguinte. Aqui um modificador novo entra em `3n²` trios **de graça**.
///
/// ⛔⛔ **Provas de mutação (2026-08-31), as duas na mesma corrida:**
/// - devolver o `deformado` da [`ph2d_field_eval::stack`] a *«um deformador já passou»* (o `|=`
///   dentro do laço) leva `[Shell, Array, Twist]` de `0,86` a **`1 973`** e
///   `[Radial, Bend, Radial]` de `0,31` a **`376`**;
/// - devolver o piso do `taper` ao `TAPER_FLOOR` fixo leva `[Bend, Twist, Taper]` de `0,04` a
///   **`2,21`** e `[Taper, Twist, Taper]` de `0,50` a **`1,88`**.
#[test]
fn every_trio_of_modifiers_keeps_the_field_marchable() {
    const SLACK: f64 = 1.02;
    let mut pior = (0.0f64, String::new());
    let mut maus: Vec<String> = Vec::new();
    let mut medidos = 0usize;
    for a in UnaryKind::ALL {
        for b in UnaryKind::ALL {
            for c in UnaryKind::ALL {
                medidos += 1;
                let nome = format!("[{a:?}, {b:?}, {c:?}]");
                let g = worst_gradient(&peca(vec![vivo(a), vivo(b), vivo(c)]), 20);
                if g > pior.0 {
                    pior = (g, nome.clone());
                }
                if g > SLACK {
                    maus.push(format!("{nome} {g:.4}"));
                }
            }
        }
    }
    assert_eq!(
        medidos,
        UnaryKind::ALL.len().pow(3),
        "a lista derivada de `UnaryKind::ALL` partiu-se"
    );
    assert!(
        maus.is_empty(),
        "{} trio(s) atravessam a superfície dentro da caixa de recorte, e cada um alcança-se em \
         TRÊS cliques: {}",
        maus.len(),
        maus.join(" · ")
    );
    // ⛔ **O CONTROLE**: se a sonda medisse zero em todo o lado, a barra acima passaria vazia.
    assert!(
        pior.0 > 0.2,
        "o trio mais castigado mede {:.4} ({}) — a sonda não está a ver os modificadores",
        pior.0,
        pior.1
    );
}
