//! **Fase do quadro: AS CENAS DE SMOKE QUE PEDEM A `App` INTEIRA** (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! Os roteadores de cena que correm antes do empréstimo do `gfx`, pela ordem de sempre — a ordem é o
//! contrato, porque duas cenas podem armar a mesma ferramenta e a última a correr ganha.

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_app_scene_smokes(&mut self) {
        // **Shape Builder:** abre/renova o arranjo das formas selecionadas. Roda ANTES do
        // borrow mutável do `gfx` (ele precisa ler a cena e escrever em `self`), e só
        // reconstrói quando a SELEÇÃO muda — refazê-lo por frame jogaria fora o memo do
        // arranjo e cada hover voltaria a pagar a booleana.
        self.build_smoke();
        self.field3d_undo_probe();
        #[cfg(feature = "sculpt3d")]
        self.sculpt3d_undo_probe();
        self.stack_smoke();
        self.with_motion_scene(ph2d_app_motion::motion_path_smoke::motion_path_smoke);
        self.harmony_smoke();
        self.timeline_onion_smoke();
        self.signal_smoke();
        self.timer_smoke();
        self.signal_action_smoke();
        self.tags_smoke();
        self.factory_smoke();
        self.topdown_smoke();
        self.statemachine_smoke();
        self.script_smoke();
        self.projectile_smoke();
        self.particles_smoke();
        // ⭐⭐⭐ **A CUTSCENE** (TOP-20 #19) — ela autora a timeline, logo corre DEPOIS das cenas
        // que também lhe tocam (a ordem é o contrato: a última a correr ganha).
        self.sequence_smoke();
        self.counter_watch_smoke();
        self.trigger_smoke();
        self.dano_smoke();
        self.ray_smoke();
        self.tween_smoke();
        // ⭐⭐⭐ **O HUD** (TOP-20 #20) — DEPOIS da câmera, que ela compõe (ver o cabeçalho).
        self.hud_smoke();
        #[cfg(feature = "panel-audio-editor")]
        self.audio_2d_smoke();
        self.game_camera_smoke();
        self.ui_motion_smoke();
        self.timescale_smoke();
        self.stagger_smoke();
        self.buffer_smoke();
        self.extrap_smoke();
        self.expr_blend_smoke();
        self.morph_fade_smoke();
        self.vec_fade_smoke();
        self.vec_appearance_smoke();
        self.svg_import_smoke();
        self.vec_stack_smoke();
        self.vec_bone_smoke();
        self.bone_undo_probe();
        self.bone_smart_probe();
        self.nest_smoke();
        self.physics_smoke();
        self.instance_smoke();
        self.flip_pose_smoke();
        self.flip_edit_smoke();
        self.flip_fill_smoke();
        self.flip_colorize_smoke();
        self.flip_tween_smoke();
        self.flip_tween_pairs_smoke();
        self.flip_strip_smoke();
        self.flip_tween_phase_smoke();
        self.flip_tween_torsion_smoke();
        self.flip_tip_smoke();
        self.flip_multiplane_smoke();
        #[cfg(feature = "sculpt3d")]
        self.sculpt3d_smoke();
    }
}
