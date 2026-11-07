//! Cena 21 (P10b): uma camada de tilemap já desenhada, com o painel Tilemap aberto.
//! Cena 22 (P10c): um mapa real pintado com um terreno, com o pincel de terreno na mão.

use ph2d_ecs::SimWorld;
use ph2d_pixel_core::NewSprite;
use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_tiles::exemplo::{self, LADO, MAPA, MAPA_REAL};
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

/// A cor do guia da cena 22 (o verde da DB32).
const VERDE: usize = 10;

pub fn stage_terreno(
    app: &mut PixelApp,
    sim: &mut SimWorld,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<u64> {
    let spec = NewSprite {
        width: u32::from(LADO) * MAPA_REAL[0].len() as u32,
        height: u32::from(LADO) * MAPA_REAL.len() as u32,
        ..NewSprite::default()
    };
    let bits = crate::object_add::spawn(sim, app, spec, spawn_root).ok()?;
    let key = crate::input::key_of(sim.world(), bits)?;
    let doc = app.docs.get_mut(&key)?;
    let (acima, f) = (doc.layer, doc.frame);
    let (op, l, set) = new_tilemap_layer(&mut doc.sprite, acima, LADO, LADO)?;
    app.commit(key, op).ok()?;
    let doc = app.docs.get_mut(&key)?;
    doc.layer = l;
    let s = &doc.sprite;
    let cor = s.value_of_rgba(*s.palette().get(VERDE)?);
    let nome = ph2d_i18n::tr_with("pixel.tiles.terrain_name", &[("n", &1.to_string())]);
    let op = terreno::novo_terreno_com_guia(s, set, &nome, cor)?;
    app.commit(key, op).ok()?;
    let op = terreno::pinta(
        &app.docs.get(&key)?.sprite,
        l,
        f,
        &exemplo::celulas_reais(),
        Some(0),
    )?;
    app.commit(key, op).ok()?;
    app.tiles.mao.alvo = Alvo::Terreno;
    Some(bits)
}
