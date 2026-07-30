//! **The Expression modal's two body columns** — the gallery and the sheet.
//!
//! Split from `expr_modal_paint` by responsibility (the panel's 600-LOC cap made
//! the cut, the line of it is the meaning): that file owns what a CARD is — where
//! it sits, its title band, its formula bar, its footer — and this one owns what
//! goes INSIDE it. The card's geometry constants stay there and are shared, so
//! there is one answer to how wide a column is.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    Button, ButtonState, IconButtonStyle, IconGlyph, NumberInput, TextInput, TextInputState,
    paint_button, paint_icon_button, paint_number_input_with_buffer, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_expr_recipes::{Knob, KnobKind};
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};

use crate::expr_modal::{ExprModal, row_result};
use crate::expr_modal_paint::{KNOB_LABEL_W, KNOB_READOUT_W, ROW_BTN_W, SHEET_W, button_state};
use crate::ids;

/// Paint a button AND make it live under the mouse.
///
/// ⚠️ The `register_if_absent` is not decoration: a hit rect alone gets the
/// pointer to the right id, and the store entry is what makes the id FOCUSABLE —
/// without it the button is painted, hit-registered and dead under the mouse,
/// while a synthetic `WidgetEvent::Click` in a gate sails straight through. That
/// pair (green gate, dead button) is the exact failure the physics panel paid for
/// with 36 collision-matrix cells.
pub(crate) fn expr_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    label: &str,
    rect: Rect,
) {
    ctx.host.hit_index_mut().register(id, rect);
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    let b = Button::new(id, label).state(button_state(ctx.host.store(), id));
    paint_button(&b, rect, ctx.scene, ctx.text_system, theme);
}

/// The same button, wearing an **ICON from the design system** instead of a letter.
///
/// ⚠️ This exists because the first cut of the card drew its eye as the letter `"O"`
/// and its remove as `"X"` (Enio: *"neste app usamos o olhinho para esconder algo. Por
/// que usou um O?"*). There was no reason — they were placeholders, and the comment on
/// the line above the eye already CALLED it an eye, which is the tell: prose describing
/// an icon over code drawing a glyph. Letters also break §3 (zero hardcoded string) and
/// they do not carry the app's meaning — an artist learns one eye, in every panel.
///
/// ⚠️ It registers the hit rect AND seeds the store, exactly like [`expr_button`]: a
/// painted-but-unregistered widget is dead under the mouse while a synthetic
/// `WidgetEvent::Click` in a gate sails straight through it.
pub(crate) fn expr_icon_button(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    glyph: IconId,
    rect: Rect,
) {
    ctx.host.hit_index_mut().register(id, rect);
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    paint_icon_button(
        rect,
        IconGlyph::Builtin(glyph),
        IconButtonStyle::Compact,
        button_state(ctx.host.store(), id),
        ctx.scene,
        theme,
    );
}

/// Width of a knob's number box.
///
/// ⚠️ Comfortably over [`NUMBER_INPUT_MIN_W_PX`], which is the app's canon floor
/// ("não permita que a caixa seja redimensionada para menor que isso"): a value knob
/// now holds things like `0.05` and `-12.75`, and the stepper column eats the right
/// end of whatever is left.
const NUM_W: f32 = 96.0; // LITERAL-PX-OK: largura da caixa numerica de um knob
/// A calha entre as duas colunas de knob.
///
/// ⚠️ `Spacing::Md`, não `Xs`: a queixa medida do header não era largura, era **gutter
/// ZERO** entre nome │ readout │ X (doc 13 §4-bis). Duas colunas encostadas repetiriam
/// exactamente esse erro no eixo que sobrou.
const KNOB_COL_GUTTER: f32 = 8.0; // LITERAL-PX-OK: = Spacing::Md, a calha das colunas
/// A calha entre as três coisas do cabeçalho de uma row (nome │ readout │ X).
const HDR_GUTTER: f32 = 6.0; // LITERAL-PX-OK: = Spacing::Sm, a calha do cabecalho
/// O respiro ENTRE dois cartões de row.
const ROW_GAP: f32 = 4.0; // LITERAL-PX-OK: = Spacing::Xs, o respiro entre rows

/// Paint a knob's **number box** and make it live: value, stepper range, drag rate.
///
/// ⚠️ **A box, not a slider** (Enio, smoke de 2026-07-29: *"no lugar de sliders,
/// melhor apenas caixas de input numérico"*). It is also the honest widget: a
/// knob's range is now the CANVAS (±40 m) while its working magnitude is the OBJECT
/// (~0.3 m), so a 120 px track spent 0.7 px on the entire useful range — the artist
/// could not land on a number, only near one. A box is typed, and it still drags
/// (the dispatch's range-proportional drag) and still steps.
///
/// The range is REGISTERED rather than clamped afterwards, because registering is
/// what makes the arrows step by the knob's own increment instead of the dispatch's
/// buffer heuristic; the drag rate is that same increment, so **one pixel is worth
/// one click** and the two agree without anyone calibrating them.
///
/// ⚠️ Typing is NOT clamped, by design and by the dispatch: only the arrows and the
/// drag honour the range. A knob range is where the thumb stops, not where the model
/// does.
fn paint_knob_number(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    k: &Knob,
    value: f32,
    reseed: bool,
    rect: Rect,
) {
    let step = k.step_value();
    let seed = InteractiveState::NumberInput {
        state: TextInputState::Normal,
        value: f64::from(value),
        buffer: ph2d_expr_recipes::fmt_num(value),
        caret: 0,
        last_committed: f64::from(value),
        selection_anchor: None,
    };
    {
        let store = ctx.host.store_mut();
        if reseed {
            store.register(id, seed);
        } else {
            store.register_if_absent(id, seed);
        }
        store.set_number_range(
            id,
            f64::from(k.range.0),
            f64::from(k.range.1),
            f64::from(step),
        );
        store.set_number_drag_rate(id, f64::from(step));
    }
    let (state, v, buf, caret, anchor) = ctx
        .host
        .store()
        .number_input(id)
        .map(|(s, v, b, c, a)| (s, v, b.to_string(), c, a))
        .unwrap_or((
            TextInputState::Normal,
            f64::from(value),
            String::new(),
            0,
            None,
        ));
    let input = NumberInput::new(id, k.label, v)
        .step(f64::from(step))
        .state(state);
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        rect,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(id, rect);
}

/// **Quantas linhas de knob uma receita gasta, e onde cada knob senta.**
///
/// ⚠️ **A porta única do layout do sheet**, e ela existe porque a capacidade vertical era
/// onde de fato apertava: `BODY_SLOTS = 12`, uma row custava `1 + knobs`, e uma receita de
/// 4 knobs deixava caber **2 rows** (medido, doc 13 §4-bis). O `+N more rows` não era falta
/// de scroll: era uma row **dirigindo o objeto sem um pixel de UI**.
///
/// A cura não é scroll — é os **128 px MORTOS** que toda row de knob já tinha (`ctrl_w`
/// computado como 168 e descartado no braço `Number|Literal`). Dois knobs numéricos por
/// linha, e o pior caso do catálogo cai de **5 slots para 3** (4 rows garantidas, 6 no
/// caso típico) sem introduzir um 2º eixo de scroll dentro de um painel que já rola.
///
/// ⚠️ **Um knob de TEXTO fica sozinho na linha** (um `Link` carrega um nome, um `Text` uma
/// fórmula): parear dois campos de texto de 72 px seria trocar um aperto por outro. Então a
/// conta é *pares de numéricos* + *um por texto*, e ela é feita AQUI para que o pintor e o
/// contador de slots não possam discordar — um container medido por uma regra e preenchido
/// por outra é como a próxima seção pinta por cima dos botões.
fn knob_rows(rec: &ph2d_expr_recipes::Recipe) -> usize {
    let (mut numeric, mut wide) = (0usize, 0usize);
    for k in rec.knobs {
        match k.kind {
            KnobKind::Number | KnobKind::Literal => numeric += 1,
            KnobKind::Link | KnobKind::Text => wide += 1,
        }
    }
    numeric.div_ceil(2) + wide
}

/// Onde o knob `i` de `rec` senta: a linha (relativa ao topo dos knobs) e a COLUNA (0 ou 1,
/// e sempre 0 para um knob de texto, que ocupa a linha inteira).
///
/// Deriva da MESMA varredura que [`knob_rows`] conta, na mesma ordem, então o pintor não
/// pode pousar um widget fora do espaço que o contador reservou.
fn knob_slot(rec: &ph2d_expr_recipes::Recipe, want: usize) -> (usize, usize) {
    let (mut line, mut col) = (0usize, 0usize);
    for (i, k) in rec.knobs.iter().enumerate() {
        let wide = matches!(k.kind, KnobKind::Link | KnobKind::Text);
        if wide && col == 1 {
            // Um knob largo não divide linha com um numérico já pousado à esquerda.
            line += 1;
            col = 0;
        }
        if i == want {
            return (line, col);
        }
        if wide {
            line += 1;
            col = 0;
        } else if col == 0 {
            col = 1;
        } else {
            line += 1;
            col = 0;
        }
    }
    (line, col)
}

/// As linhas de knob de UMA row do sheet, e o `y` em que a próxima row começa.
///
/// Split de `paint_sheet` quando as duas colunas o levaram a 211 > 200 LOC; o corte é o
/// mesmo que o resto deste arquivo usa — *a ROW* contra *os knobs dela* — e é o que dá à
/// geometria de coluna um lugar só.
#[expect(
    clippy::too_many_arguments,
    reason = "os mutáveis por-frame do paint mais a row e o índice dela; a alternativa é um \
              struct de contexto que existiria só para este chamador"
)]
fn paint_knob_rows(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    cy_in: f32,
    reseed: bool,
    ri: usize,
    rec: &ph2d_expr_recipes::Recipe,
    row: &ph2d_expr_recipes::Row,
) -> f32 {
    let font = TypeToken::Sm.px();
    let mut cy = cy_in;
    // ── Knob rows, DOIS numéricos por linha (ver `knob_rows`). ──
    //
    // ⚠️ A geometria vem da porta, não de aritmética repetida aqui: `knob_slot` diz a
    // linha e a coluna, e `knob_rows` — que varre na MESMA ordem — diz quantas linhas
    // reservar. Antes desta wave `ctrl_w` era computado como 168 e **descartado** no
    // braço numérico, deixando 128 px (40% do sheet) mortos em toda linha de knob.
    let knob_top = cy;
    for (ki, k) in rec.knobs.iter().enumerate() {
        let (line, col) = knob_slot(rec, ki);
        let wide = matches!(k.kind, KnobKind::Link | KnobKind::Text);
        cy = knob_top + ROW_H_PX * line as f32;
        let indent = Spacing::Md.px();
        // A largura de uma COLUNA de knob: o sheet menos o recuo e a calha do meio,
        // dividido por dois. Um knob largo toma as duas.
        let col_w = (SHEET_W - indent - KNOB_COL_GUTTER) * 0.5;
        let lx = x
            + indent
            + if col == 1 {
                col_w + KNOB_COL_GUTTER
            } else {
                0.0
            };
        let label_w = if wide {
            KNOB_LABEL_W
        } else {
            (col_w - NUM_W - Spacing::Xs.px()).max(1.0)
        };
        paint_text(
            ctx.text_system,
            ctx.scene,
            k.label,
            lx,
            cy + (ROW_H_PX - font) * 0.5,
            font,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        let ctrl_x = lx + label_w + Spacing::Xs.px();
        // Um knob largo chega até a borda do sheet; um numérico, até o fim da sua coluna.
        let ctrl_w = if wide {
            (x + SHEET_W - ctrl_x).max(1.0)
        } else {
            (lx + col_w - ctrl_x).max(1.0)
        };
        let id = ids::expr_knob_id(ri, ki);
        match k.kind {
            KnobKind::Number | KnobKind::Literal => {
                // ⚠️ `ctrl_w`, não `NUM_W`: era exactamente aqui que os 168 px
                // computados eram jogados fora.
                let r = Rect::new(ctrl_x, cy, ctrl_w.min(NUM_W), ROW_H_PX);
                paint_knob_number(ctx, theme, id, k, row.knobs[ki].as_num(), reseed, r);
            }
            KnobKind::Link | KnobKind::Text => {
                let seed = row.knobs[ki].as_text().to_string();
                let caret = seed.len();
                let init = InteractiveState::TextInput {
                    state: TextInputState::Normal,
                    text: seed,
                    caret,
                    selection_anchor: None,
                };
                if reseed {
                    ctx.host.store_mut().register(id, init);
                } else {
                    ctx.host.store_mut().register_if_absent(id, init);
                }
                let (st, t, c, a) = match ctx.host.store().get(id) {
                    Some(InteractiveState::TextInput {
                        state,
                        text,
                        caret,
                        selection_anchor,
                    }) => (*state, text.clone(), *caret, *selection_anchor),
                    _ => (TextInputState::Normal, String::new(), 0, None),
                };
                let r = Rect::new(ctrl_x, cy, ctrl_w, ROW_H_PX);
                ctx.host.hit_index_mut().register(id, r);
                let ti = TextInput::new(id, k.label).state(st);
                paint_text_input_with_buffer(
                    &ti,
                    Some(t.as_str()),
                    Some(c),
                    a,
                    r,
                    ctx.scene,
                    ctx.text_system,
                    theme,
                );
            }
        }
    }

    knob_top + ROW_H_PX * knob_rows(rec) as f32
}

/// The centre column: the stack, one row per recipe, each with its knobs and the
/// number it produces RIGHT NOW.
pub(crate) fn paint_sheet(
    m: &ExprModal,
    ctx: &mut PaintCtx,
    theme: Theme,
    at: Rect,
    reseed: bool,
    base: f32,
) {
    // ⚠️ **Uma BANDA, não `(x, y, slots)`.** O sheet pinta dentro de um retângulo, e o
    // orçamento dele é a altura desse retângulo — que é exactamente o que
    // `body_slots(viewport) * ROW_H_PX` já vale. Passar os três separados era pedir ao
    // chamador que mantivesse coerentes três números que descrevem uma coisa só.
    let (x, font) = (at.x, TypeToken::Sm.px());
    let mut cy = at.y;
    // ⚠️ **O orçamento é em PIXELS, não em slots** (FASE C.4). A calha entre rows não é um
    // múltiplo de `ROW_H_PX`, então contar em slots a deixaria de fora da conta — e o
    // modo de falha disso é o pior que este arquivo já teve: uma row que a aritmética diz
    // caber e o desenho empurra para fora do card, viva na fórmula e invisível na tela.
    let budget = at.h;
    let mut used_px = 0.0_f32;

    for (ri, row) in m.stack.rows.iter().enumerate() {
        let Some(rec) = ph2d_expr_recipes::by_id(row.recipe) else {
            continue;
        };
        let need = 1 + knob_rows(rec);
        let need_px = ROW_H_PX * need as f32 + ROW_GAP;
        if used_px + need_px > budget {
            paint_text(
                ctx.text_system,
                ctx.scene,
                &format!("+{} more rows", m.stack.rows.len() - ri),
                x + Spacing::Sm.px(),
                cy + (ROW_H_PX - font) * 0.5,
                font,
                SHEET_W,
                resolve(ColorToken::Text2, theme),
            );
            return;
        }
        used_px += need_px;

        // ── A LINHA É UM CARTÃO. ──
        //
        // O §5.2 do plano nomeou dois defeitos que são um só: *sem hierarquia* (uma row de
        // receita e um knob dela tinham o MESMO peso visual — mesmo `ROW_H_PX`, mesmo
        // fundo) e *sem respiro*. A planilha lia como uma lista plana de doze itens em vez
        // de três blocos, e nada dizia onde uma receita acabava e a outra começava.
        //
        // ⚠️ **Copiado, não inventado:** é a superfície elevada + raio que o
        // `ph2d-panel-wet-tuning` e o `motion-params` já usam para a mesma estrutura
        // (linhas de um stack). Um artista aprende UM cartão.
        //
        // ⚠️ O fundo cobre a row INTEIRA (cabeçalho + as linhas de knob dela) porque é
        // exactamente essa a extensão que o `need` acima reservou — desenhar só sob o
        // cabeçalho separaria a receita dos próprios knobs.
        fill_rounded_rect(
            ctx.scene,
            Rect::new(x, cy, SHEET_W, ROW_H_PX * need as f32),
            Radius::Sm.px(),
            resolve(ColorToken::BgElev, theme),
        );

        // Row header: bypass eye · label · result · remove.
        //
        // ⚠️ The icons follow the Vector line's **effect stack**
        // (`ph2d-panel-vector/src/paint_effects.rs:160-176`), because that is the same
        // structure: rows of a stack, each one bypassable, each one removable. Eye when
        // the row is LIVE and closed eye when it is bypassed is the app's convention in
        // four places (hierarchy, painter layers, mask rows, vector effects) — an artist
        // learns one eye, not one per panel.
        let eye = Rect::new(x, cy, ROW_BTN_W, ROW_H_PX);
        let eye_id = ids::expr_bypass_id(ri);
        let eye_glyph = if row.bypass {
            IconId::EyeClosed
        } else {
            IconId::Eye
        };
        expr_icon_button(ctx, theme, eye_id, eye_glyph, eye);

        let rm = Rect::new(x + SHEET_W - ROW_BTN_W, cy, ROW_BTN_W, ROW_H_PX);
        // ⚠️ **A calha do header** — a queixa que a auditoria MEDIU (doc 13 §4-bis): não era
        // a largura do nome (198 px, ~16 caracteres), era `gutter ZERO` entre
        // nome │ readout │ X, três coisas encostadas lendo como uma só.
        let readout_x = rm.x - HDR_GUTTER - KNOB_READOUT_W;
        let rm_id = ids::expr_remove_id(ri);
        expr_icon_button(ctx, theme, rm_id, IconId::Close, rm);

        // **The combine chip** — how this row lands on the rows above it.
        //
        // ⚠️ Painted for a SOURCE and for nothing else. A modifier folds the value
        // itself (`Limit` clamps it), so a mode chip on one would be a control with no
        // meaning — and `Row::combines` is the SAME question the fold asks, so the chip
        // and the picture cannot disagree about whether the row has a mode.
        //
        // ⚠️ It is a glyph and not a word because it sits inside a 28 px row next to the
        // eye; the label is in the tooltip's vocabulary (`Combine::label`), and the
        // three are the operators an artist already reads in the formula bar.
        let label_x = if row.combines() {
            let mode = Rect::new(x + ROW_BTN_W, cy, ROW_BTN_W, ROW_H_PX);
            expr_button(
                ctx,
                theme,
                ids::expr_combine_id(ri),
                row.combine.glyph(),
                mode,
            );
            mode.x + ROW_BTN_W + Spacing::Xs.px()
        } else {
            x + ROW_BTN_W + Spacing::Xs.px()
        };

        // ⚠️ The result readout is the payload of the spreadsheet metaphor: in a
        // spreadsheet you never wonder what a formula IS, you see what it GIVES.
        let result = row_result(&m.stack, ri, m.time, base);
        paint_text(
            ctx.text_system,
            ctx.scene,
            rec.label,
            label_x,
            cy + (ROW_H_PX - font) * 0.5,
            font,
            (readout_x - HDR_GUTTER - label_x).max(0.0),
            resolve(
                if row.bypass {
                    ColorToken::Text2
                } else {
                    ColorToken::Text1
                },
                theme,
            ),
        );
        paint_text(
            ctx.text_system,
            ctx.scene,
            &result,
            readout_x,
            cy + (ROW_H_PX - font) * 0.5,
            font,
            KNOB_READOUT_W,
            resolve(ColorToken::Text2, theme),
        );
        cy += ROW_H_PX;

        cy = paint_knob_rows(ctx, theme, x, cy, reseed, ri, rec, row);
        cy += ROW_GAP;
    }

    if m.stack.rows.is_empty() {
        paint_text(
            ctx.text_system,
            ctx.scene,
            ph2d_i18n::tr("panel.timeline.expr_empty"),
            x + Spacing::Sm.px(),
            cy + (ROW_H_PX - font) * 0.5,
            font,
            SHEET_W,
            resolve(ColorToken::Text2, theme),
        );
    }
}
