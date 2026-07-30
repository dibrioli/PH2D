//! **Uma linha de TEMPO diz quando não tem o que retimar** (FASE B3 do plano 12).
//!
//! O defeito, MEDIDO antes de qualquer código: as **seis** receitas da família Time
//! projetam literalmente `value` quando estão sozinhas na folha. Seis cards que o artista
//! escolhe e que não fazem **nada** — indistinguível de quebrado. E não é só a folha vazia:
//!
//! | pilha | a linha de Time retima? |
//! |---|---|
//! | `speed` sozinha | **não** — a fórmula é `value` |
//! | `speed` → `sway` | sim (`sin(((time*2) + 0)*3)`) |
//! | `speed` → `shake` | **não** — o `wiggle` carrega o próprio relógio (`ClockUse::Own`) |
//! | `speed` → `jitter` | **não** — `jitter` não lê relógio nenhum (`ClockUse::None`) |
//! | `speed` → `delay` | **não** — as duas são Time, e nenhuma OBSERVA o relógio |
//!
//! A última linha é a que decide o desenho da porta: as seis receitas de Time declaram
//! `ClockUse::Explicit` (elas leem o relógio *para o reescrever*), então perguntar
//! ingenuamente *"alguém abaixo lê o relógio?"* diria que `speed → delay` alcança alguém.
//! Só um consumidor **de valor** observa o relógio.

use ph2d_expr_recipes::{RecipeStack, RowInert};

fn reason(ids: &[&'static str], i: usize) -> Option<RowInert> {
    RecipeStack::of(ids).inert_reason(i)
}

/// **Sozinha, uma linha de Time não tem o que retimar — e diz isso.**
///
/// As SEIS, porque o defeito era da família inteira e um gate sobre uma delas ficaria
/// verde no dia em que outra fosse acrescentada.
///
/// **Mutação que deve sangrar:** `inert_reason` devolver `None` para `RowKind::Time`.
#[test]
fn a_lone_time_row_has_nothing_to_retime() {
    for id in [
        "stepped-time",
        "delay",
        "speed",
        "freeze-after",
        "start-at",
        "ping-pong-time",
    ] {
        assert_eq!(
            reason(&[id], 0),
            Some(RowInert::NothingToRetime),
            "{id} sozinha projeta `value` — a folha tem de dizer isso"
        );
        // E o oráculo do porquê: a fórmula É a identidade.
        assert_eq!(
            RecipeStack::of(&[id]).to_formula(),
            "value",
            "{id} sozinha não produz nada"
        );
    }
}

/// **Com um consumidor de relógio abaixo, ela está viva.**
///
/// O CONTROLE do gate acima: sem esta metade, `inert_reason` poderia devolver
/// `NothingToRetime` para toda linha de Time e continuar verde.
#[test]
fn a_time_row_over_a_clock_reader_is_live() {
    assert_eq!(
        reason(&["speed", "sway"], 0),
        None,
        "`sway` lê o relógio compartilhado, então o `speed` o retima"
    );
    assert!(
        RecipeStack::of(&["speed", "sway"]).to_formula() != RecipeStack::of(&["sway"]).to_formula(),
        "e o texto emitido MUDA — é isso que 'retimar' significa"
    );
}

/// **`Shake` tem relógio PRÓPRIO, e `Jitter` não lê relógio nenhum: nos dois casos a linha
/// de Time é inerte.**
///
/// ⚠️ É o fato que o plano queria tornar visível: *"o defeito nunca foi elas existirem, foi
/// a UI não dizer isso"*. O `wiggle` do parser carrega o próprio tempo, então um `Speed`
/// por cima de um `Shake` é um controle que não alcança nada.
///
/// **Mutação que deve sangrar:** aceitar `ClockUse::Own` (ou qualquer clock) como
/// observador.
#[test]
fn own_clock_and_no_clock_are_both_out_of_reach() {
    assert_eq!(
        reason(&["speed", "shake"], 0),
        Some(RowInert::NothingToRetime),
        "o `wiggle` carrega o próprio relógio"
    );
    assert_eq!(
        reason(&["speed", "jitter"], 0),
        Some(RowInert::NothingToRetime),
        "e o `jitter` não lê relógio nenhum"
    );
    // O oráculo: o desenho não muda.
    for other in ["shake", "jitter"] {
        assert_eq!(
            RecipeStack::of(&["speed", other]).to_formula(),
            RecipeStack::of(&[other]).to_formula(),
            "`speed` por cima de `{other}` emite o MESMO texto"
        );
    }
}

/// **Duas linhas de Time empilhadas seguem inertes — nenhuma delas OBSERVA o relógio.**
///
/// A armadilha do desenho, num gate: as seis receitas de Time são `ClockUse::Explicit`,
/// então a pergunta tem de excluir `RowKind::Time` do conjunto de observadores.
///
/// **Mutação que deve sangrar:** tirar o `rec.kind != RowKind::Time` do filtro.
#[test]
fn two_stacked_time_rows_still_retime_nothing() {
    assert_eq!(
        reason(&["speed", "delay"], 0),
        Some(RowInert::NothingToRetime)
    );
    assert_eq!(
        reason(&["speed", "delay"], 1),
        Some(RowInert::NothingToRetime)
    );
    assert_eq!(
        RecipeStack::of(&["speed", "delay"]).to_formula(),
        "value",
        "e o par ainda projeta a identidade"
    );
    // ...e as duas acordam juntas quando um consumidor entra embaixo.
    assert_eq!(reason(&["speed", "delay", "sway"], 0), None);
    assert_eq!(reason(&["speed", "delay", "sway"], 1), None);
}

/// **A razão do knob vazio continua vindo pela mesma porta.**
///
/// `inert_reason` subsumiu o `waiting_for` do painel; se ela deixasse de reportá-lo, a
/// linha inacabada voltaria a ser um readout que não muda.
#[test]
fn the_unfinished_row_still_reports_what_it_waits_for() {
    assert!(
        matches!(reason(&["follow"], 0), Some(RowInert::WaitingFor(_)),),
        "um `Follow` sem alvo espera o alvo"
    );
}
