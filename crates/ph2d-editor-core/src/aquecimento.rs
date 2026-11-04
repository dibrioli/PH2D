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

thread_local! {
    static PASSO_FIXO: Cell<Option<f64>> = const { Cell::new(None) };
}

/// O passo (s) de uma animação a mola neste quadro: o do relógio, salvo se o arnês o fixou.
///
/// ⛔ O relógio de parede NÃO pode decidir o que um censo mede: com a máquina carregada o quadro
/// seguinte chega mais tarde, a mola da altura de um cartão avança mais, e o mesmo painel regista
/// outras linhas (galeria: `1`, `2` ou `3` colunas na mesma árvore). Quem anima por mola lê o `dt`
/// por aqui; o arnês fixa-o ([`com_passo_fixo`]) e o resultado deixa de depender de quando se corre.
#[must_use]
pub fn passo_do_quadro(relogio: f64) -> f64 {
    PASSO_FIXO.get().unwrap_or(relogio)
}

/// Corre `f` com o passo fixo `dt` — reentrante (repõe o de fora).
pub fn com_passo_fixo<R>(dt: f64, f: impl FnOnce() -> R) -> R {
    let antes = PASSO_FIXO.replace(Some(dt));
    let r = f();
    PASSO_FIXO.set(antes);
    r
}

/// O passo de um quadro a 60 Hz — o que um teste que pinta `n` quadros passa a `com_passo_fixo`:
/// `n` quadros avançam `n × 16,7 ms` de mola, com a máquina rápida ou lenta.
pub const PASSO_DE_QUADRO_S: f64 = 1.0 / 60.0;

/// Mede `quadro` já ASSENTE: repete-o com passo fixo de `0,1 s` (o teto do quadro real) até duas
/// medidas seguidas casarem (2 no mínimo, 60 no máximo) e devolve a medida a passo `0`. Os censos e
/// os testes de pintura que comparam quadros usam-no em vez de dormir à espera da mola.
pub fn assenta_e_mede<T: PartialEq>(mut quadro: impl FnMut() -> T) -> T {
    let mut anterior: Option<T> = None;
    for _ in 0..60 {
        let n = com_passo_fixo(0.1, &mut quadro);
        if anterior.as_ref() == Some(&n) {
            break;
        }
        anterior = Some(n);
    }
    com_passo_fixo(0.0, quadro)
}
