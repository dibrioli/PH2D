//! ⭐ **Casos reais: animação, pincel, ossos, restrição** (doc `CartoesUI/08` §1.3). As decisões do dono:
//! UM cartão Animation com regras «quando → animação» (25/10); Paint type Water·Oil, um só Dilution, o
//! Volume independente e o Paper press no painel do pincel (28/10, 31/10, 01/11).

use crate::controlos::{card, dica, dropdown, head, marca, num, segs};
use crate::ids as g;
use crate::state::{Choices, Flag, Pick, RULES};
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_i18n::tr;

/// Uma regra: `[quando ▾] [toca ▾] −`; a primeira da lista leva os rótulos.
fn rule_row(c: &mut CardCtx<'_>, ch: &Choices, i: usize, rot: bool, y: f32) -> f32 {
    let r = g::RULES[i];
    let quando = segs(r.whens, ch, c.store);
    let toca = segs(r.plays, ch, c.store);
    let menos = segs([r.remove], ch, c.store);
    let mut a = Cell::new(dropdown(r.when, &quando));
    let mut b = Cell::new(dropdown(r.play, &toca));
    if rot {
        a = a.label(tr("panel.widget_gallery.real.when"));
        b = b.label(tr("panel.widget_gallery.real.play"));
    }
    c.row(&[a, b, Cell::new(Widget::Bare(&menos))], y)
}

pub(crate) fn animation(ch: Choices, store: &WidgetStore) -> CardDef<'static> {
    let regras = ch.pick(Pick::AnimWho) == 2;
    let actions = if regras && !RULES.iter().all(|f| ch.on(*f)) {
        vec![head(g::RULE_ADD, &ch, store)]
    } else {
        Vec::new()
    };
    card(
        g::CARD_ANIMATION,
        tr("panel.widget_gallery.card.animation"),
        actions,
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let clips = segs(
                [
                    g::ANIM_CLIP_0,
                    g::ANIM_CLIP_1,
                    g::ANIM_CLIP_2,
                    g::ANIM_CLIP_3,
                ],
                &ch,
                c.store,
            );
            let toca = segs([g::ANIM_PLAY], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(dropdown(g::ANIM_CLIP, &clips))
                        .label(tr("panel.widget_gallery.real.clip")),
                    Cell::new(Widget::Bare(&toca)),
                ],
                y,
            );
            let y = c.row(
                &[num!(
                    g::ANIM_FPS,
                    IconId::Play,
                    tr("panel.inspector.anim_card.fps")
                )],
                y,
            );
            let quem = segs([g::ANIM_ME, g::ANIM_SCRIPT, g::ANIM_RULES], &ch, c.store);
            let mut y = c.row(
                &[Cell::new(Widget::Choice(&quem))
                    .label(tr("panel.inspector.anim_card.who_picks"))],
                y,
            );
            match ch.pick(Pick::AnimWho) {
                0 => dica(c, tr("panel.widget_gallery.real.who_me"), y),
                1 => dica(c, tr("panel.widget_gallery.real.who_script"), y),
                _ => {
                    let mut primeira = true;
                    for (i, f) in RULES.iter().enumerate() {
                        if ch.on(*f) {
                            y = rule_row(c, &ch, i, primeira, y);
                            primeira = false;
                        }
                    }
                    if primeira {
                        return dica(c, tr("panel.widget_gallery.real.no_rules"), y);
                    }
                    y
                }
            }
        }),
    )
}

pub(crate) fn brush(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_BRUSH,
        tr("panel.widget_gallery.card.brush"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let presets = segs(
                [
                    g::BR_PRESET_0,
                    g::BR_PRESET_1,
                    g::BR_PRESET_2,
                    g::BR_PRESET_3,
                ],
                &ch,
                c.store,
            );
            let y = c.row(
                &[Cell::new(dropdown(g::BR_PRESET, &presets))
                    .label(tr("panel.widget_gallery.real.preset"))],
                y,
            );
            let y = c.row(
                &[num!(
                    g::BR_SIZE,
                    IconId::CardBrush,
                    tr("panel.painter_layers.wetpaint.vol_size")
                )],
                y,
            );
            let tipos = segs([g::BR_WATER, g::BR_OIL], &ch, c.store);
            let y = c.row(
                &[Cell::new(Widget::Choice(&tipos))
                    .label(tr("panel.painter_layers.wetpaint.paint_type"))
                    .key(&[g::BR_WATER, g::BR_OIL])],
                y,
            );
            let y = c.row(
                &[
                    num!(
                        g::BR_DILUTION,
                        IconId::CardOpacity,
                        tr("panel.painter_layers.wetpaint.dilution")
                    ),
                    num!(
                        g::BR_VOLUME,
                        IconId::CardWeight,
                        tr("panel.painter_layers.wetpaint.volume_um")
                    ),
                ],
                y,
            );
            let proprio = if ch.pick(Pick::PaintType) == 0 {
                num!(
                    g::BR_REWET,
                    IconId::CardFlow,
                    tr("panel.painter_wet_cartao.rewet")
                )
            } else {
                num!(
                    g::BR_BODY,
                    IconId::CardWeight,
                    tr("panel.painter_layers.wetpaint.body")
                )
            };
            c.row(
                &[
                    proprio,
                    num!(
                        g::BR_PRESS,
                        IconId::CardSoft,
                        tr("panel.widget_gallery.real.paper_press")
                    ),
                ],
                y,
            )
        }),
    )
}

pub(crate) fn bones(ch: Choices) -> CardDef<'static> {
    card(
        g::CARD_BONES,
        tr("panel.widget_gallery.card.bones"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let dobra = segs([g::BO_BEND_CCW, g::BO_BEND_CW], &ch, c.store);
            let y = c.row(
                &[
                    num!(g::BO_LENGTH, IconId::CardLen, tr("card.bones.length")),
                    Cell::new(Widget::Icons(&dobra))
                        .label(tr("card.bones.bend"))
                        .key(&[g::BO_BEND_CCW, g::BO_BEND_CW]),
                ],
                y,
            );
            let mut y = marca(
                c,
                g::BO_LIMIT,
                "card.bones.limit",
                ch.on(Flag::BoneLimit),
                y,
            );
            if ch.on(Flag::BoneLimit) {
                y = c.row(
                    &[
                        num!(g::BO_MIN, IconId::CardAngle, tr("card.bones.min")),
                        num!(g::BO_MAX, IconId::CardAngle, tr("card.bones.max")),
                    ],
                    y,
                );
            }
            let y = marca(
                c,
                g::BO_SMART,
                "card.bones.smart",
                ch.on(Flag::BoneSmart),
                y,
            );
            let y = marca(c, g::BO_IK, "card.bones.ik", ch.on(Flag::BoneIk), y);
            if !ch.on(Flag::BoneIk) {
                return y;
            }
            c.row(
                &[
                    num!(g::BO_MIX, IconId::CardMix, tr("card.bones.mix")),
                    num!(g::BO_CHAIN, IconId::CardChain, tr("card.bones.chain")),
                ],
                y,
            )
        }),
    )
}

/// Um cartão de pilha: `[tipo ▾] 👁 −`, depois o que aquele tipo pede. Sem linha, o cartão fica
/// ausente com «+».
pub(crate) fn constraint(ch: Choices) -> CardDef<'static> {
    let mut d = card(
        g::CARD_CONSTRAINT,
        tr("panel.widget_gallery.card.constraint"),
        Vec::new(),
        Box::new(move |c: &mut CardCtx<'_>, y| {
            let tipos = segs([g::CS_KIND_0, g::CS_KIND_1, g::CS_KIND_2], &ch, c.store);
            let olho = segs([g::CS_EYE, g::CS_REMOVE], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(dropdown(g::CS_KIND, &tipos)),
                    Cell::new(Widget::Bare(&olho)),
                ],
                y,
            );
            let conta_gotas = segs([g::CS_PICK], &ch, c.store);
            let y = c.row(
                &[
                    Cell::new(Widget::Text {
                        id: g::CS_TARGET,
                        placeholder: tr("panel.widget_gallery.real.target_hint"),
                    })
                    .label(tr("component.field.follow_fields.0")),
                    Cell::new(Widget::Bare(&conta_gotas)),
                ],
                y,
            );
            let espacos = segs([g::CS_SPACE_WORLD, g::CS_SPACE_LOCAL], &ch, c.store);
            c.row(
                &[
                    num!(
                        g::CS_INFLUENCE,
                        IconId::CardMix,
                        tr("panel.widget_gallery.real.influence")
                    ),
                    Cell::new(dropdown(g::CS_SPACE, &espacos))
                        .label(tr("panel.inspector.constraint.space"))
                        .key(&[g::CS_SPACE]),
                ],
                y,
            )
        }),
    );
    if !ch.on(Flag::ConsRow) {
        d.add = Some(g::CS_ADD);
    }
    d
}
