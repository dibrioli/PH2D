//! ⭐⭐⭐ **CADA MODIFICADOR AGE NO EIXO QUE LHE MANDAM** — Enio, 2026-08-31: *«o efeito está atuando
//! num eixo diferente do desejado»*.
//!
//! # As três perguntas, e nenhuma delas responde as outras
//!
//! 1. **O eixo de omissão não muda nada** — byte a byte. É o que faz a feature não ser uma migração.
//! 2. **Escolher outro eixo MUDA a peça** — senão o controlo é decoração.
//! 3. ⭐⭐⭐ **A peça noutro eixo é a MESMA peça, rodada** — e é esta que apanha o erro caro. Uma
//!    permutação errada dá uma peça diferente e as duas primeiras ficam verdes na mesma.

use ph2d_field::{Axis, FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::Field;

/// Uma caixa **assimétrica nos três eixos** — de propósito: numa caixa cúbica trocar dois eixos não
/// muda nada, e o gate passaria com a lei a agir no sítio errado.
const HALF: [f32; 3] = [0.14, 0.26, 0.41];

fn peca(mods: Vec<Unary>) -> FieldDoc {
    peca_com(HALF, mods)
}

fn peca_com(half: [f32; 3], mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half,
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Os pontos da grelha, sempre os mesmos.
fn grelha() -> Vec<[f64; 3]> {
    let mut v = Vec::new();
    for i in 0..13 {
        for j in 0..13 {
            for k in 0..13 {
                let p = |n: i32| -0.9 + 0.15 * f64::from(n);
                v.push([p(i), p(j), p(k)]);
            }
        }
    }
    v
}

/// O mesmo modificador com o eixo trocado. ⚠️ O `match` é **exaustivo**: um modificador novo não
/// compila até alguém dizer se ele tem eixo.
fn com_eixo(m: Unary, a: Axis) -> Unary {
    match m {
        Unary::Array {
            count,
            spacing,
            joint,
            ..
        } => Unary::Array {
            count,
            spacing,
            joint,
            axis: a,
        },
        Unary::Taper { slope, .. } => Unary::Taper { slope, axis: a },
        Unary::Radial { count, joint, .. } => Unary::Radial {
            count,
            joint,
            axis: a,
        },
        Unary::Twist {
            turns,
            lower,
            upper,
            falloff,
            ..
        } => Unary::Twist {
            turns,
            lower,
            upper,
            falloff,
            axis: a,
        },
        Unary::Bend {
            turns,
            lower,
            upper,
            falloff,
            ..
        } => Unary::Bend {
            turns,
            lower,
            upper,
            falloff,
            axis: a,
        },
        outro => outro,
    }
}

/// Um exemplar **vivo** de cada modificador com eixo, e o eixo canónico dele.
///
/// ⛔ **Vivo, e não o de nascimento**: a inclinação nasce em `0,0` (a peça intacta), e uma sonda que
/// a instancie por `born` mede **o modificador desligado** — a armadilha que o irmão
/// `the_stack_composes_without_tearing` já nomeia.
fn exemplares() -> Vec<(&'static str, Unary, Axis)> {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    vec![
        (
            "Array",
            Unary::Array {
                count: 3,
                spacing: 0.5,
                joint: ph2d_field::Joint::SHARP,
                axis: ARRAY_AXIS,
            },
            ARRAY_AXIS,
        ),
        (
            "Taper",
            Unary::Taper {
                slope: 0.6,
                axis: TAPER_AXIS,
            },
            TAPER_AXIS,
        ),
        (
            "Radial",
            Unary::Radial {
                count: 5,
                joint: ph2d_field::Joint::SHARP,
                axis: RADIAL_AXIS,
            },
            RADIAL_AXIS,
        ),
        (
            "Twist",
            Unary::Twist {
                turns: 0.35,
                lower: -2.0,
                upper: 2.0,
                falloff: 0.1,
                axis: TWIST_AXIS,
            },
            TWIST_AXIS,
        ),
        (
            "Bend",
            Unary::Bend {
                turns: 0.25,
                lower: -2.0,
                upper: 2.0,
                falloff: 0.1,
                axis: BEND_AXIS,
            },
            BEND_AXIS,
        ),
    ]
}

/// Uma grelha de amostras do campo, para comparar dois documentos.
fn amostras(doc: &FieldDoc) -> Vec<f64> {
    let f = Field::new(doc);
    grelha()
        .into_iter()
        .map(|q| f.at(q[0], q[1], q[2]))
        .collect()
}

/// ⭐ **1. O eixo de omissão não muda um bit** — a metade que faz disto uma extensão e não uma
/// migração.
///
/// ⛔⛔ **Prova de mutação (2026-08-31):** tirar o curto-circuito `if s == 0` da
/// `ph2d_field_eval::stack::conjugado` (deixando os dois `remap_xyz` sempre) mantém os valores mas
/// **muda a árvore** — e é por isso que a régua aqui é o campo E o gate irmão do tamanho da fita
/// existe. Trocar o eixo canónico de um modificador reprova este gate em cheio.
#[test]
fn the_default_axis_leaves_every_modifier_bit_identical() {
    for (nome, m, canon) in exemplares() {
        let a = amostras(&peca(vec![m]));
        let b = amostras(&peca(vec![com_eixo(m, canon)]));
        assert_eq!(
            a, b,
            "{nome}: escrever o eixo canónico à mão mudou o campo — o eixo de omissão tem de ser \
             uma identidade ao bit"
        );
    }
}

/// ⭐⭐ **2. Escolher outro eixo MUDA a peça** — senão o controlo é decoração.
///
/// ⚠️ **A caixa é assimétrica nos três eixos de propósito**: numa cúbica a permutação não teria o
/// que mover, e um controlo morto passaria.
#[test]
fn choosing_another_axis_actually_moves_the_piece() {
    for (nome, m, canon) in exemplares() {
        let base = amostras(&peca(vec![m]));
        let mut mexeu = 0;
        for outro in Axis::ALL {
            if outro == canon {
                continue;
            }
            let d = amostras(&peca(vec![com_eixo(m, outro)]));
            if d != base {
                mexeu += 1;
            }
        }
        assert_eq!(
            mexeu, 2,
            "{nome}: só {mexeu} dos dois outros eixos mudaram a peça — um eixo que não muda um \
             pixel é um controlo morto"
        );
    }
}

/// ⭐⭐⭐ **3. A LEI NOUTRO EIXO É A LEI CANÓNICA, CONJUGADA** — o gate que apanha o erro caro.
///
/// As duas perguntas acima ficam verdes com uma permutação **errada**: ela muda a peça e é uma
/// identidade no eixo canónico. O que ela não é é uma **rotação**, e é isso que se mede aqui.
///
/// # ⚠️ A afirmação exacta, e a 1.ª versão deste gate estava ERRADA
///
/// ⛔ *«a peça no eixo A é a peça canónica rodada»* é **falso**, e reprovou sobre código correcto:
/// mudar o eixo do deformador **não roda a caixa**. O que roda é a lei, e a caixa fica onde estava.
///
/// ⇒ a comparação certa põe a rotação nos **dois** sítios: a peça no eixo `A` tem de dar, no ponto
/// `p`, o mesmo que a peça **com as meias-extensões permutadas** e o modificador canónico dá em
/// `P(p)`. *Uma régua que roda só um dos lados mede a rotação, não a conjugação.*
///
/// ⛔⛔ **Prova de mutação (2026-08-31):** trocar a permutação da `stack::conjugado` pela inversa
/// (`leva(3 - s)` na entrada e `leva(s)` na saída) deixa **1 e 2 verdes** e mata esta com folga.
#[test]
fn the_law_on_another_axis_is_the_canonical_law_conjugated() {
    for (nome, m, canon) in exemplares() {
        for outro in Axis::ALL {
            let s = outro.shift_to(canon);
            let a = Field::new(&peca(vec![com_eixo(m, outro)]));
            let b = Field::new(&peca_com(Axis::to_canonical(HALF, s), vec![m]));
            let mut pior = 0.0f64;
            for p in grelha() {
                #[allow(clippy::cast_possible_truncation)]
                let q = Axis::to_canonical([p[0] as f32, p[1] as f32, p[2] as f32], s);
                let d = (a.at(p[0], p[1], p[2])
                    - b.at(f64::from(q[0]), f64::from(q[1]), f64::from(q[2])))
                .abs();
                pior = pior.max(d);
            }
            // ⚠️ A permutação é exacta; o que não é ao bit é a **ordem das somas** que o `Tree`
            // monta depois dela.
            assert!(
                pior < 1.0e-6,
                "{nome} em {outro:?}: difere da lei canónica conjugada em {pior} — ela foi \
                 reescrita, não conjugada"
            );
        }
    }
}

/// ⛔ **O CENSO**: os modificadores que oferecem a linha do eixo são exactamente os que a lei
/// conjuga.
///
/// ⚠️ *Um modificador novo com eixo que ninguém conjugue oferece um botão que não faz nada* — e o
/// contrário (conjugar um que não oferece) é uma lei inalcançável. O `match` da
/// [`ph2d_field::Unary::dims`] e o da `stack` são dois sítios, e este liga-os.
#[test]
fn every_modifier_that_offers_an_axis_row_is_one_the_law_conjugates() {
    let com_linha: Vec<UnaryKind> = UnaryKind::ALL
        .into_iter()
        .filter(|k| {
            Unary::born(*k, 1.0)
                .dims()
                .iter()
                .any(|d| d.key == "field.mod.axis")
        })
        .collect();
    let conjugados: Vec<UnaryKind> = exemplares().into_iter().map(|(_, m, _)| m.kind()).collect();
    // ⚠️ **Conjuntos, e não listas ordenadas**: a ordem do `UnaryKind::ALL` e a da fixtura deste
    // ficheiro são duas escolhas independentes, e prender uma à outra faria o gate reprovar no dia
    // em que alguém reordenasse a paleta. *O gate é a POPULAÇÃO.*
    assert_eq!(
        com_linha.len(),
        conjugados.len(),
        "{com_linha:?} mostram a linha do eixo e {conjugados:?} são conjugados pela lei"
    );
    for k in &com_linha {
        assert!(
            conjugados.contains(k),
            "{k:?} MOSTRA a linha do eixo e a lei não o conjuga — o botão não faz nada"
        );
    }
    for k in &conjugados {
        assert!(
            com_linha.contains(k),
            "a lei conjuga {k:?} e o painel não oferece o eixo — a lei é inalcançável"
        );
    }
}
