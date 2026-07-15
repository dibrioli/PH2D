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
        // ADR-0114 W2: GPU-data do traço Flip em curso (preview ao vivo), construída
        // ANTES do borrow de `self.gfx` (lê a câmera + o gesto; devolve owned).
        // ADR-0114 C2: no modo Colorize o "em curso" são os RABISCOS acumulados — mesmo
        // slot, porque Draw e Colorize são modos distintos e nunca coexistem.
        let flip_preview = self
            .flip_preview_data()
            .or_else(|| self.flip_colorize_preview_data());
        // Os fantasmas do onion, cozidos no bloco de overlay deste frame (ADR-0142).
        // Retirados ANTES do borrow de `self.gfx`; concatenados ao slot `extra` do passe
        // de sprite abaixo. Vazio quando o onion está desligado.
        let onion_ghosts = std::mem::take(&mut self.onion_ghosts);
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
            motion_fx,
            tonemap,
            compositor,
            vello_pass,
            vector_scene,
            // Motion Nodes M0.T10/T11: the cooked stream, injected into the sprite
            // pass below when the `motion` tool is active (the bridge pumped it
            // into `motion.pump.instances` earlier this frame).
            motion,
            // Motion Nodes M0.T13: read the center split to frame the scene into
            // its sub-rect (set by the bridge earlier this frame).
            hero_screen,
            // ADR-0114 W1: a cena Flip + o rasterizador do traço + a composição
            // por-camada (compositor 22-modos), tudo no passe 1b abaixo.
            flip,
            flip_render,
            flip_compose,
            flip_composite,
            // ADR-0111: o `Transform` de cada objeto Flip vem do ECS; o passe o dobra
            // no `world_to_clip` (a arte é LOCAL). Identidade = sem custo.
            sim,
            ..
        } = gfx;
        let window_size = surface.size();
        let motion_active = tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor::ToolId::new("motion"));

        // Motion Nodes M0.T13 — Fase B: when the center is split (Motion mode),
        // frame the scene into its sub-rect (top for a horizontal Cavalry split,
        // left for a vertical one) in RENDER-TARGET pixels — the SAME fraction
        // the graph panel uses, so scene and graph align at any DPI without
        // plumbing the editor-core layout rect.
        // PORTA ÚNICA (`CenterSplit::scene_viewport`): o MESMO sub-retângulo que o chrome
        // (a grade do mundo + o gizmo de field, em `snapshots`/`field_gizmo`) usa para
        // mapear mundo↔tela — senão a cena e o chrome discordam sobre onde um ponto de
        // mundo cai (o drift crônico do Motion). O split só é != None na tool Motion, mas
        // o gate em `motion_active` fica por robustez.
        let scene_viewport: Option<[f32; 4]> = motion_active
            .then(|| {
                hero_screen.as_ref().and_then(|hs| {
                    hs.view
                        .center_split
                        .scene_viewport(window_size.width as f32, window_size.height as f32)
                })
            })
            .flatten();

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
                // Motion Nodes M0.T11: append the cooked node-graph stream to the
                // sprite pass (empty when the tool is inactive) — drawn without
                // being spawned into the ECS `present` (stream ≠ ECS, ADR-0035).
                // GPU/M5 Fase 1 (ADR-0126): when the bridge cooked this frame on
                // the GPU (`PH2D_GPU_COOK=1`, fully-covered chain), the instance
                // buffer to draw ALREADY lives on the GPU — bind it directly and
                // pass an empty CPU slice (the pump never ran; its buffer is
                // stale). Otherwise the classic CPU slice path, byte-identical.
                let motion_gpu: Option<(&wgpu::Buffer, u32)> = (motion_active && motion.gpu_live)
                    .then(|| motion.gpu_cook.instances())
                    .flatten()
                    .map(|gi| (gi.buffer(), gi.len()));
                let motion_slice: &[ph2d_render::RenderInstance] =
                    if motion_active && motion_gpu.is_none() {
                        &motion.pump.instances
                    } else {
                        &[]
                    };
                // O slot `extra` do passe carrega DOIS produtores CPU: os fantasmas do
                // onion (ADR-0142) + o stream do Motion. Concatenados num só slice; os dois
                // raramente coexistem, então o `Vec` é vazio no caso comum.
                let sprite_extra: Vec<ph2d_render::RenderInstance> = if onion_ghosts.is_empty() {
                    // Sem fantasmas: passa o slice do Motion direto (zero alloc no caso comum).
                    Vec::new()
                } else {
                    onion_ghosts.iter().chain(motion_slice).copied().collect()
                };
                let extra: &[ph2d_render::RenderInstance] = if sprite_extra.is_empty() {
                    motion_slice
                } else {
                    &sprite_extra
                };
                renderer.render_with_streams(
                    game_rt.view(),
                    present,
                    camera,
                    window_size,
                    wgpu::Color { r, g, b, a: 1.0 },
                    extra,
                    motion_gpu,
                    scene_viewport,
                );
                // Pass 1b: Flip (ADR-0114 W1) composto por-camada (blend/opacity via
                //   compositor 22-modos) no `game_rt`, amostrado pelo playhead,
                //   MESMA câmera dos sprites. O blit final usa LoadOp::Load (preserva
                //   os sprites por baixo). No-op sem camada Flip ativa (default).
                let flip_models = crate::flip_transform::build(sim, &self.flip_entities);
                // Ghost Frames só existem enquanto a tool Flip está no comando (é
                // chrome de autoria, não da cena) — e só fora do play.
                // O PEEK (F1/F2/F3 presos): uma folha vizinha na mão — os fantasmas
                // somem JUNTO (a folha na mão não é uma pilha translúcida) e não há
                // peek no play (o relógio já está folheando por conta própria).
                let peek = if self.flip_active && !self.playhead.is_playing() {
                    self.flip_peek
                } else {
                    None
                };
                let ghost_selection = (self.flip_active && peek.is_none()).then(|| {
                    super::flip_pass_ghosts::GhostSources {
                        selected: self.flip_strip.selected_keys(),
                        pinned: self.flip_strip.pinned_keys(),
                        trace: Some(&self.flip_strip.trace),
                    }
                });
                super::flip_pass::render(
                    flip,
                    flip_render,
                    flip_compose,
                    flip_composite,
                    flip_preview.as_ref(),
                    self.flip_active_layer,
                    &flip_models,
                    &self.playhead,
                    ghost_selection,
                    peek,
                    game_rt,
                    camera,
                    window_size,
                    surface.gpu(),
                );
                // Pass 1c: Motion glow (doc 67, Option B) — the Motion module's
                //   OWN HDR effect, authored as an `fx.glow` node in the graph.
                //   Only runs when the artist has dropped that node (and dialed
                //   intensity > 0); otherwise this whole block is skipped and the
                //   frame is byte-identical (the fused sprite+Motion pass and the
                //   tonemap are untouched → blast radius zero). Re-render the
                //   Motion instances IN ISOLATION into motion_fx's own Rgba16Float
                //   RT, bright-pass + blur them, and ADD the glow over game_rt
                //   (before the tonemap, so the glow tonemaps with everything
                //   else). Additive = emitted light, so it bleeds over whatever is
                //   in front — the sparks look lit, not pasted.
                let glow = ph2d_node_fx_glow::from_graph(&motion.doc.graph);
                if let Some(glow) = glow
                    && motion_active
                    && glow.intensity > 0.0
                    && !motion.pump.instances.is_empty()
                {
                    renderer.render_instances_only(
                        motion_fx.rt_view(),
                        camera,
                        window_size,
                        wgpu::Color::TRANSPARENT,
                        &motion.pump.instances,
                        // SAME sub-rect the fused scene used above — or the glow
                        // desyncs from the sparks (the halo floats away).
                        scene_viewport,
                    );
                    motion_fx.bloom_over(
                        surface.gpu(),
                        game_rt.view(),
                        &ph2d_render::BloomParams {
                            threshold: glow.threshold,
                            knee: glow.knee,
                            intensity: glow.intensity,
                            radius: glow.radius,
                        },
                    );
                }
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
                // GPU pass profiler: Vello submits internally (its passes are out of
                // reach), so bracket the whole call with marker submits — queue order
                // makes `end − begin` cover everything Vello enqueued. No-op when off.
                let vello_span = {
                    let g = surface.gpu();
                    ph2d_gpu::pass_profiler::span_begin(&g.device, &g.queue, "render.vello")
                };
                if let Err(e) = vello_pass.render_to_intermediate(
                    surface.gpu(),
                    vector_scene.inner(),
                    (window_size.width, window_size.height),
                    VelloColor::TRANSPARENT,
                    prefer_msaa,
                ) {
                    eprintln!("M14.5 vello_pass.render_to_intermediate error: {e}");
                }
                if let Some(t) = vello_span {
                    let g = surface.gpu();
                    ph2d_gpu::pass_profiler::span_end(&g.device, &g.queue, t);
                }
                // Pass 4: compositor
                //   reads: tonemap output + vello intermediate
                //   target: swap chain
                compositor.run(surface.gpu(), frame.view());
                // GPU pass profiler frame tail: resolve this frame's timestamp
                // queries + kick the pipelined readback (prints every 120 frames).
                // After the LAST instrumented submit; the resolve submit ordering
                // vs the present is irrelevant (same queue). No-op when off.
                {
                    let g = surface.gpu();
                    ph2d_gpu::pass_profiler::end_frame(&g.device, &g.queue);
                }
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
