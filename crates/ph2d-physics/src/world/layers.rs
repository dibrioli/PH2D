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
//! ## Why 32 layers (decisão do dono, 07/11/2026)
//!
//! `32` is the width of rapier's `Group` (`u32`): the resource ceiling. The panel shows only the
//! layers in use, so its legibility no longer binds this number; the cost of the cells is measured
//! in `docs/GameEngineFuncional/componentes/f1_physics_static_shape.md`.

use rapier2d::geometry::{Group, InteractionGroups, InteractionTestMode};

/// How many collision layers exist: the bits of rapier's `Group`.
pub const MAX_LAYERS: usize = 32;

/// How many layers a project written up to schema 218 could address.
pub const LEGACY_LAYERS: usize = 8;

/// Which layers each layer collides with. Row `i`, bit `j` set = layers `i` and
/// `j` collide.
///
/// **Always symmetric** — see the module docs. Plain `Copy` data so the ECS
/// bridge can hold one without a rapier dependency.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LayerMatrix {
    rows: [u32; MAX_LAYERS],
}

impl LayerMatrix {
    /// Everything collides with everything — the default, and what the engine
    /// did before layers existed.
    pub const fn all() -> Self {
        Self {
            rows: [u32::MAX; MAX_LAYERS],
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
    pub fn row(&self, layer: usize) -> u32 {
        self.rows.get(layer).copied().unwrap_or(u32::MAX)
    }

    /// The rows, for serialization by the ECS layer.
    pub fn rows(&self) -> [u32; MAX_LAYERS] {
        self.rows
    }

    /// Rebuild from raw rows (a loaded project). **Symmetrized on the way in**:
    /// a file written by hand, or by a future build with different rules, must
    /// not be able to install a matrix this type says cannot exist. A pair is
    /// taken to collide only if BOTH halves say so — the same conservative
    /// reading rapier's `AND` already applies.
    pub fn from_rows(rows: [u32; MAX_LAYERS]) -> Self {
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

    /// As linhas de um projeto até o schema 218 (8 camadas de 8 bits), por extenso.
    ///
    /// **A regra:** os pares `0..8` ficam como estão e todo par com uma camada nova colide, porque
    /// é o que uma camada vale de fábrica ([`Self::all`]). Nada do projeto antigo está nelas, logo
    /// nenhum contato muda; a matriz de fábrica antiga vira exatamente a de fábrica nova.
    #[must_use]
    pub fn rows_from_legacy(old: [u8; LEGACY_LAYERS]) -> [u32; MAX_LAYERS] {
        const NOVAS: u32 = u32::MAX << LEGACY_LAYERS;
        let mut rows = [u32::MAX; MAX_LAYERS];
        for (linha, velha) in rows.iter_mut().zip(old) {
            *linha = NOVAS | u32::from(velha);
        }
        rows
    }
}

impl Default for LayerMatrix {
    fn default() -> Self {
        Self::all()
    }
}

/// `(layer, matrix)` → rapier's `InteractionGroups`. **The single door.**
///
/// `memberships` is the body's own layer bit — which is also how a collider
/// remembers its layer, so nothing else has to store it. `filter` is that
/// layer's row of the matrix.
/// ⚠️ **`InteractionTestMode::And` é o comportamento de sempre, escrito à mão.** A rapier 0.31
/// acrescentou um terceiro argumento que escolhe como os dois lados se testam: `And` (o `default`,
/// e o único que existia antes — *ambos os lados têm de aceitar o outro*) ou `Or` (*basta um, e só
/// se os DOIS pedirem `Or`*).
///
/// ⛔ **Passar o `And` explicitamente, em vez de o herdar de um `..Default`, é deliberado:** esta é
/// a porta ÚNICA das camadas de colisão, e a regra que ela implementa — *«uma camada só interage
/// com outra se as duas linhas da matriz concordarem»* — é **exactamente** o `And`. Se um dia o
/// upstream mudar o `default`, a matriz do artista passaria a significar outra coisa **sem uma
/// linha nossa mudar**. O terceiro argumento é a nossa resposta, não a herança da deles.
pub(crate) fn groups_for(layer: usize, matrix: LayerMatrix) -> InteractionGroups {
    let layer = layer.min(MAX_LAYERS - 1);
    let row = matrix.row(layer);
    // Linha vazia: «as camadas não vazias», para que só o lado ABERTO de uma forma com exceção
    // chegue ao gancho (prova em `layer_override`). Sem exceção nenhum par muda.
    let filter = if row == 0 {
        !matrix.rows().iter().enumerate().fold(
            0u32,
            |e, (i, r)| {
                if *r == 0 { e | (1 << i) } else { e }
            },
        )
    } else {
        row
    };
    InteractionGroups::new(
        Group::from_bits_truncate(1 << layer),
        Group::from_bits_truncate(filter),
        InteractionTestMode::And,
    )
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
        let mut rows = [u32::MAX; MAX_LAYERS];
        rows[6] &= !(1 << 2);
        let m = LayerMatrix::from_rows(rows);
        assert!(
            !m.collides(2, 6) && !m.collides(6, 2),
            "an asymmetric file must resolve to the reading rapier would ACT on \
             (no collision), not to the half that happens to be set"
        );
    }

    #[test]
    fn os_bits_altos_sobrevivem_a_from_rows() {
        let mut m = LayerMatrix::all();
        m.set(20, 25, false);
        let lida = LayerMatrix::from_rows(m.rows());
        assert!(!lida.collides(20, 25) && !lida.collides(25, 20));
        assert!(lida.collides(20, 24) && lida.collides(31, 31));
        assert_eq!(lida.row(31), u32::MAX);
    }

    #[test]
    fn a_matriz_antiga_guarda_os_seus_pares_e_as_novas_colidem_como_as_de_fabrica() {
        assert_eq!(
            LayerMatrix::rows_from_legacy([u8::MAX; LEGACY_LAYERS]),
            LayerMatrix::all().rows(),
            "a de fábrica antiga é a de fábrica nova"
        );
        let mut velha = [u8::MAX; LEGACY_LAYERS];
        velha[1] &= !(1 << 5);
        velha[5] &= !(1 << 1);
        velha[3] = 0;
        let m = LayerMatrix::from_rows(LayerMatrix::rows_from_legacy(velha));
        for a in 0..LEGACY_LAYERS {
            for b in 0..LEGACY_LAYERS {
                let antes = velha[a] & (1 << b) != 0 && velha[b] & (1 << a) != 0;
                assert_eq!(m.collides(a, b), antes, "o par antigo {a},{b}");
            }
        }
        for a in 0..MAX_LAYERS {
            for b in LEGACY_LAYERS..MAX_LAYERS {
                assert!(m.collides(a, b), "a camada nova {b} colide com {a}");
            }
        }
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
