//! ⭐ **A imagem que uma sprite mostra** — a porta que responde «que textura é esta?» e a decisão da troca
//! para um override de script.
//!
//! Desceram da `ph2d-app-components` (`asset_index_build::texture_of`, `script_image::OverrideLaw`) em
//! 2026-10-28: a família da animação de sprite (a troca de imagem) lê e produz as MESMAS, e uma família não
//! chama outra (gate `architecture_no_dependency_climbs_a_layer`).

use ph2d_asset::AssetId;
use ph2d_ecs::{SpriteGrid, SpritePixels};
use std::collections::BTreeMap;

/// ⭐⭐⭐ **QUE TEXTURA esta entidade referencia** — e a resposta tem DUAS formas.
///
/// ⛔⛔ **É a porta que faltava, e a ausência dela custou o report do Enio de 2026-08-30**
/// (*«as imagens não aparecem no painel nem importando nem criando as imagens no app»*): eu
/// escrevi *«uma textura é um asset quando uma entidade a referencia»* e implementei só metade
/// disso — o `SpritePixels`, que é o carimbo da MINORIA.
///
/// | forma | quem a tem |
/// |---|---|
/// | `SpritePixels(id)` | só o que não coube no átlas (16 bits, sobredimensionado) ou foi promovido |
/// | célula de átlas + `atlas_asset_map` | o caminho NORMAL de todo import e de todo canvas novo |
///
/// ⚠️ **O `atlas_assets` é o que separa proveniência de presença:** ele é preenchido pelo import,
/// pelo canvas novo e pelo load do projecto, e ⛔ **nunca pelo arranque** — as 16 do átlas de
/// demonstração ficam de fora sem um caso especial.
///
/// ⚠️ **UMA porta, dois leitores** — o índice (*«que assets existem?»*) e o
/// `ph2d_app_components::asset_card_verbs::users_of` (*«quem usa isto?»*). A 1.ª versão respondia a segunda
/// pergunta com a metade errada, e o *Select users* numa imagem importada devolvia **zero**.
pub fn texture_of(
    pixels: Option<&SpritePixels>,
    sprite: Option<&crate::Sprite>,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    if let Some(p) = pixels {
        return Some(p.0);
    }
    match sprite?.source {
        crate::SpriteSource::Atlas { key } => atlas_assets.get(&key).copied(),
        _ => None,
    }
}

/// ⭐ A decisão da porta da troca (`ph2d_app_sprite_anim::image_choice::override_law`) para um override.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverrideLaw {
    /// A célula do atlas que já tem a imagem — sem cópia.
    pub atlas_key: Option<u32>,
    /// O tamanho do quad (a escala do objecto fica, o tamanho segue a imagem).
    pub size: [f32; 2],
    /// O corte da imagem e a célula a mostrar.
    pub grid: SpriteGrid,
}

/// [`texture_of`] lida no mundo: a imagem da folha `0` de `e`.
#[must_use]
pub fn texture_of_entity(
    world: &ph2d_ecs::World,
    e: ph2d_ecs::Entity,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    texture_of(
        world.get::<SpritePixels>(e),
        world.get::<crate::Sprite>(e),
        atlas_assets,
    )
}

/// ⭐ **TODAS as imagens que `e` usa** (plano GEF 26 F2–F4): a da folha `0` e a de cada folha extra
/// ([`ph2d_ecs::sheet_assets`]), sem repetidas. É a pergunta de «em uso», gravar, macro e Painter.
#[must_use]
pub fn images_of(
    world: &ph2d_ecs::World,
    e: ph2d_ecs::Entity,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Vec<AssetId> {
    let mut out: Vec<AssetId> = texture_of_entity(world, e, atlas_assets)
        .into_iter()
        .collect();
    for a in ph2d_ecs::sheet_assets(world, e) {
        if !out.contains(&a) {
            out.push(a);
        }
    }
    out
}

/// ⭐ **A imagem do quadro na tela** (plano GEF 26 F6): a da folha do quadro, a `0` por [`texture_of_entity`]; o
/// quadro vazio não mostra nenhuma. Junta-se a [`crate::drawn_geometry`] (tamanho, Pivot, grelha) na miniatura.
#[must_use]
pub fn drawn_texture(
    world: &ph2d_ecs::World,
    e: ph2d_ecs::Entity,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    match world.get::<SpriteGrid>(e).map_or(0, |g| g.sheet) {
        0 => texture_of_entity(world, e, atlas_assets),
        k => world
            .get::<ph2d_ecs::SpriteSheets>(e)?
            .sheet(k)
            .map(|s| s.asset),
    }
}

#[cfg(test)]
#[path = "image_of_tests.rs"]
mod tests;
