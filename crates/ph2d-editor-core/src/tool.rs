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
    /// fold the event back into the tool's model state (e.g. update
    /// `BrushTool::size` when the Size slider was dragged).
    fn handle_panel_event(&mut self, _event: PanelEvent) {}

    /// Mutable `Any` view for downcasting in the host (e.g. snapshot
    /// push into `BgRemovalTool`). Implementors override with
    /// `fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }`.
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Capability upcast: a tool that transforms the active entity's
    /// raster returns `Some(self)`; everything else uses the default
    /// `None`. The shell drives **any** image-edit tool through one
    /// generic path ([`ImageEditTool`]) without naming a concrete
    /// type — the contract that lets ferramentas de imagem viverem em
    /// crates satélite (ADR-0040 §2.1). Implementors that also impl
    /// [`ImageEditTool`] override with `{ Some(self) }`.
    fn as_image_edit_mut(&mut self) -> Option<&mut dyn ImageEditTool> {
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
/// **every** `ImageEditTool` through a single generic loop (ADR-0040
/// §2.1): feed the source when selection changes, recompute the
/// preview as panel edits arrive (via [`Tool::handle_panel_event`]),
/// and commit at full resolution when the tool asks. No per-tool
/// `EditorAction` variant, no per-tool branch in the shell — adding an
/// image-edit tool touches nothing central.
///
/// All buffers are straight-alpha RGBA8 (`w*h*4` bytes, row-major).
/// Tool-specific interactions that don't fit this contract (eyedropper
/// colour picking, protection-brush dabs) stay on the concrete type
/// and are reached by the shell via [`Tool::as_any_mut`] downcast — a
/// documented exception (ADR-0040 §3), generalized only if a second
/// tool needs the same shape.
pub trait ImageEditTool: Tool {
    /// Hand the active entity's source pixels (straight alpha) to the
    /// tool when the selection changes. The tool caches them and
    /// computes whatever downscaled preview it needs internally.
    fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32);

    /// Current preview as `(rgba, width, height)`, if the tool
    /// produces a live preview. `None` while there is nothing to show
    /// (no source yet, or a tool that only acts on commit).
    fn preview(&self) -> Option<(&[u8], u32, u32)>;

    /// Drain the "user requested commit" flag (e.g. the Apply button).
    /// Returns `true` exactly once per request; the shell then calls
    /// [`Self::run_full`] and swaps the entity's texture.
    fn take_pending_commit(&mut self) -> bool;

    /// Run the edit at full source resolution, returning the new
    /// `(rgba, width, height)`. Called by the shell on commit.
    fn run_full(&mut self) -> (Vec<u8>, u32, u32);
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

    /// Append a tool. The first registered tool is auto-activated
    /// (and receives `on_activate`) so the editor always has a
    /// usable default.
    pub fn register(&mut self, mut tool: Box<dyn Tool>) {
        let first = self.tools.is_empty();
        if first {
            tool.on_activate();
        }
        self.tools.push(tool);
        if first {
            self.active = Some(0);
        }
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
    fn first_register_auto_activates() {
        let mut reg = ToolRegistry::new();
        let (t, a, d) = hooked("a");
        reg.register(t);
        assert_eq!(reg.tools().len(), 1);
        let active = reg.active().expect("first register should activate");
        assert_eq!(active.id(), ToolId::new("a"));
        assert_eq!(a.get(), 1);
        assert_eq!(d.get(), 0);
    }

    #[test]
    fn second_register_does_not_steal_active() {
        let mut reg = ToolRegistry::new();
        let (ta, _, _) = hooked("a");
        let (tb, ab, _) = hooked("b");
        reg.register(ta);
        reg.register(tb);
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
        assert_eq!(ab.get(), 0); // b was never activated
    }

    #[test]
    fn set_active_swaps_by_id() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.register(hooked("b").0);
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
        assert!(reg.set_active(&ToolId::new("b")));
        assert_eq!(reg.active().unwrap().id(), ToolId::new("b"));
    }

    #[test]
    fn set_active_unknown_id_returns_false_and_keeps_active() {
        let mut reg = ToolRegistry::new();
        reg.register(hooked("a").0);
        reg.register(hooked("b").0);
        assert!(!reg.set_active(&ToolId::new("nope")));
        assert_eq!(reg.active().unwrap().id(), ToolId::new("a"));
    }

    #[test]
    fn activate_and_deactivate_hooks_fire_on_switch() {
        let mut reg = ToolRegistry::new();
        let (ta, aa, da) = hooked("a");
        let (tb, ab, db) = hooked("b");
        reg.register(ta);
        // First register auto-activates.
        assert_eq!(aa.get(), 1);
        assert_eq!(da.get(), 0);

        reg.register(tb);
        assert_eq!(ab.get(), 0);

        assert!(reg.set_active(&ToolId::new("b")));
        assert_eq!(da.get(), 1); // a was deactivated
        assert_eq!(ab.get(), 1); // b was activated

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
        let m = reg.active_mut().expect("should have active");
        assert_eq!(m.id(), ToolId::new("a"));
    }

    /// A minimal `ImageEditTool` that doubles the source size on commit
    /// — exercises the generic capability upcast without any concrete
    /// tool. `Hooked` (a plain `Tool`) must NOT upcast; `Raster` must.
    struct Raster {
        id: ToolId,
        src: (Vec<u8>, u32, u32),
        pending: bool,
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
        fn as_image_edit_mut(&mut self) -> Option<&mut dyn ImageEditTool> {
            Some(self)
        }
    }

    impl ImageEditTool for Raster {
        fn set_source(&mut self, rgba: Vec<u8>, width: u32, height: u32) {
            self.src = (rgba, width, height);
        }
        fn preview(&self) -> Option<(&[u8], u32, u32)> {
            if self.src.0.is_empty() {
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
    }

    #[test]
    fn plain_tool_does_not_upcast_to_image_edit() {
        let (mut t, _, _) = hooked("plain");
        assert!(t.as_image_edit_mut().is_none());
    }

    #[test]
    fn image_edit_tool_upcasts_and_drives_through_generic_contract() {
        let mut reg = ToolRegistry::new();
        reg.register(Box::new(Raster {
            id: ToolId::new("raster"),
            src: (Vec::new(), 0, 0),
            pending: false,
        }));
        let tool = reg.active_mut().expect("active");
        let ie = tool
            .as_image_edit_mut()
            .expect("Raster should upcast to ImageEditTool");
        assert!(ie.preview().is_none(), "no source yet");
        ie.set_source(vec![1, 2, 3, 4], 1, 1);
        assert_eq!(ie.preview(), Some((&[1u8, 2, 3, 4][..], 1, 1)));
        assert!(!ie.take_pending_commit(), "no commit requested");
        let (px, w, h) = ie.run_full();
        assert_eq!((px, w, h), (vec![1, 2, 3, 4], 1, 1));
    }
}
