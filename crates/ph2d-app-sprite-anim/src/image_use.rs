//! ⭐ **O USO de uma imagem num objecto** (planos [GEF 13](../../../docs/GameEngineFuncional/13_isolamento_por_uso_plano.md)
//! D4 e [GEF 15](../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md) D3): parada · repetida (bloco,
//! fundo) · animada. O seletor «Use» do Inspector escolhe-o; um uso exclui os outros, e por isso «imagem animada
//! num bloco» deixou de ser um estado que se possa escrever.

use crate::image_choice::{ImageFacts, Slicing, library_facts};
use ph2d_app_components::asset_index_build::texture_of;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{
    AnimationTag, Entity, RepeatMode, ScrollRepeat, SpriteAnimCursor, SpriteAnimFrames,
    SpriteAnimations, SpriteAnimator, SpriteGrid, SpritePixels, TextureRepeat, TiledDraw,
    UvTransform, World,
};
use ph2d_editor_core::{ImageUseEdit, InspectorDraw, InspectorImageUse};
use ph2d_render::Sprite;
use std::collections::BTreeMap;

/// Para que serve a imagem neste objecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageUse {
    /// Uma imagem (ou uma célula dela) parada.
    Still,
    /// A imagem repete-se: um bloco (*Repeat*, Tiled) ou uma camada de fundo (`ScrollRepeat`).
    Repeated,
    /// As células da imagem tocam como animação.
    Animated,
}

/// O uso de `e`; `None` se não é uma sprite. Animada ganha (as animações mandam no que se desenha), e um
/// animador sem animações já é animada: é o que a secção Animation mostra.
#[must_use]
pub fn image_use(world: &World, e: Entity) -> Option<ImageUse> {
    world.get::<Sprite>(e)?;
    if world.get::<SpriteAnimations>(e).is_some() || world.get::<SpriteAnimator>(e).is_some() {
        return Some(ImageUse::Animated);
    }
    let repete = world.get::<ScrollRepeat>(e).is_some()
        || world.get::<TiledDraw>(e).is_some()
        || matches!(
            world.get::<TextureRepeat>(e),
            Some(TextureRepeat(RepeatMode::Enabled | RepeatMode::Mirror))
        );
    Some(if repete {
        ImageUse::Repeated
    } else {
        ImageUse::Still
    })
}

/// Quantas células tem a imagem.
#[must_use]
pub fn cells(facts: ImageFacts) -> u32 {
    facts
        .slicing
        .map_or(1, |s| s.grid[0].max(1) * s.grid[1].max(1))
}

/// O nome da animação que [`make_animated`] cria.
pub const LOOP_TAG: &str = "loop";

/// ⭐ **Passar a animada uma imagem com corte** (planos GEF 13 D5 e 15 D3): o objecto deixa de repetir e toca
/// todas as células da imagem em laço. O tamanho passa a uma célula, à escala de UM ladrilho de antes pela
/// largura (a imagem não se deforma).
/// `false` se a imagem tem uma célula só, ou o objecto já anima.
pub fn make_animated(world: &mut World, e: Entity, facts: ImageFacts) -> bool {
    let Some(slicing) = facts.slicing else {
        return false;
    };
    let n = cells(facts);
    if n < 2 || image_use(world, e).is_none_or(|u| u == ImageUse::Animated) {
        return false;
    }
    let Some(sprite) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let tiled = world.get::<TiledDraw>(e).map(|t| t.tile[0]);
    // Uma parada que mostra a imagem INTEIRA (sem corte): a célula fica à escala a que a imagem se via.
    let inteira = image_use(world, e) == Some(ImageUse::Still)
        && world
            .get::<SpriteGrid>(e)
            .is_none_or(|g| g.hframes.max(1) * g.vframes.max(1) == 1);
    let ladrilho = match (world.get::<ScrollRepeat>(e), world.get::<UvTransform>(e)) {
        _ if tiled.is_some_and(|t| t > 0.0) => tiled.unwrap_or(1.0),
        (Some(r), _) if r.tile[0] > 0.0 => r.tile[0],
        (_, Some(uv)) if uv.scale[0] > 0.0 => sprite.size[0] / uv.scale[0],
        _ if inteira => sprite.size[0] / slicing.grid[0].max(1) as f32,
        _ => sprite.size[0],
    };
    let celula = [
        facts.native_m[0] / slicing.grid[0].max(1) as f32,
        facts.native_m[1] / slicing.grid[1].max(1) as f32,
    ];
    let escala = if celula[0] > 0.0 {
        ladrilho / celula[0]
    } else {
        1.0
    };
    let mut tags = SpriteAnimations::new();
    if tags.insert(AnimationTag::new(LOOP_TAG, 0, n - 1)).is_err() {
        return false;
    }
    let mut animator = SpriteAnimator::new(LOOP_TAG);
    animator.autoplay = true;
    let mut ent = world.entity_mut(e);
    ent.remove::<(TextureRepeat, UvTransform, ScrollRepeat, TiledDraw)>();
    ent.insert((
        Sprite {
            size: [celula[0] * escala, celula[1] * escala],
            ..sprite
        },
        SpriteGrid {
            hframes: slicing.grid[0].max(1),
            vframes: slicing.grid[1].max(1),
            frame: 0,
        },
        tags,
        animator,
    ));
    true
}

/// Tira a animação de `e` (a lista, o animador e o estado de corrida). As regras «quando → animação» ficam:
/// sem animações dormem, e voltar a animada devolve-as.
fn stop_animating(world: &mut World, e: Entity) {
    world.entity_mut(e).remove::<(
        SpriteAnimations,
        SpriteAnimator,
        SpriteAnimFrames,
        SpriteAnimCursor,
    )>();
}

/// A grelha que a transparência propõe para `rgba` (`w × h`, RGBA8 reto): colunas e linhas transparentes
/// inteiras entre as figuras. `None` sem calhas (uma figura só, ou figuras juntas) — nunca adivinhar.
#[must_use]
pub fn proposed_grid(rgba: &[u8], w: u32, h: u32) -> Option<[u32; 2]> {
    if rgba.len() != (w as usize) * (h as usize) * 4 {
        return None;
    }
    let alpha: Vec<u8> = rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    let g = ph2d_sprite_sheet::guess_grid(&alpha, w, h)?;
    (g.cols * g.rows > 1).then_some([g.cols, g.rows])
}

/// Os factos de uma sprite sem imagem da biblioteca (uma pintura, um recorte): a grelha dela e o tamanho.
fn own_facts(world: &World, e: Entity) -> Option<ImageFacts> {
    let size = world.get::<Sprite>(e)?.size;
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let grid = [g.hframes.max(1), g.vframes.max(1)];
    Some(ImageFacts {
        native_m: [size[0] * grid[0] as f32, size[1] * grid[1] as f32],
        slicing: (grid[0] * grid[1] > 1).then_some(Slicing {
            grid,
            still_frame: g.frame,
        }),
    })
}

/// → Animada. Uma imagem sem corte recebe o que a transparência propõe; sem proposta fica 1 × 1 com a lista
/// vazia (o artista corta) — nenhuma animação inventada.
fn to_animated(
    world: &mut World,
    e: Entity,
    facts: Option<ImageFacts>,
    rgba: Option<(u32, u32, &[u8])>,
) -> bool {
    let Some(mut facts) = facts.or_else(|| own_facts(world, e)) else {
        return false;
    };
    if cells(facts) < 2
        && let Some(grid) = rgba.and_then(|(w, h, px)| proposed_grid(px, w, h))
    {
        facts.slicing = Some(Slicing {
            grid,
            still_frame: 0,
        });
    }
    if cells(facts) >= 2 {
        return make_animated(world, e, facts);
    }
    let tile = world.get::<TiledDraw>(e).map(|t| t.tile);
    let mut ent = world.entity_mut(e);
    if let (Some(tile), Some(mut s)) = (tile, ent.get_mut::<Sprite>()) {
        s.size = tile;
    }
    ent.remove::<(TextureRepeat, UvTransform, TiledDraw)>();
    ent.insert((
        SpriteGrid::SINGLE,
        SpriteAnimations::new(),
        SpriteAnimator::new(""),
    ));
    true
}

/// ⭐ **O seletor «Use»** (plano GEF 15 D3): `e` passa a ser usada como `to`. Um passo de desfazer devolve tudo
/// (as animações, as regras, o Tiled). `false` se já o era, se não é uma sprite, ou se é uma camada de fundo
/// (`ScrollRepeat`, outra lei: repete sempre). `rgba` é a imagem que ela mostra, para propor a grelha.
pub fn set_use(
    world: &mut World,
    e: Entity,
    facts: Option<ImageFacts>,
    rgba: Option<(u32, u32, &[u8])>,
    to: ImageUse,
) -> bool {
    let Some(cur) = image_use(world, e) else {
        return false;
    };
    if cur == to || world.get::<ScrollRepeat>(e).is_some() {
        return false;
    }
    match to {
        ImageUse::Animated => to_animated(world, e, facts, rgba),
        ImageUse::Repeated => {
            stop_animating(world, e);
            crate::tiled::set_tiled(world, e, facts, true);
            true
        }
        ImageUse::Still => {
            stop_animating(world, e);
            // Um bloco que repetia mostra UM ladrilho, à escala a que se via — nunca a imagem esticada ao bloco.
            let ladrilho = world.get::<TiledDraw>(e).map(|t| t.tile).or_else(|| {
                let size = world.get::<Sprite>(e)?.size;
                let uv = world.get::<UvTransform>(e)?;
                (uv.scale[0] > 0.0 && uv.scale[1] > 0.0)
                    .then(|| [size[0] / uv.scale[0], size[1] / uv.scale[1]])
            });
            let mut ent = world.entity_mut(e);
            if let (Some(tile), Some(mut s)) = (ladrilho, ent.get_mut::<Sprite>()) {
                s.size = tile;
            }
            ent.remove::<(TiledDraw, TextureRepeat, UvTransform)>();
            true
        }
    }
}

/// A imagem da biblioteca que `e` mostra.
fn shown_id(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas)
}

/// A imagem da biblioteca que `e` mostra e os factos dela (o corte dos utilizadores incluído).
fn shown_facts(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ImageFacts> {
    library_facts(world, shown_id(world, e, atlas)?, db, atlas, ppm)
}

/// ⭐ O que o Inspector publica de `e` (plano GEF 13 D4): o uso e quantas células tem a imagem (sem imagem da
/// biblioteca — uma pintura, um recorte — conta a grelha do objecto).
#[must_use]
pub fn inspector_use(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> (InspectorImageUse, u32, InspectorDraw) {
    let uso = match image_use(world, e) {
        Some(ImageUse::Animated) => InspectorImageUse::Animated,
        Some(ImageUse::Repeated) => InspectorImageUse::Repeated,
        _ => InspectorImageUse::Still,
    };
    let n = match shown_facts(world, e, db, atlas, ppm) {
        Some(f) => cells(f),
        None => world
            .get::<SpriteGrid>(e)
            .map_or(1, |g| g.hframes.max(1) * g.vframes.max(1)),
    };
    (uso, n, crate::tiled::draw_of(world, e))
}

/// ⭐ **Uma edição do uso da imagem que `e` mostra** — o seletor «Use» e Width/Height. `false` se nada mudou.
pub fn apply_edit(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    edit: ImageUseEdit,
) -> bool {
    let facts = shown_facts(world, e, db, atlas, ppm);
    match edit {
        ImageUseEdit::Use(u) => {
            let to = match u {
                InspectorImageUse::Still => ImageUse::Still,
                InspectorImageUse::Repeated => ImageUse::Repeated,
                InspectorImageUse::Animated => ImageUse::Animated,
            };
            let asset = shown_id(world, e, atlas).and_then(|id| db.get(&id));
            let px = asset.as_ref().and_then(|a| a.image_rgba8());
            let rgba = px.as_ref().map(|(w, h, p)| (*w, *h, &p[..]));
            set_use(world, e, facts, rgba, to)
        }
        ImageUseEdit::Width(m) => crate::tiled::set_tiled_side(world, e, 0, m),
        ImageUseEdit::Height(m) => crate::tiled::set_tiled_side(world, e, 1, m),
    }
}

/// ⭐ **O corte é da IMAGEM** (plano GEF 13 D3, o *importer* da Unity): depois de editar colunas/linhas em `e`,
/// todo objecto que mostra a MESMA imagem (a mesma célula do atlas, ou os mesmos píxeis próprios) passa ao
/// mesmo corte; a célula de cada um fica dentro da grelha. Devolve quantos mudaram.
pub fn recut_users(world: &mut World, e: Entity) -> usize {
    let Some(s) = world.get::<Sprite>(e).copied() else {
        return 0;
    };
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let px = world.get::<SpritePixels>(e).copied();
    let mut q = world.query::<(Entity, &Sprite, Option<&SpritePixels>, Option<&SpriteGrid>)>();
    let outros: Vec<(Entity, u32)> = q
        .iter(world)
        .filter(|(o, os, opx, _)| {
            *o != e
                && match (px, opx) {
                    (Some(a), Some(b)) => a == **b,
                    (None, None) => {
                        matches!(s.source, ph2d_render::SpriteSource::Atlas { .. })
                            && os.source == s.source
                    }
                    _ => false,
                }
        })
        .filter(|(_, _, _, og)| {
            og.map_or((1, 1), |x| (x.hframes, x.vframes)) != (g.hframes, g.vframes)
        })
        .map(|(o, _, _, og)| (o, og.map_or(0, |x| x.frame)))
        .collect();
    let n = (g.hframes.max(1) * g.vframes.max(1)).max(1);
    for (o, frame) in &outros {
        world.entity_mut(*o).insert(SpriteGrid {
            hframes: g.hframes,
            vframes: g.vframes,
            frame: (*frame).min(n - 1),
        });
    }
    outros.len()
}

#[cfg(test)]
#[path = "image_use_gate_tests.rs"]
mod gate_tests;

#[cfg(test)]
#[path = "image_use_15_tests.rs"]
mod uso_15_tests;
