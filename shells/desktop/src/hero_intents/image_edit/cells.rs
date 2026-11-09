//! ⭐ **Trim, Padding e Make Square numa folha cortada em quadros** (dono 09/11, opção A). A lei é
//! [`ph2d_anim_sheets::cell_edit`]; aqui só a gravação pelo funil de sempre.

use ph2d_anim_sheets::cell_edit::{self, CellChange, Cut};
use ph2d_asset::AssetDb;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::{Toast, ToastQueue};
use ph2d_i18n::tr_with;
use ph2d_render::SpriteRenderer;

use crate::ImageEditSnapshot;
use crate::hero_intents::texture_edit::{self, SourceRead};

/// Grava a folha `src` com cada célula mudada por `ch`, o Pivot dela anda e o desenho de cada quadro fica; o undo da
/// folha `0` entra como nos verbos de sempre. `falhou` = a chave do aviso de erro do verbo (com `{err}`). Devolve
/// `true` se gravou.
#[allow(clippy::too_many_arguments)]
pub(super) fn grava(
    entity_bits: u64,
    src: &SourceRead,
    (cut, ch): (Cut, CellChange),
    (label, falhou): (&'static str, &str),
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    toasts: &mut ToastQueue,
    pending_undo_entries: &mut Vec<ImageEditSnapshot>,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let (w, h) = (ch.cell[0] * cut.cols, ch.cell[1] * cut.rows);
    let max_dim = renderer.max_texture_dimension_2d();
    if w > max_dim || h > max_dim {
        let err = tr_with(
            "shell.cells.too_big",
            &[("width", &w), ("height", &h), ("max_dim", &max_dim)],
        );
        toasts.push(Toast::error(tr_with(falhou, &[("err", &err)])));
        return false;
    }
    let (px, ..) = cell_edit::recompose(&src.image.pixels, src.image.width, &cut, ch);
    let halves = src
        .pixels_16
        .as_ref()
        .map(|p| cell_edit::recompose(p, src.image.width, &cut, ch).0);
    let edited = ph2d_render::SpriteImage::from_bytes(w, h, px, src.image.alpha);
    let gravado = texture_edit::commit_geometric_edit(
        (entity, src.sheet),
        sim,
        renderer,
        asset_db,
        &edited,
        halves.as_deref(),
        cell_edit::commit_size(&cut, ch, src.sheet),
        toasts,
    );
    let texture_id = match gravado {
        Ok(id) => id,
        Err(err) => {
            toasts.push(Toast::error(tr_with(falhou, &[("err", &err)])));
            return false;
        }
    };
    let flip = sim
        .world()
        .get::<ph2d_render::Sprite>(entity)
        .map_or([false, false], |s| [s.flip_x, s.flip_y]);
    let d = cell_edit::pivot_shift(&cut, ch, flip);
    cell_edit::keep_drawing(sim.world_mut(), entity, src.sheet, d);
    if src.sheet == 0 {
        pending_undo_entries.push(ImageEditSnapshot {
            entity_bits,
            pre_source: src.old_source,
            pre_size: src.old_size_world,
            pre_translation: src.old_translation,
            pre_premultiplied: src.old_premultiplied,
            pre_anchor: src.old_anchor,
            post_individual_id: texture_id,
            label,
        });
    }
    toasts.push(Toast::success(tr_with(
        "shell.cells.every_frame",
        &[
            ("label", &label),
            ("width", &ch.cell[0]),
            ("height", &ch.cell[1]),
        ],
    )));
    true
}
