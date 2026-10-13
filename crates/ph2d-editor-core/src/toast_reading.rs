//! ⭐⭐⭐ **Um aviso é LIDO inteiro** ([doc 132 §5](../../../docs/Motion%20Nodes/132_zerar_as_seis_pendencias.md), P5).
//!
//! A lei de leitura do balão da [`crate::toast::ToastQueue`]: o texto QUEBRA à largura que recebe
//! de verdade ([`text_budget_px`], um orçamento só), até [`MAX_LINES`]; o balão
//! abarca as linhas da coluna que essa altura pede ([`rows_for_height`]); e vive o tempo de ser
//! lido ([`reading_ttl_s`]). O pintor e quem conta as linhas (a shell, para as barras de trabalho
//! descerem) usam a MESMA conta — [`shown`].

use crate::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{Spacing, TypeToken};

/// O máximo de linhas de texto de um balão — o mínimo que faz caber INTEIRA toda frase de aviso
/// do censo (`toast_reading_census`) em todo estilo de texto: a tabela pede até 5 (a recusa
/// `skeleton.recusa.varios_esqueletos`), uma frase crua de smoke pede 7 (Large × Atkinson).
/// Acima disto a última linha elide (e o censo das elisões vê-o).
pub const MAX_LINES: usize = 7;

/// Palavras por minuto da leitura silenciosa de prosa em inglês por adultos — a média da
/// meta-análise de Brysbaert (2019), *J. Memory and Language* 109, 104047.
pub const READING_WPM: f32 = 238.0;

/// O tempo de vida mínimo de um aviso, em segundos de PAREDE — o de sempre
/// ([`crate::toast::Toast::DEFAULT_TTL_S`]).
pub const MIN_TTL_S: f32 = 3.0;

/// O lado do ícone de severidade de um balão de aviso.
///
/// ⚠️ **Não há token para este tamanho, e a ausência é a nota:** o `chrome.inline-icon` (14) é o
/// glifo de uma linha e o `chrome.icon-btn-size` (36) é um botão — este está no meio, num balão
/// que não é nem linha nem botão. Fica NOMEADO em vez de escrito no meio da pintura.
// LITERAL-PX-OK: lado do ícone de severidade do balão de aviso
pub(crate) const ICON_PX: f32 = 24.0;

/// ⭐⭐⭐ **O ORÇAMENTO DE TEXTO de um balão de aviso** — a largura com que a mensagem é QUEBRADA.
///
/// ⛔⛔ **Ela existe por um report do dono** (22/09): *«as mensagens estão cortadas com … não
/// consigo ler tudo»*. Era a largura em que a frase tinha de caber numa linha; desde o doc 132 §5
/// é a largura a que ela quebra — e é a ÚNICA: o segundo desconto do `label_budget` (−2·`Md`)
/// que o `paint_text_centered` fazia por baixo dela saiu.
#[must_use]
pub fn text_budget_px() -> f32 {
    (crate::progress::toast_column_w()
        - Spacing::Xl.px() * 2.0
        - ICON_PX
        - ph2d_tokens::icon_label_gap_px())
    .max(0.0)
}

/// O tamanho da letra do balão.
#[must_use]
pub fn font_px() -> f32 {
    TypeToken::Base.px()
}

/// O que um balão pinta: o texto (inteiro, ou com a última linha elidida acima de
/// [`MAX_LINES`]), a largura a que quebra, quantas linhas, a altura do bloco e a da 1.ª linha
/// (onde o ícone se centra).
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub text: String,
    pub width: f32,
    pub lines: usize,
    pub height: f32,
    pub first_line_h: f32,
}

fn layout(ts: &mut TextSystem, text: &str) -> ph2d_text::Layout<()> {
    ts.layout_for_rendering(
        text,
        font_px(),
        text_budget_px(),
        FontWeight::MEDIUM,
        ph2d_text::active_text_rendering(),
    )
}

/// Quantas linhas a mensagem pede ao orçamento, SEM tecto, e se alguma passa da largura (uma
/// palavra que não quebra).
#[must_use]
pub fn lines_needed(ts: &mut TextSystem, message: &str) -> (usize, bool) {
    let budget = text_budget_px();
    let l = layout(ts, message);
    let larga = l
        .lines()
        .any(|ln| ln.metrics().advance - ln.metrics().trailing_whitespace > budget + 0.5);
    (l.len().max(1), larga)
}

/// ⭐ **A conta única** — o que o balão de `message` pinta.
#[must_use]
pub fn shown(ts: &mut TextSystem, message: &str) -> Shown {
    let l = layout(ts, message);
    let start = (l.len() > MAX_LINES)
        .then(|| l.get(MAX_LINES - 1).map(|ln| ln.text_range().start))
        .flatten();
    drop(l);
    let text = match start {
        None => message.to_string(),
        Some(i) => {
            let resto =
                crate::text_elide::fit(ts, message[i..].trim_start(), font_px(), text_budget_px());
            format!("{}{resto}", &message[..i])
        }
    };
    let l = layout(ts, &text);
    let first_line_h = l
        .lines()
        .next()
        .map_or(l.height(), |ln| ln.metrics().line_height);
    Shown {
        width: text_budget_px(),
        lines: l.len().max(1),
        height: l.height(),
        first_line_h,
        text,
    }
}

/// Quantas linhas da coluna um bloco de texto de altura `h` pede, com o recuo `Md` em cima e em
/// baixo. Uma linha de texto cabe numa linha da coluna (é o balão de sempre).
#[must_use]
pub fn rows_for_height(h: f32) -> usize {
    let pad = Spacing::Md.px() * 2.0;
    (1..=MAX_LINES)
        .find(|&r| crate::progress::column_span_h(r) - pad >= h)
        .unwrap_or(MAX_LINES)
}

/// Onde o bloco de texto e o ícone ficam dentro do cartão `r`: o `y` do texto (centrado) e o
/// `y` do centro da 1.ª linha.
#[must_use]
pub fn text_y(r: Rect, s: &Shown) -> (f32, f32) {
    let y = r.y + (r.h - s.height) * 0.5;
    (y, y + s.first_line_h * 0.5)
}

/// ⭐ **O tempo de vida de um aviso**: o de o LER à [`READING_WPM`], nunca abaixo dos
/// [`MIN_TTL_S`] de sempre.
#[must_use]
pub fn reading_ttl_s(message: &str) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "palavras de uma frase")]
    let palavras = message.split_whitespace().count() as f32;
    (palavras * 60.0 / READING_WPM).max(MIN_TTL_S)
}

#[cfg(test)]
#[path = "toast_reading_census.rs"]
mod census;
