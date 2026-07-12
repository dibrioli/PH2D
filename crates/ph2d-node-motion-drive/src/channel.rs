//! Value → transform-channel plumbing for `motion.drive`, including the **one
//! broadcast rule** the whole value domain hangs on (doc 12): a value field of
//! length 1 is HELD (broadcast) across every instance; a length-N field applies
//! element-wise; any other length is a mismatch (`debug_assert`, then a lenient
//! element-wise fallback — no silent no-op). This is the TouchDesigner
//! "held constant" / Houdini "detail→point" rule, restricted to `1→N` only so
//! the strict substrate stays honest.
//!
//! Self-contained per drop-crate (like every behaviour's `channel`/`falloff`
//! helpers). Combine modes lerp the RESULT toward the driven value by the
//! multiplicative `falloff` field, so a focus mask limits which instances the
//! value drives — consistent with the rest of the family.

use ph2d_nodegraph::attr::{Column, Stream};

/// The multiplicative `falloff` weight for instance `i` (absent → `1.0`).
pub(crate) fn falloff_at(stream: &Stream, i: usize) -> f32 {
    match stream.get("falloff") {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

/// The value driving instance `i`, applying the broadcast rule: a length-1
/// field broadcasts (the constant is held at every instance); a length-N field
/// is read element-wise. A length that is neither 1 nor `n` is a mismatch —
/// `debug_assert`ed loudly, then read leniently (element-wise, `0.0` past the
/// end) so a release build degrades rather than panics.
pub(crate) fn value_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 0.0,
        1 => vals[0], // broadcast: one value → every instance (the 1→N rule)
        _ => vals.get(i).copied().unwrap_or(0.0),
    }
}

/// How the driven value combines with the existing channel.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Combine {
    /// `channel + value` — the additive default (matches `motion.step`).
    Add,
    /// `value` — overwrite the channel with the driven value.
    Set,
    /// `channel * value` — scale the existing channel by the value.
    Multiply,
}

impl Combine {
    pub(crate) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Combine::Set,
            2 => Combine::Multiply,
            _ => Combine::Add,
        }
    }
    fn apply(self, channel: f32, value: f32) -> f32 {
        match self {
            Combine::Add => channel + value,
            Combine::Set => value,
            Combine::Multiply => channel * value,
        }
    }
}

/// The channel index of Opacity — the alpha of the `tint` column.
pub(crate) const CH_OPACITY: i32 = 4;

/// The stream column a channel index writes to: X/Y → `P`, Rotation → `rot`,
/// Opacity → `tint` (its alpha), Size (or any out-of-range value) → `size`.
fn channel_column(channel: i32) -> &'static str {
    match channel {
        0 | 1 => "P",
        2 => "rot",
        CH_OPACITY => "tint",
        _ => "size",
    }
}

fn base_vec2(input: &Stream, name: &str, n: usize, identity: [f32; 2]) -> Vec<[f32; 2]> {
    let mut v = match input.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}
fn base_vec4(input: &Stream, name: &str, n: usize, identity: [f32; 4]) -> Vec<[f32; 4]> {
    let mut v = match input.get(name) {
        Some(Column::Vec4(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, identity);
    v
}
fn base_scalar(input: &Stream, name: &str, n: usize) -> Vec<f32> {
    let mut v = match input.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    v.resize(n, 0.0);
    v
}

/// Drive the selected `channel` of `input` from the value field `vals`
/// (scaled by `scale`, combined by `mode`), broadcast per the rule above and
/// masked per-instance by `falloff`. Returns a new stream with that column
/// rewritten and every other column copied through. Size drives both
/// components (uniform); an absent target column starts from its identity
/// (`0` for P/rot, unit `[1,1]` for size).
pub(crate) fn drive_channel(
    input: &Stream,
    channel: i32,
    vals: &[f32],
    scale: f32,
    mode: Combine,
) -> Stream {
    let n = input.count();
    debug_assert!(
        vals.is_empty() || vals.len() == 1 || vals.len() == n,
        "motion.drive: value field len {} matches neither 1 (broadcast) nor {n} (element-wise)",
        vals.len()
    );
    let target = channel_column(channel);
    let mut out = Stream::new(n);
    for (name, col) in input.columns() {
        if name != target {
            out.set(name.clone(), col.clone());
        }
    }
    // Lerp the combined result toward the original by falloff: `falloff = 0`
    // leaves the channel untouched, `1` takes the full drive.
    let blend = |orig: f32, driven: f32, f: f32| orig + (driven - orig) * f.clamp(0.0, 1.0);
    match channel {
        0 | 1 => {
            let comp = channel as usize; // 0 = X, 1 = Y
            let mut p = base_vec2(input, "P", n, [0.0, 0.0]);
            for (i, pi) in p.iter_mut().enumerate() {
                let driven = mode.apply(pi[comp], value_at(vals, i) * scale);
                pi[comp] = blend(pi[comp], driven, falloff_at(input, i));
            }
            out.set("P", Column::Vec2(p));
        }
        2 => {
            let mut r = base_scalar(input, "rot", n);
            for (i, ri) in r.iter_mut().enumerate() {
                let driven = mode.apply(*ri, value_at(vals, i) * scale);
                *ri = blend(*ri, driven, falloff_at(input, i));
            }
            out.set("rot", Column::Scalar(r));
        }
        // **Opacity** — the ALPHA of the tint, and the reason a particle can fade out at all
        // (doc 51). An element with no tint starts from opaque white, so driving the opacity of
        // an uncoloured stream does exactly what it says instead of silently doing nothing.
        //
        // Clamped to `[0, 1]`: the renderer alpha-blends, and an alpha of 1.4 or -0.2 is not a
        // brighter or a darker particle — it is a particle that reads as a bug.
        CH_OPACITY => {
            let mut t = base_vec4(input, "tint", n, [1.0, 1.0, 1.0, 1.0]);
            for (i, ti) in t.iter_mut().enumerate() {
                let driven = mode.apply(ti[3], value_at(vals, i) * scale);
                ti[3] = blend(ti[3], driven, falloff_at(input, i)).clamp(0.0, 1.0); // CLAMP-OK: alpha
            }
            out.set("tint", Column::Vec4(t));
        }
        _ => {
            let mut s = base_vec2(input, "size", n, [1.0, 1.0]);
            for (i, si) in s.iter_mut().enumerate() {
                let f = falloff_at(input, i);
                let v = value_at(vals, i) * scale;
                si[0] = blend(si[0], mode.apply(si[0], v), f);
                si[1] = blend(si[1], mode.apply(si[1], v), f);
            }
            out.set("size", Column::Vec2(s));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_length_one_value_broadcasts_to_every_instance() {
        // Three instances, ONE value (2.0) → all three shift by 2 in X.
        let input =
            Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
        let out = drive_channel(&input, 0, &[2.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[2.0, 0.0], [3.0, 0.0], [4.0, 0.0]]),
            _ => panic!(),
        }
    }

    #[test]
    fn a_length_n_value_applies_element_wise() {
        let input = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
        let out = drive_channel(&input, 0, &[1.0, 2.0, 3.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v, &vec![[1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
            _ => panic!(),
        }
    }

    #[test]
    fn set_and_multiply_combine_against_the_existing_channel() {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[5.0, 0.0]]));
        let set = drive_channel(&input, 0, &[2.0], 1.0, Combine::Set);
        let mul = drive_channel(&input, 0, &[2.0], 1.0, Combine::Multiply);
        assert_eq!(px(&set), 2.0, "set overwrites");
        assert_eq!(px(&mul), 10.0, "multiply scales the existing 5");
    }

    #[test]
    fn falloff_zero_leaves_the_channel_untouched() {
        let input = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
            .with("falloff", Column::Scalar(vec![1.0, 0.0]));
        let out = drive_channel(&input, 0, &[3.0], 1.0, Combine::Add);
        match out.get("P").unwrap() {
            Column::Vec2(v) => {
                assert_eq!(v[0], [3.0, 0.0], "focused instance driven");
                assert_eq!(v[1], [0.0, 0.0], "masked instance untouched");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn size_channel_drives_both_components_from_unit_identity() {
        // A bare P-only stream driven on Size (multiply by 2) → unit×2 on both.
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
        let out = drive_channel(&input, 3, &[2.0], 1.0, Combine::Multiply);
        match out.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v[0], [2.0, 2.0], "unit identity × 2"),
            _ => panic!(),
        }
    }

    fn px(s: &Stream) -> f32 {
        match s.get("P").unwrap() {
            Column::Vec2(v) => v[0][0],
            _ => panic!(),
        }
    }
}
