//! ⭐ **Edit in Painter na folha VISÍVEL** (plano GEF 26 F13): com a folha `k ≥ 1` à vista, o Painter abre a imagem
//! DELA e a pintura volta para ela. A folha guarda só o nome da imagem ([`ExtraSheet::asset`]), e a textura refaz-se
//! sozinha pelo cache derivado: gravar é pôr os píxeis na biblioteca e trocar o nome. O desfazer é o do documento
//! (as `SpriteSheets` são registadas).

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, ExtraSheet, SpriteSheets, World, shown_sheet};

/// A folha extra visível de `e`: `None` com a folha `0` à vista.
#[must_use]
pub fn shown_extra(world: &World, e: Entity) -> Option<(u8, ExtraSheet)> {
    let k = shown_sheet(world, e);
    world
        .get::<SpriteSheets>(e)?
        .sheet(k)
        .map(|s| (k, *s))
}

/// Os píxeis RETOS da folha extra visível, `(rgba, w, h)`: o que o Painter abre.
#[must_use]
pub fn read_shown_extra(world: &World, e: Entity, db: &AssetDb) -> Option<(Vec<u8>, u32, u32)> {
    let (_, s) = shown_extra(world, e)?;
    let asset = db.get(&s.asset)?;
    let (w, h, px) = asset.image_rgba8()?;
    Some((px.into_owned(), w, h))
}

/// ⭐ **Grava os píxeis RETOS `rgba` (`w × h`) na folha extra visível de `e`**: a imagem nova entra na biblioteca e a
/// folha passa a ela; se mudou de tamanho, o tamanho na tela segue com o MESMO metro por pixel. Devolve
/// `(antiga, nova)`; `None` com a folha `0` à vista.
pub fn commit_shown_extra(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    rgba: Vec<u8>,
    w: u32,
    h: u32,
) -> Option<(AssetId, AssetId)> {
    let (k, s) = shown_extra(world, e)?;
    let antes = db
        .get(&s.asset)
        .and_then(|a| a.image_rgba8().map(|(w, h, _)| [w, h]));
    let novo = db.insert_image_rgba8(w, h, rgba);
    let mut sheets = world.get::<SpriteSheets>(e)?.clone();
    let x = sheets.0.get_mut(usize::from(k) - 1)?;
    if let Some([w0, h0]) = antes.filter(|p| *p != [w, h] && p[0] > 0 && p[1] > 0) {
        x.size = [
            x.size[0] * w as f32 / w0 as f32,
            x.size[1] * h as f32 / h0 as f32,
        ];
    }
    x.asset = novo;
    world.entity_mut(e).insert(sheets);
    Some((s.asset, novo))
}

#[cfg(test)]
#[path = "shown_image_tests.rs"]
mod tests;
