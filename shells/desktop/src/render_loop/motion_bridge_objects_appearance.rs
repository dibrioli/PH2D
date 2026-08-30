//! **COMO UM SPRITE VIRA UMA IMAGEM** — de que loja sai a aparência de cada variante do
//! `SpriteSource`, e o tile de uma instância só que o grafo recebe.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/`), e o corte é por
//! FAMÍLIA, como o dos irmãos `_lod` / `_shift` / `_streams`: o `objects.rs` responde *quem é
//! publicado e sob que nome*, e este responde *com que cara*.

use super::appearance_tile;
use ph2d_nodegraph::attr::Stream;
use ph2d_render::{RenderInstance, Sprite, SpriteSource, TextureAtlas};

/// The appearance tile for one sprite: one instance at the origin carrying
/// `(P, size, tint, uv_rect, texture_id)`. `None` for a source the atlas cannot
/// resolve here (a cooked KTX2 sprite needs the renderer's cooked-texture store,
/// not in hand — deferred to a later wave; it is skipped, not guessed).
pub(super) fn sprite_tile(spr: &Sprite, look: Appearance<'_>) -> Option<Stream> {
    let (uv_rect, texture_id) = sprite_appearance(spr, look)?;
    // `collapsed_tint` = self_tint × tint (the per-sprite modulate). The
    // inherited ancestor cascade the extract folds in is a refinement of a
    // template, deferred: a source is *this object's* appearance.
    Some(appearance_tile(
        spr.size,
        spr.collapsed_tint(),
        uv_rect,
        texture_id,
        spr.premultiplied,
    ))
}

/// **De onde a APARÊNCIA de um sprite sai** — as DUAS lojas, não só o atlas.
///
/// ⚠️ **A cerca 7 da folha 14 era ENCANAMENTO com cara de desenho:** ela dizia *"um sprite
/// KTX2 nomeado é fonte invisível, por decisão declarada"*, e a razão ao lado do `None` era
/// *"resolve through `renderer.cooked_texture_id`, **not in hand**"* — o chamador tem-no na
/// linha de onde já tira o `atlas()`. *Uma decisão cuja razão é «não tenho isto aqui» é um
/// adiamento, e o que a dissolve é passar aquilo.*
///
/// ⚠️ **Um resolvedor e não o `Renderer`:** esta é uma PONTE, e a superfície inteira dele
/// convidaria a próxima linha a chamar outra coisa dali.
#[derive(Clone, Copy)]
pub(crate) struct Appearance<'a> {
    pub(crate) atlas: &'a TextureAtlas,
    pub(crate) cooked: &'a dyn Fn(ph2d_asset::LogicalTextureId) -> Option<u32>,
}

/// The `(uv_rect, texture_id)` a sprite resolves to — the branch `sim_extract`
/// runs. Shared by the single-sprite path and the group-child path (doc 86 §2 A4).
pub(crate) fn sprite_appearance(spr: &Sprite, look: Appearance<'_>) -> Option<([f32; 4], u32)> {
    appearance_of(spr.source, &|k| look.atlas.region_uv(k), look.cooked)
}

/// **A escolha por VARIANTE, sem as lojas** — a metade da lei que se mede sem uma GPU.
///
/// ⚠️ Ela existe por uma razão de TESTE que é também de desenho: um `TextureAtlas` só se
/// constrói com um contexto de GPU (`TextureAtlas::dummy(ctx)`), então enquanto a decisão
/// vivesse colada a ele a única testemunha possível era um gate `#[ignore]` — e *skip gracioso
/// não é verde*. Separar *qual loja responde* de *o que a loja diz* torna a primeira metade
/// uma função pura de três argumentos.
pub(super) fn appearance_of(
    src: SpriteSource,
    atlas_uv: &dyn Fn(u32) -> [f32; 4],
    cooked: &dyn Fn(ph2d_asset::LogicalTextureId) -> Option<u32>,
) -> Option<([f32; 4], u32)> {
    match src {
        // Atlas → the packed cell's UV, sampling the shared atlas (`0`); the
        // cheap direct path (no bake) the sprite renderer already uses.
        SpriteSource::Atlas { key } => Some((atlas_uv(key), RenderInstance::ATLAS_TEXTURE_ID)),
        // Individual → the full unit rect + the store handle it already carries.
        SpriteSource::Individual { texture_id } => Some(([0.0, 0.0, 1.0, 1.0], texture_id)),
        // ⚠️ **KTX2 assado: o rect INTEIRO, como o `Individual`** — uma textura assada é
        // dela própria, não uma célula de um atlas partilhado. `None` só quando a loja não
        // conhece o id lógico (o artefacto ainda não carregou), e aí o nó continua a não
        // emitir nada — *não adivinha e não falha*, a cerca 6 da mesma folha.
        SpriteSource::CookedTexture { logical_id } => {
            cooked(logical_id).map(|tid| ([0.0, 0.0, 1.0, 1.0], tid))
        }
    }
}
