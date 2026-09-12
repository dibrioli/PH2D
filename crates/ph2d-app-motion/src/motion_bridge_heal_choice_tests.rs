//! **O gate do `Deficit::MissingChoice`** — o sujeito de um nó escolhido por NOME, e o ⚠ que
//! aparece até alguém o escolher (ordem do dono, 2026-09-08).
//!
//! Irmão de `motion_bridge_heal_tests.rs` por TETO DE LOC (HR-18, 600 na shell), com o mesmo
//! corte que o `_toggle_tests` e o `_advisory_tests` já fizeram: um ficheiro por espécie de
//! diagnóstico. Um FILHO do módulo de testes, para o `use super::*` alcançar as fixturas dele
//! (`add`, `wire`) e o `inert_reaching_output` do avô.

use super::super::inert_reaching_output;
use super::*;

/// ⭐⭐⭐ **UM `motion.spline_wrap` SEM CAMINHO ESCOLHIDO BADGEIA** — a metade do pedido do dono
/// (2026-09-08) que o artista **vê**: *«até que o path esteja selecionado, um sinal de alerta fica
/// visível no nó»*.
///
/// ⚠️ **Nenhum diagnóstico que já existia o apanhava, e é essa a razão de o `Deficit::MissingChoice`
/// existir.** O `MissingSource` pergunta por uma entrada vazia (ele TEM uma), o `MissingInput`
/// pergunta por uma PORTA declarada (o sujeito dele não é uma porta), e o `InertProducer` pergunta
/// por uma coluna transiente (ele não escreve nenhuma). O sujeito deste nó é escolhido por NOME, no
/// canal de texto — e um `ParamSpec` é `f32`, que é a mesma assimetria que obrigou o
/// `ParamGateText` a existir ao lado do `ParamGate`.
///
/// ⚠️ **E o `reaches_output` continua a valer, de propósito.** Um nó ainda por ligar não é um
/// defeito, é uma montagem a meio — a lei que este ficheiro já protege para as forças. O badge
/// aparece quando o nó está numa cadeia VIVA a passar a folha adiante em silêncio, que é
/// exactamente onde o report nasceu.
///
/// FALSIFICADO por tirar o `register_required_text_params` do nó, ou por escolher um caminho.
#[test]
fn a_spline_wrap_with_no_path_chosen_badges_until_one_is() {
    let mut m = MotionState::new();
    m.doc = MotionDoc::new();
    let grid = add(&mut m, "motion.grid");
    let sw = add(&mut m, "motion.spline_wrap");
    let out = add(&mut m, "motion.output");
    wire(&mut m, grid, 0, sw, 0);

    // Ainda por ligar à saída: montagem a meio, e o badge cala-se.
    assert!(
        !inert_reaching_output(&m).contains(&sw.0),
        "controle: um no' que ainda nao alcanca a saida esta' a ser MONTADO, nao errado"
    );

    wire(&mut m, sw, 0, out, 0);
    assert!(
        inert_reaching_output(&m).contains(&sw.0),
        "numa cadeia viva e sem caminho escolhido, o no' passa a folha adiante em SILENCIO — e e' \
         isso que o ⚠ existe para dizer"
    );

    // E escolher um caminho cala-o — a metade sem a qual o badge seria permanente.
    m.doc.graph.set_text_param(sw, "path", "Curva");
    assert!(
        !inert_reaching_output(&m).contains(&sw.0),
        "com um caminho escolhido o no' tem sujeito, e o aviso sai"
    );
}
