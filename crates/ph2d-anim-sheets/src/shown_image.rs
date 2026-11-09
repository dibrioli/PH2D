//! ⭐ **Edit in Painter na folha VISÍVEL** (plano GEF 26 F13): com a folha `k ≥ 1` à vista, o Painter abre a imagem
//! DELA. A folha guarda só o nome da imagem ([`ExtraSheet::asset`]), e a textura refaz-se sozinha pelo cache derivado;
//! a pintura volta pelo funil de todas as ferramentas de imagem ([`crate::shown_edit`]).

use ph2d_asset::AssetDb;
use ph2d_ecs::{Entity, ExtraSheet, SpriteSheets, World, shown_sheet};

/// A folha extra visível de `e`: `None` com a folha `0` à vista.
#[must_use]
pub fn shown_extra(world: &World, e: Entity) -> Option<(u8, ExtraSheet)> {
    let k = shown_sheet(world, e);
    world.get::<SpriteSheets>(e)?.sheet(k).map(|s| (k, *s))
}

/// Os píxeis RETOS da folha extra visível, `(rgba, w, h)`: o que o Painter abre.
#[must_use]
pub fn read_shown_extra(world: &World, e: Entity, db: &AssetDb) -> Option<(Vec<u8>, u32, u32)> {
    let (_, s) = shown_extra(world, e)?;
    let asset = db.get(&s.asset)?;
    let (w, h, px) = asset.image_rgba8()?;
    Some((px.into_owned(), w, h))
}

#[cfg(test)]
#[path = "shown_image_tests.rs"]
mod tests;

/// ⭐ **No modo Edit a imagem abre inteira** (dono 08/11: «não faz sentido utilizar as operações do modo Edit numa
/// sprite pronta»): a selecionada mostra-se como um Still, a folha visível inteira, sem corte nem ladrilhos, como no
/// Painter (`ph2d_sprite_screen::imagem_aberta`). É uma vista: o documento, o modo Object e o Play não a veem.
/// `mode_on` = o Edit de uma imagem está ligado. `(objeto, píxeis da imagem da folha 0)`.
#[must_use]
pub fn em_edit(
    world: &ph2d_ecs::World,
    (mode_on, selection): (bool, Option<u64>),
    db: &ph2d_asset::AssetDb,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<(ph2d_ecs::Entity, (u32, u32))> {
    let e = ph2d_ecs::Entity::try_from_bits(selection.filter(|_| mode_on)?)?;
    world.get::<ph2d_render::Sprite>(e)?;
    let px = ph2d_render::texture_of_entity(world, e, atlas)
        .and_then(|id| db.get(&id))
        .and_then(|a| match &*a {
            ph2d_asset::Asset::ImageRgba8 { width, height, .. }
            | ph2d_asset::Asset::ImageRgba16 { width, height, .. } => Some((*width, *height)),
            _ => None,
        });
    Some((e, px.unwrap_or((0, 0))))
}
