//! Os parâmetros de uma camada de ajuste em linhas de cartão (fatia H11, `docs/CartoesUI/13_PAINTER_PLANO.md`): barras,
//! curva, degradê, Mixer, Selective Color, Black & White. Mesmos ids e mesmos eventos de `paint_adjust`, que continua
//! a ser o corpo de `PH2D_CARDS=0`. As tabelas são as da crate de efeitos: uma espécie nova não pede código aqui.

use crate::cards_linhas::{arma_barra, em_pares, escolha};
use crate::cards_tela::Tela;
use crate::paint::register_button;
use crate::paint_adjust::{
    CURVE_TAB_LABELS, SELCOLOR_TABS, barra_na_tela, curva_na_tela, ids_da_barra, ligar_numero,
    segment_slot_kind, toggle_slot_kind,
};
use crate::paint_rows::register_bare_slider;
use crate::{adjust_nomes, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::linhas_extra::CardBar;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids::{
    painter_curve_add_id, painter_curve_remove_id, painter_curve_tab_id, painter_gradient_add_id,
    painter_gradient_remove_id, painter_layer_widget_id, painter_mixer_tab_id,
    painter_selcolor_bucket_id,
};
use ph2d_tool_painter::{AdjustmentParams, CurvesParams, SliderNumber};

/// As barras até aqui são os seis tons do Black & White; o que vem depois é a tinta, que o interruptor governa.
const PESOS_DO_PRETO_E_BRANCO: usize = 6;
/// O número mostra-se com o passo da pista: 1 % de uma faixa de 100 é um inteiro.
const FAIXA_SEM_CASAS: f32 = 100.0; // LITERAL-PX-OK: faixa do número, não é medida de UI
const CASAS_DE_FRACAO: usize = 2;

const DICAS_DA_SELETIVA: [&str; 9] = [
    "card.painter.adjust.reds",
    "card.painter.adjust.yellows",
    "card.painter.adjust.greens",
    "card.painter.adjust.cyans",
    "card.painter.adjust.blues",
    "card.painter.adjust.magentas",
    "card.painter.adjust.whites",
    "card.painter.adjust.neutrals",
    "card.painter.adjust.blacks",
];

/// Uma barra deste ajuste: o nome, os ids, a pista viva e a de fábrica.
struct Barra {
    nome: &'static str,
    slider: NodeId,
    chip: NodeId,
    pista: f32,
    numero: SliderNumber,
    fabrica: f32,
}

fn monocromatico(p: &AdjustmentParams) -> bool {
    ph2d_tool_painter::adjustment_toggle_params(p)
        .first()
        .is_some_and(|(_, on)| *on)
}

/// The Channel Mixer output row the panel shows AND the weight sliders write (B39.1): Monochrome has
/// a single Gray row — the engine's `red_out` — whatever tab was last picked.
pub(crate) fn saida(layer: u64, p: &AdjustmentParams) -> u8 {
    if monocromatico(p) {
        0
    } else {
        state::active_mixer_channel(layer).min(2)
    }
}

fn parada(layer: u64, paradas: usize) -> usize {
    state::selected_gradient_stop(layer).min(paradas.saturating_sub(1))
}

/// As pistas à vista, slot a slot: a tabela que cada espécie usa no painel de antes.
fn pistas(layer: u64, p: &AdjustmentParams) -> Vec<(&'static str, f32)> {
    match p {
        AdjustmentParams::Curves(_) => Vec::new(),
        AdjustmentParams::ChannelMixer(mx) => {
            ph2d_tool_painter::channel_mixer_slider_params(mx, usize::from(saida(layer, p)))
                .into_iter()
                .collect()
        }
        AdjustmentParams::SelectiveColor(s) => ph2d_tool_painter::selective_color_slider_params(
            s,
            usize::from(state::active_selective_bucket(layer).min(8)),
        )
        .into_iter()
        .collect(),
        AdjustmentParams::GradientMap(g) => {
            ph2d_tool_painter::gradient_stop_color_params(g, parada(layer, g.stops.len()))
                .into_iter()
                .collect()
        }
        _ => ph2d_tool_painter::adjustment_slider_params(p)
            .into_iter()
            .collect(),
    }
}

fn numeros(p: &AdjustmentParams) -> Vec<SliderNumber> {
    match p {
        AdjustmentParams::Curves(_) => Vec::new(),
        AdjustmentParams::ChannelMixer(_) => ph2d_tool_painter::channel_mixer_slider_numbers()
            .into_iter()
            .collect(),
        AdjustmentParams::SelectiveColor(_) => ph2d_tool_painter::selective_color_slider_numbers()
            .into_iter()
            .collect(),
        AdjustmentParams::GradientMap(_) => ph2d_tool_painter::gradient_stop_color_numbers()
            .into_iter()
            .collect(),
        _ => ph2d_tool_painter::adjustment_slider_numbers(p)
            .into_iter()
            .collect(),
    }
}

fn chave(p: &AdjustmentParams, fonte: &str) -> Option<&'static str> {
    match p {
        AdjustmentParams::ChannelMixer(_) => adjust_nomes::chave_do_misturador(fonte),
        AdjustmentParams::SelectiveColor(_) => adjust_nomes::chave_da_seletiva(fonte),
        AdjustmentParams::GradientMap(_) => adjust_nomes::chave_do_gradiente(fonte),
        _ => adjust_nomes::chave_da_barra(p, fonte),
    }
}

fn barras(layer: u64, p: &AdjustmentParams) -> Vec<Barra> {
    let neutro = AdjustmentParams::neutral_for(p.kind());
    let de_fabrica = pistas(layer, &neutro);
    let nums = numeros(p);
    pistas(layer, p)
        .into_iter()
        .zip(nums)
        .enumerate()
        .filter_map(|(slot, ((fonte, pista), numero))| {
            let (slider, chip) = ids_da_barra(layer, slot)?;
            Some(Barra {
                nome: adjust_nomes::pintar(chave(p, fonte), fonte),
                slider,
                chip,
                pista: pista.clamp(0.0, 1.0),
                numero,
                fabrica: de_fabrica.get(slot).map_or(pista, |(_, f)| *f),
            })
        })
        .collect()
}

fn casas(n: SliderNumber) -> usize {
    match n {
        SliderNumber::Affine { integer: true, .. } => 0,
        SliderNumber::Affine { scale, .. } if scale.abs() >= FAIXA_SEM_CASAS => 0,
        _ => CASAS_DE_FRACAO,
    }
}

/// Os botões e as abas têm estado e dica: o corpo não escreve no store.
fn registra(store: &mut WidgetStore, ids: &[(NodeId, &str)]) {
    for (id, dica) in ids {
        register_button(store, *id);
        if !dica.is_empty() {
            store.set_tooltip(*id, tr(dica));
        }
    }
}

/// Com o store MUTÁVEL, antes de pintar: uma camada de ajuste.
pub(crate) fn prepare(store: &mut WidgetStore, layer: u64, p: &AdjustmentParams) {
    for b in barras(layer, p) {
        register_bare_slider(store, b.slider, b.pista);
        let chip = ligar_numero(store, b.slider, b.chip, b.numero);
        if chip != NodeId(0) {
            let n = casas(b.numero);
            arma_barra(store, [b.slider, chip], b.pista, b.fabrica, n);
            // A pista é um `f32`: `0.65 * 100.0` apareceria `65.000` no campo.
            let fator = 10_f32.powi(n as i32);
            let mostrado = store.linked_chip_display(chip, b.pista);
            crate::number_field::mirror_value(store, chip, (mostrado * fator).round() / fator, n);
        }
    }
    let toggles = ph2d_tool_painter::adjustment_toggle_params(p);
    for slot in 0..toggles.len() {
        if let Some(k) = toggle_slot_kind(slot) {
            register_button(store, painter_layer_widget_id(layer, k));
        }
    }
    if let Some((opcoes, _)) = ph2d_tool_painter::adjustment_segment_params(p) {
        for o in 0..opcoes.len() {
            if let Some(k) = segment_slot_kind(o) {
                register_button(store, painter_layer_widget_id(layer, k));
            }
        }
    }
    match p {
        AdjustmentParams::Curves(_) => {
            for ch in 0..CURVE_TAB_LABELS.len() {
                register_button(store, painter_curve_tab_id(layer, ch as u8));
            }
            registra(
                store,
                &[
                    (
                        painter_curve_add_id(layer),
                        "card.painter.adjust.add_point_tip",
                    ),
                    (
                        painter_curve_remove_id(layer),
                        "card.painter.adjust.remove_point_tip",
                    ),
                ],
            );
        }
        AdjustmentParams::GradientMap(_) => registra(
            store,
            &[
                (painter_gradient_add_id(layer), "card.painter.ramp.add_tip"),
                (
                    painter_gradient_remove_id(layer),
                    "card.painter.ramp.remove_tip",
                ),
            ],
        ),
        AdjustmentParams::ChannelMixer(_) => {
            for ch in 0..3 {
                register_button(store, painter_mixer_tab_id(layer, ch));
            }
        }
        AdjustmentParams::SelectiveColor(_) => {
            for (i, dica) in DICAS_DA_SELETIVA.iter().enumerate() {
                registra(store, &[(painter_selcolor_bucket_id(layer, i as u8), dica)]);
            }
        }
        _ => {}
    }
}

/// Uma barra: o campo, ou, onde o número só se mostra (o Gamma do Levels), a pista nua com o valor no rótulo.
fn barra(c: &mut CardCtx<'_>, b: &Barra, y: f32) -> f32 {
    if let SliderNumber::Shown(v) = b.numero {
        let rotulo = format!(
            "{} {}",
            tr(b.nome),
            ph2d_editor_core::widget::zero_sem_sinal(&format!("{v:.2}"))
        );
        let fim = c.full(Some(&rotulo), Widget::Readout { text: "" }, y);
        let r = Rect::new(c.x, fim - m::FIELD_H, c.w, m::FIELD_H);
        c.bar(
            &CardBar {
                id: b.slider,
                value: b.pista,
                vertical: false,
                ticks: &[],
            },
            r,
        );
        return fim;
    }
    let nome = tr(b.nome);
    c.row(
        &[Cell::new(Widget::Field {
            id: b.chip,
            glyph: Glyph::Text(linhas::letra_da_grandeza(nome)),
        })
        .label(nome)],
        y,
    )
}

/// Dois a dois enquanto os nomes couberem na metade (05 §4, leis 1 e 5).
fn fila_de_barras(c: &mut CardCtx<'_>, lista: &[Barra], y: f32) -> f32 {
    let meia = ((c.w - m::FIELD_GAP) * 0.5).max(0.0);
    let mut y = y;
    for par in lista.chunks(2) {
        let cabe = par
            .iter()
            .all(|b| c.text_system.prefix_width(tr(b.nome), m::LABEL_PX) <= meia);
        if cabe {
            y = em_pares(c, par, y, barra);
        } else {
            for b in par {
                y = barra(c, b, y);
            }
        }
    }
    y
}

fn interruptores(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let toggles = ph2d_tool_painter::adjustment_toggle_params(p);
    let cells: Vec<Cell<'_>> = toggles
        .iter()
        .enumerate()
        .filter_map(|(slot, (fonte, on))| {
            let id = painter_layer_widget_id(layer, toggle_slot_kind(slot)?);
            let nome = adjust_nomes::chave_do_interruptor(fonte).map_or_else(
                || Rotulo::vista_de(fonte),
                |k| Rotulo::vista(TextKey::new(k)),
            );
            Some(Cell::new(Widget::Check {
                id,
                label: nome,
                value: if *on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
                off: false,
            }))
        })
        .collect();
    if cells.is_empty() {
        return y;
    }
    c.compact(&cells, y)
}

/// O nome da escolha de cada espécie: uma lista de opções soltas não se explica sozinha.
fn rotulo_da_escolha(p: &AdjustmentParams) -> &'static str {
    tr(match p {
        AdjustmentParams::ColorBalance(_) => "card.painter.adjust.tonal_range",
        AdjustmentParams::GradientMap(_) => "card.painter.ramp.interpolation",
        AdjustmentParams::SelectiveColor(_) => "card.painter.adjust.method",
        _ => "card.painter.adjust.kind",
    })
}

fn escolha_da_especie(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let Some((opcoes, escolhida)) = ph2d_tool_painter::adjustment_segment_params(p) else {
        return y;
    };
    let segs: Vec<Seg<'_>> = opcoes
        .iter()
        .enumerate()
        .filter_map(|(i, nome)| {
            Some(Seg {
                id: painter_layer_widget_id(layer, segment_slot_kind(i)?),
                icon: None,
                text: adjust_nomes::pintar(adjust_nomes::chave_do_segmento(nome), nome),
                on: i == escolhida,
            })
        })
        .collect();
    escolha(c, rotulo_da_escolha(p), &segs, y)
}

/// A escolha cabe em meia linha? Cada opção na sua parte, com o texto e a folga dele.
fn escolha_cabe_em_meia(c: &mut CardCtx<'_>, p: &AdjustmentParams, meia: f32) -> bool {
    ph2d_tool_painter::adjustment_segment_params(p).is_some_and(|(opcoes, _)| {
        let parte = meia / opcoes.len().max(1) as f32;
        opcoes.iter().all(|nome| {
            let texto = adjust_nomes::pintar(adjust_nomes::chave_do_segmento(nome), nome);
            c.text_system.prefix_width(texto, m::VALUE_PX) + 2.0 * m::SEG_INSET <= parte
        })
    })
}

/// As barras e a escolha: a última barra sem par divide a linha com a escolha curta (05 §4, lei 1).
fn barras_e_escolha(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let todas = barras(layer, p);
    let (x, w) = (c.x, c.w);
    let meia = ((w - m::FIELD_GAP) * 0.5).max(0.0);
    let par_com_a_ultima = todas.len() % 2 == 1 && escolha_cabe_em_meia(c, p, meia);
    if !par_com_a_ultima {
        let y = fila_de_barras(c, &todas, y);
        return escolha_da_especie(c, layer, p, y);
    }
    let (resto, ultima) = todas.split_at(todas.len() - 1);
    let y = fila_de_barras(c, resto, y);
    c.w = meia;
    let fim = barra(c, &ultima[0], y);
    c.x = x + meia + m::FIELD_GAP;
    let fim2 = escolha_da_especie(c, layer, p, y);
    (c.x, c.w) = (x, w);
    fim.max(fim2)
}

/// Um «+» e um «−» sem texto.
fn mais_e_menos(mais: NodeId, menos: NodeId) -> [Seg<'static>; 2] {
    [
        Seg {
            id: mais,
            icon: Some(IconId::Plus),
            text: "",
            on: false,
        },
        Seg {
            id: menos,
            icon: Some(IconId::Minus),
            text: "",
            on: false,
        },
    ]
}

fn curva(c: &mut CardCtx<'_>, layer: u64, cp: &CurvesParams, y: f32) -> f32 {
    let ativo = state::active_curve_channel(layer);
    let abas: Vec<Seg<'_>> = CURVE_TAB_LABELS
        .iter()
        .enumerate()
        .map(|(ch, nome)| Seg {
            id: painter_curve_tab_id(layer, ch as u8),
            icon: None,
            text: nome,
            on: ch as u8 == ativo,
        })
        .collect();
    let botoes = mais_e_menos(painter_curve_add_id(layer), painter_curve_remove_id(layer));
    let y = c.row(
        &[
            Cell::new(Widget::Choice(&abas)).label(tr("card.painter.adjust.channel")),
            Cell::new(Widget::Bare(&botoes)),
        ],
        y,
    );
    let (x, w) = (c.x, c.w);
    curva_na_tela(
        &mut Tela::do_cartao(c),
        layer,
        cp,
        x,
        w,
        y + m::FIELD_ROW_GAP,
    )
}

fn degrade(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let AdjustmentParams::GradientMap(g) = p else {
        return y;
    };
    let contagem = g.stops.len().to_string();
    let botoes = mais_e_menos(
        painter_gradient_add_id(layer),
        painter_gradient_remove_id(layer),
    );
    let y = c.row(
        &[
            Cell::new(Widget::Readout { text: &contagem }).label(tr("card.painter.adjust.stops")),
            Cell::new(Widget::Bare(&botoes)),
        ],
        y,
    );
    let (x, w) = (c.x, c.w);
    let sel = parada(layer, g.stops.len());
    let y = barra_na_tela(
        &mut Tela::do_cartao(c),
        layer,
        p,
        sel,
        x,
        w,
        y + m::FIELD_ROW_GAP,
    );
    barras_e_escolha(c, layer, p, y)
}

fn misturador(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let ativa = saida(layer, p);
    let nomes: &[&str] = if monocromatico(p) {
        &[tr("panel.painter_layers.adjust.gray")]
    } else {
        &[
            tr("panel.painter_layers.adjust.red"),
            tr("panel.painter_layers.adjust.green"),
            tr("panel.painter_layers.adjust.blue"),
        ]
    };
    let abas: Vec<Seg<'_>> = nomes
        .iter()
        .enumerate()
        .map(|(ch, nome)| Seg {
            id: painter_mixer_tab_id(layer, ch as u8),
            icon: None,
            text: nome,
            on: ch as u8 == ativa,
        })
        .collect();
    let y = escolha(c, tr("card.painter.adjust.output"), &abas, y);
    let y = fila_de_barras(c, &barras(layer, p), y);
    interruptores(c, layer, p, y)
}

fn seletiva(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let ativo = state::active_selective_bucket(layer).min(8);
    let abas: Vec<Seg<'_>> = SELCOLOR_TABS
        .iter()
        .enumerate()
        .map(|(i, nome)| Seg {
            id: painter_selcolor_bucket_id(layer, i as u8),
            icon: None,
            text: nome,
            on: i as u8 == ativo,
        })
        .collect();
    let y = escolha(c, tr("card.painter.adjust.color_group"), &abas, y);
    let y = fila_de_barras(c, &barras(layer, p), y);
    escolha_da_especie(c, layer, p, y)
}

fn preto_e_branco(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    let todas = barras(layer, p);
    let corte = todas.len().min(PESOS_DO_PRETO_E_BRANCO);
    let y = fila_de_barras(c, &todas[..corte], y);
    let y = interruptores(c, layer, p, y);
    fila_de_barras(c, &todas[corte..], y)
}

/// O corpo de um ajuste, debaixo da linha dele.
pub(crate) fn corpo(c: &mut CardCtx<'_>, layer: u64, p: &AdjustmentParams, y: f32) -> f32 {
    match p {
        AdjustmentParams::Curves(cp) => curva(c, layer, cp, y),
        AdjustmentParams::ChannelMixer(_) => misturador(c, layer, p, y),
        AdjustmentParams::BlackAndWhite(_) => preto_e_branco(c, layer, p, y),
        AdjustmentParams::SelectiveColor(_) => seletiva(c, layer, p, y),
        AdjustmentParams::GradientMap(_) => degrade(c, layer, p, y),
        _ => {
            let y = barras_e_escolha(c, layer, p, y);
            let y = interruptores(c, layer, p, y);
            if ph2d_tool_painter::relief_effect(p.kind()) == ph2d_tool_painter::ReliefEffect::Tone {
                c.hint(tr("panel.painter_layers.adjust.tone_not_relief"), y)
            } else {
                y
            }
        }
    }
}
