//! **A ponte da seção Effects** (ADR-0132) — o painel pede, isto escreve na cena.
//!
//! Não é um `*_live.rs`: não há recook por frame aqui. A pilha é **dado de documento**, e o
//! `cooked()` a avalia sozinho quando alguém consome a geometria. O que esta ponte faz é o
//! que o painel não pode fazer (ele não conhece a `VecScene`): pôr, tirar e ajustar.
//!
//! Por isso também **não** há aqui uma 2ª resposta a *"a alça de raio pode existir?"* — um
//! efeito na pilha não reescreve os `verts` autorados, ele deriva por cima deles.

use ph2d_vec_scene::effect::PathEffect;
use ph2d_vec_scene::fx_trim::TrimSpec;
use ph2d_vec_scene::{VecPathId, VecScene};

/// Qual dos três parâmetros do Trim um `SetValue` endereça.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum TrimParam {
    Start,
    End,
    Offset,
}

/// **O caminho que a seção Effects governa** — exatamente UM selecionado, ou nada.
///
/// A seção é por-caminho: com dois selecionados, *"o Trim"* não tem referente, e oferecer
/// controles que escrevem num deles em silêncio seria pior que não os oferecer.
#[must_use]
pub(crate) fn sole_path(selected: &[VecPathId]) -> Option<VecPathId> {
    match selected {
        [id] => Some(*id),
        _ => None,
    }
}

/// O Trim do caminho, se ele tiver um — a tupla que o painel desenha.
#[must_use]
pub(crate) fn trim_of(scene: &VecScene, id: VecPathId) -> Option<(f64, f64, f64)> {
    let t = scene
        .path(id)?
        .effects
        .iter()
        .find_map(PathEffect::as_trim)?;
    Some((t.start, t.end, t.offset))
}

/// **Põe ou tira** o Trim — o toggle do botão.
///
/// Ao PÔR, ele nasce no ponto **neutro** (`0..1`, sem offset), que é um no-op byte-idêntico:
/// o clique não pode mudar o desenho, senão o artista veria a forma saltar antes de tocar em
/// qualquer parâmetro. Ao TIRAR, remove **todos** os Trims — a UI expõe um, mas um documento
/// vindo de código (ou de um save futuro) pode ter mais, e deixar órfãos invisíveis seria a
/// pior das saídas.
pub(crate) fn toggle_trim(scene: &mut VecScene, id: VecPathId) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    if p.effects.iter().any(|e| matches!(e, PathEffect::Trim(_))) {
        p.effects.retain(|e| !matches!(e, PathEffect::Trim(_)));
    } else {
        p.effects.push(PathEffect::Trim(TrimSpec::default()));
    }
}

/// Ajusta um parâmetro do Trim. No-op se o caminho não tiver um — o painel não oferece os
/// sliders nesse caso, mas a ponte não depende de o painel ter razão.
pub(crate) fn set_trim_param(scene: &mut VecScene, id: VecPathId, which: TrimParam, v: f64) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    // A UI governa o PRIMEIRO Trim; os demais (se houver) vieram de código.
    let Some(t) = p.effects.iter_mut().find_map(PathEffect::as_trim_mut) else {
        return;
    };
    match which {
        TrimParam::Start => t.start = v,
        TrimParam::End => t.end = v,
        TrimParam::Offset => t.offset = v,
    }
}

#[cfg(test)]
#[path = "fx_bridge_tests.rs"]
mod tests;
