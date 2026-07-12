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
