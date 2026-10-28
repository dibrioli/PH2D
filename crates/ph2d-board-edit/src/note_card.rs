//! W11 do Kanban — **a nota que um cartão vira** («Convert to sticky note»): uma nota do tamanho das
//! próximas, da cor que a etiqueta do cartão pede, com o texto dele — e já da altura que o texto
//! pede (a medida do ajuste ao texto, [`crate::notes::fitted_height`]), não quadrada à espera da
//! próxima edição.

use ph2d_board_model::{BoardDoc, Element, Rgba, RichText, ShapeType};
use ph2d_text::TextSystem;

use crate::Editor;

impl Editor {
    /// A nota de cor `fill` com `text`, com o topo onde a nota quadrada centrada em `c` o teria, à
    /// frente de tudo (não entra no documento: quem chama põe-na por um passo de desfazer).
    #[must_use]
    pub fn note_for_card(
        &self,
        doc: &mut BoardDoc,
        ts: &mut TextSystem,
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
        el.h = crate::notes::fitted_height(ts, &el);
        el
    }
}
