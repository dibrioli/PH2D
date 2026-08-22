//! **External values — the door from the APP into the cook** (Motion Nodes doc 65).
//!
//! A node gets three things: its params, its inputs, and the playhead. That is deliberate — it is
//! what makes a node a pure function of the graph, and it is why the cook can memoize, scrub and
//! replay bit-exactly. But it also means a node **cannot see anything the application owns**: the
//! vector document, the selection, the mouse. And `motion.path` — "walk this drawn curve" — needs
//! exactly that: a shape the artist drew, which lives in `ph2d-vec-scene`, in the shell.
//!
//! The plan said *"integra `vector.*`"* — but that node family was **RETIRED** (ADR-0108), and the
//! geometry moved into a document the graph has no reach into. So the question stopped being *how
//! do I import a curve* and became **how does anything outside the graph get in at all**.
//!
//! ## The answer is the one this crate keeps giving
//!
//! Not a new port kind (`NodeManifest.inputs` is `&'static` — ADR-0039, frozen). Not a dependency
//! from `ph2d-nodegraph` on the vector crates (a leaf substrate that knows one domain's data type
//! is not a substrate). **A named channel on the `Cook`**, published by whoever owns the data:
//!
//! ```text
//! shell:  cook.set_external("Track", polyline_of(the_path_named_Track))
//! node:   ctx.external("Track")   // -> a Stream, or empty if nobody published one
//! ```
//!
//! It is the third turn of the same crank — the text-param channel (doc 32) and the driven-param
//! channel (doc 58) both got here by asking *"where does state that the manifest cannot describe
//! actually live?"* and answering **"beside it, not inside it"**.
//!
//! ## The memo has to see it, and that is the whole difficulty
//!
//! An external is an INPUT the fingerprint does not know about. Edit the curve, and every node
//! downstream must recompute — but the cook decides whether to recompute *before* it evaluates,
//! and it only learns which external a node read *during* evaluation. Chicken and egg.
//!
//! So the cook **remembers what the node read last time** (`Cook` keeps the names beside the memo)
//! and folds *those* names' current revisions into the fingerprint. If the node has never cooked,
//! the field is 0 and it cooks. If it starts reading a *different* external, the text param that
//! names it changed — and that is already in the fingerprint. Precise, and it costs one `Vec<String>`
//! per cached node.
//!
//! The revision is the **content**: `set_external` hashes the stream, so a caller cannot get the
//! bookkeeping wrong by forgetting to bump a counter. Publishing the same curve twice is free.

use crate::attr::{Column, Stream};
use std::collections::BTreeMap;

/// **The prefix the EDITOR reserves for values of its own.**
///
/// Every external is keyed by a name the ARTIST typed — that is the whole design of
/// this channel. But the editor also has values a graph wants and a document cannot
/// hold: the cursor, and whatever follows it. They arrive through the same door,
/// because there is only one door.
///
/// With one flat namespace that is a collision waiting for a name: somebody calls a
/// sprite `$cursor` and their sprite silently BECOMES the mouse — no error, no
/// warning, just a `motion.look_at` aiming at a thing that moves with the pointer.
///
/// So the prefix is declared HERE, beside the table, rather than inside whichever
/// node happens to read one of these values first. The rule has two halves and both
/// belong to the publisher: the editor publishes INTO the namespace, and it refuses
/// to publish an artist's name that is already in it.
pub const RESERVED_PREFIX: char = '$';

/// Is `name` inside the editor's reserved namespace? ([`RESERVED_PREFIX`])
///
/// Leading whitespace is trimmed first, so ` $cursor` cannot slip past a check that
/// `$cursor` fails — the publishers already trim before rejecting an empty name.
#[must_use]
pub fn is_reserved(name: &str) -> bool {
    name.trim_start().starts_with(RESERVED_PREFIX)
}

/// The external the editor publishes the **world-space cursor** under.
///
/// The cursor is not a document value and cannot become one: it is an editor input
/// that changes every frame. Naming it here is what lets a node aim at the mouse
/// without learning what a window or a camera is.
pub const CURSOR: &str = "$cursor";

/// The external the editor publishes a named thing's **world POSITION** under.
///
/// ⚠️ **A thing's appearance and a thing's position are different questions, and the
/// appearance channel answers only the first.** An object is published as a
/// one-instance APPEARANCE stream whose `P` is `[0, 0]` — deliberately, because it
/// describes what the object looks like and the graph decides where copies of it go.
/// Reading that `P` as "where the object is" gives the ORIGIN for every object in the
/// scene: not a wrong number so much as another question's answer wearing the right
/// column name, which is the quietest way to be wrong.
///
/// So position gets its own channel. One lookup, one meaning — and it covers a drawn
/// curve too, so a node asking "where is the thing called X" never has to know whether
/// X is a sprite or a path.
#[must_use]
pub fn position_of(name: &str) -> String {
    format!("{RESERVED_PREFIX}at:{name}")
}

/// The external the editor publishes a named object's **POSE** under — the rotation and
/// the scale it carries in the scene, `rotation` (radians) + `size` (a `Vec2` factor).
///
/// ⚠️ **Um terceiro canal, e pela MESMA razão que o segundo existe** (ver
/// [`position_of`]): a aparência diz *como a coisa é* e mora na origem, sem pose, porque
/// o grafo é que decide onde as cópias vão. A POSIÇÃO já tinha canal próprio; a rotação
/// e a escala não tinham nenhum — o `Transform` estava na query do shell e era
/// **descartado** (doc 89 folha 14).
///
/// ⚠️ **Ele é lido só por quem PEDE a pose.** O `source.object` nasce em *Position Only*
/// — o template de sempre, byte-idêntico — e só o modo *Object Pose* consulta este
/// canal. Um canal publicado que ninguém lê custa um `set_external` por objeto por
/// quadro; o revision é um hash do conteúdo, então um objeto parado não invalida nada.
#[must_use]
pub fn pose_of(name: &str) -> String {
    format!("{RESERVED_PREFIX}pose:{name}")
}

/// The external the editor publishes a named drawing's **GEOMETRY** under — the
/// flattened polyline of the curve, `P` per vertex.
///
/// ⚠️ **The third question about the same name, and it needed the same cure.** The
/// appearance channel (the raw name) answers *"what does X look like"* with ONE
/// instance at the origin; [`position_of`] answers *"where is X"*. A node walking a
/// drawn curve asks a third thing — *"what SHAPE is X"* — and the polyline used to
/// ride the raw name, where the object-bake publisher **overwrote it every frame**
/// with the appearance (the two publishers say so in their own comments: *"objects
/// publish after curves; the last write on a name clash wins"*). The reader got a
/// one-point stream, could not find an arc, and fell back — silently.
///
/// The same shape of bug `position_of` was cut for, one question over: another
/// question's answer wearing the right column name. So the geometry gets its own
/// channel, and the raw name keeps meaning **appearance** for `source.object`.
#[must_use]
pub fn curve_of(name: &str) -> String {
    format!("{RESERVED_PREFIX}curve:{name}")
}

/// The external a named object's **APPEARANCE at a SHIFTED time** rides on — the
/// fourth question about the same name, and the first that is about WHEN.
///
/// The raw name answers *"what does X look like"* — at the playhead, now. That is
/// the only answer a cel-animated object could give, because the shell bakes its
/// tile **once per object, at the app's current frame**: two `source.object` nodes
/// naming the same Flip got the SAME drawing, and nothing downstream could ask for
/// another. A stagger of copies each showing a different drawing — the canonical
/// pair with an offset in every reference that has one — was not expressible.
///
/// So a shifted appearance gets its own key, exactly as [`position_of`] and
/// [`curve_of`] did when they were another question wearing the right column name.
///
/// ⚠️ **The zero offset returns the RAW NAME, and that is the whole neutrality
/// argument.** A graph that never touches the param mints no key, publishes no extra
/// external, bakes no extra tile and reads the channel it always read — byte-identical
/// to every frame this app has drawn. There is no "shifted by nothing" state to keep
/// in agreement with the unshifted one, because it is the same string.
///
/// ⚠️ **`-0.0` also returns the raw name**, because `-0.0 == 0.0` in IEEE-754 — and a
/// shift of negative nothing is no shift. The alternative (comparing bits) would mint
/// a second key for a value the artist cannot distinguish from zero on any slider.
///
/// The offset is encoded as its **BITS in hex**, never as a formatted decimal: the two
/// callers are a leaf node crate and the shell, and a float formatted by two different
/// call sites is a divergence waiting for a locale or a precision change. Here there is
/// one call site — this function — and the bits are exact.
#[must_use]
pub fn appearance_of(name: &str, time_offset: f32) -> String {
    if time_offset == 0.0 {
        return name.to_string();
    }
    format!(
        "{RESERVED_PREFIX}shift:{:08x}:{name}",
        time_offset.to_bits()
    )
}

/// One published value, and the revision that IS its content.
#[derive(Clone, Debug, PartialEq)]
pub struct External {
    /// FNV-1a over the stream's columns. Two identical publishes have the same revision, so a
    /// shell that republishes every frame (which is the simple thing to do) invalidates nothing.
    pub rev: u64,
    pub value: Stream,
}

/// Everything the app has published, by name.
pub type All = BTreeMap<String, External>;

/// FNV-1a over a stream's shape and contents.
///
/// It walks the columns in `BTreeMap` order, so the hash is deterministic — the same curve, on any
/// machine, is the same revision.
///
/// **Every column kind is hashed, by value.** A hash that skipped one would make a change to it
/// read as *unchanged*, and the memo would hand back the pre-edit curve forever — the exact bug the
/// driven-param fingerprint had to be rescued from (doc 58 §4). The `f32`s go in as BITS: `NaN` is
/// not equal to itself and `-0.0 == 0.0`, and a revision has to be a bitwise fact.
pub fn fingerprint(s: &Stream) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    mix(&(s.count() as u64).to_le_bytes());
    for (name, col) in s.columns() {
        mix(name.as_bytes());
        let mut floats = |xs: &[f32]| {
            for x in xs {
                mix(&x.to_bits().to_le_bytes());
            }
        };
        match col {
            Column::Scalar(v) => floats(v),
            Column::Vec2(v) => v.iter().for_each(|p| floats(p)),
            Column::Vec3(v) => v.iter().for_each(|p| floats(p)),
            Column::Vec4(v) => v.iter().for_each(|p| floats(p)),
        }
    }
    h
}

/// The revisions of the externals `names` refer to, folded into one number for the fingerprint. A
/// name nobody published contributes its absence (not zero — *absence*), so a curve that appears or
/// disappears recomputes the nodes that were asking for it.
pub fn revs_of(all: &All, names: &[String]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut mix = |bytes: &[u8]| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for n in names {
        mix(n.as_bytes());
        match all.get(n) {
            Some(e) => mix(&e.rev.to_le_bytes()),
            None => mix(b"<absent>"),
        }
    }
    h
}

#[cfg(test)]
mod reserved_tests {
    use super::*;

    /// The namespace is a PREFIX rule, and the trim is load-bearing: the publishers
    /// already trim before rejecting an empty name, so a name that survives their trim
    /// must be judged after the same one — otherwise ` $cursor` publishes where
    /// `$cursor` is refused, which is the collision with an extra step.
    #[test]
    fn the_reserved_namespace_is_a_trimmed_prefix() {
        assert!(is_reserved("$cursor"));
        assert!(is_reserved("  $cursor"));
        assert!(is_reserved("$anything at all"));
        assert!(!is_reserved("cursor"));
        assert!(
            !is_reserved("my $cursor"),
            "only the FIRST character reserves"
        );
        assert!(!is_reserved(""));
    }

    /// The cursor's own name is inside the namespace it claims. Obvious, and it is the
    /// half that would rot: rename the constant to something outside the prefix and the
    /// editor happily publishes it into the artist's namespace, where an object can
    /// take it over.
    #[test]
    fn the_cursor_lives_in_the_namespace_it_claims() {
        assert!(is_reserved(CURSOR));
        assert!(CURSOR.starts_with(RESERVED_PREFIX));
    }

    /// **The unshifted appearance IS the raw name** — the neutrality of the whole
    /// channel. If this ever minted a key for zero, every graph in every saved
    /// document would start reading an external nobody publishes: an empty stream,
    /// and the object silently vanishes from the canvas.
    ///
    /// `-0.0` is the same case and not a curiosity: a slider dragged to the left
    /// edge and back can leave it there, and `-0.0 == 0.0` is what makes the two
    /// indistinguishable to the artist. Minting a second key for it would bake a
    /// second tile for the same picture.
    #[test]
    fn a_zero_offset_is_the_name_itself() {
        assert_eq!(appearance_of("Ball", 0.0), "Ball");
        assert_eq!(appearance_of("Ball", -0.0), "Ball");
        assert!(!is_reserved(&appearance_of("Ball", 0.0)));
    }

    /// A shifted appearance lands INSIDE the editor's namespace, so an artist can
    /// never name an object into it. The complement of the test above: the two
    /// halves are what stop a shifted key and a real object from ever being the
    /// same string.
    #[test]
    fn a_shifted_appearance_is_reserved_and_carries_the_name() {
        let k = appearance_of("Ball", 0.25);
        assert!(is_reserved(&k), "{k} escaped the namespace");
        assert!(k.ends_with("Ball"), "{k} lost the name it refers to");
    }

    /// **Distinct offsets are distinct keys, and equal offsets are the SAME key** —
    /// the property the bake cache rides on. Two nodes asking for the same shift of
    /// the same object share one tile; two asking for different shifts must not.
    ///
    /// The pair `0.1` / `0.2` is here because it is the one a decimal format would
    /// most plausibly collapse (one significant digit at a coarse precision); the
    /// bits cannot.
    #[test]
    fn the_key_separates_offsets_and_joins_equal_ones() {
        assert_eq!(appearance_of("Ball", 0.1), appearance_of("Ball", 0.1));
        assert_ne!(appearance_of("Ball", 0.1), appearance_of("Ball", 0.2));
        assert_ne!(appearance_of("Ball", 0.1), appearance_of("Ball", -0.1));
        assert_ne!(appearance_of("Ball", 0.1), appearance_of("Box", 0.1));
    }

    /// The four questions about one name are four DIFFERENT keys. This is the gate
    /// the module's own history asks for: `curve_of` exists because the geometry
    /// used to ride the raw name and the appearance publisher overwrote it every
    /// frame. A shifted appearance colliding with the position channel would be the
    /// same bug, and `$at:` vs `$shift:` is one character of prefix away from it.
    #[test]
    fn the_four_questions_about_a_name_are_four_keys() {
        let keys = [
            appearance_of("Ball", 0.0),
            position_of("Ball"),
            curve_of("Ball"),
            appearance_of("Ball", 0.25),
        ];
        for (i, a) in keys.iter().enumerate() {
            for b in keys.iter().skip(i + 1) {
                assert_ne!(a, b, "two questions about `Ball` share a key");
            }
        }
    }
}
