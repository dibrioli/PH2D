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
    /// A folha `0` sem acerto no quadro: o `Sprite` e a grelha de sempre.
    Own,
    /// A folha `0` com acerto no quadro: o `Sprite` com o quad deslocado; o resto (grelha, região) é o de sempre.
    OwnNudged(Sprite),
    /// A folha `k`: uma CÓPIA do `Sprite` com a imagem, o tamanho e o Pivot dela, e a grelha dela.
    Other(Sprite, SpriteGrid),
    /// Um quadro vazio, ou a textura da folha ainda por subir: nada a desenhar.
    Nothing,
}

/// A folha extra `s` sobre o resto de `sprite`: o tamanho e o Pivot dela. O Pivot da folha já é o resolvido (sem
/// `offset` nem canto), logo a cópia nasce centrada.
#[must_use]
pub fn sheet_sprite(s: &ph2d_ecs::ExtraSheet, sprite: &Sprite) -> Sprite {
    Sprite {
        size: s.size,
        anchor: s.anchor,
        centered: true,
        offset: [0.0, 0.0],
        premultiplied: false,
        ..*sprite
    }
}

/// A GEOMETRIA da folha `k` (tamanho, Pivot, grelha na célula `frame`), sem textura: `k = 0` ou inexistente = o
/// próprio `Sprite` e a grelha dele.
#[must_use]
pub fn sheet_geometry(
    world: &World,
    entity: Entity,
    k: u8,
    frame: u32,
) -> Option<(Sprite, Option<SpriteGrid>)> {
    let sprite = world.get::<Sprite>(entity)?;
    let grid = world.get::<SpriteGrid>(entity).copied();
    let (mut s, g) = match world.get::<SpriteSheets>(entity).and_then(|s| s.sheet(k)) {
        Some(s) => (sheet_sprite(s, sprite), Some(s.grid(frame))),
        None => (*sprite, grid),
    };
    if let Some(a) = ph2d_ecs::anchor_da_celula(world, entity, k, frame, s.size) {
        s.anchor = a;
    }
    Some((s, g))
}

/// `sprite` com o desenho deslocado `d` metros locais (o acerto de um quadro, GEF 28): o Pivot fica, o quad anda.
#[must_use]
pub fn nudged(sprite: Sprite, d: [f32; 2]) -> Sprite {
    Sprite {
        anchor: [sprite.anchor[0] + d[0], sprite.anchor[1] + d[1]],
        ..sprite
    }
}

/// ⭐ **O quad na tela** sem textura: o que a caixa do gizmo, o encaixe e a grelha do canvas medem — com o acerto do
/// quadro na tela, como o [`sheet_view`] o desenha. Um quadro vazio mede a folha `0` (não há desenho, mas o objeto
/// continua agarrável). [`sheet_geometry`] é a da FOLHA (sem acerto): a que o Pivot e o Trim editam.
#[must_use]
pub fn drawn_geometry(world: &World, entity: Entity) -> Option<(Sprite, Option<SpriteGrid>)> {
    let g = world
        .get::<SpriteGrid>(entity)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let (s, grid) = sheet_geometry(world, entity, g.sheet, g.frame)?;
    let d = ph2d_ecs::cell_offset(world, entity, g.sheet, g.frame);
    Some((nudged(s, d), grid))
}

/// ⭐ **A lei de um ponto só**: a folha do quadro na tela decide a imagem, o tamanho, o Pivot e a grelha; o acerto da
/// célula (GEF 28) desloca o quad. A folha `0` sem acerto desenha o próprio `Sprite` ([`SheetView::Own`]).
#[must_use]
pub fn sheet_view(world: &World, entity: Entity, sprite: &Sprite) -> SheetView {
    if ph2d_ecs::imagem_aberta(world, entity).is_some() {
        return open_sheet_view(world, entity, sprite);
    }
    let Some(grid) = world.get::<SpriteGrid>(entity).copied() else {
        return SheetView::Own;
    };
    let d = ph2d_ecs::cell_offset(world, entity, grid.sheet, grid.frame);
    match grid.sheet {
        // P14e: o pivô da célula é a âncora base; o acerto do quadro (GEF 28) desloca o quad por cima dela.
        0 => match ph2d_ecs::anchor_da_celula(world, entity, 0, grid.frame, sprite.size) {
            Some(anchor) => SheetView::Other(nudged(Sprite { anchor, ..*sprite }, d), grid),
            None if d == [0.0, 0.0] => SheetView::Own,
            None => SheetView::OwnNudged(nudged(*sprite, d)),
        },
        SpriteGrid::EMPTY_SHEET => SheetView::Nothing,
        k => {
            let sheet = world.get::<SpriteSheets>(entity).and_then(|s| s.sheet(k));
            let source = world
                .get::<SheetTextures>(entity)
                .and_then(|t| t.0.get(usize::from(k) - 1).copied().flatten());
            match (sheet, source) {
                (Some(s), Some((asset, source))) if asset == s.asset => {
                    let mut c = Sprite {
                        source,
                        ..sheet_sprite(s, sprite)
                    };
                    if let Some(a) =
                        ph2d_ecs::anchor_da_celula(world, entity, k, grid.frame, c.size)
                    {
                        c.anchor = a;
                    }
                    SheetView::Other(nudged(c, d), s.grid(grid.frame))
                }
                _ => SheetView::Nothing,
            }
        }
    }
}

/// ⭐ **A imagem aberta no Painter é a da folha VISÍVEL** (plano GEF 26 F13), desenhada inteira sobre a geometria da
/// folha `0` — a mesma que o ponteiro e o chrome do Painter medem (`ph2d_sprite_screen::sprite_mostrada` + a grelha do
/// objeto); o quad que a desdobra ao tamanho dela é o de `ph2d_ecs::ImagemAberta`.
fn open_sheet_view(world: &World, entity: Entity, sprite: &Sprite) -> SheetView {
    let k = ph2d_ecs::shown_sheet(world, entity);
    if k == 0 {
        return SheetView::Own;
    }
    let source = world
        .get::<SheetTextures>(entity)
        .and_then(|t| t.0.get(usize::from(k) - 1).copied().flatten());
    let grid = world
        .get::<SpriteGrid>(entity)
        .map_or(SpriteGrid::SINGLE, |g| SpriteGrid {
            frame: 0,
            sheet: 0,
            ..*g
        });
    match source {
        Some((_, source)) => SheetView::Other(
            Sprite {
                source,
                premultiplied: false,
                ..*sprite
            },
            grid,
        ),
        None => SheetView::Nothing,
    }
}

/// ⭐ **A prévia da animação ao lado da imagem aberta no Painter** corta-se pela folha VISÍVEL: com a folha `k ≥ 1`
/// aberta, o tamanho e a grelha são os DELA (a célula na tela, se é dela; senão a primeira). Fora disso, `(sprite,
/// grid)` tal como vieram.
#[must_use]
pub fn open_preview_cell(
    world: &World,
    entity: Entity,
    sprite: &Sprite,
    grid: SpriteGrid,
) -> (Sprite, SpriteGrid) {
    let k = ph2d_ecs::shown_sheet(world, entity);
    let sheet = world.get::<SpriteSheets>(entity).and_then(|s| s.sheet(k));
    match sheet {
        Some(s) if k > 0 && ph2d_ecs::imagem_aberta(world, entity).is_some() => {
            let na_tela = world.get::<SpriteGrid>(entity).copied();
            let frame = na_tela.filter(|g| g.sheet == k).map_or(0, |g| g.frame);
            (sheet_sprite(s, sprite), s.grid(frame))
        }
        _ => (*sprite, grid),
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
