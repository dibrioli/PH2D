//! ⭐ **O cartão do Pixel Sprite no Inspector** ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md),
//! dono 06/11): a família publica no store, a cada quadro, a miniatura da seleção e de cada objecto preso. O `Arc` só
//! muda quando o desenho muda: é ele a chave da textura do painel, e sem isto a placa recebia os bytes todo quadro.

use std::collections::BTreeMap;
use std::sync::Arc;

use ph2d_editor_core::ids::INSP_PANEL;
use ph2d_editor_core::pixel_panel::PixelSpriteCard;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_pixel_core::FrameId;

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
        let (rgba, lado) = ph2d_pixel_io::frame_rgba_reduzido(&doc.sprite, frame, LADO_MAX);
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
        doc: key,
        pivots: doc
            .sprite
            .slices_by_name()
            .into_iter()
            .filter(|c| c.keys.iter().any(|k| k.shape.pivot.is_some()))
            .map(|c| (c.id.0, c.name.clone()))
            .collect(),
        pivot: doc.sprite.pivot_slice().map(|s| s.0),
        luz_na_paleta: doc.sprite.ramps().luz_na_paleta,
    })
}
