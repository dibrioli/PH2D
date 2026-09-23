//! ⭐ **O QUADRO DE AQUECIMENTO** — um quadro que se pinta para o painel APRENDER, e que nenhum
//! censo deve contar.
//!
//! A coluna do valor é UMA por painel (`widget::ColunaDoPainel`) e só converge no 3.º
//! quadro, logo o arnês (`ph2d-ui-testkit`) pinta dois quadros antes do que mede. ⚠️ **Os censos
//! que acumulam através da pintura inteira** (o balão, os avisos do Inspector) contavam os três:
//! medido em 2026-09-23, o balão guardava `26` rótulos que no quadro visto CABEM e o censo dos
//! avisos lia cada facto do painel escrito `3×`. No app isto nunca acontece — cada quadro é o que
//! o artista vê —, e é por isso que a porta existe só para o arnês.
//!
//! ⚠️ **Um módulo FOLHA, sem dependência nenhuma — de propósito:** o 1.º endereço foi `panel::`, e
//! o balão (em `text_elide`) a perguntar-lhe fechou um CICLO entre os módulos de topo da fundação
//! (`text_elide → panel → action_bus → interaction → text_elide`), que o gate do DAG apanhou.
//!
//! ⚠️ **Uma porta, N leitores:** um censo novo que acumule através da pintura pergunta
//! [`aquecendo`] em vez de ganhar um «descartar» próprio — duas maneiras de ignorar o mesmo
//! quadro divergem no dia em que uma delas não for chamada.

use std::cell::Cell;

thread_local! {
    static AQUECENDO: Cell<bool> = const { Cell::new(false) };
}

/// Estamos a pintar um quadro de aquecimento?
#[must_use]
pub fn aquecendo() -> bool {
    AQUECENDO.get()
}

/// Corre `f` como quadro de aquecimento — reentrante (repõe o estado de fora).
pub fn aquecendo_durante<R>(f: impl FnOnce() -> R) -> R {
    let antes = AQUECENDO.replace(true);
    let r = f();
    AQUECENDO.set(antes);
    r
}
