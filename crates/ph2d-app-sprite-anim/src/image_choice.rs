//! ⭐ **Trocar a imagem de um objecto** ([plano GEF 12](../../../docs/GameEngineFuncional/12_imagem_como_asset_plano.md)):
//! a lei pura da escolha, a mesma por qualquer porta (Inspector, queda, seletor, script).
//!
//! O objecto guarda só QUAL imagem mostra (`Sprite.source` + `SpritePixels`); os pixels são do asset. As
//! quatro regras e o oráculo (Godot 4.7.2, `ferramentas/textura_godot.gd`) estão no plano.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{
    Entity, RepeatMode, ScrollRepeat, SimWorld, SpriteAnimations, SpriteGrid, SpritePixels,
    SpriteRegion, SpriteSheetRef, TextureRepeat, TiledDraw, UvTransform, World,
};
use ph2d_render::{Sprite, SpriteRenderer, SpriteSource};
use std::collections::BTreeMap;

/// O corte de uma imagem em quadros: `[colunas, linhas]` e a célula que um objecto SEM animação mostra
/// (a do meio de um azulejo de fábrica).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slicing {
    pub grid: [u32; 2],
    pub still_frame: u32,
}

/// O que a lei precisa de saber de uma imagem.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageFacts {
    /// O tamanho NATIVO (m) da imagem inteira.
    pub native_m: [f32; 2],
    /// O corte da própria imagem, se ela o traz (a fábrica; um `.ase`).
    pub slicing: Option<Slicing>,
}

/// Onde a sprite vai buscar os pixels da imagem escolhida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSource {
    /// Uma célula do atlas (fábrica ou importada) — a referência partilhada.
    Atlas { key: u32 },
    /// Uma textura própria (16 bits, grande demais para o atlas) e o nome durável dos pixels.
    Individual { texture_id: u32, pixels: AssetId },
}

/// ⭐ **A sprite `e` passa a mostrar a imagem `new`.** `old` são os factos da imagem que ela mostra agora
/// (`None` = desconhecida ⇒ a escala é 1). `false` se `e` não tem `Sprite`.
pub fn choose_image(
    world: &mut World,
    e: Entity,
    old: Option<ImageFacts>,
    source: ImageSource,
    new: ImageFacts,
) -> bool {
    let Some(sprite) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let animated =
        crate::image_use::image_use(world, e) == Some(crate::image_use::ImageUse::Animated);
    let old_grid = world.get::<SpriteGrid>(e).copied();
    let tiled = world.get::<TiledDraw>(e).copied();
    // Lei 2: só uma ANIMADA tem corte — o da imagem, senão o seu; parada e repetida mostram a imagem INTEIRA
    // (plano GEF 15, dono 27/10: «por que trata a imagem como uma folha?»).
    let grid = match (new.slicing, animated, old_grid) {
        (Some(s), true, _) => Some(SpriteGrid {
            hframes: s.grid[0].max(1),
            vframes: s.grid[1].max(1),
            frame: old_grid.map_or(0, |g| g.frame),
        }),
        (None, true, Some(g)) => Some(g),
        _ => None,
    };
    let old_cell = old.map(|f| cell_m(f.native_m, old_grid));
    let new_cell = cell_m(new.native_m, grid);
    // Lei 1: a escala do objecto fica (o tamanho em relação à imagem nativa).
    let escala = match old_cell {
        Some(c) if c[0] > 0.0 && c[1] > 0.0 => {
            let base = match world.get::<ScrollRepeat>(e) {
                Some(r) if r.tile[0] > 0.0 => [r.tile[0], sprite.size[1]],
                _ => sprite.size,
            };
            [base[0] / c[0], base[1] / c[1]]
        }
        _ => [1.0, 1.0],
    };
    let mut size = [new_cell[0] * escala[0], new_cell[1] * escala[1]];
    let repeats = tiled.is_some()
        || matches!(
            world.get::<TextureRepeat>(e),
            Some(TextureRepeat(RepeatMode::Enabled | RepeatMode::Mirror))
        );
    let mut ent = world.entity_mut(e);
    if let Some(t) = tiled {
        // Lei 5 (planos GEF 14 D4 e 15): o ladrilho de um Tiled é a imagem nova INTEIRA (nunca uma célula da folha),
        // à escala a que o bloco mostrava a antiga.
        let k: [f32; 2] = std::array::from_fn(|i| match old_cell {
            Some(c) if c[i] > 0.0 => t.tile[i] / c[i],
            _ => 1.0,
        });
        ent.insert(TiledDraw {
            tile: [new.native_m[0] * k[0], new.native_m[1] * k[1]],
        });
    }
    // O recorte da imagem antiga (região, folha empacotada à mão) não descreve a nova.
    ent.remove::<(SpriteRegion, SpriteSheetRef)>();
    // Nem o aspecto que os outros usos guardaram (plano GEF 15); as animações deles ficam.
    if let Some(mut m) = ent.get_mut::<ph2d_ecs::ImageUseMemory>() {
        m.forget_looks();
    }
    if let Some(tile_antigo) = ent.get::<ScrollRepeat>().map(|r| r.tile) {
        // Lei 3 (Unity *Tiled*): o ladrilho segue a imagem; os ladrilhos cobrem pelo menos a largura antiga.
        let n = (sprite.size[0] / size[0]).ceil().max(1.0);
        let uv_y = ent.get::<UvTransform>().map_or(1.0, |u| u.scale[1]);
        let offset = ent.get::<UvTransform>().map_or([0.0, 0.0], |u| u.offset);
        ent.insert((
            ScrollRepeat {
                tile: [size[0], if tile_antigo[1] > 0.0 { size[1] } else { 0.0 }],
            },
            UvTransform {
                scale: [n, uv_y],
                offset,
            },
        ));
        size[0] *= n;
    } else if repeats {
        // Lei 4: um bloco que repete guarda o tamanho e as repetições.
        size = sprite.size;
    }
    let src = match source {
        ImageSource::Atlas { key } => {
            ent.remove::<SpritePixels>();
            SpriteSource::Atlas { key }
        }
        ImageSource::Individual { texture_id, pixels } => {
            ent.insert(SpritePixels(pixels));
            SpriteSource::Individual { texture_id }
        }
    };
    ent.insert(Sprite {
        source: src,
        size,
        premultiplied: false,
        ..sprite
    });
    match grid {
        Some(g) => {
            ent.insert(g);
        }
        None => {
            ent.remove::<SpriteGrid>();
        }
    }
    true
}

/// O lado (m) de uma célula da imagem com esta grelha.
fn cell_m(native: [f32; 2], grid: Option<SpriteGrid>) -> [f32; 2] {
    let g = grid.unwrap_or(SpriteGrid::SINGLE);
    [
        native[0] / g.hframes.max(1) as f32,
        native[1] / g.vframes.max(1) as f32,
    ]
}

/// Os factos de uma imagem de fábrica.
#[must_use]
pub fn factory_facts(art: &ph2d_render::FactoryArt) -> ImageFacts {
    let cell = art.native_m();
    let grid = art.frames();
    ImageFacts {
        native_m: [cell[0] * grid[0] as f32, cell[1] * grid[1] as f32],
        slicing: (grid[0] * grid[1] > 1).then_some(Slicing {
            grid,
            still_frame: 0,
        }),
    }
}

/// ⭐ O corte de uma imagem importada é o que os objectos que a mostram levam (plano GEF 13 D3: re-cortar num
/// re-corta todos, como no *importer* da Unity) — o primeiro, pela ordem do mundo, com mais de uma célula.
#[must_use]
pub fn cut_of_users(world: &World, id: AssetId, atlas: &BTreeMap<u32, AssetId>) -> Option<Slicing> {
    let mut q = world.try_query::<(Option<&SpritePixels>, &Sprite, &SpriteGrid)>()?;
    q.iter(world)
        .filter(|(_, _, g)| g.hframes.max(1) * g.vframes.max(1) > 1)
        .find(|(px, s, _)| {
            ph2d_app_components::asset_index_build::texture_of(*px, Some(*s), atlas) == Some(id)
        })
        .map(|(_, _, g)| Slicing {
            grid: [g.hframes.max(1), g.vframes.max(1)],
            still_frame: 0,
        })
}

/// Os factos de uma imagem importada de `w × h` px a `ppm` px por metro (sem corte próprio).
#[must_use]
pub fn imported_facts(w: u32, h: u32, ppm: f32) -> ImageFacts {
    let ppm = ppm.max(f32::EPSILON);
    ImageFacts {
        native_m: [w as f32 / ppm, h as f32 / ppm],
        slicing: None,
    }
}

/// Sobe uma textura própria (`w`, `h`, RGBA8) e devolve o id dela.
pub type Acquire<'a> = dyn FnMut(u32, u32, &[u8]) -> Result<u32, String> + 'a;

/// ⭐ Os factos de uma imagem da biblioteca — a UMA consulta (plano GEF 13 D3): a fábrica pela chave, as outras
/// pelos píxeis e o `ppm` do projecto.
#[must_use]
pub fn image_facts(
    id: AssetId,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ImageFacts> {
    if let Some(art) = atlas
        .iter()
        .find(|(_, a)| **a == id)
        .and_then(|(k, _)| ph2d_app_components::factory_images::art_of_key(*k))
    {
        return Some(factory_facts(art));
    }
    let (w, h) = db.get(&id)?.image_dimensions()?;
    Some(imported_facts(w, h, ppm))
}

/// [`image_facts`] com o corte que os utilizadores de uma importada levam ([`cut_of_users`]).
#[must_use]
pub fn library_facts(
    world: &World,
    id: AssetId,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ImageFacts> {
    let mut f = image_facts(id, db, atlas, ppm)?;
    let fabrica = atlas
        .iter()
        .any(|(k, a)| *a == id && ph2d_app_components::factory_images::is_factory_key(*k));
    if f.slicing.is_none() && !fabrica {
        f.slicing = cut_of_users(world, id, atlas);
    }
    Some(f)
}

/// ⭐⭐ **A PORTA: a sprite `e` passa a mostrar a imagem `id` da biblioteca** — sem copiar píxeis: a célula do
/// atlas que já a tem (fábrica ou importada), senão uma textura própria com o nome dos píxeis.
///
/// `acquire` sobe uma textura própria (`w`, `h`, RGBA8) e devolve o id dela — o `SpriteRenderer` no app.
///
/// # Errors
/// O texto para o artista se `id` não é uma imagem, `e` não é uma sprite, ou a placa recusa a textura.
pub fn choose_library_image(
    sim: &mut SimWorld,
    acquire: &mut Acquire<'_>,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    e: Entity,
    id: AssetId,
) -> Result<(), String> {
    let no_image = || ph2d_i18n::tr("app.sprite_anim.image_choice.not_an_image").to_owned();
    let new = library_facts(sim.world(), id, db, atlas, ppm).ok_or_else(no_image)?;
    let w = sim.world();
    let old = ph2d_app_components::asset_index_build::texture_of(
        w.get::<SpritePixels>(e),
        w.get::<Sprite>(e),
        atlas,
    )
    .and_then(|old| image_facts(old, db, atlas, ppm));
    let source = match atlas.iter().find(|(_, a)| **a == id) {
        Some((key, _)) => ImageSource::Atlas { key: *key },
        None => {
            let asset = db.get(&id).ok_or_else(no_image)?;
            let (iw, ih, px) = asset.image_rgba8().ok_or_else(no_image)?;
            let texture_id = acquire(iw, ih, &px)?;
            ImageSource::Individual {
                texture_id,
                pixels: id,
            }
        }
    };
    if choose_image(sim.world_mut(), e, old, source, new) {
        Ok(())
    } else {
        Err(ph2d_i18n::tr("app.sprite_anim.image_choice.not_a_sprite").to_owned())
    }
}

/// ⭐ **A porta do SCRIPT** (`sprite.image`, plano GEF 13 D6): a MESMA lei sobre uma cópia da sprite `e`, sem
/// tocar no documento — o override da corrida leva a célula do atlas, o tamanho e o corte que a troca no editor
/// daria. Num objecto que repete o tamanho fica o autorado (o ladrilho não viaja no override).
#[must_use]
pub fn override_law(
    world: &World,
    e: Entity,
    id: AssetId,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ph2d_app_components::script_image::OverrideLaw> {
    let new = library_facts(world, id, db, atlas, ppm)?;
    let sprite = *world.get::<Sprite>(e)?;
    let old = ph2d_app_components::asset_index_build::texture_of(
        world.get::<SpritePixels>(e),
        Some(&sprite),
        atlas,
    )
    .and_then(|old| image_facts(old, db, atlas, ppm));
    let mut copia = World::new();
    let c = copia.spawn(sprite).id();
    let mut ce = copia.entity_mut(c);
    if let Some(g) = world.get::<SpriteGrid>(e) {
        ce.insert(*g);
    }
    if let Some(a) = world.get::<SpriteAnimations>(e) {
        ce.insert(a.clone());
    }
    if let Some(r) = world.get::<ScrollRepeat>(e) {
        ce.insert(*r);
    }
    if let Some(u) = world.get::<UvTransform>(e) {
        ce.insert(*u);
    }
    if let Some(r) = world.get::<TextureRepeat>(e) {
        ce.insert(*r);
    }
    let atlas_key = atlas.iter().find(|(_, a)| **a == id).map(|(k, _)| *k);
    let source = match atlas_key {
        Some(key) => ImageSource::Atlas { key },
        None => ImageSource::Individual {
            texture_id: 0,
            pixels: id,
        },
    };
    choose_image(&mut copia, c, old, source, new);
    let repete =
        crate::image_use::image_use(world, e) == Some(crate::image_use::ImageUse::Repeated);
    Some(ph2d_app_components::script_image::OverrideLaw {
        atlas_key,
        size: if repete {
            sprite.size
        } else {
            copia.get::<Sprite>(c)?.size
        },
        grid: copia
            .get::<SpriteGrid>(c)
            .copied()
            .unwrap_or(SpriteGrid::SINGLE),
    })
}

/// ⭐ **Uma sprite nova com a imagem `id` da biblioteca, no ponto `at`** — a queda no canvas. Uma sprite parada
/// nova e a MESMA porta da troca ([`choose_library_image`]): a referência do atlas, o tamanho nativo de uma célula.
///
/// # Errors
/// O texto para o artista se `id` não é uma imagem, ou a placa recusa a textura (a sprite não fica).
#[allow(clippy::too_many_arguments)]
pub fn place_library_image(
    sim: &mut SimWorld,
    acquire: &mut Acquire<'_>,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    at: [f32; 2],
    base: &str,
    id: AssetId,
) -> Result<Entity, String> {
    // O objecto chama-se como a imagem na biblioteca (a Unity faz o mesmo); `base` se ela não tem nome.
    let nome = ph2d_app_components::library_names::session_library(sim, db, atlas)
        .remove(&id)
        .unwrap_or_else(|| base.to_owned());
    let (_, bits) = ph2d_image_import::spawn_sprite(
        sim,
        ph2d_image_import::PackedSource::Atlas {
            cell_idx: ph2d_render::WHITE_TILE_KEY,
        },
        ph2d_core::Vec2::new(at[0], at[1]),
        [1.0, 1.0],
        &nome,
    );
    let e = Entity::from_bits(bits);
    if let Err(err) = choose_library_image(sim, acquire, db, atlas, ppm, e, id) {
        sim.world_mut().despawn(e);
        return Err(err);
    }
    // Um azulejo de fábrica nasce Tiled (plano GEF 14 D4): o mesmo tipo de objecto dos blocos do nível.
    let azulejo = atlas
        .iter()
        .find(|(_, a)| **a == id)
        .is_some_and(|(key, _)| {
            ph2d_render::FACTORY_ART
                .iter()
                .any(|art| art.key == *key && art.tile)
        });
    let w = sim.world_mut();
    if let Some(size) = w.get::<Sprite>(e).map(|s| s.size).filter(|_| azulejo) {
        w.entity_mut(e).insert(TiledDraw { tile: size });
    }
    Ok(e)
}

/// ⭐ **Traz um ficheiro de imagem para a biblioteca do projecto, sem criar objecto** (o *Import* da Unity):
/// os píxeis no `db` (com o caminho, que dá o nome), uma célula do atlas e o par `chave → AssetId` que o save
/// grava. Idempotente por conteúdo: a mesma imagem não ocupa duas células.
///
/// # Errors
/// O ficheiro não abre, não é imagem, ou o atlas não tem lugar.
pub fn bring_into_library(
    db: &AssetDb,
    renderer: &mut SpriteRenderer,
    next_cell: &mut u32,
    atlas: &mut BTreeMap<u32, AssetId>,
    path: &std::path::Path,
) -> Result<AssetId, String> {
    let id = db.load_png_path(path).map_err(|e| e.to_string())?;
    if atlas.values().any(|a| *a == id) {
        return Ok(id);
    }
    let asset = db.get(&id).ok_or_else(|| path.display().to_string())?;
    let (w, h, px) = asset
        .image_rgba8()
        .ok_or_else(|| path.display().to_string())?;
    let key = *next_cell;
    atlas.insert(key, id);
    let fetch = |k: u32| -> Option<Vec<u8>> {
        let a = db.get(atlas.get(&k)?)?;
        a.image_rgba8().map(|(_, _, p)| p.into_owned())
    };
    if let Err(e) = renderer.insert_atlas_sprite_with_regrow(key, w, h, &px, fetch) {
        atlas.remove(&key);
        return Err(e.to_string());
    }
    *next_cell = key.saturating_add(1);
    Ok(id)
}

#[cfg(test)]
#[path = "image_choice_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "image_choice_gate_tests.rs"]
mod gate_tests;
