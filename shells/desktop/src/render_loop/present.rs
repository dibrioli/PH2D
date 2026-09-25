//! Paint+present phase: 4 GPU passes (sprite → tonemap → vello → compositor)
//! + window title refresh + request_redraw.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a sibling
//! method on `App` via split impl. Behavior-preserving lift. Caller in
//! mod.rs invokes via `self.run_present_phase(cpu_start, r, g, b)` after
//! the editor-mode / M5-demo paint branch finishes encoding the
//! `VectorScene`.

use crate::AppGfx;
use ph2d_gpu::AcquireError;
use ph2d_vector::Color as VelloColor;
use std::time::Instant;

/// O título da janela e o pedido do quadro seguinte — filho por assunto, num ficheiro irmão.
#[path = "present_title.rs"]
mod title;

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
        // ⭐⭐⭐ **A TINTA da máscara de protecção da Remoção de fundo** (2026-09-15) — uma instância
        // com a MALHA da arte, retirada aqui pela mesma razão dos fantasmas. Vazia sem prévia.
        let bgremoval_tint = std::mem::take(&mut self.bgremoval.tint_extra);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // A janela tem de existir para o quadro se apresentar; o título e o pedido do quadro seguinte re-derivam-na.
        if self.host.is_none() {
            return;
        }
        // Só o que o `acquire` e o sub-retângulo da cena usam: cada passagem abaixo re-deriva o seu.
        let AppGfx {
            surface,
            tools,
            // Motion Nodes M0.T13: read the center split to frame the scene into
            // its sub-rect (set by the bridge earlier this frame).
            hero_screen,
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
                let Some((plan, banded)) = self.present_world_sprites(
                    r,
                    g,
                    b,
                    window_size,
                    scene_viewport,
                    motion_active,
                    frosting,
                    &onion_ghosts,
                    &bgremoval_tint,
                ) else {
                    return;
                };
                self.present_flip_sculpt_fx(
                    window_size,
                    scene_viewport,
                    motion_active,
                    &flip_preview,
                );
                self.present_chrome_and_composite(
                    &frame,
                    window_size,
                    scene_viewport,
                    plan,
                    banded,
                    frosting,
                    r,
                    g,
                    b,
                );
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

        self.present_title_and_redraw();
    }

    /// O passe 1 do quadro adquirido: o stream do Motion e os fantasmas do onion no slot `extra`, as faixas de baixo,
    /// o passe de sprites (com as peças da receita aberta retidas) e a sonda do drift. Devolve o plano das faixas.
    #[allow(clippy::too_many_arguments)]
    fn present_world_sprites(
        &mut self,
        r: f64,
        g: f64,
        b: f64,
        window_size: ph2d_host::WindowSize,
        scene_viewport: Option<[f32; 4]>,
        motion_active: bool,
        frosting: bool,
        onion_ghosts: &ph2d_render::LiftedInstances,
        // ⭐ A tinta da máscara da Remoção de fundo — o TERCEIRO produtor deste slot.
        bgremoval_tint: &ph2d_render::LiftedInstances,
    ) -> Option<(super::present_bands::FramePlan, bool)> {
        let gfx = self.gfx.as_mut()?;
        let AppGfx {
            surface,
            renderer,
            present,
            // ⭐ As partículas dos objectos (TOP-20 #18) — ver o bloco do `extra`, abaixo.
            particles,
            camera,
            game_rt,
            tonemap,
            vello_pass,
            // Motion Nodes M0.T10/T11: the cooked stream, injected into the sprite
            // pass below when the `motion` tool is active (the bridge pumped it
            // into `motion.pump.instances` earlier this frame).
            motion,
            sim,
            world_rt,
            band_blit,
            band_doc_scenes,
            frame_order,
            hero_screen,
            grid_behind_scene,
            ..
        } = gfx;
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
        // ⭐⭐⭐ **A LEI DO DONO também vale no DISPOSITIVO** (report de 2026-09-19: *«os retângulos
        // voltaram»*). O lowering de CPU cala uma corrente de posições, e o caminho da GPU **nunca
        // perguntava** — ver `lei_da_aparencia::a_arte_desenha`, que responde pela FRONTEIRA e sem
        // ler o dispositivo de volta.
        let arte_desenha = ph2d_app_motion::lei_da_aparencia::a_arte_desenha(
            motion,
            ph2d_eval_motion::so_com_forma_por_ordem(),
        );
        let motion_gpu: Option<(&wgpu::Buffer, u32, &[ph2d_render::GpuTexRun])> =
            (motion_active && arte_desenha && motion.gpu_live)
                .then(|| motion.gpu_cook.instances())
                .flatten()
                .map(|gi| (gi.buffer(), gi.len(), motion.gpu_cook.texture_runs()));
        let motion_slice: &[ph2d_render::RenderInstance] = if motion_active && motion_gpu.is_none()
        {
            &motion.pump.instances
        } else {
            &[]
        };
        // O slot `extra` do passe carrega TRÊS produtores CPU: os fantasmas do onion (ADR-0142),
        // o stream do Motion e — desde o TOP-20 #18 — as PARTÍCULAS dos objectos.
        //
        // ⚠️ **Os fantasmas e a TINTA DA MÁSCARA levam MALHA; o Motion e as partículas não**
        // (plano `docs/Skeleton/03` W7 + a wave do bgremoval de 15/09), então os quatro viajam
        // juntos numa `LiftedInstances` — ⛔ um vector paralelo de malhas ao lado de uma fatia crua
        // é o padrão que o `corner_radius` proíbe por escrito.
        //
        // ⚠️⚠️ **As partículas desenham-se SEMPRE, e é isso que as separa do Motion:** o stream do
        // grafo só existe com a ferramenta MOTION na mão, e um jacto preso a um objecto tem de arder
        // com qualquer ferramenta — senão o componente some quando o artista pega no pincel.
        //
        // ⚠️ **O caso comum não copia nada:** com só os fantasmas vivos, o `extra` é a lista deles,
        // já montada pela fase de overlay. É por isso que o atalho é testado contra os outros TRÊS.
        let particulas: &[ph2d_render::RenderInstance] = &particles.instances;
        let so_fantasmas =
            motion_slice.is_empty() && bgremoval_tint.is_empty() && particulas.is_empty();
        let sprite_extra: ph2d_render::LiftedInstances = if so_fantasmas {
            ph2d_render::LiftedInstances::default()
        } else {
            let mut e = ph2d_render::LiftedInstances::default();
            for (i, inst) in onion_ghosts.instances().iter().enumerate() {
                e.push(*inst, onion_ghosts.mesh_of(i));
            }
            for (i, inst) in bgremoval_tint.instances().iter().enumerate() {
                e.push(*inst, bgremoval_tint.mesh_of(i));
            }
            for i in motion_slice {
                e.push(*i, None);
            }
            for i in particulas {
                e.push(*i, None);
            }
            e
        };
        let extra: &ph2d_render::LiftedInstances = if so_fantasmas {
            onion_ghosts
        } else {
            &sprite_extra
        };
        // ⭐⭐⭐ **AS FAIXAS DE DESENHO** (ADR-0154 Fase 2) — a lei, os cinco passos e o
        // porquê de a ÚLTIMA faixa de sprites não se ter movido vivem no cabeçalho do
        // irmão `present_bands`. Sem intercalação nada disto corre e o quadro é
        // **byte-idêntico** ao de sempre.
        let mut plan = super::present_bands::plan_frame(frame_order);
        // ⭐⭐ **A GRADE ATRÁS DOS OBJECTOS força o quadro em CAMADAS** (report do dono de
        //    2026-09-24): é o acumulador do mundo que sabe pôr alguma coisa por baixo dos sprites.
        //    ⚠️ Sem intercalação e sem `Behind`, nada muda — o quadro de sempre, byte a byte.
        let grid_behind = hero_screen.as_ref().is_some_and(|h| {
            ph2d_editor_core::screens::hero::grid_layer::paint_behind(h, grid_behind_scene)
        });
        plan.banded |= grid_behind;
        let banded = plan.banded;
        // ⭐⭐⭐ **QUEM SOBE PARA CIMA DO VIDRO** — as peças da receita aberta saem do
        // fundo (senão o borrão delas escapa por fora da silhueta, como um halo) e são
        // desenhadas depois, do outro lado do vidro. `None` sem receita aberta, e aí toda
        // linha abaixo é a de sempre.
        let held =
            frosting.then(|| super::present_frost::lift(sim, present, &mut self.frost_instances));
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
                    grid_behind: grid_behind.then_some(&*grid_behind_scene),
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
        Some((plan, banded))
    }

    /// Os passes 1b a 1c do quadro adquirido: o Flip composto por camada (com os fantasmas e o *peek*), a malha 3D, e
    /// os passes de luz (a sprite emissiva e o glow do Motion).
    fn present_flip_sculpt_fx(
        &mut self,
        window_size: ph2d_host::WindowSize,
        scene_viewport: Option<[f32; 4]>,
        motion_active: bool,
        flip_preview: &Option<ph2d_flip_render::FlipGpuData>,
    ) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            surface,
            renderer,
            present,
            camera,
            game_rt,
            motion_fx,
            motion,
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
        // Pass 1b: Flip (ADR-0114 W1) composto por-camada (blend/opacity via
        //   compositor 22-modos) no `game_rt`, amostrado pelo playhead,
        //   MESMA câmera dos sprites. O blit final usa LoadOp::Load (preserva
        //   os sprites por baixo). No-op sem camada Flip ativa (default).
        let flip_models = ph2d_flip_entities::transform::build(sim, &self.flip_state.entities);
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
    }
}

#[path = "present_chrome.rs"]
mod chrome_e_composicao;
