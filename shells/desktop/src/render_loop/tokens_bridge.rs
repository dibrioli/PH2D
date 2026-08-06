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
use ph2d_tokens::overrides::{TokenValue, color_override, set_color_override, set_color_overrides};

use ph2d_editor::screens::hero::HeroScreen;
use ph2d_editor::{Toast, ToastQueue};

/// Escreve um LITERAL. ⚠️ O `expect` documenta a propriedade no sítio: um literal TERMINA uma
/// cadeia de aliases, nunca a alonga, então a porta não tem como o recusar.
fn put(theme: ph2d_tokens::Theme, token: ColorToken, colour: Option<Color>) {
    set_color_override(theme, token, colour.map(TokenValue::Literal))
        .expect("um literal nunca fecha um laco de alias");
}

/// Roda uma vez por frame, na MESMA fase das outras pontes de painel.
///
/// Devolve `true` se a camada mudou — o chamador usa para marcar o título como sujo, do mesmo modo
/// que qualquer outra edição de documento.
pub(crate) fn dispatch(hero: &mut HeroScreen, toasts: &mut ToastQueue) -> bool {
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
        let token = ColorToken::ALL[row];
        // ⚠️ Escolher uma cor numa linha que SEGUE outra QUEBRA o elo, mesmo que a cor coincida —
        // o artista abriu o picker e escolheu, e isso é a decisão de ter um valor próprio. Sem esta
        // metade, picar exactamente a cor que o alias já mostrava seria um clique sem efeito.
        let breaks_a_link = matches!(color_override(theme, token), Some(TokenValue::Alias(_)));
        // ⚠️ Só escreve quando MUDA: o picker publica o valor a cada frame em que está aberto, e
        // escrever sempre marcaria o projeto sujo por olhar para ele.
        if breaks_a_link || token.resolve(theme) != picked {
            put(theme, token, Some(picked));
            changed = true;
        }
    }

    // ── 2. Os intents do painel ───────────────────────────────────────────
    for intent in ph2d_panel_tokens::drain_intents() {
        match intent {
            TokensIntent::Reset(row) => {
                if let Some(&token) = ColorToken::ALL.get(row) {
                    put(theme, token, None);
                    changed = true;
                }
            }
            // ⚠️ **A recusa é DITA.** Um laço não tem valor a devolver, então a porta o recusa — e
            // um gesto que não acontece sem nada na tela é indistinguível de um botão quebrado.
            // O toast nomeia os dois tokens, que é o que torna o "por quê" acionável.
            TokensIntent::Link { from, to } => {
                if let (Some(&a), Some(&b)) = (ColorToken::ALL.get(from), ColorToken::ALL.get(to)) {
                    match set_color_override(theme, a, Some(TokenValue::Alias(b))) {
                        Ok(()) => changed = true,
                        Err(e) => {
                            toasts.push(Toast::warning(format!(
                                "Can't make {} follow {}: that closes a loop at {}",
                                e.token.key(),
                                e.target.key(),
                                e.at.key()
                            )));
                        }
                    }
                }
            }
            // ⚠️ Só o MODO VIGENTE. A lista é filtrada em vez de limpa: apagar os outros três
            // levaria trabalho que o artista não está a olhar.
            TokensIntent::ResetAll => {
                let keep: Vec<_> = ph2d_tokens::overrides::color_overrides()
                    .into_iter()
                    .filter(|e| e.theme != theme)
                    .collect();
                // ⚠️ Filtrar uma tabela ACÍCLICA não pode produzir um laço (remover arestas nunca
                // fecha um ciclo), então o descarte é zero por aritmética — não por confiança.
                debug_assert_eq!(set_color_overrides(keep), 0);
                changed = true;
            }
        }
    }
    changed
}

#[cfg(test)]
#[path = "tokens_bridge_tests.rs"]
mod tests;
