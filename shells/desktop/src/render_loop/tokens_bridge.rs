//! **A ponte do painel de TOKENS** (plano UI/UX W6, degrau 1) — a shell é o ÚNICO escritor da
//! camada de override.
//!
//! # Duas metades, e as duas têm de morar aqui
//!
//! 1. **O read-back do picker.** O clique numa swatch abre o OKLCH partilhado (dispatch genérico
//!    do `register_picker_swatch`), e o valor escolhido vive no `store` que a shell possui — o
//!    painel não o alcança. É o mesmo protocolo do `vector_bridge` para as swatches de Fill/Stroke.
//! 2. **Os intents** (`Reset`, `ResetAll`) — o painel enfileira, isto drena.
//!
//! ⚠️ **Deixar o painel escrever a camada daria DOIS escritores** para a mesma tabela, e o segundo
//! é sempre o que esquece de marcar o projeto como sujo.
//!
//! # O modo é o do HOST
//!
//! O override é do par `(modo, token)`, e o modo vigente é o `hero.theme` — a MESMA fonte que a
//! tecla `M` cicla e que o painel lê para pintar. Perguntá-lo a um segundo lugar faria o artista
//! editar um modo e ver outro re-vestir.

use ph2d_panel_tokens::TokensIntent;
use ph2d_tokens::ColorToken;
use ph2d_tokens::color::Color;
use ph2d_tokens::overrides::{set_color_override, set_color_overrides};

use ph2d_editor::screens::hero::HeroScreen;

/// Roda uma vez por frame, na MESMA fase das outras pontes de painel.
///
/// Devolve `true` se a camada mudou — o chamador usa para marcar o título como sujo, do mesmo modo
/// que qualquer outra edição de documento.
pub(crate) fn dispatch(hero: &mut HeroScreen) -> bool {
    let theme = hero.theme;
    let mut changed = false;

    // ── 1. Read-back do picker ────────────────────────────────────────────
    // ⚠️ A varredura é sobre `ColorToken::ALL` — o mesmo intervalo que o `populate` do painel
    // regista. Um teto que só um dos dois conhecesse deixaria as últimas linhas com o picker a
    // abrir e a cor a não chegar a lado nenhum.
    if let Some(target) = hero.store.picker_target()
        && let Some(row) =
            (0..ColorToken::ALL.len()).find(|&r| ph2d_editor::ids::tokens_swatch_id(r) == target)
        && let Some((value, _, _, _)) = hero
            .store
            .blender_picker(ph2d_editor::ids::INSP_BLENDER_PICKER)
    {
        let [r, g, b, a] = value.rgba;
        let picked = Color { r, g, b, a };
        // ⚠️ Só escreve quando MUDA: o picker publica o valor a cada frame em que está aberto, e
        // escrever sempre marcaria o projeto sujo por olhar para ele.
        if ColorToken::ALL[row].resolve(theme) != picked {
            set_color_override(theme, ColorToken::ALL[row], Some(picked));
            changed = true;
        }
    }

    // ── 2. Os intents do painel ───────────────────────────────────────────
    for intent in ph2d_panel_tokens::drain_intents() {
        match intent {
            TokensIntent::Reset(row) => {
                if let Some(&token) = ColorToken::ALL.get(row) {
                    set_color_override(theme, token, None);
                    changed = true;
                }
            }
            // ⚠️ Só o MODO VIGENTE. A lista é filtrada em vez de limpa: apagar os outros três
            // levaria trabalho que o artista não está a olhar.
            TokensIntent::ResetAll => {
                let keep: Vec<_> = ph2d_tokens::overrides::color_overrides()
                    .into_iter()
                    .filter(|e| e.theme != theme)
                    .collect();
                set_color_overrides(keep);
                changed = true;
            }
        }
    }
    changed
}

#[cfg(test)]
#[path = "tokens_bridge_tests.rs"]
mod tests;
