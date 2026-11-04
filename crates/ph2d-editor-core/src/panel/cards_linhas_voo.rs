//! A linha (ou as linhas) que a mão leva: inclinadas e com a sombra do voo do cartão (lei do arrastar,
//! 05 §6), sem registar clique. O corpo do cartão corre dentro do recorte dele, e o voo não pode: fica
//! guardado e a lista pinta-o depois de tudo ([`pinta_os_voos`]).

use super::ListRow;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_elided};
use crate::panel::cards::{CardCtx, flight_shadow};
use crate::zones::Rect;
use ph2d_card_stack::metrics as m;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::{Affine, VectorScene};
use std::cell::RefCell;

thread_local! {
    static VOOS: RefCell<Vec<(VectorScene, Affine)>> = const { RefCell::new(Vec::new()) };
}

/// Pinta o que voa, já fora de qualquer recorte de cartão.
pub(in crate::panel) fn pinta_os_voos(tela: &mut VectorScene) {
    for (voo, t) in VOOS.with(RefCell::take) {
        tela.append_transformed(&voo, t);
    }
}

impl CardCtx<'_> {
    /// Uma linha na mão. `(x, y)` é o topo esquerdo do fantasma.
    pub fn list_row_flight(&mut self, row: &ListRow<'_>, x: f32, y: f32) {
        self.list_rows_flight(std::slice::from_ref(row), x, y);
    }

    /// Várias linhas na mão, em bloco: a multisselecção mostra o que leva, a agarrada primeiro.
    pub fn list_rows_flight(&mut self, rows: &[ListRow<'_>], x: f32, y: f32) {
        let local = Rect::new(0.0, 0.0, self.w, m::FIELD_H * rows.len() as f32);
        let mut voo = VectorScene::new();
        flight_shadow(&mut voo, self.ink.theme, local);
        fill_rounded_rect(
            &mut voo,
            local,
            m::SEG_RADIUS,
            self.ink.color(ColorToken::KanbanCardRaised),
        );
        for (i, row) in rows.iter().enumerate() {
            let topo = m::FIELD_H * i as f32;
            let mut nome_x = m::SEG_INSET;
            if let Some(icon) = row.icon {
                let lado = m::SEG_TEXT_PX;
                paint_icon(
                    &mut voo,
                    icon,
                    Rect::new(nome_x, topo + (m::FIELD_H - lado) * 0.5, lado, lado),
                    self.ink.color(ColorToken::Text1),
                    StrokeToken::Default.px(),
                );
                nome_x += lado + m::SEG_ICON_GAP;
            }
            paint_text_elided(
                self.text_system,
                &mut voo,
                row.label,
                nome_x,
                topo + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
                m::VALUE_PX,
                (local.w - nome_x - m::SEG_INSET).max(0.0),
                self.ink.color(ColorToken::Text1),
            );
        }
        let pivo = ph2d_card_stack::flight_pivot(local.w, local.h);
        let t = Affine::translate((f64::from(x), f64::from(y)))
            * Affine::translate(pivo)
            * Affine::rotate(ph2d_card_stack::FLIGHT_TILT_DEG.to_radians())
            * Affine::translate((-pivo.0, -pivo.1));
        VOOS.with(|v| v.borrow_mut().push((voo, t)));
    }
}
