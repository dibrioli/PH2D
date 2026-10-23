//! **O encaixe vivo** (W18, `docs/Macros/16_W18_plano.md` §2.7; decisões do dono 23/10, 5 e 6): uma
//! macro aplicada por cima pinta a imagem de baixo no TAMANHO ORIGINAL, onde o gizmo do objecto Macro
//! está — lido do mundo a cada quadro, nunca assado (mover o objecto muda onde a reprodução pinta).
//!
//! ```text
//! px gravado ─(centrado na origem do objecto Macro, 1 px = 1/ppm, y para cima)→ local do objecto Macro
//!            ─(o mundo dele)→ mundo ─(o inverso do mundo da sprite)→ local da sprite
//!            ─(a lei do picking: anchor, size, v para baixo)→ texel do alvo
//! ```
//! `ppm` = os texels por unidade local da sprite-alvo (`W / size.x`): uma gravação encaixada na pose do
//! alvo cai texel a texel, sem esticar.

use ph2d_ecs::{Entity, GlobalTransform, World};
use ph2d_tool_painter::macro_track::encaixe::Encaixe;

/// Afim 2×3 `[a, b, c, d, e, f]`: `x' = a·x + b·y + c`, `y' = d·x + e·y + f` (o idioma do [`Encaixe`]).
type Afim = [f64; 6];

fn compoe(p: Afim, q: Afim) -> Afim {
    // p ∘ q
    [
        p[0] * q[0] + p[1] * q[3],
        p[0] * q[1] + p[1] * q[4],
        p[0] * q[2] + p[1] * q[5] + p[2],
        p[3] * q[0] + p[4] * q[3],
        p[3] * q[1] + p[4] * q[4],
        p[3] * q[2] + p[4] * q[5] + p[5],
    ]
}

fn inverso(p: Afim) -> Option<Afim> {
    let det = p[0] * p[4] - p[1] * p[3];
    if det.abs() < 1e-12 || !det.is_finite() {
        return None;
    }
    let (a, b, d, e) = (p[4] / det, -p[1] / det, -p[3] / det, p[0] / det);
    Some([a, b, -(a * p[2] + b * p[5]), d, e, -(d * p[2] + e * p[5])])
}

/// O mundo de `e` como afim (a MESMA base do `GlobalTransform` que o render e o picking usam).
fn mundo(world: &World, e: Entity) -> Option<Afim> {
    let t = ph2d_ecs::world_transform(world, e)?;
    let [a, b, c, d, tx, ty] = GlobalTransform::from_transform(t).affine().map(f64::from);
    Some([a, c, tx, b, d, ty])
}

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
    let s = world.get::<ph2d_render::Sprite>(alvo)?;
    let (sw, sh) = (f64::from(s.size[0]), f64::from(s.size[1]));
    let (w, h) = (f64::from(texels.0), f64::from(texels.1));
    if sw <= 0.0 || sh <= 0.0 || w <= 0.0 || h <= 0.0 {
        return None;
    }
    let (ax, ay) = (f64::from(s.anchor[0]), f64::from(s.anchor[1]));
    let ppm = w / sw;
    let (w0, h0) = (f64::from(gravado.0), f64::from(gravado.1));
    let gravacao = [
        1.0 / ppm,
        0.0,
        -w0 / (2.0 * ppm),
        0.0,
        -1.0 / ppm,
        h0 / (2.0 * ppm),
    ];
    let texel = [
        w / sw,
        0.0,
        (0.5 - ax / sw) * w,
        0.0,
        -h / sh,
        (0.5 + ay / sh) * h,
    ];
    let m = mundo(world, objeto)?;
    let t = inverso(mundo(world, alvo)?)?;
    let afim = compoe(texel, compoe(t, compoe(m, gravacao)));
    afim.iter()
        .all(|x| x.is_finite())
        .then_some(Encaixe { afim })
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
