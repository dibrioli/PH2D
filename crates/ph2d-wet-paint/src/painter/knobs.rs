//! **COMO O MOTOR REAGE A UM KNOB** — filho de [`super`] (teto de LOC da
//! workspace), cortado por RESPONSABILIDADE: lá mora *o que o motor É e o que
//! ele FAZ com um traço*; aqui, *o que uma escrita de knob INVALIDA* — a
//! textura do pincel, o papel a re-assar, o canvas inteiro.
//!
//! ⚠️ É o `onChange` do JS, e ele é load-bearing: engolir as reações de um
//! `resetGroup` deixava textura de pincel velha e papel não-assado (achado de
//! port-verify), e é por isso que o `reset_knob_group` devolve os defs mudados
//! com `#[must_use]`.

use super::*;

impl Engine {
    /// Write a knob and react to what it invalidates (the JS onChange).
    pub fn set_knob(&mut self, knob: Knob, v: f64) {
        if let Some(def) = self.tuning.set(knob, v) {
            self.react_to_knob_change(def);
        }
    }

    fn react_to_knob_change(&mut self, def: &'static crate::tuning::KnobDef) {
        match def.rebuild {
            Some(Rebuild::Brush) => self.brush_tex = None, // lazy rebuild
            Some(Rebuild::Paper) => self.paper_dirty = true, // re-bake on release
            Some(Rebuild::Render) => self.mark_dirty_full(),
            None => {}
        }
    }

    /// Reset a whole knob group, reacting to every changed knob exactly as
    /// [`Engine::set_knob`] would — the JS fires onChange per knob from
    /// resetGroup; dropping those left a stale brush texture / unbaked paper
    /// (port-verify finding). The panel's group-reset button goes HERE.
    pub fn reset_knob_group(&mut self, group: crate::tuning::KnobGroup) {
        for def in self.tuning.reset_group(group) {
            self.react_to_knob_change(def);
        }
    }

    /// O **Tooth** do artista — quanto o dente do papel morde a tinta (`0..=2`, `1` = o papel como ele é).
    /// Mexer nele pede o papel de novo: o assado do motor re-assa no próximo `reconcile`; um papel do
    /// artista volta a semear porque o Tooth entra na chave dele.
    pub fn set_paper_tooth(&mut self, k: f32) {
        let k = k.max(0.0);
        if k != self.paper_tooth {
            self.paper_tooth = k;
            self.paper_dirty = true;
        }
    }

    /// O Tooth vigente (ver [`Self::set_paper_tooth`]).
    pub fn paper_tooth(&self) -> f32 {
        self.paper_tooth
    }

    /// The shell calls this on slider release when a paper knob changed.
    pub fn paper_dirty(&self) -> bool {
        self.paper_dirty
    }

    /// A tile de cerdas, para um gate que dirige o [`crate::trail::Trail`]
    /// direto — a mesma que o depósito usa, nunca uma segunda.
    #[must_use]
    pub fn bristle_texture_for_measure(&mut self) -> Vec<f32> {
        self.texture().to_vec()
    }
}
