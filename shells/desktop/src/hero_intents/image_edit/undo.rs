//! Image-edit undo — restores every sprite an [`ImageEditTransaction`]
//! touched (multi-sprite Apply now restores all N sprites, not just the
//! last one — §1 audit CRITICAL data-loss bug, was M14.x scope).

use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::ImageEditTransaction;

/// Drain a `pending_undo_image_edit` flag: restore the previous
/// `ImageEditTransaction` — for every entry it carries, restore the
/// pre-edit Sprite (source / size / premultiplied / anchor) +
/// Transform translation, and release the post-edit individual texture
/// (skipping the `0` sentinel that EqualizeSizes' fit-by-scale entries
/// use). Atomic: one Cmd+Z restores every sprite the transaction
/// touched.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
pub(crate) fn drain_undo_image_edit(
    image_edit_undo: &mut Option<ImageEditTransaction>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) -> bool {
    match image_edit_undo.take() {
        None => {
            toasts.push(Toast::info(ph2d_i18n::tr(
                "edit.undo.image_edit.toast_nothing_to_undo",
            )));
            true
        }
        Some(tx) => {
            // Reverse-iterate by convention (each entry targets a
            // distinct entity, so order does not affect correctness;
            // reverse keeps the restore symmetric with the apply order).
            let n = tx.entries.len();
            for snap in tx.entries.iter().rev() {
                let entity = ph2d_ecs::Entity::from_bits(snap.entity_bits);
                let sim_w = sim.world_mut();
                if let Some(mut sprite) = sim_w.get_mut::<Sprite>(entity) {
                    sprite.source = snap.pre_source;
                    sprite.size = snap.pre_size;
                    sprite.premultiplied = snap.pre_premultiplied;
                    sprite.anchor = snap.pre_anchor;
                }
                if let Some(mut transform) = sim_w.get_mut::<ph2d_ecs::Transform>(entity) {
                    transform.translation.x = snap.pre_translation[0];
                    transform.translation.y = snap.pre_translation[1];
                }
                // `0` is the EqualizeSizes fit-by-scale sentinel — no
                // texture was acquired, nothing to release.
                if snap.post_individual_id != 0 {
                    renderer.individual_mut().release(snap.post_individual_id);
                }
            }
            let toast_body = if n == 1 {
                format!(
                    "{} · {}",
                    ph2d_i18n::tr("edit.undo.image_edit.toast_done"),
                    tx.label,
                )
            } else {
                format!(
                    "{} · {} ({} sprites)",
                    ph2d_i18n::tr("edit.undo.image_edit.toast_done"),
                    tx.label,
                    n,
                )
            };
            toasts.push(Toast::success(toast_body));
            true
        }
    }
}
