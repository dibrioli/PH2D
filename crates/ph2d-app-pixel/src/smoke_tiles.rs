//! Cena 21 (P10b): uma camada de tilemap já desenhada, com o painel Tilemap aberto.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::NewSprite;
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_tiles::exemplo::{self, LADO, MAPA};

use crate::PixelApp;

pub fn stage(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let spec = NewSprite {
        width: u32::from(LADO) * MAPA[0].len() as u32,
        height: u32::from(LADO) * MAPA.len() as u32,
        ..NewSprite::default()
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get_mut(&key)?;
    let (acima, f) = (doc.layer, doc.frame);
    let (op, l, set) = new_tilemap_layer(&mut doc.sprite, acima, LADO, LADO)?;
    app.commit(key, op).ok()?;
    app.docs.get_mut(&key)?.layer = l;
    let op = exemplo::tiles(&app.docs.get(&key)?.sprite, set)?;
    app.commit(key, op).ok()?;
    for r in 0..MAPA.len() {
        for c in 0..MAPA[r].len() {
            let op = exemplo::celula(&app.docs.get(&key)?.sprite, l, f, (c, r));
            if let Some(op) = op {
                app.commit(key, op).ok()?;
            }
        }
    }
    Some(bits)
}
