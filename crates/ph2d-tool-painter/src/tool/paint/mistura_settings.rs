//! A porta do painel da **mistura opaca** do Digital (`BrushSpec::mistura_opaca`,
//! `ph2d_painter_brush::mistura_opaca`): o seletor `ids::PAINTER_BRUSH_MIX_MODES` e os números
//! `ids::PAINTER_BRUSH_MIX_FIELDS`. Uma opção a mais: `Off` (o default) é o pincel de sempre.

use super::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::mistura_opaca::{AmostraDaMistura, ModoMisturaOpaca};

impl PainterTool {
    /// O modo do Color mixing (Off · Wet blend).
    pub fn set_brush_mistura_modo(&mut self, modo: ModoMisturaOpaca) {
        self.paint.brush.mistura_opaca.modo = modo;
    }

    /// De onde a mistura apanha a cor (This layer · All visible layers).
    pub fn set_brush_mistura_amostra(&mut self, amostra: AmostraDaMistura) {
        self.paint.brush.mistura_opaca.amostra = amostra;
    }

    /// Um número da mistura, pela ordem de `ids::PAINTER_BRUSH_MIX_FIELDS`, preso a `0..1`.
    pub fn set_brush_mistura_numero(&mut self, campo: usize, v: f32) {
        let v = if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            return;
        };
        let m = &mut self.paint.brush.mistura_opaca;
        match campo {
            0 => m.paint_amount = v,
            1 => m.paint_density = v,
            2 => m.color_stretch = v,
            3 => m.amount_pressure = v,
            4 => m.density_pressure = v,
            _ => {}
        }
    }

    /// O seletor e os números da mistura, do canal do painel.
    pub(crate) fn route_mistura_event(&mut self, event: &PanelEvent) -> bool {
        let modos = &crate::ids::PAINTER_BRUSH_MIX_MODES;
        let campos = &crate::ids::PAINTER_BRUSH_MIX_FIELDS;
        let amostras = &crate::ids::PAINTER_BRUSH_MIX_SAMPLES;
        match event {
            PanelEvent::Click(id) if modos.contains(id) => {
                let i = modos.iter().position(|m| m == id).unwrap_or(0);
                self.set_brush_mistura_modo(ModoMisturaOpaca::TODOS[i]);
                true
            }
            PanelEvent::Click(id) if amostras.contains(id) => {
                let i = amostras.iter().position(|a| a == id).unwrap_or(0);
                self.set_brush_mistura_amostra(AmostraDaMistura::TODAS[i]);
                true
            }
            PanelEvent::SetValue(id, v) if campos.contains(id) => {
                let i = campos.iter().position(|c| c == id).unwrap_or(0);
                self.set_brush_mistura_numero(i, *v as f32);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ids;
    use crate::tool::PainterTool;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::PanelEvent;
    use ph2d_painter_brush::mistura_opaca::{AmostraDaMistura, MisturaOpaca, ModoMisturaOpaca};

    /// ⭐ O seletor escolhe o modo e os números chegam ao PINCEL (o leitor decide: o carimbo lê
    /// `BrushSpec::mistura_opaca`); o retrato do painel mostra o mesmo.
    #[test]
    fn a_mistura_do_painel_chega_ao_pincel() {
        let mut t = PainterTool::default();
        assert_eq!(t.paint.brush.mistura_opaca, MisturaOpaca::default());
        for (id, modo) in ids::PAINTER_BRUSH_MIX_MODES
            .iter()
            .zip(ModoMisturaOpaca::TODOS)
            .rev()
        {
            t.handle_panel_event(PanelEvent::Click(*id));
            assert_eq!(t.paint.brush.mistura_opaca.modo, modo);
            assert_eq!(t.brush_settings().mistura_opaca.modo, modo);
        }
        let valores: [f32; 5] = [0.1, 0.2, 0.3, 0.4, 0.5];
        for (id, v) in ids::PAINTER_BRUSH_MIX_FIELDS.iter().zip(valores) {
            t.handle_panel_event(PanelEvent::SetValue(*id, f64::from(v)));
        }
        let m = t.brush_settings().mistura_opaca;
        let lidos = [
            m.paint_amount,
            m.paint_density,
            m.color_stretch,
            m.amount_pressure,
            m.density_pressure,
        ];
        assert_eq!(lidos, valores);
        t.handle_panel_event(PanelEvent::SetValue(ids::PAINTER_BRUSH_MIX_FIELDS[0], 7.0));
        assert_eq!(
            t.paint.brush.mistura_opaca.paint_amount, 1.0,
            "preso a 0..1"
        );
        for (id, a) in ids::PAINTER_BRUSH_MIX_SAMPLES
            .iter()
            .zip(AmostraDaMistura::TODAS)
            .rev()
        {
            t.handle_panel_event(PanelEvent::Click(*id));
            assert_eq!(t.brush_settings().mistura_opaca.amostra, a);
        }
    }

    /// Os ids da mistura são distintos entre si e de todo outro id do pincel que conta aqui.
    #[test]
    fn os_ids_da_mistura_sao_unicos() {
        let mut v: Vec<_> = ids::PAINTER_BRUSH_MIX_MODES
            .iter()
            .chain(&ids::PAINTER_BRUSH_MIX_FIELDS)
            .chain(&ids::PAINTER_BRUSH_MIX_SAMPLES)
            .copied()
            .collect();
        v.extend([
            ids::PAINTER_BRUSH_SMEAR_FIO,
            ids::PAINTER_BRUSH_WASH,
            ids::PAINTER_BRUSH_MIX_MODE,
            ids::PAINTER_BRUSH_MIX_SAMPLE,
        ]);
        let n = v.len();
        v.sort_unstable_by_key(|i| i.0);
        v.dedup();
        assert_eq!(v.len(), n);
    }

    /// Esmaecidos com a lei desligada, vivos com ela ligada (`Dependente::MisturaDesligada`).
    #[test]
    fn os_numeros_da_mistura_esmaecem_com_ela_desligada() {
        use crate::Dependente as D;
        let mut t = PainterTool::default();
        assert!(t.inercias().inerte(D::MisturaDesligada));
        assert!(
            ids::PAINTER_BRUSH_MIX_SAMPLES
                .iter()
                .all(|id| D::MisturaDesligada.cobre(*id))
        );
        t.set_brush_mistura_modo(ModoMisturaOpaca::WetBlend);
        assert!(!t.inercias().inerte(D::MisturaDesligada));
    }
}
