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
        let rate = self.spec.airbrush_rate_s.max(1e-3);
        self.airbrush_accum_s += dt.max(0.0);
        let mut emitted = 0u32;
        while self.airbrush_accum_s >= rate {
            self.airbrush_accum_s -= rate;
            let pr = self.method_pressure(self.last_pressure);
            // per-event: no spacing attenuation
            self.stamp_at(self.last_pos, pr, 1.0, self.arc_len, out);
            self.tot_samples = self.tot_samples.wrapping_add(1);
            emitted += 1;
            // Stall guard: a frame-driven tick can hand us a huge `dt` after a hitch (GC/resize/
            // breakpoint). Blender's async timer wouldn't fire retroactively, so cap the burst and
            // drop the backlog rather than dump a flood of overlapping dabs at one spot.
            if emitted >= MAX_AIRBRUSH_DABS_PER_TICK {
                self.airbrush_accum_s = 0.0;
                break;
            }
        }
    }
}
