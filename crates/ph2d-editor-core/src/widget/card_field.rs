//! ⭐ **O campo Figma de um cartão** (`docs/CartoesUI/02_especificacao.md` §1–§3): o rótulo com a
//! unidade ACIMA, a caixa com a célula do ícone (a PEGA do scrub) e o valor, e a marca de animação
//! na linha do rótulo, alinhada à borda direita da caixa.
//!
//! O estado é o de uma `NumberInput` (valor, buffer, foco, Mixed); o gesto próprio vive no
//! despacho (`interaction::state::card_field_ops`). Geometria: `ph2d_card_stack::metrics`.

use crate::icons::IconId;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_icon, paint_text};
use crate::widget::card_ink::CardInk;
use crate::widget::property_box::key_mark;
use crate::widget::showcase::read_number_input;
use crate::widget::text_input::TextInputState;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_vector::VectorScene;

/// O que o painel declara de um campo de cartão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CardFieldSpec {
    /// Unidades por toque de seta (↑ / ↓), e as casas que o valor mostra.
    pub step: f64,
    /// Unidades por pixel de arrasto sem modificador (campos sem intervalo conhecido).
    pub scrub: f64,
    /// O valor de FÁBRICA que o duplo clique no ícone repõe (`None` = sem reposição).
    pub default: Option<f64>,
}

/// O que mora na célula do ícone: um glifo, ou uma letra curta (`X`, `Y`, `In`, `Out`).
#[derive(Copy, Clone, Debug)]
pub enum Glyph<'a> {
    Icon(IconId),
    Text(&'a str),
}

/// A célula do ícone — a pega do scrub.
#[must_use]
pub fn handle_rect(field: Rect) -> Rect {
    Rect::new(field.x, field.y, m::ICON_CELL.min(field.w), field.h)
}

/// A área do VALOR (depois da célula do ícone) — onde o texto, o caret e o clique de caret vivem.
#[must_use]
pub fn value_rect(field: Rect) -> Rect {
    let x = field.x + m::ICON_CELL;
    Rect::new(
        x,
        field.y,
        (field.x + field.w - m::FIELD_PAD_R - x).max(0.0),
        field.h,
    )
}

/// O Down caiu na pega?
#[must_use]
pub fn on_handle(field: Rect, x: f32) -> bool {
    x >= field.x && x < field.x + m::ICON_CELL
}

/// O passo a partir do qual o valor mostra duas casas.
const PASSO_CENTESIMAL: f64 = 0.01; // LITERAL-PX-OK: a ordem de grandeza do passo, não medida de UI

/// As casas que o valor mostra, pelo passo do campo (`1` → inteiro; `0,1` → `0.35`).
#[must_use]
pub fn decimals(step: f64) -> usize {
    if step >= 1.0 {
        0
    } else if step >= PASSO_CENTESIMAL {
        2
    } else {
        3
    }
}

thread_local! {
    static SOBREPOSTOS: std::cell::RefCell<Vec<(NodeId, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
    static LETRAS: std::cell::RefCell<std::collections::BTreeMap<NodeId, String>> = const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// A letra que a pega do campo `id` mostrou da última vez que se pintou (`None` = ícone ou nunca).
#[must_use]
pub fn painted_letter(id: NodeId) -> Option<String> {
    LETRAS.with(|l| l.borrow().get(&id).cloned())
}

fn regista_letra(id: NodeId, glyph: Glyph<'_>) {
    LETRAS.with(|l| {
        let mut l = l.borrow_mut();
        match glyph {
            Glyph::Text(t) if l.get(&id).is_none_or(|v| v != t) => {
                l.insert(id, t.to_owned());
            }
            Glyph::Icon(_) => {
                l.remove(&id);
            }
            Glyph::Text(_) => {}
        }
    });
}

/// T40: pinta `f` com estes números no lugar dos do store — o cartão PRESO mostra o sujeito dele
/// com o mesmo id que a lista usa para o da seleção.
pub fn com_valores<R>(valores: &[(NodeId, f64)], f: impl FnOnce() -> R) -> R {
    let antes = SOBREPOSTOS.with(|s| std::mem::replace(&mut *s.borrow_mut(), valores.to_vec()));
    let r = f();
    SOBREPOSTOS.with(|s| *s.borrow_mut() = antes);
    r
}

fn sobreposto(id: NodeId) -> Option<f64> {
    SOBREPOSTOS.with(|s| s.borrow().iter().find(|(i, _)| *i == id).map(|(_, v)| *v))
}

/// ⭐ O cursor ↔ sob `x`: a pega de um campo ou a borda de um cartão solto (T39).
#[must_use]
pub fn ew_resize_under(store: &WidgetStore, id: NodeId, r: Rect, x: f32) -> bool {
    store.loose_edge(id).is_some() || (store.card_field(id).is_some() && on_handle(r, x))
}

/// ⭐ **Uma caixa do campo Figma.** Regista o rect da caixa inteira para o `id` (o despacho
/// separa pega e valor pela [`on_handle`]).
#[allow(clippy::too_many_arguments)]
pub fn paint_field(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    rect: Rect,
    id: NodeId,
    glyph: Glyph<'_>,
) {
    let parado = valor::parado(id);
    if !parado {
        hit_index.register(id, rect);
    }
    regista_letra(id, glyph);
    let (st, value, buffer, caret, anchor) = read_number_input(store, id);
    let value = sobreposto(id).unwrap_or(value);
    let focused = !parado && st == TextInputState::Focused;
    let scrubbing = !parado
        && store
            .number_input_drag()
            .is_some_and(|d| d.id == id && d.crossed_threshold);
    let hover = !parado && (store.hover_live(id) > 0.0 || st == TextInputState::Hovered);
    paint_field_frame(scene, ink, rect, hover, focused || scrubbing);
    // Mudado e ninguém o gravou (maquete B1): o contorno e o valor no laranja do losango.
    let mudado = !(parado || focused || scrubbing) && key_mark::mudado(id);
    if mudado {
        // FRAME-RAW-OK: o anel do campo MUDADO (a mensagem é «isto perde-se»)
        crate::paint::stroke_rounded_rect(
            scene,
            rect,
            m::FIELD_RADIUS,
            m::FIELD_BORDER,
            key_mark::contorno_mudado(ink.theme),
        );
    }

    let tinta = ink.color(if scrubbing {
        ColorToken::KanbanBlueBold
    } else if parado {
        ColorToken::Text3
    } else {
        ColorToken::CardLabel
    });
    paint_glyph(scene, text_system, handle_rect(rect), glyph, tinta);

    // O valor.
    let area = value_rect(rect);
    let step = store.card_field(id).map_or(1.0, |s: CardFieldSpec| s.step);
    let misto = store.number_input_mixed(id) && !focused;
    let owned;
    let (texto, cor) = if focused {
        (buffer, ColorToken::Text1)
    } else if misto {
        (ph2d_i18n::tr("card.field.mixed"), ColorToken::Text3)
    } else {
        // T29: um passo inteiro não esconde a fração do valor (Friction 0.5 não se lê «0»).
        owned = texto_do_valor(value, step);
        let cor = if parado {
            ColorToken::Text3
        } else {
            ColorToken::Text1
        };
        (owned.as_str(), cor)
    };
    let y = area.y + crate::paint::folga_do_texto(area.h, m::VALUE_PX);
    let clip = ph2d_vector::Rect::new(
        f64::from(area.x),
        f64::from(rect.y),
        f64::from(area.x + area.w),
        f64::from(rect.y + rect.h),
    );
    scene.push_clip(&clip);
    if focused && let Some(a) = anchor.filter(|a| *a != caret) {
        let (s, e) = (a.min(caret).min(texto.len()), a.max(caret).min(texto.len()));
        let x0 = area.x + text_system.prefix_width(&texto[..s], m::VALUE_PX);
        let w = text_system.prefix_width(&texto[s..e], m::VALUE_PX);
        let sel = Rect::new(x0, y, w.min(area.x + area.w - x0), m::VALUE_PX);
        fill_rounded_rect(scene, sel, m::SEG_PAD, ink.color(ColorToken::AccentSoft));
    }
    paint_text(
        text_system,
        scene,
        texto,
        area.x,
        y,
        m::VALUE_PX,
        area.w,
        if mudado {
            key_mark::cor_mudado(ink.theme)
        } else {
            ink.color(cor)
        },
    );
    if focused {
        let c = caret.min(texto.len());
        let x = area.x + text_system.prefix_width(&texto[..c], m::VALUE_PX);
        paint_field_caret(scene, ink, x, y, m::VALUE_PX);
    }
    scene.pop_layer();
}

/// ⭐ **A caixa de um campo de texto do cartão**: o fundo (mais claro sob o rato ou com foco) e, com
/// foco, o anel «é este». A do campo Figma e a da busca da paleta (dono 08/11) — um estado, um desenho.
pub fn paint_field_frame(
    scene: &mut VectorScene,
    ink: CardInk,
    rect: Rect,
    hover: bool,
    focused: bool,
) {
    let fundo = if hover || focused {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(scene, rect, m::FIELD_RADIUS, ink.color(fundo));
    if focused {
        // FRAME-RAW-OK: o anel de FOCO do campo do cartão (a mensagem é «é este»)
        crate::paint::stroke_rounded_rect(
            scene,
            rect,
            m::FIELD_RADIUS,
            m::FIELD_BORDER,
            ink.color(ColorToken::KanbanBlueBold),
        );
    }
}

/// O caret de um campo de texto do cartão, em `x`, da altura `h` a partir de `y`.
pub fn paint_field_caret(scene: &mut VectorScene, ink: CardInk, x: f32, y: f32, h: f32) {
    let r = Rect::new(x, y, StrokeToken::Default.px(), h);
    fill_rounded_rect(scene, r, 0.0, ink.color(ColorToken::Accent));
}

/// A pega: o glifo, ou a letra, centrados na célula.
fn paint_glyph(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    cell: Rect,
    glyph: Glyph<'_>,
    tinta: ph2d_vector::Color,
) {
    match glyph {
        Glyph::Icon(icon) => {
            let g = m::ICON_GLYPH;
            let r = Rect::new(
                cell.x + (cell.w - g) * 0.5,
                cell.y + (cell.h - g) * 0.5,
                g,
                g,
            );
            paint_icon(scene, icon, r, tinta, StrokeToken::Default.px());
        }
        Glyph::Text(t) => {
            let w = text_system.prefix_width(t, m::LABEL_PX);
            paint_text(
                text_system,
                scene,
                t,
                cell.x + (cell.w - w).max(0.0) * 0.5,
                cell.y + crate::paint::folga_do_texto(cell.h, m::LABEL_PX),
                m::LABEL_PX,
                cell.w,
                tinta,
            );
        }
    }
}

/// ⭐ **A caixa de um campo que ainda não é editável aqui** (a opacidade de uma camada fechada,
/// T17): a MESMA cara do [`paint_field`] em repouso — glifo e valor —, e o clique em `id` abre-a.
#[allow(clippy::too_many_arguments)]
pub fn paint_field_face(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    rect: Rect,
    id: NodeId,
    glyph: Glyph<'_>,
    text: &str,
) {
    hit_index.register(id, rect);
    let fundo = if store.hover_live(id) > 0.0 {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(scene, rect, m::FIELD_RADIUS, ink.color(fundo));
    paint_glyph(
        scene,
        text_system,
        handle_rect(rect),
        glyph,
        ink.color(ColorToken::CardLabel),
    );
    let area = value_rect(rect);
    paint_text(
        text_system,
        scene,
        text,
        area.x,
        area.y + crate::paint::folga_do_texto(area.h, m::VALUE_PX),
        m::VALUE_PX,
        area.w,
        ink.color(ColorToken::Text1),
    );
}

/// O rótulo acima de uma coluna de caixas (`Length (px)`), elidido na largura.
pub fn paint_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: CardInk,
    x: f32,
    y: f32,
    w: f32,
    label: &str,
) {
    crate::paint::paint_text_elided(
        text_system,
        scene,
        label,
        x,
        y,
        m::LABEL_PX,
        w,
        ink.color(ColorToken::CardLabel),
    );
}

/// ⭐ **A marca de animação** (D17): ponto de 6 px (pode animar) ou losango (animado / chave /
/// mudado), centrada em `(cx, cy)`. Devolve o id clicável quando o campo tem canal.
pub fn paint_key_mark(
    scene: &mut VectorScene,
    ink: CardInk,
    hit_index: &mut HitIndex,
    cx: f32,
    cy: f32,
    fields: &[NodeId],
) {
    use crate::widget::property_box::key_mark::{
        cor_do_losango, key_mark_of, pinta_losango, register_key_mark_target,
    };
    let caixa = Rect::new(
        cx - m::KEY_MARK_BOX * 0.5,
        cy - m::KEY_MARK_BOX * 0.5,
        m::KEY_MARK_BOX,
        m::KEY_MARK_BOX,
    );
    let marca = key_mark_of(fields);
    let (x, y) = (f64::from(cx), f64::from(cy));
    match marca {
        // Sem canal: o ponto inerte de sempre.
        None => {
            let c = ph2d_vector::Circle::new((x, y), f64::from(m::KEY_MARK_PX) * 0.5);
            scene.inner_mut().fill(
                ph2d_vector::Fill::NonZero,
                ph2d_vector::Affine::IDENTITY,
                ink.color(ColorToken::CardLabel),
                None,
                &c,
            );
        }
        Some(k) => {
            let cor = cor_do_losango(k, ColorToken::CardLabel, ink.theme, |t| ink.color(t));
            pinta_losango(scene, (x, y), k, cor);
        }
    }
    // O campo PARADO (travado) também não grava pela marca: sem alvo.
    if marca.is_some()
        && !fields.iter().all(|&f| valor::parado(f))
        && let Some(alvo) = register_key_mark_target(fields)
    {
        hit_index.register(alvo, caixa);
    }
}

#[path = "card_field/conta.rs"]
mod conta;
pub use conta::eval_number;
#[path = "card_field/valor.rs"]
mod valor;
use valor::texto_do_valor;
pub use valor::{paint_amostra, parado, parados};

#[cfg(test)]
#[path = "card_field/tests.rs"]
mod tests;
