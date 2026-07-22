//! `ph2d-ui-testkit` — headless harness to drive a `Panel` + `Tool` seam
//! in a unit test, **without** the desktop shell or a GPU.
//!
//! ## Why this exists (blindagem Fase 0.1)
//!
//! The 2026-06-20 forensic diagnosis found the recurring "green-but-dead"
//! bug class: a widget that PAINTS, REGISTERS and COMPILES but is silently
//! inert because one of the ~13 hand-wired sites between the panel
//! (`WidgetStore` ids + `apply_event`) and the tool (`handle_panel_event`
//! → `apply_ui_edit`) was forgotten. Unit tests on the tool stay green
//! (they call `apply_ui_edit` directly) and the `*_contract_surface` gates
//! stay green (they count symbols, not behavior). Nobody finds the dead
//! wire until a human clicks it.
//!
//! [`MockPanelHost`] closes that gap: it is a real `PanelHostInternal`
//! backed by a real [`WidgetStore`] + [`ActionBus`], so a test can run the
//! FULL path the shell runs —
//!
//! ```text
//! P::populate(store)               // boot registration
//!   → set the widget's stored value (what a drag does)
//!   → P::apply_event(state, host, WidgetEvent::ValueChanged(id))   // panel
//!   → host.drained_actions()       // what the shell drains each frame
//!   → tool.handle_panel_event(pe)  // shell forwards ToolPanelEvent
//!   → assert tool.<observable state> changed
//! ```
//!
//! If ANY site in that chain is missing, the assertion goes red. That is
//! the behavioral signal the project was missing.
//!
//! ## Placement note
//!
//! This crate is deliberately **not** named `ph2d-panel-*` / `ph2d-tool-*`
//! / `ph2d-node-*`: those prefixes are swept by the registry codegen and
//! the LOC-cap gate's `collect_panel_dirs`. A test-only crate with one of
//! those prefixes would be mis-registered (the `node-sync glob` gotcha).
//! Consume it from a panel/tool crate's `[dev-dependencies]` — the
//! `architecture_cycle_prevention` gate reads `[dependencies]` only, so a
//! dev-dep edge builds no runtime cycle.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::dispatch_pointer;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHost, PanelHostInternal};
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::screens::{HeroLayout, HeroSelection};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// One second of the synthetic pointer clock — the gap [`MockPanelHost::click_at`] leaves
/// between clicks so the dispatcher never mistakes two of them for a double-click.
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// A throwaway [`PanelHostInternal`] for tests. Holds the real widget
/// store + action bus a panel reads/writes; the grid-snap / selection /
/// visibility surfaces are present (the trait requires them) but inert —
/// most seam tests only touch `store()` / `store_mut()` / `bus_mut()`.
pub struct MockPanelHost {
    store: WidgetStore,
    hit_index: HitIndex,
    bus: ActionBus,
    selection: Option<HeroSelection>,
    grid_snap: GridSnapState,
    grid_snap_panel_rect: Option<Rect>,
    project: ProjectSettings,
    theme: Theme,
    visible: BTreeMap<String, bool>,
    /// Monotonic clock for the synthetic pointer events ([`MockPanelHost::click_at`]). A
    /// counter, not a wall clock: the dispatcher's double-click window is a real threshold,
    /// so the gap between clicks has to be deterministic or the gate flakes.
    clock_ns: u128,
}

impl MockPanelHost {
    /// An empty host: no widgets registered yet. Use [`Self::with_panel`]
    /// to also run a panel's `populate`.
    pub fn new() -> Self {
        Self {
            store: WidgetStore::with_capacity(32),
            hit_index: HitIndex::new(),
            bus: ActionBus::new(),
            selection: None,
            grid_snap: GridSnapState::default(),
            grid_snap_panel_rect: None,
            project: ProjectSettings::default(),
            theme: Theme::default(),
            visible: BTreeMap::new(),
            clock_ns: 0,
        }
    }

    /// A host with panel `P`'s widgets pre-registered (runs `P::populate`,
    /// the same boot step the host orchestrator runs once).
    pub fn with_panel<P: Panel>() -> Self {
        let mut host = Self::new();
        P::populate(&mut host.store);
        host
    }

    /// Drive one event through `P::apply_event` — the exact entry point the
    /// host dispatcher uses. Returns the panel's [`EventOutcome`].
    pub fn apply_panel_event<P: Panel>(
        &mut self,
        state: &mut P::State,
        ev: WidgetEvent,
    ) -> EventOutcome {
        P::apply_event(state, self, ev)
    }

    /// Set a registered slider's stored value — what a pointer drag writes
    /// into the store *before* the dispatch emits `ValueChanged(id)`. Panics
    /// (never silently no-ops) if `id` is absent or not a slider.
    /// Read-only view of the widget store — so a seam test can assert on the state a
    /// panel's `populate` seeded (collapsed sections, default values, …), not just on
    /// what an event did to it. Additive: nothing in the testkit changes shape.
    pub fn store(&self) -> &WidgetStore {
        &self.store
    }

    pub fn set_slider_value(&mut self, id: NodeId, value: f32) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Slider { value: v, .. }) => *v = value,
            Some(_) => panic!("set_slider_value: {id:?} is registered but is not a Slider"),
            None => panic!("set_slider_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered number chip's committed value. Panics if `id` is
    /// absent or not a `NumberInput`.
    pub fn set_number_value(&mut self, id: NodeId, value: f64) {
        match self.store.get_mut(id) {
            Some(InteractiveState::NumberInput { value: v, .. }) => *v = value,
            Some(_) => panic!("set_number_value: {id:?} is registered but is not a NumberInput"),
            None => panic!("set_number_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered toggle's stored on-state — what the paint pass mirrors
    /// from the snapshot before dispatch emits `Toggled(id)`. Panics if `id` is
    /// absent or not a `Toggle`.
    pub fn set_toggle_on(&mut self, id: NodeId, on: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Toggle { on: o, .. }) => *o = on,
            Some(_) => panic!("set_toggle_on: {id:?} is registered but is not a Toggle"),
            None => panic!("set_toggle_on: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered dropdown's open state — what the generic dispatcher writes when
    /// the user clicks a chip. Panics if `id` is absent or not a `Dropdown`.
    ///
    /// **Why the testkit needs this at all:** the open/close of a dropdown is done by the
    /// SHELL's generic dispatch, not by the panel's `apply_event` — so a seam test driving
    /// `apply_event` alone cannot reach the state *"this popover is open"*, and any rule
    /// about it (e.g. *opening one closes the other*) would be untestable at this seam.
    pub fn set_dropdown_open(&mut self, id: NodeId, open: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Dropdown { open: o, .. }) => *o = open,
            Some(_) => panic!("set_dropdown_open: {id:?} is registered but is not a Dropdown"),
            None => panic!("set_dropdown_open: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Read a registered dropdown's open state (the mirror of
    /// [`Self::set_dropdown_open`], so a gate can assert on it).
    #[must_use]
    pub fn dropdown_is_open(&self, id: NodeId) -> Option<bool> {
        match self.store.get(id) {
            Some(InteractiveState::Dropdown { open, .. }) => Some(*open),
            _ => None,
        }
    }

    /// Drain everything the panel pushed onto the action bus so far. The
    /// shell does the same each frame; tests inspect the result to assert
    /// the panel actually emitted the right [`EditorAction`].
    pub fn drained_actions(&mut self) -> Vec<EditorAction> {
        self.bus.drain().collect()
    }

    /// **Run panel `P`'s REAL paint pass, headless, and return what it made
    /// clickable** — every `(id, rect)` the paint registered in the hit index.
    ///
    /// This closes the last hole in the "green-but-dead" family. The seam test
    /// above proves `populate → apply_event → tool`; the wiring-parity gate
    /// reads *source text*. **Neither one runs `paint`.** So a widget can be
    /// registered, wired, unit-tested and contract-clean while its paint call
    /// sits behind an early `return` (or was never written at all) — and the
    /// user's report is simply *"the button doesn't exist"*, with every gate
    /// green. What a user can click is what the PAINT registered, so that is
    /// what a test must read.
    ///
    /// The panel is forced visible first (a paint gated on `panel_visible`
    /// would otherwise return before drawing anything).
    pub fn paint<P: Panel>(&mut self, state: &mut P::State, viewport: Rect) -> Vec<(NodeId, Rect)> {
        self.paint_with_layout::<P>(state, HeroLayout::for_viewport(viewport), viewport)
    }

    /// [`Self::paint`], but with the layout given explicitly.
    ///
    /// `for_viewport` builds an UNSPLIT centre (`CenterSplit::None`), and a panel that lives
    /// in the split — the Motion graph, the Motion timeline slot — then gets a **zero-sized
    /// rect and returns before drawing anything**. Every paint gate for those panels was
    /// therefore impossible to write, which is why none existed: the add-menu could be
    /// unclickable in the running app with every test in the crate green (Enio, smoke
    /// 2026-07-13). A harness that cannot lay a panel out cannot test what the artist clicks.
    pub fn paint_with_layout<P: Panel>(
        &mut self,
        state: &mut P::State,
        layout: HeroLayout,
        viewport: Rect,
    ) -> Vec<(NodeId, Rect)> {
        self.set_panel_visible(P::ID, true);
        self.hit_index.clear_for_frame();
        let mut scene = VectorScene::new();
        let mut text_system = TextSystem::without_system_fonts();
        {
            let mut ctx = PaintCtx {
                host: self,
                layout: &layout,
                viewport,
                scene: &mut scene,
                text_system: &mut text_system,
            };
            P::paint(state, &mut ctx);
        }
        self.hit_index.iter_registrations().collect()
    }

    /// **Drive a real pointer event through the real dispatcher**, against the hit index the
    /// last [`Self::paint`] filled in.
    ///
    /// This is the half that hand-pushed gestures skip — and therefore the half where a
    /// regression hides: the dispatcher decides whether a click on a popup becomes a widget
    /// event, a graph gesture, or nothing at all, purely from what the PAINT registered. A
    /// test that pushes the gesture itself has already assumed the answer.
    ///
    /// Returns the widget events the dispatch emitted (a graph gesture is not one of them —
    /// it lands in the store, for the panel's next `paint` to drain).
    pub fn dispatch_pointer_event(&mut self, event: ph2d_host::PointerEvent) -> Vec<WidgetEvent> {
        let arena = bumpalo::Bump::new();
        // `_with_text` — the variant the SHELL uses (it threads a live `TextSystem` so a click
        // into a text field lands the caret on the right glyph). A harness that dispatched the
        // no-text variant would be testing a path the app does not take.
        let mut ts = TextSystem::without_system_fonts();
        ph2d_editor_core::interaction::dispatch_pointer_with_text(
            &mut self.store,
            &self.hit_index,
            event,
            Some(&mut ts),
            &arena,
        )
        .to_vec()
    }

    /// Sugar over [`Self::paint`] for the common assertion: *is this widget on
    /// screen and clickable?* A widget painted with a degenerate (zero-area)
    /// rect is NOT clickable, so it does not count as painted.
    pub fn painted_rect<P: Panel>(
        &mut self,
        state: &mut P::State,
        viewport: Rect,
        id: NodeId,
    ) -> Option<Rect> {
        self.paint::<P>(state, viewport)
            .into_iter()
            .find(|(w, r)| *w == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| r)
    }

    /// **Drive a REAL pointer click at `(x, y)`** — Down then Up — through the same
    /// [`dispatch_pointer`] the shell runs, over the hit index the last [`Self::paint`] built.
    /// Returns the `WidgetEvent`s the dispatcher emitted (feed them to `apply_panel_event`).
    ///
    /// ## Why this exists (the last hole in the "green-but-dead" family)
    ///
    /// [`Self::paint`] proves a widget REGISTERS A HIT RECT and [`Self::apply_panel_event`]
    /// proves the panel FORWARDS an event it is handed. Neither proves a POINTER on that rect
    /// ever becomes that event — and it does not, unless the id also carries an
    /// `InteractiveState` in the store: `dispatch_pointer`'s Down only makes a hit `active`
    /// when it is *focusable*, and an id absent from the store is not. So a widget can paint,
    /// hit-register, forward and route — every gate green — and still be **stone dead under
    /// the mouse** because `populate` never registered it. That is not a hypothetical: it is
    /// the Impasto light rig (Enio 2026-07-12, *"nem o checkbox nem se pode selecionar outra
    /// luz"*), and before it the hierarchy companions.
    ///
    /// A widget is not done when it paints. It is done when a test CLICKS it.
    pub fn click_at(&mut self, x: f32, y: f32) -> Vec<WidgetEvent> {
        let mut out = Vec::new();
        for kind in [PointerKind::Down, PointerKind::Up] {
            // Space the clicks a full second apart: inside the double-click window the
            // dispatcher would upgrade the second Click to a DoubleClick and the assertion
            // would fail for a reason that has nothing to do with the seam under test.
            self.clock_ns += NANOS_PER_SECOND;
            let arena = Bump::new();
            let event = PointerEvent {
                x,
                y,
                pressure: 1.0,
                kind,
                source: PointerSource::Mouse,
                button: PointerButton::Primary,
                timestamp_ns: self.clock_ns,
            };
            out.extend_from_slice(dispatch_pointer(
                &mut self.store,
                &self.hit_index,
                event,
                &arena,
            ));
        }
        out
    }

    /// The id a pointer at `(x, y)` actually LANDS ON — the dispatcher's own resolution
    /// (topmost = last registered wins). When [`Self::click_at`] emits nothing, this says
    /// whether the widget lost the hit to something painted over it, or was never hit at all.
    pub fn hit_at(&self, x: f32, y: f32) -> Option<NodeId> {
        self.hit_index.hit(x, y)
    }
}

impl Default for MockPanelHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PanelHost for MockPanelHost {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn project(&self) -> &ProjectSettings {
        &self.project
    }
}

impl PanelHostInternal for MockPanelHost {
    fn store(&self) -> &WidgetStore {
        &self.store
    }

    fn store_mut(&mut self) -> &mut WidgetStore {
        &mut self.store
    }

    fn hit_index_mut(&mut self) -> &mut HitIndex {
        &mut self.hit_index
    }

    fn store_and_hit_index_mut(&mut self) -> (&WidgetStore, &mut HitIndex) {
        (&self.store, &mut self.hit_index)
    }

    fn bus(&self) -> &ActionBus {
        &self.bus
    }

    fn bus_mut(&mut self) -> &mut ActionBus {
        &mut self.bus
    }

    fn selection(&self) -> Option<&HeroSelection> {
        self.selection.as_ref()
    }

    fn selection_mut(&mut self) -> &mut Option<HeroSelection> {
        &mut self.selection
    }

    fn panel_visible(&self, id: &str) -> bool {
        self.visible.get(id).copied().unwrap_or(false)
    }

    fn set_panel_visible(&mut self, id: &str, value: bool) {
        self.visible.insert(id.to_string(), value);
    }

    fn grid_snap_state(&self) -> &GridSnapState {
        &self.grid_snap
    }

    fn grid_snap_state_mut(&mut self) -> &mut GridSnapState {
        &mut self.grid_snap
    }

    fn store_and_grid_snap_state_mut(&mut self) -> (&WidgetStore, &mut GridSnapState) {
        (&self.store, &mut self.grid_snap)
    }

    fn grid_snap_panel_rect(&self) -> Option<Rect> {
        self.grid_snap_panel_rect
    }

    fn set_grid_snap_panel_rect(&mut self, rect: Option<Rect>) {
        self.grid_snap_panel_rect = rect;
    }
}
