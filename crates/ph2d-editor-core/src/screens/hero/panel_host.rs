//! ⭐ **O HERO COMO HOSPEDEIRO DE PAINÉIS** — as duas travessias que um painel usa para chegar ao
//! que ele não possui.
//!
//! ⚠️ **Cortado do `hero.rs` em 2026-08-30 pelo tecto de LOC (701/700), e o corte é por
//! RESPONSABILIDADE:** aquele ficheiro é *o que o hero É* (o estado, os módulos, o `apply_event`) e
//! isto é *o que ele EMPRESTA*. Ele é uma superfície que só cresce com a migração dos painéis
//! (ADR-0029), e crescer ali empurrava o tecto de quem não tinha nada a ver com painéis.

use super::{HeroScreen, HeroSelection, panel_ids};
use crate::interaction::{HitIndex, WidgetStore};
use ph2d_tokens::Theme;

/// ADR-0029 Phase B.3 — `PanelHostInternal` is the
/// `#[doc(hidden)] pub` trait surface that the four in-tree panels
/// consume in Phase C. The initial impl exposes only the minimal
/// foundation (theme + project + widget store + hit index); the
/// remaining ~25-30 accessors (selection, gizmo, grid, view, …)
/// land alongside each panel's migration in Phase C as they're
/// actually needed.
impl crate::panel::PanelHost for HeroScreen {
    fn theme(&self) -> Theme {
        self.theme
    }

    fn project(&self) -> &crate::project::ProjectSettings {
        &self.project
    }
}

impl crate::panel::PanelHostInternal for HeroScreen {
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

    fn bus(&self) -> &crate::action_bus::ActionBus {
        &self.bus
    }

    fn bus_mut(&mut self) -> &mut crate::action_bus::ActionBus {
        &mut self.bus
    }

    fn selection(&self) -> Option<&HeroSelection> {
        self.selection.as_ref()
    }

    fn selection_mut(&mut self) -> &mut Option<HeroSelection> {
        &mut self.selection
    }

    fn panel_visible(&self, id: &str) -> bool {
        self.is_panel_visible(id)
    }

    fn set_panel_visible(&mut self, id: &str, value: bool) {
        // Use the canonical interned id when one matches a known
        // panel so the HashMap lookup is keyed by `&'static str`.
        let key = panel_ids::canonical_panel_id(id).unwrap_or_else(|| {
            // Fall back to leaking — unknown panels are rare (3rd
            // party / future migrations); a single allocation per
            // unique id is acceptable for the unstable internal tier.
            Box::leak(id.to_string().into_boxed_str()) as &'static str
        });
        self.panel_visibility.insert(key, value);
    }

    fn grid_snap_state(&self) -> &crate::grid_snap::GridSnapState {
        &self.grid.snap_state
    }

    fn grid_snap_state_mut(&mut self) -> &mut crate::grid_snap::GridSnapState {
        &mut self.grid.snap_state
    }

    fn store_and_grid_snap_state_mut(
        &mut self,
    ) -> (&WidgetStore, &mut crate::grid_snap::GridSnapState) {
        (&self.store, &mut self.grid.snap_state)
    }

    fn grid_snap_panel_rect(&self) -> Option<crate::zones::Rect> {
        self.grid.snap_state.panel_rect
    }

    fn set_grid_snap_panel_rect(&mut self, rect: Option<crate::zones::Rect>) {
        self.grid.snap_state.panel_rect = rect;
    }
}
