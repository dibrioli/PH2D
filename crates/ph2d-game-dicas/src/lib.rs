//! As ilustrações das dicas do menu Add (`05_N1_plano.md` §4): desenhos em DADOS na moldura 2:1
//! (x para a direita, y para baixo, de 0 a 1; raios em fracção da altura), cores do tema.

pub mod add;
pub mod componente;
mod componente_desenhos;

use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

pub(crate) const CHAO: Mark = Mark::Rect {
    at: [0.0, 0.82],
    size: [1.0, 0.18],
    fill: C::BorderStrong,
    round: 0.0,
};

/// O boneco: corpo em cápsula, olho, sobre o chão em `x`.
pub(crate) const fn boneco(x: f32, y: f32) -> [Mark; 2] {
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

/// Uma curva com os pontos e uma alça: o desenho vetorial.
pub const VECTOR: &[Mark] = &[
    Mark::Line {
        a: [0.15, 0.75],
        b: [0.38, 0.3],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.38, 0.3],
        b: [0.62, 0.62],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.62, 0.62],
        b: [0.85, 0.25],
        stroke: C::Accent,
        arrow: false,
    },
    Mark::Line {
        a: [0.28, 0.18],
        b: [0.48, 0.42],
        stroke: C::Text3,
        arrow: false,
    },
    Mark::Disc {
        c: [0.15, 0.75],
        r: 0.05,
        fill: C::Text1,
    },
    Mark::Disc {
        c: [0.38, 0.3],
        r: 0.05,
        fill: C::Text1,
    },
    Mark::Disc {
        c: [0.62, 0.62],
        r: 0.05,
        fill: C::Text1,
    },
    Mark::Disc {
        c: [0.85, 0.25],
        r: 0.05,
        fill: C::Text1,
    },
];

/// Um T grande e as linhas de texto ao lado.
pub const TEXT: &[Mark] = &[
    Mark::Rect {
        at: [0.12, 0.2],
        size: [0.22, 0.1],
        fill: C::Text1,
        round: 0.0,
    },
    Mark::Rect {
        at: [0.2, 0.2],
        size: [0.06, 0.6],
        fill: C::Text1,
        round: 0.0,
    },
    Mark::Rect {
        at: [0.44, 0.24],
        size: [0.44, 0.08],
        fill: C::Accent,
        round: 0.04,
    },
    Mark::Rect {
        at: [0.44, 0.44],
        size: [0.36, 0.08],
        fill: C::Text3,
        round: 0.04,
    },
    Mark::Rect {
        at: [0.44, 0.64],
        size: [0.4, 0.08],
        fill: C::Text3,
        round: 0.04,
    },
];

/// A bola desenhada quadro a quadro: os quadros de antes esmaecidos (a pele de cebola).
pub const FLIP: &[Mark] = &[
    CHAO,
    Mark::Disc {
        c: [0.2, 0.3],
        r: 0.1,
        fill: C::Text3,
    },
    Mark::Disc {
        c: [0.4, 0.55],
        r: 0.1,
        fill: C::Text3,
    },
    Mark::Disc {
        c: [0.6, 0.7],
        r: 0.1,
        fill: C::Text2,
    },
    Mark::Disc {
        c: [0.8, 0.45],
        r: 0.1,
        fill: C::Accent,
    },
    Mark::Line {
        a: [0.62, 0.9],
        b: [0.86, 0.9],
        stroke: C::Text1,
        arrow: true,
    },
];

/// Os ossos de um braço: ombro, cotovelo, mão, e o desenho que os segue.
pub const SKELETON: &[Mark] = &[
    Mark::Capsule {
        c: [0.25, 0.5],
        half_h: 0.18,
        r: 0.12,
        fill: C::Info,
    },
    Mark::Line {
        a: [0.25, 0.32],
        b: [0.5, 0.45],
        stroke: C::Text1,
        arrow: false,
    },
    Mark::Line {
        a: [0.5, 0.45],
        b: [0.72, 0.25],
        stroke: C::Text1,
        arrow: false,
    },
    Mark::Disc {
        c: [0.25, 0.32],
        r: 0.05,
        fill: C::Accent,
    },
    Mark::Disc {
        c: [0.5, 0.45],
        r: 0.05,
        fill: C::Accent,
    },
    Mark::Disc {
        c: [0.72, 0.25],
        r: 0.05,
        fill: C::Accent,
    },
    Mark::Line {
        a: [0.8, 0.5],
        b: [0.74, 0.32],
        stroke: C::Text3,
        arrow: true,
    },
];

/// Formas que caem e quicam sozinhas: a simulação.
pub const MOTION: &[Mark] = &[
    CHAO,
    Mark::Disc {
        c: [0.22, 0.25],
        r: 0.08,
        fill: C::Warn,
    },
    Mark::Rect {
        at: [0.44, 0.14],
        size: [0.1, 0.2],
        fill: C::Success,
        round: 0.02,
    },
    Mark::Disc {
        c: [0.72, 0.6],
        r: 0.1,
        fill: C::Accent,
    },
    Mark::Line {
        a: [0.22, 0.38],
        b: [0.22, 0.7],
        stroke: C::Text1,
        arrow: true,
    },
    Mark::Line {
        a: [0.49, 0.42],
        b: [0.49, 0.7],
        stroke: C::Text1,
        arrow: true,
    },
    Mark::Line {
        a: [0.78, 0.56],
        b: [0.88, 0.3],
        stroke: C::Text1,
        arrow: true,
    },
];

/// Uma figura feita de quadradinhos: cada um é um pixel.
const fn pixel(x: u8, y: u8, fill: C) -> Mark {
    let s = 0.1;
    Mark::Rect {
        at: [0.3 + x as f32 * s * 0.5, 0.15 + y as f32 * s],
        size: [s * 0.5, s],
        fill,
        round: 0.0,
    }
}
pub const PIXEL_SPRITE: &[Mark] = &[
    pixel(2, 0, C::Warn),
    pixel(3, 0, C::Warn),
    pixel(4, 0, C::Warn),
    pixel(5, 0, C::Warn),
    pixel(1, 1, C::Warn),
    pixel(2, 1, C::AccentFg),
    pixel(3, 1, C::Warn),
    pixel(4, 1, C::Warn),
    pixel(5, 1, C::AccentFg),
    pixel(6, 1, C::Warn),
    pixel(1, 2, C::Warn),
    pixel(2, 2, C::Warn),
    pixel(3, 2, C::Warn),
    pixel(4, 2, C::Warn),
    pixel(5, 2, C::Warn),
    pixel(6, 2, C::Warn),
    pixel(2, 3, C::Accent),
    pixel(3, 3, C::Accent),
    pixel(4, 3, C::Accent),
    pixel(5, 3, C::Accent),
    pixel(1, 4, C::Accent),
    pixel(2, 4, C::Accent),
    pixel(3, 4, C::Accent),
    pixel(4, 4, C::Accent),
    pixel(5, 4, C::Accent),
    pixel(6, 4, C::Accent),
    pixel(2, 5, C::Text1),
    pixel(5, 5, C::Text1),
];

/// A lâmpada, os raios, e o bloco que faz sombra.
pub const LIGHT: &[Mark] = &[
    CHAO,
    Mark::Rect {
        at: [0.56, 0.62],
        size: [0.34, 0.2],
        fill: C::Text3,
        round: 0.0,
    },
    Mark::Rect {
        at: [0.5, 0.5],
        size: [0.1, 0.32],
        fill: C::BorderStrong,
        round: 0.0,
    },
    Mark::Disc {
        c: [0.2, 0.3],
        r: 0.12,
        fill: C::Warn,
    },
    Mark::Line {
        a: [0.32, 0.36],
        b: [0.48, 0.5],
        stroke: C::Warn,
        arrow: false,
    },
    Mark::Line {
        a: [0.2, 0.46],
        b: [0.2, 0.74],
        stroke: C::Warn,
        arrow: false,
    },
    Mark::Line {
        a: [0.34, 0.26],
        b: [0.56, 0.2],
        stroke: C::Warn,
        arrow: false,
    },
];

/// O que se fez uma vez, gravado (o ponto vermelho) e tocado de novo (os mesmos traços repetidos).
pub const MACRO: &[Mark] = &[
    Mark::Disc {
        c: [0.14, 0.24],
        r: 0.08,
        fill: C::Danger,
    },
    Mark::Line {
        a: [0.3, 0.3],
        b: [0.5, 0.3],
        stroke: C::Accent,
        arrow: true,
    },
    Mark::Line {
        a: [0.3, 0.52],
        b: [0.5, 0.52],
        stroke: C::Text3,
        arrow: true,
    },
    Mark::Line {
        a: [0.3, 0.74],
        b: [0.5, 0.74],
        stroke: C::Text3,
        arrow: true,
    },
    Mark::Rect {
        at: [0.62, 0.22],
        size: [0.16, 0.16],
        fill: C::Success,
        round: 0.03,
    },
    Mark::Rect {
        at: [0.62, 0.44],
        size: [0.16, 0.16],
        fill: C::Success,
        round: 0.03,
    },
    Mark::Rect {
        at: [0.62, 0.66],
        size: [0.16, 0.16],
        fill: C::Success,
        round: 0.03,
    },
];
