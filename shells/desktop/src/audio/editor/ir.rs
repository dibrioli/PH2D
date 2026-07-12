//! The loaded **impulse response** — the room the convolution reverb puts sounds in.
//!
//! An IR is a *resource*, not a parameter. Every other effect in the rack is fully described
//! by a handful of floats, which is what lets the panel auto-populate from a table and the
//! `build` function be a plain `fn(&[f32]) -> FxCommand`. A room is a buffer.
//!
//! The seam that keeps that from infecting the rack: the IR lives **here**, and `build` — which
//! runs in the shell and therefore may reach shell state — **bakes it into the effect value**
//! (`TailEffect::Convolution` carries an `Arc<[f32]>`). Downstream of `build`, the effect is
//! still a self-contained value: pure, testable, and byte-identical at its neutral point, like
//! every other stage in the chain. The alternative — an ambient IR that `apply` reaches for —
//! would have quietly broken all three.
//!
//! Thread-local: the rack, the panel and this bridge all run on the main thread.

use std::cell::RefCell;
use std::sync::Arc;

/// The room, as loaded: interleaved samples, its channel count, its rate, and the file's name
/// (so the panel can say which room is loaded, rather than leaving the user to remember).
#[derive(Clone, Default)]
struct Ir {
    samples: Arc<[f32]>,
    channels: u8,
    rate: u32,
    name: String,
}

thread_local! {
    static IR: RefCell<Ir> = RefCell::new(Ir {
        samples: Vec::new().into(),
        ..Default::default()
    });
}

/// Load an impulse response from disk. Any format the audio decoder reads is a room.
///
/// A failed load leaves the previous IR in place rather than clearing it: silently swapping a
/// working room for no room, because a file did not decode, is the kind of thing the user
/// discovers three edits later.
pub(crate) fn load(path: &std::path::Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        eprintln!("audio: cannot read IR {}", path.display());
        return false;
    };
    let data = match ph2d_audio_decode::decode(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("audio: IR decode failed for {}: {e}", path.display());
            return false;
        }
    };
    let name = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("room")
        .to_string();
    set(
        data.samples().to_vec(),
        data.format().channel_count().min(255) as u8,
        data.format().sample_rate,
        &name,
    );
    true
}

/// Install a room. The load path above goes through here, and so does the rack's own gate —
/// which has to be able to seed a room, because the Convolution Reverb is the one effect whose
/// arming knob is not sufficient on its own: without an IR it is bypassed *by design*, however
/// wet the Mix.
pub(crate) fn set(samples: Vec<f32>, channels: u8, rate: u32, name: &str) {
    let ir = Ir {
        samples: samples.into(),
        channels,
        rate,
        name: name.to_string(),
    };
    IR.with(|c| *c.borrow_mut() = ir);
}

/// The room, for `build` to bake into a `TailEffect::Convolution`.
pub(crate) fn samples() -> Arc<[f32]> {
    IR.with(|c| c.borrow().samples.clone())
}

/// Its channel count (mono = one room for both sides; stereo = the room's own width).
pub(crate) fn channels() -> u8 {
    IR.with(|c| c.borrow().channels)
}

/// The rate it was captured at — resampled into the clip's at render, so the room is the room.
pub(crate) fn rate() -> u32 {
    IR.with(|c| c.borrow().rate)
}

/// The loaded room's name, and how long it rings for — the readout the panel shows so the user
/// knows *which* room is loaded, and so an empty slot is visibly empty.
pub(crate) fn readout() -> String {
    IR.with(|c| {
        let ir = c.borrow();
        if ir.samples.is_empty() || ir.rate == 0 {
            return String::new();
        }
        let frames = ir.samples.len() / usize::from(ir.channels.max(1));
        let secs = frames as f32 / ir.rate as f32;
        format!("{} \u{b7} {secs:.1}s", ir.name)
    })
}
