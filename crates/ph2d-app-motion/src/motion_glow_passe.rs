//! **O passe do `fx.glow`** (doc 67, opção B) — saiu do `present_fx` da shell (W4 item 5.5 das
//! Macros) para correr UMA vez por fonte: o editor e cada palco de uma macro em cena. Re-desenha a
//! camada Motion da fonte em ISOLAMENTO no RT `Rgba16Float` do `motion_fx`, passa-lhe o bright-pass
//! borrado e SOMA o halo sobre o `game_rt`, antes do tonemap. Em sequência por fonte funciona: cada
//! uma escreve o RT inteiro e consome-o logo a seguir.
//!
//! ⛔ Sem `fx.glow` no grafo (ou intensidade 0, ou halo vazio) nada disto toca a GPU e o quadro é
//! byte-idêntico ao de antes da feature.

use crate::motion_glow_layer::{diag, halo_do_quadro};
use crate::motion_shape_placa::PlacaDeFormas;
use crate::motion_state::MotionState;

/// O que o passe pede da shell — as peças partilhadas por todas as fontes.
pub struct Brilho<'a> {
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub motion_fx: &'a mut ph2d_render::MotionFx,
    pub game_rt: &'a ph2d_render::GameRt,
    pub camera: &'a ph2d_render::Camera2d,
    pub window_size: ph2d_host::WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    pub sim: &'a mut ph2d_ecs::SimWorld,
}

/// **O halo de uma fonte** — `motion` e as `formas` que a placa dela desenhou neste quadro.
pub fn passe(
    gpu: &ph2d_gpu::GpuContext,
    g: &mut Brilho<'_>,
    motion: &MotionState,
    formas: &PlacaDeFormas,
    motion_active: bool,
) {
    let glow = ph2d_node_fx_glow::from_graph(&motion.doc.graph);
    // Lida FORA do `if`: dentro dele o `glow` já foi movido. Assa-se uma vez por texto.
    let halo_lut = ph2d_node_fx_glow::bake_halo_lut(&motion.doc.graph);
    // A lista do glow é a CAMADA Motion inteira, pela rota do quadro (doc 121 §9.14: num quadro do
    // DISPOSITIVO as listas da CPU são de um quadro velho).
    let halo = halo_do_quadro(motion, formas.desenhou());
    // `PH2D_GLOW_DIAG=1` — de que é feito o halo, quando muda.
    diag(motion, &halo, glow.as_ref().map(|k| k.intensity));
    let Some(glow) = glow else {
        return;
    };
    if !motion_active || glow.intensity <= 0.0 || halo.vazio() {
        return;
    }
    g.renderer.render_instances_only(
        g.motion_fx.rt_view(),
        g.camera,
        g.window_size,
        wgpu::Color::TRANSPARENT,
        &halo.cpu,
        halo.placa,
        // O MESMO sub-rect da cena fundida — senão o halo desliza para fora do que o emitiu.
        g.scene_viewport,
    );
    let tamanho = (g.window_size.width, g.window_size.height);
    if halo.formas_da_placa && !formas.redesenha_em(gpu, g.motion_fx.rt_view(), tamanho) {
        eprintln!(
            "[glow-diag] (!) as formas da placa nao entraram no halo (outro tamanho de alvo)"
        );
    }
    // A MÁSCARA DE SUJIDADE (doc 89 folha 11): o nó guarda o NOME de um objecto; resolve-se só com
    // nome autorado (sem ele, nenhuma varredura da cena por quadro).
    let dirt_cooked = |id| g.renderer.cooked_texture_id(id);
    let dirt = ph2d_node_fx_glow::dirt::source(&motion.doc.graph).and_then(|n| {
        crate::motion_glow_dirt::resolve(
            g.sim,
            crate::motion_bridge::Appearance {
                atlas: g.renderer.atlas(),
                cooked: &dirt_cooked,
            },
            &n,
        )
        .or_else(|| {
            // Um nome que não resolve é legítimo (escrito antes da sprite existir), mas não mudo.
            crate::motion_glow_dirt::diag_unresolved(&n);
            None
        })
    });
    let dirt = dirt.and_then(|r| crate::motion_glow_dirt::mask(r, g.renderer));
    g.motion_fx.bloom_over(
        gpu,
        g.game_rt.view(),
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
        // A RAMPA do halo, assada pelo nó; `None` = o `tint` constante manda.
        halo_lut.as_deref(),
        dirt,
    );
}

#[cfg(test)]
#[path = "motion_glow_passe_tests.rs"]
mod tests;
