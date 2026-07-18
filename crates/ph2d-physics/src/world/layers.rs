//! Collision layers — *who collides with whom*, as a world-level rule.
//!
//! ## The model, and why this one
//!
//! Two models ship in the wild:
//!
//! * **Godot / Box2D**: every body carries a `layer` bitmask AND a `mask`
//!   bitmask. Maximum flexibility, no global state — and the rule
//!   "bullets do not hit the player who fired them" has to be re-typed on every
//!   bullet and every player.
//! * **Unity**: one global **matrix** (layer × layer → collide?), and each body
//!   names a single layer. The rule is authored ONCE, where it belongs — in the
//!   world.
//!
//! rapier's native shape is the first (`InteractionGroups { memberships,
//! filter }`), so this module is the second built on top: `memberships` is the
//! body's own layer bit, and `filter` is that layer's **row of the matrix**.
//!
//! ## ⚠️ The matrix must be SYMMETRIC, and here it is unrepresentable otherwise
//!
//! rapier's own rule is:
//!
//! ```text
//! interact  ⟺  (A.memberships ∩ B.filter) ≠ ∅  AND  (B.memberships ∩ A.filter) ≠ ∅
//! ```
//!
//! Both directions. So an asymmetric matrix — `[i][j]` set, `[j][i]` clear —
//! does not mean "i sees j but not the reverse"; the `AND` makes it mean **no
//! collision at all**, which is a rule nobody wrote and nobody can see. Rather
//! than gate against that state, [`LayerMatrix::set`] writes **both halves**, so
//! the asymmetric matrix does not exist. One cell of the UI is one fact.
//!
//! ## Why eight layers
//!
//! The representation allows 32 (rapier's `Group` is a `u32`) — that is the
//! hard ceiling and it is not what binds here. What binds is the panel: a
//! triangular matrix of `N` layers has `N(N+1)/2` cells, so 8 → **36**,
//! 16 → 136, 32 → **528**. Unity ships 32 and its matrix is the standard example
//! of a settings screen nobody can read. Eight is what stays legible at the
//! width of a docked panel.
//!
//! Growing it later is a UI change plus a schema bump, not a physics change —
//! the storage below is the only thing that would move.

/// How many collision layers the editor exposes. See the module docs for why
/// this is 8 and not rapier's representational 32.
pub const MAX_LAYERS: usize = 8;

/// Which layers each layer collides with. Row `i`, bit `j` set = layers `i` and
/// `j` collide.
///
/// **Always symmetric** — see the module docs. Plain `Copy` data so the ECS
/// bridge can hold one without a rapier dependency.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LayerMatrix {
    rows: [u8; MAX_LAYERS],
}

impl LayerMatrix {
    /// Everything collides with everything — the default, and what the engine
    /// did before layers existed.
    pub const fn all() -> Self {
        Self {
            rows: [u8::MAX; MAX_LAYERS],
        }
    }

    /// Do layers `a` and `b` collide?
    pub fn collides(&self, a: usize, b: usize) -> bool {
        if a >= MAX_LAYERS || b >= MAX_LAYERS {
            return false;
        }
        self.rows[a] & (1 << b) != 0
    }

    /// Set whether layers `a` and `b` collide.
    ///
    /// **Writes both halves.** The caller is stating one fact about a pair, not
    /// two facts about two directions — and rapier's `AND` rule means a
    /// half-written pair silently stops colliding entirely (module docs).
    pub fn set(&mut self, a: usize, b: usize, collides: bool) {
        if a >= MAX_LAYERS || b >= MAX_LAYERS {
            return;
        }
        for (i, j) in [(a, b), (b, a)] {
            if collides {
                self.rows[i] |= 1 << j;
            } else {
                self.rows[i] &= !(1 << j);
            }
        }
    }

    /// Row `i` as a raw bitmask — what becomes a collider's `filter`.
    pub fn row(&self, layer: usize) -> u8 {
        self.rows.get(layer).copied().unwrap_or(u8::MAX)
    }

    /// The rows, for serialization by the ECS layer.
    pub fn rows(&self) -> [u8; MAX_LAYERS] {
        self.rows
    }

    /// Rebuild from raw rows (a loaded project). **Symmetrized on the way in**:
    /// a file written by hand, or by a future build with different rules, must
    /// not be able to install a matrix this type says cannot exist. A pair is
    /// taken to collide only if BOTH halves say so — the same conservative
    /// reading rapier's `AND` already applies.
    pub fn from_rows(rows: [u8; MAX_LAYERS]) -> Self {
        let mut m = Self {
            rows: [0; MAX_LAYERS],
        };
        for a in 0..MAX_LAYERS {
            for b in 0..MAX_LAYERS {
                let both = rows[a] & (1 << b) != 0 && rows[b] & (1 << a) != 0;
                if both {
                    m.rows[a] |= 1 << b;
                }
            }
        }
        m
    }
}

impl Default for LayerMatrix {
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_one_cell_writes_both_halves() {
        let mut m = LayerMatrix::all();
        m.set(1, 5, false);
        assert!(!m.collides(1, 5));
        assert!(
            !m.collides(5, 1),
            "the mirror half was left set — rapier ANDs both directions, so this \
             pair would silently stop colliding for a reason nobody wrote"
        );
    }

    #[test]
    fn a_lopsided_file_is_read_conservatively() {
        // Row 2 says "collides with 6", row 6 does not agree.
        let mut rows = [u8::MAX; MAX_LAYERS];
        rows[6] &= !(1 << 2);
        let m = LayerMatrix::from_rows(rows);
        assert!(
            !m.collides(2, 6) && !m.collides(6, 2),
            "an asymmetric file must resolve to the reading rapier would ACT on \
             (no collision), not to the half that happens to be set"
        );
    }

    #[test]
    fn the_default_collides_with_everything() {
        let m = LayerMatrix::all();
        for a in 0..MAX_LAYERS {
            for b in 0..MAX_LAYERS {
                assert!(m.collides(a, b), "default must be permissive at {a},{b}");
            }
        }
    }
}
