//! Runtime panel registry — `OnceLock` of `Vec<ErasedPanel>`,
//! installed at boot before the first `HeroScreen::new`.
//!
//! ADR-0029 §4.6 + Wave 8 Phase 1 (audit B1/S2/S3 closure). Compared
//! to the old `PANEL_REGISTRY` of `&'static PanelManifest`s, this
//! version owns each `ErasedPanel` because the new design has
//! per-panel state living inside the registry (not flat on
//! HeroScreen). Statics no longer suffice; the registry holds Box
//! state.

use super::ErasedPanel;
use ph2d_a11y::NodeId;
use std::sync::{Mutex, OnceLock};

const REGISTRY_NOT_INSTALLED_MSG: &str = "\
PANEL_REGISTRY not installed. Host must call \
`ph2d_panel_registry_init::register_all_panels()` (or build a \
`PanelRegistry` manually) before the first `HeroScreen::new`. \
Tests should call `ph2d_editor::test_support::ensure_panel_registry()`.";

/// Append-only registry built once at boot.
pub struct PanelRegistry {
    panels: Vec<ErasedPanel>,
}

impl PanelRegistry {
    pub fn new_empty() -> Self {
        Self { panels: Vec::new() }
    }

    pub fn push(&mut self, panel: ErasedPanel) {
        self.panels.push(panel);
    }

    pub fn panels(&self) -> &[ErasedPanel] {
        &self.panels
    }

    pub fn panels_mut(&mut self) -> &mut [ErasedPanel] {
        &mut self.panels
    }

    /// Look up the panel whose `manifest.panel_node_id == id`.
    pub fn find_by_panel_node_id(&self, id: NodeId) -> Option<usize> {
        self.panels
            .iter()
            .position(|p| p.manifest.panel_node_id == id)
    }
}

/// Process-wide registry. Wrapped in `Mutex` because panels mutate
/// their owned state on every paint/event; concurrent access from
/// multiple frames would be a race.
///
/// In practice only one paint thread accesses this, so the mutex
/// is uncontended. The wrapping exists for soundness, not perf.
pub static PANEL_REGISTRY: OnceLock<Mutex<PanelRegistry>> = OnceLock::new();

/// Install the process-wide registry. Idempotent — returns `false`
/// if a registry was already installed.
pub fn install_panel_registry(reg: PanelRegistry) -> bool {
    PANEL_REGISTRY.set(Mutex::new(reg)).is_ok()
}

/// Run `f` with the locked registry. Panics if no registry installed.
pub fn with_registry<R>(f: impl FnOnce(&mut PanelRegistry) -> R) -> R {
    let mtx = PANEL_REGISTRY.get().expect(REGISTRY_NOT_INSTALLED_MSG);
    let mut guard = mtx.lock().expect("PANEL_REGISTRY mutex poisoned");
    f(&mut guard)
}

/// Lenient variant — returns `None` if no registry has been
/// installed. The orchestrator uses this so editor-core tests +
/// `--no-default-features` builds without typed panels render
/// the legacy registry without panicking.
pub fn with_registry_opt<R>(f: impl FnOnce(&mut PanelRegistry) -> R) -> Option<R> {
    let mtx = PANEL_REGISTRY.get()?;
    let mut guard = mtx.lock().expect("PANEL_REGISTRY mutex poisoned");
    Some(f(&mut guard))
}

/// Convenience: read-only access.
pub fn with_registry_ref<R>(f: impl FnOnce(&PanelRegistry) -> R) -> R {
    with_registry(|r| f(r))
}
