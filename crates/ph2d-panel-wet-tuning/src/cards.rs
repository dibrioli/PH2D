//! ⭐⭐ **O painel Wet Paint em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): a MESMA
//! tabela ([`rows::rows`]) e os MESMOS ids do corpo de antes; o despacho (`event.rs`) não muda. Cada
//! knob continua com o seu repor, e a linha inerte continua esmaecida e editável. Janela FLUTUANTE:
//! a lista pinta-se no rect que o artista arrastou e redimensionou (dono E10: nada de avançado se perde).

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget};
use ph2d_editor_core::panel::{PaintCtx, TextKey};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::panel_chrome::{
    paint_panel_corner_dot, paint_panel_corner_dot_bl, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{IconId, paint::fill_rounded_rect};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::ColorToken;
use ph2d_tool_painter::BrushSettings;
use ph2d_tool_painter::ids as tid;

use crate::rows::{self, TuneRow};

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

/// O rótulo de cartão dos knobs com unidade (D13): a unidade mora no rótulo, nunca na caixa.
const COM_UNIDADE: &[(&str, &str)] = &[("textureSizeRef", "card.wet_tuning.texture_size_ref")];

fn rotulo(row: &'static TuneRow) -> &'static str {
    COM_UNIDADE
        .iter()
        .find(|(k, _)| *k == row.key)
        .map_or_else(|| tr(row.label.as_str()), |(_, chave)| tr(chave))
}

/// Antes de pintar: os campos marcados, os valores do pincel nas caixas e a dica de cada ícone. O
/// campo que a mão arrasta não se semeia, senão o valor do pincel puxava-o debaixo do gesto.
pub(crate) fn prepare(store: &mut WidgetStore, brush: &BrushSettings) {
    store.mark_card_list_close(ids::WET_TUNING_PANEL, tid::WET_TUNING_CLOSE);
    let arrastado = store.number_input_drag().map(|d| d.id);
    for row in rows::rows() {
        if store.tooltip_for(row.reset).is_none() {
            store.set_tooltip(
                row.reset,
                tr_with("card.wet_tuning.reset_knob", &[("knob", &rotulo(row))]),
            );
        }
        if row.options.is_some() || row.interruptor() {
            continue;
        }
        store.mark_card_field(
            row.chip,
            CardFieldSpec {
                step: row.step,
                scrub: row.step,
                default: Some(row.default),
            },
        );
        if arrastado != Some(row.chip) {
            store.set_number_value(row.chip, brush.wet_knobs.knobs[row.knob]);
        }
    }
    for s in rows::SECTIONS {
        if store.tooltip_for(s.reset).is_none() {
            store.set_tooltip(
                s.reset,
                tr_with("card.wet_tuning.reset_group", &[("group", &tr(s.label))]),
            );
        }
    }
}

fn repor(row: &TuneRow) -> [Seg<'static>; 1] {
    [Seg {
        id: row.reset,
        icon: Some(IconId::Reset),
        text: "",
        on: false,
    }]
}

/// As linhas de números vizinhas: duas por fila, cada uma com o seu repor (T56). O véu da inerte só
/// cobre a metade dela.
fn numeros(c: &mut CardCtx<'_>, linhas: &[&'static TuneRow], brush: &BrushSettings, y: f32) -> f32 {
    // Dois campos com o seu repor só dividem a linha se o número couber (05 §4, lei 5).
    let um_par = (m::FIELD_VALUE_MIN_W + m::ICON_CELL * 2.0) * 2.0 + m::FIELD_GAP * 3.0; // LITERAL-PX-OK: dois campos, três vãos
    let por_fila = if c.w < um_par { 1 } else { 2 };
    linhas.chunks(por_fila).fold(y, |topo, par| {
        let repoes: Vec<[Seg<'static>; 1]> = par.iter().map(|r| repor(r)).collect();
        let mut cells: Vec<Cell<'_>> = Vec::with_capacity(par.len() * 2);
        for (r, seg) in par.iter().zip(&repoes) {
            let label = rotulo(r);
            cells.push(
                Cell::new(Widget::Field {
                    id: r.chip,
                    glyph: Glyph::Text(letra_da_grandeza(label)),
                })
                .label(label),
            );
            cells.push(Cell::new(Widget::Bare(seg)));
        }
        let fim = c.row(&cells, topo);
        let w = if par.len() == 2 {
            (c.w - m::FIELD_GAP) * 0.5
        } else {
            c.w
        };
        for (n, r) in par.iter().enumerate() {
            if brush.inercias.dica_de(r.slider).is_some() {
                let x = c.x + (w + m::FIELD_GAP) * n as f32;
                veu(c, Rect::new(x, topo, w, fim - topo));
            }
        }
        fim
    })
}

/// ⭐ **A linha ESMAECIDA** (a lei `Inercias`, a mesma do corpo de antes): o véu da tinta do cartão por
/// cima dela. Continua editável: o artista prepara o valor antes de ligar o que o acorda.
fn veu(c: &mut CardCtx<'_>, area: Rect) {
    let alfa = ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme);
    let cor = c
        .ink
        .color(ColorToken::KanbanCard)
        .multiply_alpha(1.0 - alfa);
    fill_rounded_rect(c.scene, area, 0.0, cor);
}

fn escolha(
    c: &mut CardCtx<'_>,
    row: &'static TuneRow,
    opcoes: &'static [(String, NodeId); 2],
    brush: &BrushSettings,
    y: f32,
) -> f32 {
    let aceso = brush.wet_knobs.knobs[row.knob].round() as usize;
    let segs: Vec<Seg<'_>> = opcoes
        .iter()
        .enumerate()
        .map(|(i, (k, id))| Seg {
            id: *id,
            icon: None,
            text: tr(k.as_str()),
            on: i == aceso,
        })
        .collect();
    let seg = repor(row);
    let fim = c.row(
        &[
            Cell::new(Widget::Choice(&segs)).label(rotulo(row)),
            Cell::new(Widget::Bare(&seg)),
        ],
        y,
    );
    if brush.inercias.dica_de(row.slider).is_some() {
        veu(c, Rect::new(c.x, y, c.w, fim - y));
    }
    fim
}

/// Um interruptor (B35.7): a caixa de marcar com o nome dela e o repor; o clique é o `Click` do chip (o id do slider é uma barra no store).
fn caixa(c: &mut CardCtx<'_>, row: &'static TuneRow, brush: &BrushSettings, y: f32) -> f32 {
    let value = if brush.wet_knobs.knobs[row.knob] > 0.5 {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let seg = repor(row);
    let fim = c.row(
        &[
            Cell::new(Widget::Check {
                id: row.chip,
                label: Rotulo::vista_de(rotulo(row)),
                value,
                off: false,
            }),
            Cell::new(Widget::Bare(&seg)),
        ],
        y,
    );
    if brush.inercias.dica_de(row.slider).is_some() {
        veu(c, Rect::new(c.x, y, c.w, fim - y));
    }
    fim
}

/// O rótulo cabe, inteiro, nas duas linhas de meia coluna? Quebra como a faixa dos rótulos: na última
/// palavra que cabe. O que não cabe ganha a linha toda: nome de controlo nunca vira «...».
fn cabe_em_duas_linhas(c: &mut CardCtx<'_>, label: &'static str) -> bool {
    let w = (c.w - m::FIELD_GAP) * 0.5 - m::FIELD_GAP - m::FIELD_H;
    let ts = &mut *c.text_system;
    cabe_memo(w, label, |t| ts.prefix_width(t, m::LABEL_PX))
}

/// A resposta de [`cabe`] por rótulo, à largura do último quadro: quadro parado não mede texto (B35.A3).
fn cabe_memo(w: f32, label: &'static str, mede: impl FnMut(&str) -> f32) -> bool {
    thread_local! {
        static CABE: std::cell::RefCell<(u32, std::collections::BTreeMap<&'static str, bool>)> =
            const { std::cell::RefCell::new((0, std::collections::BTreeMap::new())) };
    }
    CABE.with_borrow_mut(|(largura, memo)| {
        if *largura != w.to_bits() {
            *largura = w.to_bits();
            memo.clear();
        }
        *memo.entry(label).or_insert_with(|| cabe(w, label, mede))
    })
}

fn cabe(w: f32, label: &str, mut mede: impl FnMut(&str) -> f32) -> bool {
    if mede(label) <= w {
        return true;
    }
    let corte = label
        .match_indices(' ')
        .map(|(i, _)| i)
        .filter(|&i| mede(&label[..i]) <= w)
        .last();
    corte.is_some_and(|i| mede(label[i + 1..].trim_start()) <= w)
}

/// O corpo de um grupo, pela ordem da tabela: as escolhas a toda a largura, os números aos pares.
fn corpo_do_grupo(
    c: &mut CardCtx<'_>,
    group: ph2d_wet_paint::tuning::KnobGroup,
    brush: &BrushSettings,
    y: f32,
) -> f32 {
    let mut y = y;
    let mut pendentes: Vec<&'static TuneRow> = Vec::new();
    for row in rows::rows()
        .iter()
        .filter(|r| r.group == group && r.visivel(brush))
    {
        match &row.options {
            Some(opcoes) => {
                y = numeros(c, &pendentes, brush, y);
                pendentes.clear();
                y = escolha(c, row, opcoes, brush, y);
            }
            None if row.interruptor() => {
                y = numeros(c, &pendentes, brush, y);
                pendentes.clear();
                y = caixa(c, row, brush, y);
            }
            None if !cabe_em_duas_linhas(c, rotulo(row)) => {
                y = numeros(c, &pendentes, brush, y);
                pendentes.clear();
                y = numeros(c, &[row], brush, y);
            }
            None => pendentes.push(row),
        }
    }
    numeros(c, &pendentes, brush, y)
}

/// ⭐ Os cartões do painel, pela ordem natural de [`crate::plano::seccoes_moveis`].
pub(crate) fn cards(brush: &BrushSettings) -> Vec<CardDef<'_>> {
    let mut out: Vec<CardDef<'_>> = rows::SECTIONS
        .iter()
        .map(|s| CardDef {
            section: s.header,
            title: tr(s.label),
            actions: vec![HeadAction {
                id: s.reset,
                icon: IconId::Reset,
                on: false,
            }],
            add: None,
            summary: None,
            body: Some(Box::new(move |c: &mut CardCtx<'_>, y: f32| {
                corpo_do_grupo(c, s.group, brush, y)
            })),
        })
        .collect();
    out.push(CardDef {
        section: crate::plano::EXPERIMENTAL,
        title: tr("panel.wet_tuning.group.experimental"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y: f32| {
            let y = c.check(
                tid::WET_TUNING_KM_GLAZE,
                Rotulo::vista(TextKey::new("panel.wet_tuning.km_glaze")),
                brush.wet_km_glaze,
                y,
            );
            c.hint(tr("panel.wet_tuning.note"), y)
        })),
    });
    out
}

/// A janela flutuante: a faixa do título move e os cantos redimensionam, como antes. Registados
/// DEPOIS da lista (o último ganha), a faixa pára antes do «…» e do «×» dela.
pub(crate) fn chrome(ctx: &mut PaintCtx, lista: Rect) {
    let theme = ctx.host.theme();
    paint_panel_corner_dot(lista, ctx.scene, theme);
    paint_panel_corner_dot_bl(lista, ctx.scene, theme);
    let reserva = m::HEAD_ICON_BOX * 2.0 + m::LIST_PAD_X;
    let hit = ctx.host.hit_index_mut();
    hit.register(
        tid::WET_TUNING_RESIZE_HANDLE,
        panel_resize_handle_rect(lista),
    );
    hit.register(
        tid::WET_TUNING_RESIZE_HANDLE_BL,
        panel_resize_handle_rect_bl(lista),
    );
    hit.register(
        tid::WET_TUNING_DRAG_HANDLE,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B35.A3: o segundo quadro à mesma largura não mede texto; outra largura volta a medir.
    #[test]
    fn o_tuning_nao_mede_texto_em_quadro_parado() {
        let rotulos: [&'static str; 3] = ["Pigment load", "Edge darkening strength", "Wet"];
        let medidas = std::cell::Cell::new(0usize);
        let mede = |t: &str| {
            medidas.set(medidas.get() + 1);
            t.len() as f32 * 7.0
        };
        let w = 80.0;
        let primeiro: Vec<bool> = rotulos.iter().map(|r| cabe_memo(w, r, mede)).collect();
        let antes = medidas.get();
        assert!(antes > 0);
        let segundo: Vec<bool> = rotulos.iter().map(|r| cabe_memo(w, r, mede)).collect();
        assert_eq!(primeiro, segundo);
        assert_eq!(medidas.get(), antes, "quadro parado voltou a medir texto");
        let outro: Vec<bool> = rotulos
            .iter()
            .map(|r| cabe_memo(w * 3.0, r, mede))
            .collect();
        assert!(medidas.get() > antes, "outra largura tem de medir de novo");
        assert_eq!(
            outro,
            rotulos.map(|r| cabe(w * 3.0, r, |t| t.len() as f32 * 7.0))
        );
    }
}
