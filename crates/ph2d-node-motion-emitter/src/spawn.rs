//! **QUANDO as partículas nascem** — a lei de contagem do emissor, num irmão (TOP-20 #18, W0).
//!
//! Saiu do `lib.rs` pelo tecto de LOC (700) na costura que a própria pergunta desenha: lá fica
//! *como* uma partícula é (onde nasce, para onde vai, de que tamanho), aqui *quando* ela existe.
//! O texto movido é o de sempre, ao bit; o que esta wave acrescenta é o braço `Scheduled`.

use ph2d_nodegraph::gpu::{ID_WRAP, SourceWindow};

use crate::schedule::{EMIT_SCHEDULED, Schedule};

/// **WHEN particles are born** — a steady stream, or all at once, or a stream that switches on
/// and off.
///
/// The three are not a flag on one law, they are three closed forms in `t`, and the type is what
/// keeps them from being mixed: a burst has no rate and a stream has no period, so passing the
/// wrong number is not expressible.
#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum Spawn {
    /// A steady stream: particle `k` is born at `k / rate`. The emitter that always shipped.
    Continuous { rate: f32 },
    /// `count` particles at `time`, and again every `period` after it (`0` = one burst only).
    Burst { count: u32, time: f32, period: f32 },
    /// **A stream on a SCHEDULE** (TOP-20 #18): particle `k` is born when the emission clock `τ`
    /// reaches `k / rate` — see [`crate::schedule`]. The schedule itself travels beside the
    /// spawn (it owns a list), so every law below takes it as an argument.
    Scheduled { rate: f32 },
}

impl Spawn {
    /// The integer the `emit_mode` param carries; anything else reads as continuous, which is
    /// what every graph that predates the param means.
    pub(crate) fn from_params(mode: f32, rate: f32, count: f32, time: f32, period: f32) -> Self {
        match mode.round() as i32 {
            1 => Self::Burst {
                count: count.max(0.0) as u32,
                time,
                period: period.max(0.0),
            },
            EMIT_SCHEDULED => Self::Scheduled { rate },
            _ => Self::Continuous { rate },
        }
    }

    /// **The always-on schedule IS the continuous stream** — resolved here, once, so the arm
    /// that always shipped runs character for character and the identity is byte-exact by
    /// construction rather than by an arithmetic that happens to agree.
    fn resolved(self, sched: &Schedule) -> Self {
        match self {
            Self::Scheduled { rate } if sched.is_always_on() => Self::Continuous { rate },
            other => other,
        }
    }

    /// **When particle `id` was born** — in `f64`, for the count law, which is the one place that
    /// can compute `age_first` without cancellation. `+∞` = never (a schedule that ended first).
    fn born_at(self, sched: &Schedule, id: f64) -> f64 {
        match self.resolved(sched) {
            Self::Scheduled { rate } => sched.birth(id / f64::from(rate)).unwrap_or(f64::INFINITY),
            Self::Continuous { rate } => id / f64::from(rate),
            Self::Burst {
                count,
                time,
                period,
            } => f64::from(time) + (id / f64::from(count)).floor() * f64::from(period),
        }
    }

    /// **How much younger particle `first + k` is than particle `first`** — the per-particle step
    /// the `age` column walks, in `f32`, and the expression the WGSL mirror restates.
    ///
    /// ⚠️ **The continuous arm is the expression that always shipped, character for character,
    /// and that is deliberate.** Deriving it from [`Self::born_at`] instead (`born_at(first+k) −
    /// born_at(first)`, in `f64`, cast down) is *equivalent* arithmetic and NOT the same bits —
    /// and byte-identity for a mode that already ships is a thing to construct, never to promise.
    /// The two-step form is also the one that cancels: `t − first/rate − k/rate` loses the large
    /// `t` twice, `age_first − k/rate` loses it once.
    pub(crate) fn age_step(self, sched: &Schedule, first: u32, k: usize) -> f32 {
        match self.resolved(sched) {
            Self::Continuous { rate } => k as f32 / rate,
            // ⚠️ **Across a gap the step is not constant** — two particles either side of an
            // `off` are a whole pause apart — so the scheduled arm takes the `f64` difference of
            // the two births. The device never runs it (`applicable` refuses the mode), so there
            // is no WGSL mirror to keep in step with.
            Self::Scheduled { .. } => {
                (self.born_at(sched, f64::from(first) + k as f64)
                    - self.born_at(sched, f64::from(first))) as f32
            }
            // Within one burst every particle was born together, so the step is ZERO and jumps by
            // `period` at each burst boundary — which is what `floor(id / count)` counts.
            Self::Burst { count, period, .. } => {
                let n = f64::from(count.max(1));
                ((f64::from(first) + k as f64) / n).floor().mul_add(
                    f64::from(period),
                    -(f64::from(first) / n).floor() * f64::from(period),
                ) as f32
            }
        }
    }
}

/// **The count law, and the only copy of it** — in `f64`, because the spawn index
/// is what has to stay exact.
///
/// `emit` and the GPU `count_law` used to compute this independently in
/// `f32`, and parity held only because both read the same `f32`. Two doors to
/// one question ([[feedback_two_doors_to_the_same_question_diverge]]), and both
/// of them wrong at scale: `floor(t·rate)` in `f32` starts skipping integers at
/// 2²⁴, so a rate in the millions loses the window after four seconds. Now there
/// is one door, it answers in integers, and the kernel is TOLD the answer.
///
/// ⚠️ **Both modes return ONE CONTIGUOUS id range, and that is the design.** For the burst that
/// is not a coincidence: numbering burst `k`'s particles `[k·N, (k+1)·N)` makes the alive set the
/// union of an INTERVAL of `k`, whose ids are contiguous by construction. A representation that
/// needed two ranges would have forced `SourceWindow` (one `first`, one `count`) and the kernel's
/// `first = newest + 1 − count` to grow a second population each — for a picture the artist can
/// already build by composing two emitters through `motion.combine`.
pub(crate) fn window(spawn: Spawn, life: f32, max: usize, t: f32) -> SourceWindow {
    window_in(spawn, &Schedule::ALWAYS, life, max, t)
}

/// [`window`] under a schedule — the one the node calls, and the only one that can answer the
/// `Scheduled` arm.
///
/// ⚠️ **The scheduled window is still ONE contiguous range**, because `τ` is monotone: the
/// newest is the largest `k` already born, the oldest the smallest still inside its life. Both
/// edges are found from `τ` and then CHECKED against the birth itself — `floor`/`ceil` of a
/// product can land one id off exactly at an `off` instant, where `τ` stops but the next birth is
/// a whole pause away.
pub(crate) fn window_in(
    spawn: Spawn,
    sched: &Schedule,
    life: f32,
    max: usize,
    t: f32,
) -> SourceWindow {
    if life <= 0.0 || t < 0.0 {
        return SourceWindow::of_count(0);
    }
    let spawn = spawn.resolved(sched);
    let (t, life) = (f64::from(t), f64::from(life));
    let (newest, oldest) = match spawn {
        Spawn::Scheduled { rate } => {
            if rate <= 0.0 {
                return SourceWindow::of_count(0);
            }
            let r = f64::from(rate);
            let mut newest = (sched.on_time(t) * r).floor();
            while newest >= 0.0 && spawn.born_at(sched, newest) > t {
                newest -= 1.0;
            }
            let mut oldest = (sched.on_time(t - life) * r).ceil().max(0.0);
            while oldest <= newest && spawn.born_at(sched, oldest) < t - life {
                oldest += 1.0;
            }
            if newest < 0.0 {
                return SourceWindow::of_count(0);
            }
            (newest, oldest)
        }
        Spawn::Continuous { rate } => {
            if rate <= 0.0 {
                return SourceWindow::of_count(0);
            }
            let rate = f64::from(rate);
            // Particle k is born at k/rate. Alive at t iff `0 ≤ t − k/rate < life`.
            ((t * rate).floor(), ((t - life) * rate).ceil().max(0.0))
        }
        Spawn::Burst {
            count,
            time,
            period,
        } => {
            let time = f64::from(time);
            if count == 0 || t < time {
                return SourceWindow::of_count(0);
            }
            let n = f64::from(count);
            // Burst `k` fires at `time + k·period` and owns ids `[k·n, (k+1)·n)`. With no period
            // there is exactly one burst, so the interval of `k` collapses to `0..=0` — the same
            // expression, not a special case with its own arithmetic.
            let (k_last, k_first) = if period > 0.0 {
                let p = f64::from(period);
                (
                    ((t - time) / p).floor(),
                    // The SAME boundary convention the continuous arm uses (`ceil`, so an age of
                    // exactly `life` is still alive): the two modes must agree about the edge, or
                    // an artist crossing between them would see a particle blink.
                    ((t - time - life) / p).ceil().max(0.0),
                )
            } else if t - time < life {
                (0.0, 0.0)
            } else {
                return SourceWindow::of_count(0);
            };
            ((k_last + 1.0) * n - 1.0, k_first * n)
        }
    };
    if newest < oldest {
        return SourceWindow::of_count(0);
    }
    // The cap keeps the NEWEST particles: an emitter whose rate outruns the cap
    // should look like a dense young jet, not a frozen ancient cloud.
    let span = (newest - oldest) as u64 + 1;
    let count = span.min(max as u64);
    let first = newest as u64 + 1 - count;
    SourceWindow {
        count: count as usize,
        // Wrapped into the exact `f32` integer range. Below 2²⁴ this is the
        // identity, so every scene that works today is BYTE-identical.
        first: (first % u64::from(ID_WRAP)) as u32,
        // The oldest particle's age, from the one place that can compute it
        // without cancellation. ⚠️ `first`, not `oldest`: the cap may have dropped the oldest
        // bursts, and the age reported has to be the age of the particle that SURVIVED.
        age_first: (t - spawn.born_at(sched, first as f64)) as f32,
    }
}
