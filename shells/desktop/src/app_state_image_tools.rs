//! **O que a `App` guarda para as FERRAMENTAS DE IMAGEM** — a transação de undo de um Apply (uma
//! entrada por sprite tocado), as caches de pré-visualização de cada ferramenta (aliases das
//! genéricas), o predicado que decide que ferramentas a paleta mostra no modo Image Tools, e as
//! duas portas que gravam a transação (estas vieram do `main.rs`, pelo tecto de LOC dele).
//! Irmão de `app_state.rs` pelo tecto de 600 LOC da shell.
//!
//! Corte mecânico: os tipos, os aliases e as funções saíram inteiros, verbatim. Os caminhos não
//! mudam — o `app_state.rs` re-exporta-os (`crate::app_state::UpscalePreview`, …) e o `main.rs`
//! re-exporta o que já re-exportava (`crate::ImageEditTransaction`, …).

use ph2d_render::SpriteRenderer;

/// One Apply pass — covers `entries.len()` sprites (1 for a single-sprite
/// tool like Trim, N for a multi-sprite Apply like Color EQ over a
/// selection). Restored atomically by `drain_undo_image_edit`.
pub(crate) struct ImageEditTransaction {
    /// One entry per sprite the Apply pass mutated. Drained in reverse
    /// on undo (conventional — irrelevant here since each entry targets
    /// a distinct entity, but keeps the restore order deterministic).
    pub(crate) entries: Vec<ImageEditSnapshot>,
    /// Toast label for the transaction (`"Color EQ"`, `"Bg Removal"`,
    /// `"Padding"`, …). One label per transaction even when N sprites
    /// were touched — the user sees ONE undo toast.
    pub(crate) label: &'static str,
}

/// Pre-edit snapshot of ONE sprite that an image-edit action mutated.
/// Multiple `ImageEditSnapshot`s aggregate into an
/// [`ImageEditTransaction`] for multi-sprite Apply (one entry per
/// affected sprite). Populated by the 8 image-edit drainers
/// (`drain_trim_transparency` / `drain_make_square` / `drain_rasterize`
/// / `drain_padding` / `drain_color_equalization` / `drain_upscale` /
/// `drain_equalize_sizes` / `drain_bgremoval`), consumed by
/// `drain_undo_image_edit`.
pub(crate) struct ImageEditSnapshot {
    /// Bevy entity bits of the sprite the edit targeted.
    pub(crate) entity_bits: u64,
    /// `Sprite.source` before the edit. When this is
    /// `Individual { texture_id }`, the texture is **retained**
    /// (refcount + 1 vs the natural acquire-by-the-edit path) so the
    /// undo restore can repoint without re-uploading pixels. The
    /// drainer that captured the snapshot is responsible for the
    /// matching `acquire`/refcount bump.
    pub(crate) pre_source: ph2d_render::SpriteSource,
    /// `Sprite.size` before the edit (world meters).
    pub(crate) pre_size: [f32; 2],
    /// `Transform.translation` before the edit (world meters).
    pub(crate) pre_translation: [f32; 2],
    /// `Sprite.premultiplied` before the edit. BG-Removal Apply sets it
    /// `true` (premultiplied bake, fringe fix); undo must restore the
    /// pre-edit value so the original straight-alpha source renders
    /// straight again. Trim / Make-Square leave it `false`.
    pub(crate) pre_premultiplied: bool,
    /// `Sprite.anchor` (pivot offset) before the edit. Padding's Keep
    /// mode rebases the anchor to keep content + pivot world-fixed under
    /// an asymmetric resize; undo restores the pre-edit value. `[0,0]`
    /// for edits that don't touch the pivot (and every sprite until the
    /// TOOL_PIVOT tool moves a pivot).
    pub(crate) pre_anchor: [f32; 2],
    /// The new individual texture id that the edit acquired. Released
    /// on undo so the now-orphaned post-edit texture doesn't leak.
    pub(crate) post_individual_id: u32,
    /// Human-readable label for the toast: "Trim" / "Make square".
    pub(crate) label: &'static str,
}

/// When the image-edit undo slot is being overwritten by a new edit,
/// release every pre-edit Individual texture across the previous
/// transaction's entries (multi-sprite Apply leaves N entries; the
/// single-sprite case degenerates to N=1). Atlas-backed pre-sources
/// don't need release — they share the texture via the asset_db.
/// No-op when the slot is empty.
pub(crate) fn drop_undo_pre_sources_if_individual(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
) {
    if let Some(prev) = slot.take() {
        for entry in prev.entries {
            if let ph2d_render::SpriteSource::Individual { texture_id } = entry.pre_source {
                renderer.individual_mut().release(texture_id);
            }
        }
    }
}

/// Commit `entries` (one per sprite the multi-sprite Apply touched) as
/// the new undo transaction, releasing the previous transaction's
/// pre-edit individual textures. No-op when `entries.is_empty()` (no
/// sprite actually changed → nothing to undo). The transaction label
/// comes from the first entry; per-drain code pushes the same label on
/// every entry it appends, so all N entries agree by construction.
/// ⛔ **Privada desde 2026-09-16:** todo Apply grava pela [`commit_edit`], que decide antes se a
/// ferramenta solta a imagem dos ossos — uma porta de gravar que a contornasse seria a ferramenta
/// seguinte a esquecer a regra.
fn commit_image_edit_transaction(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
    entries: Vec<ImageEditSnapshot>,
) {
    if entries.is_empty() {
        return;
    }
    let label = entries[0].label;
    drop_undo_pre_sources_if_individual(renderer, slot);
    *slot = Some(ImageEditTransaction { entries, label });
}

/// O que uma ferramenta mudou num Apply, com o id dela (`ph2d_app_painter::skin_suspend::Edicao`).
pub(crate) type Edicao = ph2d_app_painter::skin_suspend::Edicao<ImageEditSnapshot>;

/// ⭐⭐⭐ **A PORTA DE TODO APPLY** — solta dos ossos o que uma ferramenta de MOLDURA mudou (a
/// decisão é a tabela de `ph2d_app_painter::skin_suspend`) e grava a transação, no mesmo passo.
pub(crate) fn commit_edit(
    renderer: &mut SpriteRenderer,
    slot: &mut Option<ImageEditTransaction>,
    edicao: Edicao,
    sim: &mut ph2d_ecs::SimWorld,
    toasts: &mut ph2d_editor_core::ToastQueue,
) {
    let mudadas = edicao.iter().map(|e| e.entity_bits);
    solta_os_ossos(sim, toasts, edicao.ferramenta, mudadas);
    commit_image_edit_transaction(renderer, slot, edicao.entradas);
}

/// A metade da [`commit_edit`] sem transação (o *Real Size* só escreve a escala).
pub(crate) fn solta_os_ossos(
    sim: &mut ph2d_ecs::SimWorld,
    toasts: &mut ph2d_editor_core::ToastQueue,
    tool: &str,
    mudadas: impl IntoIterator<Item = u64>,
) {
    let solta = |b| ph2d_skeleton_live::skin_image::release_image(sim, b);
    ph2d_app_painter::skin_suspend::solta_se_mudou_a_moldura(tool, mudadas, solta, toasts);
}

/// Cached on-canvas preview bitmap for the Background-Removal tool.
///
/// Wave 10 / Etapa 3 STATUS: all three image-tool previews (BgR, CEQ,
/// Upscale) are now aliases of the generic `ph2d_tool_runtime::PreviewCache`
/// — same shape (entity_bits + Arc<Vec<u8>> rgba + width + height). The
/// type aliases are kept so existing call sites that say
/// `app_state::BgremovalPreview` / `ColorEqualizationPreview` /
/// `UpscalePreview` still resolve; new code should prefer
/// `ph2d_tool_runtime::PreviewCache` directly via the `drive_*` helpers.
pub(crate) type BgremovalPreview = ph2d_tool_runtime::PreviewCache;

/// GPU-side companion to [`BgremovalPreview`] (Lens F, 2026-05-26).
///
/// ⭐ **O TIPO mudou-se para a folha [`ph2d_preview_slot`]** (W2 Fase D). ⚠️ Ele NÃO foi para a
/// `ph2d-tool-runtime`, onde o gémeo de CPU (`PreviewCache`) vive: aquela crate tem um teto de
/// LOC que se declara *«the discipline mechanism»* e exige ADR para subir — o header da folha
/// tem a tabela dos DOIS destinos medidos e recusados. Ele era uma `struct` declarada aqui, e o doc dela dizia
/// *«tool-agnostic — shared by BgRemoval and the Painter»* por escrito: estava no `app_state.rs`
/// por INÉRCIA, escrito pela remoção de fundo, que era da shell. Era ele que prendia **nove**
/// ficheiros da família `painter` a este ficheiro.
///
/// ⚠️ **O alias fica, e é ele o ponto:** os dois ficheiros da remoção de fundo — e toda a prosa
/// que os cita — continuam byte a byte iguais. *O TIPO muda-se para junto do conteúdo dele; os
/// CAMPOS da `App` ficam com quem possui uma pré-visualização em curso* (a cura que o
/// `GroupDragSnapshot` levou na Fase C da `line/app-physics`).
pub(crate) type BgremovalPreviewGpu = ph2d_preview_slot::PreviewGpu;

/// Cached on-canvas live preview bitmap for the Painter tool (W1 T1.5).
/// Same generic `ph2d_tool_runtime::PreviewCache` shape as BgR / CEQ /
/// Upscale — `drive_source_push` + `drive_preview_cache` consume directly.
pub(crate) type PainterPreview = ph2d_tool_runtime::PreviewCache;

/// GPU preview slot for the Painter — same tool-agnostic shape as
/// [`BgremovalPreviewGpu`] (texture_id + dims + arc_token + entity_bits).
/// Aliased so the Painter bridge reads as its own type while sharing the
/// one implementation (W3 sprite-suppression).
pub(crate) type PainterPreviewGpu = ph2d_preview_slot::PreviewGpu;

/// Cached on-canvas live preview bitmap for the Color Equalization
/// tool. Wave 10 / Etapa 3: now uniformized with BgR + Upscale as
/// the generic `ph2d_tool_runtime::PreviewCache`. CEQ still holds
/// a `BTreeMap<u64, ColorEqualizationPreview>` (one entry per
/// selected sprite — CEQ paints per-sprite previews), driven by
/// the new `drive_multi_preview_cache` helper that replaced the
/// hand-written loop + downcast.
pub(crate) type ColorEqualizationPreview = ph2d_tool_runtime::PreviewCache;

/// Cached on-canvas live preview bitmap for the Upscale tool.
///
/// Wave 10 / Etapa 2: re-exported as the generic
/// `ph2d_tool_runtime::PreviewCache` (same shape as BgRemoval —
/// entity_bits + Arc<Vec<u8>> rgba + width + height). The
/// `UpscalePreview` alias is kept so existing call sites still
/// resolve; new code uses `PreviewCache` directly via the
/// `drive_*` helpers in `ph2d-tool-runtime`.
pub(crate) type UpscalePreview = ph2d_tool_runtime::PreviewCache;
