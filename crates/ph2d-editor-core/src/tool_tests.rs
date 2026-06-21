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
    reg.register(Box::new(Painty {
        id: ToolId::new("painty"),
        samples: Vec::new(),
    }));
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
    cp.on_canvas_pointer(CanvasPointer {
        phase: PointerPhase::Up,
        ..down
    });
    // Downcast back to the concrete type to inspect what it recorded.
    let painty = tool
        .as_any_mut()
        .downcast_mut::<Painty>()
        .expect("downcast");
    assert_eq!(painty.samples.len(), 2);
    assert_eq!(painty.samples[0].pos, [12.0, 34.0]);
    assert!((painty.samples[0].pressure - 0.5).abs() < 1e-6);
    assert_eq!(painty.samples[1].phase, PointerPhase::Up);
}
