//! Desenhos das dicas do Add Component (ver `componente.rs`).

use crate::{CHAO, boneco};
use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

const fn r(x: f32, y: f32, w: f32, h: f32, fill: C) -> Mark {
    Mark::Rect {
        at: [x, y],
        size: [w, h],
        fill,
        round: 0.03,
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
const fn a(ax: f32, ay: f32, bx: f32, by: f32, stroke: C) -> Mark {
    Mark::Line {
        a: [ax, ay],
        b: [bx, by],
        stroke,
        arrow: true,
    }
}
const fn t(x: f32, y: f32, w: f32, h: f32, stroke: C) -> Mark {
    Mark::Dashed {
        at: [x, y],
        size: [w, h],
        stroke,
    }
}
/// O losango de uma chave: duas pontas de seta costas com costas.
const fn chave(x: f32, y: f32) -> [Mark; 2] {
    [a(x, y, x, y - 0.07, C::Warn), a(x, y, x, y + 0.07, C::Warn)]
}

/// A tela tracejada com o placar, os corações e o botão presos em cima; a cena atrás.
const H_CANVAS: [Mark; 2] = boneco(0.5, 0.62);
pub const UI_CANVAS: &[Mark] = &[
    d(0.24, 0.84, 0.3, C::Info),
    CHAO,
    H_CANVAS[0],
    H_CANVAS[1],
    t(0.05, 0.06, 0.9, 0.84, C::Text3),
    r(0.1, 0.12, 0.18, 0.1, C::Warn),
    d(0.36, 0.17, 0.05, C::Danger),
    d(0.42, 0.17, 0.05, C::Danger),
    r(0.8, 0.11, 0.1, 0.13, C::Accent),
];

/// A tela com os algarismos do placar em cima e uma linha de mensagem.
pub const UI_LABEL: &[Mark] = &[
    t(0.06, 0.08, 0.88, 0.8, C::Text3),
    r(0.36, 0.2, 0.06, 0.24, C::Text1),
    r(0.45, 0.2, 0.06, 0.24, C::Text1),
    r(0.54, 0.2, 0.06, 0.24, C::Text1),
    r(0.63, 0.2, 0.06, 0.24, C::Text1),
    r(0.24, 0.6, 0.52, 0.09, C::Accent),
];

/// O botão, a seta do clique sobre ele, e o sinal que sai dele.
pub const UI_BUTTON: &[Mark] = &[
    Mark::Rect {
        at: [0.26, 0.3],
        size: [0.4, 0.32],
        fill: C::Accent,
        round: 0.16,
    },
    l(0.36, 0.46, 0.56, 0.46, C::AccentFg),
    a(0.62, 0.9, 0.5, 0.58, C::Text1),
    a(0.7, 0.4, 0.86, 0.22, C::Warn),
    a(0.71, 0.5, 0.88, 0.5, C::Warn),
];

/// O boneco com a barra de vida sobre a cabeça, meio cheia.
const H_VIDA: [Mark; 2] = boneco(0.5, 0.6);
pub const HEALTH_BAR: &[Mark] = &[
    CHAO,
    H_VIDA[0],
    H_VIDA[1],
    Mark::Rect {
        at: [0.3, 0.14],
        size: [0.4, 0.12],
        fill: C::Text3,
        round: 0.06,
    },
    Mark::Rect {
        at: [0.3, 0.14],
        size: [0.26, 0.12],
        fill: C::Success,
        round: 0.06,
    },
];

/// Dois pontos no boneco (cabeça e mão), cada um com a sua etiqueta de nome.
const H_PONTOS: [Mark; 2] = boneco(0.5, 0.6);
pub const NAMED_ANCHOR_LIST: &[Mark] = &[
    CHAO,
    H_PONTOS[0],
    H_PONTOS[1],
    d(0.5, 0.38, 0.05, C::Warn),
    d(0.57, 0.64, 0.05, C::Warn),
    l(0.48, 0.36, 0.36, 0.22, C::Text3),
    l(0.6, 0.64, 0.72, 0.64, C::Text3),
    r(0.16, 0.18, 0.18, 0.08, C::Text2),
    r(0.74, 0.6, 0.16, 0.08, C::Text2),
];

/// A espada sai do chão (fantasma tracejado) e vai presa ao ponto da mão do boneco.
const H_ESPADA: [Mark; 2] = boneco(0.38, 0.62);
pub const ANCHOR_MOUNT: &[Mark] = &[
    CHAO,
    t(0.8, 0.3, 0.025, 0.42, C::Text3),
    a(0.76, 0.46, 0.56, 0.46, C::Text1),
    H_ESPADA[0],
    H_ESPADA[1],
    r(0.475, 0.2, 0.025, 0.44, C::Text2),
    r(0.46, 0.6, 0.055, 0.05, C::Warn),
    d(0.45, 0.64, 0.04, C::Warn),
];

/// A janela: dentro dela os filhos aparecem; o que passa da borda fica só tracejado.
pub const CLIP_CHILDREN: &[Mark] = &[
    t(0.1, 0.4, 0.2, 0.16, C::Text3),
    t(0.7, 0.4, 0.2, 0.16, C::Text3),
    t(0.52, 0.06, 0.05, 0.14, C::Text3),
    r(0.3, 0.4, 0.4, 0.16, C::Success),
    r(0.52, 0.2, 0.05, 0.16, C::Warn),
    l(0.3, 0.2, 0.7, 0.2, C::Text1),
    l(0.7, 0.2, 0.7, 0.8, C::Text1),
    l(0.7, 0.8, 0.3, 0.8, C::Text1),
    l(0.3, 0.8, 0.3, 0.2, C::Text1),
];

/// O boneco anda; a moldura vem atrás dele (seta) e o mantém no meio.
const H_SEGUE: [Mark; 2] = boneco(0.64, 0.62);
pub const CAMERA_FOLLOW: &[Mark] = &[
    CHAO,
    t(0.12, 0.16, 0.3, 0.7, C::Text3),
    H_SEGUE[0],
    H_SEGUE[1],
    a(0.72, 0.5, 0.86, 0.5, C::Text1),
    t(0.48, 0.16, 0.32, 0.7, C::ObjCamera),
    a(0.2, 0.5, 0.44, 0.5, C::ObjCamera),
];

/// As bordas do nível (vermelhas): a moldura encosta nelas e não passa.
const H_BORDA: [Mark; 2] = boneco(0.22, 0.62);
pub const CAMERA_LIMITS: &[Mark] = &[
    CHAO,
    H_BORDA[0],
    H_BORDA[1],
    t(0.1, 0.16, 0.36, 0.7, C::ObjCamera),
    l(0.08, 0.1, 0.08, 0.9, C::Danger),
    l(0.92, 0.1, 0.92, 0.9, C::Danger),
    a(0.64, 0.5, 0.52, 0.5, C::Text3),
];

/// A moldura repetida fora do lugar e os riscos de tremor dos dois lados.
const H_TREME: [Mark; 2] = boneco(0.5, 0.6);
pub const CAMERA_SHAKE: &[Mark] = &[
    CHAO,
    H_TREME[0],
    H_TREME[1],
    t(0.32, 0.12, 0.4, 0.66, C::Text3),
    t(0.28, 0.18, 0.4, 0.66, C::ObjCamera),
    l(0.18, 0.32, 0.22, 0.44, C::Text1),
    l(0.22, 0.44, 0.18, 0.56, C::Text1),
    l(0.82, 0.32, 0.78, 0.44, C::Text1),
    l(0.78, 0.44, 0.82, 0.56, C::Text1),
];

/// A explosão manda o sinal (seta) e a moldura ao lado treme.
pub const SHAKE_EMITTER: &[Mark] = &[
    d(0.2, 0.5, 0.18, C::Danger),
    d(0.2, 0.5, 0.09, C::Warn),
    l(0.2, 0.12, 0.2, 0.24, C::Warn),
    l(0.2, 0.76, 0.2, 0.88, C::Warn),
    a(0.32, 0.5, 0.52, 0.5, C::Text1),
    t(0.6, 0.18, 0.28, 0.6, C::Text3),
    t(0.57, 0.24, 0.28, 0.6, C::ObjCamera),
];

/// As montanhas ao longe andam pouco (seta curta); as árvores da frente andam muito (seta longa).
pub const SCROLL_FACTOR: &[Mark] = &[
    d(0.22, 0.78, 0.36, C::Info),
    d(0.52, 0.82, 0.26, C::Info),
    CHAO,
    r(0.66, 0.46, 0.06, 0.36, C::Success),
    r(0.84, 0.54, 0.06, 0.28, C::Success),
    a(0.16, 0.18, 0.28, 0.18, C::Text1),
    a(0.5, 0.18, 0.92, 0.18, C::Text1),
];

/// O fundo anda até a linha vermelha e pára; além dela não há nada.
pub const SCROLL_LIMITS: &[Mark] = &[
    d(0.24, 0.82, 0.3, C::Info),
    d(0.5, 0.86, 0.22, C::Info),
    CHAO,
    t(0.76, 0.36, 0.14, 0.4, C::Text3),
    l(0.72, 0.14, 0.72, 0.82, C::Danger),
    a(0.2, 0.24, 0.68, 0.24, C::Text1),
];

/// Duas nuvens que deslizam sozinhas, cada uma com a sua seta.
pub const SCROLL_MOTION: &[Mark] = &[
    CHAO,
    d(0.2, 0.32, 0.1, C::Text3),
    d(0.28, 0.27, 0.14, C::Text3),
    d(0.36, 0.32, 0.1, C::Text3),
    a(0.45, 0.3, 0.58, 0.3, C::Text1),
    d(0.58, 0.6, 0.08, C::Text3),
    d(0.64, 0.56, 0.11, C::Text3),
    d(0.7, 0.6, 0.08, C::Text3),
    a(0.77, 0.58, 0.9, 0.58, C::Text1),
];

/// O mesmo pedaço de fundo três vezes lado a lado, sem fim, e a seta da câmera.
pub const SCROLL_REPEAT: &[Mark] = &[
    d(0.2, 0.82, 0.24, C::Info),
    d(0.5, 0.82, 0.24, C::Info),
    d(0.8, 0.82, 0.24, C::Info),
    CHAO,
    t(0.06, 0.24, 0.28, 0.58, C::Text3),
    t(0.36, 0.24, 0.28, 0.58, C::Text3),
    t(0.66, 0.24, 0.28, 0.58, C::Text3),
    a(0.2, 0.13, 0.8, 0.13, C::Text1),
];

/// A linha do tempo com três losangos de chave e a agulha.
const K0: [Mark; 2] = chave(0.2, 0.6);
const K1: [Mark; 2] = chave(0.45, 0.6);
const K2: [Mark; 2] = chave(0.8, 0.6);
pub const ANIMATION: &[Mark] = &[
    r(0.08, 0.58, 0.84, 0.04, C::Text3),
    K0[0],
    K0[1],
    K1[0],
    K1[1],
    K2[0],
    K2[1],
    l(0.62, 0.3, 0.62, 0.84, C::Accent),
    d(0.62, 0.26, 0.06, C::Accent),
];

/// A folha de quadros em três filas, cada fila uma animação com o seu nome (parado, correr, pular).
pub const SPRITE_ANIMATIONS: &[Mark] = &[
    r(0.1, 0.2, 0.12, 0.08, C::Text2),
    r(0.3, 0.14, 0.12, 0.2, C::Success),
    r(0.46, 0.14, 0.12, 0.2, C::Success),
    r(0.62, 0.14, 0.12, 0.2, C::Success),
    r(0.1, 0.46, 0.12, 0.08, C::Text2),
    r(0.3, 0.4, 0.12, 0.2, C::Warn),
    r(0.46, 0.4, 0.12, 0.2, C::Warn),
    r(0.62, 0.4, 0.12, 0.2, C::Warn),
    r(0.1, 0.72, 0.12, 0.08, C::Text2),
    r(0.3, 0.66, 0.12, 0.2, C::Info),
    r(0.46, 0.66, 0.12, 0.2, C::Info),
    r(0.62, 0.66, 0.12, 0.2, C::Info),
];

/// O boneco parado (verde) troca para o boneco a pular (laranja): uma animação de cada vez.
const H_PARADO: [Mark; 2] = boneco(0.23, 0.46);
const H_PULO: [Mark; 2] = boneco(0.77, 0.38);
pub const SPRITE_ANIMATOR: &[Mark] = &[
    t(0.1, 0.14, 0.26, 0.6, C::Text3),
    t(0.64, 0.14, 0.26, 0.6, C::Text3),
    H_PARADO[0],
    H_PARADO[1],
    H_PULO[0],
    H_PULO[1],
    r(0.14, 0.8, 0.18, 0.06, C::Success),
    r(0.68, 0.8, 0.18, 0.06, C::Warn),
    a(0.4, 0.44, 0.6, 0.44, C::Text1),
];

/// Três estados ligados por setas; o aceso é o de agora.
pub const STATE_MACHINE: &[Mark] = &[
    r(0.08, 0.16, 0.22, 0.22, C::Accent),
    r(0.7, 0.16, 0.22, 0.22, C::Text3),
    r(0.39, 0.62, 0.22, 0.22, C::Text3),
    l(0.13, 0.27, 0.25, 0.27, C::AccentFg),
    l(0.75, 0.27, 0.87, 0.27, C::Text1),
    l(0.44, 0.73, 0.56, 0.73, C::Text1),
    a(0.33, 0.27, 0.67, 0.27, C::Text1),
    a(0.8, 0.42, 0.62, 0.66, C::Text1),
    a(0.38, 0.68, 0.2, 0.42, C::Text1),
];

/// A bola desliza, cresce e acende aos poucos: do fantasma pequeno à bola cheia.
pub const TWEENS: &[Mark] = &[
    d(0.16, 0.48, 0.08, C::Text3),
    d(0.36, 0.48, 0.13, C::Text2),
    d(0.66, 0.48, 0.22, C::Accent),
    a(0.14, 0.84, 0.86, 0.84, C::Text1),
];

/// Três passos em fila (feito, a tocar, a seguir) e a barra do progresso por baixo.
pub const SEQUENCE_PLAYER: &[Mark] = &[
    r(0.08, 0.3, 0.18, 0.32, C::Success),
    a(0.28, 0.46, 0.39, 0.46, C::Text1),
    r(0.41, 0.3, 0.18, 0.32, C::Accent),
    a(0.61, 0.46, 0.72, 0.46, C::Text1),
    r(0.74, 0.3, 0.18, 0.32, C::Text3),
    l(0.08, 0.8, 0.5, 0.8, C::Accent),
    l(0.5, 0.8, 0.92, 0.8, C::Text3),
];

/// Linhas de código coloridas, recuadas como um bloco, e a margem dos números.
pub const LUAU_SCRIPT: &[Mark] = &[
    l(0.1, 0.16, 0.1, 0.82, C::Text3),
    r(0.16, 0.18, 0.16, 0.09, C::Accent),
    r(0.35, 0.18, 0.3, 0.09, C::Text2),
    r(0.24, 0.36, 0.2, 0.09, C::Info),
    r(0.47, 0.36, 0.22, 0.09, C::Warn),
    r(0.24, 0.54, 0.38, 0.09, C::Text2),
    r(0.16, 0.72, 0.12, 0.09, C::Accent),
];
