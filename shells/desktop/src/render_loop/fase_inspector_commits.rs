//! **Fase do quadro: OS COMMITS DO INSPECTOR** — Transform, Visibility, Name, a origem da sprite e o Reimport
//! pelo `inspector_commits::dispatch`, a captura do pivô de um joint para o re-assento da fase seguinte, as
//! secções AUDIO e CAMERA (edições de DUAS naturezas: campo do documento, e dispositivo/vista) e a fila (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;
use ph2d_i18n::tr_with;

/// ⭐⭐⭐ A fase-filha dos gestos sobre a ÁRVORE de tags — ficheiro irmão por `#[path]`, para o
/// `render_loop/mod.rs` não crescer acima do tecto dele (o molde do `fase_snapshot_readouts`).
#[path = "fase_tag_tree_commits.rs"]
mod tag_tree_commits;

/// ⭐⭐⭐ A fase-filha das edições da FÁBRICA e do CICLO DE VIDA — irmã da de cima, e pela mesma
/// razão: o tecto de LOC desta fase, e uma fronteira que o comentário do bloco já escrevia.
#[path = "fase_factory_commits.rs"]
mod factory_commits;
#[path = "fase_topdown_commits.rs"]
mod topdown_commits;

/// ⭐⭐⭐ A fase-filha das edições do PROJÉCTIL — irmã das de cima, e pela mesma razão.
#[path = "fase_projectile_commits.rs"]
mod projectile_commits;

/// ⭐⭐⭐ A fase-filha das edições do CÉREBRO (TOP-20 #15) — irmã das de cima.
#[path = "fase_statemachine_commits.rs"]
mod statemachine_commits;

/// ⭐⭐⭐ A fase-filha das edições da secção ÁUDIO — irmã das de cima, e pelo mesmo tecto. Ver o
/// cabeçalho de lá para a fronteira dela (o dispositivo e o diálogo).
#[path = "fase_inspector_commits_audio.rs"]
mod audio_commits;
/// ⭐⭐⭐ A fase-filha das edições do SCRIPT (TOP-20 #16) — irmã das de cima.
#[path = "fase_script_commits.rs"]
mod script_commits;
#[path = "fase_inspector_commits_top20.rs"]
mod top20_commits;

/// ⭐⭐⭐ A fase-filha das edições da secção TAGS — irmã das de cima, e pelo mesmo tecto.
#[path = "fase_tags_commits.rs"]
mod tags_commits;

/// As edições do Inspector que o dreno do barramento recolheu neste quadro.
pub(super) struct InspectorIntents {
    pub(super) reimport_entity: Option<u64>,
    pub(super) transform_edit: Option<ph2d_editor_core::InspectorTransformInfo>,
    pub(super) visibility_edits: Vec<(u64, bool)>,
    pub(super) sprite_edits: Vec<(u64, ph2d_editor_core::SpriteFieldEdit)>,
    pub(super) ordering_edits: Vec<(u64, ph2d_editor_core::OrderingFieldEdit)>,
    pub(super) sampling_edits: Vec<(u64, ph2d_editor_core::SamplingFieldEdit)>,
    pub(super) blend_edits: Vec<(u64, ph2d_editor_core::BlendFieldEdit)>,
    pub(super) slice_edits: Vec<(u64, ph2d_editor_core::SliceFieldEdit)>,
    pub(super) anchor_edits: Vec<(u64, ph2d_editor_core::AnchorFieldEdit)>,
    pub(super) anim_edits: Vec<(u64, ph2d_editor_core::AnimFieldEdit)>,
    pub(super) timer_edits: Vec<(u64, ph2d_editor_core::TimerFieldEdit)>,
    pub(super) audio_edits: Vec<(u64, ph2d_editor_core::AudioFieldEdit)>,
    pub(super) camera_edits: Vec<(u64, ph2d_editor_core::CameraFieldEdit)>,
    /// ⭐ As edições das secções FACTORY e LIFECYCLE (TOP-20 #11 e #12).
    pub(super) factory_edits: Vec<(u64, ph2d_editor_core::FactoryFieldEdit)>,
    /// ⭐ As edições do MOVER DE VISTA DE CIMA (TOP-20 #13).
    pub(super) topdown_edits: Vec<(u64, ph2d_editor_core::topdown_edits::TopDownFieldEdit)>,
    /// ⭐ As edições do PROJÉCTIL (TOP-20 #14).
    pub(super) projectile_edits:
        Vec<(u64, ph2d_editor_core::projectile_edits::ProjectileFieldEdit)>,
    /// ⭐⭐⭐ As edições da secção RAY SENSOR (suplente #21).
    pub(super) ray_edits: Vec<(u64, ph2d_editor_core::ray_edits::RayFieldEdit)>,
    /// ⭐ As edições do CÉREBRO (TOP-20 #15).
    pub(super) statemachine_edits: Vec<(
        u64,
        ph2d_editor_core::statemachine_edits::StateMachineFieldEdit,
    )>,
    /// ⭐ As edições do SCRIPT (TOP-20 #16).
    pub(super) script_edits: Vec<(u64, ph2d_editor_core::script_edits::ScriptFieldEdit)>,
    /// ⭐ As edições do EMISSOR DE PARTÍCULAS (TOP-20 #18).
    pub(super) particles_edits: Vec<(u64, ph2d_editor_core::particles_edits::ParticlesFieldEdit)>,
    /// ⭐⭐⭐ A secção HUD (TOP-20 #20).
    pub(super) hud_edits: Vec<(u64, ph2d_editor_core::hud_edits::HudFieldEdit)>,
    /// ⭐ As edições da secção SEQUENCE (TOP-20 #19).
    pub(super) sequence_edits: Vec<(u64, ph2d_editor_core::sequence_edits::SequenceFieldEdit)>,
    /// As edições da secção COUNTER WATCH.
    pub(super) counter_watch_edits: Vec<(
        u64,
        ph2d_editor_core::counter_watch_edits::CounterWatchFieldEdit,
    )>,
    /// As edições da secção GATILHO (suplente #24).
    pub(super) action_trigger_edits: Vec<(
        u64,
        ph2d_editor_core::action_trigger_edits::ActionTriggerFieldEdit,
    )>,
    pub(super) tags_edits: Vec<(u64, ph2d_editor_core::TagsFieldEdit)>,
    pub(super) tag_tree_edits: Vec<ph2d_editor_core::TagTreeEdit>,
    pub(super) inspector_queue_dirty: bool,
    pub(super) action_edits: Vec<(u64, ph2d_editor_core::ActionFieldEdit)>,
    pub(super) physics_edits: Vec<(u64, ph2d_editor_core::PhysicsFieldEdit)>,
    pub(super) visibility_section_edits: Vec<(u64, ph2d_editor_core::VisibilityFieldEdit)>,
    pub(super) name_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_edit: Option<ph2d_editor_core::InspectorNameInfo>,
    pub(super) signal_leave_edit: Option<ph2d_editor_core::InspectorNameInfo>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_inspector_commits(
        &mut self,
        intents: InspectorIntents,
    ) -> Option<Option<(u64, [f32; 2])>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            asset_db,
            toasts,
            hero_screen,
            atlas_asset_map,
            component_registry,
            editor_queue,
            transform_type_id,
            visibility_type_id,
            name_type_id,
            sprite_type_id,
            tags,
            tags_problem,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let InspectorIntents {
            reimport_entity,
            transform_edit,
            visibility_edits,
            sprite_edits,
            ordering_edits,
            sampling_edits,
            blend_edits,
            slice_edits,
            anchor_edits,
            anim_edits,
            timer_edits,
            audio_edits,
            camera_edits,
            factory_edits,
            topdown_edits,
            projectile_edits,
            ray_edits,
            statemachine_edits,
            script_edits,
            particles_edits,
            hud_edits,
            sequence_edits,
            counter_watch_edits,
            action_trigger_edits,
            tags_edits,
            tag_tree_edits,
            mut inspector_queue_dirty,
            action_edits,
            physics_edits,
            visibility_section_edits,
            name_edit,
            signal_edit,
            signal_leave_edit,
        } = intents;
        // Inspector commits phase — Transform / Visibility / Name
        // / Sprite source-strategy + Reimport. Extracted to sibling
        // `inspector_commits.rs` as a free fn (Wave 3.2 stage A).
        //
        // W-J2: a physics joint's Position IS its A anchor, so the committed
        // pivot is re-seated through the bridge's anchor door just below.
        // Captured here because `transform_edit` is consumed by the call.
        let joint_pivot_commit = transform_edit.map(|info| (info.entity_bits, info.translation));
        if inspector_commits::dispatch(
            reimport_entity,
            transform_edit,
            &visibility_edits,
            name_edit,
            signal_edit,
            signal_leave_edit,
            &sprite_edits,
            &ordering_edits,
            &sampling_edits,
            &blend_edits,
            &slice_edits,
            &anchor_edits,
            &anim_edits,
            &timer_edits,
            &action_edits,
            &physics_edits,
            &visibility_section_edits,
            hero,
            sim,
            asset_db,
            atlas_asset_map,
            toasts,
            editor_queue,
            component_registry,
            *transform_type_id,
            *visibility_type_id,
            *name_type_id,
            *sprite_type_id,
        ) {
            self.title_dirty = true;
        }
        // ⭐⭐⭐ **A secção AUDIO** (TOP-20 #4) — na fase-filha, pelo tecto desta função e por
        // uma fronteira que o comentário dela já escrevia: as edições são de DUAS naturezas.
        inspector_queue_dirty |= audio_commits::aplicar(
            sim,
            &mut self.audio,
            editor_queue,
            component_registry,
            toasts,
            &audio_edits,
        );
        // ⭐⭐⭐ **A secção CAMERA** (TOP-20 #7, W3) — aqui pela MESMA razão da irmã de cima:
        // as edições são de DUAS naturezas. O `Preview` liga a VISTA (que só a `App` tem) e as
        // restantes escrevem um campo do documento. *Duas naturezas, um sítio que tem as duas.*
        for (bits, edit) in &camera_edits {
            if let ph2d_editor_core::CameraFieldEdit::Preview(on) = edit {
                self.game_camera_preview = *on;
                continue;
            }
            inspector_camera::apply_camera_edit(sim, *bits, edit, editor_queue, component_registry);
            inspector_queue_dirty = true;
        }
        // ⭐⭐⭐ **As secções FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3) — na fase-filha.
        inspector_queue_dirty |= factory_commits::aplicar(sim, tags, &factory_edits);
        // ⭐ **AS OITO SECÇÕES DO TOP-20 que cabem numa porta só** — na fase-filha, pelo tecto
        // desta função.
        inspector_queue_dirty |= top20_commits::aplicar(
            sim,
            &particles_edits,
            &hud_edits,
            &sequence_edits,
            &counter_watch_edits,
            &action_trigger_edits,
            &topdown_edits,
            &projectile_edits,
            &ray_edits,
            &statemachine_edits,
            &script_edits,
        );
        // ⭐⭐⭐ **A secção TAGS** (TOP-20 #9) — na fase-filha, pela mesma razão das irmãs acima e
        // pelo mesmo tecto de LOC (esta função chegou a `202` contra `200` ao ganhar o cérebro).
        // ⛔ *Partir por RESPONSABILIDADE, nunca subir o número* — e a fronteira já estava escrita
        // no comentário dela: **ela é a única secção do Inspector que escreve em DOIS documentos**,
        // e o segundo nem sequer está no mundo.
        inspector_queue_dirty |=
            tags_commits::aplicar(sim, tags, &tags_edits, editor_queue, component_registry);
        // ⭐⭐⭐ **O painel TAGS** (TOP-20 #9, W4) — os gestos sobre a ÁRVORE, na fase-filha.
        //
        // ⚠️ **Ela é um ficheiro irmão e não um bloco aqui**, e o tecto de LOC é que o disse: o
        // bloco levava esta função a `219` contra um tecto de `200`. ⛔ *Partir por
        // RESPONSABILIDADE, nunca subir o número* — e a fronteira já estava escrita no comentário
        // dele: **nenhum destes gestos é uma edição do Inspector**. Esta função chama-a porque é a
        // fase do quadro que tem as três coisas que ela pede (a árvore · o mundo · o ecrã).
        tag_tree_commits::aplicar(sim, tags, tags_problem, hero, &tag_tree_edits);
        self.title_dirty |= drena_a_fila(
            inspector_queue_dirty,
            sim,
            editor_queue,
            component_registry,
            toasts,
        );
        Some(joint_pivot_commit)
    }
}

/// ⭐ **A fila de comandos do editor, drenada** — devolve `true` quando o título fica sujo.
///
/// ⚠️ **Função livre e não um bloco na fase**, e o tecto de LOC é que o disse (a fase chegou a
/// `201` contra `200` ao ganhar a cutscene). ⛔ *Partir por RESPONSABILIDADE, nunca subir o
/// número* — e a fronteira já estava escrita: as linhas acima **aplicam edições de secção**, e esta
/// **drena uma fila de comandos** que elas encheram. Duas coisas, dois sítios.
fn drena_a_fila(
    sujo: bool,
    sim: &mut ph2d_ecs::SimWorld,
    editor_queue: &mut ph2d_ecs::scene::EditorCommandQueue,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> bool {
    if !sujo {
        return false;
    }
    let Err(e) = ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), editor_queue, registry)
    else {
        return false;
    };
    toasts.push(ph2d_editor_core::Toast::error(tr_with(
        "shell.fase_inspector_commits.audio_commit_failed",
        &[("e", &e)],
    )));
    true
}
