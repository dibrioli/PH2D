//! ⭐ **As ferramentas de imagem na folha VISÍVEL** (dono 08/11: Trim, Padding, Make Square, Upscale, Rasterize, BG
//! removal, Color equalization, Equalize sizes — «como o Painter»). O verbo vê a folha à vista como uma imagem
//! inteira, `célula × corte` na tela, e não sabe de folhas; esta lei lê-a, grava-a e diz onde o objeto fica.
//!
//! Numa folha extra o objeto NÃO anda (o `Transform` é de todas as folhas): o deslocamento que o verbo daria ao
//! objeto vai para o Pivot dessa folha, e a arte fica no lugar. O desfazer é o do documento (as `SpriteSheets` são
//! registadas). Censo dos verbos: `docs/GameEngineFuncional/27_ferramentas_na_folha_visivel.md`.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SpriteSheets, Transform, World};

use crate::shown_image::shown_extra;

/// A folha extra visível, como um verbo a lê: os píxeis RETOS e o tamanho na tela da imagem INTEIRA.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtraRead {
    pub sheet: u8,
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
    /// `célula × corte`, em metros: o que um verbo chama «o tamanho da sprite».
    pub whole: [f32; 2],
    pub anchor: [f32; 2],
}

/// A folha extra visível de `e` para um verbo; `None` com a folha `0` à vista (o caminho de sempre).
#[must_use]
pub fn read_extra(world: &World, e: Entity, db: &AssetDb) -> Option<ExtraRead> {
    let (sheet, s) = shown_extra(world, e)?;
    let asset = db.get(&s.asset)?;
    let (w, h, px) = asset.image_rgba8()?;
    Some(ExtraRead {
        sheet,
        rgba: px.into_owned(),
        w,
        h,
        whole: [
            s.size[0] * s.hframes.max(1) as f32,
            s.size[1] * s.vframes.max(1) as f32,
        ],
        anchor: s.anchor,
    })
}

/// ⭐ **Põe o resultado do verbo na folha `sheet` de `e`**: a folha passa à imagem `asset` (já na biblioteca e na
/// placa: quem chama sobe-a antes, para uma falha não deixar a folha trocada) e `whole`, o tamanho que o verbo deu à
/// imagem inteira, volta a ser o de uma célula. Com arte LIGADA sobe ao mestre ([`ph2d_ecs::linked_art_chain`]), como
/// os píxeis da folha 0. `false` se a folha não existe em `e`.
pub fn set_image(world: &mut World, e: Entity, sheet: u8, asset: AssetId, whole: [f32; 2]) -> bool {
    let set = |s: &mut ph2d_ecs::ExtraSheet| {
        s.asset = asset;
        s.size = [
            whole[0] / s.hframes.max(1) as f32,
            whole[1] / s.vframes.max(1) as f32,
        ];
    };
    if !edit_sheet(world, e, sheet, set) {
        return false;
    }
    for x in ph2d_ecs::linked_art_chain(world, e).into_iter().skip(1) {
        edit_sheet(world, x, sheet, set);
    }
    true
}

/// Edita a folha `sheet ≥ 1` de `e`; `false` se não existe.
fn edit_sheet(
    world: &mut World,
    e: Entity,
    sheet: u8,
    f: impl FnOnce(&mut ph2d_ecs::ExtraSheet),
) -> bool {
    let Some(mut sheets) = world.get::<SpriteSheets>(e).cloned() else {
        return false;
    };
    let Some(x) = usize::from(sheet)
        .checked_sub(1)
        .and_then(|i| sheets.0.get_mut(i))
    else {
        return false;
    };
    f(x);
    world.entity_mut(e).insert(sheets);
    true
}

/// ⭐ **Onde o objeto fica depois do verbo.** Folha `0`: o `Transform` vai para `new_t` e, se o verbo o deu, o
/// Pivot do `Sprite` para `new_anchor` (o de sempre). Folha `k`: o objeto fica, e o Pivot da folha `k` anda o que o
/// objeto andaria (`new_t − old_t`), a partir de `new_anchor` ou do dela.
pub fn place(
    world: &mut World,
    e: Entity,
    sheet: u8,
    [old_t, new_t]: [[f32; 2]; 2],
    new_anchor: Option<[f32; 2]>,
) {
    if sheet == 0 {
        if let Some(mut t) = world.get_mut::<Transform>(e) {
            t.translation.x = new_t[0];
            t.translation.y = new_t[1];
        }
        if let Some(a) = new_anchor
            && let Some(mut s) = world.get_mut::<ph2d_render::Sprite>(e)
        {
            s.anchor = a;
        }
        return;
    }
    // O Pivot da folha é arte: sobe com ela (a folha 0 não, o `Transform` é do objeto).
    for x in ph2d_ecs::linked_art_chain(world, e) {
        edit_sheet(world, x, sheet, |s| {
            let base = new_anchor.unwrap_or(s.anchor);
            s.anchor = [base[0] + new_t[0] - old_t[0], base[1] + new_t[1] - old_t[1]];
        });
    }
}

#[cfg(test)]
#[path = "shown_edit_tests.rs"]
mod tests;
