//! **Add ▸ Pixel Sprite** — o objecto nasce com um documento novo (o *New Sprite* do Aseprite com os
//! valores por omissão: 32 × 32, RGBA, transparente) e pede o modo Edit.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_pixel_core::NewSprite;

use crate::PixelApp;

pub const PIXEL_SPRITE: AddEntry = AddEntry::new("object_add.pixel_sprite", AddGroup::TwoD);
pub const ENTRIES: &[AddEntry] = &[PIXEL_SPRITE];

/// Cria o objecto desta entrada — `None` se a entrada não é desta família.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    app: &mut PixelApp,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<Result<u64, &'static str>> {
    if entry != PIXEL_SPRITE {
        return None;
    }
    Some(spawn(sim, app, NewSprite::default(), spawn_root))
}

/// Cria um objecto Pixel Sprite com o documento `spec` e pede o Edit para ele.
pub fn spawn(
    sim: &mut SimWorld,
    app: &mut PixelApp,
    spec: NewSprite,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Result<u64, &'static str> {
    let Ok(key) = app.create(spec) else {
        return Err(ph2d_i18n::tr("object_add.not_born"));
    };
    let bits = spawn_root(sim, PIXEL_SPRITE.key.tr());
    let ent =
        ph2d_ecs::Entity::try_from_bits(bits).and_then(|e| sim.world_mut().get_entity_mut(e).ok());
    let Some(mut ent) = ent else {
        return Err(ph2d_i18n::tr("object_add.not_born"));
    };
    ent.insert(ph2d_ecs::PixelSprite(key));
    app.wants_edit = Some(bits);
    Ok(bits)
}
