//! A cena 24 (P11): o botão e o boneco de `ph2d_pixel_cenas::slices`, com a Slice Tool na mão.

use ph2d_ecs::SimWorld;
pub use ph2d_pixel_cenas::slices::*;
use ph2d_pixel_core::{Background, ColorMode, NewSprite};

use crate::{PixelApp, Tool};

pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let spec = NewSprite {
        width: SIZE.0,
        height: SIZE.1,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get_mut(&key)?;
    let (arte, hero) = monta(&mut doc.sprite)?;
    (doc.frame, doc.layer) = (doc.sprite.frames()[0].id, arte);
    app.slices.sel.insert(key, vec![hero]);
    app.tool = Tool::Slice;
    Some(bits)
}
