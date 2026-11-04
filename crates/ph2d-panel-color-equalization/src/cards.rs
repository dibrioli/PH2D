//! ⭐ **O painel Color Equalization em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os
//! mesmos ids, as mesmas linhas de [`crate::rows`] com a mesma lei `show`, agrupadas por assunto.
//! `event` intocado; o `populate` só troca a espécie da caixa de lista suspensa (ver [`em_cartoes`]).

use crate::paint_sections::{lut_options_for_slot, posterize_options, quantize_options};
use crate::rows::{SliderRow, build_slider_rows};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::cards::dropdown;
use ph2d_editor_core::panel::cards::ferramenta::{
    Numero, Verbos, aos_pares, caixa, caixas_aos_pares, campo, cartao, prepara_numeros,
};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;
use ph2d_tool_color_equalization::ids as t;
use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot as Snap;

/// As caixas são `Button` nos cartões: a lista abre pelo `Click` (`panel::cards::dropdown::on_click`).
#[must_use]
pub(crate) fn em_cartoes() -> bool {
    ph2d_editor_core::panel::cards::ferramenta::cards_on()
}

/// A altura do histograma: 64 barras, o mesmo que o corpo de antes reservava.
pub(crate) const HISTOGRAM_H: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (histogram strip height)

const PASSO: f64 = 0.01; // LITERAL-PX-OK: passo de seta na faixa 0..1 do valor da ferramenta
const ARRASTO: f64 = 0.005; // LITERAL-PX-OK: valor por px de arrasto na faixa 0..1 da ferramenta
const INTEIRO: f64 = 1.0;

/// O passo de cada número: os inteiros e a faixa estreita do Clip.
fn passo_de(chip: NodeId) -> (f64, f64) {
    if chip == t::CEQ_TILE_GRID_NUM || chip == t::CEQ_POSTERIZE_DITHER_GRAIN_NUM {
        (INTEIRO, INTEIRO)
    } else {
        (PASSO, ARRASTO)
    }
}

/// O rótulo de cartão das linhas com unidade (D13); as outras reusam a chave de antes.
fn rotulo(r: &SliderRow) -> &'static str {
    if r.chip_id == t::CEQ_EXPOSURE_NUM {
        tr("card.ceq.exposure")
    } else if r.chip_id == t::CEQ_SHARPEN_RADIUS_NUM {
        tr("card.ceq.radius")
    } else if r.chip_id == t::CEQ_SHARPEN_AMOUNT_NUM {
        // O cartão já se chama Sharpen.
        tr("card.ceq.amount")
    } else {
        r.label
    }
}

pub(crate) fn prepare(store: &mut WidgetStore, snap: &Snap) {
    for id in [
        t::CEQ_LUT_1_DROPDOWN,
        t::CEQ_LUT_2_DROPDOWN,
        t::CEQ_POSTERIZE_DROPDOWN,
        t::CEQ_QUANTIZE_DROPDOWN,
    ] {
        dropdown::declare(id);
    }
    let fabrica = Snap::default();
    let mut numeros: Vec<Numero> = build_slider_rows(snap)
        .iter()
        .zip(build_slider_rows(&fabrica).iter())
        .map(|(r, f)| {
            let (step, scrub) = passo_de(r.chip_id);
            Numero {
                chip: r.chip_id,
                step,
                scrub,
                fabrica: f.snap_chip,
                valor: r.snap_chip,
            }
        })
        .collect();
    numeros.push(Numero {
        chip: t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
        step: PASSO,
        scrub: ARRASTO,
        fabrica: f64::from(fabrica.posterize_dither_strength),
        valor: f64::from(snap.posterize_dither_strength),
    });
    numeros.push(Numero {
        chip: t::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
        step: INTEIRO,
        scrub: INTEIRO,
        fabrica: f64::from(fabrica.posterize_dither_grain),
        valor: f64::from(snap.posterize_dither_grain),
    });
    prepara_numeros(store, &numeros);
    // Os dois que menos se abrem nascem dobrados: um painel de 13 números não cabe num ecrã.
    store.set_collapsed_if_unchosen(t::CEQ_SEC_LUT, true);
    store.set_collapsed_if_unchosen(t::CEQ_SEC_STYLIZE, true);
}

/// Os campos da tabela que este estado mostra, na ordem dada.
fn campos(snap: &Snap, chips: &[NodeId]) -> Vec<Cell<'static>> {
    let rows = build_slider_rows(snap);
    chips
        .iter()
        .filter_map(|chip| rows.iter().find(|r| r.chip_id == *chip))
        .filter(|r| (r.show)(snap))
        .map(|r| campo(r.chip_id, rotulo(r)))
        .collect()
}

fn cartao_de_numeros(
    section: NodeId,
    title: &'static str,
    snap: Snap,
    chips: &'static [NodeId],
) -> CardDef<'static> {
    cartao(section, title, None, move |c, y| {
        aos_pares(c, &campos(&snap, chips), y)
    })
}

/// Os segmentos de uma lista suspensa, e o nome da opção escolhida.
fn lista<T: PartialEq>(
    opcoes: &[DropdownOption<T>],
    escolhida: &T,
) -> (Vec<(NodeId, String, bool)>, String) {
    let nome = opcoes
        .iter()
        .find(|o| o.value == *escolhida)
        .map_or_else(String::new, |o| o.label.clone());
    let linhas = opcoes
        .iter()
        .map(|o| (o.id, o.label.clone(), o.value == *escolhida))
        .collect();
    (linhas, nome)
}

fn segs(linhas: &[(NodeId, String, bool)]) -> Vec<Seg<'_>> {
    linhas
        .iter()
        .map(|(id, text, on)| Seg {
            id: *id,
            icon: None,
            text,
            on: *on,
        })
        .collect()
}

fn cartao_do_histograma() -> CardDef<'static> {
    cartao(
        t::CEQ_SEC_HISTOGRAM,
        tr("card.ceq.histogram"),
        None,
        move |c, y| {
            let ink = c.ink;
            c.legacy(y, |r, y| {
                let rect = Rect::new(r.inner_x, y, r.inner_w, HISTOGRAM_H);
                crate::state::with_current_histogram(|h| {
                    crate::paint_histogram::paint_histogram_overlay(
                        rect,
                        h,
                        r.scene,
                        r.theme,
                        &|t| ink.color(t),
                    );
                });
                y + HISTOGRAM_H
            })
        },
    )
}

fn cartao_do_auto(snap: Snap) -> CardDef<'static> {
    cartao(t::CEQ_SEC_AUTO, tr("card.ceq.auto"), None, move |c, y| {
        caixas_aos_pares(
            c,
            &[
                (
                    t::CEQ_AUTO_LEVELS,
                    "panel.color_eq.adjust.auto_levels",
                    snap.auto_levels,
                ),
                (
                    t::CEQ_AUTO_CONTRAST,
                    "panel.color_eq.adjust.auto_contrast",
                    snap.auto_contrast,
                ),
                (
                    t::CEQ_AUTO_COLORS,
                    "panel.color_eq.adjust.auto_colors",
                    snap.auto_colors,
                ),
                (
                    t::CEQ_AUTO_WB,
                    "panel.color_eq.adjust.auto_wb",
                    snap.auto_wb,
                ),
            ],
            y,
        )
    })
}

fn cartao_do_lut(snap: Snap) -> CardDef<'static> {
    cartao(t::CEQ_SEC_LUT, tr("card.ceq.lut"), None, move |c, y| {
        let (l1, n1) = lista(&lut_options_for_slot(1), &snap.lut_preset_1);
        let (l2, n2) = lista(&lut_options_for_slot(2), &snap.lut_preset_2);
        let (s1, s2) = (segs(&l1), segs(&l2));
        let y = c.row(
            &[
                Cell::new(Widget::Dropdown {
                    id: t::CEQ_LUT_1_DROPDOWN,
                    text: &n1,
                    options: &s1,
                })
                .label(tr("panel.color_eq.adjust.lut_1")),
                Cell::new(Widget::Dropdown {
                    id: t::CEQ_LUT_2_DROPDOWN,
                    text: &n2,
                    options: &s2,
                })
                .label(tr("panel.color_eq.adjust.lut_2")),
            ],
            y,
        );
        aos_pares(
            c,
            &campos(&snap, &[t::CEQ_LUT_INTENSITY_NUM, t::CEQ_LUT_MIX_NUM]),
            y,
        )
    })
}

/// Posterize e Quantize lado a lado (listas curtas), e o Dither de um e o outro em baixo.
fn posterizar(c: &mut CardCtx<'_>, snap: &Snap, y: f32) -> f32 {
    let (linhas, nome) = lista(&posterize_options(), &snap.posterize_levels);
    let opcoes = segs(&linhas);
    let (linhas_q, nome_q) = lista(&quantize_options(), &snap.quantize_colors);
    let opcoes_q = segs(&linhas_q);
    let mut y = c.row(
        &[
            Cell::new(Widget::Dropdown {
                id: t::CEQ_POSTERIZE_DROPDOWN,
                text: &nome,
                options: &opcoes,
            })
            .label(tr("panel.color_eq.adjust.posterize")),
            Cell::new(Widget::Dropdown {
                id: t::CEQ_QUANTIZE_DROPDOWN,
                text: &nome_q,
                options: &opcoes_q,
            })
            .label(tr("panel.color_eq.adjust.quantize")),
        ],
        y,
    );
    // Sem Posterize não há etapa que o Dither modifique; sem Dither, nem força nem grão.
    if snap.posterize_stage_runs() {
        y = caixa(
            c,
            t::CEQ_POSTERIZE_DITHERING,
            "panel.color_eq.adjust.dither",
            snap.posterize_dithering,
            y,
        );
    }
    if snap.dither_stage_runs() {
        let cells = [
            campo(
                t::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
                tr("panel.color_eq.adjust.dither_strength"),
            ),
            campo(
                t::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
                tr("panel.color_eq.adjust.dither_grain"),
            ),
        ];
        y = c.row(&cells, y);
    }
    y
}

fn cartao_do_estilo(snap: Snap) -> CardDef<'static> {
    cartao(
        t::CEQ_SEC_STYLIZE,
        tr("card.ceq.stylize"),
        None,
        move |c, y| posterizar(c, &snap, y),
    )
}

const TOM: &[NodeId] = &[
    t::CEQ_EXPOSURE_NUM,
    t::CEQ_BRIGHTNESS_NUM,
    t::CEQ_CONTRAST_NUM,
];
const COR: &[NodeId] = &[
    t::CEQ_TEMPERATURE_NUM,
    t::CEQ_TINT_NUM,
    t::CEQ_VIBRANCE_NUM,
    t::CEQ_SATURATION_NUM,
];
const NITIDEZ: &[NodeId] = &[t::CEQ_SHARPEN_AMOUNT_NUM, t::CEQ_SHARPEN_RADIUS_NUM];
const LOCAL: &[NodeId] = &[t::CEQ_CLIP_LIMIT_NUM, t::CEQ_TILE_GRID_NUM];

pub(crate) fn cards(snap: &Snap) -> Vec<CardDef<'static>> {
    let s = *snap;
    vec![
        cartao_do_histograma(),
        cartao_do_auto(s),
        cartao_de_numeros(t::CEQ_SEC_TONE, tr("card.ceq.tone"), s, TOM),
        cartao_de_numeros(t::CEQ_SEC_COLOR, tr("card.ceq.color"), s, COR),
        cartao_de_numeros(t::CEQ_SEC_SHARPEN, tr("card.ceq.sharpen"), s, NITIDEZ),
        cartao_de_numeros(t::CEQ_SEC_LOCAL, tr("card.ceq.local"), s, LOCAL),
        cartao_do_lut(s),
        cartao_do_estilo(s),
    ]
}

pub(crate) fn verbos() -> Verbos<'static> {
    Verbos {
        reset: (t::CEQ_RESET, tr("panel.color_eq.adjust.reset_to_defaults")),
        cancel: (t::CEQ_CANCEL, tr("panel.color_eq.adjust.cancel")),
        apply: (t::CEQ_APPLY, tr("panel.color_eq.adjust.apply")),
    }
}

/// O fundo do histograma no corpo de antes: o tema do painel.
#[must_use]
pub(crate) fn tinta_do_tema(
    theme: ph2d_tokens::Theme,
) -> impl Fn(ColorToken) -> ph2d_vector::Color {
    move |token| resolve(token, theme)
}
