//! **A ponte da seção Effects** (ADR-0132) — o painel pede, isto escreve na cena, e é aqui que
//! o vocabulário do motor vira o snapshot que o painel desenha.
//!
//! Não é um `*_live.rs`: não há recook por frame. A pilha é **dado de documento** e o
//! `cooked()` a avalia sozinho quando alguém consome a geometria. O que esta ponte faz é o que
//! o painel não pode (ele não conhece a `VecScene`): pôr, tirar, reordenar e ajustar.
//!
//! **Nenhuma função aqui nomeia um efeito.** Acrescentar um tipo ao motor não toca neste
//! arquivo — é a mesma propriedade que o `paint_effects` ganhou, do outro lado da fronteira.

use ph2d_panel_vector::{FalloffRole, FxParamView, FxRowView};
use ph2d_vec_scene::effect::{FxEntry, MAX_PATH_EFFECTS, PathEffect};
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
            .enumerate()
            .map(|(row, e)| FxRowView {
                label: e.effect.label(),
                enabled: e.enabled,
                params: e
                    .effect
                    .params()
                    .iter()
                    .enumerate()
                    .map(|(i, d)| FxParamView {
                        name: d.name,
                        min: d.min,
                        max: d.max,
                        toggle: d.toggle,
                        integer: d.integer,
                        value: e.effect.get(i),
                    })
                    .collect(),
                // Se este é um Falloff: para onde a força aponta. Procura o próximo efeito LIGADO
                // que NÃO é Falloff (falloffs compõem entre si — o alvo é o deformador). É a MESMA
                // pergunta que o `run_stack` responde ao consumir o campo; o painel só a exibe.
                falloff_role: falloff_role(&p.effects, row),
            })
            .collect()
    })
}

/// O papel de dica de um Falloff na linha `row` — a metade que só a ponte conhece (ela vê o
/// motor). O painel a desenha; um Falloff sozinho não pode parecer quebrado (DIRETIVA §2).
///
/// O alvo é o próximo efeito **ligado** que NÃO é Falloff (dois Falloffs compõem sobre o mesmo
/// deformador, como no `run_stack`). Se esse alvo consome força (`takes_falloff`), o campo modula
/// abaixo; senão (nada abaixo, ou só Trim/Repeater), é inerte na geometria.
fn falloff_role(effects: &[FxEntry], row: usize) -> FalloffRole {
    if effects[row].effect.as_falloff().is_none() {
        return FalloffRole::NotFalloff;
    }
    let target = effects[row + 1..]
        .iter()
        .find(|n| n.enabled && n.effect.as_falloff().is_none());
    match target {
        Some(n) if n.effect.takes_falloff() => FalloffRole::ModulatesBelow,
        _ => FalloffRole::Inert,
    }
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
        p.effects.push(FxEntry::new(fx));
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
    let Some(e) = p.effects.get_mut(row) else {
        return;
    };
    let Some(d) = e.effect.params().get(param).copied() else {
        return;
    };
    e.effect
        .set(param, d.min + track.clamp(0.0, 1.0) * (d.max - d.min));
}

/// Alterna um parâmetro de CAIXINHA. O painel desenha um botão (não um slider), então o clique
/// não traz valor — quem sabe o estado atual é a cena.
pub(crate) fn toggle_param(scene: &mut VecScene, id: VecPathId, row: usize, param: usize) {
    let Some(p) = scene.path_mut(id) else { return };
    let Some(e) = p.effects.get_mut(row) else {
        return;
    };
    let on = e.effect.get(param) >= 0.5;
    e.effect.set(param, f64::from(u8::from(!on)));
}

/// Este parâmetro é uma caixinha? O dispatch precisa saber: um clique num slider não existe, e
/// num toggle não há valor a ler.
#[must_use]
pub(crate) fn is_toggle(scene: &VecScene, id: VecPathId, row: usize, param: usize) -> bool {
    scene
        .path(id)
        .and_then(|p| p.effects.get(row))
        .and_then(|e| e.effect.params().get(param).map(|d| d.toggle))
        .unwrap_or(false)
}

/// **Desarma (ou rearma) o efeito da linha `row`** — o olho.
///
/// Os parâmetros ficam INTACTOS: é essa a diferença entre desarmar e zerar, e é por isso que o
/// "ligado" mora na ENTRADA e não no efeito.
pub(crate) fn toggle_enabled(scene: &mut VecScene, id: VecPathId, row: usize) {
    let Some(p) = scene.path_mut(id) else { return };
    if let Some(e) = p.effects.get_mut(row) {
        e.enabled = !e.enabled;
    }
}

#[cfg(test)]
#[path = "fx_bridge_tests.rs"]
mod tests;
