//! ⭐ **A imagem de uma sprite trocada por um script a correr** — `ph2d.set(e, "sprite.image",
//! "door_open")` (doc CodeEditor 08 §5 d4). É um OVERRIDE da corrida, nunca o documento:
//!
//! - [`ImageOverride`] não é registado nem serializado — não entra no `Ctrl+Z`, no gravar, nem no
//!   livro da corrida (`FieldDrive`); o rebobinar ([`rewind`]) tira-os todos e a imagem do artista volta.
//! - O nome é o da biblioteca ([`crate::library_names`]); `nil` (ou o nome da imagem autorada) tira o
//!   override.
//! - A textura sobe uma vez por imagem ([`upload`], na fase do quadro que tem o renderer) e vive até
//!   ao fim da corrida: o rebobinar devolve-as todas. O tecto é a própria biblioteca — uma textura por
//!   imagem DISTINTA que um script nomeou nessa corrida, `w × h × 4` bytes (+⅓ das mips) cada.
//! - O quad continua do tamanho autorado (`Sprite.size`), como o largar de uma imagem no editor.

use crate::asset_index_build::texture_of;
use bevy_ecs::prelude::{Component, World};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_component_desc::FieldKind;
use ph2d_ecs::scene::property::PropertyError;
use ph2d_ecs::{Entity, SimWorld, SpriteGrid, SpritePixels, SpriteRegion};
use ph2d_render::{Sprite, SpriteRenderer};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O nome da propriedade (`ph2d_script::api_table::RUNTIME_PROPERTIES`).
pub const NAME: &str = "sprite.image";

/// ⭐ A imagem que um script pôs nesta sprite. `texture_id == 0` = ainda por subir.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageOverride {
    pub asset: AssetId,
    pub texture_id: u32,
    pub premultiplied: bool,
    pub dims: (u32, u32),
}

/// Os nomes da biblioteca que o script vê neste quadro — calculados só se um script os pedir.
pub struct ImageLibrary<'a> {
    source: Option<(&'a AssetDb, &'a BTreeMap<u32, AssetId>)>,
    names: Option<BTreeMap<AssetId, String>>,
}

impl<'a> ImageLibrary<'a> {
    /// Sem biblioteca (uma cena de teste sem imagens).
    #[must_use]
    pub const fn none() -> Self {
        Self {
            source: None,
            names: None,
        }
    }

    /// A biblioteca do projecto: o `AssetDb` e a proveniência do átlas.
    #[must_use]
    pub const fn of(db: &'a AssetDb, atlas: &'a BTreeMap<u32, AssetId>) -> Self {
        Self {
            source: Some((db, atlas)),
            names: None,
        }
    }

    /// Nomes fixos (os testes).
    #[must_use]
    pub const fn fixed(names: BTreeMap<AssetId, String>) -> Self {
        Self {
            source: None,
            names: Some(names),
        }
    }

    fn names(&mut self, sim: &mut SimWorld) -> &BTreeMap<AssetId, String> {
        let source = self.source;
        self.names.get_or_insert_with(|| match source {
            Some((db, atlas)) => crate::library_names::session_library(sim, db, atlas),
            None => BTreeMap::new(),
        })
    }

    /// A imagem autorada desta sprite (a mesma porta do painel Assets).
    fn authored(&self, sim: &SimWorld, entity: Entity) -> Option<AssetId> {
        let w = sim.world();
        let empty = BTreeMap::new();
        let atlas = self.source.map_or(&empty, |(_, a)| a);
        texture_of(
            w.get::<SpritePixels>(entity),
            w.get::<Sprite>(entity),
            atlas,
        )
    }
}

fn no_sprite() -> String {
    crate::script_properties::says(PropertyError::Missing {
        component: "sprite".into(),
    })
}

/// `ph2d.get(e, "sprite.image")`: o nome do override, senão o da imagem autorada, senão `nil`.
pub(crate) fn get(
    sim: &mut SimWorld,
    lib: &mut ImageLibrary<'_>,
    entity: Entity,
) -> Result<(Value, FieldKind), String> {
    if sim.world().get::<Sprite>(entity).is_none() {
        return Err(no_sprite());
    }
    let id = match sim.world().get::<ImageOverride>(entity) {
        Some(o) => Some(o.asset),
        None => lib.authored(sim, entity),
    };
    let v = id
        .and_then(|id| lib.names(sim).get(&id).cloned())
        .map_or(Value::Null, Value::from);
    Ok((v, FieldKind::Text))
}

/// `ph2d.set(e, "sprite.image", nome | nil)`; `dry` só verifica.
pub(crate) fn set(
    sim: &mut SimWorld,
    lib: &mut ImageLibrary<'_>,
    entity: Entity,
    value: &Value,
    dry: bool,
) -> Result<(), String> {
    if sim.world().get::<Sprite>(entity).is_none() {
        return Err(no_sprite());
    }
    let asset = match value {
        Value::Null => None,
        Value::String(s) => Some(
            lib.names(sim)
                .iter()
                .find(|(_, n)| *n == s)
                .map(|(id, _)| *id)
                .ok_or_else(|| {
                    ph2d_i18n::tr("app.components.script_property.no_image_named")
                        .replace("{name}", s)
                })?,
        ),
        _ => {
            return Err(crate::script_properties::says(PropertyError::WrongType {
                name: NAME.to_owned(),
                kind: FieldKind::Text,
            }));
        }
    };
    if dry {
        return Ok(());
    }
    // A imagem do artista não é override: escrevê-la é voltar a ela.
    let asset = asset.filter(|a| lib.authored(sim, entity) != Some(*a));
    let mut em = sim.world_mut().entity_mut(entity);
    match asset {
        None => {
            em.remove::<ImageOverride>();
        }
        Some(a) if em.get::<ImageOverride>().map(|o| o.asset) != Some(a) => {
            em.insert(ImageOverride {
                asset: a,
                texture_id: 0,
                premultiplied: false,
                dims: (0, 0),
            });
        }
        Some(_) => {}
    }
    Ok(())
}

/// ⭐ O que a sprite desenha: o override (já subido) pinta a imagem INTEIRA — sem região, sem a célula
/// da grelha nem a folha aberta. Sob a pré-visualização de uma ferramenta (`previewing`), a
/// ferramenta ganha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteCut {
    pub image: Option<ImageOverride>,
    pub region: Option<SpriteRegion>,
    pub grid: SpriteGrid,
    pub sheet_grid: Option<SpriteGrid>,
}

/// O [`SpriteCut`] desta entidade, para o extract.
#[must_use]
pub fn cut(world: &World, entity: Entity, previewing: bool) -> SpriteCut {
    let image = world
        .get::<ImageOverride>(entity)
        .filter(|o| o.texture_id != 0 && !previewing)
        .copied();
    let authored = image.is_none();
    let sheet_grid = world
        .get::<SpriteGrid>(entity)
        .copied()
        .filter(|_| authored);
    SpriteCut {
        image,
        region: world
            .get::<SpriteRegion>(entity)
            .copied()
            .filter(|_| authored),
        grid: sheet_grid.unwrap_or(SpriteGrid::SINGLE),
        sheet_grid,
    }
}

#[derive(Default)]
struct Textures {
    by_asset: BTreeMap<AssetId, (u32, (u32, u32))>,
    release: bool,
}

thread_local! {
    /// As texturas subidas nesta corrida, por imagem (a chave é o conteúdo).
    static TEXTURES: RefCell<Textures> = RefCell::new(Textures::default());
}

/// ⭐ **Uma vez por quadro, antes do extract**: devolve as texturas de uma corrida que acabou e sobe as
/// imagens que um script acabou de pôr (uma vez por imagem).
pub fn upload(renderer: &mut SpriteRenderer, db: &AssetDb, sim: &mut SimWorld) {
    TEXTURES.with(|t| {
        let mut t = t.borrow_mut();
        if std::mem::take(&mut t.release) {
            for (_, (id, _)) in std::mem::take(&mut t.by_asset) {
                renderer.individual_mut().release(id);
            }
        }
        let w = sim.world_mut();
        let mut q = w.query::<&mut ImageOverride>();
        for mut o in q.iter_mut(w) {
            if o.texture_id != 0 {
                continue;
            }
            let up = match t.by_asset.get(&o.asset) {
                Some(up) => *up,
                None => {
                    let Some(asset) = db.get(&o.asset) else {
                        continue;
                    };
                    let Some((iw, ih, px)) = asset.image_rgba8() else {
                        continue;
                    };
                    let Ok(id) = renderer.acquire_individual(iw, ih, &px) else {
                        continue;
                    };
                    t.by_asset.insert(o.asset, (id, (iw, ih)));
                    (id, (iw, ih))
                }
            };
            o.texture_id = up.0;
            o.dims = up.1;
        }
    });
}

/// O rebobinar: a imagem do artista volta, e as texturas da corrida devolvem-se no próximo [`upload`].
pub(crate) fn rewind(sim: &mut SimWorld) {
    let w = sim.world_mut();
    let alvos: Vec<Entity> = w
        .query_filtered::<Entity, bevy_ecs::prelude::With<ImageOverride>>()
        .iter(w)
        .collect();
    for e in alvos {
        w.entity_mut(e).remove::<ImageOverride>();
    }
    TEXTURES.with(|t| {
        let mut t = t.borrow_mut();
        t.release = !t.by_asset.is_empty();
    });
}

#[cfg(test)]
#[path = "script_image_tests.rs"]
mod tests;
