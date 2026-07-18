//! **A ponte da seção Effects** (ADR-0132) — o painel pede, isto escreve na cena, e é aqui que
//! o vocabulário do motor vira o snapshot que o painel desenha.
//!
//! Não é um `*_live.rs`: não há recook por frame. A pilha é **dado de documento** e o
//! `cooked()` a avalia sozinho quando alguém consome a geometria. O que esta ponte faz é o que
//! o painel não pode (ele não conhece a `VecScene`): pôr, tirar, reordenar e ajustar.
//!
//! **Nenhuma função aqui nomeia um efeito.** Acrescentar um tipo ao motor não toca neste
//! arquivo — é a mesma propriedade que o `paint_effects` ganhou, do outro lado da fronteira.

use ph2d_panel_vector::{FxParamView, FxRowView};
use ph2d_vec_scene::effect::{MAX_PATH_EFFECTS, PathEffect};
use ph2d_vec_scene::{VecPathId, VecScene};

/// **O caminho que a seção Effects governa** — exatamente UM selecionado, ou nada.
///
/// A seção é por-caminho: com dois selecionados, *"a pilha"* não tem referente, e oferecer
/// controles que escrevem num deles em silêncio seria pior que não os oferecer.
#[must_use]
pub(crate) fn sole_path(selected: &[VecPathId]) -> Option<VecPathId> {
    match selected {
        [id] => Some(*id),
        _ => None,
    }
}

/// A pilha do caminho, traduzida para o que o painel desenha.
#[must_use]
pub(crate) fn stack_view(scene: &VecScene, id: VecPathId) -> Vec<FxRowView> {
    scene.path(id).map_or_else(Vec::new, |p| {
        p.effects
            .iter()
            .map(|fx| FxRowView {
                label: fx.label(),
                params: fx
                    .params()
                    .iter()
                    .enumerate()
                    .map(|(i, d)| FxParamView {
                        name: d.name,
                        min: d.min,
                        max: d.max,
                        toggle: d.toggle,
                        value: fx.get(i),
                    })
                    .collect(),
            })
            .collect()
    })
}

/// **Põe** um efeito do tipo `kind` no fim da pilha. Ele nasce NEUTRO, então o clique não pode
/// mudar o desenho — senão o artista veria a forma saltar antes de tocar num parâmetro.
///
/// Recusa acima de [`MAX_PATH_EFFECTS`]: o painel para de oferecer o Add nesse ponto, mas a
/// ponte não depende de o painel ter razão.
pub(crate) fn add(scene: &mut VecScene, id: VecPathId, kind: usize) {
    let Some(p) = scene.path_mut(id) else { return };
    if p.effects.len() >= MAX_PATH_EFFECTS {
        return;
    }
    if let Some(fx) = PathEffect::from_kind(kind) {
        p.effects.push(fx);
    }
}

/// Remove o efeito da linha `row`.
pub(crate) fn remove(scene: &mut VecScene, id: VecPathId, row: usize) {
    let Some(p) = scene.path_mut(id) else { return };
    if row < p.effects.len() {
        p.effects.remove(row);
    }
}

/// Troca o efeito da linha `row` com o vizinho — `up` decide qual.
///
/// A ORDEM muda a geometria (ADR-0132), então isto é uma edição de documento como outra
/// qualquer. Nas bordas é no-op: o painel nem oferece o botão ali.
pub(crate) fn reorder(scene: &mut VecScene, id: VecPathId, row: usize, up: bool) {
    let Some(p) = scene.path_mut(id) else { return };
    let other = if up {
        row.checked_sub(1)
    } else {
        Some(row + 1)
    };
    if let Some(o) = other
        && row < p.effects.len()
        && o < p.effects.len()
    {
        p.effects.swap(row, o);
    }
}

/// Ajusta o parâmetro `param` do efeito da linha `row`. `track` é a posição NORMALIZADA
/// `0..=1` que o painel entregou; a faixa real é do EFEITO, e é aqui que ela é aplicada —
/// o painel não a conhece de um lado só, e converter lá seria uma 2ª cópia da faixa.
pub(crate) fn set_param(scene: &mut VecScene, id: VecPathId, row: usize, param: usize, track: f64) {
    let Some(p) = scene.path_mut(id) else { return };
    let Some(fx) = p.effects.get_mut(row) else {
        return;
    };
    let Some(d) = fx.params().get(param).copied() else {
        return;
    };
    fx.set(param, d.min + track.clamp(0.0, 1.0) * (d.max - d.min));
}

/// Alterna um parâmetro de CAIXINHA. O painel desenha um botão (não um slider), então o clique
/// não traz valor — quem sabe o estado atual é a cena.
pub(crate) fn toggle_param(scene: &mut VecScene, id: VecPathId, row: usize, param: usize) {
    let Some(p) = scene.path_mut(id) else { return };
    let Some(fx) = p.effects.get_mut(row) else {
        return;
    };
    let on = fx.get(param) >= 0.5;
    fx.set(param, f64::from(u8::from(!on)));
}

/// Este parâmetro é uma caixinha? O dispatch precisa saber: um clique num slider não existe, e
/// num toggle não há valor a ler.
#[must_use]
pub(crate) fn is_toggle(scene: &VecScene, id: VecPathId, row: usize, param: usize) -> bool {
    scene
        .path(id)
        .and_then(|p| p.effects.get(row))
        .and_then(|fx| fx.params().get(param).map(|d| d.toggle))
        .unwrap_or(false)
}

#[cfg(test)]
#[path = "fx_bridge_tests.rs"]
mod tests;
