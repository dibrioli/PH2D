//! **As folhas extra de um Animated Sprite no ecrã** (plano GEF 26 §3.1). O documento guarda só o nome de cada
//! imagem ([`ph2d_ecs::SpriteSheets`]); aqui mora o cache DERIVADO ([`SheetTextures`], não registado: não entra no
//! gravar nem no undo) e a lei de um ponto só que põe a folha `k` em cena ([`sheet_view`]).
//!
//! A textura resolve-se pela lei do override de script (`ph2d_app_components::script_image`): a célula do átlas
//! quando a imagem já mora nele, senão uma textura individual subida uma vez por imagem.

use crate::{Sprite, SpriteSource};
use bevy_ecs::prelude::{Component, Entity, World};
use ph2d_asset::AssetId;
use ph2d_ecs::{SpriteGrid, SpriteSheets};
use std::collections::BTreeMap;

/// A textura de cada folha extra: `[k - 1]` = a folha `k`, com a imagem para que foi resolvida.
#[derive(Component, Clone, Debug, Default, PartialEq)]
pub struct SheetTextures(pub Vec<Option<(AssetId, SpriteSource)>>);

/// O que a sprite desenha neste quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SheetView {
    /// A folha `0`: o `Sprite` e a grelha de sempre.
    Own,
    /// A folha `k`: uma CÓPIA do `Sprite` com a imagem, o tamanho e o Pivot dela, e a grelha dela.
    Other(Sprite, SpriteGrid),
    /// Um quadro vazio, ou a textura da folha ainda por subir: nada a desenhar.
    Nothing,
}

/// ⭐ **A lei de um ponto só**: a folha do quadro na tela decide a imagem, o tamanho, o Pivot e a grelha.
#[must_use]
pub fn sheet_view(world: &World, entity: Entity, sprite: &Sprite) -> SheetView {
    let Some(grid) = world.get::<SpriteGrid>(entity).copied() else {
        return SheetView::Own;
    };
    match grid.sheet {
        0 => SheetView::Own,
        SpriteGrid::EMPTY_SHEET => SheetView::Nothing,
        k => {
            let sheet = world.get::<SpriteSheets>(entity).and_then(|s| s.sheet(k));
            let source = world
                .get::<SheetTextures>(entity)
                .and_then(|t| t.0.get(usize::from(k) - 1).copied().flatten());
            match (sheet, source) {
                (Some(s), Some((asset, source))) if asset == s.asset => SheetView::Other(
                    Sprite {
                        source,
                        size: s.size,
                        anchor: s.anchor,
                        premultiplied: false,
                        ..*sprite
                    },
                    s.grid(grid.frame),
                ),
                _ => SheetView::Nothing,
            }
        }
    }
}

/// **Uma vez por quadro, antes do extract**: resolve a textura de cada folha extra que mudou de imagem. A célula do
/// átlas ganha (sem cópia); senão `upload` sobe a imagem e devolve o `texture_id` (`None` = ainda não há).
pub fn resolve_sheet_textures(
    world: &mut World,
    atlas: &BTreeMap<u32, AssetId>,
    upload: &mut dyn FnMut(AssetId) -> Option<u32>,
) {
    let mut q = world.query::<(Entity, &SpriteSheets, Option<&SheetTextures>)>();
    let mut writes = Vec::new();
    for (e, sheets, have) in q.iter(world) {
        let old = have.map_or(&[][..], |t| &t.0[..]);
        let new: Vec<_> = sheets
            .0
            .iter()
            .enumerate()
            .map(|(i, s)| match old.get(i).copied().flatten() {
                Some((asset, src)) if asset == s.asset => Some((asset, src)),
                _ => atlas
                    .iter()
                    .find(|(_, a)| **a == s.asset)
                    .map(|(key, _)| SpriteSource::Atlas { key: *key })
                    .or_else(|| {
                        upload(s.asset).map(|texture_id| SpriteSource::Individual { texture_id })
                    })
                    .map(|src| (s.asset, src)),
            })
            .collect();
        if new != old {
            writes.push((e, SheetTextures(new)));
        }
    }
    for (e, t) in writes {
        world.entity_mut(e).insert(t);
    }
    let orphans: Vec<Entity> = world
        .query_filtered::<Entity, (
            bevy_ecs::prelude::With<SheetTextures>,
            bevy_ecs::prelude::Without<SpriteSheets>,
        )>()
        .iter(world)
        .collect();
    for e in orphans {
        world.entity_mut(e).remove::<SheetTextures>();
    }
}

thread_local! {
    /// As imagens de folha já subidas nesta sessão, por imagem (a chave é o conteúdo). O tecto é a biblioteca: uma
    /// textura por imagem DISTINTA usada como folha extra, `w × h × 4` bytes cada.
    static UPLOADED: std::cell::RefCell<BTreeMap<AssetId, u32>> = const {
        std::cell::RefCell::new(BTreeMap::new())
    };
}

/// O `upload` de [`resolve_sheet_textures`] no app: sobe a imagem do `AssetDb` uma vez e devolve o `texture_id`.
pub fn upload_sheet_image(
    renderer: &mut crate::SpriteRenderer,
    db: &ph2d_asset::AssetDb,
    id: AssetId,
) -> Option<u32> {
    if let Some(t) = UPLOADED.with(|u| u.borrow().get(&id).copied()) {
        return Some(t);
    }
    let asset = db.get(&id)?;
    let (w, h, px) = asset.image_rgba8()?;
    let t = renderer.acquire_individual(w, h, &px).ok()?;
    UPLOADED.with(|u| u.borrow_mut().insert(id, t));
    Some(t)
}

#[cfg(test)]
#[path = "sheets_tests.rs"]
mod tests;
