//! ⭐ **O CANAL host ⟷ painel** — o retrato que entra, as intenções que saem, e as alturas que o
//! dock mede.
//!
//! # Por que isto é um ficheiro irmão
//!
//! Corte feito na **INTEGRAÇÃO de 2026-09-10**, com o vermelho **ACUMULADO**: o `state.rs` chegou a
//! `615 LOC` contra o teto de `600` somando a `line/sculpt3d` e a `line/quadextract`, e nenhuma o
//! estoura sozinha (`CLAUDE.md` §5.0). ⛔ A cura é **cortar por responsabilidade**, nunca uma
//! entrada nova no `FILE_OVERAGE_OK`.
//!
//! E a fronteira é limpa: o `state.rs` declara **o que as coisas SÃO** (o `Sculpt3dUi` autorado, o
//! `Sculpt3dSnapshot` que o painel lê, o `Sculpt3dIntent` que ele emite); aqui vive **como elas
//! ATRAVESSAM o quadro**. ⚠️ As quatro células `thread_local` vêm juntas de propósito — elas são o
//! canal, e reparti-las poria metade dele num sítio e metade noutro.
//!
//! ⚠️ **Nenhum caminho de chamador muda:** o `state.rs` re-exporta as seis portas com a mesma
//! visibilidade que tinham.

use crate::state::Sculpt3dSnapshot;
use crate::state_intent::Sculpt3dIntent;
use std::cell::{Cell, RefCell};

thread_local! {
    /// O retrato vivo que o host publica antes de cada `paint`. `None` até a
    /// cena 3D existir — e é isso que faz o painel se recusar a pintar.
    static CURRENT: RefCell<Option<Sculpt3dSnapshot>> = const { RefCell::new(None) };
    /// O que o artista fez, esperando o shell drenar.
    static INTENTS: RefCell<Vec<Sculpt3dIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Host → painel, uma vez por frame antes do `paint`.
pub fn set_current_sculpt3d(snapshot: Option<Sculpt3dSnapshot>) {
    CURRENT.with(|c| *c.borrow_mut() = snapshot);
}

/// O que `paint` e `event` leem. `None` quando não há cena 3D — e aí o painel
/// **não pinta**: um painel de escultura sem escultura seria seis seções de
/// controles que não alcançam nada.
pub(crate) fn current() -> Option<Sculpt3dSnapshot> {
    CURRENT.with(|c| c.borrow().clone())
}

/// Painel → host. Enfileirado pelo `event`, drenado pela ponte do shell.
pub(crate) fn push_intent(intent: Sculpt3dIntent) {
    INTENTS.with(|c| c.borrow_mut().push(intent));
}

/// Leva tudo o que o artista fez desde o último frame.
pub fn drain_intents() -> Vec<Sculpt3dIntent> {
    INTENTS.with(|c| std::mem::take(&mut *c.borrow_mut()))
}

/// Contabilidade de rolagem (o dock do shell precisa das alturas medidas).
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Ver [`last_content_h`].
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

pub(crate) fn set_last_content_h(v: f32) {
    LAST_CONTENT_H.with(|c| c.set(v));
}

pub(crate) fn set_last_visible_h(v: f32) {
    LAST_VISIBLE_H.with(|c| c.set(v));
}
