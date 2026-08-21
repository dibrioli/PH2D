//! **COMO A FUSÃO CONTA AO SHELL O QUE FEZ** — os dois canais de saída do
//! [`super::sprite_merge`].
//!
//! ⚠️ **Saiu de lá por medição** (2026-08-21): a fusão-em-camadas (plano
//! [`docs/Sprite_projeto/18`](../../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md)
//! W10) levou o `sprite_merge.rs` a **612** linhas contra o tecto de 600 da HR-18. A regra deste
//! projeto é **cortar, nunca declarar excepção**, e o corte é por responsabilidade: lá fica a
//! GEOMETRIA (ler, unir, deformar, compor), aqui *o que se conta a seguir*.
//!
//! # Dois canais, e não um, de propósito
//!
//! | canal | o que leva | por que separado |
//! |---|---|---|
//! | [`MergeResult`] | os bits da sprite nova | `Copy`, num `Cell` — barato, todo frame de fusão |
//! | [`MergedLayers`] | o documento com N camadas | tem `Vec`s; só existe no modo camadas |
//!
//! ⚠️ Enfiar o segundo dentro do primeiro obrigaria a converter o canal inteiro — `Cell` →
//! `RefCell`, `Copy` fora — por causa de um modo que quase nunca corre. *Um dado com outro tempo
//! de vida merece o seu canal, não uma emenda no do vizinho.*
//!
//! ⚠️ **Os dois CONSOMEM-SE ao ler.** Deixar um pendurado faria a fusão seguinte encontrar o
//! resultado da anterior e agir sobre a sprite errada.

/// **O documento em CAMADAS que a fusão produziu** (plano `docs/Sprite_projeto/18` W10).
///
/// ⚠️ Sai por aqui em vez de a fusão a instalar sozinha, e é a mesma divisão de sempre: quem tem a
/// `ToolRegistry` — e portanto o Painter — é o shell. *Este ficheiro sabe geometria, não sabe onde
/// vive a ferramenta.*
pub(crate) struct MergedLayers {
    /// A sprite que nasceu, e a que o documento pertence.
    pub entity_bits: u64,
    pub width: u32,
    pub height: u32,
    /// Uma por fonte, **de baixo para cima** — a mesma ordem em que o «over» as compôs, para o
    /// que o Painter mostra ser o que o ecrã já mostra.
    pub layers: Vec<(String, Vec<u8>)>,
}

/// Side channel for the caller to learn the newly-spawned merged
/// entity's bits so it can promote it to the selection (audit B-H2).
/// Threading a return field through the drain signature would change
/// the public surface; a thread-local set-once cell keeps the drain
/// API stable and follows the same pattern `brush_cursor` already
/// uses for input-overlay state.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MergeResult {
    pub new_entity_bits: u64,
}

thread_local! {
    static LAST_MERGE: std::cell::Cell<Option<MergeResult>> = const { std::cell::Cell::new(None) };
}

pub(super) fn last_merge_result_set(r: MergeResult) {
    LAST_MERGE.with(|c| c.set(Some(r)));
}

thread_local! {
    static LAST_LAYERS: std::cell::RefCell<Option<MergedLayers>> =
        const { std::cell::RefCell::new(None) };
}

pub(super) fn last_merged_layers_set(r: MergedLayers) {
    LAST_LAYERS.with(|c| *c.borrow_mut() = Some(r));
}

/// **Recolhe o documento em camadas** que a última fusão-em-camadas produziu.
///
/// ⚠️ Consome-o: o shell instala-o no Painter e ele deixa de existir. Deixá-lo ali faria a próxima
/// fusão **normal** encontrar as camadas da anterior e instalá-las na sprite errada.
pub(crate) fn take_last_merged_layers() -> Option<MergedLayers> {
    LAST_LAYERS.with(|c| c.borrow_mut().take())
}

/// Drain the most recent `MergeResult` (set by `drain_merge_sprites`
/// on success). Returns `None` outside the immediate frame after a
/// merge. Caller should read this AFTER `drain_merge_sprites` and
/// before the next frame to promote the new entity to the selection.
pub(crate) fn take_last_merge_result() -> Option<MergeResult> {
    LAST_MERGE.with(|c| c.take())
}
