//! Generic shell-side runtime helpers for `RasterEditTool` drivers.
//!
//! Wave 10 / Etapa 1.B (follows ADR-0041). Each bridge in
//! `shells/desktop/src/render_loop/*_bridge.rs` used to reimplement the
//! same skeleton (source push when selection changes, drain dirty into
//! cache, capture apply selection, lifecycle cleanup). These helpers
//! absorb that skeleton over the `RasterEditTool` trait surface — bridges
//! shrink ~30-50 LOC each and new bridges become trivially short.
//!
//! **What this crate IS:** pure helpers over the trait surface. No
//! tool-concrete types. No shell types (`Camera2d` / `WindowSize` /
//! `VectorScene`) — those stay in the bridges where they belong.
//!
//! **What this crate is NOT:** a god-runtime that drives every tool.
//! Each tool has genuinely tool-specific concerns (protect-mask tint
//! for BgRemoval, brush ring, panel snapshot publish) that stay in
//! the per-tool bridge — `RasterEditTool` covers raster I/O lifecycle,
//! not the tool's UI affordances. (ADR-0040 §3 documents this exception.)
//!
//! ### Helpers
//!
//! - [`drive_source_push`] — push fresh source pixels when selection drifts.
//! - [`drive_preview_cache`] — drain `current_preview()` into a cache.
//! - [`drive_pending_commit`] — capture multi-sprite Apply selection.
//! - [`drive_deactivate_cleanup`] — clear cache when tool deactivates.
//!
//! Each helper is independent — bridges compose them in whatever order
//! their tool requires.

#![forbid(unsafe_code)]

use ph2d_ecs::Entity;
use ph2d_editor_core::tool::RasterEditTool;
use std::sync::Arc;

/// Cached preview frame the shell paints between dirty-drain calls.
///
/// Bridges hold `Option<PreviewCache>` per active tool. When a tool
/// goes inactive, [`drive_deactivate_cleanup`] zeroes it. Pixels are
/// `Arc<Vec<u8>>` so the bridge can clone for paint without copying
/// the buffer (typical previews are 256×256×4 = 256 KiB).
#[derive(Clone, Debug)]
pub struct PreviewCache {
    /// The entity whose pixels produced this frame. Bridges use it
    /// to detect "selection changed since cache was taken".
    pub entity_bits: u64,
    /// Straight-alpha RGBA8, `width*height*4` bytes.
    pub rgba: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

/// Source pixels handed to a [`RasterEditTool::set_source`] call.
///
/// Bridges produce this via a shell-specific reader closure (because
/// reading sprite pixels requires `SimWorld + SpriteRenderer + AssetDb`,
/// which this crate refuses to depend on — they're shell foundation).
#[derive(Clone, Debug)]
pub struct RasterSource {
    /// Straight-alpha RGBA8, `width*height*4` bytes.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Pushes fresh source pixels into the active raster tool when the
/// active selection drifts away from the entity whose pixels were
/// last pushed.
///
/// Returns `true` if a push happened. The bridge tracks
/// `last_pushed_entity` (`&mut Option<u64>`) across frames; passing
/// `None` means "force push next time".
///
/// `read_source` is the shell-specific closure that reads pixels for
/// a given entity (typically wraps `texture_edit::read_sprite_source`).
/// Returns `None` if the entity has no readable source (atlas miss /
/// asset unloaded) — in which case this helper is a no-op.
///
/// **Effect on the tool:** invokes `tool.set_source(...)` if a fresh
/// push happens. The tool's internal dirty flag should flip — which
/// [`drive_preview_cache`] then drains.
pub fn drive_source_push<F>(
    tool: &mut dyn RasterEditTool,
    selection: Option<u64>,
    last_pushed_entity: &mut Option<u64>,
    read_source: F,
) -> bool
where
    F: FnOnce(Entity) -> Option<RasterSource>,
{
    let bits = match selection {
        Some(b) => b,
        None => return false,
    };
    if *last_pushed_entity == Some(bits) {
        return false; // already pushed this selection
    }
    let entity = Entity::from_bits(bits);
    let src = match read_source(entity) {
        Some(s) => s,
        None => return false, // shell couldn't read — try again next frame
    };
    tool.set_source(src.pixels, src.width, src.height);
    *last_pushed_entity = Some(bits);
    true
}

/// Drains `RasterEditTool::current_preview()` into `cache` if the tool
/// produced a new frame.
///
/// `selection` is the entity the cache should be tagged with — if it
/// doesn't match the cache's `entity_bits`, the cache is invalidated
/// before the drain (so a stale frame from a previous selection never
/// paints over a fresh one).
///
/// Returns `true` if a new frame landed in the cache this call.
///
/// `cache: &mut Option<PreviewCache>` — bridges typically own this
/// across frames; helper only mutates it when the tool reports dirty
/// content. When the tool returns `None`, the cache is left alone
/// (last-good frame keeps painting until the tool either updates or
/// the bridge calls [`drive_deactivate_cleanup`]).
pub fn drive_preview_cache(
    tool: &mut dyn RasterEditTool,
    selection: Option<u64>,
    cache: &mut Option<PreviewCache>,
) -> bool {
    // Selection drift → invalidate stale cache so the next paint doesn't
    // smear an old preview over a new sprite.
    if let (Some(existing), Some(sel)) = (cache.as_ref(), selection)
        && existing.entity_bits != sel
    {
        *cache = None;
    }
    let bits = match selection {
        Some(b) => b,
        None => return false,
    };
    // current_preview drains the tool's internal dirty flag (ADR-0041).
    // Some(...) means "new frame ready"; None means "nothing changed
    // since last call" — last_good cache keeps painting.
    let (pixels, width, height) = match tool.current_preview() {
        Some((p, w, h)) => (p.to_vec(), w, h),
        None => return false,
    };
    *cache = Some(PreviewCache {
        entity_bits: bits,
        rgba: Arc::new(pixels),
        width,
        height,
    });
    true
}

/// Drains `RasterEditTool::take_pending_commit()` and, if `true`,
/// returns the multi-sprite Apply selection captured from `selection_iter`.
///
/// Empty `Vec` means "no commit requested this frame". The caller is
/// the per-tool bridge — it already knows its own `tool_id` (it
/// dispatched itself), so it pushes one `EditorAction::OneShotImageOp
/// { tool_id, entity_bits }` per returned bit via the shell's action
/// bus. This helper is intentionally `tool_id`-agnostic: keeping the
/// runtime free of string identifiers means new tools don't need a
/// runtime patch to onboard.
pub fn drive_pending_commit<I>(tool: &mut dyn RasterEditTool, selection_iter: I) -> Vec<u64>
where
    I: IntoIterator<Item = u64>,
{
    if !tool.take_pending_commit() {
        return Vec::new();
    }
    selection_iter.into_iter().collect()
}

/// Lifecycle cleanup invoked when the bridge detects the raster tool
/// is no longer the active tool (the user switched, or the editor's
/// Image Tools mode toggled off).
///
/// Calls `RasterEditTool::deactivate()` so the tool drops its source
/// buffer + dirty flag + pending_apply (ADR-0041), and zeroes the
/// shell-side preview cache so the next frame doesn't paint a stale
/// overlay on top of the freshly restored sprite.
///
/// Idempotent: safe to call every frame when inactive.
pub fn drive_deactivate_cleanup(
    tool: &mut dyn RasterEditTool,
    cache: &mut Option<PreviewCache>,
    last_pushed_entity: &mut Option<u64>,
) {
    tool.deactivate();
    *cache = None;
    *last_pushed_entity = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
    use ph2d_editor_core::tool::Tool;

    /// Minimal in-memory raster tool — echoes whatever source is set,
    /// honors dirty drain on first `current_preview` call after `set_source`.
    /// Used only to exercise the runtime helpers without dragging in any
    /// concrete production tool.
    struct EchoRasterTool {
        id: ToolId,
        src: Option<(Vec<u8>, u32, u32)>,
        dirty: bool,
        pending_commit: bool,
        deactivate_calls: u32,
    }

    impl EchoRasterTool {
        fn new() -> Self {
            Self {
                id: ToolId::new("echo-raster"),
                src: None,
                dirty: false,
                pending_commit: false,
                deactivate_calls: 0,
            }
        }
    }

    impl Tool for EchoRasterTool {
        fn id(&self) -> ToolId {
            self.id.clone()
        }
        fn label(&self) -> &str {
            "Echo"
        }
        fn icon_slug(&self) -> &str {
            "echo"
        }
        fn build_panel(&self) -> FloatingPanel {
            FloatingPanel::new(self.id.clone(), "Echo")
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
            Some(self)
        }
    }

    impl RasterEditTool for EchoRasterTool {
        fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
            self.src = Some((rgba, width, height));
            self.dirty = true;
        }
        fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
            if !std::mem::take(&mut self.dirty) {
                return None;
            }
            self.src.as_ref().map(|(p, w, h)| (p.as_slice(), *w, *h))
        }
        fn take_pending_commit(&mut self) -> bool {
            std::mem::take(&mut self.pending_commit)
        }
        fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
            self.src.clone().unwrap_or_else(|| (Vec::new(), 0, 0))
        }
        fn deactivate(&mut self) {
            self.deactivate_calls += 1;
            self.src = None;
            self.dirty = false;
            self.pending_commit = false;
        }
    }

    fn dummy_source(width: u32, height: u32) -> RasterSource {
        RasterSource {
            pixels: vec![255; (width * height * 4) as usize],
            width,
            height,
        }
    }

    #[test]
    fn drive_source_push_pushes_once_per_selection() {
        let mut tool = EchoRasterTool::new();
        let mut last: Option<u64> = None;
        // First call with selection → push happens.
        let pushed =
            drive_source_push(&mut tool, Some(42), &mut last, |_| Some(dummy_source(2, 2)));
        assert!(pushed);
        assert_eq!(last, Some(42));
        assert!(tool.src.is_some());
        // Second call same selection → no push.
        let pushed2 = drive_source_push(&mut tool, Some(42), &mut last, |_| {
            panic!("read_source should NOT be called when selection unchanged");
        });
        assert!(!pushed2);
    }

    #[test]
    fn drive_source_push_no_op_on_none_selection() {
        let mut tool = EchoRasterTool::new();
        let mut last: Option<u64> = None;
        let pushed = drive_source_push(&mut tool, None, &mut last, |_| {
            panic!("read_source should NOT be called when no selection");
        });
        assert!(!pushed);
        assert!(tool.src.is_none());
    }

    #[test]
    fn drive_source_push_skips_on_unreadable_source() {
        let mut tool = EchoRasterTool::new();
        let mut last: Option<u64> = None;
        // Reader returns None → push skipped, last_pushed unchanged.
        let pushed = drive_source_push(&mut tool, Some(7), &mut last, |_| None);
        assert!(!pushed);
        assert_eq!(last, None);
    }

    #[test]
    fn drive_preview_cache_drains_dirty_and_swaps() {
        let mut tool = EchoRasterTool::new();
        let mut cache: Option<PreviewCache> = None;
        // Push source so the next current_preview returns Some.
        tool.set_source(vec![1; 16], 2, 2);
        let landed = drive_preview_cache(&mut tool, Some(99), &mut cache);
        assert!(landed);
        let c = cache.as_ref().expect("cache populated");
        assert_eq!(c.entity_bits, 99);
        assert_eq!((c.width, c.height), (2, 2));
        // Second call: tool dirty drained, so no new frame.
        let landed2 = drive_preview_cache(&mut tool, Some(99), &mut cache);
        assert!(!landed2);
        // Cache untouched (last-good frame stays).
        assert!(cache.is_some());
    }

    #[test]
    fn drive_preview_cache_invalidates_on_selection_drift() {
        let mut tool = EchoRasterTool::new();
        let mut cache: Option<PreviewCache> = None;
        tool.set_source(vec![1; 16], 2, 2);
        drive_preview_cache(&mut tool, Some(11), &mut cache);
        assert!(cache.is_some());
        // Selection drifts to a different entity → cache invalidated
        // even though no new frame is dirty (tool dirty was drained).
        let landed = drive_preview_cache(&mut tool, Some(22), &mut cache);
        assert!(!landed); // tool had nothing new
        assert!(
            cache.is_none(),
            "stale cache from entity 11 must be cleared"
        );
    }

    #[test]
    fn drive_pending_commit_returns_empty_when_no_commit_pending() {
        let mut tool = EchoRasterTool::new();
        let v = drive_pending_commit(&mut tool, vec![1u64, 2, 3]);
        assert!(v.is_empty());
    }

    #[test]
    fn drive_pending_commit_returns_full_selection_when_pending() {
        let mut tool = EchoRasterTool::new();
        tool.pending_commit = true;
        let v = drive_pending_commit(&mut tool, vec![10u64, 20, 30]);
        assert_eq!(v, vec![10, 20, 30]);
        // Drained — second call returns empty.
        let v2 = drive_pending_commit(&mut tool, vec![10u64, 20, 30]);
        assert!(v2.is_empty());
    }

    #[test]
    fn drive_deactivate_cleanup_calls_tool_and_zeros_cache() {
        let mut tool = EchoRasterTool::new();
        let mut cache: Option<PreviewCache> = Some(PreviewCache {
            entity_bits: 7,
            rgba: Arc::new(vec![0; 4]),
            width: 1,
            height: 1,
        });
        let mut last: Option<u64> = Some(7);
        drive_deactivate_cleanup(&mut tool, &mut cache, &mut last);
        assert_eq!(tool.deactivate_calls, 1);
        assert!(cache.is_none());
        assert_eq!(last, None);
    }

    #[test]
    fn drive_deactivate_cleanup_is_idempotent() {
        let mut tool = EchoRasterTool::new();
        let mut cache: Option<PreviewCache> = None;
        let mut last: Option<u64> = None;
        // Calling on already-inactive state should not panic and should
        // increment deactivate_calls each time (tool decides if its own
        // deactivate is idempotent — runtime only invokes it).
        drive_deactivate_cleanup(&mut tool, &mut cache, &mut last);
        drive_deactivate_cleanup(&mut tool, &mut cache, &mut last);
        assert_eq!(tool.deactivate_calls, 2);
        assert!(cache.is_none());
    }
}
