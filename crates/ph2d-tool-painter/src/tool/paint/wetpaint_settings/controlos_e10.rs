//! **Os controlos de lei da E10** (`docs/Painter/Arquitetura/F3_controlos.md` §9): Dilution, Medium, Volume (µm),
//! Body e a Hardness (vista exacta do `paperGate`) — o roteamento dos ids e os acessores. Filho por LOC.

use super::WetKnobs;
use ph2d_wet_paint::painter::receita::{H_REF, K_W, veiculo};
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};

/// O máximo da faixa do `paperGate` (a `KNOB_DEFS`): a Hardness `0` é ele, a `1` é `0`.
const PAPER_GATE_MAX: f64 = KNOB_DEFS[Knob::PaperGate as usize].max;

/// Massa por µm de tinta (`grid::tinta::M_UM`), em `f64`.
const M_UM: f64 = ph2d_wet_paint::grid::tinta::M_UM as f64;

/// **A migração da E10, em `const`** (a mesma conta de [`ph2d_wet_paint::painter::receita::migra_para_controlos`]):
/// `(pigmento P' já com a Opacity, WaterPerDab, Water, carga, Opacity)` → `(Dilution, Volume µm)`. Só para as
/// omissões `const` do pincel sem preset; os presets passam pela função do motor.
pub(super) const fn dilution_e_volume(
    pigmento: f64,
    agua_por_dab: f64,
    agua: f64,
    carga: f64,
    opacidade: f64,
) -> (f64, f64) {
    let v = K_W * veiculo(agua, agua_por_dab);
    let diluicao = if pigmento + v > 0.0 {
        1.0 - pigmento / (pigmento + v)
    } else {
        1.0
    };
    (diluicao, opacidade * carga * H_REF / M_UM)
}

impl crate::PainterTool {
    /// As opções dos menus do cartão Wet Paint (`SelectOption`, wire = o índice): o Environment do Real Oil e o
    /// **Paint type** (`0` Water → `Knob::Medium = 0`, `1` Oil → `1`, pela porta do `SetValue`). `false` = outro id
    /// ou índice fora.
    pub(super) fn route_wet_select(&mut self, id: ph2d_a11y::NodeId, v: &str) -> bool {
        let Ok(i) = v.parse::<u8>() else {
            return false;
        };
        match id {
            x if x == crate::ids::PAINTER_WETPAINT_OLEO_AMBIENTE => {
                self.set_wet_oleo_ambiente_indice(i)
            }
            x if x == crate::ids::PAINTER_WETPAINT_MEDIUM
                && i < crate::ids::PAINTER_WETPAINT_MEDIUM_COUNT =>
            {
                self.set_wet_knob_value(x, f64::from(i))
            }
            _ => false,
        }
    }
}

impl WetKnobs {
    /// Dilution (`Knob::Dilution`): `1 − φ` da receita.
    #[must_use]
    pub fn dilution(&self) -> f64 {
        self.get(Knob::Dilution)
    }
    /// Medium (`Knob::Medium`): o meio `m` — só `0` (Water) ou `1` (Oil), o degrau de [`ph2d_wet_paint::tuning::KnobDef::na_lei`].
    #[must_use]
    pub fn medium(&self) -> f64 {
        self.get(Knob::Medium)
    }
    /// O **Paint type** do seletor: `true` = Oil (`Medium > 0,5`), `false` = Water.
    #[must_use]
    pub fn e_oleo(&self) -> bool {
        self.medium() > 0.5
    }
    /// Volume (`Knob::Volume`): a quantidade por dab, em µm de tinta.
    #[must_use]
    pub fn volume_um(&self) -> f64 {
        self.get(Knob::Volume)
    }
    /// Body (`Knob::Body`): a tensão de cedência do pincel, somada à do meio.
    #[must_use]
    pub fn body(&self) -> f64 {
        self.get(Knob::Body)
    }
    /// Hardness: a vista do `paperGate` — `1 − paperGate ÷ máx`.
    #[must_use]
    pub fn hardness(&self) -> f64 {
        1.0 - self.get(Knob::PaperGate) / PAPER_GATE_MAX
    }
    /// Escreve a Hardness no `paperGate`: `(1 − v).clamp(0, 1) · máx` (NaN = a omissão do `paperGate`).
    pub fn set_hardness(&mut self, v: f64) {
        let g = if v.is_nan() {
            f64::NAN
        } else {
            (1.0 - v).clamp(0.0, 1.0) * PAPER_GATE_MAX
        };
        self.set(Knob::PaperGate, g);
    }

    /// **A linha do painel** de um controlo NUMÉRICO da E10 (E10-B2; o Paint type é um menu, [`Self::e_oleo`]):
    /// `(valor, mín, máx, passo)` — a faixa e o passo da
    /// `KNOB_DEFS` (a Hardness, vista sem knob próprio: `0..1` no passo dos outros controlos `0..1` da E10, o do
    /// Body). `None` = não é um deles.
    #[must_use]
    pub fn linha_e10(&self, id: ph2d_a11y::NodeId) -> Option<(f64, f64, f64, f64)> {
        use crate::ids;
        let knob = match id {
            x if x == ids::PAINTER_WETPAINT_DILUTION => Knob::Dilution,
            x if x == ids::PAINTER_WETPAINT_BODY => Knob::Body,
            x if x == ids::PAINTER_WETPAINT_SMOOTH_EDGES => Knob::SmoothEdges,
            x if x == ids::PAINTER_WETPAINT_VOL_OPACITY => Knob::Volume,
            x if x == ids::PAINTER_WETPAINT_HARDNESS => {
                let passo = KNOB_DEFS[Knob::Body as usize].step;
                return Some((self.hardness(), 0.0, 1.0, passo));
            }
            _ => return None,
        };
        let d = &KNOB_DEFS[knob as usize];
        Some((self.get(knob), d.min, d.max, d.step))
    }

    /// O roteamento dos ids da E10 (`false` = não é um deles). O Medium cai no degrau Water · Oil ([`ph2d_wet_paint::tuning::KnobDef::na_lei`]).
    pub(super) fn set_controlo_e10(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        use crate::ids;
        match id {
            x if x == ids::PAINTER_WETPAINT_DILUTION => self.set(Knob::Dilution, v),
            x if x == ids::PAINTER_WETPAINT_MEDIUM => self.set(Knob::Medium, v),
            x if x == ids::PAINTER_WETPAINT_BODY => self.set(Knob::Body, v),
            x if x == ids::PAINTER_WETPAINT_HARDNESS => self.set_hardness(v),
            x if x == ids::PAINTER_WETPAINT_VOL_OPACITY => self.set(Knob::Volume, v),
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dureza_e_uma_vista_exacta_do_paper_gate() {
        let mut k = WetKnobs::DEFAULT;
        for h in [0.0, 0.1, 0.25, 0.5, 0.73, 1.0] {
            k.set_hardness(h);
            assert!(
                (k.hardness() - h).abs() < 1e-12,
                "Hardness {h} → {}",
                k.hardness()
            );
        }
        let d = &KNOB_DEFS[Knob::PaperGate as usize];
        let mut g = d.min;
        while g <= d.max {
            k.set(Knob::PaperGate, g);
            let h = k.hardness();
            k.set_hardness(h);
            assert!((k.get(Knob::PaperGate) - g).abs() < 1e-12, "paperGate {g}");
            g += d.step;
        }
        k.set_hardness(1.0);
        assert_eq!(k.get(Knob::PaperGate), 0.0);
        k.set_hardness(0.0);
        assert_eq!(k.get(Knob::PaperGate), d.max);
        k.set_hardness(-3.0);
        assert_eq!(k.get(Knob::PaperGate), d.max, "fora da faixa prende");
    }

    #[test]
    fn o_pincel_sem_preset_e_o_wet_round_migrado() {
        let k = WetKnobs::DEFAULT;
        let (d, v) = ph2d_wet_paint::painter::receita::migra_para_controlos(
            k.get(Knob::PigmentPerDab),
            k.get(Knob::WaterPerDab),
            k.water,
            k.get(Knob::Intensity),
            1.0,
        );
        assert!(
            (k.dilution() - d).abs() <= 1e-12 * d.abs(),
            "{} × {d}",
            k.dilution()
        );
        assert!(
            (k.volume_um() - v).abs() <= 1e-12 * v.abs(),
            "{} × {v}",
            k.volume_um()
        );
        assert_eq!((k.medium(), k.body()), (0.0, 0.0));
    }
}
