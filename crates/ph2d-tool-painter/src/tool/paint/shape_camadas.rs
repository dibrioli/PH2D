//! **As duas camadas da cor do Shape** (prompt 38, dono 15/10): a **Texture Color** (a rampa de sempre,
//! `shape_color_ramp`, agora indexada pelo VALOR da textura do Shape) e a **Falloff Color** (uma segunda
//! rampa, indexada pela cobertura — a lei de antes do prompt 38), na mesma secção, com a ordem trocável e
//! um Blend entre elas. A lei vive no motor ([`ph2d_painter_brush::ShapeRampLayers`]); aqui ficam o
//! estado, os setters do painel e as LUTs assadas.

use crate::tool::PainterTool;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_painter_brush::BrushBlend;

/// O estado das camadas. A Texture Color é a rampa `shape_color_ramp`; a Falloff Color é `queda_rampa`.
#[derive(Clone, Debug)]
pub(super) struct CamadasDoShape {
    pub(super) textura: bool,
    pub(super) queda: bool,
    pub(super) queda_rampa: ColorRamp,
    /// A Falloff Color é a camada de CIMA (a primeira da lista).
    pub(super) queda_primeiro: bool,
    pub(super) blend: BrushBlend,
    pub(super) mix: f32,
    /// A LUT da Falloff Color (sRGB reto), assada com a da Texture Color em `ensure_ramp_lut`.
    pub(super) lut_queda: Vec<[f32; 4]>,
    /// As duas compostas por índice — o carimbo em cache sem Grain, quando os dois índices coincidem.
    pub(super) lut_combinada: Vec<[f32; 4]>,
}

impl Default for CamadasDoShape {
    fn default() -> Self {
        Self {
            textura: true,
            queda: false,
            queda_rampa: ColorRamp::default(),
            queda_primeiro: false,
            blend: BrushBlend::Mix,
            mix: 0.5,
            lut_queda: Vec::new(),
            lut_combinada: Vec::new(),
        }
    }
}

impl CamadasDoShape {
    /// Alguma camada pinta a cor.
    pub(super) fn alguma(&self) -> bool {
        self.textura || self.queda
    }

    /// As camadas como o motor as lê (as LUTs já assadas).
    pub(super) fn motor(&self) -> ph2d_painter_brush::ShapeRampLayers<'_> {
        ph2d_painter_brush::ShapeRampLayers {
            texture: self.textura,
            falloff: self.queda.then_some(self.lut_queda.as_slice()),
            falloff_first: self.queda_primeiro,
            blend: self.blend,
            mix: self.mix,
        }
    }
}

impl PainterTool {
    /// Uma mudança nas camadas: re-assa as LUTs (a de cor e a do tom) e invalida o carimbo em cache.
    fn camadas_mudaram(&mut self) {
        self.paint.shape_ramp_dirty = true;
        self.paint.texture_ramp_dirty = true;
        self.paint.shape_ramp_version = self.paint.shape_ramp_version.wrapping_add(1);
    }

    /// Liga/desliga a **Texture Color** (a cor pela textura do Shape).
    pub fn toggle_shape_texture_color(&mut self) {
        self.paint.shape_camadas.textura = !self.paint.shape_camadas.textura;
        self.camadas_mudaram();
    }

    /// Liga/desliga a **Falloff Color** (a cor pela cobertura — a lei de antes do prompt 38).
    pub fn toggle_shape_falloff_color(&mut self) {
        self.paint.shape_camadas.queda = !self.paint.shape_camadas.queda;
        self.camadas_mudaram();
    }

    /// Troca qual das duas camadas fica por cima.
    pub fn swap_shape_color_layers(&mut self) {
        self.paint.shape_camadas.queda_primeiro = !self.paint.shape_camadas.queda_primeiro;
        self.camadas_mudaram();
    }

    /// O modo de mistura da camada de cima sobre a de baixo (`BrushBlend` wire discriminant).
    pub fn set_shape_color_blend(&mut self, m: u8) {
        self.paint.shape_camadas.blend = BrushBlend::from_u8(m);
        self.camadas_mudaram();
    }

    /// A opacidade da camada de cima, `[0, 1]`.
    pub fn set_shape_color_mix(&mut self, v: f32) {
        self.paint.shape_camadas.mix = v.clamp(0.0, 1.0);
        self.camadas_mudaram();
    }

    /// Substitui a rampa da Falloff Color (o painel semeia-a; os testes também).
    pub fn set_shape_falloff_ramp(&mut self, ramp: ColorRamp) {
        self.paint.shape_camadas.queda_rampa = ramp;
        self.camadas_mudaram();
    }

    /// A rampa da Falloff Color (só leitura; o snapshot lê os stops).
    #[must_use]
    pub fn shape_falloff_ramp(&self) -> &ColorRamp {
        &self.paint.shape_camadas.queda_rampa
    }

    /// O espaço de interpolação da Falloff Color (`RampColorMode` wire discriminant).
    pub fn set_shape_falloff_ramp_mode(&mut self, m: u8) {
        self.paint.shape_camadas.queda_rampa.color_mode = RampColorMode::from_u8(m);
        self.camadas_mudaram();
    }

    /// A interpolação da Falloff Color (`RampInterp` wire discriminant).
    pub fn set_shape_falloff_ramp_interp(&mut self, i: u8) {
        self.paint.shape_camadas.queda_rampa.interp = RampInterp::from_u8(i);
        self.camadas_mudaram();
    }

    /// Move o stop de id estável `id` da Falloff Color para `pos`.
    pub fn shape_falloff_ramp_move_stop(&mut self, id: u8, pos: f32) {
        let r = &mut self.paint.shape_camadas.queda_rampa;
        if let Some(idx) = r.index_of_id(id) {
            r.set_position(idx, pos.clamp(0.0, 1.0));
            self.camadas_mudaram();
        }
    }

    /// A cor do stop `id` da Falloff Color, em bytes sRGB RGBA (o espaço do picker).
    pub fn shape_falloff_ramp_set_stop_color(&mut self, id: u8, rgba: [u8; 4]) {
        let r = &mut self.paint.shape_camadas.queda_rampa;
        let Some(idx) = r.index_of_id(id) else {
            return;
        };
        let lin = ph2d_color::srgb::srgb_to_linear_byte;
        let a = f32::from(rgba[3]) / 255.0;
        r.set_color(idx, [lin(rgba[0]), lin(rgba[1]), lin(rgba[2]), a]);
        self.camadas_mudaram();
    }

    /// Um stop novo na Falloff Color, no meio do maior intervalo; devolve o índice.
    pub fn shape_falloff_ramp_add_stop(&mut self) -> usize {
        let r = &mut self.paint.shape_camadas.queda_rampa;
        let pos = super::ramp::largest_gap_midpoint(r.stops().iter().map(|s| s.pos), r.len());
        let color = r.eval(pos);
        let i = r.add_stop(RampStop::new(pos, color));
        self.camadas_mudaram();
        i
    }

    /// Tira o último stop da Falloff Color (fica sempre ≥ 1).
    pub fn shape_falloff_ramp_remove_last_stop(&mut self) {
        let r = &mut self.paint.shape_camadas.queda_rampa;
        let last = r.len().saturating_sub(1);
        r.remove_stop(last);
        self.camadas_mudaram();
    }

    /// Espelha a Falloff Color.
    pub fn shape_falloff_ramp_invert(&mut self) {
        self.paint.shape_camadas.queda_rampa.invert();
        self.camadas_mudaram();
    }

    /// Assa a LUT da Falloff Color e a LUT combinada (chamado por `ensure_ramp_lut` com a dona Shape,
    /// depois de assar a da Texture Color em `texture_ramp_lut`).
    pub(super) fn assa_camadas_do_shape(&mut self) {
        let c = &mut self.paint.shape_camadas;
        c.lut_queda = super::ramp_lut::lut_srgb(&c.queda_rampa, false);
        let tex = &self.paint.texture_ramp_lut;
        let camadas = c.motor();
        let combinada = (0..tex.len())
            .map(|i| camadas.color(tex, i as f32 / 255.0, i as f32 / 255.0))
            .collect();
        c.lut_combinada = combinada;
    }
}

impl PainterTool {
    /// The Shape Color layers' panel events (the two checkboxes, the swap, the Blend and the Falloff
    /// ramp's editor — the Mix slider rides the `SetValue` router). `true` when consumed.
    pub(super) fn route_shape_color_layers_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use crate::ids as i;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_TEXTURE_COLOR => {
                self.toggle_shape_texture_color();
            }
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_FALLOFF_COLOR => {
                self.toggle_shape_falloff_color();
            }
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_COLOR_SWAP => {
                self.swap_shape_color_layers();
            }
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_ADD => {
                self.shape_falloff_ramp_add_stop();
            }
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_REMOVE => {
                self.shape_falloff_ramp_remove_last_stop();
            }
            PanelEvent::Click(id) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_INVERT => {
                self.shape_falloff_ramp_invert();
            }
            PanelEvent::SelectOption(id, v) if *id == i::PAINTER_SHAPE_COLOR_BLEND => {
                if let Ok(m) = v.parse::<u8>() {
                    self.set_shape_color_blend(m);
                }
            }
            PanelEvent::SelectOption(id, v) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_MODE => {
                if let Ok(m) = v.parse::<u8>() {
                    self.set_shape_falloff_ramp_mode(m);
                }
            }
            PanelEvent::SelectOption(id, v) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_INTERP => {
                if let Ok(m) = v.parse::<u8>() {
                    self.set_shape_falloff_ramp_interp(m);
                }
            }
            PanelEvent::SelectOption(id, v) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH => {
                let mut it = v.split(',').filter_map(|p| p.parse::<i32>().ok());
                if let (Some(sid), Some(r), Some(g), Some(b), Some(a)) =
                    (it.next(), it.next(), it.next(), it.next(), it.next())
                {
                    self.shape_falloff_ramp_set_stop_color(
                        sid as u8,
                        [r as u8, g as u8, b as u8, a as u8],
                    );
                }
            }
            PanelEvent::SelectOption(id, v) if *id == i::PAINTER_SHAPE_FALLOFF_RAMP_EDIT => {
                if let Some((sid, x)) = super::jitter_settings::parse_id_f32(v) {
                    self.shape_falloff_ramp_move_stop(sid, x);
                }
            }
            _ => return false,
        }
        true
    }
}

/// A ramp's stops as the panel snapshot carries them: `(pos, r, g, b, a, id)` in display sRGB, sorted,
/// the first `count` valid.
pub(super) fn stops_do_painel(
    ramp: &ColorRamp,
) -> ([[f32; 6]; super::brush_settings::PANEL_RAMP_STOPS], u8) {
    let srgb = |x: f32| f32::from(ph2d_color::srgb::linear_to_srgb_byte(x)) / 255.0;
    let mut out = [[0.0f32; 6]; super::brush_settings::PANEL_RAMP_STOPS];
    for (slot, s) in out.iter_mut().zip(ramp.stops()) {
        *slot = [
            s.pos,
            srgb(s.color[0]),
            srgb(s.color[1]),
            srgb(s.color[2]),
            s.color[3],
            f32::from(s.id),
        ];
    }
    (
        out,
        ramp.stops()
            .len()
            .min(super::brush_settings::PANEL_RAMP_STOPS) as u8,
    )
}
