//! **Shipping targets** (W6) — price every platform, and write the whole set in one click.
//!
//! The table of targets lives in [`ph2d_audio_encode::platform`] (and so does the argument for
//! why a target is a *format* and not just a codec). This is the bridge: conform the clip into
//! each target's format, price the result, publish three finished lines, and — on Export Set —
//! write three files.
//!
//! **Sizing is measuring, not guessing**, exactly as the single-codec readout already is: the
//! same writers that produce the file produce the figure. Which means Export Set's preview costs
//! three real encodes, so it is **cached on the buffer** — without that the panel would conform
//! and re-encode the clip three times on every frame it is painted.

use ph2d_audio::SampleData;
use ph2d_audio_encode::{Codec, PLATFORMS, Platform, WavMeta, format_bytes};

/// The cached price of the whole set, for one buffer.
#[derive(Default)]
pub(crate) struct PlatformCache {
    /// `SampleData` is an immutable `Arc<[f32]>`, so a new buffer is a new pointer — and any
    /// edit, audition or force-mono hands us a different one. Pointer + length identifies it.
    key: Option<(usize, usize)>,
    rows: Vec<(String, String, f32)>,
}

/// Conform the clip into what this platform actually ships, and price THAT.
///
/// The conform is the whole point: it is the only step that moves RAM (a codec never does), so
/// pricing the unconformed master under a different codec would print the same memory figure for
/// every target — which is the lie `ph2d_audio_encode::platform` documents at length.
fn price(data: &SampleData, p: &Platform) -> Option<(String, String, f32)> {
    let conformed = ph2d_audio_edit::conform(data, p.format());
    let cost = ph2d_audio_encode::cost(&conformed, p.codec, p.quality).ok()?;
    let disk = if cost.disk_exact {
        format_bytes(cost.disk_bytes)
    } else {
        format!("~{}", format_bytes(cost.disk_bytes))
    };
    Some((
        p.name.to_string(),
        format!(
            "{disk} \u{b7} RAM {} ({:.0}%)",
            format_bytes(cost.ram_bytes),
            cost.ram_budget_frac * 100.0
        ),
        cost.ram_budget_frac,
    ))
}

impl super::super::AudioSystem {
    /// Price every shipping target and publish the rows. A cache hit on all but the frame after
    /// the buffer actually changed.
    pub(crate) fn editor_publish_platforms(&mut self) {
        let Some(clip) = self.editor_sounding() else {
            ph2d_panel_audio_editor::delivery_state::set_platforms(Vec::new());
            self.platforms.key = None;
            self.platforms.rows.clear();
            return;
        };
        let data = clip.data();
        let key = (data.samples().as_ptr() as usize, data.samples().len());
        if self.platforms.key != Some(key) {
            self.platforms.rows = PLATFORMS.iter().filter_map(|p| price(data, p)).collect();
            self.platforms.key = Some(key);
        }
        ph2d_panel_audio_editor::delivery_state::set_platforms(self.platforms.rows.clone());
    }

    /// Write one file per shipping target, each conformed to that target's own format first.
    ///
    /// The name carries the target (`footstep.mobile.ogg`), because three files called
    /// `footstep` in three folders is how a build pipeline ships the wrong one.
    pub(crate) fn editor_export_set(&mut self, dir: &std::path::Path) {
        let Some(clip) = self.editor_sounding() else {
            return;
        };
        let data = clip.data().clone();
        let stem = if self.editor.name.is_empty() {
            "clip".to_string()
        } else {
            // `foo.wav` -> `foo`: the target's own extension is appended below.
            std::path::Path::new(&self.editor.name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("clip")
                .to_string()
        };

        // The loop points and the markers are in FRAMES of the master. A target that conforms the
        // rate moves every frame index, so they have to move with it — see `rescale`.
        let src_rate = data.format().sample_rate;
        let meta = self
            .editor
            .clip
            .as_ref()
            .map(|c| super::meta::wav_meta(c, data.frame_count() as u32))
            .unwrap_or_default();

        let mut written = 0usize;
        for p in PLATFORMS {
            let conformed = ph2d_audio_edit::conform(&data, p.format());
            let path = dir.join(format!(
                "{stem}.{}.{}",
                p.name.to_lowercase(),
                p.codec.extension()
            ));
            let res = match p.codec {
                Codec::Wav16 | Codec::Wav24 => {
                    let depth = if p.codec == Codec::Wav16 {
                        ph2d_audio_encode::BitDepth::Pcm16
                    } else {
                        ph2d_audio_encode::BitDepth::Pcm24
                    };
                    let m = rescale(&meta, src_rate, p.sample_rate);
                    ph2d_audio_encode::write_wav_with_meta(&path, &conformed, depth, &m)
                }
                Codec::OggVorbis => ph2d_audio_encode::write_ogg(&path, &conformed, p.quality),
                // There is no `write_opus`: the Opus writer is encode-then-write, and
                // `EncodeError` absorbs the io error via `#[from]`.
                Codec::Opus => ph2d_audio_encode::encode_opus(&conformed, p.quality)
                    .and_then(|bytes| Ok(std::fs::write(&path, bytes)?)),
            };
            match res {
                Ok(()) => {
                    written += 1;
                    println!("audio: wrote {}", path.display());
                }
                Err(e) => eprintln!("audio: {} failed for {}: {e}", p.name, path.display()),
            }
        }
        println!(
            "audio: export set -- {written}/{} targets written",
            PLATFORMS.len()
        );
    }
}

/// Move loop points and markers onto a **conformed** timeline.
///
/// They are stored as frame indices, and a target that resamples changes what a frame *is*. A
/// 24 kHz variant written with the master's 48 kHz frame numbers would put its loop point at
/// twice the time — a seam in the wrong place, in a shipped asset, that nothing downstream could
/// detect. Today only Console writes WAV and it happens to be at the master's rate, so this is
/// dormant; it is here because "happens to be" is not a design, and the next WAV target will not
/// be so lucky.
fn rescale(meta: &WavMeta, from: u32, to: u32) -> WavMeta {
    if from == to || from == 0 {
        return meta.clone();
    }
    let map = |f: u32| ((f as u64 * to as u64) / from as u64) as u32;
    WavMeta {
        loops: meta
            .loops
            .iter()
            .map(|l| ph2d_audio_encode::LoopRegion {
                start: map(l.start),
                end: map(l.end),
            })
            .collect(),
        markers: meta
            .markers
            .iter()
            .map(|m| ph2d_audio_encode::Marker {
                frame: map(m.frame),
                name: m.name.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_audio::{AudioFormat, ChannelLayout};
    use ph2d_audio_encode::{Codec, ram_bytes};

    fn clip(secs: usize) -> SampleData {
        let frames = secs * 48_000;
        SampleData::from_interleaved(
            (0..frames * 2)
                .map(|i| ((i % 977) as f32 / 977.0) * 0.5 - 0.25)
                .collect(),
            AudioFormat::new(48_000, ChannelLayout::Stereo),
        )
    }

    /// **The point of the whole feature, in one assertion: the targets hold DIFFERENT amounts of
    /// memory.**
    ///
    /// If someone ever "simplifies" the variants back into "the same clip, three codecs", this
    /// goes red — and it is the only thing that would notice, because every other number on the
    /// screen (disk, quality, bitrate) would still look perfectly reasonable. RAM is the one the
    /// codec cannot touch, and RAM is the budget that actually bites (HR-13: 30 MB on an iPad).
    #[test]
    fn the_targets_do_not_all_hold_the_same_memory() {
        let d = clip(10);
        let ram: Vec<usize> = PLATFORMS
            .iter()
            .map(|p| ram_bytes(&ph2d_audio_edit::conform(&d, p.format())))
            .collect();
        let master = ram_bytes(&d);
        println!("RAM per target: {ram:?}  (the master is {master})");

        let mobile = ram[0];
        let desktop = ram[1];
        assert!(
            mobile * 3 < desktop,
            "Mobile holds {mobile} B against Desktop's {desktop} B -- a platform variant that \
             does not conform the AUDIO buys back no memory at all, however it is compressed"
        );
        // ...and Desktop/Console are the master untouched, which is what makes Mobile's number
        // mean something rather than being an artefact of the measurement.
        assert_eq!(
            desktop, master,
            "Desktop is supposed to be the master, unconformed"
        );
    }

    /// **A loop point survives a resample.**
    ///
    /// Loop points are frame indices, and a 24 kHz variant does not have the master's frames. Write
    /// the master's numbers into a resampled file and the loop lands at **twice the time** -- a
    /// seam in the wrong place, in a shipped asset, that nothing downstream would catch. Dormant
    /// today (only Console writes WAV, and at the master's rate); this is the gate that keeps it
    /// dormant instead of latent.
    #[test]
    fn a_loop_point_moves_with_the_rate_it_is_conformed_to() {
        let meta = WavMeta {
            loops: vec![ph2d_audio_encode::LoopRegion {
                start: 48_000,
                end: 96_000,
            }],
            markers: vec![ph2d_audio_encode::Marker {
                frame: 24_000,
                name: "M1".into(),
            }],
        };
        // Halve the rate: every frame index halves, and the loop still starts at 1.0 s.
        let out = rescale(&meta, 48_000, 24_000);
        assert_eq!(out.loops[0].start, 24_000);
        assert_eq!(out.loops[0].end, 48_000);
        assert_eq!(out.markers[0].frame, 12_000);
        // The same rate is a no-op, or every ordinary export would be re-timed for nothing.
        let same = rescale(&meta, 48_000, 48_000);
        assert_eq!(same.loops[0].start, 48_000);
        assert_eq!(same.markers[0].frame, 24_000);
    }

    /// **Every target actually encodes.** A row that priced fine but could not be written would
    /// only be discovered by Enio, in a file dialog, at the end.
    #[test]
    fn every_target_prices_and_names_itself() {
        let d = clip(2);
        for p in PLATFORMS {
            let row = price(&d, &p).unwrap_or_else(|| panic!("{} did not price", p.name));
            assert!(row.1.contains("RAM"), "{}: {}", p.name, row.1);
            assert!(
                !p.codec.extension().is_empty(),
                "{} has no extension",
                p.name
            );
        }
        // The lossless target keeps what the lossy ones drop -- that is why it is on the list.
        let console = PLATFORMS.iter().find(|p| p.name == "Console").unwrap();
        assert_eq!(console.codec, Codec::Wav16);
        assert!(
            console.codec.carries_metadata(),
            "the lossless target must be the one that keeps loop points and markers"
        );
    }
}
