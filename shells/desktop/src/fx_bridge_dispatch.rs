//! **A tradução id → intenção** da seção Effects — módulo irmão do [`crate::fx_bridge`].
//!
//! O `render_loop` não pode saber que efeitos existem, e não sabe: ele recebe um `NodeId`,
//! pergunta aqui o que ele significa, e guarda a resposta. Toda a aritmética de linha e
//! parâmetro mora neste arquivo, e é testável sem janela.
//!
//! Mora fora do `fx_bridge` porque são duas perguntas diferentes: aquele sabe **escrever na
//! cena**, este sabe **ler a tela**. Juntá-los faria um arquivo que conhece as duas pontas.

use ph2d_vec_scene::{VecPathId, VecScene};

/// O que um clique numa LINHA da pilha pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FxRowAction {
    /// O botão do rótulo remove aquele efeito.
    Remove,
    /// Sobe na pilha (a ordem muda a geometria — ADR-0132).
    Up,
    /// Desce na pilha.
    Down,
    /// O olho: desarma (ou rearma) o efeito, sem lhe tocar nos parâmetros.
    Hide,
    /// Um parâmetro de CAIXINHA: o clique alterna, não traz valor.
    Toggle(usize),
}

/// O que um clique na seção pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FxClick {
    /// Põe um efeito do tipo `kind`.
    Add(usize),
    /// Age sobre a linha `row`.
    Row(usize, FxRowAction),
}

/// Classifica um clique. `None` se o id não é da seção.
///
/// A varredura é sobre os TETOS — os ids são hashes de NOME, então não há aritmética que os
/// inverta. Barato, e é o padrão que os presets do Envelope já usam.
#[must_use]
pub(crate) fn classify_click(id: ph2d_editor::ids::NodeId) -> Option<FxClick> {
    use ph2d_editor::ids as i;
    for k in 0..i::MAX_FX_KINDS {
        if id == i::vector_fx_add_id(k) {
            return Some(FxClick::Add(k));
        }
    }
    for r in 0..i::MAX_FX_ROWS {
        if id == i::vector_fx_remove_id(r) {
            return Some(FxClick::Row(r, FxRowAction::Remove));
        }
        if id == i::vector_fx_up_id(r) {
            return Some(FxClick::Row(r, FxRowAction::Up));
        }
        if id == i::vector_fx_down_id(r) {
            return Some(FxClick::Row(r, FxRowAction::Down));
        }
        if id == i::vector_fx_hide_id(r) {
            return Some(FxClick::Row(r, FxRowAction::Hide));
        }
        // Uma caixinha é pintada como BOTÃO e tem id PRÓPRIO. Reusar o id do slider punha dois
        // tipos de widget num id só: o store registava um `Slider`, e um slider NÃO emite Click
        // no Up — este ramo nunca corria (Enio, 2026-07-18).
        for p in 0..i::MAX_FX_ROW_PARAMS {
            if id == i::vector_fx_toggle_id(r, p) {
                return Some(FxClick::Row(r, FxRowAction::Toggle(p)));
            }
        }
    }
    None
}

/// A `(linha, parâmetro)` de um slider da pilha, ou `None`.
#[must_use]
pub(crate) fn classify_param(id: ph2d_editor::ids::NodeId) -> Option<(usize, usize)> {
    use ph2d_editor::ids as i;
    (0..i::MAX_FX_ROWS).find_map(|r| {
        (0..i::MAX_FX_ROW_PARAMS)
            .find(|&p| id == i::vector_fx_param_id(r, p))
            .map(|p| (r, p))
    })
}

/// Aplica o que o frame juntou.
///
/// ⚠️ **A ordem importa**: o Add vem primeiro (a linha nova tem de existir antes de alguém
/// mexer nela), e o Toggle é resolvido consultando a CENA — o clique não traz valor, e um
/// parâmetro de caixinha e um slider partilham o id.
pub(crate) fn apply(
    scene: &mut VecScene,
    id: VecPathId,
    add: Option<usize>,
    button: Option<(usize, FxRowAction)>,
    param: Option<(usize, usize, f64)>,
) {
    if let Some(kind) = add {
        crate::fx_bridge::add(scene, id, kind);
    }
    if let Some((row, action)) = button {
        match action {
            FxRowAction::Remove => crate::fx_bridge::remove(scene, id, row),
            FxRowAction::Up => crate::fx_bridge::reorder(scene, id, row, true),
            FxRowAction::Down => crate::fx_bridge::reorder(scene, id, row, false),
            FxRowAction::Hide => crate::fx_bridge::toggle_enabled(scene, id, row),
            // Só alterna se o parâmetro for MESMO uma caixinha: o id é partilhado com o
            // slider, e um clique perdido num slider não pode virar um toggle silencioso.
            FxRowAction::Toggle(p) => {
                if crate::fx_bridge::is_toggle(scene, id, row, p) {
                    crate::fx_bridge::toggle_param(scene, id, row, p);
                }
            }
        }
    }
    if let Some((row, p, track)) = param {
        // ⚠️ **A caixinha e o slider partilham o id**, e o store regista SEMPRE um slider (o
        // `populate` regista o teto, antes de saber que efeito cai na linha). Então um press
        // numa caixinha também emite `ValueChanged` do slider, com o track = a posição
        // HORIZONTAL do cursor dentro do botão — e essa escrita vinha DEPOIS do flip acima.
        //
        // O resultado era o reportado (Enio, 2026-07-18): clicar repetidamente no mesmo sítio
        // dava sempre o mesmo estado, porque quem decidia era o *onde* do clique e não o flip.
        // Duas escritas para um fato: a do Click é a verdadeira, e esta tem de se recusar.
        if !crate::fx_bridge::is_toggle(scene, id, row, p) {
            crate::fx_bridge::set_param(scene, id, row, p, track);
        }
    }
}

#[cfg(test)]
#[path = "fx_bridge_dispatch_tests.rs"]
mod tests;
