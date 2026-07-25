//! Gates for the onion settings modal's shell half (ADR-0142 W3b): the count↔slider mapping and the
//! store→onion read-back. The drag `impl App` fns are covered by the shell's pointer-dispatch smoke
//! (they need a window); here we gate the pure glue.

use super::{count_to_frac, read_into, rgb_to_u8, u8_to_rgb};
use ph2d_editor::interaction::WidgetStore;
use ph2d_timeline::{OnionMode, OnionSettings};

// The count↔slider mapping (`MAX_GHOSTS`/`count_to_frac`/`frac_to_count`) now lives in editor-core
// next to the modal's painter, gated there (`the_count_mapping_round_trips_every_whole_count`). The
// shell only re-exports it; the read-back gate below exercises the re-export end to end.

/// The colour mapping round-trips (alpha is dropped — a ghost's alpha is the Opacity slider).
#[test]
fn colour_mapping_round_trips() {
    let rgb = [0.145, 0.420, 0.137];
    let back = u8_to_rgb(rgb_to_u8(rgb));
    for i in 0..3 {
        assert!(
            (rgb[i] - back[i]).abs() < 1.0 / 255.0,
            "channel {i} within a step"
        );
    }
    assert_eq!(rgb_to_u8([0.0, 1.0, 0.0]), [0, 255, 0, 255]);
}

/// The read-back writes the modal's slider/swatch values into the onion — the live-edit path. This is
/// the gate the whole modal exists for: a mutation making `read_into` a no-op (or dropping the count
/// mapping) leaves the onion at its defaults, which this catches.
#[test]
fn read_back_writes_the_onion_while_open() {
    let mut store = WidgetStore::default();
    // Open at (0,0) seeding: opacity 0.25 · before 4/8 · after 2/8 · two distinct colours.
    store.open_onion_modal(
        0.0,
        0.0,
        0.25,
        count_to_frac(4),
        count_to_frac(2),
        [10, 100, 10, 255],
        [30, 20, 130, 255],
    );
    // `enabled`/`mode` are owned by the transport toggle, not the modal — they must survive.
    let mut onion = OnionSettings {
        enabled: true,
        mode: OnionMode::Frames,
        ..Default::default()
    };
    read_into(&store, &mut onion);

    assert!((onion.opacity - 0.25).abs() < 1e-6, "opacity read back");
    assert_eq!(onion.frames_before, 4, "before count read back");
    assert_eq!(onion.frames_after, 2, "after count read back");
    assert_eq!(onion.color_before, u8_to_rgb([10, 100, 10, 255]));
    assert_eq!(onion.color_after, u8_to_rgb([30, 20, 130, 255]));
    // The transport toggles' fields are untouched.
    assert!(onion.enabled, "enabled stays owned by the toggle");
    assert_eq!(
        onion.mode,
        OnionMode::Frames,
        "mode stays owned by the toggle"
    );
}

/// Closed → read-back is a no-op (the onion keeps whatever the toggles set).
#[test]
fn read_back_is_a_no_op_when_closed() {
    let store = WidgetStore::default(); // never opened
    let mut onion = OnionSettings {
        opacity: 0.9,
        frames_before: 7,
        ..Default::default()
    };
    read_into(&store, &mut onion);
    assert!(
        (onion.opacity - 0.9).abs() < 1e-6,
        "closed modal writes nothing"
    );
    assert_eq!(onion.frames_before, 7);
}
