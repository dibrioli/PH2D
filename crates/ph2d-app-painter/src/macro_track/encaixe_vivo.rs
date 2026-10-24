//! **O encaixe vivo** (W18, `docs/Macros/16_W18_plano.md` §2.7; decisões do dono 23/10, 5 e 6): uma
//! macro aplicada por cima pinta a imagem de baixo no TAMANHO ORIGINAL, onde o gizmo do objecto Macro
//! está — lido do mundo a cada quadro, nunca assado (mover o objecto muda onde a reprodução pinta).
//!
//! A lei vive na folha ([`ph2d_sprite_screen::gravacao_no_alvo`]): a família MACROS também a lê.

use ph2d_ecs::{Entity, World};
use ph2d_tool_painter::macro_track::encaixe::Encaixe;

/// **O encaixe px-gravado → px-alvo** do objecto Macro `objeto` sobre a sprite `alvo`, cuja tela tem
/// `texels` e a gravação `gravado`. `None` = falta uma pose, a sprite, ou uma base degenerada.
#[must_use]
pub fn do_mundo(
    world: &World,
    objeto: Entity,
    alvo: Entity,
    texels: (u32, u32),
    gravado: (u32, u32),
) -> Option<Encaixe> {
    ph2d_sprite_screen::gravacao_no_alvo::gravacao_no_alvo(world, objeto, alvo, texels, gravado)
        .map(|afim| Encaixe { afim })
}

/// O leitor de píxeis (RGBA directo) de uma sprite — o da shell (`read_sprite_straight`).
pub type LePixeis = fn(
    Entity,
    &ph2d_ecs::SimWorld,
    &mut ph2d_render::SpriteRenderer,
    &ph2d_asset::AssetDb,
    &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<(Vec<u8>, u32, u32)>;

/// A montagem de uma cena do Painter (`super::smoke::monta`…): a tela nasce na célula dada.
pub type MontaTela<T> = fn(
    &mut ph2d_ecs::SimWorld,
    &mut ph2d_render::SpriteRenderer,
    &ph2d_asset::AssetDb,
    u32,
    f32,
    &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<T>;

/// **O atlas da cena para APLICAR uma macro** (W18): a imagem que nasce pela porta de produto
/// (`spawn_rgba`: o `AssetDb` e o atlas — sobrevive ao gravar, a mesma do modal de imagem nova) e os
/// píxeis de uma imagem que o Painter ainda não tem.
pub struct AtlasDaCena<'a> {
    pub renderer: &'a mut ph2d_render::SpriteRenderer,
    pub asset_db: &'a ph2d_asset::AssetDb,
    pub proxima_celula: &'a mut u32,
    pub atlas: &'a mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    /// `None` = sem ecrã ainda (nada nasce).
    pub ppm: Option<f32>,
    pub le: LePixeis,
}

impl AtlasDaCena<'_> {
    /// Nasce a imagem `w×h`; devolve os bits.
    ///
    /// # Errors
    /// O porquê do atlas.
    pub fn nasce(
        &mut self,
        sim: &mut ph2d_ecs::SimWorld,
        (w, h, rgba): (u32, u32, Vec<u8>),
        nome: &str,
    ) -> Result<u64, String> {
        let ppm = self.ppm.ok_or_else(String::new)?;
        let at = ph2d_core::Vec2::new(0.0, 0.0);
        let c = *self.proxima_celula;
        let (_, bits) = ph2d_image_import::spawn_rgba(
            sim,
            self.renderer,
            self.asset_db,
            c,
            w,
            h,
            rgba,
            at,
            ppm,
            self.atlas,
            nome,
        )?;
        *self.proxima_celula = c.saturating_add(1);
        Ok(bits)
    }

    /// Monta uma cena de `PH2D_MACRO_SMOKE` cuja tela nasce na próxima célula do atlas.
    pub fn monta<T>(&mut self, sim: &mut ph2d_ecs::SimWorld, monta: MontaTela<T>) -> Option<T> {
        let c = *self.proxima_celula;
        let r = monta(sim, self.renderer, self.asset_db, c, self.ppm?, self.atlas);
        *self.proxima_celula += u32::from(r.is_some());
        r
    }

    pub fn pixeis(&mut self, sim: &ph2d_ecs::SimWorld, bits: u64) -> Option<(Vec<u8>, u32, u32)> {
        (self.le)(
            Entity::from_bits(bits),
            sim,
            self.renderer,
            self.asset_db,
            self.atlas,
        )
    }
}

/// O objecto mostra píxeis (tem `Sprite`): pode receber uma macro do Painter por cima.
#[must_use]
pub fn e_imagem(sim: &ph2d_ecs::SimWorld, bits: u64) -> bool {
    let e = Entity::from_bits(bits);
    sim.world().get::<ph2d_render::Sprite>(e).is_some()
}

#[cfg(test)]
#[path = "encaixe_vivo_tests.rs"]
mod tests;
