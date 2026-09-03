//! **A PRIMEIRA migração de `PROJECT_SCHEMA` da história do repo** — v95 para o schema
//! CORRENTE (ADR-0164 F1). Nasceu `v95 -> v96`; a integração de 2026-08-24 fê-la aterrar na
//! **v97**, porque uma linha irmã apendou um campo na mesma jornada — ver
//! [`migrate_v95_to_v96`].
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

/// **v95 → o schema CORRENTE** (nasceu `v95 -> v96`; ver a nota abaixo).
///
/// A conversão do mundo é a [`migrate_v1_to_v2`] do `ph2d-ecs` (ids na ordem das linhas, que
/// na v1 já era a ordem canónica); tudo o resto viaja intacto.
///
/// ⚠️⚠️ **O DESTINO desta função subiu na integração de 2026-08-24, e o nome não a segue de
/// propósito** — o que está congelado é a ORIGEM (`ProjectFileV95`, os bytes que se sabem ler),
/// e é a origem que o nome anuncia. Uma linha irmã apendou o `input_map` ao `ProjectFile` na
/// mesma jornada, o degrau dela foi recontado de `96` para `97`, e este salto passou a aterrar
/// lá: o campo novo entra com o **default vazio**, que é exactamente o que um ficheiro v95 tem
/// a dizer sobre um mapa de controlos que ainda não existia.
///
/// ⚠️ **Não há `migrate_v96_to_v97`, e a ausência é a decisão:** a v96 viveu apenas dentro de
/// duas worktrees durante uma jornada e nunca foi publicada, logo **não existe ficheiro v96 no
/// mundo** para migrar. O `_ =>` do [`crate::project_load`] recusa-a, que é a resposta certa
/// para uma versão que ninguém pode ter. Quem um dia precisar de um degrau intermédio congela
/// o tipo primeiro — como o [`ProjectFileV95`] foi congelado.
///
/// ⛔ **O compilador é a cerca deste salto:** todo campo novo no `ProjectFile` torna esta
/// função um erro de compilação até alguém dizer com que valor um ficheiro v95 o preenche.
/// Foi assim que o `input_map` foi apanhado.
#[must_use]
pub(crate) fn migrate_v95_to_v96(old: ProjectFileV95) -> MigratedV95 {
    let stable_id_counter = next_free_after_migration(&old.state.world);
    MigratedV95 {
        file: crate::project::ProjectFile {
            state: ProjectState {
                world: migrate_v1_to_v2(&old.state.world),
                // ⚠️ A cena passou a ser partilhada entre passos ([`ProjectState::vec`]); um
                // ficheiro velho traz-na por valor e entra aqui embrulhada. Os bytes são os mesmos.
                vec: std::sync::Arc::new(old.state.vec),
                flip: old.state.flip,
                guides: old.state.guides,
                ui_states: old.state.ui_states,
                // ⚠️ **VAZIA, e o vazio é a resposta certa:** um ficheiro v95 foi gravado por um
                // build sem navegador de assets nenhum, logo ele não tem taxonomia a dizer nem
                // imagem nenhuma mandada sair. Uma biblioteca vazia é o que ele já significava.
                library: crate::project_library::LibraryDoc::default(),
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
            // ⚠️ **VAZIO, e o vazio é a resposta certa** (v97): um ficheiro v95 foi gravado por
            // um build em que nenhuma acção nomeada existia, então ele não tem nada a dizer
            // sobre controlos. E um `InputMap` vazio devolve silêncio em toda leitura — o
            // comportamento byte-a-byte de todo ficheiro anterior ao mapa.
            input_map: ph2d_input::InputMap::default(),
            // ⚠️ **Um v95 não tem padrão nenhum, e a resposta é o vazio** — a variante
            // `Paint::Pattern` só existe desde o `VEC_SCENE_SCHEMA_VERSION` 15 (plano 33 W3), que é
            // posterior. Vazio aqui não é "não sei": é o que aquele ficheiro de facto tem.
            pattern_art: Vec::new(),
        },
        stable_id_counter,
    }
}

#[cfg(test)]
#[path = "project_migrate_tests.rs"]
mod tests;
