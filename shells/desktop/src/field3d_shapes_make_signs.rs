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

// ─────────────────────────── W122 — o fluxograma ───────────────────────────
//
// ⚠️ **Elas moram na família `Plates` e não na `Signs`**, e a razão não é gosto: há exactamente
// sete tokens `NodeCat*` e as sete famílias já os têm todos, então uma família nova partilharia
// tinta ou pediria um token, que é decisão de design (§7). E o critério bate: as quatro são um
// contorno 2D de fórmula puxado em Z, que é o que a `Plates` diz que é.

/// ⚠️ **Nasce inclinado**: `skew = 0` é o retângulo, e uma forma que nasce igual a outra da paleta
/// não ensina o que ela faz.
pub(crate) fn a_parallelogram(r: f32) -> Primitive {
    Primitive::Parallelogram {
        half_width: r * 0.80,
        half_span: r * 0.55,
        skew: r * 0.30,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ **Mais largo que alto**, como o símbolo é desenhado — e a razão fica longe da parede
/// (`half_span ≤ 2·half_width`).
pub(crate) fn a_delay(r: f32) -> Primitive {
    Primitive::Delay {
        half_width: r,
        half_span: r * 0.55,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

/// ⚠️ **O bico a `40 %` da parede dele** — com ele curto a peça lê-se como o atraso, e com ele no
/// máximo as faces retas desaparecem.
pub(crate) fn a_display(r: f32) -> Primitive {
    Primitive::Display {
        half_width: r,
        half_span: r * 0.55,
        point: r * 0.58,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

pub(crate) fn an_off_page(r: f32) -> Primitive {
    Primitive::OffPage {
        half_width: r * 0.80,
        half_span: r * 0.70,
        point: r * 0.45,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

// ─────────────────────────── W123 — as duas que saíram do «fica desenhada» ───────────────────────

/// ⚠️ **Três voltas e a fita a `55 %` do passo** — com uma volta não se lê como espiral, e com a
/// fita a encher o passo o vale entre as voltas (que é a forma) desaparece.
pub(crate) fn a_spiral(r: f32) -> Primitive {
    let pitch = r * 0.30;
    Primitive::Spiral {
        radius: r * 0.16,
        pitch,
        turns: 3.0,
        thickness: pitch * 0.275,
        half_height: r * 0.25,
        // ⚠️ **Metade do que as outras nascem**: a parede do filete de uma fita é a meia-espessura
        // dela, e a fita mede `27,5 %` do passo.
        round: pitch * 0.06,
        chamfer: 0.0,
    }
}

/// ⚠️ **Nasce COM onda** — a zero ele é o retângulo, e uma forma que nasce igual a outra da paleta
/// não ensina o que ela faz.
pub(crate) fn a_document(r: f32) -> Primitive {
    Primitive::Document {
        half_width: r,
        half_span: r * 0.62,
        wave: r * 0.20,
        half_height: r * 0.25,
        round: round_of(r),
        chamfer: 0.0,
    }
}

// ─────────────────────────── W124 — a mola e a rede ───────────────────────────

/// ⚠️ **Três voltas e o tubo a `30 %` do passo** — com uma volta não se lê como mola, e com o tubo
/// a encher o passo ela vira um cilindro.
pub(crate) fn a_helix(r: f32) -> Primitive {
    let pitch = r * 0.42;
    Primitive::Helix {
        radius: r * 0.62,
        pitch,
        turns: 3.0,
        thickness: pitch * 0.15,
        // ⚠️ **Metade da meia-espessura do tubo**: a parede do filete de uma mola é o raio do tubo,
        // e ele é fino de propósito.
        round: pitch * 0.05,
        chamfer: 0.0,
    }
}

/// ⚠️ **Quatro células no bloco** — com duas não se lê como rede, e a parede nasce a `20 %` da
/// célula, que é a proporção com que a impressão 3D a usa.
pub(crate) fn a_gyroid(r: f32) -> Primitive {
    let cell = r * 0.5;
    Primitive::Gyroid {
        half: [r; 3],
        cell,
        thickness: cell * 0.1,
        // ⚠️ **A parede do filete aqui é a PAREDE da rede**, que é `10 %` da célula.
        round: cell * 0.03,
        chamfer: 0.0,
    }
}
