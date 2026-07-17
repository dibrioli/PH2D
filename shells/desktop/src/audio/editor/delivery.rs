//! The Delivery bridge (W6 asset-prep): price the loaded clip, and export it under the
//! codec the user picked.
//!
//! The panel cannot do any of this — it has no encoder and no view of the mixer — so it
//! owns the two knobs (codec, quality) and the shell owns every number.
//!
//! **Sizing is measuring, not guessing:** the same writers that produce the file produce
//! the figure ([`ph2d_audio_encode::cost`]). That means an encode — 561 ms of one on a 3-minute
//! clip under Opus — so **it does not happen on this thread**: see [`super::pricing`] (ADR-0125).
//!
//! **RAM is not part of that trade.** It is `size_of_val` on a slice; it costs nothing and it is
//! always exact. Only the *disk* figure needs an encoder, so only the disk figure is ever `…` —
//! the half of the readout that can be honest for free stays honest for free.
//!
//! Unlike the shipping targets below it, this readout has **no visibility gate**: the download size
//! is painted on the section's own header (`delivery_readout`), which is on screen whether the
//! section is folded or not. A gate on "is the section open" would blank a number the user can see.

use ph2d_audio_encode::{Codec, format_bytes, ram_bytes};

/// The measured size of the encoded file, and whether that size is the whole truth (see
/// `DeliveryCost::disk_exact` — a capped figure is an estimate and must print with a `~`).
type Disk = (usize, bool);

/// The priced download size, and the machinery that keeps the encode off this thread.
pub(crate) type DeliveryCache = super::pricing::OffThread<CostKey, Disk>;

/// Everything the **disk** figure depends on. The buffer is identified by `SampleData::version`,
/// never by its address: since ADR-0124 an edit can rewrite a clip **in place**, so the address of
/// the edited clip is the address of the old one — and this panel would price audio the user has
/// changed.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) struct CostKey {
    buf: ph2d_audio::BufferVersion,
    codec: usize,
    /// Quality, quantised: a slider drag must not re-encode on every pixel of travel.
    quality_step: u8,
}

/// Vorbis quality steps the readout distinguishes. 20 is finer than the ear or the
/// filesystem cares about, and coarse enough that dragging the slider does not encode
/// the clip a hundred times.
const QUALITY_STEPS: f32 = 20.0;

impl super::super::AudioSystem {
    /// Price what is SOUNDING (the audition / the mono view — the same buffer Export
    /// writes and the waveform shows) and publish the readout to the panel.
    ///
    /// Called once per frame from the bridge; it is a cache hit on all but the frame
    /// after something actually changed.
    pub(crate) fn editor_publish_delivery(&mut self) {
        use ph2d_panel_audio_editor::delivery_state as ds;

        let codec_idx = ds::codec().min(Codec::ALL.len() - 1);
        let codec = Codec::ALL[codec_idx];
        ds::set_codec_info(
            Codec::ALL.len(),
            codec.name(),
            codec.is_lossy(),
            if codec.uses_quality_scalar() {
                "Quality"
            } else if codec.is_lossy() {
                "Bitrate"
            } else {
                "Quality"
            },
        );

        let Some(clip) = self.editor_sounding() else {
            ds::set_cost("", "", 0.0, false);
            self.delivery.clear();
            return;
        };
        let quality = ds::quality();
        // Everything that needs the clip is taken HERE, in one place, so the borrow of `self` ends
        // before the cache below is touched. `data` is an `Arc<[f32]>`: this clones a refcount, not
        // a clip (see `platforms`). The mixer may be sounding this very buffer — it is immutable,
        // which is exactly why it is safe to hand to a worker.
        let data = clip.data().clone();
        let key = CostKey {
            buf: data.version(),
            codec: codec_idx,
            quality_step: (quality * QUALITY_STEPS) as u8,
        };
        // RAM is free (`size_of_val`) and needs no encoder — taken on this thread, on this frame.
        let ram_b = ram_bytes(&data);

        let owned = data;
        let sized = self
            .delivery
            .current(key, "Pricing the export", move || {
                ph2d_audio_encode::cost(&owned, codec, quality)
                    .map(|c| (c.disk_bytes, c.disk_exact))
                    .unwrap_or((0, false))
            })
            .copied();

        // A scaled figure says so, rather than passing an estimate off as a measurement — and a
        // figure that is not known yet says THAT, rather than leaving the previous clip's number on
        // screen wearing this clip's name.
        let disk = match sized {
            Some((bytes, true)) => format_bytes(bytes),
            Some((bytes, false)) => format!("~{}", format_bytes(bytes)),
            None => "\u{2026}".to_string(),
        };
        // RAM is exact on the very frame of the edit — it never waits on a worker and never shows
        // `…`. It carries its share of the budget with it, or the number means nothing.
        let budget = ph2d_audio::budget().ram_mb as f64 * 1_024.0 * 1_024.0;
        let ram_budget_frac = if budget > 0.0 {
            (ram_b as f64 / budget) as f32
        } else {
            0.0
        };
        let ram = format!(
            "RAM {} \u{b7} {:.0}% of budget",
            format_bytes(ram_b),
            ram_budget_frac * 100.0
        );
        // Warn only when there is something to lose: a clip with no loop and no markers
        // loses nothing by shipping as Vorbis.
        let has_meta = self
            .editor
            .clip
            .as_ref()
            .is_some_and(|c| c.loop_region().is_some() || !c.markers().is_empty());
        ds::set_cost(
            &disk,
            &ram,
            ram_budget_frac,
            has_meta && !codec.carries_metadata(),
        );
    }

    /// The codec the export must use, and the extension the file dialog should offer.
    pub(crate) fn editor_codec(&self) -> Codec {
        let i = ph2d_panel_audio_editor::delivery_state::codec();
        Codec::ALL[i.min(Codec::ALL.len() - 1)]
    }

    /// Write the clip out under the selected codec. One Export button, one codec, one
    /// place the decision lives — there is no second path that could disagree with the
    /// price the panel just showed.
    pub(crate) fn editor_export_codec(&self, path: &std::path::Path) {
        match self.editor_codec() {
            Codec::Wav16 => self.editor_export(path, ph2d_audio_encode::BitDepth::Pcm16),
            Codec::Wav24 => self.editor_export(path, ph2d_audio_encode::BitDepth::Pcm24),
            Codec::OggVorbis => {
                let q = ph2d_panel_audio_editor::delivery_state::quality();
                self.editor_export_ogg(path, q);
            }
            Codec::Opus => {
                let q = ph2d_panel_audio_editor::delivery_state::quality();
                self.editor_export_opus(path, q);
            }
        }
    }
}
