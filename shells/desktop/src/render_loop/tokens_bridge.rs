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
use ph2d_tokens::color::Color;
use ph2d_tokens::num_overrides::{
    NumRefusal, NumValue, num_override, num_overrides, set_num_override, set_num_overrides,
};
use ph2d_tokens::overrides::{TokenValue, color_override, set_color_override, set_color_overrides};
use ph2d_tokens::{ColorToken, NumToken};

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
            //
            // ⚠️ E as DUAS famílias, porque o botão diz *"Reset This Mode"*: deixar a escala de pé
            // depois de um reset que se anuncia total é a metade que ninguém procura.
            TokensIntent::ResetAll => {
                let keep: Vec<_> = ph2d_tokens::overrides::color_overrides()
                    .into_iter()
                    .filter(|e| e.theme != theme)
                    .collect();
                // ⚠️ Filtrar uma tabela ACÍCLICA não pode produzir um laço (remover arestas nunca
                // fecha um ciclo), então o descarte é zero por aritmética — não por confiança.
                debug_assert_eq!(set_color_overrides(keep), 0);
                let keep_num: Vec<_> = num_overrides()
                    .into_iter()
                    .filter(|e| e.theme != theme)
                    .collect();
                debug_assert_eq!(set_num_overrides(keep_num), 0);
                changed = true;
            }

            // ── A família NUMÉRICA (plano UI/UX W4c.1) ────────────────────
            TokensIntent::NumReset(row) => {
                if let Some(&token) = NumToken::ALL.get(row) {
                    // Soltar nunca pode falhar: `None` não é um valor a validar nem um elo a fechar.
                    let _ = set_num_override(theme, token, None);
                    changed = true;
                }
            }
            // ⚠️ **Só escreve quando MUDA**, e o oráculo é o valor EFETIVO — a mesma lei do
            // read-back do picker. Sem ela, um chip que espelha o efetivo marcaria o projeto sujo
            // por o artista ter clicado nele.
            TokensIntent::NumSet { row, px } => {
                if let Some(&token) = NumToken::ALL.get(row) {
                    // ⚠️ Digitar um número numa linha que SEGUE outra QUEBRA o elo, mesmo que o
                    // número coincida: o artista foi ao campo e escreveu, e isso é a decisão de ter
                    // um valor próprio.
                    let breaks_a_link =
                        matches!(num_override(theme, token), Some(NumValue::Alias(_)));
                    if breaks_a_link || token.px(theme) != px {
                        match set_num_override(theme, token, Some(NumValue::Literal(px))) {
                            Ok(()) => changed = true,
                            Err(e) => {
                                toasts.push(refusal_toast(&e));
                            }
                        }
                    }
                }
            }
            // ⚠️ Uma fórmula VAZIA solta o token, e é o gesto que o artista faz: ele apaga o
            // campo. Tratá-la como fórmula inválida daria um toast a explicar um erro que ele não
            // cometeu; e um botão próprio de *"apagar fórmula"* seria a segunda porta do Reset,
            // que está na mesma linha.
            TokensIntent::NumFormula { row, ref src } => {
                if let Some(&token) = NumToken::ALL.get(row) {
                    let want = if src.trim().is_empty() {
                        None
                    } else {
                        Some(NumValue::Expr(src.trim().to_string()))
                    };
                    // ⚠️ Só escreve quando MUDA — a mesma lei do `NumSet` e do read-back do
                    // picker: um campo que perde o foco sem ter sido editado marcaria o projeto
                    // sujo por o artista ter olhado para ele.
                    if want != num_override(theme, token) {
                        match set_num_override(theme, token, want) {
                            Ok(()) => changed = true,
                            Err(e) => {
                                toasts.push(refusal_toast(&e));
                            }
                        }
                    }
                }
            }
            TokensIntent::NumLink { from, to } => {
                if let (Some(&a), Some(&b)) = (NumToken::ALL.get(from), NumToken::ALL.get(to)) {
                    match set_num_override(theme, a, Some(NumValue::Alias(b))) {
                        Ok(()) => changed = true,
                        Err(e) => {
                            toasts.push(refusal_toast(&e));
                        }
                    }
                }
            }
        }
    }

    // ── 3. A ESCALA vai para a forma de RUNTIME (plano UI/UX W4c.2) ───────
    //
    // ⚠️ **No FIM, e a posição é a lei inteira.** Acima, esta função pode ter escrito a camada
    // (um `NumSet` digitado neste quadro); a publicação projeta o grafo para a tabela plana que
    // todo widget lê. Publicar ANTES faria o quadro em que o artista digita um número pintar com
    // o valor anterior — o chip mostraria 20 e a tela mediria 12, e o número "piscaria de volta".
    //
    // ⚠️ **E mora AQUI, não num sítio próprio do laço**, porque esta é a única função que já é
    // dona das duas metades: a camada de override e o modo vigente. Um segundo sítio teria de
    // perguntar o modo outra vez, e duas respostas a *"que modo é este?"* é como o artista edita
    // um modo e vê outro re-vestir. A ordem está pinada num arch-gate — ela não é observável por
    // nenhum teste de unidade.
    ph2d_tokens::num_runtime::publish(theme);

    changed
}

/// **A recusa vira frase.** Um gesto que não acontece sem nada na tela é indistinguível de um botão
/// quebrado, e o que torna a mensagem accionável é ela nomear *o que* foi pedido e *por quê* não deu.
fn refusal_toast(e: &NumRefusal) -> Toast {
    match e {
        NumRefusal::Cycle { token, target, at } => Toast::warning(format!(
            "Can't make {} follow {}: that closes a loop at {}",
            token.key(),
            target.key(),
            at.key()
        )),
        NumRefusal::NotALength(v) => Toast::warning(format!(
            "{v} is not a length: a px token needs a finite value >= 0"
        )),
        // ⚠️ A frase vem do MOTOR e é repassada inteira: dobrá-la num texto genérico poria o
        // artista a adivinhar QUAL caractere não foi entendido, que é o oposto de accionável.
        NumRefusal::BadFormula(why) => Toast::warning(why.clone()),
    }
}

#[cfg(test)]
#[path = "tokens_bridge_tests.rs"]
mod tests;
