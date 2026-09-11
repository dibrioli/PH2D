//! **A porta dos diálogos modais — o código mudou-se, o endereço ficou.**
//!
//! A lei vive agora em [`ph2d_app_host::modal`] (W2): ela é `thread_local` + `rfd` + aritmética de
//! relógio, não tem uma linha de `App`, e as **seis** famílias que saem da shell precisam dela.
//!
//! # ⚠️ Por que isto é um ALIAS e não uma reescrita dos chamadores
//!
//! `crate::modal::save_file` está escrito em ficheiros de **outras linhas vivas** (`sculpt3d`,
//! image tools, tokens, sheet, texto vetorial — 25 chamadas em 12 ficheiros, medido 2026-08-22), e
//! cinco dessas linhas estão a mover os próprios ficheiros **hoje**. Reescrever o endereço nelas
//! seria editar a árvore de outra linha para não ganhar nada: o alias custa três linhas e entrega
//! **uma porta só**, que é o que a lei desta casa pede.
//!
//! ⇒ Quem **chegar novo** escreve `ph2d_app_host::modal::…` directamente. Quem já existe não muda.

// ⚠️ **`note_stall` NÃO está aqui, e a ausência é medida:** nada na shell o chama — quem declarava
// o congelamento era o `timed` da própria porta, que se mudou com ela. Re-exportá-lo «por
// simetria» seria um `warning: unused import`, que o clippy `-D warnings` do gate de fecho
// transforma em vermelho. *Um alias exporta o que tem chamador, não o que parece completo.*
pub(crate) use ph2d_app_host::modal::{chrome_dt, pick_file, save_file, take_stall};

#[cfg(test)]
#[path = "modal_tests.rs"]
mod tests;
