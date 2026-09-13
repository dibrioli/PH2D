//! **O que a `App` guarda para as FERRAMENTAS DE IMAGEM** — a transação de undo de um Apply (uma
//! entrada por sprite tocado), as caches de pré-visualização de cada ferramenta (aliases das
//! genéricas) e o predicado que decide que ferramentas a paleta mostra no modo Image Tools.
//! Irmão de `app_state.rs` pelo tecto de 600 LOC da shell.
//!
//! Corte mecânico: os tipos, os aliases e as funções saíram inteiros, verbatim. Os caminhos não
//! mudam — o `app_state.rs` re-exporta-os (`crate::app_state::UpscalePreview`, …) e o `main.rs`
//! re-exporta o que já re-exportava (`crate::ImageEditTransaction`, …).

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

/// True for any tool that belongs to the **Image Tools** group — i.e.
/// whose manifest is registered in the `"image_tools"` cluster (Bg
/// Removal, Padding, Trim Transparency, Make Square, Real Size, and
/// EVERY future image tool). Data-driven on purpose: there is no
/// hardcoded id list to keep in sync — add a tool to the `image_tools`
/// cluster in its manifest and this predicate (and everything gated on
/// it) picks it up automatically.
///
/// `installed_registry()` is `Some` in the real editor (installed at
/// boot); the `None` fallback (pre-registry boot / isolated tests)
/// reports `false`, matching the legacy "no gating" behavior there.
pub(crate) fn is_image_edit_tool(id: &ph2d_editor_core::ToolId) -> bool {
    // `m.id` is the manifest's `&'static str` id; `id` is the editor's
    // `ToolId` newtype (what `Tool::id()` returns). Bridge the two via
    // `ToolId::new`.
    ph2d_editor_core::installed_registry()
        .map(|reg| {
            reg.cluster("image_tools")
                .iter()
                .any(|m| ph2d_editor_core::ToolId::new(m.id) == *id)
        })
        .unwrap_or(false)
}

/// Indices into `tools.tools()` that are visible in the top-right tool
/// palette right now. Brush / Move are always present; the image-edit
/// tools appear ONLY while Image Tools mode is on — off, they're fully
/// gone (no icon painted, no hit zone), the absolute rule for "Image
/// Tools off ⟹ image tools inaccessible". Paint AND hit-test must both
/// map palette slots through this so their indices can't drift.
pub(crate) fn palette_visible_tool_indices(
    tools: &ph2d_editor_core::ToolRegistry,
    image_tools_mode_on: bool,
) -> Vec<usize> {
    tools
        .tools()
        .iter()
        .enumerate()
        .filter(|(_, t)| image_tools_mode_on || !is_image_edit_tool(&t.id()))
        .map(|(i, _)| i)
        .collect()
}
