//! Gates da correção de pares (`flip_tween_correct`): o gesto de re-par (puro) e o pick em
//! espaço de tela. O oráculo é o comportamento OBSERVÁVEL — que par o plano passa a ter, que
//! traço o clique pegou — nunca a implementação reescrita ao lado.

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipDrawing, FlipStroke, TweenPlan};
use ph2d_vector::Affine;

/// Duas nuvens de traços bem separadas → o automático pareia A0↔B0, A1↔B1. É o palco dos
/// re-pares.
fn two_cluster() -> (FlipDrawing, FlipDrawing, TweenPlan) {
    let line = |x: f32, y: f32| {
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(x, y));
        s.push_default(Vec2::new(x + 2.0, y));
        s
    };
    let mut a = FlipDrawing::new();
    a.strokes.push(line(0.0, 0.0));
    a.strokes.push(line(100.0, 0.0));
    let mut b = FlipDrawing::new();
    b.strokes.push(line(0.0, 1.0));
    b.strokes.push(line(100.0, 1.0));
    let plan = TweenPlan::build(&a, &b);
    assert_eq!(plan.pair_of_a(0), Some(0), "premissa: A0↔B0");
    assert_eq!(plan.pair_of_a(1), Some(1), "premissa: A1↔B1");
    (a, b, plan)
}

const A0: PairSel = PairSel {
    side: Side::A,
    idx: 0,
};
const A1: PairSel = PairSel {
    side: Side::A,
    idx: 1,
};
const B0: PairSel = PairSel {
    side: Side::B,
    idx: 0,
};
const B1: PairSel = PairSel {
    side: Side::B,
    idx: 1,
};

/// 🔴 **O 1º clique num traço MARCA** (nada → marcado); o clique no VAZIO desmarca.
#[test]
fn a_click_selects_and_an_empty_click_deselects() {
    let (_, _, mut plan) = two_cluster();
    assert_eq!(apply_click(&mut plan, None, Some(A0)), Some(A0), "marca A0");
    assert_eq!(
        apply_click(&mut plan, Some(A0), None),
        None,
        "clique no vazio desmarca"
    );
    // Nenhum vazio/marca mexeu no plano.
    assert_eq!(plan.pair_of_a(0), Some(0));
}

/// 🔴 **Marcar A e clicar o outro LADO FORÇA o par.** É o gesto inteiro: A0 marcado + clique
/// em B1 ⇒ A0↔B1, e a marca some.
///
/// Mutação que sangra: não chamar `plan.repair` (só devolver `None`) ⇒ o par não muda.
#[test]
fn selecting_a_then_the_other_side_forces_the_pair() {
    let (_, _, mut plan) = two_cluster();
    assert_eq!(apply_click(&mut plan, Some(A0), Some(B1)), None, "desmarca");
    assert_eq!(plan.pair_of_a(0), Some(1), "A0 agora casa com B1");
    assert_eq!(plan.pair_of_b(1), Some(0));
    assert_eq!(plan.pair_of_a(1), None, "A1 ficou órfão (perdeu B1)");
}

/// 🔴 **Marcar A e clicar o MESMO traço ORFANA.** O *"click me de novo para soltar"* — A0
/// marcado + clique em A0 ⇒ A0 vira órfão.
///
/// Mutação que sangra: tratar `p == h` como "move a marca" (devolver `Some(h)`) ⇒ o par de
/// A0 sobrevive e nunca há como cortá-lo.
#[test]
fn clicking_the_same_stroke_orphans_it() {
    let (_, _, mut plan) = two_cluster();
    assert_eq!(apply_click(&mut plan, Some(A0), Some(A0)), None);
    assert_eq!(plan.pair_of_a(0), None, "A0 virou órfão");
    assert_eq!(plan.pair_of_b(0), None, "e o B que ele carregava também");
}

/// **Clicar outro traço do MESMO lado move a marca** (sem tocar o plano) — trocar de ideia
/// sobre qual A re-parear não deve orfanar nada.
#[test]
fn clicking_another_stroke_of_the_same_side_moves_the_mark() {
    let (_, _, mut plan) = two_cluster();
    assert_eq!(apply_click(&mut plan, Some(A0), Some(A1)), Some(A1));
    assert_eq!(plan.pair_of_a(0), Some(0), "o plano não mudou");
    assert_eq!(plan.pair_of_a(1), Some(1));
}

/// 🔴 **O pick pega o traço mais próximo ao alcance, e nada além dele.** Com afim identidade,
/// arte = tela. Um clique a 3 px do traço A0 (a 10 px de folga) o pega; a 50 px, ninguém.
///
/// Mutação que sangra: alcance zero (nunca pega) ⇒ o 1º `assert` falha; ou ignorar o
/// threshold (pega o mais próximo mesmo longe) ⇒ o `None` esperado vira `Some`.
#[test]
fn the_pick_takes_the_nearest_stroke_within_reach() {
    // A0 = linha (0,0)-(100,0); A1 = (0,100)-(100,100). B0 = (0,200)-(100,200).
    let seg = |y: f32| {
        let mut s = FlipStroke::new();
        s.push_default(Vec2::new(0.0, y));
        s.push_default(Vec2::new(100.0, y));
        s
    };
    let mut a = FlipDrawing::new();
    a.strokes.push(seg(0.0));
    a.strokes.push(seg(100.0));
    let mut b = FlipDrawing::new();
    b.strokes.push(seg(200.0));

    let id = Affine::IDENTITY;
    // A 3 px de A0 (dentro da folga de 10) → pega A0.
    assert_eq!(
        nearest_stroke(&a, id, &b, id, 50.0, 3.0),
        Some(A0),
        "3 px de A0 devia pegar A0"
    );
    // A 3 px de A1 → pega A1 (o MAIS próximo, não o primeiro da lista).
    assert_eq!(nearest_stroke(&a, id, &b, id, 50.0, 97.0), Some(A1));
    // Perto de B0 (lado B) → Side::B.
    assert_eq!(nearest_stroke(&a, id, &b, id, 50.0, 203.0), Some(B0));
    // No vazio (50 px de qualquer traço) → nada.
    assert_eq!(nearest_stroke(&a, id, &b, id, 50.0, 50.0), None);
}
