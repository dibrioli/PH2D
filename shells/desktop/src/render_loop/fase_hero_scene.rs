//! **O ramo do `HeroScreen` — a cena.** Um bloco do [`fase_hero_frame`](super), movido pela ordem de sempre, com as
//! trocas declaradas lá.

use super::*;
use std::mem::take;

impl crate::App {
    /// Converter em curvas, os espelhos da selecção, os campos da forma, o assentar da árvore, as recozeduras vivas, a
    /// moldura e os estados, as bandas, os overlays, as guias, o modelador 3D e a pintura do ecrã. Devolve o pedido de
    /// reparentar que o assentar da árvore devolveu (o dreno da Hierarquia lê-o).
    pub(super) fn fase_hero_scene(
        &mut self,
        pd: &mut fase_bus_drain::DrainOut,
        view: HeroView,
        motion_tool_active: bool,
        tool_preview_bits: [Option<u64>; 3],
    ) -> Option<Option<ph2d_editor_core::screens::hero::HierReparentIntent>> {
        let HeroView {
            window_size,
            viewport,
            vector_active,
            vec_px_to_world,
        } = view;
        self.fase_convert_to_curves(fase_convert_to_curves::ConvertToCurvesIntents {
            pending_vec_convert: take(&mut pd.pending_vec_convert),
        });
        // Habilita "Convert to Curves" pela porta ÚNICA (`vec_convert::is_convertible`) — a
        // MESMA que o conversor honra. Enumerar as fontes aqui foi o que apodreceu duas
        // vezes: o botão ficava desligado num caminho só-efeitos e depois num só-quinas,
        // sempre sem erro nenhum. [[feedback_a_condition_that_enumerates_its_readers_rots]]
        #[cfg(feature = "panel-vector")]
        {
            let env_container = self.fase_selection_mirror_convert_envelope()?;
            self.fase_selection_mirror_skin();
            self.fase_selection_mirror_bone_focus();
            let sel = self.fase_selection_mirror_path_links()?;
            self.fase_selection_mirror_filters(sel);
            self.fase_selection_mirror_effects_envelope(env_container);
        }
        self.fase_shape_fields(vec_px_to_world);
        self.fase_vector_upkeeps();
        let (drawing, reparent_intent) =
            self.fase_vector_tree_settle(fase_vector_tree_settle::TreeSettleIntents {
                reparent_intent: take(&mut pd.reparent_intent),
            })?;
        let (vec_view, vec_xf) = self.fase_vector_view_and_drives(vector_active)?;
        let (cam_affine, vec_xf, vec_view) =
            self.fase_vector_live_recooks(window_size, vector_active, vec_view, vec_xf)?;
        let (vec_live, vec_view, mut vec_xf) =
            self.fase_vector_live_geometry(window_size, drawing, vec_view, vec_xf)?;
        // **O Apply corre AQUI, e não no dreno**, porque ele materializa o `plan` que o
        // `recook` acabou de computar — *o que está na tela*. Chamar o motor de novo lá em
        // cima seria a segunda porta, e ela faria a forma SALTAR no clique.
        //
        // ⚠️ A shell publica também *"há grupo booleano selecionado?"* para o painel decidir
        // se oferece o botão: o painel não alcança o mundo ECS, e uma segunda resposta a essa
        // pergunta seria um Apply pintado sobre uma seleção que não tem o que consolidar.
        {
            let sel = self.fase_vector_selection_frame_panel(
                fase_vector_selection_frame_panel::FrameLayoutIntents {
                    pending_frame_clip: take(&mut pd.pending_frame_clip),
                    pending_layout_edit: take(&mut pd.pending_layout_edit),
                    pending_anchor_edit: take(&mut pd.pending_anchor_edit),
                    pending_layout_field: take(&mut pd.pending_layout_field),
                    pending_vec_z: take(&mut pd.pending_vec_z),
                },
            )?;
            let sel = self.fase_vector_selection_states_panel(
                fase_vector_selection_states_panel::ResizeBoxIntents {
                    pending_resize_box: take(&mut pd.pending_resize_box),
                },
                sel,
            )?;
            let (vec_xf_back, sel) = self.fase_vector_tokens_and_labels(
                fase_vector_tokens_and_labels::TokenBindIntents {
                    pending_token_bind: take(&mut pd.pending_token_bind),
                },
                vec_xf,
                sel,
            )?;
            vec_xf = vec_xf_back;
            let (group, sel) = self.fase_vector_bool_shape_row(
                fase_vector_bool_shape_row::BoolShapeIntents {
                    pending_bool_shape_op: take(&mut pd.pending_bool_shape_op),
                },
                sel,
            )?;
            self.fase_vector_morph_verbs(
                fase_vector_morph_verbs::MorphVerbIntents {
                    pending_morph_arrow: take(&mut pd.pending_morph_arrow),
                },
                sel,
            );
            self.fase_vector_bool_apply_and_morph_reconcile(
                fase_vector_bool_apply_and_morph_reconcile::BoolApplyIntents {
                    pending_bool_apply: take(&mut pd.pending_bool_apply),
                },
                group,
            );
        }
        let (vec_view, vec_live, vec_xf) =
            self.fase_vector_layout_recook(vec_view, vec_xf, vec_live)?;
        let (vec_view, vec_xf, cam_affine, vec_live) =
            self.fase_vector_fx_recook(vec_view, vec_xf, cam_affine, vec_live)?;
        let (vec_view, vec_xf, cam_affine) =
            self.fase_vector_bands(vec_view, vec_xf, cam_affine, vec_live, viewport)?;
        let (overlay, vec_xf, cam_affine) =
            self.fase_vector_overlays(motion_tool_active, vector_active, vec_xf, cam_affine)?;
        let (vec_xf, cam_affine) =
            self.fase_vector_edit_overlay(vec_view, vec_xf, cam_affine, overlay)?;
        let cam_affine = self.fase_vector_bone_overlay(vec_px_to_world, cam_affine, overlay)?;
        let cam_affine = self.fase_vector_guides_and_build(
            tool_preview_bits,
            window_size,
            vec_xf,
            cam_affine,
            overlay,
            viewport,
        )?;
        self.fase_vector_tool_handles(vector_active, cam_affine, overlay);
        self.fase_gizmo_suppression_and_field3d_frame(viewport);
        self.fase_field3d_smoke_draw(viewport);
        self.fase_field3d_requests();
        self.fase_hero_paint(viewport);
        Some(reparent_intent)
    }
}
