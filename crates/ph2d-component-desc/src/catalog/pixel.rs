//! **O Pixel Sprite na cena** (P14 §4 C): a «Collision Shape» que segue uma slice.

use crate::{ComponentCategory as C, ComponentDesc as D, Hidden};

/// Ordenado pelo nome canónico (o `desc_for` é busca binária).
pub const DESCS: &[D] = &[
    // Nasce do gesto «Add Collider» da janela SLICES (o id da slice só ele conhece); anexá-lo pela
    // paleta faria uma forma a seguir slice nenhuma.
    D::intrinsic(
        "ph2d::pixel::SegueSlice",
        "component.segue_slice.name",
        C::Physics,
        &[],
    )
    .hidden(Hidden::Editor),
];
