//! A seção **BLEND** do painel Vector — módulo irmão do [`super`] (teto de 600 LOC do painel).
//!
//! Ela mora sozinha porque o botão de ESCAPE dela não é enfeite, e o doc explica por quê:
//! a correspondência entre duas formas é o problema que **ninguém** resolveu.

use super::*;

impl BodyCtx<'_> {
    /// Seção **BLEND** — os passos intermediários entre as DUAS formas selecionadas.
    ///
    /// **O botão de escape (Rotate Match) não é enfeite.** A correspondência entre duas formas (que
    /// ponto de A vira que ponto de B?) é o problema que ninguém resolveu: o GSAP tem um
    /// `shapeIndex` manual E uma ferramenta de debug que admite que o automático erra; o Corel
    /// pede para o usuário **clicar um nó em cada forma**. Quando o automático errar aqui, a
    /// forma **gira** no meio do caminho, e o artista tem de ter a saída na mão. O Rotate **re-roda**
    /// o blend na hora: o erro se conserta à vista, sem desfazer nada.
    ///
    /// (Houve um 2º botão, *Reverse Match*, removido 2026-07-14: inverter o sentido de percurso de B
    /// inverte o winding e **colapsava a forma** — e o sentido correto já é escolhido pelo motor.)
    pub(crate) fn blend_section(&mut self, snap: &VectorStyleSnapshot, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BLEND,
            tr("panel.vector.section.blend"),
            y,
        );
        if collapsed {
            return y;
        }
        let track = self
            .store
            .slider(ids::VECTOR_BLEND_STEPS)
            .map_or_else(|| blend_steps_to_track(BLEND_STEPS_DEFAULT), |(_, v)| v);
        let steps = blend_steps_from_track(f64::from(track));
        y = self.slider_row(
            "Steps",
            ids::VECTOR_BLEND_STEPS,
            ids::VECTOR_BLEND_STEPS_NUM,
            track,
            f64::from(steps),
            &format!("{steps}"),
            y,
        );
        y = self.checkbox_row(
            ids::VECTOR_BLEND_STACK_UP,
            "Stack Each Above",
            snap.blend_stack_up,
            y,
        );
        y = self.action_button(ids::VECTOR_BLEND_RUN, "Blend", y);
        self.action_button(ids::VECTOR_BLEND_ROTATE, "Rotate Match", y)
    }

    /// Uma linha de checkbox (caixa + rótulo). O id fica **`Button`** no store — o clique viaja
    /// pelo canal de Click que já existe, e só o VISUAL é de checkbox (o mesmo arranjo do painel
    /// do Painter). O valor vem do snapshot: a **tool** é a dona, não o painel.
    pub(crate) fn checkbox_row(
        &mut self,
        id: ph2d_a11y::NodeId,
        label: &str,
        checked: bool,
        y: f32,
    ) -> f32 {
        let value = if checked {
            CheckboxValue::Checked
        } else {
            CheckboxValue::Unchecked
        };
        let rect = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let cb = Checkbox::new(id, label).value(value);
        paint_checkbox(&cb, rect, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, rect);
        y + self.row_h + self.row_gap
    }
}
