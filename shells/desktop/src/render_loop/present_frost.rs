//! ⭐⭐⭐ **O VIDRO JATEADO por trás da receita aberta** (Enio, 2026-09-07: *«crie a feature de
//! borrar discretamente o que está por trás do prefab como um vidro jateado»*) — irmão por ASSUNTO
//! do [`super::present`], e pelo tecto de 600 LOC do shell (HR-18).
//!
//! # A ordem do quadro com o vidro em cima
//!
//! ```text
//! sprites (SEM as peças da receita) ─tonemap→ ┐
//!                                              ├→ world_rt ─FROST→ world_rt (jateado)
//! documento (SEM a receita, pelo Vello) ──────→ ┘                      │
//!                                                                      ▼
//!                      as peças raster da receita ─tonemap→ world_rt ──┤
//!                      a receita vectorial ────────pelo Vello→ world_rt┘
//!                                                                      │
//!                                       chrome (painéis) ─compositor──→ ecrã
//! ```
//!
//! ⚠️⚠️ **O acumulador do mundo é a máscara, e por isso não há máscara nenhuma.** No `world_rt`
//! estão o fundo do canvas, as sprites e o documento — e **não** estão os painéis, que vivem na
//! cena de chrome e entram no compositor depois. ⇒ *«borrar o canvas e tudo o que está nele, e mais
//! nada»* é o conteúdo daquela textura. ⛔ A alternativa — borrar o resultado final dentro de um
//! rectângulo — precisa de conhecer o layout e erra em toda janela flutuante que passe por cima.
//!
//! ⚠️⚠️ **As peças da receita saem do fundo em vez de serem redesenhadas por cima.** Desenhá-las
//! duas vezes (uma no fundo, uma nítida por cima) parece o mesmo e não é: o borrão da peça escapa
//! por fora da silhueta dela e a receita ganha um **halo**. ⇒ o passe de sprites do fundo RETÉM as
//! entidades marcadas (`held_back`) e este módulo desenha-as depois, do outro lado do vidro.
//!
//! ⛔ **Sem receita aberta nada disto corre**, e o quadro é o de sempre — a mesma cerca do
//! `present_bands`.

use ph2d_ecs::{Entity, MasterEditing, PresentWorld, SimWorld};
use ph2d_host::WindowSize;
use ph2d_vector::Color as VelloColor;
use std::collections::BTreeSet;

/// ⭐⭐ **Quem sobe para cima do vidro** — as peças da receita aberta que o passe de sprites tem de
/// reter, e as instâncias delas para o passe de cima.
///
/// ⚠️ **As duas respostas saem da MESMA varredura**, de propósito: uma lista de *quem não desenhar*
/// e outra de *o que desenhar depois* construídas em sítios diferentes divergem no dia em que uma
/// delas ganhar um filtro — e o sintoma seria uma peça a desaparecer do ecrã, ou a aparecer
/// duplicada com halo.
///
/// ⛔ Devolve vazio quando não há receita aberta, e aí o chamador passa `None` — o caminho de
/// sempre não paga uma varredura para responder *«ninguém»*… ele paga esta, que é `O(instâncias)`
/// e corre uma vez por quadro. *A alternativa seria a shell perguntar duas vezes.*
///
/// ⚠️ **Pela porta que leva a MALHA junto** (`LiftedInstances::collect_from`, plano
/// `docs/Skeleton/03` W3): uma imagem presa ao esqueleto da receita sobe deformada, como está na
/// cena — uma cópia só do `RenderInstance` subia o quad de repouso.
pub(super) fn lift(
    sim: &SimWorld,
    present: &mut PresentWorld,
    out: &mut ph2d_render::LiftedInstances,
) -> BTreeSet<Entity> {
    let mut held = BTreeSet::new();
    out.collect_from(present, |entity, _| {
        let sobe = sim.world().get::<MasterEditing>(entity).is_some();
        if sobe {
            held.insert(entity);
        }
        sobe
    });
    held
}

/// A engrenagem do vidro. ⚠️ **Uma struct e não dezasseis argumentos** — o mesmo motivo do
/// [`super::present_bands::BandGear`].
pub(super) struct Gear<'a> {
    pub world_rt: &'a mut ph2d_render::WorldRt,
    pub band_blit: &'a ph2d_render::BandBlit,
    pub vello_pass: &'a mut ph2d_render::VelloPass,
    pub tonemap: &'a mut ph2d_render::Tonemap,
    pub frost: &'a mut ph2d_render::FrostPass,
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub game_rt: &'a ph2d_render::GameRt,
    pub camera: &'a ph2d_render::Camera2d,
    pub window_size: WindowSize,
    pub scene_viewport: Option<[f32; 4]>,
    /// O DOCUMENTO sem a receita — só quando o quadro não está intercalado (com faixas ele já foi
    /// composto por elas).
    pub doc_scene: &'a ph2d_vector::VectorScene,
    /// A RECEITA, sozinha — o que fica nítido acima do vidro.
    pub front_scene: &'a ph2d_vector::VectorScene,
    /// As peças raster da receita, retidas pelo fundo — com as malhas delas.
    pub instances: &'a ph2d_render::LiftedInstances,
    /// O quadro já está intercalado? Então o acumulador já está cheio.
    pub banded: bool,
    /// A cor de fundo do canvas, em **luz linear** — a mesma que o passe de sprite usa.
    pub clear: wgpu::Color,
}

/// ⭐⭐⭐ **Põe o vidro e desenha a receita por cima.**
pub(super) fn glass(gpu: &ph2d_gpu::GpuContext, g: Gear<'_>) {
    let size = (g.window_size.width, g.window_size.height);
    if !g.banded {
        // Sem faixas, o acumulador ainda não existe neste quadro: monta-se aqui a mesma pilha que
        // o compositor faria — o mundo de sprites, e o documento por cima dele.
        g.world_rt.ensure_size(gpu, size);
        g.world_rt.clear_linear(gpu, g.clear);
        g.band_blit.blit(
            gpu,
            g.world_rt.blend_view(),
            g.tonemap.output_view(),
            ph2d_render::BandSource::Sprites,
        );
        vector_band(
            gpu,
            g.vello_pass,
            g.band_blit,
            g.world_rt,
            g.window_size,
            g.doc_scene,
        );
    }
    // ⭐ **O VIDRO.** Ele lê e escreve a MESMA textura — as intermediárias vivem dentro do passe,
    // que é onde o wgpu exige que estejam.
    g.frost.run(gpu, g.world_rt.blend_view(), size, g.clear);
    // ⭐ **As peças RASTER da receita**, do outro lado do vidro. ⚠️ Elas re-usam o `game_rt` e o
    // tonemap — a mesma manobra de uma faixa de sprites, e pela mesma razão: a receita tem de
    // atravessar o MESMO AgX que o resto da cena, senão as cópias e a receita ficam de cores
    // diferentes.
    if !g.instances.is_empty() {
        g.renderer.render_lifted_instances(
            g.game_rt.view(),
            g.camera,
            g.window_size,
            wgpu::Color::TRANSPARENT,
            g.instances,
            g.scene_viewport,
        );
        g.tonemap.run(gpu);
        g.band_blit.blit(
            gpu,
            g.world_rt.blend_view(),
            g.tonemap.output_view(),
            ph2d_render::BandSource::Sprites,
        );
    }
    // ⭐ **E a receita VECTORIAL**, por último — acima de tudo, nítida.
    vector_band(
        gpu,
        g.vello_pass,
        g.band_blit,
        g.world_rt,
        g.window_size,
        g.front_scene,
    );
}

/// Uma faixa de vetor: rasteriza a cena no intermediário do Vello e cola-a no acumulador.
///
/// ⚠️ **Peça a peça, e não `&Gear`:** o passe do Vello é emprestado MUTÁVEL e o acumulador só
/// lido, e os dois são campos distintos da engrenagem — passar a struct inteira obrigaria a um
/// `&mut` de tudo (ou a um ponteiro cru, que aqui não compraria nada).
fn vector_band(
    gpu: &ph2d_gpu::GpuContext,
    vello_pass: &mut ph2d_render::VelloPass,
    band_blit: &ph2d_render::BandBlit,
    world_rt: &ph2d_render::WorldRt,
    window_size: WindowSize,
    scene: &ph2d_vector::VectorScene,
) {
    let size = (window_size.width, window_size.height);
    if let Err(e) =
        vello_pass.render_to_intermediate(gpu, scene.inner(), size, VelloColor::TRANSPARENT)
    {
        eprintln!("[frost] faixa de vetor falhou: {e}");
    }
    band_blit.blit(
        gpu,
        world_rt.blend_view(),
        vello_pass.intermediate_view(),
        ph2d_render::BandSource::Vector,
    );
}

#[cfg(test)]
#[path = "present_frost_tests.rs"]
mod tests;
