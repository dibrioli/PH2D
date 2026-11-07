//! Cena 21 (P10b): uma camada de tilemap já desenhada, com o painel Tilemap aberto.
//! Cena 22 (P10c): ilhas pintadas com um terreno desenhado, sobre água, com o pincel de terreno na mão.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::NewSprite;
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_tiles::exemplo::{self, ILHAS, LADO, MAPA};
use ph2d_pixel_tiles::mao::Alvo;
use ph2d_pixel_tiles::terreno;

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
    for (r, linha) in MAPA.iter().enumerate() {
        for c in 0..linha.len() {
            let op = exemplo::celula(&app.docs.get(&key)?.sprite, l, f, (c, r));
            if let Some(op) = op {
                app.commit(key, op).ok()?;
            }
        }
    }
    Some(bits)
}

pub fn stage_terreno(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let spec = NewSprite {
        width: u32::from(LADO) * ILHAS[0].len() as u32,
        height: u32::from(LADO) * ILHAS.len() as u32,
        ..NewSprite::default()
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get(&key)?;
    let (acima, f) = (doc.layer, doc.frame);
    let op = exemplo::agua(&doc.sprite, acima, f)?;
    app.commit(key, op).ok()?;
    let doc = app.docs.get_mut(&key)?;
    let (op, l, set) = new_tilemap_layer(&mut doc.sprite, acima, LADO, LADO)?;
    app.commit(key, op).ok()?;
    let doc = app.docs.get_mut(&key)?;
    doc.layer = l;
    let nome = ph2d_i18n::tr_with("pixel.tiles.terrain_name", &[("n", &1.to_string())]);
    let op = exemplo::terreno_ilhas(&doc.sprite, set, &nome)?;
    app.commit(key, op).ok()?;
    let op = terreno::pinta(
        &app.docs.get(&key)?.sprite,
        l,
        f,
        &exemplo::celulas_ilhas(),
        Some(0),
    )?;
    app.commit(key, op).ok()?;
    app.tiles.mao.alvo = Alvo::Terreno;
    Some(bits)
}
