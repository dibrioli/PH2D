//! **Fase do quadro: o ramo do `HeroScreen`** (`line/render-bodies`, 2026-09-13) — o bloco `if hero_screen.is_some()`
//! do `run_render_frame`, movido pela ordem de sempre e partido por ASSUNTO em quatro blocos que consomem os pedidos
//! do dreno ([`fase_bus_drain::DrainOut`]). Três trocas DECLARADAS, e só estas:
//! - o fim antecipado: `let Some(p) = f() else { return; };` passa a `let p = f()?;`;
//! - um pedido que uma intenção levava por ATALHO (`pending_x,`) SAI do dreno (`pending_x: take(&mut pd.pending_x),`
//!   — o `Default` fica no lugar, e o dreno não é lido outra vez), e uma leitura de pedido `Copy` lê `pd.x`;
//! - as devoluções das ferramentas de imagem viajam no `ImageToolBridgesOut` até ao bloco que as honra.

use super::*;
use std::mem::take;

/// O que HONRA os pedidos da Hierarquia, do Inspector, das folhas, da física e das ferramentas de imagem.
#[path = "fase_hero_commits.rs"]
mod hero_commits;
/// A cena: os espelhos da selecção, o assentar da árvore, as recozeduras vivas, as bandas, os overlays e a pintura.
#[path = "fase_hero_scene.rs"]
mod hero_scene;

/// O que o ramo sabe do ecrã e do vector ANTES dos verbos, e que mais de um bloco lê.
#[derive(Clone, Copy)]
pub(super) struct HeroView {
    pub(super) window_size: ph2d_host::WindowSize,
    pub(super) viewport: EditorRect,
    pub(super) vector_active: bool,
    pub(super) vec_px_to_world: f64,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hero_frame(
        &mut self,
        diag_input_events: u32,
        diag_paint_stamps: u32,
        tool_preview_bits: [Option<u64>; 3],
        report: ph2d_core::FixedStepReport,
        window_size: ph2d_host::WindowSize,
        viewport: EditorRect,
    ) -> Option<()> {
        let tool_preview_bits = self.fase_snapshots_publish(
            diag_input_events,
            diag_paint_stamps,
            tool_preview_bits,
            window_size,
        )?;
        let motion_tool_active = self.fase_gizmo_views_and_prefab(window_size)?;
        let mut pd = self.fase_bus_drain()?;
        self.fase_drain_leftovers(fase_drain_leftovers::DrainLeftoversIntents {
            bgremoval_leftover: take(&mut pd.bgremoval_leftover),
            painter_leftover: take(&mut pd.painter_leftover),
        });
        self.fase_image_tool_activation(fase_image_tool_activation::ImageToolActivationIntents {
            pending_image_tool_activation: take(&mut pd.pending_image_tool_activation),
        });
        self.fase_image_tools_mode_and_pills();
        let image_apply = self.fase_image_tool_bridges(window_size)?;
        let painter_apply_committed = self.fase_painter_dispatch(window_size, viewport)?;
        let (vector_active, vec_px_to_world) = self.fase_vector_scale()?;
        let view = HeroView {
            window_size,
            viewport,
            vector_active,
            vec_px_to_world,
        };
        self.fase_hero_document_verbs(&mut pd, report)?;
        self.fase_hero_tools(&mut pd, view)?;
        let reparent_intent =
            self.fase_hero_scene(&mut pd, view, motion_tool_active, tool_preview_bits)?;
        self.fase_hero_commits(
            &mut pd,
            view,
            reparent_intent,
            image_apply,
            painter_apply_committed,
        )
    }

    /// Os verbos do DOCUMENTO vectorial: o blend e o morph, o texto e o padrão no caminho, o contorno, os filtros, o
    /// esqueleto, o envelope, os efeitos, o perfil de largura, os estados de UI, os componentes e o expand.
    fn fase_hero_document_verbs(
        &mut self,
        pd: &mut fase_bus_drain::DrainOut,
        report: ph2d_core::FixedStepReport,
    ) -> Option<()> {
        self.fase_blend_and_morph(fase_blend_and_morph::BlendAndMorphIntents {
            pending_create_blend: take(&mut pd.pending_create_blend),
            pending_expand_blend: take(&mut pd.pending_expand_blend),
            pending_release_blend: take(&mut pd.pending_release_blend),
            pending_create_morph: take(&mut pd.pending_create_morph),
            pending_morph_t: take(&mut pd.pending_morph_t),
        });
        self.fase_text_on_path(fase_text_on_path::TextOnPathIntents {
            pending_textpath: take(&mut pd.pending_textpath),
            pending_textpath_offset: take(&mut pd.pending_textpath_offset),
            pending_text_pick: take(&mut pd.pending_text_pick),
        });
        self.fase_contour_verbs(fase_contour_verbs::ContourVerbsIntents {
            pending_contour: take(&mut pd.pending_contour),
            pending_contour_steps: take(&mut pd.pending_contour_steps),
            pending_contour_d: take(&mut pd.pending_contour_d),
            pending_contour_accel: take(&mut pd.pending_contour_accel),
            pending_contour_join: take(&mut pd.pending_contour_join),
            pending_contour_side: take(&mut pd.pending_contour_side),
        });
        // Filters (a PILHA de FX raster, plano 24): "Add" empurra um degrau, os ícones do card
        // reordenam/desarmam/apagam, e os sliders + o picker afinam a linha. Tudo pela porta
        // única `fx_live`; o `recook` do frame seguinte re-produz as imagens. Age sobre a
        // SELEÇÃO inteira — uma pilha por forma, o mesmo desenho do Contour/Offset.
        //
        // ⚠️ **Remover a última linha REMOVE o componente** — e isso mora dentro do
        // `fx_live::edit`, não aqui: uma regra escrita no chamador é uma regra que o próximo
        // chamador nasce sem.
        {
            let sel = self.fase_filter_commands(fase_filter_commands::FilterCommandsIntents {
                pending_filter_cmd: take(&mut pd.pending_filter_cmd),
                pending_filter_stop: take(&mut pd.pending_filter_stop),
            })?;
            self.fase_filter_values_and_colour(
                fase_filter_values_and_colour::FilterValuesAndColourIntents {
                    pending_filter_val: take(&mut pd.pending_filter_val),
                },
                sel,
            );
        }
        self.fase_pattern_path_and_pickers(
            fase_pattern_path_and_pickers::PatternPathAndPickersIntents {
                pending_patternpath: take(&mut pd.pending_patternpath),
                pending_pp_spacing: take(&mut pd.pending_pp_spacing),
                pending_pp_start: take(&mut pd.pending_pp_start),
                pending_pp_end: take(&mut pd.pending_pp_end),
                pending_pp_slide: take(&mut pd.pending_pp_slide),
                pending_pp_offset: take(&mut pd.pending_pp_offset),
                pending_pp_rotation: take(&mut pd.pending_pp_rotation),
                pending_pp_pick: take(&mut pd.pending_pp_pick),
                pending_texpat_pick: take(&mut pd.pending_texpat_pick),
                pending_brush_pick: take(&mut pd.pending_brush_pick),
                pending_brush: take(&mut pd.pending_brush),
            },
        );
        let mudos_antes = self.fase_skeleton_verbs(fase_skeleton_verbs::SkeletonVerbsIntents {
            pending_bone_bind: take(&mut pd.pending_bone_bind),
            pending_bone_release: take(&mut pd.pending_bone_release),
            pending_bone_knob: take(&mut pd.pending_bone_knob),
            pending_skin_law: take(&mut pd.pending_skin_law),
            pending_bone_rest: take(&mut pd.pending_bone_rest),
            osso_selecionado: pd.osso_selecionado,
            selecao_bits: take(&mut pd.selecao_bits),
        })?;
        if let Some(bits) = pd.osso_selecionado {
            let osso = ph2d_ecs::Entity::from_bits(bits);
            let osso = self.fase_bone_ik_and_limits(
                fase_bone_ik_and_limits::BoneIkAndLimitsIntents {
                    pending_ik_add: take(&mut pd.pending_ik_add),
                    pending_look_at: take(&mut pd.pending_look_at),
                    pending_ik_remove: take(&mut pd.pending_ik_remove),
                    pending_ik_bend: take(&mut pd.pending_ik_bend),
                    pending_bone_handles: take(&mut pd.pending_bone_handles),
                    pending_bone_tip: take(&mut pd.pending_bone_tip),
                    pending_limit_add: take(&mut pd.pending_limit_add),
                    pending_limit_remove: take(&mut pd.pending_limit_remove),
                },
                osso,
            )?;
            self.fase_bone_smart_and_knobs(
                fase_bone_smart_and_knobs::BoneSmartAndKnobsIntents {
                    pending_ik_knob: take(&mut pd.pending_ik_knob),
                    pending_limit_knob: take(&mut pd.pending_limit_knob),
                    pending_smart_add: take(&mut pd.pending_smart_add),
                    pending_smart_remove: take(&mut pd.pending_smart_remove),
                    pending_smart_knob: take(&mut pd.pending_smart_knob),
                    pending_smart_clip: take(&mut pd.pending_smart_clip),
                    pending_smart_pick: take(&mut pd.pending_smart_pick),
                },
                mudos_antes,
                osso,
            );
        } else if pd.pending_bone_needs_focus {
            // ⚠️ **Um verbo que morre em SILÊNCIO dá o mesmo sintoma que uma rota cortada** —
            // e foi exactamente esse o report de 2026-09-07 (*«Add IK não funciona»*), cuja
            // causa era outra. O painel só pinta estes botões com um osso em foco, então este
            // braço é a janela de UM quadro entre a publicação do painel e a leitura do dreno;
            // dizê-lo em voz alta é o que separa *«o app recusou»* de *«o botão está morto»*.
            //
            // ⛔⛔ **E a cura cobria DOIS dos oito verbos** (auditoria de 2026-09-08): o limite
            // e o osso inteligente foram acrescentados depois e não vieram a esta condição, logo
            // seis verbos voltaram a morrer calados exactamente na janela que este braço existe
            // para nomear. *Uma cura escrita para os verbos que existiam não segue os que vêm.*
            //
            // ⇒ hoje a condição é **DERIVADA das tabelas de ids** (`ids::needs_focused_bone`) e
            // cobre a seção INTEIRA — os dez verbos, os nove campos e as duas fileiras de chips.
            // Acrescentar um controlo põe-no do lado certo sem ninguém se lembrar deste braço;
            // quem age sobre as FORMAS declara-o em `VECTOR_BONE_ON_SELECTION`.
            eprintln!(
                "[ph2d-vec] osso: nenhum OSSO em foco -- seleccione um osso (na Hierarquia ou \
                 clicando nele com a ferramenta Bone) antes dos verbos da seccao Skeleton"
            );
        }
        self.fase_envelope(fase_envelope::EnvelopeIntents {
            pending_create_envelope: take(&mut pd.pending_create_envelope),
            pending_expand_envelope: take(&mut pd.pending_expand_envelope),
            pending_release_envelope: take(&mut pd.pending_release_envelope),
            pending_envelope_kind: take(&mut pd.pending_envelope_kind),
            pending_clear_pins: take(&mut pd.pending_clear_pins),
            pending_envelope_preset: take(&mut pd.pending_envelope_preset),
            pending_envelope_bend: take(&mut pd.pending_envelope_bend),
        });
        self.fase_path_effects_spine_bool(
            fase_path_effects_spine_bool::PathEffectsSpineBoolIntents {
                pending_vec_bool: take(&mut pd.pending_vec_bool),
                pending_reset_spine: take(&mut pd.pending_reset_spine),
                pending_blend_steps: take(&mut pd.pending_blend_steps),
                pending_fx_add: take(&mut pd.pending_fx_add),
                pending_fx_button: take(&mut pd.pending_fx_button),
                pending_fx_param: take(&mut pd.pending_fx_param),
                pending_fx_apply: take(&mut pd.pending_fx_apply),
            },
        );
        self.fase_live_offset_and_width(fase_live_offset_and_width::LiveOffsetAndWidthIntents {
            pending_width_preset: take(&mut pd.pending_width_preset),
        });
        self.fase_authored_controls_and_ui_states(
            fase_authored_controls_and_ui_states::AuthoredControlsAndUiStatesIntents {
                pending_widget_edit: take(&mut pd.pending_widget_edit),
                pending_ui_state: take(&mut pd.pending_ui_state),
            },
        );
        self.fase_ui_host_transition(fase_ui_host_transition::UiHostTransitionIntents {
            pending_ui_state_duration: take(&mut pd.pending_ui_state_duration),
            pending_ui_spring_toggle: take(&mut pd.pending_ui_spring_toggle),
            pending_ui_spring_knob: take(&mut pd.pending_ui_spring_knob),
            pending_ui_easing: take(&mut pd.pending_ui_easing),
            pending_ui_signal_edit: take(&mut pd.pending_ui_signal_edit),
            pending_ui_signal_name: take(&mut pd.pending_ui_signal_name),
        });
        self.fase_ui_state_preview(
            fase_ui_state_preview::UiStatePreviewIntents {
                pending_ui_preview_toggle: take(&mut pd.pending_ui_preview_toggle),
                pending_ui_move_all_toggle: take(&mut pd.pending_ui_move_all_toggle),
                pending_morph_preview_toggle: take(&mut pd.pending_morph_preview_toggle),
            },
            report,
        );
        self.fase_component_verbs(fase_component_verbs::ComponentVerbsIntents {
            pending_component: take(&mut pd.pending_component),
        });
        Some(())
    }

    /// As operações das ferramentas: o composto e o snap, os nós e o arranjo, o transform, o conector e a forma, o
    /// texto e as fontes, a tinta e a textura, o painel vector, os espelhos, o Flip, os overlays e a sincronização.
    fn fase_hero_tools(&mut self, pd: &mut fase_bus_drain::DrainOut, view: HeroView) -> Option<()> {
        let HeroView {
            window_size,
            viewport,
            vec_px_to_world,
            ..
        } = view;
        self.fase_vec_expand(fase_vec_expand::VecExpandIntents {
            pending_vec_expand: take(&mut pd.pending_vec_expand),
        })?;
        self.fase_compound_snap_rulers(fase_compound_snap_rulers::CompoundSnapRulersIntents {
            pending_vec_compound: take(&mut pd.pending_vec_compound),
            pending_vec_fill_rule: take(&mut pd.pending_vec_fill_rule),
            pending_vec_snap_on: take(&mut pd.pending_vec_snap_on),
            pending_vec_snap_path: take(&mut pd.pending_vec_snap_path),
            pending_vec_snap_cross: take(&mut pd.pending_vec_snap_cross),
            pending_vec_snap_guides: take(&mut pd.pending_vec_snap_guides),
            pending_rulers: take(&mut pd.pending_rulers),
        });
        self.fase_node_and_arrange_verbs(
            fase_node_and_arrange_verbs::NodeAndArrangeVerbsIntents {
                pending_vec_select_subpath: take(&mut pd.pending_vec_select_subpath),
                pending_vec_select_same: take(&mut pd.pending_vec_select_same),
                pending_vec_join: take(&mut pd.pending_vec_join),
                pending_vec_weld: take(&mut pd.pending_vec_weld),
                pending_vec_cut: take(&mut pd.pending_vec_cut),
                pending_vec_symmetry_apply: take(&mut pd.pending_vec_symmetry_apply),
                pending_vec_cut_discard: take(&mut pd.pending_vec_cut_discard),
                pending_vec_reverse: take(&mut pd.pending_vec_reverse),
                pending_vec_average: take(&mut pd.pending_vec_average),
                pending_vec_vertex_kind: take(&mut pd.pending_vec_vertex_kind),
                pending_vec_delete_vertex: take(&mut pd.pending_vec_delete_vertex),
                pending_vec_reorder: take(&mut pd.pending_vec_reorder),
                pending_vec_duplicate: take(&mut pd.pending_vec_duplicate),
                pending_vec_flip: take(&mut pd.pending_vec_flip),
                pending_vec_rotate: take(&mut pd.pending_vec_rotate),
            },
            vec_px_to_world,
        );
        let vec_xf_ops = self.fase_transform_ops(fase_transform_ops::TransformOpsIntents {
            pending_frame_preset: take(&mut pd.pending_frame_preset),
            pending_vec_opacity: take(&mut pd.pending_vec_opacity),
            pending_vec_blend: take(&mut pd.pending_vec_blend),
            pending_paint_verb: take(&mut pd.pending_paint_verb),
            pending_paint_width: take(&mut pd.pending_paint_width),
            pending_paint_dx: take(&mut pd.pending_paint_dx),
            pending_paint_dy: take(&mut pd.pending_paint_dy),
            pending_paint_dilate: take(&mut pd.pending_paint_dilate),
            pending_paint_join: take(&mut pd.pending_paint_join),
            pending_paint_opacity: take(&mut pd.pending_paint_opacity),
            pending_paint_blend: take(&mut pd.pending_paint_blend),
            pending_vec_transform: take(&mut pd.pending_vec_transform),
            pending_vec_vert: take(&mut pd.pending_vec_vert),
            pending_vec_rotate_by: take(&mut pd.pending_vec_rotate_by),
        })?;
        let vec_text_sel = self.fase_connector_and_shape_params(
            fase_connector_and_shape_params::ConnectorAndShapeParamsIntents {
                pending_vec_shape_param: take(&mut pd.pending_vec_shape_param),
                pending_vec_connector: take(&mut pd.pending_vec_connector),
            },
            vec_px_to_world,
        )?;
        let fase_text_fields::TextFieldsOut {
            editing_session,
            pending_vec_text_axis,
            vec_text_sel,
        } = self.fase_text_fields(
            fase_text_fields::TextFieldsIntents {
                pending_vec_text_size: take(&mut pd.pending_vec_text_size),
                pending_vec_text_weight: take(&mut pd.pending_vec_text_weight),
                pending_vec_text_line_height: take(&mut pd.pending_vec_text_line_height),
                pending_vec_text_tracking: take(&mut pd.pending_vec_text_tracking),
                pending_vec_text_wrap: take(&mut pd.pending_vec_text_wrap),
                pending_vec_text_align: take(&mut pd.pending_vec_text_align),
                pending_vec_text_axis: take(&mut pd.pending_vec_text_axis),
            },
            vec_text_sel,
        )?;
        self.fase_fonts(
            fase_fonts::FontsIntents {
                pending_vec_text_axis,
                pending_vec_font_cycle: take(&mut pd.pending_vec_font_cycle),
                pending_vec_font_pick: take(&mut pd.pending_vec_font_pick),
                pending_vec_font_import: take(&mut pd.pending_vec_font_import),
            },
            vec_text_sel,
            editing_session,
        );
        self.fase_path_shape_and_paint(fase_path_shape_and_paint::PathShapeAndPaintIntents {
            pending_vec_path_shape: take(&mut pd.pending_vec_path_shape),
            pending_vec_toggle_closed: take(&mut pd.pending_vec_toggle_closed),
            pending_vec_fill_kind: take(&mut pd.pending_vec_fill_kind),
            pending_vec_stroke_kind: take(&mut pd.pending_vec_stroke_kind),
        });
        let vec_xf_ops = self.fase_texpat_gradient_align(
            fase_texpat_gradient_align::TexpatGradientAlignIntents {
                pending_vec_pivot_edit: take(&mut pd.pending_vec_pivot_edit),
                pending_texpat: take(&mut pd.pending_texpat),
                pending_texpat_source: take(&mut pd.pending_texpat_source),
                pending_vec_grad_angle: take(&mut pd.pending_vec_grad_angle),
                pending_vec_grad_add: take(&mut pd.pending_vec_grad_add),
                pending_vec_grad_remove: take(&mut pd.pending_vec_grad_remove),
                pending_vec_grad_influence: take(&mut pd.pending_vec_grad_influence),
                pending_vec_grad_jitter: take(&mut pd.pending_vec_grad_jitter),
                pending_vec_grad_add_stop: take(&mut pd.pending_vec_grad_add_stop),
                pending_vec_grad_remove_stop: take(&mut pd.pending_vec_grad_remove_stop),
                pending_vec_align: take(&mut pd.pending_vec_align),
                pending_vec_distribute: take(&mut pd.pending_vec_distribute),
            },
            vec_xf_ops,
        )?;
        let (vec_cfg, vec_xf_ops) = self.fase_vector_panel_dispatch(
            fase_vector_panel_dispatch::VectorPanelDispatchIntents {
                pending_stroke_present: take(&mut pd.pending_stroke_present),
            },
            vec_px_to_world,
            vec_xf_ops,
        )?;
        self.fase_motion_bridge(vec_xf_ops);
        let vec_cfg = self.fase_tool_mirrors(vec_cfg)?;
        self.fase_world_panel_bridges();
        let (flip_active, flip_style) = self.fase_flip_strip_and_cursor(window_size)?;
        self.fase_physics_overlay(viewport);
        self.fase_canvas_overlays(window_size);
        self.fase_selection_highlight(flip_active, flip_style, viewport);

        self.fase_text_panel(vec_px_to_world);

        self.fase_entity_sync(vec_cfg);
        Some(())
    }
}
