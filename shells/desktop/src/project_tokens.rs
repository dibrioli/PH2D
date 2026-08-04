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
use ph2d_tokens::overrides::{ColorOverride, color_overrides, set_color_overrides};
use ph2d_tokens::{ColorToken, Theme};

/// Um token de cor autorado: **que modo, que token, que cor**.
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
    rgba: [u8; 4],
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
            rgba: [e.colour.r, e.colour.g, e.colour.b, e.colour.a],
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
pub(crate) fn install(saved: &[SavedToken]) {
    let mut dropped = 0usize;
    let list: Vec<ColorOverride> = saved
        .iter()
        .filter_map(|t| {
            let Some(token) = ColorToken::from_key(&t.key) else {
                dropped += 1;
                return None;
            };
            Some(ColorOverride {
                theme: theme_from_u8(t.theme),
                token,
                colour: Color {
                    r: t.rgba[0],
                    g: t.rgba[1],
                    b: t.rgba[2],
                    a: t.rgba[3],
                },
            })
        })
        .collect();
    set_color_overrides(list);
    if dropped > 0 {
        eprintln!(
            "[proj] {dropped} token(s) de cor do arquivo nao existem mais no design system e \
             foram descartados"
        );
    }
}

#[cfg(test)]
#[path = "project_tokens_tests.rs"]
mod tests;
