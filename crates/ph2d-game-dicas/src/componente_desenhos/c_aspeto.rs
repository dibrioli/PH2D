//! Desenhos das dicas do Add Component (ver `componente.rs`).

use ph2d_editor_core::widget::illustration::Mark;
use ph2d_tokens::ColorToken as C;

use crate::{CHAO, boneco};

const fn r(x: f32, y: f32, w: f32, h: f32, fill: C) -> Mark {
    Mark::Rect {
        at: [x, y],
        size: [w, h],
        fill,
        round: 0.03,
    }
}
const fn q(x: f32, y: f32, w: f32, h: f32, fill: C) -> Mark {
    Mark::Rect {
        at: [x, y],
        size: [w, h],
        fill,
        round: 0.0,
    }
}
const fn d(x: f32, y: f32, r: f32, fill: C) -> Mark {
    Mark::Disc { c: [x, y], r, fill }
}
const fn cap(x: f32, y: f32, fill: C) -> Mark {
    Mark::Capsule {
        c: [x, y],
        half_h: 0.08,
        r: 0.12,
        fill,
    }
}
const fn l(a: [f32; 2], b: [f32; 2], stroke: C) -> Mark {
    Mark::Line {
        a,
        b,
        stroke,
        arrow: false,
    }
}
const fn ar(a: [f32; 2], b: [f32; 2], stroke: C) -> Mark {
    Mark::Line {
        a,
        b,
        stroke,
        arrow: true,
    }
}
const fn ds(x: f32, y: f32, w: f32, h: f32, stroke: C) -> Mark {
    Mark::Dashed {
        at: [x, y],
        size: [w, h],
        stroke,
    }
}

/// Dois pares de discos sobrepostos: o encontro clareia (somar, brilho) e escurece (multiplicar, sombra).
pub const BLEND_MODE: &[Mark] = &[
    d(0.2, 0.5, 0.26, C::Info),
    d(0.32, 0.5, 0.26, C::Warn),
    d(0.26, 0.5, 0.13, C::AccentFg),
    d(0.68, 0.5, 0.26, C::Info),
    d(0.8, 0.5, 0.26, C::Warn),
    d(0.74, 0.5, 0.13, C::BgScrim),
];
/// Uma imagem e um disco-máscara; a seta leva à imagem recortada em disco.
pub const MASK_2D: &[Mark] = &[
    r(0.08, 0.2, 0.3, 0.6, C::Success),
    d(0.3, 0.36, 0.08, C::Warn),
    d(0.23, 0.5, 0.22, C::Accent),
    ar([0.44, 0.5], [0.56, 0.5], C::Text1),
    d(0.77, 0.5, 0.28, C::Success),
    d(0.8, 0.4, 0.08, C::Warn),
];
/// Uma janela-máscara tracejada: o boneco dentro aparece, as cópias fora ficam apagadas.
const DENTRO: [Mark; 2] = boneco(0.5, 0.5);
pub const MASK_INTERACTION: &[Mark] = &[
    cap(0.16, 0.5, C::Border),
    cap(0.84, 0.5, C::Border),
    ds(0.36, 0.14, 0.28, 0.72, C::Text1),
    DENTRO[0],
    DENTRO[1],
];
/// Painel pequeno e painel largo: os quatro cantos têm o MESMO tamanho nos dois.
pub const SLICE_NINE: &[Mark] = &[
    r(0.08, 0.28, 0.18, 0.44, C::Info),
    d(0.11, 0.34, 0.05, C::Warn),
    d(0.23, 0.34, 0.05, C::Warn),
    d(0.11, 0.66, 0.05, C::Warn),
    d(0.23, 0.66, 0.05, C::Warn),
    ar([0.31, 0.5], [0.4, 0.5], C::Text1),
    r(0.45, 0.28, 0.47, 0.44, C::Info),
    d(0.48, 0.34, 0.05, C::Warn),
    d(0.89, 0.34, 0.05, C::Warn),
    d(0.48, 0.66, 0.05, C::Warn),
    d(0.89, 0.66, 0.05, C::Warn),
];
/// Um disco em escada de pixels (nítido) ao lado de um disco liso (suave).
pub const TEXTURE_FILTER: &[Mark] = &[
    q(0.2, 0.2, 0.1, 0.6, C::Accent),
    q(0.15, 0.28, 0.2, 0.44, C::Accent),
    q(0.12, 0.38, 0.26, 0.24, C::Accent),
    l([0.5, 0.18], [0.5, 0.82], C::Text3),
    d(0.75, 0.5, 0.3, C::Accent),
];
/// Um ladrilho; a seta leva a quatro cópias dele lado a lado.
pub const TEXTURE_REPEAT: &[Mark] = &[
    q(0.1, 0.34, 0.16, 0.32, C::Success),
    d(0.18, 0.5, 0.07, C::Warn),
    ar([0.32, 0.5], [0.44, 0.5], C::Text1),
    q(0.52, 0.18, 0.16, 0.32, C::Success),
    d(0.6, 0.34, 0.07, C::Warn),
    q(0.69, 0.18, 0.16, 0.32, C::Success),
    d(0.77, 0.34, 0.07, C::Warn),
    q(0.52, 0.51, 0.16, 0.32, C::Success),
    d(0.6, 0.67, 0.07, C::Warn),
    q(0.69, 0.51, 0.16, 0.32, C::Success),
    d(0.77, 0.67, 0.07, C::Warn),
];
/// Água num rectângulo: as ondas dentro deslizam (seta), o rectângulo fica parado.
pub const UV_TRANSFORM: &[Mark] = &[
    r(0.2, 0.14, 0.6, 0.6, C::Info),
    l([0.3, 0.36], [0.4, 0.28], C::AccentFg),
    l([0.4, 0.28], [0.5, 0.36], C::AccentFg),
    l([0.5, 0.52], [0.6, 0.44], C::AccentFg),
    l([0.6, 0.44], [0.7, 0.52], C::AccentFg),
    ar([0.3, 0.86], [0.7, 0.86], C::Text1),
];
/// Uma caixa no chão a lançar faíscas para cima e para os lados.
pub const PARTICLE_EMITTER: &[Mark] = &[
    CHAO,
    r(0.44, 0.66, 0.12, 0.16, C::Accent),
    ar([0.47, 0.6], [0.36, 0.3], C::Text3),
    ar([0.53, 0.6], [0.64, 0.3], C::Text3),
    d(0.5, 0.2, 0.05, C::Warn),
    d(0.3, 0.2, 0.04, C::Warn),
    d(0.7, 0.18, 0.04, C::Warn),
    d(0.24, 0.42, 0.035, C::Warn),
    d(0.77, 0.4, 0.035, C::Warn),
    d(0.42, 0.12, 0.03, C::Warn),
    d(0.6, 0.1, 0.03, C::Warn),
];
/// Uma lâmpada, um bloco que pára o raio, e a sombra comprida atrás dele no chão.
pub const BLOQUEIA_LUZ: &[Mark] = &[
    CHAO,
    q(0.5, 0.82, 0.4, 0.07, C::BgScrim),
    d(0.12, 0.22, 0.1, C::Warn),
    ar([0.18, 0.3], [0.4, 0.52], C::Warn),
    l([0.18, 0.18], [0.88, 0.18], C::Warn),
    r(0.42, 0.44, 0.08, 0.38, C::Accent),
];
/// Cena escura: a forma comum fica apagada, o boneco brilha por inteiro mesmo longe da lâmpada.
const BRILHA: [Mark; 2] = boneco(0.74, 0.5);
pub const NAO_RECEBE_LUZ: &[Mark] = &[
    q(0.04, 0.08, 0.92, 0.84, C::BgScrim),
    d(0.12, 0.22, 0.08, C::Warn),
    cap(0.38, 0.5, C::BorderStrong),
    BRILHA[0],
    BRILHA[1],
    l([0.86, 0.22], [0.9, 0.14], C::Warn),
    l([0.86, 0.5], [0.92, 0.5], C::Warn),
];
/// Um alto-falante com três arcos de som, do perto (forte) ao longe (fraco).
pub const AUDIO_SOURCE_2D: &[Mark] = &[
    r(0.16, 0.38, 0.08, 0.24, C::ObjSound),
    r(0.24, 0.24, 0.06, 0.52, C::ObjSound),
    l([0.4, 0.34], [0.43, 0.5], C::Text1),
    l([0.43, 0.5], [0.4, 0.66], C::Text1),
    l([0.52, 0.24], [0.57, 0.5], C::Text2),
    l([0.57, 0.5], [0.52, 0.76], C::Text2),
    l([0.64, 0.16], [0.71, 0.5], C::Text3),
    l([0.71, 0.5], [0.64, 0.84], C::Text3),
];
/// O boneco ao centro, com a orelha, e o som a chegar-lhe dos dois lados.
const OUVE: [Mark; 2] = boneco(0.5, 0.5);
pub const AUDIO_LISTENER_2D: &[Mark] = &[
    r(0.08, 0.4, 0.06, 0.2, C::ObjSound),
    r(0.86, 0.4, 0.06, 0.2, C::ObjSound),
    ar([0.18, 0.5], [0.38, 0.5], C::Text1),
    ar([0.82, 0.5], [0.62, 0.5], C::Text1),
    OUVE[0],
    OUVE[1],
    d(0.47, 0.5, 0.045, C::Warn),
];
/// Uma cauda de discos dobrada, com a cadeia de ossos e as juntas por cima.
pub const BONE: &[Mark] = &[
    d(0.16, 0.7, 0.14, C::Success),
    d(0.3, 0.6, 0.13, C::Success),
    d(0.44, 0.48, 0.12, C::Success),
    d(0.6, 0.38, 0.11, C::Success),
    d(0.76, 0.34, 0.1, C::Success),
    l([0.16, 0.7], [0.44, 0.48], C::BoneHandle),
    l([0.44, 0.48], [0.76, 0.34], C::BoneHandle),
    d(0.16, 0.7, 0.05, C::Text1),
    d(0.44, 0.48, 0.05, C::Text1),
    d(0.76, 0.34, 0.05, C::Text1),
];
/// Braço e antebraço no cotovelo: um leque de raios marca o ângulo permitido, a linha vermelha o limite.
pub const BONE_LIMIT: &[Mark] = &[
    l([0.48, 0.64], [0.8, 0.64], C::Text3),
    l([0.48, 0.64], [0.74, 0.3], C::Text3),
    l([0.48, 0.64], [0.56, 0.2], C::Danger),
    l([0.12, 0.64], [0.48, 0.64], C::BoneHandle),
    ar([0.48, 0.64], [0.68, 0.24], C::BoneHandle),
    d(0.12, 0.64, 0.05, C::Text1),
    d(0.48, 0.64, 0.06, C::Text1),
];
/// Braço esticado com músculo pequeno; a seta leva ao braço dobrado com o músculo inchado.
pub const SMART_BONE: &[Mark] = &[
    d(0.2, 0.56, 0.08, C::Warn),
    l([0.06, 0.62], [0.24, 0.62], C::BoneHandle),
    l([0.24, 0.62], [0.4, 0.62], C::BoneHandle),
    d(0.24, 0.62, 0.04, C::Text1),
    ar([0.45, 0.5], [0.55, 0.5], C::Text1),
    d(0.7, 0.5, 0.17, C::Warn),
    l([0.6, 0.66], [0.8, 0.66], C::BoneHandle),
    l([0.8, 0.66], [0.84, 0.24], C::BoneHandle),
    d(0.8, 0.66, 0.04, C::Text1),
];
/// Uma caixa com as setas de mover (X e Y), a pega de girar em cima e a de tamanho no canto.
pub const TRANSFORM: &[Mark] = &[
    r(0.36, 0.32, 0.18, 0.4, C::Accent),
    ar([0.45, 0.52], [0.72, 0.52], C::AxisX),
    ar([0.45, 0.52], [0.45, 0.12], C::AxisY),
    l([0.54, 0.24], [0.6, 0.14], C::Text1),
    d(0.6, 0.14, 0.05, C::Text1),
    q(0.53, 0.7, 0.04, 0.08, C::Text1),
];
/// Dois inimigos com a mesma etiqueta e uma moeda com outra.
pub const TAGS: &[Mark] = &[
    cap(0.16, 0.56, C::Danger),
    cap(0.42, 0.56, C::Danger),
    d(0.76, 0.6, 0.18, C::Warn),
    r(0.19, 0.12, 0.1, 0.14, C::Info),
    d(0.21, 0.19, 0.025, C::AccentFg),
    r(0.45, 0.12, 0.1, 0.14, C::Info),
    d(0.47, 0.19, 0.025, C::AccentFg),
    r(0.79, 0.16, 0.1, 0.14, C::Success),
    d(0.81, 0.23, 0.025, C::AccentFg),
];
/// O boneco preso a um alvo por uma ligação: o alvo anda e ele segue, a olhar para ele.
const SEGUE: [Mark; 2] = boneco(0.24, 0.56);
pub const CONSTRAINTS: &[Mark] = &[
    l([0.3, 0.5], [0.7, 0.5], C::Text3),
    d(0.7, 0.5, 0.05, C::Text3),
    d(0.76, 0.5, 0.12, C::Warn),
    ar([0.8, 0.34], [0.9, 0.2], C::Text1),
    ar([0.2, 0.32], [0.3, 0.18], C::Text1),
    SEGUE[0],
    SEGUE[1],
];
/// Duas câmaras: a da esquerda vê o objecto, a da direita não (um X sobre a vista).
pub const VISIBILITY_LAYER: &[Mark] = &[
    r(0.06, 0.4, 0.1, 0.2, C::ObjCamera),
    q(0.16, 0.44, 0.03, 0.12, C::ObjCamera),
    r(0.84, 0.4, 0.1, 0.2, C::ObjCamera),
    q(0.81, 0.44, 0.03, 0.12, C::ObjCamera),
    d(0.5, 0.5, 0.18, C::Accent),
    ar([0.22, 0.5], [0.4, 0.5], C::Success),
    l([0.78, 0.5], [0.6, 0.5], C::Text3),
    l([0.65, 0.4], [0.73, 0.6], C::Danger),
    l([0.65, 0.6], [0.73, 0.4], C::Danger),
];
/// O ecrã tracejado: o boneco dentro anda, a forma fora do ecrã está em pausa.
const ATIVO: [Mark; 2] = boneco(0.26, 0.5);
pub const ON_SCREEN_ENABLER: &[Mark] = &[
    ds(0.06, 0.14, 0.54, 0.72, C::Text1),
    ATIVO[0],
    ATIVO[1],
    ar([0.34, 0.5], [0.52, 0.5], C::Text1),
    cap(0.8, 0.5, C::Border),
    q(0.775, 0.42, 0.015, 0.16, C::Text1),
    q(0.81, 0.42, 0.015, 0.16, C::Text1),
];
