//! ⭐ O VOCABULÁRIO do cartão (doc `CartoesUI/08` §1.1) — o padrão que todo painel novo copia,
//! com valores vivos: os números moram no store (registados pelo `populate`, marcados pelo
//! [`prepare`]), as escolhas no [`Choices`] do estado (o paint lê, o evento escreve).

use crate::ids as g;
use crate::state::Choices;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::corpo::{self, Marca};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget, dropdown};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::{self, KeyMark, Rotulo};
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

/// Os ícones sem texto e a dica de cada um (o censo das dicas dos cartões).
const TIPS: [(NodeId, &str); 9] = [
    (g::CAP_BUTT, "panel.widget_gallery.tip.cap_butt"),
    (g::CAP_ROUND, "panel.widget_gallery.tip.cap_round"),
    (g::CAP_SQUARE, "panel.widget_gallery.tip.cap_square"),
    (g::BARE_VISIBLE, "panel.widget_gallery.tip.visible"),
    (g::BARE_LOCK, "panel.widget_gallery.tip.lock"),
    (g::FILL_EYE, "panel.widget_gallery.tip.fill_visible"),
    (g::FILL_REMOVE, "panel.widget_gallery.tip.fill_remove"),
    (g::RESET, "panel.widget_gallery.tip.reset"),
    (g::PHYSICS_REMOVE, "panel.widget_gallery.tip.physics_remove"),
];

/// Marca os campos, dá nome aos ícones e publica os quatro estados da marca — com o store
/// MUTÁVEL, antes de pintar.
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
    for (id, key) in TIPS {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(key));
        }
    }
    store.set_tooltip(g::PHYSICS_ADD, tr("panel.widget_gallery.tip.physics_add"));
    dropdown::declare(g::BLEND);
    // O Ordering nasce DOBRADO (o resumo à direita); depois manda o artista.
    if store.collapsed_choice(g::CARD_ORDERING).is_none() {
        store.set_collapsed(g::CARD_ORDERING, true);
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

fn field(id: NodeId, glyph: Glyph<'_>) -> Widget<'_> {
    Widget::Field { id, glyph }
}

fn seg(id: NodeId, text: &str, on: bool) -> Seg<'_> {
    Seg {
        id,
        icon: None,
        text,
        on,
    }
}

fn icon(id: NodeId, icon: IconId, on: bool) -> Seg<'static> {
    Seg {
        id,
        icon: Some(icon),
        text: "",
        on,
    }
}

/// Fields: posição com marca, rotação só com glifo, tamanho, nome.
fn fields() -> CardDef<'static> {
    CardDef {
        section: g::CARD_FIELDS,
        title: tr("panel.widget_gallery.card.fields"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            const POS: &[NodeId] = &[g::POS_X, g::POS_Y];
            let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
            let mut y = c.row(
                &[
                    Cell::new(field(g::POS_X, Glyph::Text(gx)))
                        .label(tr("panel.widget_gallery.position"))
                        .key(POS),
                    Cell::new(field(g::POS_Y, Glyph::Text(gy))),
                ],
                y,
            );
            y = c.row(
                &[
                    Cell::new(field(g::ROTATION, Glyph::Icon(IconId::CardAngle)))
                        .label(tr("panel.widget_gallery.rotation")),
                ],
                y,
            );
            let (gw, gh) = (
                tr("panel.widget_gallery.glyph.w"),
                tr("panel.widget_gallery.glyph.h"),
            );
            y = c.row(
                &[
                    Cell::new(field(g::SIZE_W, Glyph::Text(gw)))
                        .label(tr("panel.widget_gallery.size")),
                    Cell::new(field(g::SIZE_H, Glyph::Text(gh))),
                ],
                y,
            );
            c.full(
                Some(tr("panel.widget_gallery.name")),
                Widget::Text {
                    id: g::NAME,
                    placeholder: tr("panel.widget_gallery.name_placeholder"),
                },
                y,
            )
        })),
    }
}

/// Choices: segmentado de texto, de ícones, ícones nus, caixa de marcar, lista suspensa.
fn choices(ch: Choices) -> CardDef<'static> {
    CardDef {
        section: g::CARD_CHOICES,
        title: tr("panel.widget_gallery.card.choices"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let levels = [
                seg(
                    g::LEVEL_LOW,
                    tr("panel.widget_gallery.level.low"),
                    ch.level == 0,
                ),
                seg(
                    g::LEVEL_MID,
                    tr("panel.widget_gallery.level.mid"),
                    ch.level == 1,
                ),
                seg(
                    g::LEVEL_HIGH,
                    tr("panel.widget_gallery.level.high"),
                    ch.level == 2,
                ),
            ];
            let mut y = c.full(
                Some(tr("panel.widget_gallery.level")),
                Widget::Choice(&levels),
                y,
            );
            let caps = [
                icon(g::CAP_BUTT, IconId::CardCapButt, ch.cap == 0),
                icon(g::CAP_ROUND, IconId::CardCapRound, ch.cap == 1),
                icon(g::CAP_SQUARE, IconId::CardCapSquare, ch.cap == 2),
            ];
            let bare = [
                icon(g::BARE_VISIBLE, IconId::Eye, ch.visible),
                icon(g::BARE_LOCK, IconId::Lock, ch.locked),
            ];
            y = c.row(
                &[
                    Cell::new(Widget::Icons(&caps)).label(tr("panel.widget_gallery.cap")),
                    Cell::new(Widget::Bare(&bare)).label(tr("panel.widget_gallery.layer")),
                ],
                y,
            );
            y = corpo::check(
                c.scene,
                c.text_system,
                c.store,
                c.hit_index,
                c.ink,
                c.x,
                c.w,
                y,
                Marca {
                    id: g::SNAP,
                    label: Rotulo::vista(TextKey::new("panel.widget_gallery.snap")),
                    on: ch.snap,
                },
            );
            let blends = [
                seg(
                    g::BLEND_NORMAL,
                    tr("panel.widget_gallery.blend.normal"),
                    ch.blend == 0,
                ),
                seg(
                    g::BLEND_MULTIPLY,
                    tr("panel.widget_gallery.blend.multiply"),
                    ch.blend == 1,
                ),
                seg(
                    g::BLEND_SCREEN,
                    tr("panel.widget_gallery.blend.screen"),
                    ch.blend == 2,
                ),
            ];
            let atual = blends.iter().find(|s| s.on).map_or("", |s| s.text);
            c.full(
                Some(tr("panel.widget_gallery.blend")),
                Widget::Dropdown {
                    id: g::BLEND,
                    text: atual,
                    options: &blends,
                },
                y,
            )
        })),
    }
}

/// Color: a linha do Fill à Figma — amostra + código, opacidade, olho e menos.
fn color(ch: Choices) -> CardDef<'static> {
    CardDef {
        section: g::CARD_COLOR,
        title: tr("panel.widget_gallery.card.color"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let rgba = c.ink.color(ColorToken::Accent).to_rgba8().to_u8_array();
            let code = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
            let olhos = [
                icon(g::FILL_EYE, IconId::Eye, ch.fill_visible),
                icon(g::FILL_REMOVE, IconId::Minus, false),
            ];
            c.row(
                &[
                    Cell::new(Widget::Swatch {
                        id: g::FILL_SWATCH,
                        rgba: Some(rgba),
                        text: &code,
                        edit: None,
                        detach: None,
                    })
                    .label(tr("panel.widget_gallery.fill")),
                    Cell::new(field(g::OPACITY, Glyph::Icon(IconId::CardOpacity)))
                        .label(tr("panel.widget_gallery.opacity")),
                    Cell::new(Widget::Bare(&olhos)),
                ],
                y,
            )
        })),
    }
}

/// Actions: botões, emblema e o ↺ no cabeçalho.
fn actions() -> CardDef<'static> {
    CardDef {
        section: g::CARD_ACTIONS,
        title: tr("panel.widget_gallery.card.actions"),
        actions: vec![HeadAction {
            id: g::RESET,
            icon: IconId::Reset,
            on: false,
        }],
        add: None,
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            c.row(
                &[
                    Cell::new(Widget::Button {
                        id: g::PRIMARY,
                        text: tr("panel.widget_gallery.primary"),
                        primary: true,
                    }),
                    Cell::new(Widget::Button {
                        id: g::SECONDARY,
                        text: tr("panel.widget_gallery.secondary"),
                        primary: false,
                    }),
                    Cell::new(Widget::Badge(IconId::Grid)),
                ],
                y,
            )
        })),
    }
}

/// Animation marks: os quatro estados da marca D17 lado a lado, cada um com o seu rótulo.
fn marks() -> CardDef<'static> {
    CardDef {
        section: g::CARD_MARKS,
        title: tr("panel.widget_gallery.card.marks"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            const FREE: &[NodeId] = &[g::MARK_FREE];
            const ANIM: &[NodeId] = &[g::MARK_ANIMATED];
            const KEY: &[NodeId] = &[g::MARK_KEY];
            const CHANGED: &[NodeId] = &[g::MARK_CHANGED];
            let gv = tr("card.glyph.x");
            let y = c.row(
                &[
                    Cell::new(field(g::MARK_FREE, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.free"))
                        .key(FREE),
                    Cell::new(field(g::MARK_ANIMATED, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.animated"))
                        .key(ANIM),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(field(g::MARK_KEY, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.key"))
                        .key(KEY),
                    Cell::new(field(g::MARK_CHANGED, Glyph::Text(gv)))
                        .label(tr("panel.widget_gallery.mark.changed"))
                        .key(CHANGED),
                ],
                y,
            )
        })),
    }
}

/// Ordering: o cartão que mostra o resumo quando dobrado («Z 3»).
fn ordering(store: &WidgetStore) -> CardDef<'static> {
    let z = store.number_value(g::Z_ORDER).unwrap_or_default();
    CardDef {
        section: g::CARD_ORDERING,
        title: tr("panel.widget_gallery.card.ordering"),
        actions: Vec::new(),
        add: None,
        summary: Some(
            tr("panel.widget_gallery.ordering_summary").replace("{z}", &format!("{z:.0}")),
        ),
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            c.row(
                &[
                    Cell::new(field(g::Z_ORDER, Glyph::Text(tr("card.glyph.z"))))
                        .label(tr("panel.widget_gallery.z_order")),
                ],
                y,
            )
        })),
    }
}

/// Physics: a secção AUSENTE — o «+» cria-a e ela passa a ter corpo (e o «−» que a tira).
fn physics(ch: Choices) -> CardDef<'static> {
    CardDef {
        section: g::CARD_PHYSICS,
        title: tr("panel.widget_gallery.card.physics"),
        actions: if ch.physics {
            vec![HeadAction {
                id: g::PHYSICS_REMOVE,
                icon: IconId::Minus,
                on: false,
            }]
        } else {
            Vec::new()
        },
        add: (!ch.physics).then_some(g::PHYSICS_ADD),
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            c.row(
                &[Cell::new(field(g::MASS, Glyph::Icon(IconId::CardWeight)))
                    .label(tr("panel.widget_gallery.mass"))],
                y,
            )
        })),
    }
}

/// Os sete cartões do vocabulário, pela ordem do doc 08 §1.1.
pub(crate) fn vocabulary(ch: Choices, store: &WidgetStore) -> Vec<CardDef<'static>> {
    vec![
        fields(),
        choices(ch),
        color(ch),
        actions(),
        marks(),
        ordering(store),
        physics(ch),
    ]
}
