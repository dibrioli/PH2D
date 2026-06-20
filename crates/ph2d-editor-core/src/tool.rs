//! Tool trait + ToolRegistry — canonical contract for editor tools.
//!
//! Each tool owns its model state + builds a [`FloatingPanel`]
//! describing its UI (Procreate-style: tabs + action grid). The
//! [`ToolRegistry`] tracks the active tool and dispatches the
//! activate / deactivate hooks on switch.
//!
//! The trait is deliberately data-only: no rendering, no input
//! plumbing here. Vello paint impls and pointer dispatch land in
//! follow-up PRs that consume `Tool::build_panel()` + the model on
//! `&dyn Tool`.

use crate::floating_panel::{FloatingPanel, ToolId};
use ph2d_a11y::NodeId;

/// Pointer-driven event delivered to the active tool when the user
/// interacts with one of its panel widgets. The shell does the
/// hit-testing (using the same per-control rect math as the paint
/// pass) and dispatches the resulting event here.
///
/// 🔒 **FROZEN at ADR-0040 TG-E (2026-05-22).** Every tool's
/// `handle_panel_event` matches on these variants, so adding one ripples
/// to the whole fan-out. The cap is enforced by
/// `tests/architecture_tool_contract_surface.rs::panel_event_variant_count_is_capped`.
#[derive(Clone, Debug, PartialEq)]
pub enum PanelEvent {
    /// Plain click on a button-style action (no payload beyond the
    /// node id).
    Click(NodeId),
    /// Slider value changed to `value` (clamped to [0, 1]).
    SetValue(NodeId, f64),
    /// Toggle flipped to `on`.
    Toggle(NodeId, bool),
    /// RadioGroup option selected (the option's `value` field).
    SelectOption(NodeId, String),
}

/// Canonical contract every editor tool implements.
///
/// Implementors typically carry their own model fields (brush size,
/// snap toggles, etc.) and project that model into a fresh
/// [`FloatingPanel`] each time `build_panel` is called.
///
/// `Any` bound + [`Self::as_any_mut`] let the host downcast the
/// active tool back to its concrete type (e.g. when BgRemoval needs
/// to push a source snapshot before paint). Implementors get the
/// downcast method "for free" via the default body — `Self: Sized`
/// + `Self: Any` make `self as &mut dyn Any` always valid.
///
/// 🔒 **FROZEN at ADR-0040 TG-E (2026-05-22).** Every `ph2d-tool-*`
/// satellite crate implements this trait, so any growth ripples to the
/// whole fan-out. The method-count cap is enforced by
/// `tests/architecture_tool_contract_surface.rs::tool_contract_is_capped`.
pub trait Tool: std::any::Any {
    /// Stable id used for registry lookup + panel position memory.
    fn id(&self) -> ToolId;

    /// Human-readable label shown in the tool palette.
    fn label(&self) -> &str;

    /// String key for the tool's icon (e.g. "brush", "move"). The
    /// concrete glyph/SVG mapping lives in the icon registry that
    /// lands with the icon system PR.
    fn icon_slug(&self) -> &str;

    /// Procreate-style panel describing the tool's controls. Called
    /// when the tool becomes active; the shell mounts the returned
    /// panel into the floating layer.
    fn build_panel(&self) -> FloatingPanel;

    /// Called when this tool transitions from inactive → active.
    /// Default no-op; override to set up per-session state.
    fn on_activate(&mut self) {}

    /// Called when this tool transitions from active → inactive.
    /// Default no-op; override to flush in-progress state (e.g.
    /// commit pending stroke, drop preview overlays).
    fn on_deactivate(&mut self) {}

    /// Called when the shell's hit-test routes a pointer event to
    /// one of this tool's panel widgets. Default no-op; override to
    /// fold the event back into the tool's model state (e.g. write a
    /// slider's float value back into the tool's stored model field).
    fn handle_panel_event(&mut self, _event: PanelEvent) {}

    /// Per-frame heartbeat (`dt_ms` = real frame delta), called every render
    /// frame on the ACTIVE tool regardless of pointer input. Default no-op;
    /// override for time-evolving state that must advance while idle — e.g. the
    /// watercolor wet-on-wet diffusion that keeps blooming + drying after pen-up
    /// (ADR-0049 fluid live tick; ADR-0077 D11). The shell only ticks the active
    /// tool, so an idle non-active tool costs nothing. Implementors that do real
    /// work should return early when there is nothing to advance.
    fn on_tick(&mut self, _dt_ms: f32) {}

    /// Mutable `Any` view for downcasting in the host (e.g. snapshot
    /// push into `BgRemovalTool`). Implementors override with
    /// `fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Capability upcast: a tool that transforms the active entity's
    /// raster returns `Some(self)`; everything else uses the default
    /// `None`. The shell drives **any** raster-edit tool through one
    /// generic path ([`RasterEditTool`]) without naming a concrete
    /// type — the contract that lets ferramentas de imagem viverem em
    /// crates satélite (ADR-0040 §2.1, renomeado em ADR-0041). Implementors
    /// that also impl [`RasterEditTool`] override with `{ Some(self) }`.
    fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
        None
    }

    /// Capability upcast: a tool that paints with the pointer on the
    /// editing canvas (a brush) returns `Some(self)`; everything else
    /// uses the default `None`. The shell delivers canvas pointer
    /// samples (image-space, with pressure/tilt) through this single
    /// generic path ([`CanvasPaintTool`]) without naming a concrete
    /// type — the same shape as [`Self::as_raster_edit_mut`]. Added in
    /// ADR-0040 Amendment 3 for the new Painter (`docs/Painter/`).
    /// Implementors that also impl [`CanvasPaintTool`] override with
    /// `{ Some(self) }`.
    fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> {
        None
    }

    /// Whether this tool is the editor's initial / fallback tool — the
    /// one selected at boot and returned to when a transient tool (an
    /// image edit) deactivates. Exactly one registered tool should
    /// return `true` (the Brush); the rest use the default `false`.
    /// Data-driven default (ADR-0040 T-close): replaces the old
    /// "first-registered-wins" rule, so codegen registration order no
    /// longer decides the opening tool.
    fn is_default(&self) -> bool {
        false
    }
}

/// A tool that produces new pixels for the active entity (background
/// removal, padding, trim, make-square, real-size). The shell drives
/// **every** `RasterEditTool` through a single generic loop (ADR-0040
/// §2.1, renamed under ADR-0041): feed the source when selection changes,
/// recompute the preview as panel edits arrive (via [`Tool::handle_panel_event`]),
/// and commit at full resolution when the tool asks. No per-tool
/// `EditorAction` variant, no per-tool branch in the shell — adding a
/// raster-edit tool touches nothing central.
///
/// All buffers are straight-alpha RGBA8 (`w*h*4` bytes, row-major).
/// Tool-specific interactions that don't fit this contract (eyedropper
/// colour picking, protection-brush dabs) stay on the concrete type
/// and are reached by the shell via [`Tool::as_any_mut`] downcast — a
/// documented exception (ADR-0040 §3), generalized only if a second
/// tool needs the same shape.
///
/// **Naming:** renamed from `ImageEditTool` to `RasterEditTool` in
/// ADR-0041 (Wave 10), to reserve the slot for parallel sub-traits in
/// other domains (`VectorEditTool`, `PhysicsEditTool`, `NodeEmitTool`)
/// without the raster family holding the generic name asymmetrically.
///
/// 🔒 **FROZEN at ADR-0040 TG-E + ADR-0041 (2026-05-23).** Every
/// raster-edit tool crate implements this on top of [`Tool`]; growth
/// ripples the fan-out. The cap is enforced by
/// `tests/architecture_tool_contract_surface.rs::raster_edit_tool_contract_is_capped`.
pub trait RasterEditTool: Tool {
    /// Hand the active entity's source pixels (straight alpha) to the
    /// tool when the selection changes. The tool caches them and
    /// computes whatever downscaled preview it needs internally.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32);

    /// Drains the tool's dirty flag and returns the current preview
    /// frame if it changed since the last call (`None` if nothing
    /// is dirty / nothing to preview yet). Implementors that don't
    /// track dirty state may return `Some(...)` every call — the
    /// shell pays one cache write per frame, well under HR-4 budget.
    ///
    /// Renamed from `preview(&self)` in ADR-0041: folding the dirty
    /// check into the accessor lets the generic shell runtime retire
    /// per-tool `take_params_dirty()` inherent calls (which today
    /// require downcast to the concrete type).
    fn current_preview(&mut self) -> Option<(&[u8], u32, u32)>;

    /// Drain the "user requested commit" flag (e.g. the Apply button).
    /// Returns `true` exactly once per request; the shell then calls
    /// [`Self::run_full`] and swaps the entity's texture.
    fn take_pending_commit(&mut self) -> bool;

    /// Run the edit at full source resolution, returning the new
    /// `(rgba, width, height)`. Called by the shell on commit.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32);

    /// Lifecycle hook: the tool was deactivated (either the user
    /// switched tools, or the editor-wide Image Tools mode was toggled
    /// off). The generic shell runtime calls this so concrete tools
    /// no longer need a separate downcast path for "clear preview
    /// overlay + drop pending_apply + release source buffer".
    ///
    /// Added in ADR-0041 (Wave 10). Separate from `Tool::on_deactivate`
    /// because `Tool::on_deactivate` fires on **any** active-tool switch
    /// (including between two raster tools), whereas this fires
    /// specifically on leaving raster editing entirely.
    fn deactivate(&mut self);
}

/// Which phase of a pointer gesture a [`CanvasPointer`] sample belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerPhase {
    /// Pointer/pen made contact (button/tip down) — start of a stroke.
    Down,
    /// Pointer moved while in contact.
    Move,
    /// Pointer/pen lifted (button/tip up) — end of a stroke.
    Up,
    /// Pointer moved without contact (pen hover) — for cursor/preview only.
    Hover,
}

/// One pointer sample delivered to the active canvas-painting tool.
///
/// [`Self::pos`] is in **image/sprite-space pixels** (origin top-left, +Y down): the shell has
/// already removed the canvas pan/zoom before delivering, so the tool paints in its own buffer
/// coordinates. Pressure/tilt come straight from the device (Apple Pencil is first-class); a mouse
/// reports `pressure = 1.0` and `tilt = [0, 0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPointer {
    /// Position in image-space pixels.
    pub pos: [f32; 2],
    /// Pen pressure in `[0, 1]`; `1.0` for devices without pressure.
    pub pressure: f32,
    /// Pen tilt in `[-1, 1]` per axis; `[0, 0]` for devices without tilt.
    pub tilt: [f32; 2],
    /// Gesture phase of this sample.
    pub phase: PointerPhase,
}

/// A tool that paints with the pointer directly on the editing canvas (a brush). The shell drives
/// **every** `CanvasPaintTool` through one generic path: resolve it via
/// [`Tool::as_canvas_paint_mut`], convert each canvas pointer sample to image space, and deliver it
/// here. No per-tool `EditorAction` variant, no per-tool branch in the shell — adding a painting
/// tool touches nothing central (the same isolation `RasterEditTool` gives raster-edit tools).
///
/// 🔒 **FROZEN at ADR-0040 Amendment 3 (2026-06-20).** Growth ripples to every painting tool; the
/// cap is enforced by `tests/architecture_tool_contract_surface.rs::canvas_paint_tool_contract_is_capped`.
pub trait CanvasPaintTool: Tool {
    /// Deliver one canvas pointer sample (image-space, with pressure/tilt and gesture phase).
    /// Returns `true` if the tool consumed the sample (e.g. painted a dab), so the shell can
    /// suppress competing canvas handling (pan/zoom) for that event.
    fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool;
}

/// Owns the registered tools and tracks which one is active. The
/// shell holds one of these for the lifetime of the editor session.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
    active: Option<usize>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            active: None,
        }
    }

    /// Append a tool. **Pure push** — does NOT activate or call
    /// `on_activate`. The caller picks the opening tool via
    /// [`Self::activate_default`] (data-driven, follows `Tool::is_default`)
    /// or [`Self::set_active`] (explicit by id), so the registration
    /// order does not decide the boot tool (ADR-0040 T-close fix M3:
    /// the old "first registered is auto-active" semantics fired a
    /// spurious `on_deactivate` on the first-alphabetical tool when
    /// codegen + `activate_default` then switched away — invisible
    /// today because the affected tools' hooks are idempotent over the
    /// default state, but a latent regression magnet).
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Borrow the full palette listing (for rendering the tool bar).
    pub fn tools(&self) -> &[Box<dyn Tool>] {
        &self.tools
    }

    /// Borrow the active tool, if any.
    pub fn active(&self) -> Option<&dyn Tool> {
        self.active.map(|i| self.tools[i].as_ref())
    }

    /// Mutably borrow the active tool — used by input dispatch when
    /// pointer events need to mutate the tool's model.
    pub fn active_mut(&mut self) -> Option<&mut dyn Tool> {
        let i = self.active?;
        Some(self.tools[i].as_mut())
    }

    /// Id of the editor's default tool — the one whose [`Tool::is_default`]
    /// returns `true`, falling back to the first registered tool if none
    /// is flagged. Used for the boot selection and as the "return to"
    /// tool when a transient image-edit tool deactivates.
    pub fn default_tool_id(&self) -> Option<ToolId> {
        self.tools
            .iter()
            .find(|t| t.is_default())
            .or_else(|| self.tools.first())
            .map(|t| t.id())
    }

    /// Activate the default tool (see [`Self::default_tool_id`]). Called
    /// once at boot after `register_all_tools`, so the codegen
    /// registration order does not decide the opening tool.
    pub fn activate_default(&mut self) {
        if let Some(id) = self.default_tool_id() {
            self.set_active(&id);
        }
    }

    /// Switch active tool by id. Calls `on_deactivate` on the
    /// previous tool and `on_activate` on the new one. Returns
    /// `false` (and leaves active unchanged) if no tool with `id`
    /// is registered. Re-setting to the already-active id is a
    /// no-op success.
    pub fn set_active(&mut self, id: &ToolId) -> bool {
        let next = match self.tools.iter().position(|t| &t.id() == id) {
            Some(i) => i,
            None => return false,
        };
        if self.active == Some(next) {
            return true;
        }
        if let Some(prev) = self.active {
            self.tools[prev].on_deactivate();
        }
        self.tools[next].on_activate();
        self.active = Some(next);
        true
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    /// Tiny test-local Tool that records activate/deactivate counts
    /// into shared `Rc<Cell<u32>>` so the test can inspect them
    /// after the tool has been moved into the boxed trait object.
    struct Hooked {
        id: ToolId,
        activated: Rc<Cell<u32>>,
        deactivated: Rc<Cell<u32>>,
    }

    impl Tool for Hooked {
        fn id(&self) -> ToolId {
            self.id.clone()
        }
        fn label(&self) -> &str {
            "Hooked"
        }
        fn icon_slug(&self) -> &str {
            "hooked"
        }
        fn build_panel(&self) -> FloatingPanel {
            FloatingPanel::new(self.id.clone(), "Hooked")
        }
        fn on_activate(&mut self) {
            self.activated.set(self.activated.get() + 1);
        }
        fn on_deactivate(&mut self) {
            self.deactivated.set(self.deactivated.get() + 1);
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    type HookCounters = (Box<dyn Tool>, Rc<Cell<u32>>, Rc<Cell<u32>>);

    fn hooked(id: &str) -> HookCounters {
        let a = Rc::new(Cell::new(0));
        let d = Rc::new(Cell::new(0));
        let tool = Hooked {
            id: ToolId::new(id),
            activated: a.clone(),
            deactivated: d.clone(),
        };
        (Box::new(tool), a, d)
    }

    #[test]
    fn registry_starts_empty_and_has_no_active() {
        let reg = ToolRegistry::new();
        assert!(reg.tools().is_empty());
        assert!(reg.active().is_none());
    }

    #[test]
    fn register_is_pure_push_no_activate() {
        // ADR-0040 T-close (M3): register no longer auto-activates the
        // first tool — the boot tool is chosen explicitly via
        // `activate_default` or `set_active`. So `on_activate` MUST NOT
        // fire just from registration.
        let mut reg = ToolRegistry::new();
        let (t, a, d) = hooked("a");
        reg.register(t);
        assert_eq!(reg.tools().len(), 1);
        assert!(reg.active().is_none());
        assert_eq!(a.get(), 0);
        assert_eq!(d.get(), 0);
    }

    #[test]
    fn second_register_keeps_active_unchanged() {
        let mut reg = ToolRegistry::new();
        let (ta, _, _) = hooked("a");
        let (tb, ab, _) = hooked("b");
        reg.register(ta);
        reg.set_active(&ToolId::new("a"));
        reg.register(tb);
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
        assert_eq!(ab.get(), 0); // b was never activated
    }

    #[test]
    fn set_active_swaps_by_id() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.register(hooked("b").0);
        assert!(reg.set_active(&ToolId::new("a")));
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
        assert!(reg.set_active(&ToolId::new("b")));
        assert_eq!(reg.active().unwrap().id(), ToolId::new("b"));
    }

    #[test]
    fn set_active_unknown_id_returns_false_and_keeps_active() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.register(hooked("b").0);
        reg.set_active(&ToolId::new("a"));
        assert!(!reg.set_active(&ToolId::new("nope")));
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
    }

    #[test]
    fn activate_and_deactivate_hooks_fire_on_switch() {
        let mut reg = ToolRegistry::new();
        let (ta, aa, da) = hooked("a");
        let (tb, ab, db) = hooked("b");
        reg.register(ta);
        reg.register(tb);
        // No hooks yet — register is pure push.
        assert_eq!(aa.get(), 0);
        assert_eq!(ab.get(), 0);

        reg.set_active(&ToolId::new("a"));
        assert_eq!(aa.get(), 1); // a activated
        assert_eq!(da.get(), 0);

        assert!(reg.set_active(&ToolId::new("b")));
        assert_eq!(da.get(), 1); // a deactivated
        assert_eq!(ab.get(), 1); // b activated

        // Re-setting to the already-active tool is a no-op.
        assert!(reg.set_active(&ToolId::new("b")));
        assert_eq!(ab.get(), 1);
        assert_eq!(da.get(), 1);
        assert_eq!(db.get(), 0);
    }

    #[test]
    fn active_mut_yields_the_active_tool() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.set_active(&ToolId::new("a"));
        let m = reg.active_mut().expect("should have active");
        assert_eq!(m.id(), ToolId::new("a"));
    }

    #[test]
    fn activate_default_picks_is_default_tool_not_first() {
        // Default tool is brush-like (is_default=true) even when it's
        // registered SECOND — proves registration order doesn't decide.
        struct Brushy;
        impl Tool for Brushy {
            fn id(&self) -> ToolId {
                ToolId::new("brushy")
            }
            fn label(&self) -> &str {
                "Brushy"
            }
            fn icon_slug(&self) -> &str {
                "brushy"
            }
            fn build_panel(&self) -> FloatingPanel {
                FloatingPanel::new(self.id(), "Brushy")
            }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
            fn is_default(&self) -> bool {
                true
            }
        }
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0); // alphabetically first
        reg.register(Box::new(Brushy));
        assert_eq!(reg.default_tool_id(), Some(ToolId::new("brushy")));
        reg.activate_default();
        assert_eq!(reg.active().unwrap().id(), ToolId::new("brushy"));
    }

    #[test]
    fn activate_default_falls_back_to_first_when_no_is_default() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.register(hooked("b").0);
        // Neither flags is_default → fall back to first registered.
        assert_eq!(reg.default_tool_id(), Some(ToolId::new("a")));
        reg.activate_default();
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
    }

    /// A minimal `RasterEditTool` that echoes its source on commit
    /// — exercises the generic capability upcast without any concrete
    /// tool. `Hooked` (a plain `Tool`) must NOT upcast; `Raster` must.
    struct Raster {
        id: ToolId,
        src: (Vec<u8>, u32, u32),
        dirty: bool,
        pending: bool,
        deactivated: u32,
    }

    impl Tool for Raster {
        fn id(&self) -> ToolId {
            self.id.clone()
        }
        fn label(&self) -> &str {
            "Raster"
        }
        fn icon_slug(&self) -> &str {
            "raster"
        }
        fn build_panel(&self) -> FloatingPanel {
            FloatingPanel::new(self.id.clone(), "Raster")
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn as_raster_edit_mut(&mut self) -> Option<&mut dyn RasterEditTool> {
            Some(self)
        }
    }

    impl RasterEditTool for Raster {
        fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
            self.src = (rgba, width, height);
            self.dirty = true;
        }
        fn current_preview(&mut self) -> Option<(&[u8], u32, u32)> {
            if !std::mem::take(&mut self.dirty) || self.src.0.is_empty() {
                None
            } else {
                Some((&self.src.0, self.src.1, self.src.2))
            }
        }
        fn take_pending_commit(&mut self) -> bool {
            std::mem::take(&mut self.pending)
        }
        fn run_full(&mut self) -> (Vec<u8>, u32, u32) {
            // Trivial: echo the source back (commit semantics tested
            // per-tool; here we only prove the dispatch wiring).
            self.src.clone()
        }
        fn deactivate(&mut self) {
            self.deactivated += 1;
            self.src = (Vec::new(), 0, 0);
            self.dirty = false;
            self.pending = false;
        }
    }

    #[test]
    fn plain_tool_does_not_upcast_to_raster_edit() {
        let (mut t, _, _) = hooked("plain");
        assert!(t.as_raster_edit_mut().is_none());
    }

    #[test]
    fn raster_edit_tool_upcasts_and_drives_through_generic_contract() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Raster {
            id: ToolId::new("raster"),
            src: (Vec::new(), 0, 0),
            dirty: false,
            pending: false,
            deactivated: 0,
        }));
        reg.set_active(&ToolId::new("raster"));
        let tool = reg.active_mut().expect("active");
        let re = tool
            .as_raster_edit_mut()
            .expect("Raster should upcast to RasterEditTool");
        assert!(re.current_preview().is_none(), "no source yet");
        re.set_source(vec![1, 2, 3, 4], 1, 1);
        // First call after dirty: returns frame.
        assert_eq!(re.current_preview(), Some((&[1u8, 2, 3, 4][..], 1, 1)));
        // Second call without new source: dirty flag drained, returns None.
        assert!(re.current_preview().is_none(), "dirty drained");
        assert!(!re.take_pending_commit(), "no commit requested");
        let (px, w, h) = re.run_full();
        assert_eq!((px, w, h), (vec![1, 2, 3, 4], 1, 1));
        // deactivate() clears state.
        re.deactivate();
        assert!(re.current_preview().is_none());
    }

    /// A minimal `CanvasPaintTool` that records the pointer samples it
    /// receives — exercises the canvas-paint capability upcast (ADR-0040
    /// Amendment 3) without any concrete painting tool.
    struct Painty {
        id: ToolId,
        samples: Vec<CanvasPointer>,
    }

    impl Tool for Painty {
        fn id(&self) -> ToolId {
            self.id.clone()
        }
        fn label(&self) -> &str {
            "Painty"
        }
        fn icon_slug(&self) -> &str {
            "painty"
        }
        fn build_panel(&self) -> FloatingPanel {
            FloatingPanel::new(self.id.clone(), "Painty")
        }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn as_canvas_paint_mut(&mut self) -> Option<&mut dyn CanvasPaintTool> {
            Some(self)
        }
    }

    impl CanvasPaintTool for Painty {
        fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool {
            self.samples.push(ev);
            true
        }
    }

    #[test]
    fn plain_tool_does_not_upcast_to_canvas_paint() {
        let (mut t, _, _) = hooked("plain");
        assert!(t.as_canvas_paint_mut().is_none());
    }

    #[test]
    fn canvas_paint_tool_upcasts_and_receives_pointer() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Painty { id: ToolId::new("painty"), samples: Vec::new() }));
        reg.set_active(&ToolId::new("painty"));
        let tool = reg.active_mut().expect("active");
        let cp = tool
            .as_canvas_paint_mut()
            .expect("Painty should upcast to CanvasPaintTool");
        let down = CanvasPointer {
            pos: [12.0, 34.0],
            pressure: 0.5,
            tilt: [0.0, 0.0],
            phase: PointerPhase::Down,
        };
        assert!(cp.on_canvas_pointer(down), "tool consumed the sample");
        cp.on_canvas_pointer(CanvasPointer { phase: PointerPhase::Up, ..down });
        // Downcast back to the concrete type to inspect what it recorded.
        let painty = tool.as_any_mut().downcast_mut::<Painty>().expect("downcast");
        assert_eq!(painty.samples.len(), 2);
        assert_eq!(painty.samples[0].pos, [12.0, 34.0]);
        assert!((painty.samples[0].pressure - 0.5).abs() < 1e-6);
        assert_eq!(painty.samples[1].phase, PointerPhase::Up);
    }
}
