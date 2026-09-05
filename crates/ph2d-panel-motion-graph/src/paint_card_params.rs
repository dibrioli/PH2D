//! ⭐⭐⭐ **OS PARAMS DESENHADOS NO CARTÃO** — o ciclo 1 da dinâmica
//! ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)), decisão do Enio de
//! 2026-09-05: *«como no Blender, os parâmetros dos nós devem ser desenhados nos nós e vamos
//! retirar o painel lateral»*.
//!
//! **Uma row = a faixa inteira do cartão**, com o rótulo à esquerda, o valor à direita e o
//! nível como PREENCHIMENTO da própria faixa — é o número do Blender (arrasta-se em qualquer
//! ponto) e não o par «rótulo em cima, calha em baixo» do Mini Cavalry, que gasta **duas**
//! linhas por param. Num cartão que vai hospedar até 24 params, a diferença é o dobro da
//! altura; e uma faixa inteira é também o maior alvo de toque possível (44 pt a partir de
//! `zoom 2`), que é a régua do tablet.
//!
//! ⚠️ **Este ficheiro é irmão do [`super::paint`] por RESPONSABILIDADE:** o pai desenha o que
//! um cartão É (moldura, cabeçalho, sockets, véu), este desenha o que ele CONTROLA. O pai
//! está a 589 linhas de um tecto de 700 — mas o corte é por assunto, não por contagem.

use crate::geom::{self, View};
use crate::snapshot::{CardParam, GraphNodeView};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::text_elide::paint_text_title_elided;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::ParamWidget;
use ph2d_tokens::{ColorToken, Theme};

/// Recuo da faixa em relação à borda do cartão — o mesmo dos dois lados, para a row ler como
/// uma peça POUSADA no cartão e não como uma banda que o atravessa.
const TRACK_INSET_X: f32 = 6.0; // LITERAL-PX-OK: card param track x-inset
/// Folga vertical dentro da fileira: a faixa não encosta na de cima nem na de baixo.
const TRACK_INSET_Y: f32 = 2.0; // LITERAL-PX-OK: card param track y-inset
/// Raio da faixa — o mesmo do cartão dividido por dois, para a peça pequena não parecer um
/// cartão pequeno.
const TRACK_R: f32 = 4.0; // LITERAL-PX-OK: card param track corner radius
/// Recuo do texto dentro da faixa.
const TEXT_PAD_X: f32 = 7.0; // LITERAL-PX-OK: card param text x-inset
/// Descida do texto dentro da fileira, para a linha de base ficar centrada.
const TEXT_PAD_Y: f32 = 5.0; // LITERAL-PX-OK: card param text y-inset

/// **O NÚMERO COMO O ARTISTA O LÊ.** O `step` do hint diz se o param é inteiro (um `Count` de
/// `3` nunca é `3.00`) e quantas casas um contínuo merece — a mesma lei do painel, e a razão
/// de ela viver aqui é que o cartão não tem acesso à row do painel.
fn value_text(p: &CardParam) -> String {
    if let ParamWidget::Enum { labels } = p.hint.widget {
        let i = p.value.round().max(0.0) as usize;
        return labels.get(i).map_or_else(|| p.value.to_string(), |s| (*s).to_string());
    }
    if matches!(p.hint.widget, ParamWidget::Toggle) {
        return if p.value >= 0.5 { "On" } else { "Off" }.to_string();
    }
    if p.hint.step >= 1.0 {
        return format!("{}", p.value.round() as i64);
    }
    // Duas casas é o que uma faixa de cartão comporta sem competir com o rótulo; o painel
    // (que tem largura) é quem mostra a precisão inteira.
    format!("{:.2}", p.value)
}

/// A fracção `0..1` da faixa que o valor preenche, ou `None` quando o param não é um NÍVEL
/// (um enum e um interruptor não têm «quanto», têm «qual»).
fn fill_fraction(p: &CardParam) -> Option<f32> {
    if matches!(p.hint.widget, ParamWidget::Enum { .. } | ParamWidget::Toggle) {
        return None;
    }
    let span = p.hint.max - p.hint.min;
    (span.abs() > f32::EPSILON).then(|| ((p.value - p.hint.min) / span).clamp(0.0, 1.0))
}

/// Desenha a faixa de params de um cartão. **Nada acontece abaixo do LOD**
/// ([`geom::params_are_drawn`]) — nem o desenho nem, do lado do hit-test, o registo: uma row
/// pintada onde não se clica é um controlo morto, e uma registada onde não se vê é um alvo
/// invisível.
pub(super) fn draw_card_params(
    ctx: &mut PaintCtx,
    n: &GraphNodeView,
    view: &View,
    theme: Theme,
) {
    if n.params.is_empty() || !geom::params_are_drawn(view) {
        return;
    }
    let z = view.zoom;
    for (i, p) in n.params.iter().enumerate() {
        let row = geom::param_row_rect(n, view, i);
        let track = Rect::new(
            row.x + TRACK_INSET_X * z,
            row.y + TRACK_INSET_Y * z,
            (row.w - 2.0 * TRACK_INSET_X * z).max(0.0),
            (row.h - 2.0 * TRACK_INSET_Y * z).max(0.0),
        );
        fill_rounded_rect(ctx.scene, track, TRACK_R * z, resolve(ColorToken::Bg0, theme));
        // O NÍVEL, dentro da mesma faixa. ⚠️ Um param DIRIGIDO não desenha nível: o número
        // vem de um fio e não obedece ao dedo — mostrar um nível arrastável seria a mentira
        // que o painel já aprendeu a não contar (a row dirigida, doc 88 B3).
        if !p.driven && let Some(f) = fill_fraction(p) && f > 0.0 {
            let fill = Rect::new(track.x, track.y, track.w * f, track.h);
            fill_rounded_rect(ctx.scene, fill, TRACK_R * z, resolve(ColorToken::AccentSoft, theme));
        }
        let text_y = row.y + TEXT_PAD_Y * z;
        let size = geom::PARAM_LABEL_SIZE * z;
        let value = value_text(p);
        // O VALOR primeiro, encostado à direita — ele é o que o artista procura, e alinhá-lo
        // à direita é o que faz uma coluna de números ler-se como uma coluna.
        let vw = ctx.text_system.prefix_width(&value, size);
        let value_x = track.x + track.w - TEXT_PAD_X * z - vw;
        let (label_tone, value_tone) = if p.driven {
            (ColorToken::Text3, ColorToken::PortValue)
        } else {
            (ColorToken::Text2, ColorToken::Text1)
        };
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            &value,
            value_x,
            text_y,
            size,
            vw,
            resolve(value_tone, theme),
        );
        // E o rótulo, elidido no espaço que SOBRA — quando os dois disputam o pixel, quem
        // encolhe é o nome, nunca o número.
        let label_w = (value_x - (track.x + TEXT_PAD_X * z) - TEXT_PAD_X * z).max(0.0);
        paint_text_title_elided(
            ctx.text_system,
            ctx.scene,
            p.hint.label,
            track.x + TEXT_PAD_X * z,
            text_y,
            size,
            label_w,
            resolve(label_tone, theme),
        );
    }
}
