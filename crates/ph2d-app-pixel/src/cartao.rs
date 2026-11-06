//! ⭐ **O cartão do Pixel Sprite no Inspector** ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md),
//! dono 06/11): a família publica no store, a cada quadro, a miniatura da seleção e de cada objecto preso. O `Arc` só
//! muda quando o desenho muda: é ele a chave da textura do painel, e sem isto a placa recebia os bytes todo quadro.

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_editor_core::ids::INSP_PANEL;
use ph2d_editor_core::pixel_panel::PixelSpriteCard;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::{FrameId, compose_rgba};

use crate::PixelApp;

/// O maior lado da miniatura guardada. O recurso é a memória: 1 MiB por cartão, e a moldura mostra 72 px.
const LADO_MAX: u32 = 512;

/// A última composição de um documento e a versão dele.
pub(crate) struct Miniatura {
    stamp: u64,
    frame: FrameId,
    rgba: Arc<Vec<u8>>,
    lado: [u32; 2],
}

pub(crate) type Miniaturas = BTreeMap<u32, Miniatura>;

/// Publica os cartões dos Pixel Sprites à vista. Em Draw a miniatura espera: refazê-la a cada passo do traço custaria
/// a composição inteira, e o Inspector só volta ao sair.
pub(crate) fn publica(
    app: &mut PixelApp,
    world: &ph2d_ecs::World,
    hero: &mut HeroScreen,
    editing: Option<u64>,
) {
    let mut alvos = hero.store.loose_pins_of(INSP_PANEL);
    alvos.extend(hero.gizmo.selection);
    alvos.sort_unstable();
    alvos.dedup();
    let mut vivas = Vec::new();
    let mut cartoes = Vec::new();
    for bits in alvos {
        let Some(key) = crate::input::key_of(world, bits) else {
            continue;
        };
        if let Some(c) = cartao(app, key, bits, editing == Some(bits)) {
            vivas.push(key);
            cartoes.push(c);
        }
    }
    app.miniaturas.retain(|k, _| vivas.contains(k));
    hero.store.set_pixel_sprite_cards(cartoes);
}

fn cartao(app: &mut PixelApp, key: u32, bits: u64, em_draw: bool) -> Option<PixelSpriteCard> {
    let doc = app.docs.get(&key)?;
    let (stamp, frame) = (doc.stamp, doc.frame);
    let fresca = app
        .miniaturas
        .get(&key)
        .is_some_and(|m| em_draw || (m.stamp == stamp && m.frame == frame));
    if !fresca {
        let (rgba, lado) = compoe(&doc.sprite, frame);
        let m = Miniatura {
            stamp,
            frame,
            rgba: Arc::new(rgba),
            lado,
        };
        app.miniaturas.insert(key, m);
    }
    let m = app.miniaturas.get(&key)?;
    Some(PixelSpriteCard {
        entity_bits: bits,
        size: [doc.sprite.width(), doc.sprite.height()],
        rgba: Arc::clone(&m.rgba),
        thumb: m.lado,
    })
}

/// O quadro `frame` achatado nas camadas visíveis e reduzido por vizinho, com o aspecto, se passa de [`LADO_MAX`].
fn compoe(sprite: &ph2d_pixel_core::Sprite, frame: FrameId) -> (Vec<u8>, [u32; 2]) {
    let mut cheio = Vec::new();
    compose_rgba(sprite, frame, None, &mut cheio);
    let (w, h) = (sprite.width(), sprite.height());
    let maior = w.max(h);
    if maior <= LADO_MAX {
        return (cheio, [w, h]);
    }
    let reduz = |n: u32| (u64::from(n) * u64::from(LADO_MAX) / u64::from(maior)).max(1) as u32;
    let (tw, th) = (reduz(w), reduz(h));
    let centro = |i: u32, de: u32, para: u32| {
        ((u64::from(i) * 2 + 1) * u64::from(de) / (u64::from(para) * 2)) as usize
    };
    let mut out = Vec::with_capacity(tw as usize * th as usize * 4);
    for y in 0..th {
        let linha = centro(y, h, th) * w as usize;
        for x in 0..tw {
            let i = (linha + centro(x, w, tw)) * 4;
            out.extend_from_slice(&cheio[i..i + 4]);
        }
    }
    (out, [tw, th])
}

#[cfg(test)]
#[path = "cartao_tests.rs"]
mod tests;
