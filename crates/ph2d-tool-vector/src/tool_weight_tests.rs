//! ⭐⭐⭐ **OS GATES DO PINCEL DE PESO NO DESPACHO** — irmão de `tool_tests.rs` pelo tecto de LOC,
//! cortado por RESPONSABILIDADE: aquele mede o que a ferramenta de vector faz em geral, este mede
//! **a ordem do dono de 2026-09-19** — *«no lugar de valores negativos em Brush Strength prefiro
//! botões Add e Subtract»*.

use super::*;
use crate::ids;
use crate::params::WeightDirection;

/// ⭐⭐⭐ **OS DOIS BOTÕES CHEGAM À FERRAMENTA, e SÓ mexem na direcção.**
///
/// ⚠️ **A 2.ª metade é a lei declarada** (e o que a separa dos três chips do verbo): escolher um
/// lado **não** arma o verbo `Weight` nem troca o modo — a secção destes dois só é pintada com ele
/// já na mão. *Armá-lo aqui arrancaria o artista do que ele estava a fazer.*
#[test]
fn os_dois_lados_chegam_a_ferramenta_e_nao_armam_o_verbo() {
    for (id, esperado) in [
        (ids::VECTOR_BONE_WEIGHT_SUB, WeightDirection::Subtract),
        (ids::VECTOR_BONE_WEIGHT_ADD, WeightDirection::Add),
    ] {
        let mut t = VectorTool::new();
        // Um modo QUALQUER que não seja o do osso: é o que torna a 2.ª metade observável.
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_MODE_PENCIL));
        let modo_antes = t.draw_config().mode;
        Tool::handle_panel_event(&mut t, PanelEvent::Click(id));
        assert_eq!(
            t.draw_config().weight_direction,
            esperado,
            "o botao {esperado:?} nao chegou a' ferramenta — ele acende sob o rato e nao faz nada"
        );
        assert_eq!(
            t.draw_config().mode,
            modo_antes,
            "escolher o lado do pincel ARRANCOU o artista do modo em que ele estava"
        );
    }
}

/// ⭐⭐⭐ **A FORÇA DO PINCEL É UMA MAGNITUDE — e um negativo nunca a deixa INERTE.**
///
/// ⛔⛔ **Até 2026-09-19 ela ia a `clamp(-1.0, 1.0)` e o SINAL era a direcção.** A premissa morreu
/// por ordem do dono. ⚠️ **E a escolha entre `abs()` e `clamp(0.0, …)` não é de estilo:** cortar a
/// zero devolve um pincel que **não faz nada e não diz porquê** — a família de reports que esta
/// casa já pagou três vezes. Com o absoluto, a tela re-semeia o campo com a magnitude que a lei
/// usa, e *o ecrã corrige-se à vista*.
#[test]
fn a_forca_do_pincel_e_uma_magnitude_e_um_negativo_nao_a_zera() {
    let mut t = VectorTool::new();
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_BONE_WEIGHT_AMOUNT, -0.4),
    );
    let v = t.draw_config().weight_amount;
    assert!(
        (v - 0.4).abs() < 1e-12,
        "um negativo escrito a` mao deu {v} — com `0` o pincel fica inerte e calado"
    );
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_BONE_WEIGHT_AMOUNT, 3.0),
    );
    assert!(
        (t.draw_config().weight_amount - 1.0).abs() < 1e-12,
        "a magnitude deixou de ter tecto — o peso vive em 0..1"
    );
    // ⭐ E o CONTROLO: o campo nao toca na direccao. Sem ele, um `SetValue` que escrevesse nos dois
    // devolveria o estado invisivel que a ordem do dono existe para apagar.
    assert_eq!(
        t.draw_config().weight_direction,
        WeightDirection::Add,
        "o campo do numero mexeu na DIRECCAO — o sinal voltou a viver dentro dele"
    );
}
