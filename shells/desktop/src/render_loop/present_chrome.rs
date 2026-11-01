//! **O CHROME e a COMPOSIÇÃO** — a última fase do `present`, irmã por tecto de 600 LOC
//! (HR-18). ⚠️ O corte é por ASSUNTO: lá o que desenha o MUNDO; aqui o que o cobre.
//!
//! O ficheiro passou o tecto por ACUMULAÇÃO na rodada de 16/09 — nenhuma das seis linhas o
//! estoura sozinha (`CLAUDE.md` §5.0).

use super::*;

impl crate::App {
    /// O MUNDO por cima dos sprites, até à última escrita dele: o tonemap, as faixas de cima, a camada do Motion, os
    /// palcos e a lente. Corre também na vista da câmera (A6, [`super::vista_da_camera`]), que pára aqui.
    pub(in crate::render_loop) fn present_mundo_por_cima(
        &mut self,
        window_size: ph2d_host::WindowSize,
        scene_viewport: Option<[f32; 4]>,
        plan: &crate::render_loop::present_bands::FramePlan,
        banded: bool,
    ) {
        let na_vista = self.na_vista_da_camera;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            camera,
            tonemap,
            vello_pass,
            world_rt,
            band_blit,
            band_doc_scenes,
            motion,
            hero_screen,
            frame_order,
            game_rt,
            flip,
            flip_render,
            flip_compose,
            flip_composite,
            sim,
            ..
        } = gfx;
        // Pass 2: AgX tonemap
        //   target: `tonemap.output_view()` (Bgra8UnormSrgb LDR)
        tonemap.run(surface.gpu());
        // ⭐⭐⭐ AS FORMAS DO MOTION, pela placa (doc 121, W2): por cima do que está abaixo delas e por baixo dos gizmos
        // e do chrome. GEF 19 E3: cada faixa Motion no rank dela; a do topo leva a camada filtrada (ciclo 20 W3) e a
        // lente; sem objecto Motion na ordem, a placa inteira por cima do mundo (`faixa::Topo`).
        // ⚠️ A rota do Vello (`PH2D_FORMAS_NA_PLACA=0`, a de bissecção) fica por cima, na cena do chrome.
        use ph2d_app_motion::motion_shape_placa::faixa;
        let cena = hero_screen
            .as_ref()
            .filter(|_| !na_vista)
            .and_then(|h| h.cena_visivel(scene_viewport));
        let gpu = surface.gpu();
        let colagem = faixa::Colagem {
            gpu,
            ordem: frame_order,
            bands: &plan.bands,
        };
        let placa = &mut self.motion_shell.placa;
        let mut cola = |band, mundo: &_, band_blit: &_, renderer: &mut _| {
            let e = ph2d_app_motion::motion_filter::Engrenagem {
                gpu,
                renderer,
                camera,
                window: window_size,
                scene_viewport,
                formas: None,
                mundo,
                band_blit,
            };
            faixa::cola(colagem, placa, motion, e, cena, band);
        };
        let mut quadro = ph2d_app_flip::pass_quadro::Quadro::new(
            flip,
            (flip_render, flip_compose, flip_composite),
            &self.flip_state,
            &self.playhead,
            sim,
            na_vista,
            camera,
            (window_size, scene_viewport),
            gpu,
        );
        let mut flip_faixa = |b, rt: &_| {
            let ids = ph2d_app_flip::pass_quadro::Quadro::da_faixa(&frame_order.flip_ids_in(b));
            quadro.desenha(&ids, rt);
        };
        // ⭐ A metade de CIMA das faixas — ver o cabeçalho do `present_bands`.
        if banded {
            crate::render_loop::present_bands::draw_upper_bands(
                gpu,
                plan,
                crate::render_loop::present_bands::UpperGear {
                    world_rt,
                    game_rt,
                    renderer,
                    window_size,
                    tonemap,
                    band_blit,
                    vello_pass,
                    band_doc_scenes,
                    motion: &mut cola,
                    flip: &mut flip_faixa,
                },
            );
        }
        // ⭐ E cada palco de uma macro em cena pousa a sua camada (ADR-0186), por cima da do editor — na vista da
        // câmera, a camada DELA (`vista_da_camera::Vista::troca`).
        let palcos = &mut self.motion_shell.palcos;
        palcos.pousa(ph2d_app_motion::macro_track::palcos::Colagem {
            gpu: surface.gpu(),
            renderer,
            camera,
            window: window_size,
            scene_viewport,
            mundo: world_rt,
            band_blit,
        });
        // ⭐ A lente de cada palco — depois da última escrita do mundo, antes do vidro e do chrome.
        palcos.lentes(surface.gpu(), world_rt, cena);
    }

    /// Os passes 2 a 4 do quadro adquirido: o mundo por cima ([`Self::present_mundo_por_cima`]), o vidro, o chrome
    /// do Vello e o compositor para o ecrã, com o fim de quadro do perfilador de passes.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn present_chrome_and_composite(
        &mut self,
        frame: &ph2d_gpu::FrameTarget,
        window_size: ph2d_host::WindowSize,
        scene_viewport: Option<[f32; 4]>,
        plan: crate::render_loop::present_bands::FramePlan,
        banded: bool,
        frosting: bool,
        r: f64,
        g: f64,
        b: f64,
    ) {
        self.present_mundo_por_cima(window_size, scene_viewport, &plan, banded);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let AppGfx {
            surface,
            renderer,
            camera,
            game_rt,
            compositor,
            tonemap,
            vello_pass,
            vector_scene,
            world_rt,
            band_blit,
            compositor_reads_world,
            // ⭐⭐⭐ **O VIDRO JATEADO do *Edit Prefab*** (2026-09-07) — o passe, as duas cenas que
            // ele separa (o mundo sem a receita · a receita sozinha) e o interruptor do quadro,
            // que a codificação lá em cima escreveu. Ver [`crate::render_loop::present_frost`].
            frost,
            frost_doc_scene,
            frost_front_scene,
            ..
        } = gfx;
        let tamanho = (window_size.width, window_size.height);
        // Pass 3: Vello chrome — `vello_pass.intermediate_view()`, TRANSPARENT clear (o compositor revela o mundo
        // por baixo). ⛔ O shell NÃO escolhe o anti-aliasing deste passe: é `AaConfig::Area` dentro do `ph2d-render`
        // (o chrome e a arte do documento partilham o `Scene`; `docs/Atualizar Stack/04_registro.md` §22.2).
        // ⭐⭐⭐ **O VIDRO** — entre o mundo e a receita, e ANTES da cena de chrome de
        // propósito: os painéis entram pelo compositor, acima de tudo, e é isso que os
        // deixa nítidos sem uma máscara os nomear. Ver [`crate::render_loop::present_frost`].
        if frosting {
            crate::render_loop::present_frost::glass(
                surface.gpu(),
                crate::render_loop::present_frost::Gear {
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
        // ⭐⭐⭐ **Uma cena que mistura com o CENÁRIO leva o mundo por baixo** (doc 118 W2): o
        // mundo é o que o compositor poria por baixo do intermédio — o acumulador quando ele o lê,
        // o tonemap no quadro de sempre. ⛔ Sem a marca, o render de sempre, byte a byte.
        // GPU pass profiler: Vello submits internally, so bracket the render with marker submits.
        // ⚠️ Só o render: aberto antes da camada filtrada, o `render.vello` contava o filtro inteiro
        // (a `=138`: 8,9 ms «de Vello» contra 0,7 sem filtro, doc 132 §7). No-op when off.
        let vello_span = {
            let g = surface.gpu();
            ph2d_gpu::pass_profiler::span_begin(&g.device, &g.queue, "render.vello")
        };
        let feito = if vector_scene.quer_o_mundo_por_baixo() {
            let mundo = if banded || frosting {
                world_rt.sample_view()
            } else {
                tonemap.output_view()
            };
            vello_pass.render_to_intermediate_over_world(
                surface.gpu(),
                vector_scene.inner(),
                mundo,
                tamanho,
            )
        } else {
            vello_pass.render_to_intermediate(
                surface.gpu(),
                vector_scene.inner(),
                tamanho,
                VelloColor::TRANSPARENT,
            )
        };
        if let Err(e) = feito {
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
        crate::render_loop::present_bands::rebind_compositor_if_mode_changed(
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
    }
}
