//! ⭐ **T56 — dois campos numa linha do cartão** (ordem do dono 02/11: *«em Physic Body sempre que
//! fizer sentido, coloque 2 campos por linha»*; `docs/CartoesUI/01_decisoes_do_dono.md` T56). Só o
//! cartão muda: fora dele cada campo pinta-se como hoje, um por linha, pela mesma porta.

use super::rows::{num_row_unit, seg_row};
use super::*;
use ph2d_editor_core::panel::cards::corpo::{self, Coluna};
use ph2d_editor_core::widget::Unit;
use ph2d_editor_core::widget::key_mark::{Eixos, Rotulo};

/// ⭐ **A escolha que sabe dizer «misto»** (`selected` `None` = nenhum aceso): o rótulo em cima e os
/// segmentos do cartão, ou fora dele o nome à esquerda e o segmentado adaptativo. `vao` é o da secção.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_row_misto<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: impl Into<Rotulo<'a>>,
    (grupo, opcoes, nomes): (NodeId, &[NodeId], &[&str]),
    selected: Option<usize>,
    sec: ph2d_editor_core::property_row::Seccao,
    vao: f32,
) -> f32 {
    use ph2d_editor_core::widget::{SegmentedAdaptive, SegmentedOption, paint_segmented_adaptive};
    let rotulo = label.into();
    if let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() {
        let segs: Vec<ph2d_editor_core::panel::cards::Seg<'_>> = opcoes
            .iter()
            .zip(nomes)
            .enumerate()
            .map(|(i, (&id, &text))| ph2d_editor_core::panel::cards::Seg {
                id,
                icon: None,
                text,
                on: selected == Some(i),
            })
            .collect();
        return corpo::choices(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            y,
            rotulo.texto,
            opcoes,
            &segs,
        );
    }
    let h = ROW_H_PX;
    let row =
        super::rows::property_label_row(scene, text_system, theme, x, w, y, h, rotulo.texto, sec);
    let seg = SegmentedAdaptive::new(
        grupo,
        rotulo.texto,
        opcoes
            .iter()
            .zip(nomes)
            .map(|(&id, &l)| SegmentedOption::new(id, l))
            .collect(),
    )
    .selected(selected.unwrap_or(usize::MAX));
    let seg_h = paint_segmented_adaptive(
        &seg,
        row.control,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene, theme, row.dot, opcoes, rotulo,
    ) {
        hit_index.register(alvo, row.dot);
    }
    y + seg_h.max(h) + vao
}

/// ⭐ **A linha de marcar que sabe dizer «misto»** (T29, os `Checkbox::new` directos das secções): a
/// do cartão (T64), ou fora dele a caixa de hoje com a coluna do losango pelo rótulo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn caixa_row<'a>(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    (id, label, valor): (NodeId, impl Into<Rotulo<'a>>, CheckboxValue),
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let rotulo = label.into();
    if let Some((ink, rows)) = ph2d_editor_core::property_row::cartao::card_rows() {
        return (rows.check)(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            y,
            id,
            rotulo,
            valor,
            false,
        );
    }
    let rect = Rect::new(x, y, w, ROW_H_PX);
    hit_index.register(id, rect);
    paint_checkbox(
        &Checkbox::new(id, rotulo.texto)
            .visual(store.checkbox_visual(id))
            .coluna_de_fora()
            .seccao(sec)
            .value(valor),
        rect,
        scene,
        text_system,
        theme,
    );
    super::rows::coluna_da_caixa(scene, theme, hit_index, rect, id, rotulo);
    y + ROW_H_PX + ph2d_tokens::control_gap_px()
}

/// Um campo de uma linha de dois: o rótulo e a unidade de HOJE (os do Inspector clássico).
pub(crate) struct Campo<'a> {
    pub rotulo: Rotulo<'a>,
    pub id: NodeId,
    pub unit: Option<Unit>,
}

impl<'a> Campo<'a> {
    pub fn new(rotulo: impl Into<Rotulo<'a>>, id: NodeId, unit: Option<Unit>) -> Self {
        Self {
            rotulo: rotulo.into(),
            id,
            unit,
        }
    }
}

/// Dois números numa linha do cartão. `titulo` = uma grandeza de duas componentes (um rótulo,
/// as letras de [`Eixos`]: «Offset (px)» X · Y); `None` = dois rótulos, um sobre cada caixa
/// («Bounce» | «Friction»). Fora do cartão: as duas linhas de hoje.
#[allow(clippy::too_many_arguments)]
pub(crate) fn par_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    titulo: Option<(&str, Eixos)>,
    campos: [Campo<'_>; 2],
) -> f32 {
    if let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() {
        let ids = [campos[0].id, campos[1].id];
        let u = |c: &Campo<'_>| crate::unidade_mundo::no_campo(&[c.id], c.unit, 0.0).0;
        let (u0, u1) = (u(&campos[0]), u(&campos[1]));
        let cols = match titulo {
            Some((rotulo, eixos)) => vec![Coluna {
                rotulo,
                eixos,
                unidade: u0.map(Unit::suffix),
                ids: &ids,
            }],
            None => vec![
                Coluna {
                    rotulo: campos[0].rotulo.texto,
                    eixos: campos[0].rotulo.eixos,
                    unidade: u0.map(Unit::suffix),
                    ids: &ids[..1],
                },
                Coluna {
                    rotulo: campos[1].rotulo.texto,
                    eixos: campos[1].rotulo.eixos,
                    unidade: u1.map(Unit::suffix),
                    ids: &ids[1..],
                },
            ],
        };
        // O passo de HOJE no cartão (`num_row_unit`: o `NumberInput` sem `.step` anda 1.0).
        return corpo::columns(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            y,
            &cols,
            1.0,
        );
    }
    let mut yy = y;
    for c in campos {
        yy = num_row_unit(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            c.rotulo,
            c.id,
            c.unit,
            None,
        );
    }
    yy
}

/// Uma escolha segmentada de uma linha de duas — os argumentos da [`seg_row`].
pub(crate) struct Escolha<'a> {
    pub rotulo: Rotulo<'a>,
    pub group: NodeId,
    pub ids: &'a [NodeId],
    pub labels: &'a [&'a str],
    pub selected: u8,
}

/// Duas escolhas lado a lado no cartão quando os segmentos cabem em meia linha
/// ([`corpo::choices_pair`]); senão — e fora do cartão — uma por linha, como hoje.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_par_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    a: Escolha<'_>,
    b: Escolha<'_>,
) -> f32 {
    if let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() {
        let (sa, sb) = (segs(&a), segs(&b));
        if let Some(yy) = corpo::choices_pair(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            x,
            w,
            y,
            (a.rotulo.texto, a.ids, &sa),
            (b.rotulo.texto, b.ids, &sb),
        ) {
            return yy;
        }
    }
    let mut yy = y;
    for e in [a, b] {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            e.rotulo,
            e.group,
            e.ids,
            e.labels,
            e.selected,
        );
    }
    yy
}

fn segs<'a>(e: &Escolha<'a>) -> Vec<ph2d_editor_core::panel::cards::Seg<'a>> {
    let sel = (e.selected as usize).min(e.labels.len() - 1);
    e.ids
        .iter()
        .zip(e.labels)
        .enumerate()
        .map(|(i, (&id, &text))| ph2d_editor_core::panel::cards::Seg {
            id,
            icon: None,
            text,
            on: i == sel,
        })
        .collect()
}

/// ⭐ **A caixa que liga e desliga uma dobra**, sem rótulo, no cabeçalho dela (a habilidade do Platform
/// Player, dono 10/11): `caixa` é a aresta do cabeçalho, não a altura de uma linha — por isso a sua
/// altura está nomeada em `a_marca_tem_a_altura_da_linha`. Alvo, hover e visto são os do cartão.
#[allow(clippy::too_many_arguments)]
pub(crate) fn caixa_de_cabecalho(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    id: NodeId,
    caixa: ph2d_editor_core::zones::Rect,
    ligada: bool,
) {
    hit_index.register(id, caixa);
    if let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() {
        let hover = store.hover_live(id) > 0.0;
        ph2d_editor_core::widget::paint_card_check(scene, ink, caixa, ligada, hover);
        return;
    }
    let valor = if ligada {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    ph2d_editor_core::widget::paint_checkbox(
        &ph2d_editor_core::widget::Checkbox::new(id, "")
            .fora_do_formulario()
            .visual(store.checkbox_visual(id))
            .value(valor),
        caixa,
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ T56: fora do cartão a linha de dois são as DUAS linhas de hoje — rótulo à esquerda, uma
    /// caixa por linha, nenhum campo de cartão.
    #[test]
    fn outside_a_card_a_pair_paints_the_two_rows_of_today() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        let store = WidgetStore::default();
        let (a, b) = (NodeId(0x0756_0001), NodeId(0x0756_0002));
        let (x, w) = (0.0, 300.0);
        let fim = par_row(
            &mut scene,
            &mut text,
            Theme::default(),
            &mut hit,
            &store,
            x,
            w,
            0.0,
            Some(("Half Size", Eixos::Tamanho)),
            [
                Campo::new("Half Width", a, Some(Unit::Px)),
                Campo::new("Half Height", b, Some(Unit::Px)),
            ],
        );
        let (ra, rb) = (hit.rect_for(a).expect("a"), hit.rect_for(b).expect("b"));
        assert!(
            rb.y > ra.y,
            "fora do cartão as duas caixas não empilham: {ra:?} {rb:?}"
        );
        assert!(ra.x > x + w * 0.25, "o rótulo deixou a esquerda: {ra:?}");
        assert!((fim - 2.0 * ph2d_tokens::row_pitch_px()).abs() < 0.5);
        assert!(store.card_field(a).is_none() && store.card_field(b).is_none());
    }
}
