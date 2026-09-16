//! **AS PEÇAS DE VECTOR DA LEI DO ALVO** — irmão (`#[path]`) do
//! [`super::stroke_target`].
//!
//! ⚠️ **O corte foi por RESPONSABILIDADE e forçado pelo tecto de LOC** (o
//! ficheiro chegou a `705` com o braço do [`crate::Verb::BoxTrim`]): ali fica
//! *que alvo cada verbo dá a cada vértice*; aqui ficam as três contas de vector
//! que essa lei usa e que não decidem nada sozinhas. ⛔ Nunca uma entrada no
//! `FILE_OVERAGE_OK`.

use super::*;

/// **O PUXÃO LATERAL, por uma porta só** — Pinch, Magnify e o termo lateral do
/// Crease e do Blob perguntam aqui, e a [`crate::RefMode::lateral_for`] responde.
///
/// ⚠️ Quatro braços com um `match mode` cada seriam quatro lugares onde o quinto
/// verbo que aperta nasce sem a resposta; e a divergência que isto fecha vale
/// `5,776e-4` no atlas, contra um piso de `5,96e-8`.
///
/// ⚠️ **Ela toma o MODO e o VERBO, não a [`crate::KernelLaw`] já resolvida**, e
/// a razão é o achado de 2026-08-15: a lei lateral do Blender é **por
/// ferramenta**, então ela não cabe num campo do `KernelLaw` — ver o doc do
/// [`crate::RefMode::lateral_for`].
pub(crate) fn lateral_pull(
    mode: crate::RefMode,
    verb: Verb,
    p: [f32; 3],
    center: [f32; 3],
    normal: [f32; 3],
    path: [f32; 3],
) -> [f32; 3] {
    let d = [center[0] - p[0], center[1] - p[1], center[2] - p[2]];
    match mode.lateral_for(verb) {
        crate::LateralPull::Tangential => remove_along(d, normal),
        // `Pinch.js:52-58` / `Crease.js:59-61`: o delta CRU até o centro, em 3D.
        crate::LateralPull::Direct => d,
        // O *Pinch*: a soma da componente PERPENDICULAR ao traço com a
        // NORMAL, ou seja com a componente ao longo do TRAÇO removida. Ver
        // [`crate::LateralPull::AcrossStroke`].
        crate::LateralPull::AcrossStroke => match stroke_axis(normal, path) {
            Some(along) => remove_along(d, along),
            // ⚠️ **Sem direção não há aperto, e é a referência que recusa** —
            // ela adia o primeiro dab de cada passe de simetria e desiste
            // quando o deslocamento do cursor é zero. Um eixo inventado aqui seria
            // uma direção que o artista não desenhou.
            None => [0.0; 3],
        },
    }
}

/// **A DIREÇÃO DO TRAÇO no plano tangente**, unitária — ou `None` quando o dab
/// não tem uma.
///
/// A referência monta o primeiro eixo como o produto vectorial da normal de
/// área com o deslocamento do cursor, e o segundo cruzando a normal com esse; é o
/// SEGUNDO que ela descarta, e é ele que esta função
/// devolve. Passar pelo `X` e cruzar de volta — em vez de projetar o `path`
/// direto — é o que **ortogonaliza** o traço contra a normal: um gesto que
/// mergulha na superfície não leva a componente que mergulha.
///
/// ⚠️ **`None` cobre os DOIS degenerados com a mesma pergunta:** um dab sem
/// traço ([`crate::Dab::path`] nasce em zero) e um traço paralelo à normal (o
/// `cross` colapsa). Distingui-los daria dois braços para uma resposta só.
pub(crate) fn stroke_axis(normal: [f32; 3], path: [f32; 3]) -> Option<[f32; 3]> {
    let x = cross(normal, path);
    let y = cross(normal, x);
    let len = (y[0] * y[0] + y[1] * y[1] + y[2] * y[2]).sqrt();
    // ⚠️ O piso é sobre o COMPRIMENTO e não sobre `len² `: um traço curto tem
    // `|path|` da ordem do espaçamento, e o quadrado dele desce a `1e-8` num
    // gesto perfeitamente normal.
    if len > 1.0e-6 {
        Some([y[0] / len, y[1] / len, y[2] / len])
    } else {
        None
    }
}

/// **A interpolação de um PONTO em direcção a outro** — `b + (t − b)·a`, canal a
/// canal.
///
/// ⚠️ **Ela chama o [`super::apply::toward`] escalar em vez de repetir a
/// aritmética**: é a MESMA lei que o aplicador usa para compor o alvo com o
/// `accum`, e escrevê-la duas vezes seria a segunda resposta a *«o que é
/// caminhar uma fracção até um alvo?»* — com a divergência a aparecer como o
/// apagador a parar num sítio diferente do que o aplicador esperava.
pub(crate) fn toward3(b: [f32; 3], t: [f32; 3], a: f32) -> [f32; 3] {
    [
        super::apply::toward(b[0], t[0], a),
        super::apply::toward(b[1], t[1], a),
        super::apply::toward(b[2], t[2], a),
    ]
}
