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

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_editor_core::grid_snap::GridSnapState;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHost, PanelHostInternal};
use ph2d_editor_core::project::ProjectSettings;
use ph2d_editor_core::screens::HeroSelection;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;

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

    /// Drain everything the panel pushed onto the action bus so far. The
    /// shell does the same each frame; tests inspect the result to assert
    /// the panel actually emitted the right [`EditorAction`].
    pub fn drained_actions(&mut self) -> Vec<EditorAction> {
        self.bus.drain().collect()
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
