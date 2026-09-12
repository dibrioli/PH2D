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
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
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
            // ADR-0154 Fase 2 — o acumulador do mundo, a colagem de faixa, as cenas do documento
            // por faixa (codificadas no `run_render_frame`) e o modo do compositor.
            world_rt,
            band_blit,
            band_doc_scenes,
            compositor_reads_world,
            frame_order,
            // ⭐⭐⭐ **O VIDRO JATEADO do *Edit Prefab*** (2026-09-07) — o passe, as duas cenas que
            // ele separa (o mundo sem a receita · a receita sozinha) e o interruptor do quadro,
            // que a codificação lá em cima escreveu. Ver [`super::present_frost`].
            frost,
            frost_doc_scene,
            frost_front_scene,
            frosting,
            ..
        } = gfx;
        let frosting = *frosting;
        let window_size = surface.size();
        let motion_active = tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("motion"));

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
        // ⚠️ **A ESPERA DO ACQUIRE É MEDIDA, e antes desta linha ela não era.**
        // O `[frame]` publicava `present/acquire-stall` como `total - encode` —
        // uma SUBTRAÇÃO, não uma medição —, e o encode começa em `cpu_start`,
        // depois do `tool-tick` e do flush de carimbo. Então o "stall" continha
        // trabalho de CPU e se lia como espera de GPU: com `tick 3,31` de um
        // "stall" de 7,91, a espera real é ~4,6 e a CPU trabalha ~12 ms de um
        // quadro de 16,6 — não os 8,25 que a linha sugeria.
        let acquire_t0 = Instant::now();
        match surface.acquire_frame() {
            Ok(frame) => {
                let after_acquire = Instant::now();
                super::note_acquire_wait(after_acquire.duration_since(acquire_t0));
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
                // The device buffer PLUS its texture-run partition (this wave):
                // the runs let the renderer draw a `source.object` graph by
                // binding the object's texture per run — an EMPTY partition (a
                // non-object stream) is the legacy single atlas draw. Both are
                // `&self` reads of the same cook; no readback.
                let motion_gpu: Option<(&wgpu::Buffer, u32, &[ph2d_render::GpuTexRun])> =
                    (motion_active && motion.gpu_live)
                        .then(|| motion.gpu_cook.instances())
                        .flatten()
                        .map(|gi| (gi.buffer(), gi.len(), motion.gpu_cook.texture_runs()));
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
                // ⭐⭐⭐ **AS FAIXAS DE DESENHO** (ADR-0154 Fase 2) — a lei, os cinco passos e o
                // porquê de a ÚLTIMA faixa de sprites não se ter movido vivem no cabeçalho do
                // irmão `present_bands`. Sem intercalação nada disto corre e o quadro é
                // **byte-idêntico** ao de sempre.
                let plan = super::present_bands::plan_frame(frame_order);
                let banded = plan.banded;
                // ⭐⭐⭐ **QUEM SOBE PARA CIMA DO VIDRO** — as peças da receita aberta saem do
                // fundo (senão o borrão delas escapa por fora da silhueta, como um halo) e são
                // desenhadas depois, do outro lado do vidro. `None` sem receita aberta, e aí toda
                // linha abaixo é a de sempre.
                let held = frosting
                    .then(|| super::present_frost::lift(sim, present, &mut self.frost_instances));
                if banded {
                    super::present_bands::draw_lower_bands(
                        surface.gpu(),
                        &plan,
                        wgpu::Color { r, g, b, a: 1.0 },
                        super::present_bands::BandGear {
                            world_rt,
                            renderer,
                            game_rt,
                            present,
                            camera,
                            window_size,
                            scene_viewport,
                            tonemap,
                            band_blit,
                            vello_pass,
                            band_doc_scenes,
                            held_back: held.as_ref(),
                        },
                    );
                }
                renderer.render_with_streams(
                    game_rt.view(),
                    present,
                    camera,
                    window_size,
                    if banded {
                        // O fundo já está no acumulador; esta faixa só contribui os pixels dela.
                        wgpu::Color::TRANSPARENT
                    } else {
                        wgpu::Color { r, g, b, a: 1.0 }
                    },
                    extra,
                    motion_gpu,
                    scene_viewport,
                    plan.rank_window(),
                    held.as_ref(),
                );
                // ⚠️ **A SONDA DO DRIFT** (`PH2D_PAN_DIAG=1`, report do Enio de 2026-08-25) —
                // DEPOIS do passe, de propósito: o que ela tem de imprimir é o sub-retângulo
                // que ele APLICOU, e esse só existe depois de ele decidir (o `.filter` do
                // clip/máscara é por conteúdo do quadro).
                ph2d_pan_diag::frame(
                    camera,
                    window_size,
                    motion_active,
                    scene_viewport,
                    renderer.applied_subrect(),
                );
                // Pass 1b: Flip (ADR-0114 W1) composto por-camada (blend/opacity via
                //   compositor 22-modos) no `game_rt`, amostrado pelo playhead,
                //   MESMA câmera dos sprites. O blit final usa LoadOp::Load (preserva
                //   os sprites por baixo). No-op sem camada Flip ativa (default).
                let flip_models =
                    ph2d_flip_entities::transform::build(sim, &self.flip_state.entities);
                // Ghost Frames só existem enquanto a tool Flip está no comando (é
                // chrome de autoria, não da cena) — e só fora do play.
                // O PEEK (F1/F2/F3 presos): uma folha vizinha na mão — os fantasmas
                // somem JUNTO (a folha na mão não é uma pilha translúcida) e não há
                // peek no play (o relógio já está folheando por conta própria).
                let peek = if self.flip_state.active && !self.playhead.is_playing() {
                    self.flip_state.peek
                } else {
                    None
                };
                let ghost_selection = (self.flip_state.active && peek.is_none()).then(|| {
                    ph2d_app_flip::pass_ghosts::GhostSources {
                        selected: self.flip_state.strip.selected_keys(),
                        pinned: self.flip_state.strip.pinned_keys(),
                        trace: Some(&self.flip_state.strip.trace),
                    }
                });
                ph2d_app_flip::pass::render(
                    flip,
                    flip_render,
                    flip_compose,
                    flip_composite,
                    flip_preview.as_ref(),
                    self.flip_state.active_layer,
                    &flip_models,
                    &self.playhead,
                    ghost_selection,
                    peek,
                    game_rt,
                    camera,
                    window_size,
                    // A MESMA porta que o passe de sprites acima: sob o split da Motion este
                    // passe projetava a janela cheia e a arte do Flip andava `1/t` do que o
                    // cursor andava (report do Enio, 2026-08-25).
                    scene_viewport,
                    surface.gpu(),
                );
                // Pass 1d: a malha 3D (ADR-0150 W1/M2) — MESMO alvo `game_rt`,
                //   câmera PRÓPRIA (perspectiva orbital) e depth-buffer próprio.
                //   `LoadOp::Load`, então a cena 2D fica por baixo. No-op sem
                //   cena armada: num run normal `sculpt3d` é `None` e o frame é
                //   byte-idêntico ao de antes deste bloco existir.
                //
                // ⚠️ **O ENCODER E O SUBMIT SÃO DA PORTA desde 2026-09-08**, e não deste sítio:
                // com a divisão aberta há N vistas, e o uniform da câmera é UM — as escritas dele
                // correm na FILA, então cada vista precisa do seu `submit` entre elas, senão as N
                // desenham com a câmera da última (report do Enio; ver `render_views`).
                #[cfg(feature = "sculpt3d")]
                if let Some(scene) = sculpt3d.as_mut() {
                    scene.render(
                        surface.gpu(),
                        game_rt.view(),
                        (window_size.width, window_size.height),
                    );
                }
                // Passes 1b-bis e 1c: **OS PASSES DE LUZ** — a sprite emissiva e o glow do
                // Motion. Cortados para o irmão [`super::present_fx`] pelo tecto de LOC, e o
                // corte é por RESPONSABILIDADE: os dois somam luz sobre o `game_rt` antes do
                // tonemap, partilham o RT do `motion_fx` e a ordem entre eles é load-bearing.
                super::present_fx::run(
                    surface.gpu(),
                    super::present_fx::FxGear {
                        renderer,
                        motion_fx,
                        game_rt,
                        camera,
                        window_size,
                        scene_viewport,
                        sim,
                        motion,
                        motion_active,
                        present,
                        instances: &mut self.emissive_instances,
                    },
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
                // ⛔ **O shell NÃO escolhe o anti-aliasing deste passe, e
                // nunca mais o lê de uma preferência de texto.** Até
                // 2026-08-30 esta linha era
                // `text_rendering().params().prefer_msaa`, e ia direita
                // para `render_to_intermediate`. O `AaConfig` do Vello é
                // por PASSE, e este passe carrega o chrome **e** a arte
                // vectorial do documento no mesmo `Scene` — logo o preset
                // de tipografia escolhia a rasterização das formas do
                // artista. `Msaa16` stippla traços finos (1-1,5 px) em
                // ângulos quase-axiais: «manchas animadas parecendo TV
                // antiga» (`docs/Atualizar Stack/04_registro.md` §22.2).
                // A decisão vive agora dentro do `ph2d-render`, é
                // `AaConfig::Area` e não é parametrizável. Dois passes
                // (chrome / documento) seria arquitectura, não uma flag.
                // GPU pass profiler: Vello submits internally (its passes are out of
                // reach), so bracket the whole call with marker submits — queue order
                // makes `end − begin` cover everything Vello enqueued. No-op when off.
                let vello_span = {
                    let g = surface.gpu();
                    ph2d_gpu::pass_profiler::span_begin(&g.device, &g.queue, "render.vello")
                };
                // ⭐ A metade de CIMA das faixas — ver o cabeçalho do `present_bands`.
                if banded {
                    super::present_bands::draw_upper_bands(
                        surface.gpu(),
                        &plan,
                        super::present_bands::UpperGear {
                            world_rt,
                            window_size,
                            tonemap,
                            band_blit,
                            vello_pass,
                            band_doc_scenes,
                        },
                    );
                }
                // ⭐⭐⭐ **O VIDRO** — entre o mundo e a receita, e ANTES da cena de chrome de
                // propósito: os painéis entram pelo compositor, acima de tudo, e é isso que os
                // deixa nítidos sem uma máscara os nomear. Ver [`super::present_frost`].
                if frosting {
                    super::present_frost::glass(
                        surface.gpu(),
                        super::present_frost::Gear {
                            world_rt,
                            band_blit,
                            vello_pass,
                            tonemap,
                            frost,
                            renderer,
                            game_rt,
                            camera,
                            window_size,
                            scene_viewport,
                            doc_scene: frost_doc_scene,
                            front_scene: frost_front_scene,
                            instances: &self.frost_instances,
                            banded,
                            clear: wgpu::Color { r, g, b, a: 1.0 },
                        },
                    );
                }
                if let Err(e) = vello_pass.render_to_intermediate(
                    surface.gpu(),
                    vector_scene.inner(),
                    (window_size.width, window_size.height),
                    VelloColor::TRANSPARENT,
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
                // ⚠️ **O compositor troca de FONTE, e só na mudança de modo.** Ele guarda o
                // `game_view` num bind group construído uma vez; re-ligá-lo por quadro seria uma
                // alocação por quadro para um valor que quase nunca muda.
                super::present_bands::rebind_compositor_if_mode_changed(
                    surface.gpu(),
                    // ⚠️ **O vidro também põe o compositor a ler o acumulador**: com ele, o mundo
                    // inteiro (fundo, sprites, documento e a receita por cima do borrão) já está
                    // no `world_rt`, e o intermediário do Vello traz só o chrome.
                    banded || frosting,
                    compositor_reads_world,
                    compositor,
                    world_rt,
                    tonemap,
                    vello_pass,
                );
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
                "PH2D — {} | sprites={SPRITE_COUNT} | atlas={} ({} assets) \
                 | script={} | theme={:?} | zen={} | toasts={} | tool={}",
                // **O NOME DO FICHEIRO**, e não a lista de milestones que morava aqui: a barra de
                // título é o único sítio do app que responde *«que projeto é este?»*, e a resposta
                // dela era «M5+M6+M7+M11+M12 demo» — verdade sobre o binário, e sobre nada que o
                // artista tenha aberto.
                crate::project_io::title_name(self.project_path.as_deref()),
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
