//! Dev smoke for the **shipping targets** (`PH2D_AUDIO_DELIVERY_SMOKE=1`) — a clip built so the
//! Delivery section's three rows *teach the trade* rather than just print it.
//!
//! # The material is chosen so each target visibly pays for something
//!
//! - A **15 kHz shimmer.** Mobile ships at 24 kHz, whose Nyquist is 12 kHz — so that shimmer
//!   **cannot exist** in the Mobile variant. Not "is quieter": cannot exist. Export the set,
//!   load `…mobile.ogg` back, and it is gone. That is what a quarter of the RAM costs, and no
//!   readout can show it to you.
//! - A **stereo image** (the two channels genuinely differ). Mobile folds to mono, which is the
//!   other half of the saving, and it is audible on headphones.
//! - A **loop region and two markers.** Only the lossless target carries them (they live in WAV
//!   `smpl` / `cue` chunks), which is why Console is on the list at all — and the panel already
//!   warns "Drops loop points and markers" while the choice can still be changed.
//!
//! # What to look at
//!
//! Three rows, and the **RAM figures differ** — Mobile holds a **quarter** of what the others do.
//! That is the entire point: a variant that only swapped the codec would print the same RAM three
//! times, because a codec has never bought back a byte of memory (ADR-0118). Then **Export Set**:
//! one folder, three files, each conformed to its own format first.

use ph2d_audio::{AudioFormat, SampleData};

use crate::audio::AudioSystem;
use crate::audio::editor::EditorTransport;

const SR: u32 = 48_000;
const SECS: usize = 3;
/// Above the 12 kHz Nyquist of a 24 kHz variant — so Mobile *cannot* carry it, whatever the
/// bitrate. This is the tone that makes the trade audible instead of theoretical.
const SHIMMER_HZ: f32 = 15_000.0;

/// A tone with harmonics, so it is **present on any speaker** and not a bare bass sine.
///
/// The first delivery clip made the body a single 220/330 Hz sine. Correct on paper, and the
/// Mobile export measured a healthy −15 dB — but Mobile is the target that *loses the 15 kHz
/// shimmer*, so what is left is only that low sine, and next to the master (which keeps the bright
/// shimmer a speaker reproduces loudest) it reads as "nothing". The fix is not more level, it is
/// **mid-range content**: a fundamental plus two harmonics puts energy from `hz` up to `3·hz`,
/// which every speaker plays, so the Mobile file stands on its own as a sound.
fn voiced(t: f32, hz: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    0.30 * (tau * hz * t).sin()
        + 0.15 * (tau * 2.0 * hz * t).sin()
        + 0.09 * (tau * 3.0 * hz * t).sin()
}

/// The smoke's material. See the module docs for why each ingredient is there.
pub(crate) fn delivery_clip() -> SampleData {
    let tau = std::f32::consts::TAU;
    let frames = SR as usize * SECS;
    SampleData::from_fn(frames * 2, AudioFormat::stereo(SR), |i| {
        let t = (i / 2) as f32 / SR as f32;
        let right = i % 2 == 1;
        // A real stereo image: the two channels are NOT the same signal, so folding to mono is a
        // loss you can hear. Different notes per channel, each with harmonics up to ~1 kHz — all
        // well under Mobile's 12 kHz Nyquist, so the whole body survives to the Mobile file.
        let body = if right {
            voiced(t, 330.0)
        } else {
            voiced(t, 220.0)
        };
        // The shimmer that a 24 kHz variant is physically unable to represent.
        let shimmer = 0.25 * (tau * SHIMMER_HZ * t).sin();
        (body + shimmer).clamp(-1.0, 1.0)
    })
}

impl AudioSystem {
    /// Stage the delivery clip, with the loop points and markers only the lossless target keeps.
    pub(crate) fn editor_delivery_smoke(&mut self) {
        let _ = self.engine.stop_preview();
        self.editor.name = "delivery-smoke".to_string();
        self.editor.clip = Some(ph2d_audio_edit::EditClip::new(delivery_clip()));
        self.editor.state = EditorTransport::Stopped;
        self.editor.scrub_frame = Some(0);
        if let Some(clip) = self.editor.clip.as_mut() {
            let frames = clip.frame_count();
            clip.set_loop_region(Some(frames / 3..frames * 2 / 3));
            clip.add_marker(frames / 4, "M1");
            clip.add_marker(frames * 3 / 4, "M2");
        }
        println!(
            "audio: delivery smoke staged (PH2D_AUDIO_DELIVERY_SMOKE)\n  \
             clip: a stereo body + a 15 kHz shimmer, with a loop region and two markers.\n  \
             look: the Delivery section lists three targets, and their RAM figures DIFFER --\n  \
                   Mobile holds a QUARTER (24 kHz, mono). A codec swap would print the same\n  \
                   number three times; only conforming the audio buys memory back.\n  \
             do:   Export Set -> a SAVE dialog (name + folder) -> three files. Load .mobile.ogg\n  \
                   back: it STILL PLAYS (the body tone), but duller -- the bright 15 kHz shimmer\n  \
                   is GONE, because 24 kHz cannot represent it (Nyquist is 12 kHz). Compare it to\n  \
                   .console.wav (full + loop points + markers) to hear the trade."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::dsp::{Biquad, BiquadCoeffs};
    use ph2d_audio_encode::{PLATFORMS, ram_bytes};

    /// RMS of whatever survives above 12 kHz, measured back at the master's rate so the two
    /// variants are comparable at all.
    fn shimmer_rms(d: &SampleData) -> f32 {
        let back = ph2d_audio_edit::conform(d, AudioFormat::stereo(SR));
        let c = BiquadCoeffs::highpass(SR as f32, 12_000.0, 0.707);
        let (mut a, mut b) = (Biquad::new(c), Biquad::new(c));
        let s: Vec<f32> = back
            .samples()
            .iter()
            .step_by(2)
            .map(|&x| b.process(a.process(x)))
            .collect();
        (s.iter().map(|x| x * x).sum::<f32>() / s.len() as f32).sqrt()
    }

    /// **The smoke's clip actually demonstrates the trade it claims to.**
    ///
    /// Mobile is 24 kHz, so its Nyquist is 12 kHz and the 15 kHz shimmer *cannot exist* in it. If
    /// the clip had no content up there, Enio would export the set, hear no difference, and the
    /// only evidence that the variants are real would be three numbers on a panel.
    #[test]
    fn the_mobile_variant_physically_cannot_carry_the_shimmer() {
        let d = delivery_clip();
        let mobile = PLATFORMS.iter().find(|p| p.name == "Mobile").unwrap();
        let desktop = PLATFORMS.iter().find(|p| p.name == "Desktop").unwrap();

        let master = shimmer_rms(&d);
        let m = shimmer_rms(&ph2d_audio_edit::conform(&d, mobile.format()));
        let dsk = shimmer_rms(&ph2d_audio_edit::conform(&d, desktop.format()));
        println!("shimmer above 12 kHz: master {master:.4}, mobile {m:.4}, desktop {dsk:.4}");

        assert!(
            master > 0.05,
            "the clip has no shimmer to lose ({master:.4})"
        );
        assert!(
            m < master * 0.1,
            "the 15 kHz shimmer survived the 24 kHz conform ({m:.4} of {master:.4}) -- it cannot \
             have, so the conform is not really resampling"
        );
        assert!(
            dsk > master * 0.5,
            "the DESKTOP variant lost the shimmer too ({dsk:.4}) -- then the smoke shows a \
             resampling bug, not a platform trade"
        );
    }

    /// ...and the panel's headline claim, on this exact clip: **the targets hold different amounts
    /// of memory.** Mobile a quarter, the others the master.
    #[test]
    fn the_smoke_clip_shows_three_different_ram_figures() {
        let d = delivery_clip();
        let ram: Vec<usize> = PLATFORMS
            .iter()
            .map(|p| ram_bytes(&ph2d_audio_edit::conform(&d, p.format())))
            .collect();
        println!("RAM: {ram:?}");
        assert!(
            ram[0] * 3 < ram[1],
            "Mobile ({}) is not meaningfully cheaper than Desktop ({}) on this clip",
            ram[0],
            ram[1]
        );
    }

    /// Broadband RMS of the left channel, at whatever rate the clip is.
    fn rms(d: &SampleData) -> f32 {
        let ch = d.format().channel_count().max(1);
        let s: Vec<f32> = d.samples().iter().step_by(ch).copied().collect();
        (s.iter().map(|x| x * x).sum::<f32>() / s.len().max(1) as f32).sqrt()
    }

    /// Manual helper: write the real Mobile file to `$PROBE_OUT` so it can be played in an external
    /// player or inspected with ffprobe. `#[ignore]` — it is a probe, not a gate.
    #[test]
    #[ignore]
    fn write_mobile_to_disk() {
        let mobile = PLATFORMS.iter().find(|p| p.name == "Mobile").unwrap();
        let conformed = ph2d_audio_edit::conform(&delivery_clip(), mobile.format());
        let bytes = ph2d_audio_encode::encode_ogg(&conformed, mobile.quality).unwrap();
        let path = std::env::var("PROBE_OUT").expect("set PROBE_OUT=/path/to/out.ogg");
        std::fs::write(&path, &bytes).unwrap();
        println!("wrote {} bytes to {path}", bytes.len());
    }

    /// **The sibling my shimmer gate was missing: the Mobile file has to still HAVE the body.**
    ///
    /// Every earlier gate proved the 15 kHz shimmer is *gone* from Mobile — an ABSENCE. None
    /// proved the 220/330 Hz body *survives* — the PRESENCE. A Mobile export that is dead silence
    /// passes every one of them (no shimmer, right RAM), which is exactly what shipped: the Enio
    /// loaded `…mobile.ogg` back and heard nothing. This walks the real product path
    /// (conform → encode_ogg → decode) and asserts the body is still there.
    #[test]
    fn the_mobile_export_is_not_silent() {
        let mobile = PLATFORMS.iter().find(|p| p.name == "Mobile").unwrap();
        let master = delivery_clip();
        let conformed = ph2d_audio_edit::conform(&master, mobile.format());
        let after_conform = rms(&conformed);

        let bytes = ph2d_audio_encode::encode_ogg(&conformed, mobile.quality)
            .expect("Mobile is Ogg Vorbis");
        // An Ogg Vorbis file must NOT sniff as Opus — both ride an Ogg container, so a check that
        // only saw "OggS" would send this to the Opus decoder, which cannot read Vorbis: the load
        // would fail, the editor would hold no clip, and Play would be silence.
        assert!(
            !ph2d_audio_opus::is_opus(&bytes),
            "the Mobile .ogg (Vorbis) sniffed as Opus -- the decode door would misroute it and the \
             editor would load nothing"
        );
        // Route through the SAME door `editor_load` uses, not `ph2d_audio_decode::decode` direct.
        let decoded = crate::audio::decode_any::decode(&bytes)
            .expect("the editor's own door must read the file the editor wrote");
        let after_ogg = rms(&decoded);

        println!(
            "RMS  master {:.4}  ->  conform(24k mono) {after_conform:.4}  ->  ogg round-trip {after_ogg:.4}",
            rms(&master)
        );
        // The surviving body has real mid-range presence, not a bare bass sine, so the Mobile file
        // is clearly a SOUND and not just numerically non-zero.
        assert!(
            after_conform > 0.1,
            "conform to Mobile produced near-silence ({after_conform:.4}) -- the body did not \
             survive the resample/fold, so the bug is in `conform`, not the codec"
        );
        assert!(
            after_ogg > 0.05,
            "the Mobile .ogg is silent after a round-trip ({after_ogg:.4}) though conform kept the \
             body ({after_conform:.4}) -- the bug is in encode_ogg or the decode"
        );
    }
}
