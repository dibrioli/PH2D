//! **SONDA** — a parede de uma casca DEPOIS de um deformador mede o número escrito?
//!
//! Previsão por álgebra: um deformador que divide o campo por `L` muda a UNIDADE do campo, e todo
//! número **geométrico** aplicado a jusante (`Shell`, `Offset`, o raio de um filete) atravessa essa
//! conversão sem saber. `|f/L| − t/2` cruza zero onde `|f| = L·t/2` ⇒ **parede `L·t`**.
//!
//! ⭐⭐⭐ **CONFIRMOU, à digit** — e era defeito **pré-existente**: a inclinação carrega-o desde a W18.
//!
//! | pilha | parede pedida | entregue ANTES | DEPOIS |
//! |---|---:|---:|---:|
//! | `Shell` sozinho | `0,060` | `0,060` | `0,060` |
//! | `Taper 0,50` + `Shell` | `0,060` | `0,120` (`2,00×`) | **`0,060`** |
//! | `Taper 1,00` + `Shell` | `0,060` | `0,180` (`3,00×` = `1+2·declive`) | **`0,060`** |
//! | `Twist 1,00` + `Shell` | `0,060` | `0,337` (`5,62×` = `σ(k·R)`) | **`0,060`** |
//!
//! A cura é o divisor **acumular e aplicar-se UMA vez, no fim da pilha** (ver `stack::stacked`): o
//! `Shell`, o `Offset` e o `min`/`max` preservam Lipschitz, então o tecto não muda — e o número que o
//! artista escreveu volta a valer o que diz.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::Field;

fn caixa(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.5, 0.5, 0.5],
            round: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// A espessura da parede ao longo de `+X` no plano `y = z = 0`: a distância entre as duas travessias
/// de zero que a casca deixa perto da superfície original (`x = 0,5`).
fn parede(doc: &FieldDoc) -> f64 {
    let f = Field::new(doc);
    let mut travessias = Vec::new();
    let mut anterior = f.at(0.0, 0.0, 0.0);
    const N: usize = 40_000;
    for i in 1..=N {
        let x = i as f64 / N as f64 * 1.2;
        let v = f.at(x, 0.0, 0.0);
        if (anterior > 0.0) != (v > 0.0) {
            travessias.push(x);
        }
        anterior = v;
    }
    match travessias.as_slice() {
        [.., a, b] => b - a,
        _ => f64::NAN,
    }
}

/// ⭐ **O GATE**: a parede mede o número escrito, com ou sem deformador por baixo.
///
/// ⚠️ A barra é `2 %` porque a sonda anda a passos de `1,2/40 000` e a leitura é de duas travessias;
/// o defeito que ela apanha vale `2×` a `5,6×`. *Uma barra a meio caminho entre o certo e o defeito
/// mede a coisa; uma colada no certo mede a sonda.*
#[test]
fn the_wall_measures_the_number_that_was_written() {
    const T: f32 = 0.06;
    const BARRA: f64 = 0.02;
    let confere = |nome: &str, mods: Vec<Unary>| {
        let p = parede(&caixa(mods));
        let erro = (p / f64::from(T) - 1.0).abs();
        assert!(
            erro <= BARRA,
            "{nome}: a parede mede {p:.5} contra os {T} pedidos ({:.2}×) — o divisor do deformador \
             está a mudar a UNIDADE do campo, e todo número geométrico a jusante o atravessa",
            p / f64::from(T)
        );
    };
    confere("sem deformador", vec![Unary::Shell { thickness: T }]);
    for slope in [0.25f32, 0.5, 1.0] {
        confere(
            "Taper + Shell",
            vec![Unary::Taper { slope }, Unary::Shell { thickness: T }],
        );
    }
    for turns in [0.25f32, 0.5, 1.0] {
        confere(
            "Twist + Shell",
            vec![
                Unary::Twist {
                    turns,
                    lower: -9.0,
                    upper: 9.0,
                    falloff: 0.0,
                },
                Unary::Shell { thickness: T },
            ],
        );
    }
    // ⛔ **O CONTROLE**: a sonda tem de saber ler uma parede errada, senão ela mediria zero sempre.
    let grossa = parede(&caixa(vec![Unary::Shell { thickness: T * 3.0 }]));
    assert!(
        (grossa / f64::from(T) - 3.0).abs() <= BARRA * 3.0,
        "a sonda não lê uma parede 3× mais grossa ({grossa:.5}) — ela não mede a parede"
    );
}

#[test]
fn measure_the_wall_after_a_warp() {
    const T: f32 = 0.06;
    let limpa = parede(&caixa(vec![Unary::Shell { thickness: T }]));
    println!("\nsem deformador     : parede {limpa:.5} (pedida {T})");
    for slope in [0.25f32, 0.5, 1.0] {
        let p = parede(&caixa(vec![
            Unary::Taper { slope },
            Unary::Shell { thickness: T },
        ]));
        println!(
            "Taper {slope:.2} + Shell  : parede {p:.5}  ⇒ {:.2}× o número escrito",
            p / f64::from(T)
        );
    }
    for turns in [0.25f32, 0.5, 1.0] {
        let p = parede(&caixa(vec![
            Unary::Twist {
                turns,
                lower: -9.0,
                upper: 9.0,
                falloff: 0.0,
            },
            Unary::Shell { thickness: T },
        ]));
        println!(
            "Twist {turns:.2} + Shell  : parede {p:.5}  ⇒ {:.2}× o número escrito",
            p / f64::from(T)
        );
    }
}
