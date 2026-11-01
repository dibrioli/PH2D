//! **Absorbency** (E10d2): a taxa com que o papel bebe a água do filme, da TELA — fora dos `WetKnobs` e de todo
//! preset, como o Tilt e o Smooth Edges. A sessão leva-a ao motor (`Sim::absorcao`) pelo `WetEngineFacts`.
//! Doc: `docs/Painter/Arquitetura/E10d2_absorbency.md`.

use crate::tool::PainterTool;

impl PainterTool {
    /// A **Absorbency** da tela, presa à faixa do motor; NaN = o neutro (o motor de antes, ao bit).
    pub fn set_wet_absorcao(&mut self, v: f64) {
        let (min, neutro, max) = crate::ids::PAINTER_WETPAINT_ABSORBENCY_FAIXA;
        self.paint.wetpaint.absorcao = if v.is_nan() {
            neutro
        } else {
            v.clamp(min, max)
        };
    }

    /// A **Absorbency** de agora.
    #[must_use]
    pub fn wet_absorcao(&self) -> f64 {
        self.paint.wetpaint.absorcao
    }

    /// A rota do painel: `true` se o id é o da Absorbency (e o valor já entrou).
    pub(super) fn set_wet_absorcao_do_painel(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        if id != crate::ids::PAINTER_WETPAINT_ABSORBENCY {
            return false;
        }
        self.set_wet_absorcao(v);
        true
    }
}

#[cfg(test)]
#[path = "absorcao_sonda.rs"]
mod absorcao_sonda;

#[cfg(test)]
mod tests {
    use crate::tool::PainterTool;
    use ph2d_brush_presets::Biblioteca;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{
        CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
    };

    /// A Absorbency é da TELA: o painel escreve-a, carregar um preset de fábrica não a toca, o preset gravado não a
    /// leva, o Reset da secção devolve o neutro e a sessão empurra-a ao motor.
    #[test]
    fn a_absorcao_e_da_tela_e_nao_viaja_no_preset() {
        let (_, neutro, max) = crate::ids::PAINTER_WETPAINT_ABSORBENCY_FAIXA;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
        t.set_paint_tool_mode("wetpaint");
        assert_eq!(t.wet_absorcao(), neutro);
        assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_ABSORBENCY,
            4.0,
        )));
        assert_eq!(t.wet_absorcao(), 4.0);
        t.set_wet_absorcao(max * 10.0);
        assert_eq!(t.wet_absorcao(), max, "presa ao tecto do motor");
        t.set_wet_absorcao(4.0);

        // Carregar cada preset de fábrica não mexe nela.
        let raiz =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
        let b = Biblioteca::carregar(&raiz, None);
        assert!(!b.entradas.is_empty(), "os pincéis de fábrica");
        for e in b.entradas.iter().take(6) {
            let _ = t.wet_preset_aplicar(&e.preset);
            assert_eq!(
                t.wet_absorcao(),
                4.0,
                "o preset {:?} mexeu na tela",
                e.preset.parametros.name
            );
        }
        // O preset gravado não a leva: gravar com 4 e com o neutro dá o MESMO preset.
        let base = b.entradas[0].preset.parametros.clone();
        let com = t.wet_preset_capturar(&base);
        t.set_wet_absorcao(neutro);
        let sem = t.wet_preset_capturar(&base);
        assert_eq!(format!("{com:?}"), format!("{sem:?}"));

        // A sessão empurra-a ao motor.
        t.set_wet_absorcao(4.0);
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            t.on_canvas_pointer(CanvasPointer {
                pos: [32.0, 32.0],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
        t.on_tick(16.0);
        let mut no_motor = 0.0;
        t.wet_sonda_motor(|e| no_motor = e.sim.absorcao);
        assert_eq!(no_motor, 4.0);

        // O Reset da secção devolve o neutro.
        t.reset_brush_wetpaint();
        assert_eq!(t.wet_absorcao(), neutro);
    }
}
