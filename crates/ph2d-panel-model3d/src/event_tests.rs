//! ⚠️ **O que se mede AQUI é a fila; a costura mede-se em `tests/seam.rs`.**
//!
//! A primeira versão deste arquivo tinha três testes que empurravam intents à mão e afirmavam o
//! que saía — ou seja, mediam o `Vec`, não o `apply_event`. O gate de arquitetura
//! `architecture_interactive_crate_has_behavioral_test` recusou a crate por isso, e estava certo:
//! um braço em falta em `event.rs` deixaria o controle **pintado, arrastável e morto**, com aqueles
//! três verdes. Sobrou o único que a costura não cobre.

use super::*;

/// **A fila esvazia** — um intent drenado não volta no quadro seguinte.
///
/// Sem isto, uma edição seria re-aplicada a cada quadro e o raio andaria sozinho enquanto ninguém
/// tocasse em nada. É propriedade da FILA, e por isso mora aqui e não na costura.
#[test]
fn draining_the_queue_empties_it() {
    let _ = state::drain_intents();
    state::push_intent(ModelIntent::SetParam {
        entity: 0,
        param: ph2d_field::Param::Dim(0),
        value: 0.1,
    });
    assert_eq!(state::drain_intents().len(), 1);
    assert!(
        state::drain_intents().is_empty(),
        "um intent drenado não pode voltar"
    );
}
