//! Guards for `value.attribute` (doc 50). `super` is the crate root.

use super::*;

fn stream() -> Stream {
    Stream::new(3)
        .with("age", Column::Scalar(vec![0.0, 1.5, 3.0]))
        .with(
            "vel",
            Column::Vec2(vec![[3.0, 4.0], [0.0, 0.0], [-1.0, 0.0]]),
        )
}

/// **Any column the stream carries becomes a value field.** This is the sentence the library
/// could not say before: *colour the sparks by how old they are*.
#[test]
fn a_named_column_becomes_a_value_field() {
    assert_eq!(field(&stream(), "age", 0), vec![0.0, 1.5, 3.0]);
}

/// `Length` mode reads a Vec2 column's magnitude — so `vel` reads as **speed**, which is what an
/// artist asking for speed means (and the 3-4-5 triangle says so).
#[test]
fn length_mode_turns_velocity_into_speed() {
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
}

/// **A column nobody wrote reads as ZERO, at full length** — not as an error, and above all not
/// as an EMPTY field.
///
/// An empty field would be broadcast downstream as a single global zero (that is what a length-1
/// value means in this library), which looks exactly like a working graph producing black. A
/// typo in an attribute name must not be indistinguishable from a correct graph.
#[test]
fn a_missing_column_reads_as_zeros_at_full_length() {
    assert_eq!(field(&stream(), "ag", 0), vec![0.0; 3], "a typo: zeros");
    assert_eq!(field(&stream(), "", 0), vec![0.0; 3], "…and so is nothing");
    // The shape is preserved: three elements in, three values out.
    assert_eq!(field(&stream(), "nope", 0).len(), stream().count());
}

/// Asking for a Vec2 column as a scalar (or the other way round) is a mistake, not a
/// reinterpretation: zeros, at full length. The stream's types are not guesses to be coerced.
#[test]
fn a_column_of_the_wrong_kind_is_not_coerced() {
    assert_eq!(
        field(&stream(), "vel", 0),
        vec![0.0; 3],
        "vel is not a scalar"
    );
    assert_eq!(field(&stream(), "age", MODE_LENGTH), vec![0.0; 3]);
}

/// **A DIRECTION becomes readable** — the gap five families found at once (doc 89 §10.0).
///
/// The value domain could read any column by name and only ever get a scalar or a magnitude
/// back, so a tangent was unreachable and *"turn to face where you're going"* had no path
/// through this library.
///
/// ⚠️ The fixture's third element is `[-1, 0]`: its X is **−1** and its LENGTH is **+1**. A
/// component mode that quietly fell back to the magnitude would agree with this test on the
/// first two elements and disagree only there — which is why the fixture has a negative lane.
#[test]
fn a_vector_column_reads_lane_by_lane() {
    let x = MODE_COMPONENT_BASE;
    let y = MODE_COMPONENT_BASE + 1;
    assert_eq!(field(&stream(), "vel", x), vec![3.0, 0.0, -1.0], "X");
    assert_eq!(field(&stream(), "vel", y), vec![4.0, 0.0, 0.0], "Y");
    // The magnitude is still its own mode, and still says +1 where X says −1.
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
}

/// **One rung, every width** — a colour is a `Vec4` and its lanes are R·G·B·A. Without this the
/// hue/saturation gap of family 9 stays inexpressible: nothing could read a colour back out.
#[test]
fn the_same_rung_reads_a_colour_and_a_vec3() {
    let s = Stream::new(2)
        .with(
            "tint",
            Column::Vec4(vec![[0.1, 0.2, 0.3, 0.4], [0.5, 0.6, 0.7, 0.8]]),
        )
        .with("nrm", Column::Vec3(vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]]));
    for (k, want) in [(0, [0.1, 0.5]), (1, [0.2, 0.6]), (3, [0.4, 0.8])] {
        let got = field(&s, "tint", MODE_COMPONENT_BASE + k);
        assert_eq!(got, want.to_vec(), "colour lane {k}");
    }
    assert_eq!(field(&s, "nrm", MODE_COMPONENT_BASE + 2), vec![3.0, 6.0]);
}

/// **A lane the column does not have is the ordinary miss, not an error** — the module's fence
/// stands: this rung adds a question the node can ANSWER; it does not change what happens when
/// it cannot. `Z` of a `Vec2` is zeros at full length, exactly like a mistyped name.
#[test]
fn a_lane_the_column_does_not_have_is_zeros_not_a_crash() {
    let z = MODE_COMPONENT_BASE + 2;
    assert_eq!(field(&stream(), "vel", z), vec![0.0; 3], "Z of a Vec2");
    assert_eq!(field(&stream(), "age", z), vec![0.0; 3], "Z of a scalar");
    // Lane 0 of a scalar IS the scalar: a scalar is a one-lane vector.
    let x = MODE_COMPONENT_BASE;
    assert_eq!(field(&stream(), "age", x), vec![0.0, 1.5, 3.0]);
}

/// **The rung is additive: the two modes that shipped are byte-identical.** A default that
/// changes what already-authored art does is not a new mode, it is a regression with a chip.
#[test]
fn the_modes_that_shipped_are_untouched() {
    assert_eq!(field(&stream(), "age", 0), vec![0.0, 1.5, 3.0]);
    assert_eq!(field(&stream(), "vel", MODE_LENGTH), vec![5.0, 0.0, 1.0]);
    assert_eq!(field(&stream(), "vel", 0), vec![0.0; 3], "Vec2 in Scalar");
    assert_eq!(field(&stream(), "age", MODE_LENGTH), vec![0.0; 3]);
}

/// **Every channel the picker offers must name a column something WRITES.**
///
/// A picker entry is a promise: the artist chooses a word and gets that quantity. The
/// entry named "Opacity" pointed at a column `"opacity"` — and nothing in the node library
/// writes one. `motion.drive`'s opacity channel writes **`tint` lane 3**
/// (`drive::channel::CH_OPACITY => "tint"`), and `lower_to_instances` reads the alpha from
/// exactly there (`RenderInstance.opacity` is hardcoded to `1.0` — there is no per-instance
/// opacity surface). So the entry resolved to the module's ORDINARY MISS: zeros at full
/// length, in silence, indistinguishable from a mistyped attribute.
///
/// This gate is written against the DATA, not against a list of channel names: it builds a
/// stream the way `motion.drive` leaves one and asserts the picker's own entry reads back
/// what was written. A gate that compared `column` to the literal `"tint"` would be a
/// mirror of the fix rather than a check on it.
#[test]
fn every_offered_channel_reads_a_column_the_library_actually_writes() {
    // A stream as `motion.drive`'s opacity channel leaves it: the alpha lives in `tint`.
    let drove = Stream::new(3).with(
        "tint",
        Column::Vec4(vec![
            [1.0, 1.0, 1.0, 0.25],
            [1.0, 1.0, 1.0, 0.50],
            [1.0, 1.0, 1.0, 0.75],
        ]),
    );
    let opacity = READ_CHANNELS
        .iter()
        .find(|c| c.label == "Opacity")
        .expect("the picker offers an Opacity channel");
    assert_eq!(
        field(&drove, opacity.column, opacity.mode),
        vec![0.25, 0.50, 0.75],
        "picking `Opacity` must read back the alpha `motion.drive` wrote"
    );
}

/// The other entries are not collateral of the fix: each still reads its own column.
/// (Without this, pointing every channel at `tint` would satisfy the gate above.)
#[test]
fn the_other_channels_still_read_their_own_columns() {
    let s = Stream::new(3)
        .with("age", Column::Scalar(vec![0.1, 0.2, 0.3]))
        .with("tint", Column::Vec4(vec![[1.0, 1.0, 1.0, 0.9]; 3]));
    let age = READ_CHANNELS.iter().find(|c| c.label == "Age").unwrap();
    assert_eq!(field(&s, age.column, age.mode), vec![0.1, 0.2, 0.3]);
}
