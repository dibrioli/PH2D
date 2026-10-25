//! As ilustrações das dicas dos objectos de jogo desta família no menu Add
//! (`docs/GameEngineFuncional/05_N1_plano.md` §4): moldura 2:1, x para a direita, y para baixo, de 0 a 1.

use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

const CHAO: Mark = Mark::Rect {
    at: [0.0, 0.82],
    size: [1.0, 0.18],
    fill: C::BorderStrong,
    round: 0.0,
};

pub const CAMERA: &[Mark] = &[
    CHAO,
    Mark::Capsule {
        c: [0.5, 0.62],
        half_h: 0.08,
        r: 0.12,
        fill: C::Accent,
    },
    Mark::Dashed {
        at: [0.25, 0.1],
        size: [0.5, 0.8],
        stroke: C::Text1,
    },
    Mark::Line {
        a: [0.8, 0.5],
        b: [0.95, 0.5],
        stroke: C::Text1,
        arrow: true,
    },
];

pub const BODY: &[Mark] = &[
    CHAO,
    Mark::Rect {
        at: [0.3, 0.12],
        size: [0.12, 0.22],
        fill: C::Info,
        round: 0.02,
    },
    Mark::Line {
        a: [0.36, 0.4],
        b: [0.36, 0.7],
        stroke: C::Text1,
        arrow: true,
    },
    Mark::Disc {
        c: [0.66, 0.7],
        r: 0.12,
        fill: C::Info,
    },
];

pub const SOUND: &[Mark] = &[
    Mark::Rect {
        at: [0.3, 0.38],
        size: [0.08, 0.24],
        fill: C::Text1,
        round: 0.02,
    },
    Mark::Line {
        a: [0.46, 0.3],
        b: [0.52, 0.2],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.48, 0.5],
        b: [0.6, 0.5],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.46, 0.7],
        b: [0.52, 0.8],
        stroke: C::Accent,
        arrow: false,
    },
];

pub const HUD: &[Mark] = &[
    Mark::Dashed {
        at: [0.05, 0.08],
        size: [0.9, 0.84],
        stroke: C::Text3,
    },
    Mark::Disc {
        c: [0.12, 0.2],
        r: 0.05,
        fill: C::Danger,
    },
    Mark::Disc {
        c: [0.18, 0.2],
        r: 0.05,
        fill: C::Danger,
    },
    Mark::Disc {
        c: [0.24, 0.2],
        r: 0.05,
        fill: C::Danger,
    },
    Mark::Disc {
        c: [0.78, 0.2],
        r: 0.05,
        fill: C::Warn,
    },
    Mark::Rect {
        at: [0.82, 0.16],
        size: [0.08, 0.08],
        fill: C::Text1,
        round: 0.02,
    },
    CHAO,
];
