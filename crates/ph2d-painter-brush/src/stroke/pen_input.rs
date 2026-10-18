//! **A caneta no traço** (W6, [`crate::pen`]): a pressão que o pincel LÊ (o piso e o tecto da mão, a
//! suavização) e a inclinação que chega com cada amostra, lida com o atraso do pincel e entregue a cada dab
//! como a ponta deitada ([`crate::DabTilt`]). Filho de [`super`] (o tecto de linhas do `stroke.rs`).

use super::*;

/// O que o traço lembra da caneta.
#[derive(Clone, Copy, Debug)]
pub(super) struct PenState {
    /// A inclinação da mão na amostra que chega (`[-1, 1]` por eixo, a do `CanvasPointer`).
    tilt: [f32; 2],
    /// A inclinação LIDA ([`crate::pen::Tilt::follow`]); `None` até ao primeiro movimento do traço.
    tilt_lida: Option<[f32; 2]>,
    /// A posição da amostra anterior: o caminho do atraso da inclinação.
    pos: [f32; 2],
    /// A pressão LIDA da amostra anterior.
    pressure_lida: f32,
}

impl Default for PenState {
    fn default() -> Self {
        Self {
            tilt: [0.0, 0.0],
            tilt_lida: None,
            pos: [0.0, 0.0],
            pressure_lida: 1.0,
        }
    }
}

impl Stroke {
    /// A inclinação da caneta na PRÓXIMA amostra (`begin`/`extend`): `[-1, 1]` por eixo (`±90°`); o rato
    /// e quem nunca a chama ficam com `[0, 0]`, a caneta de pé.
    pub fn set_tilt(&mut self, tilt: [f32; 2]) {
        self.pen.tilt = tilt;
    }

    /// A amostra da MÃO como o pincel a lê: a pressão pelo piso/tecto e pela suavização ([`crate::Pen`]), e
    /// a inclinação lida avança pelo caminho. O pen-down (`primeiro`) não suaviza: é o início da memória.
    pub(super) fn le_a_mao(&mut self, p: StrokePoint, primeiro: bool) -> StrokePoint {
        let pen = self.dynamics.pen;
        let lida = pen.read(p.pressure);
        let lida = if primeiro {
            lida
        } else {
            pen.smooth(self.pen.pressure_lida, lida)
        };
        self.pen.pressure_lida = lida;
        // O atraso começa no PRIMEIRO MOVIMENTO, não no toque (medido no Rebelle: tocar de pé e mover
        // deitado dá a ponta deitada logo).
        self.pen.tilt_lida = match (primeiro, self.pen.tilt_lida) {
            (true, _) => None,
            (false, None) => Some(self.pen.tilt),
            (false, Some(prev)) => Some(self.spec.pen_tilt.follow(
                prev,
                self.pen.tilt,
                dist(self.pen.pos, p.pos),
            )),
        };
        self.pen.pos = p.pos;
        StrokePoint {
            pressure: lida,
            ..p
        }
    }

    /// A ponta deitada do dab que se emite agora (`None` = a lei desligada ou a caneta de pé).
    pub(super) fn tilt_do_dab(&self) -> Option<crate::DabTilt> {
        let t = self.pen.tilt_lida.unwrap_or(self.pen.tilt);
        self.spec.pen_tilt.effect(t, self.heading)
    }

    /// Deita o `dab` pela caneta: o PenTilt roda-o para o azimute, o raio cresce até ao eixo maior da
    /// pegada esticada ([`crate::DabTilt::compose`]) — a caixa de todo carimbo é o raio, e assim nenhuma
    /// rota corta a elipse — e o dab leva a ponta para a pegada ([`BrushSpec::dab_footprint_of`]).
    pub(super) fn deita(&self, dab: &mut Dab, t: crate::DabTilt) {
        if self.spec.pen_tilt.rotation == crate::DabRotation::PenTilt {
            dab.rotation = crate::heading::rotate(dab.rotation, t.azimuth);
        }
        dab.tilt = Some(t);
        if t.stretches() {
            dab.radius_px =
                (dab.radius_px * self.spec.tilt_scale(dab)).clamp(0.5, MAX_BRUSH_RADIUS_PX);
        }
    }
}
