//! A porta do painel para o **modo sobre o vazio** (L7, nos meios que mostram o Blend) —
//! `ids::PAINTER_BRUSH_BLEND_ON_EMPTY`.

use super::PainterTool;

impl PainterTool {
    /// ⭐ **O modo sobre o vazio** (L7). Na água o modo é da SESSÃO (`set_brush_blend`): com tinta molhada na
    /// tela e um modo de cor a sessão fecha (a tinta já pousada fica como está); sem tinta, ou num modo em que o
    /// campo não age (o Mix), só troca — fechar a água por um interruptor inerte mudaria a tinta.
    pub fn set_blend_sobre_vazio(&mut self, on: bool) {
        let fecha = match self.paint.wetpaint.session.as_mut() {
            Some(sess) if sess.vazio != on && !sess.blend.e_de_cor() => {
                sess.vazio = on;
                false
            }
            Some(sess) if sess.vazio != on => {
                let ha_tinta = sess.ha_tinta();
                if !ha_tinta {
                    sess.vazio = on;
                }
                ha_tinta
            }
            _ => false,
        };
        if fecha {
            self.wetpaint_end_session();
        }
        self.paint.brush.blend_sobre_vazio = on;
    }

    /// A linha do modo sobre o vazio, do canal do painel (`SetValue`).
    pub(crate) fn route_blend_vazio_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        match event {
            ph2d_editor_core::tool::PanelEvent::SetValue(id, v)
                if *id == crate::ids::PAINTER_BRUSH_BLEND_ON_EMPTY =>
            {
                self.set_blend_sobre_vazio(*v >= 0.5);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::PanelEvent;

    /// ⭐ **O painel chega ao PINCEL** (o leitor decide: o traço lê `blend_sobre_vazio`).
    #[test]
    fn a_linha_do_vazio_chega_ao_pincel() {
        let mut t = PainterTool::default();
        let id = crate::ids::PAINTER_BRUSH_BLEND_ON_EMPTY;
        t.handle_panel_event(PanelEvent::SetValue(id, 1.0));
        assert!(t.paint.brush.blend_sobre_vazio);
        assert!(t.brush_settings().blend_sobre_vazio);
        t.handle_panel_event(PanelEvent::SetValue(id, 0.0));
        assert!(!t.paint.brush.blend_sobre_vazio);
    }
}
