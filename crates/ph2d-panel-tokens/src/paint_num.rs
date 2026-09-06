//! **A FAMÍLIA NUMÉRICA do painel** — uma linha por token de `NumToken::ALL` (plano UI/UX W4c.1).
//!
//! Irmão do [`crate::paint`] pelo teto de LOC (HR-18), e o corte é por **assunto**: lá mora o
//! chrome do painel e a lista de COR, aqui a lista que se mede em px. As duas percorrem a sua
//! própria tabela de compile-time, e nenhuma tem uma cópia da outra.
//!
//! # A linha é a MESMA, e só o editor muda
//!
//! `[chip] chave [→ alvo] [elo] [Reset]` contra o `[swatch] chave [→ alvo] [⚠] [elo] [Reset]` da
//! cor. É deliberado: as duas respondem à mesma pergunta (*quanto vale este token?*) e a única
//! diferença honesta é a grandeza — então o que muda é o **editor daquela grandeza**, e o resto do
//! gesto (autorar, seguir, resetar) o artista aprende uma vez.
//!
//! ⚠️ **Não há marca de contraste aqui, e a ausência é a decisão.** A WCAG fala de luminância entre
//! duas cores; um espaçamento não participa de nenhum par. Uma coluna reservada para uma marca que
//! nunca acende seria largura roubada ao rótulo em todas as 21 linhas.
//!
//! # UM SLOT, UM EDITOR (plano UI/UX W4c.3)
//!
//! Uma linha mostra o chip de px **ou** o campo de fórmula, nunca os dois. Eles editam o MESMO
//! valor por caminhos que se excluem — digitar `20` no chip de uma linha que carrega
//! `{spacing.md} * 2` destruiria a fórmula em silêncio —, e dois editores para um valor é a mesma
//! falha de duas-portas que esta camada recusa em todos os outros lugares.
//!
//! Com o campo aberto, o slot do chip vira **readout**: o número que a fórmula dá, em texto. É a
//! mesma promessa do chip (*mostrar o valor EFETIVO*) sem a mentira de parecer editável.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, format_number};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, IconButtonStyle, IconGlyph, TextInput, TextInputState, paint_icon_button,
    paint_number_chip, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::num_expr::math_available;
use ph2d_tokens::num_overrides::{NumValue, num_override};
use ph2d_tokens::{ColorToken, NumToken, ROW_H_PX, Spacing, Theme, TypeToken};

use crate::paint::{LINK_W, RESET_W, command};

/// Largura do chip de px. Larga o suficiente para `999` mais as setas do stepper.
const CHIP_W: f32 = 56.0; // LITERAL-PX-OK: panel grid metric (numeric chip width)

/// A lista numérica inteira, com o seu cabeçalho. Devolve o `y` depois dela.
pub(crate) fn paint_numeric_family(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    w: f32,
    mut y: f32,
    armed: Option<usize>,
    fx_open: Option<usize>,
) -> f32 {
    // ⚠️ O cabeçalho diz a UNIDADE. Sem ele, um chip com `8` logo abaixo de uma coluna de swatches
    // não diz de que grandeza se está a falar — e a unidade é exactamente a razão de as três
    // escalas (espaço, raio, traço) serem UMA família e partilharem este editor.
    let font = TypeToken::Sm.px();
    y += Spacing::Md.px();
    paint_text(
        ctx.text_system,
        ctx.scene,
        tr("panel.tokens.numeric"),
        x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    y += ph2d_tokens::row_pitch_px();

    for (row, &token) in NumToken::ALL.iter().enumerate() {
        y = paint_num_row(
            ctx,
            theme,
            row,
            token,
            Rect::new(x, y, w, ROW_H_PX),
            armed == Some(row),
            fx_open == Some(row),
        );
    }
    y
}

/// `[chip]  chave-do-token  [→ alvo]      [elo] [Reset]`
fn paint_num_row(
    ctx: &mut PaintCtx,
    theme: Theme,
    row: usize,
    token: NumToken,
    box_: Rect,
    armed: bool,
    fx_open: bool,
) -> f32 {
    let (x, w, y) = (box_.x, box_.w, box_.y);
    let slot = num_override(theme, token);
    let authored = slot.is_some();
    let font = TypeToken::Sm.px();

    // ⚠️ A fórmula AUTORADA domina o gesto: o campo é o editor daquela linha, então ele existe
    // enquanto ela existir, tenha o artista clicado no `f(x)` ou não.
    let formula = match &slot {
        Some(NumValue::Expr(src)) => Some(src.clone()),
        _ => None,
    };
    let field_open = formula.is_some() || fx_open;
    // O `f(x)` só se oferece a quem ainda pode ganhar uma fórmula (§ cabeçalho), e só quando há
    // como responder sobre fórmulas — sem host instalado o controlo NÃO EXISTE, em vez de existir
    // e não fazer nada (o padrão do `set_ml_available`).
    let show_fx = math_available() && formula.is_none();

    // ⚠️ O chip mostra o valor EFETIVO (o que o app usaria), nunca o de fábrica sob um token
    // autorado nem o alvo cru sob um elo: um campo que afirmasse um número que o desenho não usa é
    // a mesma rachura que a swatch de cor documenta uma lista acima.
    let value = token.px(theme);
    let chip_rect = Rect::new(x, y, CHIP_W, ROW_H_PX);
    if field_open {
        // O readout: o mesmo número, sem parecer editável.
        paint_text(
            ctx.text_system,
            ctx.scene,
            &format_number(f64::from(value)),
            chip_rect.x,
            y + (ROW_H_PX - font) * 0.5,
            font,
            CHIP_W,
            resolve(ColorToken::Text2, theme),
        );
    } else {
        paint_px_chip(ctx, theme, chip_rect, ids::tokens_num_chip_id(row), value);
    }

    let label_x = x + CHIP_W + Spacing::Sm.px();
    let tail = if authored { RESET_W } else { 0.0 }
        + LINK_W
        + if show_fx { LINK_W } else { 0.0 }
        + Spacing::Xs.px();
    let label_w = (w - CHIP_W - Spacing::Sm.px() - tail).max(1.0);
    let label_token = if authored {
        ColorToken::Accent
    } else {
        ColorToken::Text1
    };
    // Uma linha que SEGUE outra tem de DIZER quem ela segue — senão o artista vê um número que não
    // obedece ao que ele digita, sem nada a explicar porquê.
    let label = match slot {
        Some(NumValue::Alias(target)) => format!("{}  -  {}", token.key(), target.key()),
        _ => token.key().to_string(),
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        label_x,
        y + (ROW_H_PX - font) * 0.5,
        font,
        label_w,
        resolve(label_token, theme),
    );

    let link_x = x + w - LINK_W - if authored { RESET_W } else { 0.0 };
    paint_icon_row_button(
        ctx,
        theme,
        ids::tokens_num_link_id(row),
        IconId::Link,
        link_x,
        y,
        armed,
    );
    if show_fx {
        paint_icon_row_button(
            ctx,
            theme,
            ids::tokens_num_fx_id(row),
            // ⚠️ `Script` e não um glifo novo: ele já significa *uma regra escrita que computa* —
            // é o mesmo que o componente de script usa — e é uma FIGURA distinta do elo ao lado
            // (o gate de ícone compara geometria, nunca o identificador: o par `Layer`/`Layers` da
            // timeline foi reprovado num smoke exactamente por isso). Um glifo próprio é decisão
            // do design system (§7), não de um botão.
            IconId::Script,
            link_x - LINK_W,
            y,
            fx_open,
        );
    }

    if authored {
        command(
            ctx,
            ids::tokens_num_reset_id(row),
            tr("panel.tokens.reset"),
            x + w - RESET_W,
            RESET_W,
            y,
        );
    }
    let mut y = y + ph2d_tokens::row_pitch_px();

    // ⚠️ O campo é uma SEGUNDA linha, de largura cheia, e não um editor apertado no lugar do chip:
    // `{spacing.md} * 2` não cabe em 56 px, e um campo que corta o que o artista escreveu é um
    // campo que o faz digitar às cegas. É o mesmo empilhamento que a `motion.expression` usa.
    if field_open {
        y = paint_formula_field(
            ctx,
            theme,
            ids::tokens_num_formula_id(row),
            Rect::new(x, y, w, ROW_H_PX),
            formula.as_deref().unwrap_or_default(),
        );
    }
    y
}

/// O campo de fórmula da linha. Devolve o `y` depois dele.
///
/// ⚠️ O espelho do texto autorado é gateado no FOCO, exactamente como o do chip: sem isso cada
/// quadro reescreveria por cima do que está a ser digitado; com isso, uma fórmula RECUSADA pela
/// porta volta sozinha ao texto autorado quando o campo perde o foco — o *"não pegou"* fica
/// visível, e o toast diz porquê.
fn paint_formula_field(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    rect: Rect,
    authored: &str,
) -> f32 {
    let store = ctx.host.store_mut();
    let _ = store.register_if_absent(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: authored.to_string(),
            caret: authored.len(),
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) = store.get_mut(id)
    {
        if text != authored {
            text.clear();
            text.push_str(authored);
        }
        *caret = (*caret).min(text.len());
        *selection_anchor = None;
    }
    let (state, text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    let input = TextInput::new(id, "")
        .placeholder(tr("panel.tokens.formula.hint"))
        .visual((state, store.hover_live(id)));
    paint_text_input_with_buffer(
        &input,
        Some(&text),
        Some(caret),
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
    rect.y + ph2d_tokens::row_pitch_px()
}

/// Pinta o chip de px e **espelha** o valor efetivo nele enquanto ninguém o está a editar.
///
/// ⚠️ O espelho é gateado no FOCO, e é o que separa *"o app diz quanto vale"* de *"o artista está a
/// dizer quanto quer"*: sem ele, cada frame reescreveria o buffer por cima do que está a ser
/// digitado. Com ele, um valor recusado pela porta volta sozinho ao efetivo quando o campo perde o
/// foco — o "não pegou" fica visível, e o toast diz porquê.
fn paint_px_chip(ctx: &mut PaintCtx, theme: Theme, rect: Rect, id: ph2d_a11y::NodeId, value: f32) {
    let text = format_number(f64::from(value));
    let store = ctx.host.store_mut();
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::NumberInput {
            value: v,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
    {
        *v = f64::from(value);
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = f64::from(value);
    }
    let (st, buf, caret, anchor) = match store.get(id) {
        Some(InteractiveState::NumberInput {
            state,
            buffer,
            caret,
            selection_anchor,
            ..
        }) => (*state, buffer.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Normal, String::new(), 0, None),
    };
    paint_number_chip(
        rect,
        st,
        f64::from(value),
        None,
        Some(&buf),
        caret,
        anchor,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// Um botão de ícone da linha — **Pressed enquanto armado**, o mesmo desenho do irmão de cor.
///
/// ⚠️ Uma função para os DOIS (o elo e o `f(x)`): eles têm a mesma caixa, o mesmo estado-armado e o
/// mesmo lugar na fila da direita, e duas cópias divergiriam no dia em que um deles ganhasse um
/// realce.
fn paint_icon_row_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    icon: IconId,
    x: f32,
    y: f32,
    armed: bool,
) {
    let rect = Rect::new(x, y + (ROW_H_PX - LINK_W) * 0.5, LINK_W, LINK_W);
    let state = if armed {
        (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
    } else {
        ctx.host.store().button_visual(id)
    };
    paint_icon_button(
        rect,
        IconGlyph::Builtin(icon),
        IconButtonStyle::Compact,
        state,
        ctx.scene,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}
