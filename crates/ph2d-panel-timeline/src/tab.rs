//! The panel's two views — and the reason they are two.
//!
//! # A ruler measures one clock
//!
//! A key is stamped in its **clip's** time. A strip sits in the **timeline's**.
//! Drawn against one ruler they share a pixel column that means two different
//! instants: select `Right` in the clip dropdown and its keys draw at 0..3 while
//! its strip sits at 3..6, with nothing on screen admitting it. Without a stack
//! the two clocks are the same one, which is exactly why this went unseen — the
//! feature that split them is the feature nobody had used yet.
//!
//! So the halves get a tab each, and **the tab decides what the ruler means**
//! ([`ph2d_timeline::TimelineViewSnapshot::clip_time`]).
//!
//! # This is the gold standard, not a preference
//!
//! Unity will not let you edit a keyframe in the Timeline window at all — it
//! sends you to the Animation window. Blender's NLA Editor and Dope Sheet are
//! separate editors. Premiere puts keys in Effect Controls, beside the timeline.
//! After Effects mixes them in one list but keeps each level of nesting in its
//! own **tab**. The one that mixes both freely — Unreal's Sequencer — has users
//! asking it to stop (*"scrolling through endless expanded tracks of keyframed
//! data can be fatiguing"*). Enio read the same thing off the screen before any
//! of this was looked up: *"a timeline com keys e com strips misturadas é
//! confusa"*.
//!
//! # A tab is a view, not a mode (ADR-0115 R8, amended)
//!
//! R8 rejected Blender's **tweak mode**, and it was right to: tweak mode exists
//! because Blender's editors can only hold one Action at a time, and our clip
//! dropdown already answers that. A tab changes *what you are looking at and what
//! the ruler measures*; it never changes what an edit means, and the dropdown
//! still picks the clip. What R8 missed is narrower and load-bearing: with no
//! stack there is one clock, so "one view" cost nothing — the moment a stack
//! exists there are two, and one ruler cannot serve both.

/// Which half of the document the panel is showing.
///
/// **Default is [`Tab::Keys`]** — a document with no stack has nothing to arrange,
/// and Keys is the panel it has always been.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    /// The active clip's dope sheet + graph editor, ruled by the CLIP's clock.
    #[default]
    Keys,
    /// The clip stack — lanes of strips, ruled by the TIMELINE's clock.
    Arrange,
}

impl Tab {
    /// Whether the Summary channel and the track rows have any height here.
    #[must_use]
    pub fn shows_keys(self) -> bool {
        matches!(self, Self::Keys)
    }

    /// Whether the clip stack's lanes have any height here.
    #[must_use]
    pub fn shows_lanes(self) -> bool {
        matches!(self, Self::Arrange)
    }

    /// The tab's index in the strip — and the inverse of [`Self::from_index`].
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Keys => 0,
            Self::Arrange => 1,
        }
    }

    /// The tab at `i` in the strip, or [`Tab::Keys`] for anything else.
    #[must_use]
    pub fn from_index(i: usize) -> Self {
        if i == 1 { Self::Arrange } else { Self::Keys }
    }
}

/// The two tabs, in strip order: `(id, i18n key)`.
///
/// The ONE list. The strip paints from it, the hit registration walks it, and the
/// event router matches against it — a second list written by hand would drift
/// from the one on screen, which is how a painted control ends up dispatching
/// nothing ([[feedback_widget_is_done_when_a_test_clicks_it]]).
pub const TABS: [(ph2d_a11y::NodeId, &str); 2] = [
    (crate::ids::TIMELINE_TAB_KEYS, "panel.timeline.tab.keys"),
    (
        crate::ids::TIMELINE_TAB_ARRANGE,
        "panel.timeline.tab.arrange",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_panel_opens_on_the_keys_it_has_always_shown() {
        assert_eq!(Tab::default(), Tab::Keys);
    }

    /// Each tab shows exactly one half — never both (which is the bug), never
    /// neither (which is a blank panel).
    #[test]
    fn every_tab_shows_exactly_one_half() {
        for tab in [Tab::Keys, Tab::Arrange] {
            assert_ne!(
                tab.shows_keys(),
                tab.shows_lanes(),
                "{tab:?} shows both halves or neither"
            );
        }
    }

    #[test]
    fn the_index_round_trips_through_the_strip() {
        for tab in [Tab::Keys, Tab::Arrange] {
            assert_eq!(Tab::from_index(tab.index()), tab);
        }
        assert_eq!(
            TABS.len(),
            2,
            "the strip and the enum must stay the same size"
        );
        assert_eq!(
            Tab::from_index(TABS.len()),
            Tab::Keys,
            "out of range is home"
        );
    }

    /// The strip's order IS the enum's order: `TABS[i]` must be the tab whose
    /// `index()` is `i`, or clicking "Arrange" would select Keys.
    #[test]
    fn the_strips_order_is_the_enums_order() {
        assert_eq!(TABS[Tab::Keys.index()].0, crate::ids::TIMELINE_TAB_KEYS);
        assert_eq!(
            TABS[Tab::Arrange.index()].0,
            crate::ids::TIMELINE_TAB_ARRANGE
        );
    }
}
