//! ⭐⭐ **O painel Bones em CARTÕES** (`docs/CartoesUI/02_especificacao.md` §6, maquete A/B): os
//! mesmos controlos e os MESMOS ids do corpo de hoje ([`crate::section`]), repartidos por cartões
//! — o despacho (`seam.rs`, `PanelEvent=4`) não muda. Só a pintura.

use crate::section::{
    aceso, bind_e_oferecido, pinta_a_direccao, rotulo_do_numero, segmentos_da_direccao,
    segmentos_do_modo,
};
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget, dropdown};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

#[path = "cards_osso.rs"]
mod osso;

/// As secções-cartão do painel (ordem natural; a do artista grava-se em `sections.txt`).
pub const CARD_TOOL: NodeId = hash_node_id("cards.bones.tool");
pub const CARD_WEIGHT: NodeId = hash_node_id("cards.bones.weight");
pub const CARD_SKIN: NodeId = hash_node_id("cards.bones.skin");
pub const CARD_BONE: NodeId = hash_node_id("cards.bones.bone");
pub const CARD_LIMIT: NodeId = hash_node_id("cards.bones.limit");
pub const CARD_SMART: NodeId = hash_node_id("cards.bones.smart");
pub const CARD_IK: NodeId = hash_node_id("cards.bones.ik");

/// Passo de um número adimensional (uma força, uma curva, uma mistura), no domínio dele.
const DECIMO: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do dado, não medida de design
const CENTESIMO: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do dado, não medida de design

/// `(id, passo das setas, arrasto por px, valor de fábrica)` de cada campo de cartão do painel.
const CAMPOS: &[(NodeId, f64, f64, Option<f64>)] = &[
    (ids::VECTOR_BONE_LENGTH, 1.0, 1.0, None),
    (ids::VECTOR_BONE_SEGMENTS, 1.0, DECIMO, Some(1.0)),
    (ids::VECTOR_BONE_STRENGTH, DECIMO, CENTESIMO, Some(1.0)),
    (ids::VECTOR_BONE_CURVE_IN_Y, CENTESIMO, CENTESIMO, Some(0.0)),
    (
        ids::VECTOR_BONE_CURVE_OUT_Y,
        CENTESIMO,
        CENTESIMO,
        Some(0.0),
    ),
    (ids::VECTOR_BONE_CURVE_IN_X, CENTESIMO, CENTESIMO, Some(0.0)),
    (
        ids::VECTOR_BONE_CURVE_OUT_X,
        CENTESIMO,
        CENTESIMO,
        Some(0.0),
    ),
    (ids::VECTOR_BONE_LIMIT_MIN, 1.0, 1.0, None),
    (ids::VECTOR_BONE_LIMIT_MAX, 1.0, 1.0, None),
    (ids::VECTOR_BONE_IK_MIX, CENTESIMO, CENTESIMO, Some(1.0)),
    (
        ids::VECTOR_BONE_IK_SOFTNESS,
        CENTESIMO,
        CENTESIMO,
        Some(0.0),
    ),
    (ids::VECTOR_BONE_IK_CHAIN, 1.0, DECIMO, None),
    (ids::VECTOR_BONE_IK_OFFSET, 1.0, 1.0, Some(0.0)),
    (ids::VECTOR_BONE_SMART_FROM, 1.0, 1.0, None),
    (ids::VECTOR_BONE_SMART_TO, 1.0, 1.0, None),
    (
        ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_RADIUS,
        1.0,
        1.0,
        None,
    ),
    (
        ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_AMOUNT,
        CENTESIMO,
        CENTESIMO,
        None,
    ),
];

/// Marca os campos de cartão e dá nome aos ícones do cabeçalho (o tooltip) — com o store MUTÁVEL,
/// antes de pintar, pela mesma porta da semeadura dos valores.
pub(crate) fn prepare(store: &mut WidgetStore) {
    dropdown::declare(crate::ids::VECTOR_BONE_SMART_CLIP);
    dropdown::declare(crate::ids::VECTOR_BONE_TIP);
    for &(id, step, scrub, default) in CAMPOS {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step,
                scrub,
                default,
            },
        );
    }
    for (id, chave) in [
        (ids::VECTOR_BONE_REST_APPLY, "panel.vector.bone.rest.apply"),
        (ids::VECTOR_BONE_REST_SET, "panel.vector.bone.rest.set"),
        (ids::VECTOR_BONE_MIRROR, "panel.vector.bone.mirror"),
        (ids::VECTOR_BONE_EXPAND, "panel.vector.bone.expand"),
        (ids::VECTOR_BONE_RELEASE, "panel.vector.bone.release"),
        (ids::VECTOR_BONE_LIMIT_ADD, "panel.vector.bone.limit.add"),
        (
            ids::VECTOR_BONE_LIMIT_REMOVE,
            "panel.vector.bone.limit.remove",
        ),
        (ids::VECTOR_BONE_SMART_ADD, "panel.vector.bone.smart.add"),
        (
            ids::VECTOR_BONE_SMART_REMOVE,
            "panel.vector.bone.smart.remove",
        ),
        (ids::VECTOR_BONE_IK_ADD, "panel.vector.bone.ik.add"),
        (ids::VECTOR_BONE_IK_REMOVE, "panel.vector.bone.ik.remove"),
        (ids::VECTOR_BONE_LOOK_AT, "panel.vector.bone.look_at"),
        (ids::VECTOR_BONE_BEND_IDS[1], "card.bones.bend_ccw"),
        (ids::VECTOR_BONE_BEND_IDS[2], "card.bones.bend_cw"),
        (
            ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_ADD,
            "panel.vector.bone.weight.add",
        ),
        (
            ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_SUB,
            "panel.vector.bone.weight.subtract",
        ),
    ] {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    }
}

pub(super) fn field<'a>(id: NodeId, glyph: Glyph<'a>) -> Widget<'a> {
    Widget::Field { id, glyph }
}

/// ⭐ Os cartões do painel, pelo estado publicado pela shell (os que não valem não aparecem).
pub(crate) fn cards() -> Vec<CardDef<'static>> {
    let armado = state::bone_tool();
    let mut out = vec![cartao_da_ferramenta(armado)];
    if armado == Some(2) {
        out.push(cartao_do_pincel());
    }
    let presa = state::skinned();
    if bind_e_oferecido(armado) || presa.alguma() {
        out.push(cartao_da_pele(armado, presa));
    }
    let Some(spec) = state::current_bone() else {
        return out;
    };
    out.extend([
        osso::cartao_do_osso(spec),
        osso::cartao_do_limite(),
        osso::cartao_smart(),
        osso::cartao_ik(),
    ]);
    out
}

fn cartao_da_ferramenta(armado: Option<usize>) -> CardDef<'static> {
    CardDef {
        section: CARD_TOOL,
        title: tr("card.bones.tool"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let segs = [
                Seg {
                    id: ph2d_tool_bone::ids::VECTOR_BONE_ACT_CREATE,
                    icon: Some(IconId::Bone),
                    text: tr("panel.vector.bone.create"),
                    on: aceso(armado, 0),
                },
                Seg {
                    id: ph2d_tool_bone::ids::VECTOR_BONE_ACT_TRANSFORM,
                    icon: Some(IconId::Move),
                    text: tr("panel.vector.bone.transform"),
                    on: aceso(armado, 1),
                },
                Seg {
                    id: ph2d_tool_bone::ids::VECTOR_BONE_ACT_WEIGHT,
                    icon: Some(IconId::CardBrush),
                    text: tr("panel.vector.bone.weight"),
                    on: aceso(armado, 2),
                },
            ];
            c.full(None, Widget::Choice(&segs), y)
        })),
    }
}

fn cartao_do_pincel() -> CardDef<'static> {
    CardDef {
        section: CARD_WEIGHT,
        title: tr("card.bones.weight_brush"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            let (_, _, lado, modo) = state::bone_weight();
            let modos = segmentos_do_modo(modo).map(|(id, k, on)| Seg {
                id,
                icon: None,
                text: tr(k),
                on,
            });
            let dirs = [IconId::CardPlusw, IconId::CardMinusw];
            let mut i = 0;
            let direccoes = segmentos_da_direccao(lado).map(|(id, _, on)| {
                let s = Seg {
                    id,
                    icon: Some(dirs[i]),
                    text: "",
                    on,
                };
                i += 1;
                s
            });
            let mut linha = vec![Cell::new(Widget::Choice(&modos)).label(tr("card.bones.mode"))];
            if pinta_a_direccao(modo) {
                linha.push(Cell::new(Widget::Icons(&direccoes)).label(tr("card.bones.direction")));
            }
            let y = c.row(&linha, y);
            let numero = if rotulo_do_numero(modo) == "panel.vector.bone.weight.target" {
                tr("card.bones.target")
            } else {
                tr("card.bones.strength")
            };
            c.row(
                &[
                    Cell::new(field(
                        ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_RADIUS,
                        Glyph::Icon(IconId::CardRadius),
                    ))
                    .label(tr("card.bones.radius")),
                    Cell::new(field(
                        ph2d_tool_bone::ids::VECTOR_BONE_WEIGHT_AMOUNT,
                        Glyph::Icon(IconId::CardFlow),
                    ))
                    .label(numero),
                ],
                y,
            )
        })),
    }
}

fn cartao_da_pele(armado: Option<usize>, presa: state::Skinned) -> CardDef<'static> {
    let mut actions = Vec::new();
    if presa.alguma() {
        if presa.vector {
            actions.push(HeadAction {
                id: ids::VECTOR_BONE_EXPAND,
                icon: IconId::Pin,
                on: false,
            });
        }
        actions.push(HeadAction {
            id: ids::VECTOR_BONE_RELEASE,
            icon: IconId::Unlink,
            on: false,
        });
    }
    CardDef {
        section: CARD_SKIN,
        title: tr("card.bones.skin"),
        actions,
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let mut y = y;
            if bind_e_oferecido(armado) {
                y = c.full(
                    None,
                    Widget::Button {
                        id: ids::VECTOR_BONE_BIND,
                        text: tr("panel.vector.bone.bind"),
                        primary: true,
                    },
                    y,
                );
            }
            if state::skinned().alguma() {
                let env = state::skin_law_envelope();
                let leis = [
                    Seg {
                        id: ids::VECTOR_BONE_SKIN_LAW_AUTO,
                        icon: None,
                        text: tr("panel.vector.bone.skin_law.auto"),
                        on: !env,
                    },
                    Seg {
                        id: ids::VECTOR_BONE_SKIN_LAW_ENVELOPE,
                        icon: None,
                        text: tr("panel.vector.bone.skin_law.envelope"),
                        on: env,
                    },
                ];
                y = c.full(Some(tr("card.bones.deform_by")), Widget::Choice(&leis), y);
            }
            y
        })),
    }
}
