//! Gates da EDIÇÃO manual do plano (`TweenPlan::repair`/`unpair_*`) — o escape CACAni.
//! Módulo-filho de `tween_match_tests` pelo cap de LOC (HR-18); reusa os helpers de lá.

use super::super::{FlipDrawing, TweenPlan};
use super::{drawing, stroke};

/// Duas nuvens de traços bem separadas: a correspondência automática é ÓBVIA (A0↔B0 no
/// grupo da esquerda, A1↔B1 no da direita), e é sobre ela que os re-pares atuam.
fn two_cluster_plan() -> (FlipDrawing, FlipDrawing, TweenPlan) {
    // A0/B0 perto da origem; A1/B1 perto de x=100. A distância domina o custo ⇒ par limpo.
    let a = drawing(vec![
        stroke(&[(0.0, 0.0), (2.0, 0.0)]),
        stroke(&[(100.0, 0.0), (102.0, 0.0)]),
    ]);
    let b = drawing(vec![
        stroke(&[(0.0, 1.0), (2.0, 1.0)]),
        stroke(&[(100.0, 1.0), (102.0, 1.0)]),
    ]);
    let plan = TweenPlan::build(&a, &b);
    // Sanidade: o automático parEIA como esperado (senão os gates abaixo testam a premissa
    // errada — a fixture TEM de conter o fenômeno).
    assert_eq!(plan.pair_of_a(0), Some(0), "premissa: A0↔B0 no automático");
    assert_eq!(plan.pair_of_a(1), Some(1), "premissa: A1↔B1 no automático");
    (a, b, plan)
}

/// 🔴 **`repair(a,b)` força o par E solta os DOIS vínculos antigos.** Sem soltar, o mesmo B
/// teria dois donos e o inbetween mostraria dois traços derretendo no mesmo lugar.
///
/// Auto: A0↔B0, A1↔B1. `repair(0,1)` ⇒ A0↔B1, e A1 **e** B0 viram ÓRFÃOS.
///
/// Mutação que sangra: não soltar o antigo B de A (`b_to_a[old_b]` fica `Some`) ⇒
/// `pair_of_b(0)` segue apontando A0 e o gate falha.
#[test]
fn repair_forces_the_pair_and_detaches_both_old_partners() {
    let (_, _, mut plan) = two_cluster_plan();
    assert!(plan.repair(0, 1), "repair válido devia retornar true");

    assert_eq!(plan.pair_of_a(0), Some(1), "A0 agora casa com B1");
    assert_eq!(plan.pair_of_b(1), Some(0), "B1 agora é reclamado por A0");
    // Os dois que perderam o vínculo:
    assert_eq!(plan.pair_of_a(1), None, "A1 perdeu B1 → órfão");
    assert_eq!(plan.pair_of_b(0), None, "B0 perdeu A0 → órfão");
    // Um par manual não carrega pontuação de matcher (o overlay o pinta em cor própria).
    assert_eq!(plan.cost_of_a(0), None, "par manual não tem custo");
    // A contagem de pares caiu de 2 para 1 (A0↔B1; A1 e B0 órfãos).
    assert_eq!(plan.pairs(), 1);
}

/// 🔴 **`unpair_a` orfana os DOIS lados.** Orfanar A significa que o B que ele carregava
/// também fica sem dono — senão B seguiria "vindo de A" enquanto A não vai a lugar nenhum.
///
/// Mutação que sangra: não limpar `b_to_a[b]` ⇒ `pair_of_b(0)` segue `Some(0)`.
#[test]
fn unpair_a_orphans_both_sides() {
    let (_, _, mut plan) = two_cluster_plan();
    assert!(plan.unpair_a(0));
    assert_eq!(plan.pair_of_a(0), None, "A0 é órfão");
    assert_eq!(plan.pair_of_b(0), None, "o B que A0 carregava também");
    assert_eq!(plan.cost_of_a(0), None);
    assert_eq!(plan.pairs(), 1, "sobra só A1↔B1");
}

/// 🔴 **`unpair_b` é o espelho** — orfana B e o A que o reclamava.
#[test]
fn unpair_b_orphans_both_sides() {
    let (_, _, mut plan) = two_cluster_plan();
    assert!(plan.unpair_b(1));
    assert_eq!(plan.pair_of_b(1), None, "B1 é órfão");
    assert_eq!(plan.pair_of_a(1), None, "o A que o reclamava também");
    assert_eq!(plan.pairs(), 1);
}

/// **Um índice fora da faixa é no-op** — o plano não cresce por um pedido inválido (a UI
/// pode mandar um índice velho depois de uma edição, e um `panic` ali seria pior).
#[test]
fn an_out_of_range_edit_is_a_no_op() {
    let (_, _, mut plan) = two_cluster_plan();
    assert_eq!(plan.a_len(), 2);
    assert_eq!(plan.b_len(), 2);
    assert!(!plan.repair(5, 0), "A fora da faixa");
    assert!(!plan.repair(0, 5), "B fora da faixa");
    assert!(!plan.unpair_a(9));
    assert!(!plan.unpair_b(9));
    // O plano seguiu intocado.
    assert_eq!(plan.pair_of_a(0), Some(0));
    assert_eq!(plan.pair_of_a(1), Some(1));
}
