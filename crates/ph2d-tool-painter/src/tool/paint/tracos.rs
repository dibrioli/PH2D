//! **Pinceladas sobrepostas** (W16, `docs/Macros/14_W16_plano.md` §2.3): a macro que toca duas pinceladas
//! ao mesmo tempo guarda o traço de cada faixa num [`TracoGuardado`] e troca-o com o do tool à vez, antes
//! de cada passo dela. Só o estado DO TRAÇO se troca ([`DO_TRACO`]); a tela, as camadas, o desfazer, a
//! aguada, o relevo e a água são partilhados ([`PARTILHADO`]) — dois pincéis na mesma tinta — e o que
//! fecha um traço partilhado espera pelo último (`PaintState::tracos_fora`).
//!
//! ⛔ Um campo novo do `PaintState` tem de entrar numa das duas listas (gate
//! `cada_campo_do_paint_state_e_do_traco_ou_partilhado`): um esquecido seria escrito por dois traços.

use super::state::PaintState;
use super::watercolor_solido::ManchaNaAguada;
use super::wetpaint::faixas::LanesGuardadas;
use crate::tool::PainterTool;

macro_rules! campos_do_traco {
    ($($c:ident),* $(,)?) => {
        /// Os campos do `PaintState` que são de UM traço aberto (o censo lê-os).
        #[cfg(test)]
        pub(crate) const DO_TRACO: &[&str] = &[$(stringify!($c)),*];

        fn troca_campos(a: &mut PaintState, b: &mut PaintState) {
            $(std::mem::swap(&mut a.$c, &mut b.$c);)*
        }
    };
}

campos_do_traco!(
    stroke,
    dabs,
    solid_path,
    solid_fill_owed,
    pending_threads,
    tex_rng,
    brush,
    dynamics,
    stroke_authored,
    line_anchor,
    last_smear_pos,
    moved_this_frame,
    drag_preview,
    rng_camada,
    composite_mask,
    composite_arco,
    pilha,
    limite_do_smear,
    clone_offset,
    stroke_mask,
    per_layer_stroke,
    dab_groups,
    wet_smear_pos,
    wet_level_smear_pos,
    wet_stroke_dirty,
    wet_mix,
    wet_soak_pos,
);

/// Os campos do `PaintState` partilhados pelos traços abertos (a tela, as sessões, o relevo, a aguada,
/// as preferências).
#[cfg(test)]
pub(crate) const PARTILHADO: &[&str] = &[
    "hover_heading",
    "hover_pos",
    "seed",
    "stroke_undo",
    "tracos_fora",
    "eraser",
    "paint_mode",
    "brush_by_mode",
    "link_shared_settings",
    "line_show_dimensions",
    "mask_brush",
    "mask_overlay_color",
    "eyedropper_armed",
    "falloff_armed",
    "mask_scratch_rgba",
    "mask_scratch_target",
    "selection_mask",
    "selection_active",
    "selection_mode",
    "selection_bool_op",
    "selection_threshold",
    "selection_drag",
    "selection_pen",
    "selection_base",
    "selection_crisp",
    "selection_stabilizer",
    "selection_feather",
    "selection_edit_mode",
    "selection_overlay_opacity",
    "selection_shapes",
    "selection_raster_cache",
    "selection_grab",
    "selection_clipboard",
    "paste_patch",
    "paste_pristine",
    "paste_grab",
    "selection_offset_norm",
    "selection_offset_active",
    "selection_offset_rings",
    "selection_offset_source",
    "selection_offset_curves",
    "selection_offset_level_cache",
    "selection_ring_stack",
    "composite_enabled",
    "composite",
    "composite_len",
    "composite_add_op",
    "clone_source",
    "clone_aligned",
    "clone_sample_armed",
    "smear_scratch",
    "tiling",
    "repeat_image",
    "symmetry_pick",
    "symmetry_line_start",
    "symmetry_auto_center",
    "shape_draft",
    "shape_draft_hold",
    "shape_stale",
    "line_constrain",
    "scale_uniform",
    "line_snap",
    "grid_snap_pos",
    "last_non_shape_method",
    "method_by_media",
    "curve",
    "ellipse",
    "line",
    "polygon",
    "parked_shapes",
    "active_op",
    "stroke_op_mode",
    "op_tap",
    "shape_edit_wrap",
    "selection_op_tap",
    "gizmo_mods",
    "shape_grab_tol_px",
    "canvas_warp",
    "shape_offset_norm",
    "shape_offset_base_px",
    "offset_trim",
    "stencil_grab",
    "stencil_preview_s",
    "texture_image",
    "paper_image",
    "paper_image_version",
    "flow_map",
    "texture_image_pending",
    "texture_image_version",
    "shape_image",
    "shape_image_pending",
    "shape_image_version",
    "shape_layers",
    "stamp_cache",
    "color_stamp_cache",
    "ramp_color_stamp_cache",
    "canvas_tex_cache",
    "texture_ramp",
    "texture_ramp_enabled",
    "texture_ramp_bw",
    "texture_ramp_lut",
    "texture_ramp_dirty",
    "ramp_lut_version",
    "shape_camadas",
    "shape_color_ramp",
    "shape_color_ramp_enabled",
    "shape_color_ramp_bw",
    "shape_ramp_lut",
    "shape_ramp_dirty",
    "shape_ramp_version",
    "ramp_lut_owner",
    "pilha_por_replay",
    "acumulando_no_plano",
    "relief",
    "stroke_coverage",
    "stroke_color",
    "wet_mistura",
    "watercolor_base",
    "wet_backdrop",
    "paper_color",
    "impasto_show",
    "substrate_depth",
    "substrate_rough",
    "shape_relief",
    "grid_show",
    "grid_erase",
    "impasto_rig",
    "impasto_live_edit",
    "wet_soak",
    "wet_soak_active",
    "stroke_density",
    "stroke_deplete",
    "stroke_deplete_prox",
    "wet_reserve_cache",
    "canvas_wet",
    "canvas_wet_rect",
    "canvas_wet_snapshot",
    "canvas_wet_carry",
    "dry_rate_per_s",
    "wet_preview_intensity",
    "wet_session_wetness",
    "wet_shape_active",
    "wet_styles",
    "stroke_water",
    "wet_session_base",
    "wet_session_canvas",
    "wet_editable_base",
    "wet_editable_backdrop",
    "wet_editable_region",
    "wet_editable_tex",
    "wet_shape_norm",
    "wet_substrate",
    "wet_frame_dirty",
    "wet_frame_linhas",
    "wet_cum_dirty",
    "inpaint_mask",
    "fill_threshold",
    "fill_seed",
    "fill_snapshot",
    "fill_last_rect",
    "fill_return_mode",
    "inpaint_patch_norm",
    "inpaint_quality_norm",
    "inpaint_search_norm",
    "shape_color_preview",
    "deform",
    "warp",
    "sculpt",
    "wetpaint",
];

/// O traço de uma faixa enquanto outro está no tool.
#[derive(Clone)]
pub(crate) struct TracoGuardado {
    p: Box<PaintState>,
    gesto_vivo: bool,
    deposito: Vec<ph2d_painter_brush::Dab>,
    mancha: Option<ManchaNaAguada>,
    lanes: LanesGuardadas,
}

impl PainterTool {
    /// **Um traço novo por abrir**, com o pincel `pincel` (o da etapa no instante gravado dela).
    #[must_use]
    pub(crate) fn traco_novo(&self, pincel: &PincelDoTraco) -> TracoGuardado {
        let mut p = Box::<PaintState>::default();
        p.brush = pincel.brush;
        p.dynamics = pincel.dynamics;
        TracoGuardado {
            p,
            gesto_vivo: false,
            deposito: Vec::new(),
            mancha: None,
            lanes: LanesGuardadas::default(),
        }
    }

    /// **O pincel em vigor** — o que um traço aberto agora levaria.
    #[must_use]
    pub(crate) fn pincel_do_traco(&self) -> PincelDoTraco {
        PincelDoTraco {
            brush: self.paint.brush,
            dynamics: self.paint.dynamics,
            meio: self.paint_media(),
        }
    }

    /// Troca o traço do tool com `g` (o do tool passa a estar guardado em `g`).
    pub(crate) fn troca_o_traco(&mut self, g: &mut TracoGuardado) {
        troca_campos(&mut self.paint, &mut g.p);
        std::mem::swap(&mut self.paint.wetpaint.live_gesture, &mut g.gesto_vivo);
        std::mem::swap(&mut self.paint.wetpaint.pending_deposit, &mut g.deposito);
        std::mem::swap(&mut self.mancha_na_aguada, &mut g.mancha);
        self.wet_troca_as_lanes(&mut g.lanes);
    }

    /// **Um toque deste pincel é uma pincelada simples** — o pen-down abre um traço do motor (o modo
    /// Paint ou o Wet Paint, sem uma forma a desenhar): só estas se sobrepõem a outra (W16).
    #[must_use]
    pub(crate) fn toque_simples(&self) -> bool {
        use super::PaintMode;
        use ph2d_painter_brush::StrokeMethod;
        matches!(
            self.paint.paint_mode,
            PaintMode::Paint | PaintMode::WetPaint
        ) && !matches!(
            self.paint.brush.stroke_method,
            StrokeMethod::Arc
                | StrokeMethod::FreeHand
                | StrokeMethod::Ellipse
                | StrokeMethod::Polygon
                | StrokeMethod::Line
        )
    }

    /// Há um traço aberto no tool.
    #[must_use]
    pub(crate) fn traco_aberto(&self) -> bool {
        self.paint.stroke.is_some()
    }

    /// Quantos traços estão abertos FORA do do tool.
    pub(crate) fn poe_tracos_fora(&mut self, n: u32) {
        self.paint.tracos_fora = n;
    }

    /// O quadro do traço do tool, sem o que é do quadro inteiro (o tique da água, a secagem): o tocador
    /// chama-o para cada traço guardado depois do `on_tick`.
    pub(crate) fn tique_do_traco(&mut self, dt_ms: f32) {
        self.paint_tick_do_traco(dt_ms * 1e-3);
    }
}

impl TracoGuardado {
    #[must_use]
    pub(crate) fn aberto(&self) -> bool {
        self.p.stroke.is_some()
    }

    /// O quadro já correu neste traço depois do último movimento dele.
    #[cfg(test)]
    pub(crate) fn quadro_corrido(&self) -> bool {
        !self.p.moved_this_frame
    }
}

/// O pincel que uma etapa leva para a sua faixa: o do instante GRAVADO dela.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PincelDoTraco {
    brush: ph2d_painter_brush::BrushSpec,
    dynamics: ph2d_painter_brush::Dynamics,
    pub(crate) meio: crate::PaintMedia,
}

#[cfg(test)]
#[path = "tracos_tests.rs"]
mod tests;
