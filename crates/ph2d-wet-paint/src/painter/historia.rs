//! O **desfazer/refazer por camada** (SPEC §15) — filho de [`super`] por LOC.

use super::*;

impl Engine {
    // -----------------------------------------------------------------------
    // History (SPEC §15): per-layer undo/redo ring, depth 6.
    // -----------------------------------------------------------------------

    pub fn capture_history(&mut self) {
        self.drain_step();
        let layer = &mut self.layers[self.active_layer];
        layer.undo.push(snapshot_grid(&layer.grid));
        if layer.undo.len() > HISTORY_DEPTH {
            layer.undo.remove(0);
        }
        layer.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.layers[self.active_layer].undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.layers[self.active_layer].redo.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        self.drain_step();
        let layer = &mut self.layers[self.active_layer];
        let Some(snap) = layer.undo.pop() else {
            return false;
        };
        layer.redo.push(snapshot_grid(&layer.grid));
        self.apply_snapshot(snap);
        true
    }

    pub fn redo(&mut self) -> bool {
        let layer = &mut self.layers[self.active_layer];
        let Some(snap) = layer.redo.pop() else {
            return false;
        };
        layer.undo.push(snapshot_grid(&layer.grid));
        self.apply_snapshot(snap);
        true
    }

    fn apply_snapshot(&mut self, snap: GridSnapshot) {
        let layer = &mut self.layers[self.active_layer];
        let paper_changed = restore_grid(&mut layer.grid, &snap);
        if paper_changed {
            self.paper_preset = layer.grid.paper_preset;
            self.paper_sheet = layer.grid.paper_sheet;
            self.rebake_paper();
        }
        self.mark_dirty_full();
    }
}
