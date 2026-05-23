//! Single-level image-edit undo — restores the previous
//! [`ImageEditSnapshot`].

use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::ImageEditSnapshot;

/// Drain a `pending_undo_image_edit` flag: restore the previous
/// `ImageEditSnapshot` (Sprite source / size / Transform translation),
/// release the post-edit texture so it doesn't leak. Single-level
/// undo by design — see SKILL §14 image-edit notes.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
pub(crate) fn drain_undo_image_edit(
    image_edit_undo: &mut Option<ImageEditSnapshot>,
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
        Some(snap) => {
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
            renderer.individual_mut().release(snap.post_individual_id);
            toasts.push(Toast::success(format!(
                "{} · {}",
                ph2d_i18n::tr("edit.undo.image_edit.toast_done"),
                snap.label,
            )));
            true
        }
    }
}
