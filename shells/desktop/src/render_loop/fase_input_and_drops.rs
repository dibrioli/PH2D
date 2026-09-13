//! **Fase do quadro: A ENTRADA** — o carimbo coalescido do Painter, os contadores de diagnóstico, o
//! relógio do encode, o gamepad, o script e os ficheiros largados (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Devolve `cpu_start`**, e é a medição que manda: o relógio do encode ARMA aqui, depois do
//! carimbo coalescido e antes de tudo o que o quadro codifica — o `run_present_phase` lê-o no fim.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo. Devolve `(cpu_start, eventos de entrada, carimbos)` deste quadro.
    pub(super) fn fase_input_and_drops(&mut self) -> (Instant, u32, u32) {
        // Coalesced painter Move: stamp the LATEST buffered canvas position ONCE this frame, replacing
        // the per-raw-CursorMoved whole-shape re-stamp storm that ran between frames (the FPS-drop /
        // "Raw rises" path — `HANDOFF_per_layer_color_perf_artifacts` §1.R). Done before `cpu_start` so
        // the re-stamp stays OUT of the encode window and "Raw" keeps its encode-only meaning.
        self.flush_pending_painter_move();
        // Snapshot + reset the per-frame input/stamp diagnostics for the HUD (input rate vs delivered
        // re-stamps — coalescing collapses a burst of events to one stamp here). `paint_stamp_us`
        // accumulates BOTH the coalesced flush (just now) and any incremental per-event stamps since the
        // last frame, so "paint ms" is the real per-frame painter cost (not just the flush).
        let diag_input_events = std::mem::take(&mut self.input_events_this_frame);
        let diag_paint_stamps = std::mem::take(&mut self.paint_stamps_this_frame);
        self.last_paint_stamps = diag_paint_stamps;
        self.last_paint_stamp_us = std::mem::take(&mut self.paint_stamp_us_this_frame);
        // M14.7 polish (10.1): tag the start of CPU work for the
        // raw-fps measurement. Stopped after `queue.submit` (before
        // the present blocks on vsync) so the EWMA tracks pure
        // hardware capacity, independent of refresh rate.
        let cpu_start = Instant::now();
        // Pump gamepad events first so InputState reflects the latest
        // state by the time sim/extract run. Order: input → script
        // input snapshot → sim → extract → render.
        self.pump_gamepad();
        self.push_input_to_script();

        // M14.7 polish (7.3 fix): drain `pending_drops` atomically
        // BEFORE the render walks PresentWorld. Each path imports
        // exactly once, so a batch drop of N files always produces
        // exactly N sprites (winit's per-event timing no longer
        // matters). Clear the hover overlay here too — the gesture
        // is over the moment the first DroppedFile arrived.
        if !self.pending_drops.is_empty() {
            let paths = std::mem::take(&mut self.pending_drops);
            self.hovered_files.clear();
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                hero.dragging_files = None;
            }
            self.handler.on_file_drop(&paths);
            self.handle_dropped_files(&paths);
        }
        (cpu_start, diag_input_events, diag_paint_stamps)
    }
}
