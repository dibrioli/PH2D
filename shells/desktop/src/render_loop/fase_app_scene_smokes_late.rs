//! **Fase do quadro: AS CENAS DE SMOKE QUE PEDEM A `App` INTEIRA — a 2.ª metade** (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! A metade que corre DEPOIS do pré-quadro do sculpt3d; partida da primeira só porque aquele assunto
//! está no meio da lista, e a ordem é o contrato.

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_app_scene_smokes_late(&mut self) {
        self.flip_self_overlap_smoke();
        self.flip_airbrush_smoke();
        self.flip_hardness_smoke();
        self.flip_resample_smoke();
        self.flip_pressure_smoke();
        self.flip_selection_smoke();
        self.flip_segment_smoke();
        self.blend_smoke();
        self.with_motion_scene(ph2d_app_motion::motion_node_path_smoke::motion_node_path_smoke);
        self.with_motion_scene(ph2d_app_motion::motion_object_smoke::motion_object_smoke);
        self.with_motion_scene(ph2d_app_motion::motion_shape_smoke::motion_shape_smoke);
        self.with_motion_scene(ph2d_app_motion::motion_autofix_smoke::motion_autofix_smoke);
        self.with_motion_scene(ph2d_app_motion::motion_delay_smoke::motion_delay_smoke);
        self.with_motion_scene(ph2d_app_motion::motion_fx_smoke::motion_fx_smoke);
        self.with_motion_scene(ph2d_app_motion::adapter_smoke::adapter_smoke);
        self.with_motion_scene(ph2d_app_motion::attribute_demo_smoke::attribute_demo_smoke);
        self.with_motion_scene(ph2d_app_motion::picker_smoke::picker_smoke);
        self.with_motion_scene(ph2d_app_motion::value_curve_smoke::value_curve_smoke);
        self.with_motion_scene(ph2d_app_motion::gradient_smoke::gradient_smoke);
        self.with_motion_scene(ph2d_app_motion::osc_ruler_smoke::osc_ruler_smoke);
        self.with_motion_scene(ph2d_app_motion::driven_row_smoke::driven_row_smoke);
        self.with_motion_scene(ph2d_app_motion::units_smoke::units_smoke);
        self.with_motion_scene(ph2d_app_motion::emitter_smoke::emitter_smoke);
        self.with_motion_scene(ph2d_app_motion::transform_family_smoke::transform_family_smoke);
        self.with_motion_scene(ph2d_app_motion::echo_family_smoke::echo_family_smoke);
        self.with_motion_scene(ph2d_app_motion::lens_smoke::lens_smoke);
        self.with_motion_scene(ph2d_app_motion::splice_smoke::splice_smoke);
        self.with_motion_scene(ph2d_app_motion::value_noise_smoke::value_noise_smoke);
        self.with_motion_scene(ph2d_app_motion::value_mix_smoke::value_mix_smoke);
        self.with_motion_scene(ph2d_app_motion::value_quantize_smoke::value_quantize_smoke);
        self.with_motion_scene(ph2d_app_motion::value_gain_smoke::value_gain_smoke);
        self.with_motion_scene(ph2d_app_motion::value_step_smoke::value_step_smoke);
        self.with_motion_scene(ph2d_app_motion::value_normalize_smoke::value_normalize_smoke);
        self.with_motion_scene(ph2d_app_motion::value_unary_smoke::value_unary_smoke);
        self.with_motion_scene(ph2d_app_motion::value_reduce_smoke::value_reduce_smoke);
        self.with_motion_scene(ph2d_app_motion::value_smooth_smoke::value_smooth_smoke);
        self.with_motion_scene(ph2d_app_motion::value_pattern_smoke::value_pattern_smoke);
        self.with_motion_scene(ph2d_app_motion::value_wrap_smoke::value_wrap_smoke);
        self.with_motion_scene(ph2d_app_motion::value_time_smoke::value_time_smoke);
        self.with_motion_scene(ph2d_app_motion::value_slope_smoke::value_slope_smoke);
        self.with_motion_scene(ph2d_app_motion::value_median_smoke::value_median_smoke);
        self.with_motion_scene(ph2d_app_motion::value_percentile_smoke::value_percentile_smoke);
        self.with_motion_scene(ph2d_app_motion::value_wave_smoke::value_wave_smoke);
    }
}
