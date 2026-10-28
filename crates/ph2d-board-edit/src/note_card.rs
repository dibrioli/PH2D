//! W11 do Kanban — **a nota que um cartão vira** («Convert to sticky note»): uma nota quadrada do
//! tamanho das próximas, da cor que a etiqueta do cartão pede, com o texto dele.

use ph2d_board_model::{BoardDoc, Element, Rgba, RichText, ShapeType};

use crate::Editor;

impl Editor {
    /// A nota de cor `fill` com `text`, centrada em `c`, à frente de tudo (não entra no documento:
    /// quem chama põe-na por um passo de desfazer).
    #[must_use]
    pub fn note_for_card(
        &self,
        doc: &mut BoardDoc,
        fill: Rgba,
        text: RichText,
        c: [f64; 2],
    ) -> Element {
        let bx = self.note_box(ShapeType::Sticky, c);
        let mut el = self.new_note(doc, ShapeType::Sticky, bx);
        if let Some(s) = el.shape_mut() {
            s.style.fill = Some(fill);
            s.style.text_color = fill.readable_ink();
            s.text = text;
        }
        el
    }
}
