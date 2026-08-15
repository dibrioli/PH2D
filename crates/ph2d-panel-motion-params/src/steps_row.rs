//! O editor de **PASSOS** — uma lista ordenada de números desenhada como uma faixa de
//! **barras arrastáveis** com `+`/`−`, o idioma do sequenciador de passos.
//!
//! O irmão NUMÉRICO de [`crate::palette_row`], e deliberadamente da mesma família: uma
//! lista não tem posições nem interpolação, então ela não é uma curva ([`crate::curve_row`])
//! nem um gradiente ([`crate::gradient_row`]) — o que ela tem é *quantos* e *quais*. A
//! faixa **ENVOLVE**, então a row não impõe teto de comprimento; o único teto é o de
//! RECURSO ([`ph2d_steps::MAX_ENTRIES`], o buffer do device).
//!
//! ⚠️ **A string é a fonte da verdade, e o strip é uma FACE dela** — ele parseia por
//! `ph2d_steps::parse`, mexe num elemento e reescreve pela inversa EXATA
//! (`ph2d_steps::format`), então os valores que o artista digitou e não arrastou
//! sobrevivem bit a bit. Sem esse par de portas o widget teria a própria ideia de como um
//! número se escreve, e um arrasto reformataria a lista inteira.
//!
//! ⚠️ **A altura da barra é lida na faixa `min..max` DO HINT**, nunca auto-ajustada ao
//! conteúdo: uma faixa que se re-escala enquanto se arrasta **não acompanha o dedo** (a
//! armadilha que o ADR-0128 pagou cinco vezes). Um valor fora da faixa é DESENHADO
//! saturado e **não é reescrito** — só a barra que o dedo pega muda de valor.
//!
//! ⚠️ **E a faixa não é a única porta:** o checkbox `Type` troca o strip pelo campo de
//! TEXTO cru (o precedente do `Custom…` do [`ph2d_node_registry::ParamWidget::Channels`]),
//! porque um número exato se digita e uma lista vinda de fora se cola. Uma face de cada
//! vez — nunca as duas, que seriam duas portas para o mesmo valor.

use crate::snapshot::{
    StepsRow, param_checkbox_id, param_steps_add_id, param_steps_bar_id, param_steps_editor_id,
    param_steps_remove_id, param_text_id,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::math::safe_clamp;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_centered, resolve};
use ph2d_editor_core::widget::{Checkbox, CheckboxState, CheckboxValue, paint_checkbox};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

const BAR_W: f32 = 14.0; // LITERAL-PX-OK: largura de uma barra de passo
const BAR_H: f32 = 44.0; // LITERAL-PX-OK: altura da faixa de barras
const BTN_W: f32 = 22.0; // LITERAL-PX-OK: largura do +/-
const TYPE_W: f32 = 62.0; // LITERAL-PX-OK: o checkbox `Type` (caixa + rótulo)

/// A largura de um pincel de 256 barras — o degrau em que o endereço de 8 bits do
/// [`ph2d_editor_core::interaction::InteractiveState::CurvePoint`] vira 16.
const PAGE: usize = 256;

/// Os widgets que o passe de PINTURA (store imutável) devolve para o passe MUTÁVEL
/// registrar — o espelho exato de [`crate::curve_row::CurveWidgets`], com um campo a mais.
///
/// ⚠️ **O `CurvePoint` endereça o ponto com DOIS `u8` (`channel`, `index`) e a lista vai a
/// 1024**, então o índice viaja partido: `channel` é a PÁGINA de 256 e `index` o resto.
/// Não é abuso do campo — para um strip que ENVOLVE o endereço é naturalmente de duas
/// partes, e [`pack`]/[`unpack`] são a porta única dos dois lados (gate de round-trip).
pub(crate) struct StepsWidgets {
    /// `(id da barra, parent = o editor da row, página, índice na página, canvas)`
    pub points: Vec<(NodeId, NodeId, u8, u8, Rect)>,
    pub buttons: Vec<NodeId>,
}

impl StepsWidgets {
    pub(crate) fn new() -> Self {
        Self {
            points: Vec::new(),
            buttons: Vec::new(),
        }
    }
}

/// O índice `i` partido no par `(página, resto)` que o `CurvePoint` carrega.
pub(crate) fn pack(i: usize) -> (u8, u8) {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "i < MAX_ENTRIES = 1024, então página < 4 e resto < 256"
    )]
    ((i / PAGE) as u8, (i % PAGE) as u8)
}

/// A inversa de [`pack`] — o que a drenagem do arrasto lê de volta.
pub(crate) fn unpack(page: u8, index: u8) -> usize {
    page as usize * PAGE + index as usize
}

/// Quantas barras cabem numa linha de largura `w`. Pelo menos uma — uma row mais estreita
/// que uma barra ainda tem de desenhar alguma coisa, e uma-por-linha é o degenerado
/// honesto (a lei do `per_line` da paleta).
pub(crate) fn per_line(w: f32) -> usize {
    let gap = Spacing::Xs.px();
    (((w + gap) / (BAR_W + gap)) as usize).max(1)
}

/// A fração `0..1` que a barra desenha para o valor `v` na faixa do hint.
///
/// Faixa degenerada (`max <= min`) desenha vazio em vez de dividir por zero — um hint sem
/// faixa é um hint mal declarado, e o strip prefere não mentir sobre a altura.
fn frac(v: f32, min: f32, max: f32) -> f32 {
    let span = max - min;
    if span <= 0.0 || !span.is_finite() {
        0.0
    } else {
        ((v - min) / span).clamp(0.0, 1.0)
    }
}

/// O valor que uma barra arrastada até a fração `f` passa a ter.
fn value_at_frac(f: f32, min: f32, max: f32) -> f32 {
    // ⚠️ `safe_clamp`, não `f32::clamp`: os limites saem do HINT (não são literais), então
    // um hint invertido ou NaN entraria num `clamp` que PANICA (`ph2d_editor_core::math`).
    safe_clamp(min + f * (max - min), min, max)
}

/// Pinta a row e recolhe as registrações de store. Devolve a ALTURA usada.
///
/// A lista de argumentos espelha [`crate::palette_row::paint_palette_row`] uma a uma: as
/// duas são chamadas do mesmo braço de dispatch, e uma assinatura diferente aqui seria uma
/// segunda convenção para o mesmo trabalho.
#[expect(
    clippy::too_many_arguments,
    reason = "espelha a porta de paint das outras rows de editor"
)]
pub(crate) fn paint_steps_row(
    row: &StepsRow,
    slot: usize,
    x: f32,
    w: f32,
    y: f32,
    label_font: f32,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    out: &mut StepsWidgets,
) -> f32 {
    let gap = Spacing::Xs.px();
    let values = ph2d_steps::parse(&row.value);

    // ── Cabeçalho: rótulo (esquerda) + `Type` + `+` / `−` (direita) ──
    paint_text(
        text_system,
        scene,
        &row.label,
        x,
        y + (ROW_H_PX - label_font) * 0.5,
        label_font,
        (w - TYPE_W - BTN_W * 2.0 - gap * 3.0).max(0.0), // LITERAL-PX-OK: CONTAGEM (3 vaos)
        resolve(ColorToken::Text2, theme),
    );
    let rem = Rect::new(x + w - BTN_W, y, BTN_W, ROW_H_PX);
    let add = Rect::new(rem.x - BTN_W - gap, y, BTN_W, ROW_H_PX);
    for (brect, label, id) in [
        (add, "+", param_steps_add_id(slot)),
        (rem, "\u{2212}", param_steps_remove_id(slot)),
    ] {
        fill_rounded_rect(
            scene,
            brect,
            Radius::Sm.px(),
            resolve(ColorToken::Bg2, theme),
        );
        paint_text_centered(
            text_system,
            scene,
            label,
            brect,
            TypeToken::Base.px(),
            resolve(ColorToken::Text1, theme),
        );
        hit_index.register(id, brect);
        out.buttons.push(id);
    }

    // O escape do power-user: `Type` troca o strip pelo campo de texto cru.
    let cb_id = param_checkbox_id(slot);
    let (cb_state, cb_value) = store
        .checkbox(cb_id)
        .unwrap_or((CheckboxState::Normal, CheckboxValue::Unchecked));
    let raw = cb_value == CheckboxValue::Checked;
    let cb_rect = Rect::new(add.x - TYPE_W - gap, y, TYPE_W, ROW_H_PX);
    let cb = Checkbox::new(cb_id, "Type".to_string())
        .state(cb_state)
        .value(cb_value);
    paint_checkbox(&cb, cb_rect, scene, text_system, theme);
    hit_index.register(cb_id, cb_rect);

    let mut used = ROW_H_PX + gap;
    if raw {
        // A FACE crua: o mesmo `TextInput` da row de fórmula, no id de texto do slot (uma
        // row ocupa UM slot, então o id agrupado está livre — o padrão do `Custom…`).
        used += crate::text_rows::paint_text_row(
            Rect::new(x, y + used, w, ROW_H_PX),
            "",
            "0.1 0.5 0.9",
            param_text_id(slot),
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        return used;
    }

    // ── A faixa de barras, ENVOLVIDA. A altura segue a contagem; nada aqui a capa. ──
    let cols = per_line(w);
    let track = resolve(ColorToken::Bg2, theme);
    let fill = resolve(ColorToken::Accent, theme);
    // O ZERO da faixa: a barra cresce a partir dele, então uma lista com sinal desenha para
    // baixo o que é negativo em vez de mentir com uma barra curta.
    let base_f = frac(0.0, row.min, row.max);
    for (i, v) in values.iter().enumerate() {
        let (line, col) = (i / cols, i % cols);
        #[expect(
            clippy::cast_precision_loss,
            reason = "índices de grade; uma lista longa o bastante para perder precisão \
                      aqui não caberia em tela nenhuma"
        )]
        let cell = Rect::new(
            x + col as f32 * (BAR_W + gap),
            y + used + line as f32 * (BAR_H + gap),
            BAR_W,
            BAR_H,
        );
        fill_rounded_rect(scene, cell, Radius::Sm.px(), track);
        let f = frac(*v, row.min, row.max);
        let (lo, hi) = if f >= base_f {
            (base_f, f)
        } else {
            (f, base_f)
        };
        let top = cell.y + (1.0 - hi) * cell.h;
        let h = ((hi - lo) * cell.h).max(1.0); // LITERAL-PX-OK: piso de 1 px — um valor no zero ainda se vê
        fill_rounded_rect(
            scene,
            Rect::new(cell.x, top, cell.w, h),
            Radius::Sm.px(),
            fill,
        );
        let id = param_steps_bar_id(row.name, i);
        hit_index.register(id, cell);
        let (page, index) = pack(i);
        out.points
            .push((id, param_steps_editor_id(slot), page, index, cell));
    }
    let lines = values.len().div_ceil(cols);
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de linhas; ver a nota da grade acima"
    )]
    {
        used += lines as f32 * (BAR_H + gap);
    }
    used
}

/// Uma barra arrastada pousou no slot `curve_point_drag` do store — dobra o `y` no valor
/// daquele passo e emite a lista re-serializada. `None` quando o slot está vazio.
///
/// ⚠️ **Só o elemento arrastado muda**, e é o round-trip exato do `ph2d_steps` que garante
/// isso: os demais voltam da string bit a bit.
pub(crate) fn drain_drag(store: &mut WidgetStore, slot: usize, row: &StepsRow) -> Option<String> {
    // O stash é um canal GLOBAL: a pergunta de posse é parte da chamada, então um arrasto
    // de outro painel é DEIXADO para ele (a lei do `take_curve_point_drag_if`).
    let (_parent, page, index, _x, y) =
        store.take_curve_point_drag_if(|p| p == param_steps_editor_id(slot))?;
    let mut values = ph2d_steps::parse(&row.value);
    let i = unpack(page, index);
    if i >= values.len() {
        return None;
    }
    values[i] = value_at_frac(y, row.min, row.max);
    Some(ph2d_steps::format(&values))
}

/// `+` — acrescenta um passo, repetindo o ÚLTIMO valor.
///
/// Repetir (em vez de semear um default) é o que faz o gesto ler como *"mais um destes"*:
/// o artista arrasta o novo para onde quiser, e o padrão não salta de forma ao crescer.
/// Uma lista vazia nasce no meio da faixa, que é o único valor que não presume nada.
pub(crate) fn add_step(row: &StepsRow) -> String {
    let mut values = ph2d_steps::parse(&row.value);
    if values.len() >= ph2d_steps::MAX_ENTRIES {
        return row.value.clone();
    }
    let next = values
        .last()
        .copied()
        .unwrap_or_else(|| value_at_frac(0.5, row.min, row.max));
    values.push(next);
    ph2d_steps::format(&values)
}

/// `−` — tira o ÚLTIMO passo.
///
/// ⚠️ **Tirar o último passo devolve a string VAZIA**, que é o sinal de *nada autorado*: o
/// nó volta ao caminho legado e o painel volta a pintar os controles dele. Sem isso a row
/// ficaria numa lista de zero elementos — um estado que desenha nada e não é o legado.
pub(crate) fn remove_step(row: &StepsRow) -> String {
    let mut values = ph2d_steps::parse(&row.value);
    values.pop();
    ph2d_steps::format(&values)
}

#[cfg(test)]
#[path = "steps_row_tests.rs"]
mod tests;
