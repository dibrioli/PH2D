//! O RELÓGIO do traço ([`Stroke::tick`]): a fita e o aerógrafo, por quadro — filho de [`super`] por LOC.

use super::*;

impl Stroke {
    /// Advance the airbrush timer by `dt` seconds, emitting a dab at the current cursor every
    /// `rate` seconds (Blender airbrush TIMER). No-op unless the method is [`StrokeMethod::Airbrush`].
    /// The tool drives this from its tick while the button is held and the cursor is parked.
    pub fn tick(&mut self, dt: f32, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        // ⚠️ A velocidade do gesto é medida ANTES do desvio abaixo, e a ordem é a feature — o
        // porquê está no [`mod@super::speed`], junto da lei.
        self.note_tick_speed(dt);
        let segurados = self.solta_a_velocidade(dt);
        self.tick_do_metodo(dt, out);
        if !segurados.is_empty() {
            out.splice(0..0, segurados);
        }
        self.passa_a_abertura(out, 0, false);
    }

    /// O resto do [`Self::tick`]: a fita e o aerógrafo.
    fn tick_do_metodo(&mut self, dt: f32, out: &mut Vec<Dab>) {
        // ⚠️ A FITA percorre o caminho AQUI, e é a única lei que o faz — parada ou em movimento, um
        // passo por quadro. A lei e o porquê vivem em [`mod@super::ribbon`].
        if self.spec.ribbon_active() && self.spec.stroke_method == StrokeMethod::Space {
            self.tick_ribbon(dt, out);
        }
        self.tick_parado(dt, out);
        if self.spec.stroke_method != StrokeMethod::Airbrush {
            return;
        }
        let rate = self.spec.airbrush_rate_s.max(PERIODO_MIN_S);
        let mut acum = self.airbrush_accum_s;
        // per-event: no spacing attenuation
        let n = pulsos(&mut acum, dt, rate);
        self.airbrush_accum_s = acum;
        for _ in 0..n {
            let pr = self.method_pressure(self.last_pressure);
            self.stamp_at(self.last_pos, pr, 1.0, self.arc_len, out);
            self.tot_samples = self.tot_samples.wrapping_add(1);
        }
    }
}

/// O relógio do traço parado (aerógrafo e spray parado): soma `dt` (até [`DT_TIQUE_MAX_S`]; o resto de um quadro
/// preso cai) a `acum` e devolve quantos `periodo` inteiros cabem, descontando-os. A densidade é a mesma a 30, 60 e
/// 144 q/s, e o tique pousa no máximo `⌈DT_TIQUE_MAX_S / periodo⌉ + 1` eventos.
pub(super) fn pulsos(acum: &mut f32, dt: f32, periodo: f32) -> u32 {
    *acum += dt.clamp(0.0, DT_TIQUE_MAX_S);
    let mut n = 0;
    while *acum >= periodo {
        *acum -= periodo;
        n += 1;
    }
    n
}
