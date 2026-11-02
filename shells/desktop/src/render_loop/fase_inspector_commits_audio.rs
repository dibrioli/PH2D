//! **As edições da secção ÁUDIO** (TOP-20 #4) — fase-filha da [`super::fase_inspector_commits`].
//!
//! Aqui ficam só as que tocam no DISPOSITIVO (`Preview`, `StopPreview`) ou abrem um DIÁLOGO
//! (`Browse`); o commit de um campo é de `ph2d_app_components::audio_inspector::apply_audio_edit`.

use ph2d_app_components::audio_inspector::apply_audio_edit;
use ph2d_ecs::SimWorld;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_editor_core::{AudioFieldEdit, ToastQueue};

/// Aplica as edições de áudio. `true` = a fila do Inspector ficou suja.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    audio: &mut Option<ph2d_app_audio::AudioSystem>,
    (queue, registry): (&EditorCommandQueue, &ComponentRegistry),
    toasts: &mut ToastQueue,
    edits: &[(u64, AudioFieldEdit)],
) -> bool {
    let mut sujo = false;
    for (bits, edit) in edits {
        let e = ph2d_ecs::Entity::from_bits(*bits);
        let edit = match edit {
            AudioFieldEdit::Preview => {
                super::audio_2d::play_target(sim, audio.as_mut(), e);
                continue;
            }
            AudioFieldEdit::StopPreview => {
                super::audio_2d::stop_target(sim, audio.as_mut(), e);
                continue;
            }
            // ⭐ O ficheiro escolhido entra como som do PROJETO (GEF 21 E2) — o mesmo caminho de um
            // caminho escrito. A lista de extensões é a do resto do app (`decode_any`).
            AudioFieldEdit::Browse => match rfd::FileDialog::new()
                .add_filter("audio", ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS)
                .pick_file()
            {
                Some(p) => AudioFieldEdit::Sound(p.to_string_lossy().into_owned()),
                None => continue,
            },
            other => other.clone(),
        };
        match apply_audio_edit(sim, *bits, &edit, queue, registry) {
            Some(t) => {
                toasts.push(t);
            }
            None => sujo = true,
        }
    }
    sujo
}
