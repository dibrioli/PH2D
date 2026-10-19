//! **O que um clique apanha** (saiu do `gesture.rs` pelo tecto de LOC, W23 B5(2)).

use ph2d_board_model::{BoardDoc, ElementId};

use crate::{Editor, Pointer};

impl Editor {
    /// O elemento de cima em `p` (o que um clique apanha), na ordem do que se VÊ (os vídeos por cima, W21):
    /// uma forma pelo contorno, uma seta pela linha (a menos de meia pega, mais meia espessura) ou
    /// por um dos textos dela (W23 B5(2), o Miro).
    pub fn hit(&mut self, doc: &BoardDoc, p: Pointer) -> Option<ElementId> {
        let tol = self.metrics.handle / 2.0 * p.px;
        self.routes.sync(doc);
        let found =
            doc.live_in_paint_order()
                .into_iter()
                .rev()
                .filter(|el| el.as_frame().is_none() && !doc.connector_hidden(el))
                .find(|el| match el.connector() {
                    Some(c) => {
                        self.routes.get(el.id).is_some_and(|r| {
                            r.distance(p.world) <= tol + c.style.stroke_width / 2.0
                        }) || self.caption_of(el, p.world).is_some()
                    }
                    None if el.ink().is_some() => crate::ink::ink_hit(el, p.world, tol),
                    None => ph2d_board_geom::hit(el, p.world, tol),
                })
                .map(|el| el.id);
        found.or_else(|| self.frame_hit(doc, p))
    }
}
