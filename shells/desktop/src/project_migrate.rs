//! **A PRIMEIRA migração de `PROJECT_SCHEMA` da história do repo** — v95 → v96 (ADR-0164 F1).
//!
//! # O que estava aqui antes
//!
//! Nada. A auditoria de 2026-08-21 registou-o como ambiguidade em aberto (§8 item 7):
//! *"HR-14 exige `migrate_vN_to_vN+1`; o repo tem **zero** e recusa arquivos antigos. Não
//! determinado se existe alguma promessa de compatibilidade, ou se a recusa é a política
//! declarada."* Era o que havia, não uma política escrita — o `project_load.rs` comparava a
//! versão e desistia.
//!
//! ⚠️ **A v96 mexe em duas coisas ao mesmo tempo, e por isso precisa de migração e não só de
//! um degrau:** o `WorldSnapshot` passou de v1 a v2 (linha chaveada por `StableId` em vez de
//! índice) e o `ProjectFile` ganhou o `stable_id_counter`. Nenhuma das duas é aditiva no wire
//! — o postcard é **posicional**, então um leitor v96 sobre bytes v95 não erra: **lê errado**.
//!
//! # A técnica, e o seu risco
//!
//! [`ProjectFileV95`] é uma cópia congelada da forma antiga. Os campos que **não** mudaram
//! referenciam os MESMOS tipos vivos (`VecScene`, `FlipDoc`, `GuideSet`, …), o que mantém isto
//! em 40 linhas em vez de 400.
//!
//! ⛔ **E é exactamente aí que este arquivo pode apodrecer:** se um desses tipos mudar de forma
//! amanhã, o `ProjectFileV95` segue-o em silêncio e deixa de ler ficheiros v95 reais — sem
//! erro de compilação e sem teste vermelho, porque os dois lados mudaram juntos. A cerca é o
//! gate `the_frozen_v95_bytes_still_load` (em [`crate::project_migrate_tests`]), que guarda os
//! **bytes** de um ficheiro v95 e não o tipo. Se ele ficar vermelho, a resposta não é
//! re-gerar os bytes: é congelar de verdade o tipo que se mexeu.

use crate::undo::ProjectState;
use ph2d_ecs::scene::{WorldSnapshotV1, migrate_v1_to_v2, next_free_after_migration};

/// O `ProjectState` como a v95 o guardava — com o snapshot **v1**.
#[derive(serde::Deserialize)]
pub(crate) struct ProjectStateV95 {
    pub(crate) world: WorldSnapshotV1,
    pub(crate) vec: ph2d_vec_scene::VecScene,
    pub(crate) flip: ph2d_flip::FlipDoc,
    pub(crate) guides: ph2d_guides::GuideSet,
    pub(crate) ui_states: ph2d_ui_state::StateSets,
}

/// O ficheiro como a v95 o guardava — **sem** o `stable_id_counter`.
///
/// ⚠️ A ordem dos campos **é** o formato (postcard é posicional). Não a reordene.
#[derive(serde::Deserialize)]
pub(crate) struct ProjectFileV95 {
    pub(crate) state: ProjectStateV95,
    pub(crate) assets: Vec<crate::project::SavedAsset>,
    pub(crate) painted: Vec<ph2d_tool_painter::PaintedDocument>,
    pub(crate) motion: String,
    pub(crate) timeline: Vec<u8>,
    pub(crate) physics: ph2d_physics_ecs::PhysicsSettings,
    pub(crate) tokens: Vec<crate::project_tokens::SavedToken>,
    pub(crate) settings: crate::project_settings::SavedSettings,
    pub(crate) sculpt: Vec<u8>,
    pub(crate) baked_forms: Vec<crate::project_baked_form::BakedFormDocument>,
    pub(crate) player_tape: ph2d_physics_ecs::TapeWire,
    pub(crate) sprite_pixels: Vec<u8>,
}

/// O resultado de migrar: o estado já em v2 **e** a semente do contador de ids.
pub(crate) struct MigratedV95 {
    pub(crate) file: crate::project::ProjectFile,
    /// O primeiro id livre. ⚠️ Tem de ser plantado no mundo depois do restore, senão a
    /// primeira entidade criada a seguir ao load reusa um id vivo.
    pub(crate) stable_id_counter: u64,
}

/// **v95 → v96.**
///
/// A conversão do mundo é a [`migrate_v1_to_v2`] do `ph2d-ecs` (ids na ordem das linhas, que
/// na v1 já era a ordem canónica); tudo o resto viaja intacto.
#[must_use]
pub(crate) fn migrate_v95_to_v96(old: ProjectFileV95) -> MigratedV95 {
    let stable_id_counter = next_free_after_migration(&old.state.world);
    MigratedV95 {
        file: crate::project::ProjectFile {
            state: ProjectState {
                world: migrate_v1_to_v2(&old.state.world),
                vec: old.state.vec,
                flip: old.state.flip,
                guides: old.state.guides,
                ui_states: old.state.ui_states,
            },
            assets: old.assets,
            painted: old.painted,
            motion: old.motion,
            timeline: old.timeline,
            physics: old.physics,
            tokens: old.tokens,
            settings: old.settings,
            sculpt: old.sculpt,
            baked_forms: old.baked_forms,
            player_tape: old.player_tape,
            sprite_pixels: old.sprite_pixels,
            stable_id_counter,
        },
        stable_id_counter,
    }
}

#[cfg(test)]
#[path = "project_migrate_tests.rs"]
mod tests;
