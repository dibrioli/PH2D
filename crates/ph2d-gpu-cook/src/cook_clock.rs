//! **Quando um cook acontece** — o relógio que a bomba entrega ao device.
//!
//! Extraído do `lib.rs` no teto de LOC do HR-18, pela costura que já estava lá:
//! o `lib.rs` responde *o que um cook FAZ* (o walk, os passes, o lowering) e este
//! arquivo responde *em que instante ele acontece*. Os dois campos não são
//! redundantes, e o doc abaixo diz por quê.

/// When a cook happens: the continuous `playhead` the kernels see, and the
/// fixed `tick` it stands on.
///
/// They are not redundant. The playhead is what a kernel reads (and what a sim
/// derives its own `dt` from — the state carries `sim_t`); the tick is the
/// SEQUENCE number, which is how the caller knows whether it is continuing this
/// sim or jumping. A stateless plan has no sequence to keep, hence `Option`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CookClock {
    pub playhead: f64,
    /// The fixed tick, for a plan that [`GpuPlan::drives_a_loop`]. `None` — a
    /// stateless cook (`f(params, playhead)`, F1.1/Fase 2): nothing to sequence.
    pub tick: Option<u64>,
}

impl CookClock {
    /// A stateless cook at `playhead` — the F1.1/Fase 2 shape.
    pub fn at(playhead: f64) -> Self {
        Self {
            playhead,
            tick: None,
        }
    }
}
