//! **Fase do quadro: OS CAMPOS DE TEXTO** — tamanho, peso, entrelinha, quebra, tracking, eixo e alinhamento — na sessão viva ou no
//! objecto de texto seleccionado (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct TextFieldsIntents {
    pub(super) pending_vec_text_size: Option<f64>,
    pub(super) pending_vec_text_weight: Option<f32>,
    pub(super) pending_vec_text_line_height: Option<f64>,
    pub(super) pending_vec_text_tracking: Option<f64>,
    pub(super) pending_vec_text_wrap: Option<Option<f64>>,
    pub(super) pending_vec_text_align: Option<ph2d_vec_text::TextAlign>,
    pub(super) pending_vec_text_axis: Option<(usize, f64)>,
}
/// O que os campos de texto devolvem ao quadro: se havia uma sessão viva, o pedido do eixo e a selecção de texto.
pub(super) struct TextFieldsOut {
    pub(super) editing_session: bool,
    pub(super) pending_vec_text_axis: Option<(usize, f64)>,
    pub(super) vec_text_sel: Vec<ph2d_vec_scene::VecPathId>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_text_fields(
        &mut self,
        intents: TextFieldsIntents,
        vec_text_sel: Vec<ph2d_vec_scene::VecPathId>,
    ) -> Option<TextFieldsOut> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let TextFieldsIntents {
            pending_vec_text_size,
            pending_vec_text_weight,
            pending_vec_text_line_height,
            pending_vec_text_tracking,
            pending_vec_text_wrap,
            pending_vec_text_align,
            pending_vec_text_axis,
        } = intents;
        let editing_session = self.vec.text_edit.is_some();
        if let Some(size) = pending_vec_text_size {
            crate::vec_text::apply_text_size(
                &mut self.vec.text_edit,
                &mut self.vec.text.size,
                vec_scene,
                size,
            );
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.size = size,
                );
            }
        }
        if let Some(weight) = pending_vec_text_weight {
            crate::vec_text::apply_text_weight(
                &mut self.vec.text_edit,
                &mut self.vec.text.weight,
                vec_scene,
                weight,
            );
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.weight = weight,
                );
            }
        }
        if let Some(lh) = pending_vec_text_line_height {
            crate::vec_text::apply_text_line_height(
                &mut self.vec.text_edit,
                &mut self.vec.text.line_height,
                vec_scene,
                lh,
            );
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.line_height = lh,
                );
            }
        }
        if let Some(wrap) = pending_vec_text_wrap {
            crate::vec_text::apply_text_wrap(
                &mut self.vec.text_edit,
                &mut self.vec.text.wrap,
                vec_scene,
                wrap,
            );
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.wrap_width = wrap,
                );
            }
        }
        if let Some(tr) = pending_vec_text_tracking {
            crate::vec_text::apply_text_tracking(
                &mut self.vec.text_edit,
                &mut self.vec.text.tracking,
                vec_scene,
                tr,
            );
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.tracking = tr,
                );
            }
        }
        if let Some((i, v)) = pending_vec_text_axis
            && !editing_session
        {
            crate::vec_text::edit_selected_text(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_text_sel,
                |p| {
                    if let Some(a) = p.axes.get_mut(i) {
                        a.1 = v as f32;
                    }
                },
            );
        }
        if let Some(align) = pending_vec_text_align {
            if !editing_session {
                crate::vec_text::edit_selected_text(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    |p| p.align = crate::vec_text::align_to_u8(align),
                );
            }
            crate::vec_text::apply_text_align(
                &mut self.vec.text_edit,
                &mut self.vec.text.align,
                vec_scene,
                align,
            );
        }
        Some(TextFieldsOut {
            editing_session,
            pending_vec_text_axis,
            vec_text_sel,
        })
    }
}
