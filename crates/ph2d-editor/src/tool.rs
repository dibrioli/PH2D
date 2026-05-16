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
}
