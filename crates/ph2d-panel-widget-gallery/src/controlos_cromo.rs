//! A tabela dos controlos ([`super`]) das peças do cromo (`cards_cromo`).

use super::{Control, opt, seg};
use crate::ids as g;
use crate::state::Pick;
use ph2d_editor_core::IconId;

pub(super) const CROMO: &[Control] = &[
    seg(
        g::BAR_MOVE,
        Pick::BarTool,
        0,
        IconId::Move,
        "panel.widget_gallery.cromo.tip.move",
    ),
    seg(
        g::BAR_ROTATE,
        Pick::BarTool,
        1,
        IconId::Rotate,
        "panel.widget_gallery.cromo.tip.rotate",
    ),
    seg(
        g::BAR_TRANSFORM,
        Pick::BarTool,
        2,
        IconId::Transform,
        "panel.widget_gallery.cromo.tip.transform",
    ),
    opt(
        g::LIST_COPY,
        Pick::ListRow,
        0,
        "panel.widget_gallery.cromo.copy",
    ),
    opt(
        g::LIST_DUPLICATE,
        Pick::ListRow,
        1,
        "panel.widget_gallery.cromo.duplicate",
    ),
    opt(
        g::LIST_DELETE,
        Pick::ListRow,
        2,
        "panel.widget_gallery.cromo.delete",
    ),
];
