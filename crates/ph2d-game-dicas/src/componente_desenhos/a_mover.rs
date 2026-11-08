//! Desenhos das dicas do Add Component (ver `componente.rs`).

use crate::{CHAO, boneco};
use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

const fn r(x: f32, y: f32, w: f32, h: f32, fill: C) -> Mark {
    Mark::Rect {
        at: [x, y],
        size: [w, h],
        fill,
        round: 0.02,
    }
}
const fn d(x: f32, y: f32, r: f32, fill: C) -> Mark {
    Mark::Disc { c: [x, y], r, fill }
}
const fn l(ax: f32, ay: f32, bx: f32, by: f32, stroke: C) -> Mark {
    Mark::Line {
        a: [ax, ay],
        b: [bx, by],
        stroke,
        arrow: false,
    }
}
const fn s(ax: f32, ay: f32, bx: f32, by: f32) -> Mark {
    Mark::Line {
        a: [ax, ay],
        b: [bx, by],
        stroke: C::Text1,
        arrow: true,
    }
}
const fn tr(x: f32, y: f32, w: f32, h: f32, stroke: C) -> Mark {
    Mark::Dashed {
        at: [x, y],
        size: [w, h],
        stroke,
    }
}
/// O herói visto de CIMA (jogo top-down e navegação): disco + olho.
const fn de_cima(x: f32, y: f32) -> [Mark; 2] {
    [
        d(x, y, 0.12, C::Accent),
        d(x + 0.03, y - 0.04, 0.035, C::AccentFg),
    ]
}
/// A bandeira do objectivo: mastro e pano.
const fn bandeira(x: f32, y: f32) -> [Mark; 2] {
    [
        l(x, y, x, y + 0.34, C::Text1),
        r(x, y, 0.07, 0.13, C::Success),
    ]
}

/// Uma caixa a cair (seta para baixo, riscos de velocidade) e uma bola empurrada no chão.
pub const RIGID_BODY: &[Mark] = &[
    CHAO,
    l(0.45, 0.06, 0.45, 0.12, C::Text3),
    l(0.55, 0.06, 0.55, 0.12, C::Text3),
    r(0.42, 0.16, 0.16, 0.32, C::Info),
    s(0.5, 0.54, 0.5, 0.76),
    d(0.8, 0.72, 0.1, C::Warn),
    s(0.64, 0.72, 0.74, 0.72),
];

const TD: [Mark; 2] = de_cima(0.62, 0.5);
/// Visto de cima: as quatro setas do teclado e o herói a andar nas quatro direcções.
pub const TOP_DOWN_PLAYER: &[Mark] = &[
    r(0.12, 0.36, 0.05, 0.1, C::Text2),
    r(0.06, 0.48, 0.05, 0.1, C::Text2),
    r(0.12, 0.48, 0.05, 0.1, C::Text2),
    r(0.18, 0.48, 0.05, 0.1, C::Text2),
    TD[0],
    TD[1],
    s(0.7, 0.5, 0.86, 0.5),
    s(0.54, 0.5, 0.38, 0.5),
    s(0.62, 0.34, 0.62, 0.1),
    s(0.62, 0.66, 0.62, 0.88),
];

const ATIRA: [Mark; 2] = boneco(0.14, 0.62);
/// O herói atira uma pedra: ela voa sozinha num arco, cai e ressalta.
pub const PROJECTILE_MOTION: &[Mark] = &[
    CHAO,
    ATIRA[0],
    ATIRA[1],
    l(0.22, 0.4, 0.38, 0.18, C::Text3),
    l(0.38, 0.18, 0.56, 0.26, C::Text3),
    l(0.56, 0.26, 0.68, 0.66, C::Text3),
    d(0.72, 0.72, 0.08, C::Warn),
    s(0.76, 0.68, 0.86, 0.44),
];

/// Uma plataforma que segue um caminho desenhado de ponto em ponto.
pub const PATH_FOLLOW: &[Mark] = &[
    l(0.14, 0.72, 0.36, 0.28, C::Text3),
    l(0.36, 0.28, 0.66, 0.28, C::Text3),
    l(0.66, 0.28, 0.86, 0.66, C::Text3),
    d(0.14, 0.72, 0.05, C::Text2),
    d(0.36, 0.28, 0.05, C::Text2),
    d(0.66, 0.28, 0.05, C::Text2),
    d(0.86, 0.66, 0.05, C::Text2),
    r(0.44, 0.22, 0.14, 0.1, C::Success),
    s(0.46, 0.12, 0.6, 0.12),
];

/// A barra de vida do jogo entra num nó do Motion, que acende um brilho.
pub const MOTION_INPUT: &[Mark] = &[
    r(0.06, 0.42, 0.24, 0.14, C::Text3),
    r(0.06, 0.42, 0.15, 0.14, C::Danger),
    s(0.32, 0.49, 0.44, 0.49),
    Mark::Rect {
        at: [0.46, 0.28],
        size: [0.2, 0.42],
        fill: C::BorderStrong,
        round: 0.06,
    },
    d(0.46, 0.49, 0.045, C::PortValue),
    d(0.66, 0.49, 0.045, C::PortValue),
    s(0.69, 0.49, 0.76, 0.49),
    d(0.85, 0.49, 0.14, C::Warn),
    d(0.85, 0.49, 0.06, C::AccentFg),
];

/// O inimigo à beira do chão lança uma linha para baixo, não acha chão e dá meia-volta.
pub const RAY_SENSOR: &[Mark] = &[
    r(0.0, 0.82, 0.66, 0.18, C::BorderStrong),
    Mark::Capsule {
        c: [0.5, 0.62],
        half_h: 0.08,
        r: 0.12,
        fill: C::Danger,
    },
    d(0.525, 0.55, 0.035, C::AccentFg),
    l(0.54, 0.66, 0.74, 0.9, C::Info),
    d(0.74, 0.9, 0.04, C::Warn),
    s(0.44, 0.3, 0.26, 0.3),
];

const NA: [Mark; 2] = de_cima(0.16, 0.36);
const NA_FIM: [Mark; 2] = bandeira(0.82, 0.24);
/// Visto de cima: o andador contorna a parede sozinho até à bandeira.
pub const NAV_AGENT: &[Mark] = &[
    r(0.46, 0.1, 0.06, 0.58, C::BorderStrong),
    NA[0],
    NA[1],
    l(0.24, 0.44, 0.49, 0.82, C::Text1),
    s(0.49, 0.82, 0.78, 0.5),
    NA_FIM[0],
    NA_FIM[1],
];

const NR: [Mark; 2] = de_cima(0.3, 0.62);
/// Visto de cima: o chão por onde se anda pintado e contornado, com dois buracos de obstáculo.
pub const NAV_REGION: &[Mark] = &[
    Mark::Rect {
        at: [0.1, 0.14],
        size: [0.8, 0.72],
        fill: C::SuccessSoft,
        round: 0.06,
    },
    tr(0.1, 0.14, 0.8, 0.72, C::Success),
    r(0.44, 0.24, 0.12, 0.26, C::BorderStrong),
    r(0.7, 0.54, 0.1, 0.2, C::BorderStrong),
    NR[0],
    NR[1],
];

const NC: [Mark; 2] = de_cima(0.14, 0.62);
const NC_FIM: [Mark; 2] = bandeira(0.84, 0.36);
/// Visto de cima: uma poça de lama no meio, e o caminho que a contorna por cima.
pub const NAV_COST_AREA: &[Mark] = &[
    Mark::Rect {
        at: [0.36, 0.42],
        size: [0.28, 0.44],
        fill: C::WarnSoft,
        round: 0.12,
    },
    d(0.44, 0.6, 0.06, C::Warn),
    d(0.55, 0.72, 0.05, C::Warn),
    NC[0],
    NC[1],
    l(0.2, 0.52, 0.36, 0.24, C::Text1),
    l(0.36, 0.24, 0.64, 0.24, C::Text1),
    s(0.64, 0.24, 0.79, 0.52),
    NC_FIM[0],
    NC_FIM[1],
];

const NL: [Mark; 2] = boneco(0.28, 0.62);
/// De lado: uma escada liga o chão ao patamar alto, e o herói sobe por ela.
pub const NAV_LINK: &[Mark] = &[
    CHAO,
    r(0.6, 0.34, 0.3, 0.1, C::BorderStrong),
    l(0.62, 0.34, 0.62, 0.82, C::Warn),
    l(0.7, 0.34, 0.7, 0.82, C::Warn),
    l(0.62, 0.48, 0.7, 0.48, C::Warn),
    l(0.62, 0.62, 0.7, 0.62, C::Warn),
    l(0.62, 0.76, 0.7, 0.76, C::Warn),
    NL[0],
    NL[1],
    s(0.5, 0.72, 0.5, 0.28),
];

/// Duas regras numa lista: «ouviu este sinal» seta «faz isto».
pub const SIGNAL_ACTIONS: &[Mark] = &[
    d(0.18, 0.3, 0.1, C::PortSignal),
    s(0.26, 0.3, 0.42, 0.3),
    Mark::Rect {
        at: [0.46, 0.21],
        size: [0.38, 0.18],
        fill: C::Accent,
        round: 0.06,
    },
    d(0.18, 0.7, 0.1, C::PortSignal),
    s(0.26, 0.7, 0.42, 0.7),
    Mark::Rect {
        at: [0.46, 0.61],
        size: [0.38, 0.18],
        fill: C::Success,
        round: 0.06,
    },
];

/// Uma tecla premida solta um sinal que chega a duas outras peças.
pub const SIGNAL_ON_ACTION: &[Mark] = &[
    s(0.16, 0.12, 0.16, 0.3),
    Mark::Rect {
        at: [0.08, 0.36],
        size: [0.16, 0.3],
        fill: C::Text2,
        round: 0.06,
    },
    s(0.28, 0.51, 0.42, 0.51),
    d(0.5, 0.51, 0.1, C::PortSignal),
    s(0.57, 0.44, 0.72, 0.28),
    s(0.57, 0.58, 0.72, 0.74),
    r(0.76, 0.16, 0.1, 0.18, C::Accent),
    r(0.76, 0.66, 0.1, 0.18, C::Success),
];

/// Três moedas viram o número 3, feito de blocos.
pub const COUNTER: &[Mark] = &[
    d(0.12, 0.5, 0.11, C::Warn),
    d(0.25, 0.5, 0.11, C::Warn),
    d(0.38, 0.5, 0.11, C::Warn),
    s(0.46, 0.5, 0.56, 0.5),
    r(0.62, 0.2, 0.16, 0.1, C::Text1),
    r(0.62, 0.45, 0.16, 0.1, C::Text1),
    r(0.62, 0.7, 0.16, 0.1, C::Text1),
    r(0.73, 0.2, 0.05, 0.6, C::Text1),
];

const VENCE: [Mark; 2] = bandeira(0.8, 0.22);
/// As moedas chegam à marca da meta e a bandeira da vitória sobe.
pub const COUNTER_WATCH: &[Mark] = &[
    d(0.1, 0.5, 0.09, C::Warn),
    d(0.2, 0.5, 0.09, C::Warn),
    d(0.3, 0.5, 0.09, C::Warn),
    d(0.4, 0.5, 0.09, C::Warn),
    l(0.48, 0.24, 0.48, 0.76, C::Success),
    s(0.54, 0.5, 0.72, 0.5),
    VENCE[0],
    VENCE[1],
];

/// Um relógio; passada a espera, solta um sinal, e outro, e outro.
pub const TIMERS: &[Mark] = &[
    d(0.28, 0.5, 0.34, C::Text3),
    l(0.28, 0.5, 0.28, 0.26, C::Text1),
    l(0.28, 0.5, 0.36, 0.5, C::Text1),
    d(0.28, 0.5, 0.04, C::Text1),
    s(0.48, 0.5, 0.62, 0.5),
    d(0.7, 0.5, 0.1, C::PortSignal),
    d(0.8, 0.5, 0.07, C::PortSignal),
    d(0.88, 0.5, 0.05, C::PortSignal),
];

/// O objecto encolhe e esmaece com o tempo até sobrar só o contorno de onde estava.
pub const LIFETIME: &[Mark] = &[
    r(0.1, 0.24, 0.14, 0.4, C::Accent),
    r(0.36, 0.3, 0.1, 0.28, C::Text2),
    r(0.58, 0.36, 0.06, 0.16, C::Text3),
    tr(0.76, 0.24, 0.14, 0.4, C::Text3),
    s(0.1, 0.82, 0.9, 0.82),
];

/// Uma bala sai do ecrã e, lá fora, é riscada com um X.
pub const DESTROY_OUTSIDE: &[Mark] = &[
    Mark::Rect {
        at: [0.08, 0.14],
        size: [0.56, 0.72],
        fill: C::BorderStrong,
        round: 0.04,
    },
    d(0.36, 0.5, 0.09, C::Warn),
    s(0.42, 0.5, 0.74, 0.5),
    d(0.82, 0.5, 0.09, C::Text3),
    l(0.77, 0.36, 0.87, 0.64, C::Danger),
    l(0.87, 0.36, 0.77, 0.64, C::Danger),
];

/// Ouve um sinal e a fábrica solta cópias iguais de uma moeda.
pub const FACTORY: &[Mark] = &[
    CHAO,
    d(0.2, 0.2, 0.09, C::PortSignal),
    Mark::Rect {
        at: [0.1, 0.36],
        size: [0.2, 0.46],
        fill: C::Text2,
        round: 0.04,
    },
    r(0.24, 0.54, 0.06, 0.28, C::Text1),
    d(0.44, 0.66, 0.1, C::Warn),
    d(0.6, 0.66, 0.1, C::Warn),
    d(0.76, 0.66, 0.1, C::Warn),
    s(0.4, 0.38, 0.84, 0.38),
];

/// A arma dispara balas a intervalos iguais; embaixo, as balas que restam (três de cinco).
pub const WEAPON_FIRE: &[Mark] = &[
    r(0.08, 0.38, 0.22, 0.12, C::Text2),
    r(0.1, 0.48, 0.06, 0.22, C::Text2),
    r(0.38, 0.41, 0.05, 0.06, C::Warn),
    r(0.52, 0.41, 0.05, 0.06, C::Warn),
    r(0.66, 0.41, 0.05, 0.06, C::Warn),
    s(0.76, 0.44, 0.9, 0.44),
    r(0.44, 0.66, 0.04, 0.16, C::Warn),
    r(0.52, 0.66, 0.04, 0.16, C::Warn),
    r(0.6, 0.66, 0.04, 0.16, C::Warn),
    r(0.68, 0.66, 0.04, 0.16, C::Text3),
    r(0.76, 0.66, 0.04, 0.16, C::Text3),
];

const FERIDO: [Mark; 2] = boneco(0.36, 0.56);
/// O herói toca nos espinhos e é atirado para trás.
pub const DAMAGE: &[Mark] = &[
    CHAO,
    l(0.5, 0.82, 0.56, 0.62, C::Danger),
    l(0.56, 0.62, 0.62, 0.82, C::Danger),
    l(0.62, 0.82, 0.68, 0.62, C::Danger),
    l(0.68, 0.62, 0.74, 0.82, C::Danger),
    FERIDO[0],
    FERIDO[1],
    s(0.28, 0.34, 0.12, 0.18),
];

const VIVO: [Mark; 2] = boneco(0.22, 0.62);
/// Três corações, um já vazio; quando acabam, o herói fica deitado.
pub const HEALTH: &[Mark] = &[
    CHAO,
    d(0.14, 0.18, 0.08, C::Danger),
    d(0.24, 0.18, 0.08, C::Danger),
    d(0.34, 0.18, 0.08, C::Text3),
    VIVO[0],
    VIVO[1],
    s(0.34, 0.64, 0.58, 0.64),
    Mark::Rect {
        at: [0.64, 0.68],
        size: [0.24, 0.14],
        fill: C::Text3,
        round: 0.07,
    },
];
