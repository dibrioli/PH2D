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
        let na_vista = self.gfx.as_ref().is_some_and(|g| g.miniatura.na_vista);
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
            luz2d,
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
            sprites: plan.motion_por_faixa,
        };
        let (placa, palcos) = (&mut self.motion_shell.placa, &mut self.motion_shell.palcos);
        let mut cola = |band,
                        mundo: &_,
                        band_blit: &_,
                        renderer: &mut _,
                        sprites: faixa::PasseDeSprites<'_>| {
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
            faixa::cola(colagem, (placa, motion, palcos), e, cena, (band, sprites));
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
                    normais: luz2d.normais_mut(),
                    motion: &mut cola,
                    flip: &mut flip_faixa,
                },
            );
        }
        // ⭐ Os palcos das macros FORA da ordem pousam a sua camada (ADR-0186) por cima da do editor.
        let fora = |m: u64| frame_order.palco_fora(m);
        let palcos = &mut self.motion_shell.palcos;
        palcos.pousa(
            ph2d_app_motion::macro_track::palcos::Colagem {
                gpu: surface.gpu(),
                renderer,
                camera,
                window: window_size,
                scene_viewport,
                mundo: world_rt,
                band_blit,
            },
            fora,
        );
        // ⭐ A lente de cada palco — depois da última escrita do mundo, antes do vidro e do chrome.
        palcos.lentes(surface.gpu(), world_rt, cena, fora);
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
        let crt = self.crt_do_palco(window_size);
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
        // Pass 3: Vello chrome, TRANSPARENT clear (o compositor revela o mundo). ⛔ O AA não é da shell:
        // `AaConfig::Area` no `ph2d-render` (partilham o `Scene`; `docs/Atualizar Stack/04_registro.md` §22.2).
        // O VIDRO vem ANTES da cena de chrome de propósito (os painéis por cima, nítidos): `present_frost`.
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
        // ⭐⭐⭐ Uma cena que mistura com o CENÁRIO leva por baixo o mundo do compositor (o acumulador ou o
        // tonemap; doc 118 W2). ⛔ Sem a marca, o render de sempre, byte a byte.
        // GPU pass profiler: só o render, entre marker submits (com o filtro dentro, o `render.vello`
        // contava-o inteiro: 8,9 ms contra 0,7, doc 132 §7). No-op when off.
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
        // Pass 4: compositor (tonemap + vello) → ecrã; no palco com a CRT, por `ph2d_pixel_crt::palco`.
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
        let (gpu, fmt) = (surface.gpu(), surface.format());
        ph2d_pixel_crt::palco::compoe(gpu, fmt, frame.view(), tamanho, crt, |v| {
            compositor.run(gpu, v);
        });
        // GPU pass profiler frame tail, after the LAST instrumented submit (same queue). No-op when off.
        {
            let g = surface.gpu();
            ph2d_gpu::pass_profiler::end_frame(&g.device, &g.queue);
        }
    }
}
