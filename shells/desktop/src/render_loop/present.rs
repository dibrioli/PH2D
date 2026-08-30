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
                // ⭐⭐⭐ **AS FAIXAS DE DESENHO** (ADR-0154 Fase 2, report do Enio de 2026-08-30).
                //
                // Sem intercalação (`banded == false`) tudo o que se segue é **byte-idêntico** ao
                // pipeline de sempre: uma passagem de sprite, o tonemap, o Vello com documento +
                // chrome, e o compositor a ler a saída do tonemap.
                //
                // Com intercalação, o mundo passa a empilhar-se no `world_rt`:
                //   1. limpa-se o acumulador com a cor do canvas;
                //   2. desenham-se as faixas ABAIXO da última faixa de sprites;
                //   3. a última faixa de sprites é o pipeline ABAIXO, inteiro e intocado — é por
                //      isso que o Flip, a malha 3D, o bloom e o halo do Motion continuam a correr
                //      exactamente uma vez, sem uma linha movida;
                //   4. cola-se essa faixa, depois as faixas de vetor que ficam POR CIMA;
                //   5. o chrome vai para a cena de sempre e o compositor lê o acumulador.
                let (plan_bands, band_degraded) = frame_order.plan();
                let banded = crate::draw_bands::needs_banding(&plan_bands);
                let last_sprite = plan_bands
                    .iter()
                    .rposition(|b| b.family == crate::draw_bands::Family::Sprite);
                if banded {
                    if band_degraded {
                        // ⛔ Acima do tecto de faixas — a ordem da Fase 1, e ela DIZ.
                        eprintln!(
                            "[zorder] cena com mais de {} alternancias vetor/sprite: a ordem cai na da Fase 1",
                            crate::draw_bands::MAX_BANDS
                        );
                    }
                    world_rt.ensure_size(surface.gpu(), (window_size.width, window_size.height));
                    // ⚠️ A cor do canvas em espaço do DESENHISTA — o acumulador é de formato cru.
                    world_rt.clear(surface.gpu(), wgpu::Color { r, g, b, a: 1.0 });
                    let upto = last_sprite.unwrap_or(plan_bands.len());
                    let mut doc_i = 0usize;
                    for band in plan_bands.iter().take(upto) {
                        match band.family {
                            crate::draw_bands::Family::Sprite => {
                                // ⚠️ Sem `extra` e sem `gpu_extra`: o fluxo cozido do Motion não
                                // tem rank (ele não passa pelo ECS) e pertence à faixa que corre o
                                // pipeline inteiro — a última.
                                renderer.render_with_streams(
                                    game_rt.view(),
                                    present,
                                    camera,
                                    window_size,
                                    wgpu::Color::TRANSPARENT,
                                    &[],
                                    None,
                                    scene_viewport,
                                    Some((band.lo, band.hi)),
                                );
                                tonemap.run(surface.gpu());
                                band_blit.blit(
                                    surface.gpu(),
                                    world_rt.blend_view(),
                                    tonemap.output_view(),
                                    ph2d_render::BandSource::Sprites,
                                );
                            }
                            crate::draw_bands::Family::Vector => {
                                if let Some(scene) = band_doc_scenes.get(doc_i) {
                                    if let Err(e) = vello_pass.render_to_intermediate(
                                        surface.gpu(),
                                        scene.inner(),
                                        (window_size.width, window_size.height),
                                        VelloColor::TRANSPARENT,
                                    ) {
                                        eprintln!("[zorder] faixa de vetor falhou: {e}");
                                    }
                                    band_blit.blit(
                                        surface.gpu(),
                                        world_rt.blend_view(),
                                        vello_pass.intermediate_view(),
                                        ph2d_render::BandSource::Vector,
                                    );
                                }
                                doc_i += 1;
                            }
                        }
                    }
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
                    last_sprite
                        .filter(|_| banded)
                        .map(|i| (plan_bands[i].lo, plan_bands[i].hi)),
                );
                // ⚠️ **A SONDA DO DRIFT** (`PH2D_PAN_DIAG=1`, report do Enio de 2026-08-25) —
                // DEPOIS do passe, de propósito: o que ela tem de imprimir é o sub-retângulo
                // que ele APLICOU, e esse só existe depois de ele decidir (o `.filter` do
                // clip/máscara é por conteúdo do quadro).
                crate::pan_diag::frame(
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
                #[cfg(feature = "sculpt3d")]
                if let Some(scene) = sculpt3d.as_mut() {
                    let gpu = surface.gpu();
                    let mut enc =
                        gpu.device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("ph2d-mesh encoder"),
                            });
                    scene.render(
                        gpu,
                        &mut enc,
                        game_rt.view(),
                        (window_size.width, window_size.height),
                    );
                    gpu.queue.submit([enc.finish()]);
                }
                // Pass 1b-bis: **A SPRITE COMO FONTE DE LUZ** (plano `docs/Sprite_projeto/18` W8,
                //   Enio 2026-08-21). Mesma máquina do glow do Motion logo abaixo, outra lista:
                //   re-desenha em ISOLAMENTO as sprites que carregam `SpriteEmissive` — com o
                //   `tint` já multiplicado pela intensidade — no RT `Rgba16Float` do `motion_fx`,
                //   e SOMA o bright-pass borrado sobre o `game_rt`, antes do tonemap.
                //
                // ⚠️ **Corre ANTES do glow do Motion, e a ordem é load-bearing.** Os dois passes
                //   partilham o RT do `motion_fx` e cada um escreve-o inteiro para o consumir logo
                //   a seguir — em sequência funciona, entrelaçados um apagaria o outro.
                //
                // ⚠️ Sem nenhuma sprite a emitir a lista sai VAZIA e este bloco não toca a GPU: o
                //   quadro é byte-idêntico ao de antes desta feature existir (há gate).
                crate::render_loop::sprite_emissive::collect(
                    sim,
                    present,
                    &mut self.emissive_instances,
                );
                if !self.emissive_instances.is_empty() {
                    renderer.render_instances_only(
                        motion_fx.rt_view(),
                        camera,
                        window_size,
                        wgpu::Color::TRANSPARENT,
                        &self.emissive_instances,
                        // O MESMO sub-rect da cena fundida — senão o halo desliza para fora da
                        // sprite que o emitiu (o mesmo cuidado que o glow do Motion documenta).
                        scene_viewport,
                    );
                    motion_fx.bloom_over(
                        surface.gpu(),
                        game_rt.view(),
                        &crate::render_loop::sprite_emissive::bloom_params(),
                        // ⚠️ **Sem rampa, e é a mesma razão dos outros campos deste sítio**: a
                        // rampa é autoria de um NÓ, e um emissor de sprite não tem nó nenhum.
                        None,
                        // ⚠️ **E sem máscara de sujidade, pela MESMA razão levada um passo
                        // adiante**: a imagem dela nomeia um objecto da cena, e é o nó que
                        // guarda o nome — um emissor de sprite não tem onde o escrever.
                        None,
                    );
                }
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
                // ⚠️ **Assada FORA do `if` de propósito**: dentro dele o `glow` já foi movido, e
                // pô-la lá obrigaria a reordenar o bloco. O custo de a assar sem a usar é uma
                // varredura de 512 avaliações num quadro em que existe um `fx.glow` com rampa —
                // e o passe só corre nesse quadro de qualquer forma.
                let halo_lut = ph2d_node_fx_glow::bake_halo_lut(&motion.doc.graph);
                // ⚠️ **A LISTA DO GLOW É A CAMADA MOTION, e não o passe de sprites**
                // (bug do Enio, 2026-08-20: *"Glow não funciona com shape"*, e a
                // ordem dele depois: *"tudo deve brilhar"*). Ver
                // [`super::motion_glow_layer`] — a metade vetorial viva entra aqui
                // pelo TILE assado, porque um halo é imediatamente reduzido por seis
                // níveis de mip e nunca precisou de nitidez de tela.
                let glow_layer = super::motion_glow_layer::layer_instances(
                    &motion.pump.instances,
                    &motion.pump.vector_instances,
                    &motion.object_bake,
                    &motion.shape_bake,
                );
                // ⚠️ **`PH2D_GLOW_DIAG=1`** — de que é feita a camada, quando ela muda.
                // Ver o doc de [`super::motion_glow_layer::diag`]: «o halo não
                // aparece» tem cinco causas indistinguíveis a olho.
                super::motion_glow_layer::diag(
                    &motion.pump.instances,
                    &motion.pump.vector_instances,
                    &motion.object_bake,
                    &motion.shape_bake,
                    glow.as_ref().map(|g| g.intensity),
                    glow_layer.len(),
                );
                if let Some(glow) = glow
                    && motion_active
                    && glow.intensity > 0.0
                    && !glow_layer.is_empty()
                {
                    renderer.render_instances_only(
                        motion_fx.rt_view(),
                        camera,
                        window_size,
                        wgpu::Color::TRANSPARENT,
                        &glow_layer,
                        // SAME sub-rect the fused scene used above — or the glow
                        // desyncs from the sparks (the halo floats away).
                        scene_viewport,
                    );
                    // **A MÁSCARA DE SUJIDADE** (doc 89 folha 11) — o nó guarda o NOME de um
                    // objecto da cena e o passe de tela quer uma `TextureView`. As duas metades
                    // encontram-se aqui, DEPOIS do passe de isolamento: aquele leva o renderer
                    // emprestado mutável, e a resolução só o lê.
                    //
                    // ⚠️ **A resolução corre só quando há nome autorado.** Sem ele o `resolve`
                    // nem é chamado — uma varredura da cena por quadro para responder *"nada"*
                    // é o custo que o caminho de sempre não pode pagar.
                    let dirt_cooked = |id| renderer.cooked_texture_id(id);
                    let dirt = ph2d_node_fx_glow::dirt::source(&motion.doc.graph).and_then(|n| {
                        super::motion_glow_dirt::resolve(
                            sim,
                            super::motion_bridge::Appearance {
                                atlas: renderer.atlas(),
                                cooked: &dirt_cooked,
                            },
                            &n,
                        )
                        .or_else(|| {
                            // ⚠️ **Um nome que não resolve é a SEXTA causa indistinguível a
                            // olho** desta família (ver `motion_glow_layer::diag`, que já
                            // documenta cinco). Ele é legítimo — um nome pode ser escrito antes
                            // de a sprite existir —, então não é erro; mas ficar mudo é o que
                            // torna *"escrevi o nome e não aconteceu nada"* indiagnosticável.
                            super::motion_glow_dirt::diag_unresolved(&n);
                            None
                        })
                    });
                    let dirt = dirt.and_then(|r| super::motion_glow_dirt::mask(r, renderer));
                    motion_fx.bloom_over(
                        surface.gpu(),
                        game_rt.view(),
                        &ph2d_render::BloomParams {
                            threshold: glow.threshold,
                            knee: glow.knee,
                            intensity: glow.intensity,
                            radius: glow.radius,
                            saturation: glow.saturation,
                            tint: glow.tint,
                            stretch: glow.stretch,
                            angle: glow.angle,
                            clamp: glow.clamp,
                            operation: glow.operation,
                            source: glow.source,
                            dirt_intensity: glow.dirt_intensity,
                        },
                        // **A RAMPA DO HALO** (doc 89 folha 11) — assada pelo nó, que é quem
                        // possui a semântica do gradiente; aqui ela só atravessa. `None` quando
                        // o artista não desenhou nenhuma, e aí o `tint` constante manda.
                        halo_lut.as_deref(),
                        dirt,
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
                // ⭐ **A metade de CIMA das faixas** (ADR-0154 Fase 2) — a última faixa de
                // sprites (que é o pipeline inteiro acima) e as faixas de vetor que ficam por cima
                // dela. Depois disto o `vector_scene` só tem o chrome, e ele vai para o
                // intermediário do Vello como sempre.
                if banded {
                    band_blit.blit(
                        surface.gpu(),
                        world_rt.blend_view(),
                        tonemap.output_view(),
                        ph2d_render::BandSource::Sprites,
                    );
                    let from = last_sprite.map_or(0, |i| i + 1);
                    let mut doc_i = plan_bands
                        .iter()
                        .take(from)
                        .filter(|b| b.family == crate::draw_bands::Family::Vector)
                        .count();
                    for band in plan_bands.iter().skip(from) {
                        if band.family != crate::draw_bands::Family::Vector {
                            continue;
                        }
                        if let Some(scene) = band_doc_scenes.get(doc_i) {
                            if let Err(e) = vello_pass.render_to_intermediate(
                                surface.gpu(),
                                scene.inner(),
                                (window_size.width, window_size.height),
                                VelloColor::TRANSPARENT,
                            ) {
                                eprintln!("[zorder] faixa de vetor falhou: {e}");
                            }
                            band_blit.blit(
                                surface.gpu(),
                                world_rt.blend_view(),
                                vello_pass.intermediate_view(),
                                ph2d_render::BandSource::Vector,
                            );
                        }
                        doc_i += 1;
                    }
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
                if banded != *compositor_reads_world {
                    compositor.rebind(
                        surface.gpu(),
                        if banded {
                            world_rt
                                .texture()
                                .create_view(&wgpu::TextureViewDescriptor {
                                    label: Some("world RT sample view (sRGB)"),
                                    format: Some(ph2d_render::WorldRt::SAMPLE_FORMAT),
                                    ..Default::default()
                                })
                        } else {
                            tonemap
                                .output_texture()
                                .create_view(&wgpu::TextureViewDescriptor::default())
                        },
                        vello_pass
                            .intermediate_texture()
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    );
                    *compositor_reads_world = banded;
                }
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
