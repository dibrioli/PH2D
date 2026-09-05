//! ⭐ **OS CONSTRUTORES DOS SINAIS** (W119–W120) — os números com que cada seta, balão e símbolo
//! nasce.
//!
//! # Por que um arquivo irmão
//!
//! O [`super`] responde pelos sólidos e pelas chapas; este pela família dos sinais. O irmão já
//! estava nas `462` linhas de um tecto de `600`, e as dez formas desta wave passavam-no.
//! ⛔ *Split, nunca allowlist.*

use ph2d_field::Primitive;

use super::make::round_of;

/// ⚠️ **O balão nasce mais LARGO que alto** — é a proporção de uma linha de texto, e um balão
/// quadrado lê-se como uma caixa com um bico.
pub(crate) fn a_speech_rect(r: f32) -> Primitive {
    Primitive::SpeechRect {
        half_width: r,
        half_span: r * 0.66,
        tail: r * 0.45,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_speech_oval(r: f32) -> Primitive {
    Primitive::SpeechOval {
        half_width: r,
        half_span: r * 0.62,
        tail: r * 0.45,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// A nuvem: a mesma primitiva **sem** fieira.
pub(crate) fn a_cloud(r: f32) -> Primitive {
    uma_nuvem(r, 0.0)
}

/// E o balão de pensamento: a mesma, **com** fieira. ⭐ É a porta que muda, não a fórmula.
pub(crate) fn a_thought(r: f32) -> Primitive {
    uma_nuvem(r, r * 0.40)
}

fn uma_nuvem(r: f32, tail: f32) -> Primitive {
    Primitive::Cloud {
        lobes: 5,
        half_width: r,
        half_span: r * 0.50,
        tail,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ **O filete do raio nasce a METADE** do das outras: a banda do meio dele é `10 %` da peça, e o
/// limite dela é mais apertado do que o de uma chapa cheia.
pub(crate) fn a_bolt(r: f32) -> Primitive {
    Primitive::Bolt {
        half_width: r * 0.62,
        half_span: r,
        half_height: r * 0.25,
        round: round_of(r) * 0.5,
        chamfer: 0.0,
    }
}

pub(crate) fn a_shield(r: f32) -> Primitive {
    Primitive::Shield {
        half_width: r * 0.78,
        half_span: r,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_tag(r: f32) -> Primitive {
    Primitive::Tag {
        half_width: r,
        half_span: r * 0.58,
        point: r * 0.55,
        hole: r * 0.15,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_check(r: f32) -> Primitive {
    Primitive::Check {
        half_width: r,
        half_span: r * 0.72,
        thickness: r * 0.26,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn a_banner(r: f32) -> Primitive {
    Primitive::Banner {
        half_width: r,
        half_span: r * 0.50,
        notch: r * 0.32,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ A espessura nasce a `24 %` e não a `20 %`: o filete de nascimento é `10 %` do enquadramento, e
/// a parede do filete de uma chave é **metade** da espessura dela.
pub(crate) fn a_brace(r: f32) -> Primitive {
    Primitive::Brace {
        half_span: r,
        thickness: r * 0.24,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}
