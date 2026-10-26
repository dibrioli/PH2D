//! As ilustrações das dicas do menu Add (`05_N1_plano.md` §4): desenhos em DADOS na moldura 2:1
//! (x para a direita, y para baixo, de 0 a 1; raios em fracção da altura), cores do tema.

use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

const CHAO: Mark = Mark::Rect {
    at: [0.0, 0.82],
    size: [1.0, 0.18],
    fill: C::BorderStrong,
    round: 0.0,
};

/// O boneco: corpo em cápsula, olho, sobre o chão em `x`.
const fn boneco(x: f32, y: f32) -> [Mark; 2] {
    [
        Mark::Capsule {
            c: [x, y],
            half_h: 0.08,
            r: 0.12,
            fill: C::Accent,
        },
        Mark::Disc {
            c: [x + 0.025, y - 0.07],
            r: 0.035,
            fill: C::AccentFg,
        },
    ]
}

pub const SPRITE: &[Mark] = &[
    CHAO,
    Mark::Rect {
        at: [0.36, 0.18],
        size: [0.28, 0.64],
        fill: C::Success,
        round: 0.04,
    },
    Mark::Disc {
        c: [0.56, 0.36],
        r: 0.08,
        fill: C::Warn,
    },
    Mark::Line {
        a: [0.39, 0.72],
        b: [0.61, 0.72],
        stroke: C::Text1,
        arrow: false,
    },
];

const B0: [Mark; 2] = boneco(0.2, 0.5);
const B1: [Mark; 2] = boneco(0.5, 0.42);
const B2: [Mark; 2] = boneco(0.8, 0.5);
pub const ANIMATED_SPRITE: &[Mark] = &[
    Mark::Dashed {
        at: [0.08, 0.15],
        size: [0.24, 0.6],
        stroke: C::Text3,
    },
    Mark::Dashed {
        at: [0.38, 0.15],
        size: [0.24, 0.6],
        stroke: C::Text3,
    },
    Mark::Dashed {
        at: [0.68, 0.15],
        size: [0.24, 0.6],
        stroke: C::Text3,
    },
    B0[0],
    B0[1],
    B1[0],
    B1[1],
    B2[0],
    B2[1],
    Mark::Line {
        a: [0.12, 0.88],
        b: [0.88, 0.88],
        stroke: C::Text1,
        arrow: true,
    },
];

pub const STATIC_BODY: &[Mark] = &[
    CHAO,
    Mark::Rect {
        at: [0.55, 0.45],
        size: [0.32, 0.1],
        fill: C::BorderStrong,
        round: 0.0,
    },
    Mark::Rect {
        at: [0.18, 0.6],
        size: [0.12, 0.22],
        fill: C::Info,
        round: 0.02,
    },
    Mark::Line {
        a: [0.24, 0.08],
        b: [0.24, 0.5],
        stroke: C::Text1,
        arrow: true,
    },
    Mark::Line {
        a: [0.71, 0.1],
        b: [0.71, 0.36],
        stroke: C::Text1,
        arrow: true,
    },
];

const HEROI: [Mark; 2] = boneco(0.22, 0.62);
pub const CHARACTER: &[Mark] = &[
    CHAO,
    Mark::Rect {
        at: [0.6, 0.42],
        size: [0.3, 0.1],
        fill: C::BorderStrong,
        round: 0.0,
    },
    HEROI[0],
    HEROI[1],
    Mark::Line {
        a: [0.3, 0.42],
        b: [0.45, 0.14],
        stroke: C::Text1,
        arrow: false,
    },
    Mark::Line {
        a: [0.45, 0.14],
        b: [0.66, 0.34],
        stroke: C::Text1,
        arrow: true,
    },
];

const ANDA: [Mark; 2] = boneco(0.3, 0.62);
pub const COLLISION_SHAPE: &[Mark] = &[
    CHAO,
    Mark::Dashed {
        at: [0.72, 0.18],
        size: [0.1, 0.64],
        stroke: C::Danger,
    },
    ANDA[0],
    ANDA[1],
    Mark::Line {
        a: [0.42, 0.55],
        b: [0.66, 0.55],
        stroke: C::Text1,
        arrow: true,
    },
];

/// Os montes ao longe andam pouco (seta curta) enquanto o boneco e o chão passam (seta longa).
const PERTO: [Mark; 2] = boneco(0.78, 0.62);
pub const BACKGROUND_LAYER: &[Mark] = &[
    Mark::Disc {
        c: [0.2, 0.74],
        r: 0.34,
        fill: C::Info,
    },
    Mark::Disc {
        c: [0.5, 0.8],
        r: 0.26,
        fill: C::Info,
    },
    CHAO,
    PERTO[0],
    PERTO[1],
    Mark::Line {
        a: [0.14, 0.22],
        b: [0.26, 0.22],
        stroke: C::Text1,
        arrow: true,
    },
    Mark::Line {
        a: [0.56, 0.22],
        b: [0.94, 0.22],
        stroke: C::Text1,
        arrow: true,
    },
];
