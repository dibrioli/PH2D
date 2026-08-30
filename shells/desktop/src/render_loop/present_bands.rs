//! ⭐⭐⭐ **AS FAIXAS DE DESENHO** (ADR-0154 Fase 2) — irmão por ASSUNTO do [`super::present`], e
//! pelo tecto de 600 LOC do shell (HR-18).
//!
//! ⚠️ **Este corte foi imposto por um gate que esteve VERMELHO sem ninguém ver**
//! (`shell_files_respect_hr18_loc_cap`): ele vive em `shells/desktop/tests/` e o portão de fecho
//! desta linha corria `cargo test --bins`, que **não toca** naquele diretório.
//!
//! ⚠️ **O que NÃO se moveu é a última faixa de sprites** — ela é o pipeline de sempre, inteiro e
//! intocado, e é por isso que o Flip, a malha 3D, o bloom e o halo do Motion continuam a correr
//! exactamente uma vez sem uma linha movida. Aqui só vivem as faixas ABAIXO e ACIMA dela.
//!
//! ⛔ Sem intercalação nada disto corre, e o quadro é **byte-idêntico** ao de sempre.

use ph2d_host::WindowSize;
use ph2d_vector::Color as VelloColor;

/// ⚠️ **A engrenagem viaja numa struct, e não em onze argumentos.** Os dois passes usam
/// exactamente as mesmas peças; passá-las soltas faria duas listas que divergiriam na primeira
/// peça nova.
pub(super) struct BandGear<'a> {
    pub world_rt: &'a mut ph2d_render::WorldRt,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub game_rt: &'a ph2d_render::GameRt,
    pub present: &'a mut ph2d_ecs::PresentWorld,
    pub camera: &'a ph2d_render::Camera2d,
    pub window_size: WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub tonemap: &'a mut ph2d_render::Tonemap,
    pub band_blit: &'a mut ph2d_render::BandBlit,
    pub vello_pass: &'a mut ph2d_render::VelloPass,
    pub band_doc_scenes: &'a [ph2d_vector::VectorScene],
}

/// O plano de faixas deste quadro — as quatro respostas que os dois passes e o compositor leem.
pub(super) struct FramePlan {
    /// As faixas, em ordem de desenho.
    pub bands: Vec<crate::draw_bands::Band>,
    /// A cena passou do tecto de faixas e caiu na ordem da Fase 1.
    pub degraded: bool,
    /// Há intercalação a fazer? `false` ⇒ o quadro é o de sempre, **byte-idêntico**.
    pub banded: bool,
    /// O índice da ÚLTIMA faixa de sprites — a que corre o pipeline inteiro.
    pub last_sprite: Option<usize>,
}

/// Lê a ordem do quadro e responde as quatro.
pub(super) fn plan_frame(frame_order: &crate::draw_bands::FrameOrder) -> FramePlan {
    let (bands, degraded) = frame_order.plan();
    let banded = crate::draw_bands::needs_banding(&bands);
    let last_sprite = bands
        .iter()
        .rposition(|b| b.family == crate::draw_bands::Family::Sprite);
    FramePlan {
        bands,
        degraded,
        banded,
        last_sprite,
    }
}

/// As faixas **ABAIXO** da última faixa de sprites. Ela limpa o acumulador e empilha, em ordem,
/// tudo o que fica por baixo do pipeline principal.
pub(super) fn draw_lower_bands(
    gpu: &ph2d_gpu::GpuContext,
    plan: &FramePlan,
    clear: wgpu::Color,
    mut g: BandGear<'_>,
) {
    if plan.degraded {
        // ⛔ Acima do tecto de faixas — a ordem da Fase 1, e ela DIZ.
        eprintln!(
            "[zorder] cena com mais de {} alternancias vetor/sprite: a ordem cai na da Fase 1",
            crate::draw_bands::MAX_BANDS
        );
    }
    g.world_rt
        .ensure_size(gpu, (g.window_size.width, g.window_size.height));
    // ⚠️ **LINEAR** — a mesma cor que o passe de sprite usa. Quem converte para o
    // espaço do desenhista é a porta, e não o chamador (report do Enio: *«o canvas
    // escurece»*, que foi exactamente esta escolha feita no sítio errado).
    g.world_rt.clear_linear(gpu, clear);
    let upto = plan.last_sprite.unwrap_or(plan.bands.len());
    let mut doc_i = 0usize;
    for band in plan.bands.iter().take(upto) {
        match band.family {
            crate::draw_bands::Family::Sprite => {
                // ⚠️ Sem `extra` e sem `gpu_extra`: o fluxo cozido do Motion não
                // tem rank (ele não passa pelo ECS) e pertence à faixa que corre o
                // pipeline inteiro — a última.
                g.renderer.render_with_streams(
                    g.game_rt.view(),
                    g.present,
                    g.camera,
                    g.window_size,
                    wgpu::Color::TRANSPARENT,
                    &[],
                    None,
                    g.scene_viewport,
                    Some((band.lo, band.hi)),
                );
                g.tonemap.run(gpu);
                g.band_blit.blit(
                    gpu,
                    g.world_rt.blend_view(),
                    g.tonemap.output_view(),
                    ph2d_render::BandSource::Sprites,
                );
            }
            crate::draw_bands::Family::Vector => {
                if let Some(scene) = g.band_doc_scenes.get(doc_i) {
                    if let Err(e) = g.vello_pass.render_to_intermediate(
                        gpu,
                        scene.inner(),
                        (g.window_size.width, g.window_size.height),
                        VelloColor::TRANSPARENT,
                    ) {
                        eprintln!("[zorder] faixa de vetor falhou: {e}");
                    }
                    g.band_blit.blit(
                        gpu,
                        g.world_rt.blend_view(),
                        g.vello_pass.intermediate_view(),
                        ph2d_render::BandSource::Vector,
                    );
                }
                doc_i += 1;
            }
        }
    }
}

/// ⚠️ **A metade de CIMA precisa de MENOS peças**, e por isso tem engrenagem própria: ela não corre
/// o pipeline de sprites (ele já correu), então `renderer`/`game_rt`/`present`/`camera` e o
/// recorte da vista **não têm nada que fazer aqui**. *Uma engrenagem partilhada que carregasse
/// peças que um dos lados não usa esconde qual dos dois as move.*
pub(super) struct UpperGear<'a> {
    pub world_rt: &'a mut ph2d_render::WorldRt,
    pub window_size: WindowSize,
    pub tonemap: &'a mut ph2d_render::Tonemap,
    pub band_blit: &'a mut ph2d_render::BandBlit,
    pub vello_pass: &'a mut ph2d_render::VelloPass,
    pub band_doc_scenes: &'a [ph2d_vector::VectorScene],
}

/// As faixas **ACIMA** da última faixa de sprites — ela cola essa última faixa (que o tonemap
/// acabou de produzir) e depois o vetor que fica por cima dela.
pub(super) fn draw_upper_bands(gpu: &ph2d_gpu::GpuContext, plan: &FramePlan, mut g: UpperGear<'_>) {
    g.band_blit.blit(
        gpu,
        g.world_rt.blend_view(),
        g.tonemap.output_view(),
        ph2d_render::BandSource::Sprites,
    );
    let from = plan.last_sprite.map_or(0, |i| i + 1);
    let mut doc_i = plan
        .bands
        .iter()
        .take(from)
        .filter(|b| b.family == crate::draw_bands::Family::Vector)
        .count();
    for band in plan.bands.iter().skip(from) {
        if band.family != crate::draw_bands::Family::Vector {
            continue;
        }
        if let Some(scene) = g.band_doc_scenes.get(doc_i) {
            if let Err(e) = g.vello_pass.render_to_intermediate(
                gpu,
                scene.inner(),
                (g.window_size.width, g.window_size.height),
                VelloColor::TRANSPARENT,
            ) {
                eprintln!("[zorder] faixa de vetor falhou: {e}");
            }
            g.band_blit.blit(
                gpu,
                g.world_rt.blend_view(),
                g.vello_pass.intermediate_view(),
                ph2d_render::BandSource::Vector,
            );
        }
        doc_i += 1;
    }
}

/// ⭐ **O compositor troca de FONTE, e só na mudança de modo.**
///
/// ⚠️ Ele guarda o `game_view` num bind group construído uma vez; re-ligá-lo por quadro seria uma
/// alocação por quadro para um valor que quase nunca muda.
pub(super) fn rebind_compositor_if_mode_changed(
    gpu: &ph2d_gpu::GpuContext,
    banded: bool,
    reads_world: &mut bool,
    compositor: &mut ph2d_render::Compositor,
    world_rt: &ph2d_render::WorldRt,
    tonemap: &ph2d_render::Tonemap,
    vello_pass: &ph2d_render::VelloPass,
) {
    if banded == *reads_world {
        return;
    }
    compositor.rebind(
        gpu,
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
    *reads_world = banded;
}

impl FramePlan {
    /// ⭐ **A janela de rank da ÚLTIMA faixa de sprites** — o que o pipeline principal desenha
    /// quando há intercalação. `None` = sem intercalação, e aí ele desenha tudo, como sempre.
    pub(super) fn rank_window(&self) -> Option<(u32, u32)> {
        self.last_sprite
            .filter(|_| self.banded)
            .map(|i| (self.bands[i].lo, self.bands[i].hi))
    }
}
