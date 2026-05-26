//! Paint+present phase: 4 GPU passes (sprite → tonemap → vello → compositor)
//! + window title refresh + request_redraw.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a sibling
//! method on `App` via split impl. Behavior-preserving lift. Caller in
//! mod.rs invokes via `self.run_present_phase(cpu_start, r, g, b)` after
//! the editor-mode / M5-demo paint branch finishes encoding the
//! `VectorScene`.

use crate::{AppGfx, SPRITE_COUNT};
use ph2d_gpu::AcquireError;
use ph2d_host::PlatformHost;
use ph2d_vector::Color as VelloColor;
use std::time::Instant;

impl crate::App {
    /// Acquires the swap-chain frame, encodes + submits the 4-pass
    /// pipeline, advances `frame_cpu_ms_ewma`, refreshes the window
    /// title when dirty, requests the next redraw.
    ///
    /// `cpu_start` is the [`Instant`] captured at the top of
    /// `run_render_frame` — used to bound the raw-fps measurement
    /// across the encode work (excluding the vsync-blocking
    /// `acquire_frame` call).
    pub(super) fn run_present_phase(&mut self, cpu_start: Instant, r: f64, g: f64, b: f64) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(host) = self.host.as_ref() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            present,
            camera,
            asset_db,
            atlas_is_real,
            script,
            theme,
            zen,
            toasts,
            tools,
            game_rt,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            ..
        } = gfx;
        let window_size = surface.size();

        // M14.7 polish (10.1 fix): `surface.acquire_frame()` can block
        // until the next swap-chain texture is ready. Under a vsync
        // present mode that wait IS the refresh interval (~16.7 ms at
        // 60 Hz); including it in the raw-fps measurement caps the
        // reading at the refresh rate, which is exactly what we DON'T
        // want ("Unity shows 2000 fps"). Pause the clock around the
        // acquire, then resume for the actual encode + submit work.
        //
        // Present mode default is VSync (`Fifo`, set in
        // `ph2d-gpu/src/surface.rs`) for smooth motion; under it this
        // acquire blocks ~1 refresh and, when the continuously-animating
        // demo scene saturates the queue, can stall longer = the
        // mouse-move stutter. The user opts into the non-blocking
        // `Immediate` mode via Config → Display to kill that stall.
        let work_before_acquire = cpu_start.elapsed();
        match surface.acquire_frame() {
            Ok(frame) => {
                let after_acquire = Instant::now();
                // M14.5 — viewport / RT pipeline. Four GPU submissions
                // each frame, all independent.
                //
                // Pass 1: sprite (+ future light/particle/material)
                //   target: `game_rt` (Rgba16Float HDR offscreen)
                //   ↳ clear color is opaque so the canvas reads as a
                //   single tinted surface beneath sprites + grid.
                renderer.render(
                    game_rt.view(),
                    present,
                    camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                );
                // Pass 2: AgX tonemap
                //   target: `tonemap.output_view()` (Bgra8UnormSrgb LDR)
                tonemap.run(surface.gpu());
                // Pass 3: Vello chrome
                //   target: `vello_pass.intermediate_view()`
                //   ↳ TRANSPARENT clear so any pixel the editor scene
                //   doesn't paint stays α=0 and the compositor reveals
                //   `game_rt_ldr` through it.
                //
                // AA selection: the current TextRendering preset's
                // `prefer_msaa` flag drives whether Vello uses MSAA16
                // (smoother glyph edges; CrispHeavyPlus) or its
                // default Area analytical coverage (Default + CrispHeavy).
                let prefer_msaa = ph2d_editor::paint::text_rendering().params().prefer_msaa;
                if let Err(e) = vello_pass.render_to_intermediate(
                    surface.gpu(),
                    vector_scene.inner(),
                    (window_size.width, window_size.height),
                    VelloColor::TRANSPARENT,
                    prefer_msaa,
                ) {
                    eprintln!("M14.5 vello_pass.render_to_intermediate error: {e}");
                }
                // Pass 4: compositor
                //   reads: tonemap output + vello intermediate
                //   target: swap chain
                compositor.run(surface.gpu(), frame.view());
                // FrameTarget presents on Drop.
                let work_after_acquire = after_acquire.elapsed();
                let cpu_total = work_before_acquire + work_after_acquire;
                let cpu_ms_now = cpu_total.as_secs_f64() * 1000.0;
                const ALPHA_CPU: f32 = 0.1;
                self.frame_cpu_ms_ewma =
                    ALPHA_CPU * (cpu_ms_now as f32) + (1.0 - ALPHA_CPU) * self.frame_cpu_ms_ewma;
            }
            Err(AcquireError::AwaitingReconfigure) => {
                surface.reconfigure_after_lost();
            }
            Err(AcquireError::Occluded) => {}
            Err(AcquireError::Timeout) => {}
            Err(AcquireError::Other(s)) => {
                eprintln!("acquire_frame other error: {s}");
            }
        }

        // Window title carries editor state. Refresh only when state
        // actually changes — winit set_title triggers a platform call.
        if self.title_dirty {
            let tool_label = tools.active().map(|t| t.label()).unwrap_or("none");
            let title = format!(
                "PH2D — M5+M6+M7+M11+M12 demo | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                if *atlas_is_real { "PNG" } else { "dummy" },
                asset_db.len_assets(),
                if script.is_some() { "ok" } else { "off" },
                theme,
                if zen.is_active() { "on" } else { "off" },
                toasts.len(),
                tool_label,
            );
            host.window().set_title(&title);
            self.title_dirty = false;
        }

        // Continuous redraw (paired with `ControlFlow::Poll` in main.rs):
        // the frame is rebuilt every loop iteration regardless of input,
        // so any per-frame cost shows as ~100% idle CPU and, if a frame
        // gets heavy, mouse-move stutter (worst over the Hierarchy panel,
        // which has the most per-frame text).
        //
        // IF MOUSE STUTTER RETURNS, look here first:
        //  1. Per-frame text shaping — mitigated by the shaped-layout
        //     cache in `ph2d-text/src/system.rs` (`layout_cache`). A new
        //     uncached text path, or text that changes every frame and
        //     thrashes the cache, re-introduces the cost. Profile with a
        //     `PH2D_PROF`-style timer around `paint_hero_screen`.
        //  2. The continuous redraw + present saturation — the
        //     user-facing fix is Config → Display → Immediate (a
        //     non-blocking present mode, see `ph2d-gpu/src/surface.rs`
        //     `set_present_mode`), which stops `acquire_frame` stalling.
        //     Default is VSync (`Fifo`) for smooth motion. The deeper
        //     idle-CPU win (event-driven `ControlFlow::Wait`) stays
        //     deferred and only pays off once the scene is static (the
        //     M5 demo bouncing-motion sim animates every frame, so the
        //     loop is continuous regardless).
        host.request_redraw();
    }
}
