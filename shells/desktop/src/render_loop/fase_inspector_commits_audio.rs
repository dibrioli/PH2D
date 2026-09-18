//! **As edições da secção ÁUDIO** (TOP-20 #4) — fase-filha da [`super::fase_inspector_commits`],
//! num ficheiro irmão.
//!
//! # ⚠️ Por que ela corre AQUI e não no `inspector_commits`
//!
//! As edições desta secção são de **DUAS naturezas**: a maioria escreve um campo do documento, e
//! três (`Preview`, `StopPreview`, `Browse`) tocam no **DISPOSITIVO** ou abrem um **diálogo**. O
//! `inspector_commits` não tem — nem devia ter — a placa de som nem a janela. *Duas naturezas,
//! dois sítios; a fronteira é o que cada edição TOCA.*
//!
//! # ⛔ E por que ela saiu do corpo da fase-mãe (2026-09-18)
//!
//! Pelo **tecto de FUNÇÃO**: a mãe foi a `201/200` ao ganhar o gatilho, e este era o maior bloco
//! inline que lá restava (`50` linhas). ⛔ *A cura de um tecto é o CORTE, nunca uma entrada nova no
//! `FN_OVERAGE_OK`* — e o molde já existia neste mesmo ficheiro, três vezes (`factory_commits`,
//! `top20_commits`, `tags_commits`).

use ph2d_ecs::SimWorld;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_editor_core::ToastQueue;

/// Aplica as edições de áudio. `true` = a fila do Inspector ficou suja.
///
/// ⚠️ **O `audio` entra como `&mut Option<…>` e não como `Option<&mut …>`:** o dispositivo é
/// re-emprestado DENTRO do laço, uma vez por edição, e um `Option<&mut>` movido na primeira
/// iteração não sobreviveria à segunda.
pub(super) fn aplicar(
    sim: &mut SimWorld,
    audio: &mut Option<ph2d_app_audio::AudioSystem>,
    editor_queue: &EditorCommandQueue,
    component_registry: &ComponentRegistry,
    toasts: &mut ToastQueue,
    edits: &[(u64, ph2d_editor_core::AudioFieldEdit)],
) -> bool {
    let mut sujo = false;
    for (bits, edit) in edits {
        match edit {
            ph2d_editor_core::AudioFieldEdit::Preview => {
                let e = ph2d_ecs::Entity::from_bits(*bits);
                super::audio_2d::play_target(sim, audio.as_mut(), e);
            }
            ph2d_editor_core::AudioFieldEdit::StopPreview => {
                let e = ph2d_ecs::Entity::from_bits(*bits);
                super::audio_2d::stop_target(sim, audio.as_mut(), e);
            }
            ph2d_editor_core::AudioFieldEdit::Browse => {
                // ⚠️ **A lista de extensões é a MESMA do resto do app** (`decode_any`), e não uma
                // escrita à mão: uma segunda lista ao lado de um predicado é o defeito que o
                // diálogo de importação já pagou — o `.ase` esteve invisível lá durante meses.
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("audio", ph2d_audio_decode::decode_any::AUDIO_IMPORT_EXTS)
                    .pick_file()
                {
                    let edit =
                        ph2d_editor_core::AudioFieldEdit::Sound(p.to_string_lossy().into_owned());
                    if super::inspector_audio::apply_audio_edit(
                        sim,
                        *bits,
                        &edit,
                        editor_queue,
                        component_registry,
                    )
                    .is_none()
                    {
                        sujo = true;
                    }
                }
            }
            _ => {
                if let Some(t) = super::inspector_audio::apply_audio_edit(
                    sim,
                    *bits,
                    edit,
                    editor_queue,
                    component_registry,
                ) {
                    toasts.push(t);
                } else {
                    sujo = true;
                }
            }
        }
    }
    sujo
}
