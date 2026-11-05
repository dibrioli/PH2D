//! ⭐⭐ **O painel Design Tokens em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): as
//! mesmas tabelas (`ColorToken::ALL`, `NumToken::ALL`) e os MESMOS ids; o despacho (`event.rs`) não
//! muda. A amostra mostra a cor do token no tema do APP, nunca na tinta do cartão: um cartão com tema
//! próprio não muda o que o token vale.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::paint::{paint_text_block, stroke_rounded_rect};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::contrast::{failing_pairs, token_is_in_a_failing_pair};
use ph2d_tokens::num_expr::math_available;
use ph2d_tokens::num_overrides::{NumValue, num_overridden_count, num_override};
use ph2d_tokens::overrides::{TokenValue, color_override, overridden_count};
use ph2d_tokens::{ColorToken, NumToken, StrokeToken, Theme};

use crate::familias::{FAMILIAS, familia_de};
use crate::ids as i;
use crate::populate::PX_STEP;
use crate::state::TokensPanelState;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (a variável de ambiente seria partilhada entre testes paralelos).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// As secções que só existem em cartões; as famílias já são secções.
const PROPRIAS: [NodeId; 3] = [i::TOKENS_SEC_MODE, i::TOKENS_SEC_APP, i::TOKENS_SEC_SCALE];

/// O texto da fórmula autorada, ou vazio. O espelho é gateado no FOCO, como o do corpo de antes: sem
/// ele, cada quadro reescrevia o que está a ser digitado.
fn semeia_formula(store: &mut WidgetStore, id: NodeId, autorada: &str) {
    let _ = store.register_if_absent(
        id,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: autorada.to_string(),
            caret: autorada.len(),
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
        if text != autorada {
            text.clear();
            text.push_str(autorada);
        }
        *caret = (*caret).min(text.len());
        *selection_anchor = None;
    }
}

fn dica(store: &mut WidgetStore, id: NodeId, chave: &str) {
    if store.tooltip_for(id).is_none() {
        store.set_tooltip(id, tr(chave));
    }
}

/// Antes de pintar: fecho, secções, pegas, dicas e os valores EFETIVOS nas caixas. O campo que a mão
/// arrasta ou escreve não se semeia.
pub(crate) fn prepare(store: &mut WidgetStore, theme: Theme, state: &TokensPanelState) {
    store.mark_card_list_close(ids::TOKENS_PANEL, i::TOKENS_CLOSE);
    // A Escala nasce dobrada: é o que paga a catraca da altura de abertura (a linha de cartão mede
    // mais que a de antes). Só aqui: com `PH2D_CARDS=0` ela nem é secção.
    store.set_collapsed_if_unchosen(i::TOKENS_SEC_SCALE, true);
    let familias = FAMILIAS.iter().map(|f| f.id);
    for id in PROPRIAS.into_iter().chain(familias) {
        store.mark_collapsible_section(id);
        let grip = ids::grip_de(id);
        let _ = store.register_if_absent(grip, InteractiveState::Plain);
        dica(store, grip, "chrome.section.grip_hint");
    }
    for row in 0..ColorToken::ALL.len() {
        dica(store, i::tokens_reset_id(row), "panel.tokens.reset");
        dica(store, i::tokens_link_id(row), "card.tokens.link");
    }
    let arrastado = store.number_input_drag().map(|d| d.id);
    for (row, &token) in NumToken::ALL.iter().enumerate() {
        let chip = i::tokens_num_chip_id(row);
        dica(store, i::tokens_num_reset_id(row), "panel.tokens.reset");
        dica(store, i::tokens_num_link_id(row), "card.tokens.link");
        dica(store, i::tokens_num_fx_id(row), "card.tokens.formula");
        store.mark_card_field(
            chip,
            CardFieldSpec {
                step: PX_STEP,
                scrub: PX_STEP,
                default: Some(f64::from(token.factory_px())),
            },
        );
        if store.focus_id() != Some(chip) && arrastado != Some(chip) {
            store.set_number_value(chip, f64::from(token.px(theme)));
        }
        let aberta = matches!(num_override(theme, token), Some(NumValue::Expr(_)))
            || state.fx_open() == Some(row);
        if aberta {
            let autorada = match num_override(theme, token) {
                Some(NumValue::Expr(src)) => src,
                _ => String::new(),
            };
            semeia_formula(store, i::tokens_num_formula_id(row), &autorada);
        }
    }
}

fn icone(id: NodeId, icon: IconId, on: bool) -> Seg<'static> {
    Seg {
        id,
        icon: Some(icon),
        text: "",
        on,
    }
}

/// Uma linha da lista de cor: `■ chave [→ alvo]  ⚠ [elo] [↺]`. O anel marca a amostra que o selector
/// aberto está a editar.
fn linha_de_cor(c: &mut CardCtx<'_>, theme: Theme, row: usize, armada: bool, y: f32) -> f32 {
    let token = ColorToken::ALL[row];
    let slot = color_override(theme, token);
    let cor = token.resolve(theme);
    let texto = match slot {
        Some(TokenValue::Alias(alvo)) => format!("{}  -  {}", token.key(), alvo.key()),
        _ => token.key().to_string(),
    };
    let swatch = i::tokens_swatch_id(row);
    let mut icones = vec![icone(i::tokens_link_id(row), IconId::Link, armada)];
    if slot.is_some() {
        icones.push(icone(i::tokens_reset_id(row), IconId::Reset, false));
    }
    let mut cells = vec![Cell::new(Widget::Swatch {
        id: swatch,
        rgba: Some([cor.r, cor.g, cor.b, cor.a]),
        text: &texto,
        edit: None,
        detach: None,
    })];
    if token_is_in_a_failing_pair(theme, token) {
        cells.push(Cell::new(Widget::Mark {
            icon: IconId::Warning,
            tint: ColorToken::Warn,
        }));
    }
    // Coluna estreita: os ícones descem para a linha de baixo, e o nome do token não se corta (05 §4, lei 5).
    let icones_w = icones.len() as f32 * m::ICON_CELL + m::SEG_PAD * 2.0;
    let marca = if cells.len() > 1 { m::ICON_CELL } else { 0.0 };
    let usa = c.text_system.prefix_width(&texto, m::VALUE_PX)
        + m::ICON_CELL
        + m::FIELD_PAD_R
        + m::FIELD_GAP * cells.len() as f32
        + marca
        + icones_w;
    let desce = usa > c.w;
    if !desce {
        cells.push(Cell::new(Widget::Bare(&icones)));
    }
    let fim = c.row(&cells, y);
    let topo = fim - m::FIELD_H;
    let fim = if desce {
        c.row(&[Cell::new(Widget::Bare(&icones))], fim)
    } else {
        fim
    };
    if c.store.picker_target() == Some(swatch) {
        let g = m::ICON_GLYPH;
        // FRAME-RAW-OK: o anel da amostra que o selector de cor está a editar
        stroke_rounded_rect(
            c.scene,
            Rect::new(
                c.x + (m::ICON_CELL - g) * 0.5,
                topo + (m::FIELD_H - g) * 0.5,
                g,
                g,
            ),
            m::SEG_RADIUS,
            StrokeToken::Default.px(),
            c.ink.color(ColorToken::Accent),
        );
    }
    fim
}

/// Uma linha da escala: `[px] chave [→ alvo]  [f(x)] [elo] [↺]`; a fórmula aberta ganha a linha de
/// baixo, e o campo de px vira leitura (um slot, um editor).
fn linha_de_escala(
    c: &mut CardCtx<'_>,
    theme: Theme,
    row: usize,
    state: (bool, bool),
    y: f32,
) -> f32 {
    let (armada, fx_aberto) = state;
    let token = NumToken::ALL[row];
    let slot = num_override(theme, token);
    let autorada = matches!(slot, Some(NumValue::Expr(_)));
    let aberta = autorada || fx_aberto;
    let mut icones = Vec::with_capacity(3);
    if math_available() && !autorada {
        icones.push(icone(i::tokens_num_fx_id(row), IconId::Script, fx_aberto));
    }
    icones.push(icone(i::tokens_num_link_id(row), IconId::Link, armada));
    if slot.is_some() {
        icones.push(icone(i::tokens_num_reset_id(row), IconId::Reset, false));
    }
    let leitura = format_number(f64::from(token.px(theme)));
    let valor = if aberta {
        Cell::new(Widget::Readout { text: &leitura })
    } else {
        Cell::new(Widget::Field {
            id: i::tokens_num_chip_id(row),
            glyph: Glyph::Text(letra_da_grandeza(token.key())),
        })
    };
    let fim = c.row(
        &[
            valor,
            Cell::new(Widget::Readout { text: token.key() }),
            Cell::new(Widget::Bare(&icones)),
        ],
        y,
    );
    // A meia coluna do nome não leva o alvo: quem segue outro DIZ quem, na linha de baixo.
    let fim = match &slot {
        Some(NumValue::Alias(alvo)) => c.hint(
            &tr_with("card.tokens.follows", &[("alvo", &alvo.key())]),
            fim,
        ),
        _ => fim,
    };
    if aberta {
        c.full(
            None,
            Widget::Text {
                id: i::tokens_num_formula_id(row),
                placeholder: tr("panel.tokens.formula.hint"),
            },
            fim,
        )
    } else {
        fim
    }
}

/// Uma frase de aviso que quebra, na cor do aviso. O app continua correcto: `Warn`, nunca `Danger`.
fn aviso(c: &mut CardCtx<'_>, texto: &str, y: f32) -> f32 {
    let top = y + m::FIELD_ROW_GAP;
    let cor = c.ink.color(ColorToken::Warn);
    let fim = top
        + paint_text_block(
            c.text_system,
            c.scene,
            texto,
            c.x,
            top,
            m::LABEL_PX,
            c.w,
            cor,
        );
    ph2d_editor_core::panel::cards::censo::frase((c.x, c.w), top, fim, texto);
    fim
}

fn corpo_do_modo(c: &mut CardCtx<'_>, theme: Theme, y: f32) -> f32 {
    let n = overridden_count(theme) + num_overridden_count(theme);
    let cabecalho = tr_with(
        "panel.tokens.header",
        &[("tema", &tr(theme.display_name_key())), ("n", &n)],
    );
    let mut y = c.full(None, Widget::Readout { text: &cabecalho }, y);
    let falhas = failing_pairs(theme);
    if !falhas.is_empty() {
        y = aviso(c, tr("panel.tokens.contrast.title"), y);
        for par in &falhas {
            let linha = tr_with(
                "panel.tokens.contrast.line",
                &[
                    ("fg", &par.fg.key()),
                    ("bg", &par.bg.key()),
                    ("ratio", &format!("{:.1}", par.ratio(theme))),
                    ("min", &format!("{:.1}", par.min_ratio)),
                    ("criterion", &par.criterion),
                ],
            );
            y = aviso(c, &linha, y);
        }
    }
    // Só com o que repor: sobre um modo de fábrica seria um clique que não faz nada.
    if n > 0 {
        y = c.full(
            None,
            Widget::Button {
                id: i::TOKENS_RESET_ALL,
                text: tr("panel.tokens.reset_all"),
                primary: false,
            },
            y,
        );
    }
    c.row(
        &[
            Cell::new(Widget::Button {
                id: i::TOKENS_DTCG_EXPORT,
                text: tr("panel.tokens.dtcg.export"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: i::TOKENS_DTCG_IMPORT,
                text: tr("panel.tokens.dtcg.import"),
                primary: false,
            }),
        ],
        y,
    )
}

fn cartao<'a>(
    section: NodeId,
    title: &'a str,
    summary: Option<String>,
    body: impl Fn(&mut CardCtx<'_>, f32) -> f32 + 'a,
) -> CardDef<'a> {
    CardDef {
        section,
        title,
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(Box::new(body)),
    }
}

fn cartao_de_cores<'a>(
    section: NodeId,
    title: &'a str,
    theme: Theme,
    armada: Option<usize>,
    pertence: impl Fn(ColorToken) -> bool + 'a,
) -> CardDef<'a> {
    let n = ColorToken::ALL.iter().filter(|t| pertence(**t)).count();
    cartao(section, title, Some(n.to_string()), move |c, y| {
        ColorToken::ALL
            .iter()
            .enumerate()
            .filter(|(_, t)| pertence(**t))
            .fold(y, |y, (row, _)| {
                linha_de_cor(c, theme, row, armada == Some(row), y)
            })
    })
}

/// ⭐ Os cartões do painel: o modo, as cores do app, as famílias de dado (nascem dobradas) e a escala.
pub(crate) fn cards(theme: Theme, state: &TokensPanelState) -> Vec<CardDef<'static>> {
    let armada = state.armed();
    let armada_num = state.armed_num();
    let fx = state.fx_open();
    let mut out = vec![
        cartao(
            i::TOKENS_SEC_MODE,
            tr("card.tokens.mode"),
            Some(tr(theme.display_name_key()).to_string()),
            move |c, y| corpo_do_modo(c, theme, y),
        ),
        cartao_de_cores(
            i::TOKENS_SEC_APP,
            tr("card.tokens.app_colors"),
            theme,
            armada,
            |t| familia_de(t).is_none(),
        ),
    ];
    for f in &FAMILIAS {
        let id = f.id;
        out.push(cartao_de_cores(id, tr(f.rotulo), theme, armada, move |t| {
            familia_de(t).is_some_and(|g| g.id == id)
        }));
    }
    out.push(cartao(
        i::TOKENS_SEC_SCALE,
        tr("panel.tokens.numeric"),
        Some(NumToken::ALL.len().to_string()),
        move |c, y| {
            (0..NumToken::ALL.len()).fold(y, |y, row| {
                let estado = (armada_num == Some(row), fx == Some(row));
                linha_de_escala(c, theme, row, estado, y)
            })
        },
    ));
    out
}
