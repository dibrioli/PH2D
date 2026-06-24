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

    /// Mutably borrow a registered tool by id, **active or not**. Used by a bridge that must finalize
    /// its own tool after it deactivated (e.g. the Painter baking its kept canvas into the sprite on
    /// close — `active_mut` would return the NEW tool). Returns `None` if no tool has that id.
    pub fn tool_by_id_mut(&mut self, id: &ToolId) -> Option<&mut dyn Tool> {
        self.tools
            .iter_mut()
            .find(|t| t.id() == *id)
            .map(|t| t.as_mut())
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
#[path = "tool_tests.rs"]
mod tests;
