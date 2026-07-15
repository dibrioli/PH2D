//! **One door for every audio file the app opens.**
//!
//! Symphonia reads what Symphonia reads — WAV, FLAC, Ogg Vorbis, MP3, AIFF. It does **not** read
//! Opus (its `all-codecs` is aac/adpcm/alac/flac/mp1-3/pcm/vorbis), and as of ADR-0116 this app
//! *writes* Opus. Which means that without this module, the editor would export a format it
//! could not re-open: export a variation as `.opus`, and the tool that made it cannot load it
//! back. That is not a codec gap, it is a broken promise.
//!
//! So every decode in the shell comes through here, and here decides. There were five separate
//! call sites reaching for `ph2d_audio_decode::decode` — the clip Load, the variation Add, the
//! variation-set manifest, the impulse-response Load, and the batch LUFS pass. Five places to
//! forget, so: one door, and the routing lives behind it.
//!
//! The routing is by **content**, not by extension: a `.opus` file that someone renamed is still
//! an Opus file, and an extension is a hint, not a fact.

use ph2d_audio::SampleData;

/// Every extension the app can OPEN — the **single list** every import picker and folder scan
/// shares, so adding a format is one edit and not five.
///
/// `opus` lives here because the app both *writes* it (ADR-0116: the Opus export, the Desktop
/// shipping target) and *decodes* it just below — yet it was missing from all five pickers, so an
/// `.opus` the tool exported could be decoded but never SELECTED. That is precisely the broken
/// promise this module exists to close, left half-open: the door decoded, the picker hid the file.
pub(crate) const AUDIO_IMPORT_EXTS: &[&str] = &["wav", "flac", "ogg", "mp3", "opus", "aiff", "aif"];

/// Decode any audio file the app supports, routing Opus to the crate that can read it.
pub(crate) fn decode(bytes: &[u8]) -> Result<SampleData, String> {
    if ph2d_audio_opus::is_opus(bytes) {
        return ph2d_audio_opus::decode_opus(bytes).map_err(|e| e.to_string());
    }
    ph2d_audio_decode::decode(bytes).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};

    fn clip() -> SampleData {
        let s: Vec<f32> = (0..24_000)
            .map(|i| (std::f32::consts::TAU * 220.0 * (i as f32 / 48_000.0)).sin() * 0.4)
            .collect();
        SampleData::from_interleaved(
            s,
            AudioFormat {
                sample_rate: 48_000,
                channels: ChannelLayout::Mono,
            },
        )
    }

    /// **What the app writes, the app reads.** The round trip that the five call sites depend on,
    /// through the one door they now share. Without the Opus branch this is the failure the user
    /// would hit: an export they cannot open.
    #[test]
    fn an_opus_file_this_app_wrote_loads_back() {
        let bytes = ph2d_audio_encode::encode_opus(&clip(), 0.5).expect("encode");
        let back = super::decode(&bytes).expect("the app must read what it writes");
        assert_eq!(back.format().channel_count(), 1);
        assert!(back.frame_count() > 20_000, "the clip came back empty");
    }

    /// And WAV still goes where it always went — the routing added a branch, it did not move
    /// anything.
    #[test]
    fn a_wav_still_decodes() {
        let bytes = ph2d_audio_encode::encode_wav(&clip(), ph2d_audio_encode::BitDepth::Pcm16)
            .expect("wav");
        let back = super::decode(&bytes).expect("wav must still decode");
        assert_eq!(back.frame_count(), 24_000);
    }

    /// The routing reads the FILE, not its name: a renamed `.opus` is still an Opus file, and an
    /// extension is a hint rather than a fact.
    #[test]
    fn the_routing_is_by_content_not_by_extension() {
        let bytes = ph2d_audio_encode::encode_opus(&clip(), 0.5).expect("encode");
        assert!(ph2d_audio_opus::is_opus(&bytes));
        let wav = ph2d_audio_encode::encode_wav(&clip(), ph2d_audio_encode::BitDepth::Pcm16)
            .expect("wav");
        assert!(
            !ph2d_audio_opus::is_opus(&wav),
            "a WAV was taken for an Opus"
        );
    }

    /// **What the app WRITES, a picker must be able to SELECT.** Every export codec's extension has
    /// to be in the shared import list, or the tool exports a file its own Open dialog hides — the
    /// exact gap Opus fell into (decoded, but filtered out of every picker). This gate is why the
    /// list cannot drift away from the encoders again.
    #[test]
    fn every_export_codec_extension_is_importable() {
        for codec in ph2d_audio_encode::Codec::ALL {
            assert!(
                super::AUDIO_IMPORT_EXTS.contains(&codec.extension()),
                "the app exports {} as .{}, but no import picker lists that extension",
                codec.name(),
                codec.extension()
            );
        }
    }
}
