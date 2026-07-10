//! Read one transform channel of a motion stream (the mirror of the behaviours'
//! `apply_channel_delta`, which WRITES a channel). Self-contained per crate,
//! like `falloff_at` — drop-crate isolation.
//!
//! The identity of an absent column is per-name, NOT a blanket zero (the lesson
//! from the behaviours): `size`'s identity is unit scale `[1,1]`, so a Size
//! threshold on a bare generator reads `1.0`, not `0.0` (a false "below every
//! threshold"). `P`/`rot` are `0`.

use ph2d_nodegraph::attr::{Column, Stream};

/// The scalar value of channel `channel` (0 X · 1 Y · 2 Rotation · 3 Size) for
/// instance `i`. Absent / wrong-typed / short column → the channel's identity.
pub(crate) fn channel_get(s: &Stream, channel: i32, i: usize) -> f32 {
    match channel {
        0 | 1 => match s.get("P") {
            Some(Column::Vec2(v)) => v.get(i).map(|p| p[channel as usize]).unwrap_or(0.0),
            _ => 0.0,
        },
        2 => match s.get("rot") {
            Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(0.0),
            _ => 0.0,
        },
        // Size: read X of the `size` vec2 (uniform), identity unit scale.
        _ => match s.get("size") {
            Some(Column::Vec2(v)) => v.get(i).map(|s| s[0]).unwrap_or(1.0),
            _ => 1.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_each_channel_and_falls_back_to_the_per_name_identity() {
        let s = Stream::new(1)
            .with("P", Column::Vec2(vec![[2.0, 3.0]]))
            .with("rot", Column::Scalar(vec![0.7]));
        assert_eq!(channel_get(&s, 0, 0), 2.0); // X
        assert_eq!(channel_get(&s, 1, 0), 3.0); // Y
        assert_eq!(channel_get(&s, 2, 0), 0.7); // Rotation
        // `size` absent → unit identity `1.0`, NOT `0.0`.
        assert_eq!(channel_get(&s, 3, 0), 1.0);
        // `P` absent → `0.0`.
        let bare = Stream::new(1).with("rot", Column::Scalar(vec![0.1]));
        assert_eq!(channel_get(&bare, 1, 0), 0.0);
    }
}
