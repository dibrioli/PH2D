//! **O CHROME e a COMPOSIÇÃO** — a última fase do `present`, irmã por tecto de 600 LOC
//! (HR-18). ⚠️ O corte é por ASSUNTO: lá o que desenha o MUNDO; aqui o que o cobre.
//!
//! O ficheiro passou o tecto por ACUMULAÇÃO na rodada de 16/09 — nenhuma das seis linhas o
//! estoura sozinha (`CLAUDE.md` §5.0).

use super::*;

impl crate::App {
    /// Os passes 2 a 4 do quadro adquirido: o tonemap, as faixas de cima, o vidro, o chrome do Vello e o compositor para
    /// o ecrã, com o fim de quadro do perfilador de passes.
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
            // ADR-0154 Fase 2 — o acumulador do mundo, a colagem de faixa, as cenas do documento
            // por faixa (codificadas no `run_render_frame`) e o modo do compositor.
            world_rt,
            band_blit,
            band_doc_scenes,
            compositor_reads_world,
            // ⭐⭐⭐ **O VIDRO JATEADO do *Edit Prefab*** (2026-09-07) — o passe, as duas cenas que
            // ele separa (o mundo sem a receita · a receita sozinha) e o interruptor do quadro,
            // que a codificação lá em cima escreveu. Ver [`crate::render_loop::present_frost`].
            frost,
            frost_doc_scene,
            frost_front_scene,
            ..
        } = gfx;
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
            crate::render_loop::present_bands::draw_upper_bands(
                surface.gpu(),
                &plan,
                crate::render_loop::present_bands::UpperGear {
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
