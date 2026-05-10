//! Z-stack layer tokens. Source: `docs/design/tokens.json` → `z.*`.
//!
//! Canonical layers — order is monotonic and stable. A widget that
//! needs an arbitrary z between these layers is a code smell.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum Layer {
    /// `canvas` (z=0) — main viewport, sprites, scene render.
    Canvas,
    /// `panel` (z=10) — floating panels, dock, sidebar.
    Panel,
    /// `overlay` (z=20) — popovers, dropdowns, color picker.
    Overlay,
    /// `modal` (z=30) — dialogs, prompts, confirmations.
    Modal,
    /// `toast` (z=40) — non-modal notifications.
    Toast,
    /// `tooltip` (z=50) — hover hints, gesture cheatsheet.
    Tooltip,
}

impl Layer {
    pub const fn z(self) -> u32 {
        match self {
            Self::Canvas => 0,
            Self::Panel => 10,
            Self::Overlay => 20,
            Self::Modal => 30,
            Self::Toast => 40,
            Self::Tooltip => 50,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z_values_strictly_increasing() {
        let layers = [
            Layer::Canvas,
            Layer::Panel,
            Layer::Overlay,
            Layer::Modal,
            Layer::Toast,
            Layer::Tooltip,
        ];
        for w in layers.windows(2) {
            assert!(w[0].z() < w[1].z(), "{:?} → {:?}", w[0], w[1]);
        }
    }

    #[test]
    fn enum_ord_matches_z_order() {
        // Derive(Ord) on the enum must match the z semantics.
        assert!(Layer::Canvas < Layer::Panel);
        assert!(Layer::Panel < Layer::Overlay);
        assert!(Layer::Tooltip > Layer::Modal);
    }
}
