//! **A seta do cursor** sobre o que o premir agarra ([`crate::sob`]): a direção em que o arrasto
//! mexe. Linha vertical do 9-slice e bordas esquerda/direita: ↔; horizontais: ↕; cantos: as
//! diagonais; o pivot e o interior: mover.

use winit::window::CursorIcon;

use crate::alcas::Grab;

/// **A seta de uma alça.**
#[must_use]
pub const fn seta(g: Grab) -> CursorIcon {
    let l = match g {
        Grab::Pivot | Grab::Move => return CursorIcon::Move,
        Grab::Center(l) | Grab::Bounds(l) => l,
    };
    match (l.left || l.right, l.top || l.bottom) {
        (true, true) if l.left == l.top => CursorIcon::NwseResize,
        (true, true) => CursorIcon::NeswResize,
        (true, false) => CursorIcon::EwResize,
        _ => CursorIcon::NsResize,
    }
}
