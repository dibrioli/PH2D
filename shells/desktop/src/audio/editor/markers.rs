//! Marker (cue point) runtime (docs/Audio/, W6 asset-prep). A **descendant submodule**
//! of `audio::editor`, so `impl AudioSystem` here reaches the private editor fields —
//! same trick as `loops`/`fx_rack`. Named timeline points a game runtime can react to,
//! exported to the WAV `cue`+`adtl` chunks.

use super::AudioSystem;

/// Window (fraction of the sample rate) for "delete nearest" — ~50 ms, generous enough
/// to grab a marker the playhead is parked near.
const MARKER_DEL_WINDOW_DIV: usize = 20;

impl AudioSystem {
    /// Add a cue marker at the current playhead, auto-named `M{n}`.
    pub(crate) fn editor_add_marker(&mut self) {
        let frame = self.editor_playhead_frame() as usize;
        if let Some(clip) = self.editor.clip.as_mut() {
            let n = clip.markers().len() + 1;
            clip.add_marker(frame, format!("M{n}"));
        }
    }

    /// Delete the marker nearest the playhead (within a window).
    pub(crate) fn editor_del_marker(&mut self) {
        let frame = self.editor_playhead_frame() as usize;
        if let Some(clip) = self.editor.clip.as_mut() {
            let window = (clip.data().format().sample_rate as usize / MARKER_DEL_WINDOW_DIV).max(1);
            clip.remove_marker_near(frame, window);
        }
    }

    /// The markers as `(frame, name)`, for the overlay.
    pub(crate) fn editor_markers(&self) -> Vec<(u64, String)> {
        self.editor
            .clip
            .as_ref()
            .map(|c| {
                c.markers()
                    .iter()
                    .map(|m| (m.frame as u64, m.name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// How many markers exist (published to the panel readout).
    pub(crate) fn editor_marker_count(&self) -> usize {
        self.editor.clip.as_ref().map_or(0, |c| c.markers().len())
    }
}
