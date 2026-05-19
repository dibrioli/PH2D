//! Wave 5 stage C — `PanelManifest` + `PanelRegistry`, the panel-side
//! mirror of `ph2d-tool-registry`'s `ToolManifest` + `Registry`.
//!
//! Each panel (Inspector, Hierarchy, Widget Gallery, Grid Snap) owns
//! a `pub static PANEL_MANIFEST: PanelManifest` const declaring its
//! id, NodeId, default visibility, and three fn pointers:
//!
//! - `paint_fn(ctx: &mut PaintCtx)` — full per-frame paint logic
//!   (visibility check + drag/resize clamp + chrome publish + actual
//!   paint + content_h publish + clear-on-hide).
//! - `apply_event_fn(hero: &mut HeroScreen, ev: WidgetEvent) -> bool`
//!   — routes panel-specific events. Returns `true` when the event
//!   was consumed.
//! - `populate_fn(store: &mut WidgetStore)` — pre-registration of
//!   the panel's widget NodeIds at HeroScreen construction time.
//!
//! `paint_hero_screen` collapses to a single iteration over
//! [`PANEL_REGISTRY`] in z-order, with `paint_fn` doing all per-panel
//! work. Future Wave 6 can lift each panel to `ph2d-panel-<slug>/`
//! crates with zero changes to this registry shape.

use ph2d_a11y::NodeId;

use crate::interaction::{WidgetEvent, WidgetStore};
use crate::screens::hero::{HeroLayout, HeroScreen};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// Per-frame context handed to every panel's `paint_fn`. Holds the
/// mutable refs and immutable layout/viewport the panel needs to
/// run its full frame logic without further state lookups.
///
/// Borrow note: `hero`, `scene`, and `text_system` are independent
/// `&mut`s; Rust's field-level borrow splitting handles disjoint use
/// within a single thunk (`ctx.hero.foo(); ctx.scene.bar();` is fine).
pub struct PaintCtx<'a> {
    /// HeroScreen owns panel visibility flags + WidgetStore + hit
    /// index. The thunk reads `hero.<panel>.visible` (early-return
    /// when hidden) and publishes content/visible heights via
    /// `hero.store.set_panel_content_h(...)`.
    pub hero: &'a mut HeroScreen,
    /// Pre-computed sub-region rects (INSP, HIER clamps already
    /// applied). Floating panels (Widget Gallery, Grid Snap) compute
    /// their own rects locally.
    pub layout: &'a HeroLayout,
    /// Outer viewport — needed by floating-panel clamping math.
    pub viewport: Rect,
    pub scene: &'a mut VectorScene,
    pub text_system: &'a mut TextSystem,
}

/// Paint function pointer — full per-frame logic for one panel.
/// Includes the visibility early-return so the registry iteration
/// in `paint_hero_screen` doesn't need to know which `HeroScreen`
/// field gates each panel.
pub type PaintFn = for<'a> fn(&mut PaintCtx<'a>);

/// Outcome of a panel's [`ApplyEventFn`] call. Wave 8 Phase 4
/// replaced the previous `-> bool` (Consumed / Ignored) with a
/// tripartite enum so the dispatcher can distinguish:
///
/// - **Consumed**: the panel handled the event exclusively. Stop
///   iterating; no other panel needs to see it.
/// - **Ignored**: the panel did nothing. Continue to the next.
/// - **Observed**: the panel did a side-effect (state mutation,
///   buffer commit, etc.) but doesn't claim exclusivity. Continue
///   iterating; other panels may also observe.
///
/// `Observed` exists because of the audit B2 pattern: hierarchy's
/// `Blur(HIER_RENAME_INPUT)` stages a `HierRenameCommit` AND
/// returns false (so the chrome fallthrough still runs). The old
/// `-> bool` contract said "false = nothing happened" but the side
/// effect was loud. Tripartite makes the intent explicit + impossible
/// to introduce by accident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Consumed,
    Ignored,
    Observed,
}

impl EventOutcome {
    /// Migration helper: convert a legacy `-> bool` (true = Consumed,
    /// false = Ignored). Used by panels that still bool-return and
    /// don't have side-effects-without-consumption. The hierarchy
    /// Blur case returns `Observed` directly without going through
    /// `from_bool`.
    pub fn from_bool(consumed: bool) -> Self {
        if consumed {
            EventOutcome::Consumed
        } else {
            EventOutcome::Ignored
        }
    }
}

/// Apply-event function pointer — routes a `WidgetEvent` to one
/// panel's apply logic. Returns an [`EventOutcome`] declaring whether
/// the event was consumed exclusively, observed (side-effect, no
/// exclusivity), or ignored.
pub type ApplyEventFn = fn(&mut HeroScreen, WidgetEvent) -> EventOutcome;

/// Populate function pointer — pre-registers a panel's widget
/// NodeIds against the WidgetStore at construction time. Matches
/// each panel module's existing `pub fn populate(&mut WidgetStore)`.
pub type PopulateFn = fn(&mut WidgetStore);

/// Static manifest declaring one panel's identity + behavior.
/// Mirrors `ph2d-tool-registry::ToolManifest` shape.
pub struct PanelManifest {
    /// Short stable id (used by future MCP exposure, logs, ADRs).
    pub id: &'static str,
    /// NodeId of the panel's outer rect — used by `paint_hero_screen`
    /// to look up the manifest by `z_order`'s `panel_id`. Matches
    /// the const NodeId each panel already publishes via
    /// `set_panel_rect(id, ...)` (e.g., `ids::INSP_PANEL`).
    pub panel_node_id: NodeId,
    /// Default `visible` value used by `HeroScreen::new`. Currently
    /// informational only — `HeroScreen::new` sets visibility per
    /// group struct (`InspectorState { visible: true }`, etc.).
    /// Future: registry iteration on construction could drive this.
    pub default_visible: bool,
    pub paint_fn: PaintFn,
    pub apply_event_fn: ApplyEventFn,
    pub populate_fn: PopulateFn,
}

/// Append-only registry of every panel manifest. Iterated by
/// `paint_hero_screen` in z-order; `find_by_panel_node_id` lets the
/// z_order walk look up the matching manifest.
pub struct PanelRegistry {
    manifests: Vec<&'static PanelManifest>,
}

impl PanelRegistry {
    /// Construct from a `Vec` so callers can aggregate panel manifests
    /// at runtime (Wave 7 Phase 3: panel crates push their
    /// `PANEL_MANIFEST` here via `ph2d-panel-registry-init`).
    pub const fn new_empty() -> Self {
        Self {
            manifests: Vec::new(),
        }
    }

    pub fn new(manifests: Vec<&'static PanelManifest>) -> Self {
        Self { manifests }
    }

    pub fn push(&mut self, manifest: &'static PanelManifest) {
        self.manifests.push(manifest);
    }

    pub fn manifests(&self) -> &[&'static PanelManifest] {
        &self.manifests
    }

    /// Look up the manifest whose `panel_node_id == id`. Returns
    /// `None` for NodeIds that don't match any registered panel
    /// (e.g., `INSP_BLENDER_PICKER` which is painted out-of-band).
    pub fn find_by_panel_node_id(&self, id: NodeId) -> Option<&'static PanelManifest> {
        self.manifests
            .iter()
            .copied()
            .find(|m| m.panel_node_id == id)
    }
}

/// Process-wide panel registry. Wave 7 Phase 3: converted from a
/// hardcoded `pub static` to a runtime-initializable `OnceLock` so
/// downstream init crates (`ph2d-panel-registry-init`) can wire the
/// list without ph2d-editor referencing panel crates (which would
/// create a cycle when panels live in their own crates and depend
/// on ph2d-editor for `HeroScreen`).
///
/// Default empty. Callers MUST invoke [`install_panel_registry`] at
/// boot (before the first paint frame). The bundled
/// [`default_panel_registry`] aggregates the 4 in-tree panels
/// (widget_gallery + hierarchy + inspector + grid_snap) for hosts
/// that don't want feature-gated panel selection.
pub static PANEL_REGISTRY: std::sync::OnceLock<PanelRegistry> = std::sync::OnceLock::new();

/// Install the process-wide panel registry. Returns `true` on first
/// install, `false` if a registry was already installed (so re-init
/// in test harnesses is silently dropped).
pub fn install_panel_registry(reg: PanelRegistry) -> bool {
    PANEL_REGISTRY.set(reg).is_ok()
}

/// Iterate the installed panel registry.
///
/// **Panics** if no registry has been installed yet. Wave 8 Phase 1
/// removed the silent empty-slice fallback (audit S3): an editor with
/// no panels was a configuration error that booted without warning.
/// Hosts must call `ph2d_panel_registry_init::register_all_panels()`
/// (or `install_panel_registry(default_panel_registry())`) before
/// the first `HeroScreen::new`. Tests use
/// [`crate::test_support::ensure_panel_registry`].
pub fn panels() -> &'static [&'static PanelManifest] {
    PANEL_REGISTRY
        .get()
        .map(|r| r.manifests())
        .expect(REGISTRY_NOT_INSTALLED_MSG)
}

/// Look up the manifest whose `panel_node_id == id` against the
/// installed registry. Returns `None` for ids that don't match a
/// registered panel.
///
/// **Panics** if no registry has been installed yet (see [`panels`]).
pub fn find_panel_by_node_id(id: NodeId) -> Option<&'static PanelManifest> {
    PANEL_REGISTRY
        .get()
        .expect(REGISTRY_NOT_INSTALLED_MSG)
        .find_by_panel_node_id(id)
}

const REGISTRY_NOT_INSTALLED_MSG: &str = "\
PANEL_REGISTRY not installed. Host must call \
`ph2d_panel_registry_init::register_all_panels()` (or \
`ph2d_editor::panel_registry::install_panel_registry(default_panel_registry())`) \
before the first `HeroScreen::new`. Tests should call \
`ph2d_editor::test_support::ensure_panel_registry()`.";

/// Bundled default registry — all 4 in-tree panels. Hosts that don't
/// want cargo-features per panel install this at boot:
///
/// ```ignore
/// ph2d_editor::panel_registry::install_panel_registry(
///     ph2d_editor::panel_registry::default_panel_registry(),
/// );
/// ```
pub fn default_panel_registry() -> PanelRegistry {
    // ADR-0029 Phase C.1: Inspector dropped from the legacy registry.
    // ADR-0029 Phase C.2: Hierarchy dropped from the legacy registry.
    // Both register into `crate::panel::PANEL_REGISTRY` via
    // `ph2d_panel_registry_init::register_all_panels()`. The 2
    // remaining panels keep their fn-pointer manifests until they
    // migrate in C.3-C.4.
    PanelRegistry::new(vec![
        &crate::screens::hero::widget_gallery::PANEL_MANIFEST,
        &crate::grid_snap::PANEL_MANIFEST,
    ])
}
