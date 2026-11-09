//! **O Pixel Sprite na cena** (P14 §4 C): a «Collision Shape» que segue uma slice.

use crate::{ComponentCategory as C, ComponentDesc as D, Hidden};

/// Ordenado pelo nome canónico (o `desc_for` é busca binária).
pub const DESCS: &[D] = &[
    // P14e: a forma de um Sprite do motor importado que segue a célula mostrada (o import a cria).
    D::intrinsic(
        "ph2d::pixel::SegueCelula",
        "component.segue_celula.name",
        C::Physics,
        &[],
    )
    .hidden(Hidden::Editor),
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
