//! As portas do painel para o **traço como uma camada** (L8: o tecto e as orlas, só no Digital) e o **modo sobre o
//! vazio** (L7, nos meios que mostram o Blend) — `ids::PAINTER_TRACO_FIELDS`, pela ordem.

use super::PainterTool;
use ph2d_painter_brush::traco::LARGURA_MAX_PX;

impl PainterTool {
    /// O tecto de opacidade do traço, `0..1` (`0` = sem tecto).
    pub fn set_traco_tecto(&mut self, v: f32) {
        self.paint.brush.traco.tecto = v.clamp(0.0, 1.0);
    }

    /// A orla molhada, `0..1`.
    pub fn set_traco_molhada(&mut self, v: f32) {
        self.paint.brush.traco.molhada = v.clamp(0.0, 1.0);
    }

    /// A orla queimada, `0..1`.
    pub fn set_traco_queimada(&mut self, v: f32) {
        self.paint.brush.traco.queimada = v.clamp(0.0, 1.0);
    }

    /// A largura da orla, px `1..48`.
    pub fn set_traco_largura(&mut self, v: f32) {
        self.paint.brush.traco.largura = v.clamp(1.0, LARGURA_MAX_PX);
    }

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

    /// As linhas do traço e do modo sobre o vazio, do canal do painel (`SetValue`).
    pub(crate) fn route_traco_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        let ph2d_editor_core::tool::PanelEvent::SetValue(id, v) = event else {
            return false;
        };
        let Some(i) = crate::ids::PAINTER_TRACO_FIELDS
            .iter()
            .position(|f| f == id)
        else {
            return false;
        };
        let v = *v as f32;
        match i {
            0 => self.set_traco_tecto(v),
            1 => self.set_traco_molhada(v),
            2 => self.set_traco_queimada(v),
            3 => self.set_traco_largura(v),
            _ => self.set_blend_sobre_vazio(v >= 0.5),
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::PanelEvent;

    /// ⭐ **O painel chega ao PINCEL** (o leitor decide: o traço lê `brush.traco`/`blend_sobre_vazio`): cada linha
    /// pelo canal do painel muda o campo dela, presa à faixa.
    #[test]
    fn cada_linha_do_traco_chega_ao_pincel() {
        let mut t = PainterTool::default();
        let ids = crate::ids::PAINTER_TRACO_FIELDS;
        for (id, v) in ids.into_iter().zip([0.4, 0.6, 0.7, 99.0, 1.0]) {
            t.handle_panel_event(PanelEvent::SetValue(id, v));
        }
        let b = t.paint.brush;
        assert_eq!(
            (
                b.traco.tecto,
                b.traco.molhada,
                b.traco.queimada,
                b.traco.largura
            ),
            (0.4, 0.6, 0.7, 48.0)
        );
        assert!(b.blend_sobre_vazio);
        let s = t.brush_settings();
        assert_eq!((s.traco, s.blend_sobre_vazio), (b.traco, true));
        t.handle_panel_event(PanelEvent::SetValue(ids[4], 0.0));
        assert!(!t.paint.brush.blend_sobre_vazio);
    }
}
