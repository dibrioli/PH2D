//! ⭐ **T56 — dois campos numa linha do cartão** (ordem do dono 02/11: *«em Physic Body sempre que
//! fizer sentido, coloque 2 campos por linha»*; `docs/CartoesUI/01_decisoes_do_dono.md` T56). Só o
//! cartão muda: fora dele cada campo pinta-se como hoje, um por linha, pela mesma porta.

use super::rows::{num_row_unit, seg_row};
use super::*;
use ph2d_editor_core::panel::cards::corpo::{self, Coluna};
use ph2d_editor_core::widget::Unit;
use ph2d_editor_core::widget::key_mark::{Eixos, Rotulo};

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
