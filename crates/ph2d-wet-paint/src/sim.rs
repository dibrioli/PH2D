//! Simulation orchestrator (port of `sim.js`, SPEC §5): fixed 40 Hz steps
//! with per-pass cadences, plus the adaptive drying cadence.
//!
//! Per step (frame counter n, incremented first):
//!   n % 2 == 0        rebuild the active region; if no fluid remains, idle
//!   n % dry_every == 0 drying / settle / re-wet
//!   n % 4 == 0        flow-field build WITH the absorbency brake
//!   other frames      cheap velocity smoothing, never braked (+ diffusion ext)
//!   every frame       advection + gravity injection, then the drain boundaries
//!   n % 3 == 0        pressure projection, then velocity boundaries again
//!
//! The caller (facade or test) owns the accumulator and the "paused while the
//! pointer is down" rule; this module only advances single fixed steps.

use crate::colorops::ColorMix;
use crate::drying::{drying_pass, fast_dry};
use crate::grid::Grid;
use crate::solver::{
    advect, apply_boundaries, build_flow_field, diffusion_pass, project, rebuild_active_region,
    smooth_velocity,
};
use crate::tuning::{KNOB_COUNT, Knob, Tuning};

pub const SIM_HZ: f64 = 40.0;

/// Snapshot of every knob the solver reads, taken once per step so inner
/// loops never touch the registry (the JS `gatherParams`), plus the color
/// mixer routing.
///
/// `Clone` porque um passo INTERROMPIDO o carrega ([`StepCursor`]): os params são
/// capturados **uma vez por passo**, e retomar um passo tem de continuar com os
/// mesmos — se ele os re-colhesse no meio, um knob mexido entre dois frames
/// aplicaria metade de um passo com a lei nova.
#[derive(Clone)]
pub struct Params {
    values: [f64; KNOB_COUNT],
    pub mix: ColorMix,
    /// The advection's incoming-color mean also routes through K–M when ON.
    pub km_mixing: bool,
}

impl Params {
    #[inline]
    pub fn k(&self, knob: Knob) -> f64 {
        self.values[knob as usize]
    }
}

/// Sim state that lives OUTSIDE the grid: cadence, tilt, experimental flags.
/// (The JS holds the grid reference here; in Rust the caller passes the grid
/// to [`sim_step`] — the facade owns which layer's grid simulates.)
pub struct Sim {
    pub frame: u64,
    // Adaptive drying cadence, starts calm.
    pub dry_every: u64,
    pub evap_scale: f64,
    pub rewet_base: f64,
    // Gravity: the tilt dial provides direction + magnitude; boots ON
    // pointing straight down at the gravity knob's magnitude.
    pub tilt_on: bool,
    pub tilt_dir_x: f64,
    pub tilt_dir_y: f64,
    /// Dial radius fraction relative to the knob value.
    pub tilt_scale: f64,
    /// Tests may pin an exact vector.
    pub gravity_override: Option<[f64; 2]>,
    /// Experimental color routing (K–M pigment mixing checkbox).
    pub km_mixing: bool,
    /// Force-skip every SPEC §17 extension code path (neutrality test §18.10).
    pub ext_bypass: bool,
    /// **Um passo INTERROMPIDO** — ver [`StepCursor`] e [`sim_step_stage`].
    pending: Option<StepCursor>,
}

/// **Onde um passo interrompido parou.**
///
/// Um passo é uma sequência de sete estágios com cadências próprias, e na escala do produto ele custa
/// **38,7 ms** (medido, `ph2d-wet-paint/tests/measure_pass_cost.rs`) — mais que **dois quadros** de
/// 60 Hz. Enquanto ele era ATÔMICO, o frame que o continha estourava por construção, e nenhum
/// orçamento conserta isso: o smoke do Enio mediu `tool-tick pico 73,70ms` com o app a 55 fps.
///
/// O maior estágio SOZINHO custa 10,26 ms (`build_flow_field`), que **cabe** na folga de um quadro.
/// Então o passo deixou de ser atômico: o chamador roda estágios enquanto tem orçamento e retoma no
/// frame seguinte.
///
/// ⚠️ **Byte-idêntico por CONSTRUÇÃO, e é o desenho inteiro:** os mesmos estágios, na mesma ordem, com
/// os MESMOS params — quem os capturou foi o início do passo, não cada estágio. [`sim_step`] passou a
/// ser *o laço sobre os estágios*, então não existe uma segunda implementação para divergir.
#[derive(Clone)]
pub struct StepCursor {
    /// Os params CAPTURADOS no início do passo (a razão de [`Params`] ser `Clone`).
    p: Params,
    grav: [f64; 2],
    /// O `sim.frame` deste passo — as cadências (`n % 2`, `n % 3`, `n % 4`) o leem.
    n: u64,
    /// O próximo estágio a rodar (ver [`sim_step_stage`]).
    stage: u8,
    /// O `vmax` que o `advect` devolveu — o estágio final o converte em cadência de secagem.
    vmax: f64,
}

impl Default for Sim {
    fn default() -> Self {
        Sim {
            frame: 0,
            dry_every: 6,
            evap_scale: 0.001,
            rewet_base: 0.0001,
            tilt_on: true,
            tilt_dir_x: 0.0,
            tilt_dir_y: 1.0,
            tilt_scale: 1.0,
            gravity_override: None,
            km_mixing: false,
            ext_bypass: false,
            pending: None,
        }
    }
}

impl Sim {
    /// Há um passo interrompido em voo?
    #[must_use]
    pub const fn step_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// **Termina um passo em voo** — a porta que toda operação de CANVAS chama primeiro.
    ///
    /// O grid entre dois estágios é intermediário: `wet_canvas`/`dry_canvas`/`fast_dry`/`clear` e o
    /// `capture_history` do engine agem sobre a folha INTEIRA, e fazê-lo no meio de um passo
    /// misturaria meia física com a ação. Sem passo em voo é um `if` e nada mais.
    pub fn drain_step(&mut self, g: &mut Grid, tuning: &Tuning) {
        while self.pending.is_some() {
            sim_step_stage(self, g, tuning);
        }
    }
}

/// The tilt DIAL's direction for a spoke (0..11, 30° steps, 0 = +x, going
/// clockwise in screen space like the reference dial). The four cardinals
/// return EXACT unit vectors — the boot spoke is 3 (straight down) and the
/// engine boots `tilt_dir = (0, 1)` exactly, so a boot-time reconcile must be
/// a bit-exact no-op (`libm::cos(PI/2)` is `6.1e-17`, not `0`); the other
/// spokes take the reference's raw cos/sin.
#[must_use]
pub fn tilt_dir_for_spoke(spoke: u8) -> [f64; 2] {
    match spoke % 12 {
        0 => [1.0, 0.0],
        3 => [0.0, 1.0],
        6 => [-1.0, 0.0],
        9 => [0.0, -1.0],
        s => {
            let a = f64::from(s) * 30.0 * std::f64::consts::PI / 180.0;
            [libm::cos(a), libm::sin(a)]
        }
    }
}

impl Sim {
    /// The effective gravity vector for this step.
    pub fn gravity(&self, tuning: &Tuning) -> [f64; 2] {
        if let Some(g) = self.gravity_override {
            return g;
        }
        if !self.tilt_on {
            return [0.0, 0.0];
        }
        let mag = tuning.get(Knob::Gravity) * self.tilt_scale;
        [self.tilt_dir_x * mag, self.tilt_dir_y * mag]
    }

    /// Snapshot every knob + the color routing (the JS `gatherParams`).
    pub fn gather_params(&self, tuning: &Tuning) -> Params {
        Params {
            values: tuning.snapshot(),
            mix: if self.km_mixing {
                ColorMix::Km
            } else {
                ColorMix::Plain
            },
            km_mixing: self.km_mixing,
        }
    }
}

/// Advance one fixed 40 Hz step. Returns false when the sim is idle (no fluid).
/// **UM passo, atomicamente** — o laço sobre [`sim_step_stage`].
///
/// ⚠️ Ele é o LAÇO e não uma segunda implementação: a suíte de aceitação e o fingerprint pinado
/// entram por aqui, então não existe caminho paralelo para divergir do que o produto roda por
/// estágios.
pub fn sim_step(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> bool {
    loop {
        if let Some(ran) = sim_step_stage(sim, g, tuning) {
            return ran;
        }
    }
}

/// Roda **UM estágio** do passo corrente (começando um, se não houver).
///
/// - `Some(ran)` — o passo COMPLETOU nesta chamada; `ran` é o que [`sim_step`] devolveria.
/// - `None` — um estágio rodou e o passo continua no próximo chamado.
///
/// O chamador roda estágios enquanto tem orçamento de frame; ver [`StepCursor`] para o porquê.
///
/// ⚠️ **Um passo em voo é estado do `Sim`, e quem faz uma operação de CANVAS tem de drená-lo**
/// ([`Sim::drain_step`] / `Engine::drain_step`) — o grid entre dois estágios é intermediário, e um
/// `wet_canvas`/`dry_canvas`/`fast_dry`/`capture_history` sobre ele misturaria meia física com a ação.
pub fn sim_step_stage(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> Option<bool> {
    let mut c = match sim.pending.take() {
        Some(c) => c,
        None => {
            // Começo de passo: o guard, os params (UMA vez — ver `StepCursor`) e o relógio.
            if !g.has_fluid {
                return Some(false);
            }
            let p = sim.gather_params(tuning);
            let grav = sim.gravity(tuning);
            sim.frame += 1;
            StepCursor {
                p,
                grav,
                n: sim.frame,
                stage: 0,
                vmax: 0.0,
            }
        }
    };
    let n = c.n;
    match c.stage {
        0 => {
            if n.is_multiple_of(2) {
                rebuild_active_region(g);
                if !g.has_fluid {
                    // Aborta o passo: nada de cadência, nada de cursor.
                    return Some(false);
                }
            }
        }
        1 => {
            if n.is_multiple_of(sim.dry_every) {
                // The evaporation / re-wet knobs are straight multipliers on the
                // cadence-adaptive scales.
                drying_pass(
                    g,
                    &c.p,
                    sim.evap_scale * c.p.k(Knob::Evaporation),
                    sim.rewet_base * c.p.k(Knob::Rewet),
                    sim.ext_bypass,
                );
            }
        }
        2 => {
            if n.is_multiple_of(4) {
                build_flow_field(g, &c.p, c.grav[0], c.grav[1], sim.ext_bypass);
            } else {
                smooth_velocity(g, &c.p);
                if !sim.ext_bypass && c.p.k(Knob::ExtDiffusion) > 0.0 {
                    diffusion_pass(g, &c.p);
                }
            }
        }
        3 => c.vmax = advect(g, &c.p, c.grav[0], c.grav[1]),
        4 => apply_boundaries(g, false),
        5 => {
            if n.is_multiple_of(3) {
                project(g, &c.p);
                apply_boundaries(g, true);
            }
        }
        _ => {
            // Adaptive drying cadence for the NEXT frames, from this advection's max
            // velocity component: calm water dries faster per pass, flowing water is
            // dried gently but often.
            if c.vmax < 0.5 {
                sim.dry_every = 6;
                sim.evap_scale = 0.001;
                sim.rewet_base = 0.0001;
            } else {
                sim.dry_every = 3;
                sim.evap_scale = 0.00025;
                sim.rewet_base = 0.000025;
            }
            return Some(true);
        }
    }
    c.stage += 1;
    sim.pending = Some(c);
    None
}

/// **A ROTA ATÔMICA CONGELADA** — o `sim_step` que shipava antes de o passo ser retomável, verbatim.
///
/// ⚠️ **Ela existe porque o gate de identidade não pode ser `sim_step` contra `sim_step_stage`:**
/// `sim_step` **É** o laço sobre os estágios, então os dois lados passam pelo MESMO código, e uma
/// mutação dentro do estágio move os dois — *razão entre dois doentes, verde por construção*. Medido:
/// três mutações (params re-colhidos por estágio · relógio por estágio · um estágio pulado)
/// **sobreviveram** ao gate escrito daquele jeito.
///
/// `#[cfg(test)]` de propósito: um `pub fn` sem chamador não é código morto silencioso, é uma
/// **segunda resposta** esperando alguém chamá-la (a lição do `warp_axis` / do `serial_side`).
#[cfg(test)]
fn sim_step_atomic_reference(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) -> bool {
    if !g.has_fluid {
        return false;
    }
    let p = sim.gather_params(tuning);
    let grav = sim.gravity(tuning);
    sim.frame += 1;
    let n = sim.frame;

    if n.is_multiple_of(2) {
        rebuild_active_region(g);
        if !g.has_fluid {
            return false;
        }
    }
    if n.is_multiple_of(sim.dry_every) {
        drying_pass(
            g,
            &p,
            sim.evap_scale * p.k(Knob::Evaporation),
            sim.rewet_base * p.k(Knob::Rewet),
            sim.ext_bypass,
        );
    }
    if n.is_multiple_of(4) {
        build_flow_field(g, &p, grav[0], grav[1], sim.ext_bypass);
    } else {
        smooth_velocity(g, &p);
        if !sim.ext_bypass && p.k(Knob::ExtDiffusion) > 0.0 {
            diffusion_pass(g, &p);
        }
    }
    let vmax = advect(g, &p, grav[0], grav[1]);
    apply_boundaries(g, false);
    if n.is_multiple_of(3) {
        project(g, &p);
        apply_boundaries(g, true);
    }
    if vmax < 0.5 {
        sim.dry_every = 6;
        sim.evap_scale = 0.001;
        sim.rewet_base = 0.0001;
    } else {
        sim.dry_every = 3;
        sim.evap_scale = 0.00025;
        sim.rewet_base = 0.000025;
    }
    true
}

#[cfg(test)]
#[path = "sim_resumable_tests.rs"]
mod sim_resumable_tests;

/// Fast dry action (SPEC §12), routed here so it shares params.
pub fn sim_fast_dry(sim: &mut Sim, g: &mut Grid, tuning: &Tuning) {
    if !g.has_fluid {
        return;
    }
    let p = sim.gather_params(tuning);
    fast_dry(g, &p, sim.rewet_base * p.k(Knob::Rewet), sim.ext_bypass);
}
