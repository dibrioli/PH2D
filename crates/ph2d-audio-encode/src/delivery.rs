//! **Delivery cost** — what an asset costs to *ship* and what it costs to *run*
//! (W6, game asset-prep).
//!
//! The two numbers a game audio asset is judged by, and they answer to different
//! things:
//!
//! - **Disk** is what the player downloads. It is entirely the codec's business: the
//!   same clip is ~10× smaller as Ogg Vorbis than as 16-bit PCM.
//! - **RAM** is what the engine pays while the sound is loaded. It is **not** the
//!   codec's business at all — [`ph2d_audio::SampleData`] holds interleaved `f32`, and
//!   there is no streaming path in the mixer, so **every clip is decoded in full at
//!   load whatever it was shipped as**. A Vorbis asset and a WAV asset of the same
//!   audio cost exactly the same memory.
//!
//! That surprise is the whole reason this readout exists. It is also why there is no
//! "residency" control here: a *Streamed* toggle would have nothing in the engine to
//! honour it (see the module handoff), and a switch that does nothing is worse than a
//! switch that is missing.
//!
//! RAM is checked against [`ph2d_audio::budget`] — the subsystem's 30 MB envelope from
//! HR-13 / SKILL §12.1 — so "how much of the audio budget does this one sound eat?" has
//! an answer before it ships, not after.

use ph2d_audio::SampleData;

use crate::{BitDepth, EncodeError, encode_ogg, encode_wav};

/// How the asset is written to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    /// 16-bit PCM WAV — lossless, ubiquitous, big. Carries `smpl`/`cue` metadata.
    Wav16,
    /// 24-bit PCM WAV — lossless with headroom for further processing.
    Wav24,
    /// Ogg Vorbis, VBR — lossy and roughly 10× smaller. No `smpl`/`cue` side-car, so
    /// loop points and markers do not survive the trip (they are a WAV feature).
    OggVorbis,
    /// **Opus** — the modern lossy codec, and the best quality per bit here. Measured on a
    /// speech clip at the default bitrate: **6.4 %** of the WAV, at 33.8 dB SNR. Like Vorbis it
    /// carries no loop points or markers.
    ///
    /// Opus is a **48 kHz** codec: a clip at another rate is resampled on the way in (ADR-0116).
    Opus,
}

impl Codec {
    /// The order the panel's selector cycles.
    pub const ALL: [Codec; 4] = [Codec::Wav16, Codec::Wav24, Codec::OggVorbis, Codec::Opus];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Codec::Wav16 => "WAV 16-bit",
            Codec::Wav24 => "WAV 24-bit",
            Codec::OggVorbis => "Ogg Vorbis",
            Codec::Opus => "Opus",
        }
    }

    /// The conventional file extension for this codec.
    pub fn extension(self) -> &'static str {
        match self {
            Codec::Wav16 | Codec::Wav24 => "wav",
            Codec::OggVorbis => "ogg",
            Codec::Opus => "opus",
        }
    }

    /// Whether the encoder carries loop points + cue markers into the file. Only the
    /// WAV writer has chunks for them; a Vorbis export drops them on the floor, and the
    /// panel says so rather than letting the user find out on re-import.
    pub fn carries_metadata(self) -> bool {
        matches!(self, Codec::Wav16 | Codec::Wav24)
    }

    /// Whether this codec throws information away. Both compressed codecs do.
    pub fn is_lossy(self) -> bool {
        matches!(self, Codec::OggVorbis | Codec::Opus)
    }

    /// Whether the **Quality** slider drives this codec. Vorbis is VBR and takes a quality
    /// scalar; Opus is driven by a **bitrate** instead, and the panel maps the same slider onto
    /// it — so the control is live for both, and this is about which knob it becomes.
    pub fn uses_quality_scalar(self) -> bool {
        matches!(self, Codec::OggVorbis)
    }
}

/// At most this many seconds are actually encoded to size a **lossy** asset; longer clips are
/// scaled from that. A lossy codec's size is not a formula — the *only* honest way to know it is
/// to encode it — but encoding a five-minute stem to draw a label would stall the UI.
///
/// A game SFX is short, so the common case is measured whole and exact; music (which is where the
/// long clips are) is the case where an estimate is fine.
///
/// **It applies to every lossy codec, and that is not obvious enough to leave implicit.** Opus was
/// exempted from it for a year on the reasoning that it was "bitrate-driven and fast, so the honest
/// number is also the cheap one". Measured head-to-head on the same buffer with the cap off both,
/// Opus is **1.4× the Vorbis time at 2 s and 2.0× at 10 s** — it is the *slower* of the two, and it
/// was the one running uncapped. The exemption was reasoning about libopus in C; what this
/// workspace links is `unsafe-libopus`, the same library **transpiled by c2rust with none of the
/// SIMD** (ADR-0116). "Fast in C" does not survive the trip.
///
/// The cost of that: a 180 s clip priced under Opus cost **941 ms**, on the UI thread, every time
/// the user touched the clip. See ADR-0125.
const MEASURE_SECS: f64 = 10.0;

/// What one asset costs, in bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeliveryCost {
    /// Size of the encoded file.
    pub disk_bytes: usize,
    /// What the engine holds once the clip is loaded and decoded: interleaved `f32`,
    /// the whole thing. **Codec-independent** — see the module docs.
    pub ram_bytes: usize,
    /// `ram_bytes` as a fraction of the audio subsystem's RAM budget (HR-13, 30 MB).
    pub ram_budget_frac: f32,
    /// Whether `disk_bytes` was measured in full, or extrapolated from the first
    /// [`MEASURE_SECS`] (only ever false for a long lossy clip).
    pub disk_exact: bool,
}

/// The RAM a decoded clip occupies in the mixer: interleaved `f32`, one allocation.
pub fn ram_bytes(data: &SampleData) -> usize {
    std::mem::size_of_val(data.samples())
}

/// Size `data` as it would ship under `codec`, and as it would sit in memory once
/// loaded.
///
/// The disk figure is the **real encoded length**, not a bitrate guess: the same
/// writers that produce the file produce the number.
pub fn cost(
    data: &SampleData,
    codec: Codec,
    ogg_quality: f32,
) -> Result<DeliveryCost, EncodeError> {
    let ram = ram_bytes(data);
    let budget_bytes = ph2d_audio::budget().ram_mb as f64 * 1_024.0 * 1_024.0;

    let (disk_bytes, disk_exact) = match codec {
        // A PCM size is arithmetic — the writer is a memcpy with a header, and it is already
        // O(clip) at memory speed. There is nothing to cap and nothing to extrapolate.
        Codec::Wav16 => (encode_wav(data, BitDepth::Pcm16)?.len(), true),
        Codec::Wav24 => (encode_wav(data, BitDepth::Pcm24)?.len(), true),
        // **Both lossy codecs go through the same cap**, through the same function, so the two
        // cannot drift on what a long clip costs to size. They did: Opus was exempt, and the
        // exemption cost 941 ms of UI thread on a 3-minute clip (see `MEASURE_SECS`).
        Codec::OggVorbis => measure_capped(data, |d| encode_ogg(d, ogg_quality))?,
        Codec::Opus => measure_capped(data, |d| encode_opus(d, ogg_quality))?,
    };

    Ok(DeliveryCost {
        disk_bytes,
        ram_bytes: ram,
        ram_budget_frac: if budget_bytes > 0.0 {
            (ram as f64 / budget_bytes) as f32
        } else {
            0.0
        },
        disk_exact,
    })
}

/// Encode `data` as Opus. The panel's 0..1 quality slider becomes a **bitrate** — Opus is not
/// driven by a quality scalar the way Vorbis is, so the same control means a different thing,
/// and pretending otherwise would put a knob on screen that does not do what it says.
pub fn encode_opus(data: &SampleData, quality: f32) -> Result<Vec<u8>, EncodeError> {
    let bitrate = opus_bitrate(quality);
    ph2d_audio_opus::encode_opus(data, bitrate).map_err(|e| EncodeError::Codec(e.to_string()))
}

/// The quality slider (0..1) as an Opus bitrate. 32 kbps is thin-but-usable for a game SFX;
/// 256 kbps is past transparent for anything this editor will hold. The default slider position
/// lands near 96 kbps, which is where the crate's own gates measured 33.8 dB SNR.
pub fn opus_bitrate(quality: f32) -> i32 {
    const MIN: f32 = 32_000.0;
    const MAX: f32 = 256_000.0;
    (MIN + (MAX - MIN) * quality.clamp(0.0, 1.0)) as i32
}

/// Encode (up to [`MEASURE_SECS`] of) the clip with `encode`, and report the size — plus whether
/// that size is the whole truth or a scaled figure.
///
/// **One function for both lossy codecs**, because "how is a long clip sized?" is one question. It
/// was two, and they disagreed: Vorbis capped, Opus did not, and nothing on screen said which of
/// the two numbers you were reading. A second copy of a policy is a second place for it to rot.
///
/// The returned `bool` is [`DeliveryCost::disk_exact`]: false means the caller must not present
/// this as a measurement. The panel prints it with a `~`.
fn measure_capped(
    data: &SampleData,
    encode: impl Fn(&SampleData) -> Result<Vec<u8>, EncodeError>,
) -> Result<(usize, bool), EncodeError> {
    let sr = data.format().sample_rate as f64;
    let total = data.frame_count();
    let cap = (MEASURE_SECS * sr) as usize;
    if total <= cap {
        return Ok((encode(data)?.len(), true));
    }
    // Long clip: encode the head and scale. The container's header is a fixed cost that
    // does not repeat, so scaling the whole thing over-counts it — by a few kilobytes on
    // a multi-megabyte file, which is well inside what the "~" in the readout promises.
    let ch = data.format().channel_count();
    let head = SampleData::from_interleaved(data.samples()[..cap * ch].to_vec(), data.format());
    let measured = encode(&head)?.len();
    let scale = total as f64 / cap as f64;
    Ok(((measured as f64 * scale) as usize, false))
}

/// Render a byte count the way a person reads it.
pub fn format_bytes(bytes: usize) -> String {
    const KB: f64 = 1_024.0;
    const MB: f64 = KB * 1_024.0;
    let b = bytes as f64;
    if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::AudioFormat;

    /// Stereo tone, `secs` long — busy enough that Vorbis cannot encode it to nothing.
    ///
    /// The phase is **wrapped into one second** rather than grown from `t`: at 300 s a
    /// raw `sin(TAU * 220 * t)` in `f32` is evaluating a sine at ~400_000 radians, where
    /// the mantissa has run out and the "tone" quietly degrades into NOISE — which costs
    /// more bits, so the fixture, not the codec, was making the bitrate climb with
    /// duration (it looked exactly like a non-linear encoder). Real audio is stationary;
    /// the fixture has to be too.
    fn clip(secs: f32) -> SampleData {
        let n = (48_000.0 * secs) as usize;
        let tau = std::f64::consts::TAU;
        let s: Vec<f32> = (0..n)
            .flat_map(|i| {
                let t = (i % 48_000) as f64 / 48_000.0;
                let v = (0.5 * (tau * 220.0 * t).sin() + 0.2 * (tau * 3_000.0 * t).sin()) as f32;
                [v, v * 0.8]
            })
            .collect();
        SampleData::from_interleaved(s, AudioFormat::stereo(48_000))
    }

    /// THE point of the readout, pinned: the codec decides what the player DOWNLOADS and
    /// has **no say** in what the engine spends. There is no streaming path in the
    /// mixer, so a Vorbis asset is decoded to the very same `f32` buffer a WAV would be.
    ///
    /// Red if someone wires RAM to the encoded size — which is the intuitive-but-wrong
    /// thing, and would tell a user that shipping Vorbis saves them memory. It does not.
    #[test]
    fn the_codec_moves_disk_and_never_ram() {
        let d = clip(1.0);
        let wav = cost(&d, Codec::Wav16, 0.5).unwrap();
        let ogg = cost(&d, Codec::OggVorbis, 0.5).unwrap();

        assert!(
            ogg.disk_bytes * 3 < wav.disk_bytes,
            "Vorbis must be far smaller on disk: {} vs {}",
            ogg.disk_bytes,
            wav.disk_bytes
        );
        assert_eq!(
            ogg.ram_bytes, wav.ram_bytes,
            "the codec must not change what the engine holds"
        );
    }

    /// RAM is exactly the decoded buffer: interleaved `f32`, nothing else.
    #[test]
    fn ram_is_the_decoded_buffer() {
        let d = clip(1.0);
        // 1 s, 48 kHz, 2 ch, 4 bytes.
        assert_eq!(ram_bytes(&d), 48_000 * 2 * 4);
        assert_eq!(
            cost(&d, Codec::Wav16, 0.5).unwrap().ram_bytes,
            48_000 * 2 * 4
        );
    }

    /// ...and it is measured against the subsystem's real budget, so the number means
    /// something. 384 KB of a 30 MB envelope is ~1.25%.
    #[test]
    fn ram_is_scored_against_the_audio_budget() {
        let c = cost(&clip(1.0), Codec::Wav16, 0.5).unwrap();
        let expect = (48_000.0 * 2.0 * 4.0) / (30.0 * 1_024.0 * 1_024.0);
        assert!(
            (c.ram_budget_frac - expect as f32).abs() < 1e-6,
            "budget fraction was {}",
            c.ram_budget_frac
        );
    }

    /// A 24-bit WAV is half again as big as a 16-bit one — the lossless trade, priced.
    #[test]
    fn bit_depth_prices_the_lossless_trade() {
        let d = clip(1.0);
        let a = cost(&d, Codec::Wav16, 0.5).unwrap().disk_bytes;
        let b = cost(&d, Codec::Wav24, 0.5).unwrap().disk_bytes;
        let ratio = b as f64 / a as f64;
        assert!((ratio - 1.5).abs() < 0.01, "24/16 came out at {ratio}");
    }

    /// A clip inside the measuring window is sized by really encoding it — no guess, no
    /// "~" in the readout.
    #[test]
    fn a_short_lossy_clip_is_measured_exactly() {
        let c = cost(&clip(1.0), Codec::OggVorbis, 0.5).unwrap();
        assert!(c.disk_exact);
        assert_eq!(c.disk_bytes, encode_ogg(&clip(1.0), 0.5).unwrap().len());
    }

    /// A long one is extrapolated from the head — flagged, and in the right ballpark.
    #[test]
    fn a_long_lossy_clip_is_extrapolated_and_says_so() {
        let d = clip(25.0); // past MEASURE_SECS
        let c = cost(&d, Codec::OggVorbis, 0.5).unwrap();
        assert!(!c.disk_exact, "a scaled figure must admit it is one");
        let truth = encode_ogg(&d, 0.5).unwrap().len();
        let err = (c.disk_bytes as f64 - truth as f64).abs() / truth as f64;
        assert!(
            err < 0.25,
            "the estimate was {err:.0e} off: {c:?} vs {truth}"
        );
    }

    /// Higher quality costs more bytes — the slider has to actually mean something.
    #[test]
    fn quality_moves_the_size() {
        let d = clip(2.0);
        let lo = cost(&d, Codec::OggVorbis, 0.1).unwrap().disk_bytes;
        let hi = cost(&d, Codec::OggVorbis, 0.9).unwrap().disk_bytes;
        assert!(hi > lo, "quality 0.9 ({hi}) must cost more than 0.1 ({lo})");
    }

    /// **Sizing a long clip is O(the cap), not O(the clip) — for EVERY lossy codec.**
    ///
    /// This is the gate the Opus exemption walked through. It is a **ratio**, not a wall-clock
    /// bar: `ci-test` builds at `opt-level=1`, so an absolute millisecond figure here would be
    /// measuring the build profile rather than the algorithm. Four times the clip must not cost
    /// four times the sizing — the cap means it costs the same.
    ///
    /// Mutation-tested: exempt either codec from `measure_capped` (the exact shape of the bug)
    /// and its ratio goes to ~4×, far past the bar.
    #[test]
    fn sizing_a_long_lossy_clip_is_capped_for_every_lossy_codec() {
        for codec in [Codec::OggVorbis, Codec::Opus] {
            // Both are past MEASURE_SECS, so both are sized from the same 10 s head.
            let short = clip(15.0);
            let long = clip(60.0);
            let t = std::time::Instant::now();
            cost(&short, codec, 0.5).unwrap();
            let short_ms = t.elapsed().as_secs_f64();
            let t = std::time::Instant::now();
            cost(&long, codec, 0.5).unwrap();
            let long_ms = t.elapsed().as_secs_f64();

            let ratio = long_ms / short_ms.max(1e-9);
            println!("{codec:?}: 4x the clip cost {ratio:.2}x the sizing");
            assert!(
                ratio < 2.0,
                "{codec:?}: quadrupling the clip cost {ratio:.2}x the sizing -- it is being \
                 encoded WHOLE to draw a label. That is the bug that cost 941 ms of UI thread \
                 on a 3-minute clip (ADR-0125); the cap exists so the figure is O(MEASURE_SECS)."
            );
        }
    }

    /// ...and the capped figure still has to be **roughly true**, or the gate above would be
    /// satisfied by returning a constant.
    ///
    /// The presence sibling of the cap: fast *and wrong* is not the trade. Opus is the codec that
    /// was uncapped, so it is the one whose estimate had never been checked against its own truth.
    #[test]
    fn the_capped_opus_figure_is_close_to_the_real_one_and_admits_it_is_an_estimate() {
        let d = clip(30.0); // past MEASURE_SECS -> extrapolated
        let c = cost(&d, Codec::Opus, 0.5).unwrap();
        assert!(
            !c.disk_exact,
            "a scaled figure must admit it is one -- the panel prints the '~' from this flag, and \
             an estimate presented as a measurement is worse than a slow measurement"
        );
        let truth = encode_opus(&d, 0.5).unwrap().len();
        let err = (c.disk_bytes as f64 - truth as f64).abs() / truth as f64;
        println!("capped Opus estimate is {:.1}% off", err * 100.0);
        assert!(
            err < 0.25,
            "the Opus estimate was {err:.0e} off: {} vs {truth}",
            c.disk_bytes
        );
    }

    /// **A clip inside the window is still measured for real, under both lossy codecs.**
    ///
    /// The absence gate above says "do not encode the whole thing". On its own it goes green on a
    /// readout that never encodes anything — so this is the half that says the number is real when
    /// it claims to be: `disk_exact` is only ever true when the bytes were actually counted.
    #[test]
    fn a_short_clip_is_still_measured_exactly_under_both_lossy_codecs() {
        let d = clip(1.0);
        let ogg = cost(&d, Codec::OggVorbis, 0.5).unwrap();
        assert!(ogg.disk_exact);
        assert_eq!(ogg.disk_bytes, encode_ogg(&d, 0.5).unwrap().len());

        let opus = cost(&d, Codec::Opus, 0.5).unwrap();
        assert!(opus.disk_exact);
        assert_eq!(opus.disk_bytes, encode_opus(&d, 0.5).unwrap().len());
    }

    /// Only the WAV writer has chunks for loop points and markers; the panel warns on
    /// the others rather than letting the user discover it on re-import.
    #[test]
    fn only_wav_carries_the_loop_metadata() {
        assert!(Codec::Wav16.carries_metadata());
        assert!(Codec::Wav24.carries_metadata());
        assert!(!Codec::OggVorbis.carries_metadata());
    }

    #[test]
    fn bytes_read_like_a_person_wrote_them() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2_048), "2 KB");
        assert_eq!(format_bytes(3_145_728), "3.0 MB");
    }
}
