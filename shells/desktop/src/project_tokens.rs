//! **A tabela de COR autorada viaja no arquivo** (plano UI/UX W6, degrau 1) — irmão do
//! [`crate::project`] pelo teto de LOC (HR-18), e o corte é por assunto: aqui mora *como uma
//! re-vestida do design system sobrevive a um save*.
//!
//! # Ela fica FORA do `ProjectState`, e não é arrumação
//!
//! O `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não deve rebobinar a cara do
//! editor — o mesmo motivo que mantém `physics`, `motion` e `timeline` fora dele. O preço honesto
//! é que **editar um token não entra na fila do Ctrl+Z**; quem desfaz é o *Reset* da linha, que é
//! o que o painel de física também faz.
//!
//! # A chave, nunca o índice
//!
//! O que viaja é a **chave do token no `tokens.json`** (`"accent"`, `"bg-0"`). Guardar o índice do
//! variant amarraria todo projeto salvo à ORDEM da lista, e acrescentar um token no meio da tabela
//! re-pintaria o app com as cores trocadas — a mesma lei que a W4a aplicou ao binding.

use ph2d_tokens::color::Color;
use ph2d_tokens::overrides::{ColorOverride, TokenValue, color_overrides, set_color_overrides};
use ph2d_tokens::{ColorToken, Theme};

/// **O que um token autorado vale, no arquivo** — as duas espécies do [`TokenValue`].
///
/// ⚠️ Um ENUM, e não um `rgba` com um `alias: Option<String>` ao lado: os dois campos seriam
/// mutuamente exclusivos e nada no formato o diria, então um arquivo poderia trazer os dois e o
/// leitor teria de escolher um vencedor que ninguém especificou. A representação apaga o caso.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum SavedValue {
    /// Uma cor literal.
    Literal([u8; 4]),
    /// **A CHAVE do token seguido** — nunca o índice, a mesma lei do campo `key` abaixo.
    Alias(String),
}

/// Um token de cor autorado: **que modo, que token, valendo o quê**.
///
/// ⚠️ Tipo PRÓPRIO do arquivo, e não o [`ColorOverride`] da `ph2d-tokens` — a crate de tokens não
/// depende de `serde` para o valor de runtime dela, e fazer o formato do arquivo herdar o layout
/// de um tipo de runtime é o que torna um refactor interno numa quebra de save.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedToken {
    /// O modo (`Theme as u8`) — o discriminante é estável porque o enum é append-only.
    theme: u8,
    /// A chave do token no `tokens.json` (`"accent"`, `"bg-0"`, …).
    key: String,
    value: SavedValue,
}

/// O modo a partir do byte guardado. **Porta única** da direção inversa do `theme as u8`.
///
/// ⚠️ Um byte que o enum não tem cai no default (`Forge`) em vez de recusar o arquivo inteiro: um
/// modo desconhecido é um override que não se sabe mostrar, e jogar fora o projeto por causa dele
/// seria a resposta errada — o `PROJECT_SCHEMA` é quem recusa formato, não um campo.
const fn theme_from_u8(b: u8) -> Theme {
    match b {
        1 => Theme::Workshop,
        2 => Theme::Sunstone,
        3 => Theme::Blueprint,
        _ => Theme::Forge,
    }
}

/// O que o save grava.
///
/// ⚠️ Sai da PORTA (`color_overrides()`), que já devolve a lista em ordem canônica: dois projetos
/// logicamente iguais dão os mesmos bytes, seja qual for a ordem dos cliques.
pub(crate) fn collect() -> Vec<SavedToken> {
    color_overrides()
        .into_iter()
        .map(|e| SavedToken {
            theme: e.theme as u8,
            key: e.token.key().to_string(),
            value: match e.value {
                TokenValue::Literal(c) => SavedValue::Literal([c.r, c.g, c.b, c.a]),
                TokenValue::Alias(t) => SavedValue::Alias(t.key().to_string()),
            },
        })
        .collect()
}

/// **A tabela do documento anterior morre aqui, e a do arquivo entra.**
///
/// ⚠️ Instalar a lista INTEIRA (em vez de acrescentar) é o que faz o load ESQUECER: sem isto,
/// abrir um projeto de fábrica depois de um re-vestido deixaria o app com as cores do documento
/// ANTERIOR, e nada na tela diria porquê — a mesma classe da timeline.
///
/// Um token que o `tokens.json` já não tem é **DESCARTADO** (o `from_key` devolve `None`): a
/// tabela de fábrica é a autoridade sobre quais tokens existem. E o que cai é **DITO** — uma
/// tabela que encolhe em silêncio lê-se como *"eu nunca autorei isto"*, e o artista procuraria a
/// cor onde ela não está.
///
/// Devolve **quantas entradas do arquivo não puderam entrar** — o mesmo número que a mensagem diz.
/// ⚠️ Duas representações do MESMO fato, para dois públicos: a linha de log é para a pessoa que
/// abriu o projeto, o retorno é para o gate. Sem ele, um descarte que deixasse de ser contado
/// passaria com a suíte verde, que é precisamente o *encolher em silêncio* que este doc proíbe.
pub(crate) fn install(saved: &[SavedToken]) -> usize {
    let mut dropped = 0usize;
    let list: Vec<ColorOverride> = saved
        .iter()
        .filter_map(|t| {
            let Some(token) = ColorToken::from_key(&t.key) else {
                dropped += 1;
                return None;
            };
            // ⚠️ Um alias cujo ALVO já não existe cai pelo mesmo motivo que o token desconhecido:
            // a tabela de fábrica é a autoridade sobre quais tokens existem, e um elo pendurado no
            // vazio não tem valor a devolver.
            let value = match &t.value {
                SavedValue::Literal([r, g, b, a]) => TokenValue::Literal(Color {
                    r: *r,
                    g: *g,
                    b: *b,
                    a: *a,
                }),
                SavedValue::Alias(key) => match ColorToken::from_key(key) {
                    Some(target) => TokenValue::Alias(target),
                    None => {
                        dropped += 1;
                        return None;
                    }
                },
            };
            Some(ColorOverride {
                theme: theme_from_u8(t.theme),
                token,
                value,
            })
        })
        .collect();
    // ⚠️ A porta descarta os elos que fechariam um laço e DIZ quantos — um arquivo editado à mão
    // pode trazer um, e recusar o projeto inteiro por causa dele seria jogar fora a re-vestida.
    dropped += set_color_overrides(list);
    if dropped > 0 {
        eprintln!(
            "[proj] {dropped} token(s) de cor do arquivo nao existem mais no design system (ou \
             fechavam um laco de alias) e foram descartados"
        );
    }
    dropped
}

#[cfg(test)]
#[path = "project_tokens_tests.rs"]
mod tests;
