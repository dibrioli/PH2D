//! ⭐ **O inspector de ferramentas fictícias à Figma** (dono 03/11, doc `CartoesUI/08` §1.1) — o
//! padrão que todo painel novo copia, com valores vivos: os números moram no store (registados pelo
//! `populate`, marcados pelo [`prepare`]), as escolhas no [`Choices`] do estado, e os controlos
//! vêm da tabela [`crate::controlos`]. Aqui ficam a ordem dos cartões e os que sobreviveram ao
//! vocabulário antigo: Actions, marcas, Ordering e Physics.

use crate::controlos::{Kind, card, controls, field, head, num};
use crate::ids as g;
use crate::state::{Choices, Flag, WidgetGalleryState};
use crate::{
    cards_cromo, cards_layout, cards_objeto, cards_pintura, cards_reais_anim, cards_reais_media,
    cards_reais_mundo, cards_rive, cards_texto, cards_transform,
};
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget, dropdown};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::{self, KeyMark};
use ph2d_i18n::tr;

/// Marca os campos, dá nome aos ícones, declara as listas e publica os quatro estados da marca —
/// com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for (id, factory, step, scrub) in g::FIELDS {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step,
                scrub,
                default: Some(factory),
            },
        );
    }
    for c in controls() {
        if let Some(key) = c.tip
            && store.tooltip_for(c.id).is_none()
        {
            store.set_tooltip(c.id, tr(key));
        }
        if c.kind == Kind::Box {
            dropdown::declare(c.id);
        }
    }
    // O Ordering nasce DOBRADO (o resumo à direita); depois manda o artista.
    if store.collapsed_choice(g::GAL_CARD_ORDERING).is_none() {
        store.set_collapsed(g::GAL_CARD_ORDERING, true);
    }
    for (id, mark) in [
        (g::MARK_FREE, KeyMark::Livre),
        (g::MARK_ANIMATED, KeyMark::Animado),
        (g::MARK_KEY, KeyMark::Chave),
        (g::MARK_CHANGED, KeyMark::Mudado),
    ] {
        key_mark::add_key_mark(id, mark);
    }
}

/// Actions: Apply grava os valores de partida, Cancel e o ↺ voltam a eles; o rótulo sobre os
/// botões e o emblema dizem se há mudanças por aplicar. ⚠️ O rótulo mora na célula do Apply: sobre
/// o emblema (uma célula de `FIELD_H`) ele comia letras.
fn actions(ch: Choices, dirty: bool, store: &WidgetStore) -> CardDef<'static> {
    let (badge, status) = if dirty {
        (
            IconId::CircleDashed,
            tr("panel.widget_gallery.status.edited"),
        )
    } else {
        (IconId::Check, tr("panel.widget_gallery.status.saved"))
    };
    card(
        g::GAL_CARD_ACTIONS,
        tr("panel.widget_gallery.card.actions"),
        vec![head(g::RESET, &ch, store)],
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = c.row(
                &[
                    Cell::new(Widget::Button {
                        id: g::PRIMARY,
                        text: tr("panel.widget_gallery.primary"),
                        primary: true,
                    })
                    .label(status),
                    Cell::new(Widget::Button {
                        id: g::SECONDARY,
                        text: tr("panel.widget_gallery.secondary"),
                        primary: false,
                    }),
                ],
                y,
            );
            c.full(None, Widget::Badge(badge), y)
        }),
    )
}

/// Animation marks: os quatro estados da marca D17 lado a lado, cada um com o seu rótulo.
fn marks() -> CardDef<'static> {
    card(
        g::GAL_CARD_MARKS,
        tr("panel.widget_gallery.card.marks"),
        Vec::new(),
        Box::new(|c: &mut CardCtx<'_>, y| {
            let gv = tr("card.glyph.x");
            let y = c.row(
                &[
                    Cell::new(field(g::MARK_FREE, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.free"))
                        .key(&[g::MARK_FREE]),
                    Cell::new(field(g::MARK_ANIMATED, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.animated"))
                        .key(&[g::MARK_ANIMATED]),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(field(g::MARK_KEY, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.key"))
                        .key(&[g::MARK_KEY]),
                    Cell::new(field(g::MARK_CHANGED, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.changed"))
                        .key(&[g::MARK_CHANGED]),
                ],
                y,
            )
        }),
    )
}

/// Ordering: o cartão que mostra o resumo quando dobrado («Z 3»).
fn ordering(store: &WidgetStore) -> CardDef<'static> {
    let z = store.number_value(g::Z_ORDER).unwrap_or_default();
    let mut d = card(
        g::GAL_CARD_ORDERING,
        tr("panel.widget_gallery.card.ordering"),
        Vec::new(),
        Box::new(|c: &mut CardCtx<'_>, y| {
            c.row(
                &[
                    Cell::new(field(g::Z_ORDER, Glyph::Text(tr("card.glyph.z"))))
                        .label(tr("panel.widget_gallery.z_order"))
                        .key(&[g::Z_ORDER]),
                ],
                y,
            )
        }),
    );
    d.summary =
        Some(tr("panel.widget_gallery.ordering_summary").replace("{z}", &format!("{z:.0}")));
    d
}

/// Physics: a secção AUSENTE — o «+» cria-a e ela passa a ter corpo (e o «−» que a tira).
fn physics(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let tem = ch.on(Flag::Physics);
    let mut d = card(
        g::GAL_CARD_PHYSICS,
        tr("panel.widget_gallery.card.physics"),
        if tem {
            vec![head(g::PHYSICS_REMOVE, &ch, store)]
        } else {
            Vec::new()
        },
        Box::new(|c: &mut CardCtx<'_>, y| {
            c.row(&[num!(g::MASS, tr("panel.widget_gallery.mass"))], y)
        }),
    );
    d.add = (!tem).then_some(g::PHYSICS_ADD);
    d
}

/// Os cartões das ferramentas, pela ordem do doc 08 §1.1.
pub(crate) fn vocabulary(st: &WidgetGalleryState, store: &WidgetStore) -> Vec<CardDef<'static>> {
    let ch = st.choices;
    let dirty = crate::valores::current(ch, store) != st.applied;
    vec![
        cards_objeto::identity(ch),
        cards_objeto::align(ch),
        cards_objeto::boolean(ch),
        cards_transform::transform(ch, store),
        cards_layout::layout(ch),
        cards_layout::corners(ch),
        cards_layout::constraints(ch),
        cards_pintura::appearance(ch, store),
        cards_pintura::fill(ch, st.estilos.clone(), store),
        cards_pintura::stroke(ch, st.estilos.clone(), store),
        cards_pintura::effects(ch, store),
        cards_texto::text(ch),
        cards_texto::export(ch, st.exported),
        actions(ch, dirty, store),
        marks(),
        ordering(store),
        physics(ch, store),
        // Os casos reais (doc 08 §1.3): depois das ferramentas, antes dos de sempre.
        cards_reais_mundo::image(ch),
        cards_reais_mundo::rigid_body(ch),
        cards_reais_mundo::collider(ch),
        cards_reais_mundo::health(ch),
        cards_reais_mundo::player(),
        cards_reais_mundo::camera(ch),
        cards_reais_anim::animation(ch, store),
        cards_reais_anim::brush(ch),
        cards_reais_anim::bones(ch),
        cards_reais_anim::constraint(ch),
        cards_reais_media::light(ch),
        cards_reais_media::audio(ch, store),
        cards_reais_media::pixel(ch),
        cards_reais_media::script(ch),
        // As amostras do Rive (doc 08 §1.4).
        cards_rive::interpolation(ch),
        cards_rive::inputs(st, store),
        cards_rive::trim(ch),
        cards_rive::solo(ch),
        cards_rive::gradient(ch, store),
        // As peças do cromo: a barra de ferramentas e o que flutua.
        cards_cromo::tool_bar(ch),
        cards_cromo::floating_list(ch),
    ]
}
