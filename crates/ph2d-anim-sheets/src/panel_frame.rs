//! A porta entre o quadro do motor ([`FrameRef`]) e o do painel ([`AnimFrameRef`]): uma lei, dois sentidos.

use ph2d_ecs::FrameRef;
use ph2d_editor_core::AnimFrameRef;

/// O quadro do motor como o painel o mostra.
#[must_use]
pub const fn to_panel(f: FrameRef) -> AnimFrameRef {
    match f {
        FrameRef::Cell { sheet, cell } => AnimFrameRef { sheet, cell },
        FrameRef::Empty => AnimFrameRef::EMPTY,
    }
}

/// O quadro que o painel pediu, para o motor.
#[must_use]
pub const fn from_panel(f: AnimFrameRef) -> FrameRef {
    if f.is_empty() {
        FrameRef::Empty
    } else {
        FrameRef::Cell {
            sheet: f.sheet,
            cell: f.cell,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_quadro_vai_ao_painel_e_volta_igual() {
        for f in [
            FrameRef::cell(0),
            FrameRef::Cell { sheet: 3, cell: 11 },
            FrameRef::Empty,
        ] {
            assert_eq!(from_panel(to_panel(f)), f);
        }
        assert_eq!(
            to_panel(FrameRef::Cell { sheet: 2, cell: 5 }),
            AnimFrameRef { sheet: 2, cell: 5 }
        );
        assert!(to_panel(FrameRef::Empty).is_empty());
    }
}
