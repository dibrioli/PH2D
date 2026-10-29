//! **Add ▸ Pixel Sprite** — a entrada abre o diálogo *New Sprite* (o *File ▸ New* do Aseprite:
//! Width, Height, Color Mode, Background; por omissão 32 × 32, RGBA, transparente). O OK nasce o
//! objecto com um documento novo e pede o modo Draw; Cancel não cria nada.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_editor_core::pixel_panel::{NewSpriteBackground, NewSpriteChoice, NewSpriteMode};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect, apply};

use crate::PixelApp;

pub const PIXEL_SPRITE: AddEntry = AddEntry::new("object_add.pixel_sprite", AddGroup::TwoD);
pub const ENTRIES: &[AddEntry] = &[PIXEL_SPRITE];

/// **O pick do menu Add**: abre o diálogo *New Sprite* — `false` se a entrada não é desta família.
pub fn open_dialog(
    entry: AddEntry,
    store: &mut ph2d_editor_core::interaction::WidgetStore,
) -> bool {
    if entry != PIXEL_SPRITE {
        return false;
    }
    store.open_new_sprite_dialog();
    true
}

/// **O OK do diálogo**: drena o pedido e cria o objecto com aquelas escolhas — `None` sem pedido.
pub fn born_from_dialog(
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    sim: &mut SimWorld,
    app: &mut PixelApp,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<Result<u64, &'static str>> {
    let c = store.take_new_sprite_request()?;
    Some(spawn(sim, app, new_sprite(c), spawn_root))
}

/// As escolhas do diálogo no vocabulário do documento.
#[must_use]
pub fn new_sprite(c: NewSpriteChoice) -> NewSprite {
    NewSprite {
        width: c.width,
        height: c.height,
        mode: match c.mode {
            NewSpriteMode::Rgba => ColorMode::Rgba,
            NewSpriteMode::Gray => ColorMode::Gray,
            NewSpriteMode::Indexed => ColorMode::Indexed,
        },
        background: match c.background {
            NewSpriteBackground::Transparent => Background::Transparent,
            NewSpriteBackground::White => Background::White,
            NewSpriteBackground::Black => Background::Black,
        },
    }
}

/// **Paste as New Sprite**: drena o recorte pedido e nasce um objecto do tamanho dele, com o modo
/// e a paleta dele, fundo transparente e o recorte em (0, 0) — `None` sem pedido. O documento novo
/// nasce sem passos de desfazer (é o conteúdo do nascimento, como o do Aseprite).
pub fn born_from_paste(
    sim: &mut SimWorld,
    app: &mut PixelApp,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Option<Result<u64, &'static str>> {
    let c = app.nasce.take()?;
    let spec = NewSprite {
        width: c.rect.w,
        height: c.rect.h,
        mode: c.mode,
        background: Background::Transparent,
    };
    let Ok(key) = app.create(spec) else {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    };
    if let Some(doc) = app.docs.get_mut(&key) {
        let s = &mut doc.sprite;
        let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
        let r = Rect::new(0, 0, c.rect.w, c.rect.h);
        let palette = Op::Palette {
            colors: c.palette,
            transparent: c.transparent,
        };
        let patch = Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: c.data,
        });
        let paleta_ok = c.mode != ColorMode::Indexed || apply(s, palette).is_ok();
        if !paleta_ok || apply(s, patch).is_err() {
            app.docs.remove(&key);
            return Some(Err(ph2d_i18n::tr("object_add.not_born")));
        }
    }
    Some(spawn_doc(sim, app, key, spawn_root))
}

/// Cria o objecto desta entrada com os valores por omissão, sem diálogo (o caminho dos testes e do
/// smoke) — `None` se a entrada não é desta família.
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

/// Cria um objecto Pixel Sprite com o documento `spec` e pede o Draw para ele.
pub fn spawn(
    sim: &mut SimWorld,
    app: &mut PixelApp,
    spec: NewSprite,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Result<u64, &'static str> {
    let Ok(key) = app.create(spec) else {
        return Err(ph2d_i18n::tr("object_add.not_born"));
    };
    spawn_doc(sim, app, key, spawn_root)
}

/// O objecto do documento `key` já criado; pede o Draw para ele.
fn spawn_doc(
    sim: &mut SimWorld,
    app: &mut PixelApp,
    key: u32,
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Result<u64, &'static str> {
    let bits = spawn_root(sim, PIXEL_SPRITE.key.tr());
    let ent =
        ph2d_ecs::Entity::try_from_bits(bits).and_then(|e| sim.world_mut().get_entity_mut(e).ok());
    let Some(mut ent) = ent else {
        app.docs.remove(&key);
        return Err(ph2d_i18n::tr("object_add.not_born"));
    };
    ent.insert(ph2d_ecs::PixelSprite(key));
    app.wants_edit = Some(bits);
    Ok(bits)
}
