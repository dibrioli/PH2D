//! Background-Removal panel ⟷ tool bridge + on-canvas live preview.
//!
//! Extracted from `render_loop::mod.rs` (HR-18 LOC cap) as a free
//! function, run once per frame BEFORE `paint_hero_screen`. Behavior-
//! preserving lift. Does, in order:
//!
//! 1. (Generic) Pushes the active sprite's RGBA into the `BgRemovalTool`
//!    snapshot when the selection drifts — via
//!    [`ph2d_tool_runtime::drive_source_push`] over the
//!    [`RasterEditTool`] upcast (so the tool segments the live pixels).
//! 2. (Generic) Drains `current_preview` via
//!    [`ph2d_tool_runtime::drive_preview_cache`] into the shell's
//!    `BgremovalPreview` cache (Arc-backed) for the on-canvas overlay.
//! 3. (Generic) Captures the multi-sprite Apply selection via
//!    [`ph2d_tool_runtime::drive_pending_commit`] (panel events that
//!    flipped `pending_apply` arrived earlier in the frame via
//!    `EditorAction::ToolPanelEvent → Tool::handle_panel_event`,
//!    ADR-0040 TG-B).
//! 4. (BgR-specific) Panel visibility (shown iff bgremoval is active,
//!    Inspector hidden while active), panel-store reset on Reset click,
//!    BgRemoval snapshot publish, protect-mask tint overlay, brush ring.
//! 5. (BgR-specific) On-canvas overlay paints the cached preview RGBA
//!    on top of the sprite's footprint (the sprite itself is suppressed
//!    from the sprite pass while previewing — see `sim_extract`).
//!
//! Wave 10 / Etapa 1.B (ADR-0041 follow-up): the parts marked
//! "(Generic)" used to be inlined ~80 LOC of per-tool boilerplate; they
//! now live in `ph2d-tool-runtime` and are composed by helper calls.
//! What stays here is BgR-specific (protect mask, brush ring, panel
//! snapshot — all documented `as_any_mut` exceptions per ADR-0040 §3).

use crate::app_state::{BgremovalPreview, BgremovalPreviewGpu};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ToolRegistry;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, SpriteRenderer};
use ph2d_tokens::ColorToken;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A prévia de GPU e os overlays — filho por ASSUNTO, num ficheiro próprio para este caber no tecto.
#[path = "bgremoval_preview_gpu.rs"]
mod preview_gpu;
use preview_gpu::{draw_overlays, upload_preview};

/// Returns `true` iff an Apply committed this frame (the caller then
/// tears the tool down — deactivate + restore Inspector — so the
/// on-canvas preview overlay stops re-rendering on top of the freshly
/// baked sprite, which otherwise reads as a ghost edge outline while the
/// tool stays selected).
///
/// Lens F (2026-05-26): also owns the lifecycle of a transient
/// `IndividualTextureStore` slot that backs the live preview through
/// the sprite pipeline (replacing the gamma/blend-divergent Vello
/// overlay). The slot id lives in `bgremoval_preview_gpu`; the next
/// frame's `sim_extract` reads it directly to emit a
/// [`PreviewOverride`](super::sim_extract::PreviewOverride) — no
/// override is plumbed through this return.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub(super) fn dispatch(
    hero: &mut HeroScreen,
    tools: &mut ToolRegistry,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    last_bgremoval_pushed_entity: &mut Option<u64>,
    bgremoval_preview: &mut Option<BgremovalPreview>,
    bgremoval_preview_gpu: &mut Option<BgremovalPreviewGpu>,
    toasts: &mut ToastQueue,
) -> bool {
    let bgremoval_is_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor_core::ToolId::new("bgremoval"))
        .unwrap_or(false);

    // ── (Generic) Source push when selection drifts ───────────────────────
    // Wave 10 / Etapa 1.B: replaces the previous hand-written push block
    // (~30 LOC) with one call to `drive_source_push` over the
    // `as_raster_edit_mut()` upcast. The closure carries the
    // shell-specific pixel readback (asset_db + sprite renderer).
    if bgremoval_is_active
        && let Some(tool) = tools.active_mut()
        && let Some(raster) = tool.as_raster_edit_mut()
    {
        let _pushed = ph2d_tool_runtime::drive_source_push(
            raster,
            hero.gizmo.selection,
            last_bgremoval_pushed_entity,
            |entity| {
                // PRECISION-READONLY: alimenta a PRÉVIA da ferramenta e mais nada. Quem escreve
                // os pixels de volta é o `hero_intents::image_edit::bgremoval`, que relê a fonte e
                // passa por `commit_geometric_edit` — o funil que preserva 16 bits quando pode e
                // avisa quando não pode.
                let src = crate::hero_intents::texture_edit::read_sprite_source(
                    entity,
                    sim,
                    renderer,
                    asset_db,
                    atlas_asset_map,
                )?;
                // Single readback chokepoint: pixels arrive carrying their
                // alpha mode (a prior premultiplied BG-Removal bake
                // included). The segmentation snapshot reasons about true
                // colours, so normalize to straight once.
                let straight = src.image.into_straight();
                Some(ph2d_tool_runtime::RasterSource {
                    pixels: straight.pixels,
                    width: straight.width,
                    height: straight.height,
                })
            },
        );
    }

    // ── (BgR-specific) Panel visibility + Inspector toggle ────────────────
    // Shown iff bgremoval is the active tool (keyed "bgremoval" to match
    // `BgRemovalPanel::ID`). Hide the Inspector while bgremoval is active
    // (image tools dock into the Inspector slot).
    //
    // Wave 10 Etapa 4 smoke fix (Enio 2026-05-24): only write
    // `inspector` visibility on the activate↔deactivate EDGE, not every
    // frame. The level-triggered version stomped on the user's rail
    // toggle whenever no image tool was active — every frame would
    // force `inspector = true`, undoing the manual hide. Edge trigger
    // preserves the rail-toggle semantics while still hiding inspector
    // at tool activation + restoring at deactivation.
    hero.panel_visibility
        .insert("bgremoval", bgremoval_is_active);
    // ⭐⭐⭐ **A FERRAMENTA ACOMPANHA O INSPECTOR, NÃO O SUBSTITUI** — ver o irmão no
    //    `upscale_bridge.rs`, que traz o mecanismo inteiro (report do Enio, 2026-09-08).
    //
    // ⚠️⚠️ **Este bridge escapou à wave 40 e a razão é da RÉGUA, não do código:** o censo que
    //    apagou os outros cinco procurava `panel_visibility.insert("inspector"` no texto CRU, e
    //    aqui o `rustfmt` parte a chamada em duas linhas porque o receptor é longo. *Um censo
    //    textual tem de conhecer TODAS as formas do que lê* — hoje ele tira o espaço todo antes de
    //    comparar, e ao fazê-lo acusou TRÊS de uma vez.

    // Captured for the protection-mask overlay tint + brush ring (built
    // while the tool is borrowed below, drawn in the on-canvas overlay block).
    let theme = hero.theme;
    let mut protect_tint: Option<(Arc<Vec<u8>>, u32, u32)> = None;
    let mut brush_ring: Option<(f32, u32)> = None;
    let mut needs_panel_reset = false;
    let mut apply_selection: Vec<u64> = Vec::new();

    // ── (Mixed) Generic preview cache drain + BgR-specific extras ─────────
    // The (Generic) parts go through tool-runtime helpers; the BgR-specific
    // bits (panel snapshot, panel reset, protect tint, brush ring) keep
    // their downcast — ADR-0040 §3 documented exception. We acquire the
    // BgRemovalTool downcast once and do BOTH inside it so we don't
    // borrow the tool twice (the runtime helpers take &mut dyn
    // RasterEditTool; the BgR-specific bits take &mut BgRemovalTool).
    if let Some(tool) = tools.active_mut()
        && let Some(bg) = tool
            .as_any_mut()
            .downcast_mut::<ph2d_tool_bgremoval::BgRemovalTool>()
    {
        // (Generic) Drain current_preview into the shell cache.
        // ADR-0041: current_preview drains the tool's dirty flag, so the
        // helper just needs to call it. Selection drift is handled by the
        // helper's own invalidation pass.
        ph2d_tool_runtime::drive_preview_cache(bg, hero.gizmo.selection, bgremoval_preview);

        // (Generic) Capture the multi-sprite Apply selection.
        apply_selection = ph2d_tool_runtime::drive_pending_commit(bg, hero.gizmo.iter_selected());

        // (BgR-specific) Panel-store reset propagation.
        if bg.take_pending_panel_reset() {
            needs_panel_reset = true;
        }

        // (BgR-specific) Snapshot publish for the docked panel.
        #[cfg(feature = "panel-bgremoval")]
        ph2d_panel_bgremoval::set_current_bgremoval_snapshot(if bgremoval_is_active {
            Some(bg.ui_snapshot())
        } else {
            None
        });

        // (BgR-specific) Protection-mask overlay tint, gated on Show-Mask.
        //
        // ⚠️ **A LEI VEM DA PORTA, e não é soletrada aqui.** Ela era a conjunção escrita à mão, e
        // esta era a última das cópias: o painel decide se o interruptor sequer é alcançável pela
        // mesma pergunta (`ph2d_tool_bgremoval::params::mask_overlay_renders`), e duas escritas da
        // mesma lei podem divergir — o interruptor ofereceria o que o canvas não pinta.
        // *Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
        if bgremoval_is_active && bg.mask_overlay_renders() {
            let (mask, mw, mh) = bg.protect_mask_source();
            let accent = ColorToken::Accent.resolve(theme);
            if let Some((tw, th, buf)) =
                build_protect_tint(mask, mw, mh, [accent.r, accent.g, accent.b])
            {
                protect_tint = Some((Arc::new(buf), tw, th));
            }
        }

        // (BgR-specific) Brush-size ring: capture the radius (source px)
        // + source width while the protect brush is armed. The
        // "Add area" selector is a single-click flood-fill — no ring.
        if bgremoval_is_active && bg.is_protect_armed() {
            brush_ring = Some((bg.brush_radius_px(), bg.source_size().0));
        }
    }

    // ── Inactive path — clear LOCAL bridge state only ────────────────────
    // Wave 10 / Etapa 2 audit [C1 CRITICAL fix]: previous version called
    // `drive_deactivate_cleanup` on `tools.active_mut()` here, but that
    // returns the CURRENTLY-ACTIVE tool (which may be CEQ or Upscale —
    // other RasterEditTools), NOT the BgRemoval tool we're a bridge for.
    // Calling `RasterEditTool::deactivate()` on the wrong tool zeroes
    // the state of whichever raster tool is active (drains its
    // pending_apply, params_dirty, cached_canvas_preview, etc.) —
    // destroying its session.
    //
    // BgRemoval's own `Tool::on_deactivate` already fires when
    // `ToolRegistry::set_active` switches AWAY from bgremoval, and that
    // path already clears `cached_canvas_preview` + all transient flags
    // (Etapa 1.B audit fix A2). The bridge only needs to clear its own
    // shell-side cache here.
    if !bgremoval_is_active {
        *bgremoval_preview = None;
        *last_bgremoval_pushed_entity = None;
    }
    // Reset just fired — re-populate the panel's `WidgetStore` so
    // every slider knob / chip text snaps back to defaults. Without
    // this, `params` resets but the slider visuals stay where the
    // user dragged them (panel paints `store.slider(id)`, not the
    // snapshot).
    if needs_panel_reset {
        ph2d_editor_core::panel::with_registry_opt(|reg| {
            if let Some(idx) = reg.find_by_panel_node_id(ph2d_editor_core::ids::BGR_PANEL) {
                reg.panels_mut()[idx].populate(&mut hero.store);
            }
        });
    }
    upload_preview(bgremoval_preview, bgremoval_preview_gpu, renderer, toasts);

    // ── Apply commit dispatch ─────────────────────────────────────────────
    if !apply_selection.is_empty() {
        for bits in &apply_selection {
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::OneShotImageOp {
                    tool_id: "bgremoval",
                    entity_bits: *bits,
                });
        }
        // Committed result becomes the new sprite texture; drop the
        // preview cache so the next frame's lifecycle releases the
        // transient GPU slot and the override returns to `None`.
        *bgremoval_preview = None;
    }

    draw_overlays(
        bgremoval_preview,
        protect_tint,
        brush_ring,
        sim,
        hero,
        camera,
        window_size,
        vector_scene,
        theme,
    );
    !apply_selection.is_empty()
}

/// Build a capped-resolution RGBA tint image from a source-resolution
/// protection mask: protected pixels get `rgb` at [`TINT_ALPHA`], the
/// rest are fully transparent. Downsampled (nearest) to at most
/// [`TINT_CAP`] on the long axis — it's a coarse visual hint, drawn
/// scaled to the footprint anyway. Returns `None` when nothing is
/// painted (so the overlay draws nothing).
fn build_protect_tint(mask: &[u8], mw: u32, mh: u32, rgb: [u8; 3]) -> Option<(u32, u32, Vec<u8>)> {
    /// Long-axis cap for the tint buffer (visual hint, not crisp art).
    const TINT_CAP: u32 = 256;
    /// Tint opacity (~38%). // LITERAL-OK: overlay hint alpha budget
    const TINT_ALPHA: u8 = 96;
    /// Mask byte threshold (`>=` ⇒ protected).
    const PROTECT_THRESHOLD: u8 = 128;

    if mask.is_empty() || mw == 0 || mh == 0 {
        return None;
    }
    let (tw, th) = if mw <= TINT_CAP && mh <= TINT_CAP {
        (mw, mh)
    } else if mw >= mh {
        (
            TINT_CAP,
            ((mh as u64 * TINT_CAP as u64 / mw as u64).max(1)) as u32,
        )
    } else {
        (
            ((mw as u64 * TINT_CAP as u64 / mh as u64).max(1)) as u32,
            TINT_CAP,
        )
    };
    let mut out = vec![0u8; (tw as usize) * (th as usize) * 4];
    let mut any = false;
    for y in 0..th as usize {
        let sy = ((y as u64) * (mh as u64) / (th as u64)).min(mh as u64 - 1) as usize;
        for x in 0..tw as usize {
            let sx = ((x as u64) * (mw as u64) / (tw as u64)).min(mw as u64 - 1) as usize;
            if mask[sy * mw as usize + sx] >= PROTECT_THRESHOLD {
                let b = (y * tw as usize + x) * 4;
                out[b] = rgb[0];
                out[b + 1] = rgb[1];
                out[b + 2] = rgb[2];
                out[b + 3] = TINT_ALPHA;
                any = true;
            }
        }
    }
    if any { Some((tw, th, out)) } else { None }
}

/// ⛔ **CENSO: a lei do overlay da máscara só se pergunta pela PORTA** — *"o tint desenha agora?"*
/// tem **uma** resposta (`BgRemovalTool::mask_overlay_renders`) e a visibilidade do interruptor é
/// derivada dela; o que duas escritas produziram está em `seam_show_mask_reachability.rs`.
/// ⚠️ **Lê o fonte SEM comentários**, senão a prosa que **cita** a expressão proibida para a
/// explicar reprovaria o gate. *Um gate que lê a prosa sobre a lei mede o autor, não o código.*
#[cfg(test)]
mod show_mask_census {
    /// Todo `.rs` sob `shells/desktop/src/`, já sem as linhas de comentário.
    fn shell_code() -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut stack = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                    && let Ok(t) = std::fs::read_to_string(&p)
                {
                    let c = t.lines().filter(|l| !l.trim_start().starts_with("//"));
                    let code = c.collect::<Vec<_>>().join("\n");
                    out.push((p.display().to_string(), code));
                }
            }
        }
        out
    }

    /// ⚠️⚠️ **Agulha MONTADA** — a 1.ª corrida deste censo reprovou o ficheiro que o hospeda, por
    /// achar a própria lista; e a metade JUSTA, por extenso, lia-se a si mesma e ficaria verde sem
    /// overlay nenhum. *Uma sonda que se lê a si mesma mede a sonda.*
    fn needle(a: &str, b: &str) -> String {
        [a, b].concat()
    }

    #[test]
    fn the_mask_overlay_law_is_never_spelled_out_again_in_the_shell() {
        let files = shell_code();
        assert!(!files.is_empty(), "o censo nao encontrou fonte nenhum");
        for (path, code) in &files {
            for cru in [needle("show", "_mask()"), needle("has_protect", "_mask()")] {
                assert!(
                    !code.contains(&cru),
                    "{path}: `{cru}` e' a lei soletrada a' mao. Quem DESENHA pergunta a PORTA \
                     (`mask_overlay` + `_renders()`); o acessor cru e' o PEDIDO do artista, nao a \
                     resposta. *Uma lei escrita em dois sitios ainda nao e' uma lei; so' uma \
                     PORTA e'.*"
                );
            }
        }
        // ⚠️ **A metade JUSTA:** a porta é de facto chamada — senão o censo ficava verde no dia em
        // que alguém apagasse o overlay (*ausência lê-se igual a obediência*).
        let porta = needle("bg.mask_overlay", "_renders()");
        assert!(
            files.iter().any(|(_, c)| c.contains(&porta)),
            "ninguem na shell pergunta a' porta — o tint da mascara deixou de ter consumidor"
        );
    }
}
