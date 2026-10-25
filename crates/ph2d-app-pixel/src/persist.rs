//! **O blob do projecto**: os documentos dos Pixel Sprites vivos, no formato `PH2DPIXEL` do
//! `ph2d-pixel-core::codec` (versão própria, dentro do campo do `ProjectFile`).

use std::collections::BTreeMap;

use ph2d_pixel_core::CodecError;
use ph2d_pixel_core::codec::{read_library, write_library};

use crate::PixelApp;
use crate::mode::sprites;
use crate::state::Doc;

/// Os bytes a gravar: só os documentos que algum objecto vivo usa (o desfazer do mundo pode ter
/// deixado outros na memória — não vão para o ficheiro). Sem Pixel Sprites = vazio.
#[must_use]
pub fn to_bytes(app: &PixelApp, world: &ph2d_ecs::World) -> Vec<u8> {
    let live: BTreeMap<u32, ()> = sprites(world).into_iter().map(|(_, k)| (k, ())).collect();
    if live.is_empty() {
        return Vec::new();
    }
    let lib = app
        .docs
        .iter()
        .filter(|(k, _)| live.contains_key(k))
        .map(|(k, d)| (u64::from(*k), d.sprite.clone()))
        .collect();
    write_library(&lib)
}

/// Os documentos lidos, ainda por instalar — ler e instalar são dois passos para o load recusar o
/// projecto inteiro ANTES de tocar em qualquer coisa.
pub struct Loaded(BTreeMap<u32, ph2d_pixel_core::Sprite>);

/// Lê o blob de um projecto. Vazio = nenhum Pixel Sprite.
pub fn read(bytes: &[u8]) -> Result<Loaded, CodecError> {
    let mut out = BTreeMap::new();
    for (k, sprite) in read_library(bytes)? {
        let key = u32::try_from(k).map_err(|_| CodecError::Invalid("entry.key"))?;
        out.insert(key, sprite);
    }
    Ok(Loaded(out))
}

/// Instala os documentos lidos (o desfazer de cada um recomeça vazio).
pub fn install(app: &mut PixelApp, loaded: Loaded) {
    let mut docs = BTreeMap::new();
    for (key, sprite) in loaded.0 {
        let stamp = app.stamp();
        let mut doc = Doc::new(sprite, stamp);
        doc.fresh = false;
        docs.insert(key, doc);
    }
    app.docs = docs;
    app.gesture = None;
    app.hover = None;
}
