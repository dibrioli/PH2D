//! **O blob do projecto**: os documentos dos Pixel Sprites vivos, no formato `PH2DPIXEL` do
//! `ph2d-pixel-core::codec` (versão própria, dentro do campo do `ProjectFile`).

use std::collections::BTreeMap;

use ph2d_pixel_core::CodecError;
pub use ph2d_pixel_core::codec::Entries;
use ph2d_pixel_core::codec::read_library;

use crate::PixelApp;
use crate::mode::sprites;
use crate::state::Doc;

/// Os bytes a gravar: só os documentos que algum objecto vivo usa (o desfazer do mundo pode ter
/// deixado outros na memória — não vão para o ficheiro). Sem Pixel Sprites = vazio.
///
/// A forma viva e a moldura solta ASSENTAM antes (um passo do desfazer cada): o ficheiro leva o que
/// o artista vê, nunca o documento de antes delas.
///
/// Cada documento reaproveita a entrada da cópia anterior se nada gravável mudou
/// (`ph2d_pixel_core::codec::EntryCache`): os mesmos bytes, sem reescrever os que não mudaram.
#[must_use]
pub fn to_bytes(app: &mut PixelApp, world: &ph2d_ecs::World) -> Vec<u8> {
    entries(app, world).library()
}

/// [`to_bytes`] por juntar: as entradas partilhadas, para o fio da cópia (P12 S) as juntar.
#[must_use]
pub fn entries(app: &mut PixelApp, world: &ph2d_ecs::World) -> Entries {
    crate::viva::assenta(app);
    app.assenta();
    let live: BTreeMap<u32, ()> = sprites(world).into_iter().map(|(_, k)| (k, ())).collect();
    let mut out = Vec::new();
    for (k, d) in &mut app.docs {
        if live.contains_key(k) {
            d.encoded
                .entry(u64::from(*k), d.history.revision(), &d.sprite);
            out.extend(d.encoded.shared());
        } else {
            d.encoded.clear();
        }
    }
    Entries(out)
}

/// A impressão barata dos documentos (chave e revisão de cada um): muda com toda edição.
#[must_use]
pub fn print(app: &PixelApp) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for (k, d) in &app.docs {
        std::hash::Hash::hash(&(k, d.history.revision()), &mut h);
    }
    std::hash::Hasher::finish(&h)
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
    // As células levantadas e o laço eram do projecto que saiu: assentá-los escreveria no novo.
    app.gesture = None;
    app.solta = None;
    app.laco = None;
    app.chave = None;
    app.was_editing = None;
    app.pressed = false;
    app.panning = false;
    app.sobre_eixo = false;
    app.hover = None;
}

#[cfg(test)]
mod tests {
    use ph2d_pixel_core::{Op, codec::write_library};

    /// Cada edição (passo, prefs, desfazer): os bytes com a cache = `write_library` sobre clones.
    #[test]
    fn a_entrada_guardada_e_a_escrita_inteira_a_cada_edicao() {
        let (mut app, mut sim) = (crate::PixelApp::default(), ph2d_ecs::SimWorld::new());
        for _ in 0..2 {
            let key = app.create(Default::default()).expect("sprite");
            (sim.world_mut()).spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::PixelSprite(key)));
        }
        for i in 0..4 {
            let d = app.docs.values_mut().last().expect("doc");
            match i {
                1 => {
                    let op = Op::Palette {
                        colors: vec![[0; 4], [7; 4]],
                        transparent: 0,
                    };
                    d.history.commit(&mut d.sprite, op).expect("passo");
                }
                2 => {
                    let mut p = *d.sprite.prefs();
                    p.grid.w = 3;
                    d.sprite.set_prefs(p);
                }
                3 => assert!(d.history.undo(&mut d.sprite)),
                _ => {}
            }
            let lib = app
                .docs
                .iter()
                .map(|(k, d)| (u64::from(*k), d.sprite.clone()));
            let fresh = write_library(&lib.collect());
            assert_eq!(super::to_bytes(&mut app, sim.world()), fresh, "edição {i}");
        }
    }
}
