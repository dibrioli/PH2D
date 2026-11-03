//! A tabela dos controlos ([`super`]) das amostras do Rive (doc `CartoesUI/08` §1.4).

use super::{Act, Control, Kind, add_of, icon, minus, opt, plain, seg};
use crate::ids as g;
use crate::state::{Flag, Pick};
use ph2d_editor_core::IconId;

/// As linhas de entrada da máquina de estados e o tipo de cada uma.
pub const INPUT_ROWS: [Flag; 3] = [Flag::Input0, Flag::Input1, Flag::Input2];
pub const INPUT_TYPES: [Pick; 3] = [Pick::Input0, Pick::Input1, Pick::Input2];
const INPUT_CHECKS: [Flag; 3] = [Flag::InputBool0, Flag::InputBool1, Flag::InputBool2];
pub const STOP_ROWS: [Flag; 3] = [Flag::Stop0, Flag::Stop1, Flag::Stop2];
/// A primeira amostra de cor das paragens (`controlos::SWATCHES`).
pub const STOP_SWATCH: usize = 10;
const SEM_AMOSTRA: usize = usize::MAX;

const fn entrada(i: usize) -> [Control; 3] {
    let r = g::INPUTS[i];
    [
        plain(r.check, Kind::Toggle(INPUT_CHECKS[i])),
        plain(r.fire, Kind::Act(Act::Fire(i))),
        minus_sem_cor(r.remove, INPUT_ROWS[i]),
    ]
}

const fn minus_sem_cor(id: ph2d_a11y::NodeId, f: Flag) -> Control {
    icon(
        id,
        Kind::Remove(f, SEM_AMOSTRA),
        IconId::Minus,
        "panel.widget_gallery.rive.tip.input_remove",
    )
}

const fn paragem(i: usize) -> [Control; 3] {
    let r = g::STOPS[i];
    [
        plain(r.handle, Kind::Pick(Pick::GradStop, i as u8)),
        plain(r.swatch, Kind::Swatch(STOP_SWATCH + i)),
        minus(
            r.remove,
            STOP_ROWS[i],
            STOP_SWATCH + i,
            "panel.widget_gallery.rive.tip.stop_remove",
        ),
    ]
}

const E0: [Control; 3] = entrada(0);
const E1: [Control; 3] = entrada(1);
const E2: [Control; 3] = entrada(2);
const P0: [Control; 3] = paragem(0);
const P1: [Control; 3] = paragem(1);
const P2: [Control; 3] = paragem(2);

/// As linhas geradas por família.
pub(super) fn linhas() -> impl Iterator<Item = &'static Control> {
    E0.iter()
        .chain(&E1)
        .chain(&E2)
        .chain(&P0)
        .chain(&P1)
        .chain(&P2)
}

pub(super) const RIVE: &[Control] = &[
    // Interpolation.
    seg(
        g::IN_LINEAR,
        Pick::Interp,
        0,
        IconId::CardInterpLinear,
        "panel.widget_gallery.rive.tip.linear",
    ),
    seg(
        g::IN_CUBIC,
        Pick::Interp,
        1,
        IconId::CardInterpCubic,
        "panel.widget_gallery.rive.tip.cubic",
    ),
    seg(
        g::IN_HOLD,
        Pick::Interp,
        2,
        IconId::CardInterpHold,
        "panel.widget_gallery.rive.tip.hold",
    ),
    // State machine inputs.
    icon(
        g::SM_ADD,
        Kind::Box,
        IconId::Add,
        "panel.widget_gallery.rive.tip.input_add",
    ),
    add_of(
        g::SM_ADD_NUMBER,
        INPUT_ROWS,
        INPUT_TYPES,
        0,
        "panel.widget_gallery.rive.input.number",
    ),
    add_of(
        g::SM_ADD_BOOLEAN,
        INPUT_ROWS,
        INPUT_TYPES,
        1,
        "panel.widget_gallery.rive.input.boolean",
    ),
    add_of(
        g::SM_ADD_TRIGGER,
        INPUT_ROWS,
        INPUT_TYPES,
        2,
        "panel.widget_gallery.rive.input.trigger",
    ),
    // Trim path.
    plain(g::TR_MODE, Kind::Box),
    opt(
        g::TR_SEQUENTIAL,
        Pick::TrimMode,
        0,
        "panel.widget_gallery.rive.trim.sequential",
    ),
    opt(
        g::TR_SYNCED,
        Pick::TrimMode,
        1,
        "panel.widget_gallery.rive.trim.synced",
    ),
    // Solo.
    plain(g::SOLO[0], Kind::Pick(Pick::Solo, 0)),
    plain(g::SOLO[1], Kind::Pick(Pick::Solo, 1)),
    plain(g::SOLO[2], Kind::Pick(Pick::Solo, 2)),
    // Gradient.
    seg(
        g::GR_SOLID,
        Pick::FillType,
        0,
        IconId::PixelRectFilled,
        "panel.widget_gallery.rive.tip.solid",
    ),
    seg(
        g::GR_LINEAR,
        Pick::FillType,
        1,
        IconId::PixelGradient,
        "panel.widget_gallery.rive.tip.linear_gradient",
    ),
    seg(
        g::GR_RADIAL,
        Pick::FillType,
        2,
        IconId::Circle,
        "panel.widget_gallery.rive.tip.radial_gradient",
    ),
    icon(
        g::GR_ADD,
        Kind::Add(STOP_ROWS),
        IconId::Add,
        "panel.widget_gallery.rive.tip.stop_add",
    ),
];
