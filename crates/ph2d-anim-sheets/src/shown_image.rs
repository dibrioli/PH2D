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
