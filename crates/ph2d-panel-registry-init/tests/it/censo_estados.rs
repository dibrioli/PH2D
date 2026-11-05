//! ⭐ **Os estados que o censo da compactação semeia** (R4). `paineis_armados::TABELA` dá UM estado por
//! painel e é lida por outras réguas; aqui há vários por painel (seleção, documento, modos), e só a
//! compactação os lê. Medido em 04/11: de fábrica, `vector` tinha 6 linhas e `skeleton` 2: o zero deles
//! não era aprovação. Um estado novo é um painel que passa a ser visto: o que ele acusar vai a `PENDENTES`.

use std::collections::BTreeMap;

use ph2d_editor_core::widget::key_mark::set_key_marks;

/// Um estado semeado de um painel, e como se tira.
pub struct Estado {
    pub painel: &'static str,
    pub arma: fn(),
    pub desarma: fn(),
}

pub const ESTADOS: &[Estado] = &[
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::molhado,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::aguada,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::impasto,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::mascara,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::deform,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::selecao,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::composto,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::esculpe,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::linha,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::clone,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::inpaint,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-painter-layers")]
    Estado {
        painel: "painter_layers",
        arma: pintor::camadas,
        desarma: pintor::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::transformar,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::forma,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::efeitos,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::no_e_padrao,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::texto,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-vector")]
    Estado {
        painel: "vector",
        arma: vector::avancado,
        desarma: vector::limpa,
    },
    #[cfg(feature = "panel-skeleton")]
    Estado {
        painel: "skeleton",
        arma: osso::em_foco,
        desarma: osso::limpa,
    },
    #[cfg(feature = "panel-physics")]
    Estado {
        painel: "physics",
        arma: || ph2d_panel_physics::set_current_physics(Some(Default::default())),
        desarma: || ph2d_panel_physics::set_current_physics(None),
    },
    #[cfg(feature = "panel-flip-frames")]
    Estado {
        painel: "flip_frames",
        arma: tira::com_camada,
        desarma: tira::limpa,
    },
    #[cfg(feature = "panel-audio-editor")]
    Estado {
        painel: "audio_editor",
        arma: audio::com_clip,
        desarma: audio::limpa,
    },
];

#[cfg(feature = "panel-flip-frames")]
mod tira {
    use ph2d_panel_flip_frames::{FlipCell, FlipStripSnapshot, set_current_flip_strip};

    pub fn com_camada() {
        set_current_flip_strip(FlipStripSnapshot {
            has_layer: true,
            layer_name: "Ink".to_owned(),
            fps: 24.0,
            ghost: true,
            ghost_before: 2,
            ghost_after: 1,
            tween_count: 3,
            cells: (0..6)
                .map(|i| FlipCell {
                    key: i * 4,
                    exposure: 4,
                    breakdown: false,
                    instanced: i == 3,
                    selected: i == 2,
                    pinned: i == 4,
                    weight: 0.5,
                })
                .collect(),
            current_key: Some(4),
            current_frame: 6,
            ..Default::default()
        });
    }

    pub fn limpa() {
        set_current_flip_strip(FlipStripSnapshot::default());
    }
}

#[cfg(feature = "panel-vector")]
mod vector {
    use super::{BTreeMap, set_key_marks};
    use ph2d_panel_vector as v;
    use ph2d_panel_vector::state;

    pub fn limpa() {
        state::set_current_transform(None);
        state::set_current_selection_count(0);
        state::set_current_fill(None, None);
        state::set_current_appearance(None);
        v::set_current_effects(false, false, &[], Vec::new());
        v::set_current_filters(Vec::new());
        v::set_current_filter_can_add(false);
        v::set_current_vertex_pos(None);
        v::set_current_vertex_count(0);
        v::set_selected_vertex_type(None);
        v::set_current_connector(None);
        v::set_current_brush(None);
        v::set_current_texture_pattern(0, None);
        v::set_current_texture_pattern(1, None);
        v::set_current_text_visible(false);
        v::set_current_text(None);
        v::set_current_text_axes(Vec::new());
        v::set_current_has_envelope(false);
        v::set_current_patternpath_can_link(false);
        v::set_current_contour_can_add(false);
        set_key_marks(BTreeMap::new());
    }

    pub fn transformar() {
        state::set_length_suffix("px");
        state::set_current_transform(Some([10.0, 20.0, 100.0, 40.0]));
        state::set_resize_box(Some(false));
        state::set_frame_present(true);
        state::set_frame_clip(Some(false));
        state::set_frame_panel_open(false);
        state::set_current_snap(false);
    }

    pub fn forma() {
        state::set_length_suffix("px");
        state::set_current_selection_count(1);
        state::set_current_fill(Some(v::FillKind::Solid), Some(30.0));
        state::set_stroke_present(Some(true));
        state::set_stroke_above_effects(Some(false));
        state::set_stroke_paint_kind(Some(state::StrokePaintKind::Solid));
    }

    pub fn efeitos() {
        let parametro = |name, max, integer, value| v::FxParamView {
            name,
            min: 0.0,
            max,
            toggle: false,
            integer,
            value,
        };
        let efeito = |label| v::FxRowView {
            label,
            enabled: true,
            params: vec![
                parametro("Size", 100.0, false, 12.0),
                parametro("Ridges", 128.0, true, 8.0),
                parametro("Bend", 1.0, false, 0.25),
            ],
            falloff_role: v::FalloffRole::NotFalloff,
        };
        v::set_current_effects(
            true,
            false,
            &["Zig Zag", "Wave"],
            vec![efeito("Zig Zag"), efeito("Wave")],
        );
    }

    pub fn no_e_padrao() {
        use ph2d_tool_vector::{VertexSel, VertexType};
        let padrao = || v::TexturePatternRow {
            kind: 1,
            offset_denom: 3.0,
            size: [2.0, 1.5],
            lock_aspect: true,
            gap: [0.25, -0.25],
            link_gap: false,
            angle_deg: 30.0,
            shift_pct: [10.0, 20.0],
            mode: 0,
            wrap_seam_visible: true,
            art: ph2d_vec_pattern::PatternArt::Ready,
        };
        v::set_selected_vertex_type(Some(VertexSel::Uniform(VertexType::Smooth)));
        v::set_current_vertex_pos(Some([1.5, -2.5]));
        v::set_current_vertex_count(2);
        v::set_current_connector(Some(v::ConnectorSnapshot {
            route: 1,
            jetty: 0.25,
            spread: 0.5,
            corner: 0.125,
            curve: 0.75,
        }));
        v::set_current_brush(Some(v::BrushRow {
            has_art: true,
            spacing: 1.5,
            scale: 2.5,
            offset: -0.5,
            rotation_deg: 45.0,
            flip: false,
        }));
        v::set_current_texture_pattern(0, Some(padrao()));
        v::set_current_texture_pattern(1, Some(padrao()));
    }

    pub fn texto() {
        let eixo = |name: &str| v::TextAxisSlot {
            name: name.to_owned(),
            min: 0.0,
            max: 100.0,
            value: 25.0,
        };
        state::set_length_suffix("px");
        v::set_current_text_visible(true);
        v::set_current_text(Some("Hello\nworld".to_owned()));
        v::set_current_text_font(Some("Inter".to_owned()));
        v::set_current_text_has_weight(true);
        v::set_current_text_wrap(Some(6.0));
        v::set_current_text_axes(vec![eixo("Width"), eixo("Slant"), eixo("Optical Size")]);
        v::set_current_textpath(false, 0.0, false);
        v::set_current_textpath_can_link(true);
    }

    pub fn avancado() {
        v::set_current_selection_count(2);
        v::set_current_has_envelope(true);
        v::set_current_envelope_mode(2);
        v::set_current_envelope_presets(&["Arc", "Bulge", "Wave"], Some(1), 0.25);
        v::set_current_patternpath(true, 0.25, 0.75, 1.5, 0.5, false, 30.0);
        v::set_current_patternpath_can_link(true);
        v::set_current_contour(true, 6.0, 0.125, 2.0, 1, 2, [16, 32, 48, 255]);
        v::set_current_contour_can_add(true);
    }
}

#[cfg(feature = "panel-skeleton")]
mod osso {
    use ph2d_panel_skeleton::{self as skel, SmartBoneView, TipView};

    pub fn em_foco() {
        super::super::o_cartao_solto_no_canvas::osso_em_foco(1, "Upper Arm", 0.5);
        skel::set_current_bone_smart(Some(SmartBoneView {
            from: 0.0,
            to: 90.0,
            clip: "Wave".to_owned(),
            target: String::new(),
            picking: false,
        }));
        skel::set_current_bone_actions(vec!["Wave".to_owned(), "Idle".to_owned()]);
        skel::set_current_bone_tip(Some(TipView {
            rotulos: vec!["Hand".to_owned(), "Elbow".to_owned()],
            ligado: 0,
            escondidos: 0,
        }));
    }

    pub fn limpa() {
        skel::state::set_current_bone(None);
        skel::set_current_bone_subject(None);
        skel::set_current_bone_ik(None);
        skel::set_current_bone_smart(None);
        skel::set_current_bone_actions(Vec::new());
        skel::set_current_bone_tip(None);
    }
}

#[cfg(feature = "panel-audio-editor")]
mod audio {
    use ph2d_panel_audio_editor as a;

    pub fn com_clip() {
        a::set_loaded(true);
        a::set_clip_name("Footsteps");
        a::set_duration_secs(4.0);
        a::set_position_secs(1.0);
        a::set_has_selection(true);
        a::set_can_undo(true);
    }

    pub fn limpa() {
        a::set_loaded(false);
        a::set_has_selection(false);
        a::set_can_undo(false);
    }
}

/// O Painter de fábrica só mostra um meio: cada meio e cada ferramenta dona do corpo é população (medido em 04/11: 138
/// linhas sem estes estados, em 2 pinturas, e Máscara, Deform, Selection e Layers não eram vistas).
#[cfg(feature = "panel-painter-layers")]
mod pintor {
    use ph2d_panel_painter_layers::FALLBACK_BRUSH;
    use ph2d_panel_painter_layers::state::{
        set_current_brush, set_current_dock_shows_layers, set_current_layers,
    };
    use ph2d_tool_painter::{BrushSettings, LayerStack, LineKind, PaintMedia};

    fn com(muda: impl FnOnce(&mut BrushSettings)) {
        let mut b = FALLBACK_BRUSH;
        muda(&mut b);
        set_current_brush(Some(b));
    }

    pub fn molhado() {
        com(|b| {
            b.media = PaintMedia::WetPaint.to_u8();
            b.wetpaint = true;
        });
    }

    pub fn aguada() {
        com(|b| {
            b.media = PaintMedia::Watercolor.to_u8();
            b.watercolor = true;
            b.watercolor_active = true;
        });
    }

    pub fn impasto() {
        com(|b| {
            b.media = PaintMedia::Impasto.to_u8();
            b.impasto = true;
            b.impasto_applies = true;
        });
    }

    pub fn mascara() {
        com(|b| b.is_mask = true);
    }

    pub fn deform() {
        com(|b| {
            b.is_deform = true;
            b.deform_temperament = ph2d_tool_painter::DEFORM_TEMPERAMENT_RESHAPE;
        });
    }

    pub fn selecao() {
        com(|b| {
            b.is_selection = true;
            b.selection_edit = true;
        });
    }

    pub fn composto() {
        com(|b| {
            b.composite_enabled = true;
            b.composite_len = 3;
            b.composite_ops[1] = 2;
            b.composite_ops[2] = 3;
        });
    }

    pub fn esculpe() {
        com(|b| {
            b.media = PaintMedia::Impasto.to_u8();
            b.impasto = true;
            b.impasto_applies = true;
            b.is_sculpt = true;
            b.sculpt_knob_family = 1;
            b.sculpt_is_chisel = true;
            b.sculpt_filters = true;
        });
    }

    pub fn linha() {
        com(|b| b.line_kind = LineKind::Sketchy.to_wire());
    }

    pub fn clone() {
        com(|b| b.is_clone = true);
    }

    pub fn inpaint() {
        com(|b| b.is_inpaint = true);
    }

    pub fn camadas() {
        let mut s = LayerStack::new();
        let _ = s.add_raster("Base", 8, 8);
        let _ = s.add_raster("Top", 8, 8);
        set_current_layers(Some(s));
        set_current_dock_shows_layers(true);
    }

    pub fn limpa() {
        set_current_brush(None);
        set_current_layers(None);
        set_current_dock_shows_layers(false);
    }
}
