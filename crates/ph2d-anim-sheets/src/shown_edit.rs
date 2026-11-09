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

/// O que um verbo lê de `e`: a folha `0` à vista (o caminho de sempre), a extra, ou a extra SEM imagem legível —
/// esta nunca cai na folha `0`, que o artista não está a ver (auditoria do fecho 08/11, item C).
#[derive(Clone, Debug, PartialEq)]
pub enum ShownRead {
    First,
    Extra(ExtraRead),
    Unreadable,
}

/// `e` tem folhas extra (o `Transform` é partilhado por mais de uma imagem).
#[must_use]
pub fn has_extra_sheets(world: &World, e: Entity) -> bool {
    world.get::<SpriteSheets>(e).is_some_and(|s| !s.0.is_empty())
}

/// A folha visível de `e` para um verbo.
#[must_use]
pub fn read_shown(world: &World, e: Entity, db: &AssetDb) -> ShownRead {
    let Some((sheet, s)) = shown_extra(world, e) else {
        return ShownRead::First;
    };
    let Some(asset) = db.get(&s.asset) else {
        return ShownRead::Unreadable;
    };
    let Some((w, h, px)) = asset.image_rgba8() else {
        return ShownRead::Unreadable;
    };
    ShownRead::Extra(ExtraRead {
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

/// A folha `sheet ≥ 1` de `e` tem um recurso de 16 bits: o verbo lê-o e grava-o em 8, e tem de o dizer, como a
/// folha 0 (`warn_precision_loss`; auditoria do fecho 08/11, item I).
#[must_use]
pub fn sheet_is_16_bit(world: &World, e: Entity, sheet: u8, db: &AssetDb) -> bool {
    world
        .get::<SpriteSheets>(e)
        .and_then(|x| x.sheet(sheet))
        .filter(|_| sheet > 0)
        .and_then(|s| db.get(&s.asset))
        .is_some_and(|a| matches!(*a, ph2d_asset::Asset::ImageRgba16 { .. }))
}

/// O centro da caixa de píxeis `[x, y, w, h]` (Y para baixo) como o objeto andaria, sem o Pivot: a imagem ocupa
/// `old_t ± size / 2`. O BG removal «separar» leva-o ao `place` (a ilha maior); a ilha NOVA nasce nele mais o Pivot
/// da fonte, para cair onde foi desenhada (auditoria do fecho 08/11, G).
#[must_use]
pub fn pixel_box_centre(
    old_t: [f32; 2],
    size: [f32; 2],
    ppm: f32,
    [x, y, w, h]: [u32; 4],
) -> [f32; 2] {
    let cx = x as f32 + w as f32 * 0.5;
    let cy = y as f32 + h as f32 * 0.5;
    [
        old_t[0] - size[0] * 0.5 + cx / ppm,
        old_t[1] + size[1] * 0.5 - cy / ppm,
    ]
}

/// Onde nasce a ilha NOVA da caixa `bx`: o [`pixel_box_centre`] mais o Pivot `anchor` da fonte.
#[must_use]
pub fn island_spawn(
    old_t: [f32; 2],
    size: [f32; 2],
    ppm: f32,
    anchor: [f32; 2],
    bx: [u32; 4],
) -> [f32; 2] {
    let c = pixel_box_centre(old_t, size, ppm, bx);
    [c[0] + anchor[0], c[1] + anchor[1]]
}

/// O recurso da folha `sheet ≥ 1` de `e`; `None` na folha `0` ou numa folha que já não existe.
#[must_use]
pub fn sheet_asset(world: &World, e: Entity, sheet: u8) -> Option<AssetId> {
    let x = world
        .get::<SpriteSheets>(e)?
        .sheet(sheet)
        .filter(|_| sheet > 0)?;
    Some(x.asset)
}

/// ⭐ **Uma prévia ao vivo só desfaz o que ela própria escreveu** (Color equalization, auditoria do fecho 08/11, F):
/// a folha `0` existe sempre; a folha `k` só se ainda mostra `preview`, o recurso que a prévia lá deixou. Apagada,
/// ou trocada por outra coisa, o original larga-se calado — gravar no índice escreveria na folha que chegou lá.
#[must_use]
pub fn preview_still_owns(world: &World, e: Entity, sheet: u8, preview: Option<AssetId>) -> bool {
    sheet == 0 || (preview.is_some() && sheet_asset(world, e, sheet) == preview)
}

/// O aviso da perda (o mesmo texto do `warn_precision_loss` da folha 0) quando [`sheet_is_16_bit`].
pub fn warn_if_16_bit(
    world: &World,
    e: Entity,
    sheet: u8,
    db: &AssetDb,
    toasts: &mut ph2d_editor_core::ToastQueue,
) {
    if sheet_is_16_bit(world, e, sheet, db) {
        toasts.push(ph2d_editor_core::Toast::info(ph2d_i18n::tr_with(
            "shell.texture_edit.converted_to_rgba8",
            &[(
                "because",
                &ph2d_i18n::tr("shell.texture_edit.image_tools_work_in_8"),
            )],
        )));
    }
}

/// ⭐ **Só a folha `0` mexe no `Transform`** (ele é de todas as folhas). Um verbo que muda a DENSIDADE (Upscale)
/// numa folha extra deixa a imagem inteira com o tamanho que tinha na tela: `old_whole`; na folha `0`, `px / ppm`.
#[must_use]
pub fn density_size(sheet: u8, old_whole: [f32; 2], px: [u32; 2], ppm: f32) -> [f32; 2] {
    if sheet == 0 {
        [px[0] as f32 / ppm, px[1] as f32 / ppm]
    } else {
        old_whole
    }
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

/// ⭐ **Onde o objeto fica depois do verbo.** O verbo diz `old_t → new_t` como se o objeto não tivesse
/// rotação, escala nem flip (`recenter_after_crop`: a diferença é em metros LOCAIS, medida nos píxeis). Folha `0`:
/// o `Transform` anda essa diferença passada pela base dele, e o Pivot do `Sprite` vai para `new_anchor` se o verbo
/// o deu. Folha `k`: o objeto fica, e o Pivot da folha `k` anda a diferença, a partir de `new_anchor` ou do dela.
/// Nos dois o flip do `Sprite` espelha a diferença (ele vira os píxeis dentro do quad, não o quad).
pub fn place(
    world: &mut World,
    e: Entity,
    sheet: u8,
    [old_t, new_t]: [[f32; 2]; 2],
    new_anchor: Option<[f32; 2]>,
) {
    let (fx, fy) = world
        .get::<ph2d_render::Sprite>(e)
        .map_or((false, false), |s| (s.flip_x, s.flip_y));
    let sinal = |f: bool| if f { -1.0 } else { 1.0 };
    let d = [
        (new_t[0] - old_t[0]) * sinal(fx),
        (new_t[1] - old_t[1]) * sinal(fy),
    ];
    if sheet == 0 {
        if let Some(mut t) = world.get_mut::<Transform>(e) {
            let mut base = *t;
            (base.translation.x, base.translation.y) = (old_t[0], old_t[1]);
            let mut filho = Transform::default();
            (filho.translation.x, filho.translation.y) = (d[0], d[1]);
            t.translation = Transform::compose(base, filho).translation;
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
            s.anchor = [base[0] + d[0], base[1] + d[1]];
        });
    }
}

#[cfg(test)]
#[path = "shown_edit_tests.rs"]
mod tests;
