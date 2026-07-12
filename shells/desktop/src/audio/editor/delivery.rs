//! The Delivery bridge (W6 asset-prep): price the loaded clip, and export it under the
//! codec the user picked.
//!
//! The panel cannot do any of this — it has no encoder and no view of the mixer — so it
//! owns the two knobs (codec, quality) and the shell owns every number.
//!
//! **Sizing is measuring, not guessing:** the same writers that produce the file produce
//! the figure ([`ph2d_audio_encode::cost`]). That means an encode, so the result is
//! **cached** on everything it depends on — the buffer, the codec, the quality — and
//! recomputed only when one of them moves. Without the cache the panel would re-encode
//! the clip on every frame it is painted.

use ph2d_audio_encode::{Codec, DeliveryCost, format_bytes};

/// The cached price of one (clip, codec, quality) — recomputed only when the key moves.
#[derive(Default)]
pub(crate) struct DeliveryCache {
    key: Option<CostKey>,
    cost: Option<DeliveryCost>,
}

/// Everything the cost depends on. The buffer is identified by its pointer + length:
/// `SampleData` is an immutable `Arc<[f32]>`, so a new buffer is a new pointer, and any
/// edit (or an audition, or force-mono) hands us a different one.
#[derive(PartialEq, Eq, Clone, Copy)]
struct CostKey {
    ptr: usize,
    len: usize,
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
        ds::set_codec_info(Codec::ALL.len(), codec.name(), codec.is_lossy());

        let Some(clip) = self.editor_sounding() else {
            ds::set_cost("", "", 0.0, false);
            self.delivery.key = None;
            self.delivery.cost = None;
            return;
        };
        let quality = ds::quality();
        let data = clip.data();
        let key = CostKey {
            ptr: data.samples().as_ptr() as usize,
            len: data.samples().len(),
            codec: codec_idx,
            quality_step: (quality * QUALITY_STEPS) as u8,
        };
        if self.delivery.key != Some(key) {
            self.delivery.cost = ph2d_audio_encode::cost(data, codec, quality).ok();
            self.delivery.key = Some(key);
        }
        let Some(cost) = self.delivery.cost else {
            ds::set_cost("", "", 0.0, false);
            return;
        };

        // A scaled figure says so, rather than passing an estimate off as a measurement.
        let disk = if cost.disk_exact {
            format_bytes(cost.disk_bytes)
        } else {
            format!("~{}", format_bytes(cost.disk_bytes))
        };
        // RAM carries its share of the budget with it, or the number means nothing.
        let ram = format!(
            "RAM {} \u{b7} {:.0}% of budget",
            format_bytes(cost.ram_bytes),
            cost.ram_budget_frac * 100.0
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
            cost.ram_budget_frac,
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
        }
    }
}
