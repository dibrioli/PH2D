//! Os cartões que mostram UM osso: Bone, Angle Limit, Smart Bone e IK (irmão de [`super`]). As linhas
//! de lista seguem o padrão do cartão, rótulo por cima e caixa na largura dele (foto do dono 03/11).

use super::{CARD_BONE, CARD_IK, CARD_LIMIT, CARD_SMART, field};
use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;

/// As opções de uma lista publicada pela shell, cortadas no POOL de ids (o chrome não cunha ids).
fn opcoes<'a>(pool: &[NodeId], nomes: &'a [String], acesa: impl Fn(usize) -> bool) -> Vec<Seg<'a>> {
    pool.iter()
        .zip(nomes)
        .enumerate()
        .map(|(i, (&id, nome))| Seg {
            id,
            icon: None,
            text: nome.as_str(),
            on: acesa(i),
        })
        .collect()
}

/// O objecto que o Smart Bone anima: o botão diz o nome, convida, ou (azul) diz o que espera.
fn linha_do_objecto(c: &mut CardCtx<'_>, sb: &state::SmartBoneView, y: f32) -> f32 {
    let texto = if sb.picking {
        tr("panel.vector.bone.smart.picking")
    } else if sb.target.is_empty() {
        tr("panel.vector.bone.smart.pick")
    } else {
        sb.target.as_str()
    };
    c.full(
        Some(tr("panel.vector.bone.smart.object")),
        Widget::Button {
            id: ids::VECTOR_BONE_SMART_PICK,
            text: texto,
            primary: sb.picking,
        },
        y,
    )
}

/// A acção do Smart Bone: a caixa diz a ligada (ou o traço) e abre as que o documento tem.
fn linha_da_accao(c: &mut CardCtx<'_>, sb: &state::SmartBoneView, y: f32) -> f32 {
    let nomes = state::bone_actions();
    let lista = opcoes(&ids::VECTOR_BONE_SMART_CLIP_IDS, &nomes, |i| {
        nomes[i] == sb.clip
    });
    let texto = if sb.clip.is_empty() {
        tr("panel.vector.bone.smart.none")
    } else {
        sb.clip.as_str()
    };
    c.full(
        Some(tr("panel.vector.bone.smart.action")),
        Widget::Dropdown {
            id: crate::ids::VECTOR_BONE_SMART_CLIP,
            text: texto,
            options: &lista,
        },
        y,
    )
}

/// Quem manda na ponta da curva. Os filhos que não couberam no pool aparecem no rótulo.
fn linha_da_ponta(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(v) = state::bone_tip() else {
        return y;
    };
    let Some(texto) = v.rotulos.get(v.ligado) else {
        return y;
    };
    let etiqueta = if v.escondidos == 0 {
        tr("panel.vector.bone.tip").to_string()
    } else {
        format!("{} +{}", tr("panel.vector.bone.tip"), v.escondidos)
    };
    let lista = opcoes(&ids::VECTOR_BONE_TIP_IDS, &v.rotulos, |i| i == v.ligado);
    c.full(
        Some(&etiqueta),
        Widget::Dropdown {
            id: crate::ids::VECTOR_BONE_TIP,
            text: texto,
            options: &lista,
        },
        y,
    )
}

pub(super) fn cartao_do_osso(osso: ph2d_skeleton::bend::BoneSpec) -> CardDef<'static> {
    CardDef {
        section: CARD_BONE,
        title: tr("card.bones.bone"),
        actions: vec![
            HeadAction {
                id: ids::VECTOR_BONE_REST_APPLY,
                icon: IconId::CardRest,
                on: false,
            },
            HeadAction {
                id: ids::VECTOR_BONE_REST_SET,
                icon: IconId::Save,
                on: false,
            },
            HeadAction {
                id: ids::VECTOR_BONE_MIRROR,
                icon: IconId::CardMirror,
                on: false,
            },
        ],
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            const LEN: &[NodeId] = &[ids::VECTOR_BONE_LENGTH];
            const SEG: &[NodeId] = &[ids::VECTOR_BONE_SEGMENTS];
            let mut y = c.row(
                &[
                    Cell::new(field(ids::VECTOR_BONE_LENGTH, Glyph::Icon(IconId::CardLen)))
                        .label(tr("card.bones.length"))
                        .key(LEN),
                    Cell::new(field(
                        ids::VECTOR_BONE_SEGMENTS,
                        Glyph::Icon(IconId::CardSeg),
                    ))
                    .label(tr("card.bones.segments"))
                    .key(SEG),
                ],
                y,
            );
            if state::envelope_manda() {
                const STR: &[NodeId] = &[ids::VECTOR_BONE_STRENGTH];
                y = c.row(
                    &[Cell::new(field(
                        ids::VECTOR_BONE_STRENGTH,
                        Glyph::Icon(IconId::CardFlow),
                    ))
                    .label(tr("card.bones.reach"))
                    .key(STR)],
                    y,
                );
            }
            let curvo = !osso.is_rigid() || ph2d_skeleton::bend::segments_of(osso.segments) > 1;
            if curvo {
                let auto = state::current_bone_handles() == Some(1);
                let modos = [
                    Seg {
                        id: ids::VECTOR_BONE_HANDLES_AUTHORED,
                        icon: None,
                        text: tr("panel.vector.bone.handles.authored"),
                        on: !auto,
                    },
                    Seg {
                        id: ids::VECTOR_BONE_HANDLES_AUTO,
                        icon: None,
                        text: tr("panel.vector.bone.handles.auto"),
                        on: auto,
                    },
                ];
                y = c.full(Some(tr("card.bones.handles")), Widget::Choice(&modos), y);
                if !auto {
                    const CIN: &[NodeId] = &[ids::VECTOR_BONE_CURVE_IN_Y];
                    const COUT: &[NodeId] = &[ids::VECTOR_BONE_CURVE_OUT_Y];
                    const AIN: &[NodeId] = &[ids::VECTOR_BONE_CURVE_IN_X];
                    const AOUT: &[NodeId] = &[ids::VECTOR_BONE_CURVE_OUT_X];
                    let (gi, go) = (tr("card.glyph.in"), tr("card.glyph.out"));
                    y = c.row(
                        &[
                            Cell::new(field(ids::VECTOR_BONE_CURVE_IN_Y, Glyph::Text(gi)))
                                .label(tr("card.bones.curve_in"))
                                .key(CIN),
                            Cell::new(field(ids::VECTOR_BONE_CURVE_OUT_Y, Glyph::Text(go)))
                                .label(tr("card.bones.curve_out"))
                                .key(COUT),
                        ],
                        y,
                    );
                    y = c.row(
                        &[
                            Cell::new(field(ids::VECTOR_BONE_CURVE_IN_X, Glyph::Text(gi)))
                                .label(tr("card.bones.along_in"))
                                .key(AIN),
                            Cell::new(field(ids::VECTOR_BONE_CURVE_OUT_X, Glyph::Text(go)))
                                .label(tr("card.bones.along_out"))
                                .key(AOUT),
                        ],
                        y,
                    );
                }
            }
            if state::bone_tip().is_some() {
                y = linha_da_ponta(c, y);
            }
            y
        })),
    }
}

pub(super) fn cartao_do_limite() -> CardDef<'static> {
    let limite = state::current_bone_limit().is_some();
    CardDef {
        section: CARD_LIMIT,
        title: tr("card.bones.limit"),
        actions: if limite {
            vec![HeadAction {
                id: ids::VECTOR_BONE_LIMIT_REMOVE,
                icon: IconId::Minus,
                on: false,
            }]
        } else {
            Vec::new()
        },
        add: (!limite).then_some(ids::VECTOR_BONE_LIMIT_ADD),
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            const MIN: &[NodeId] = &[ids::VECTOR_BONE_LIMIT_MIN];
            const MAX: &[NodeId] = &[ids::VECTOR_BONE_LIMIT_MAX];
            c.row(
                &[
                    Cell::new(field(
                        ids::VECTOR_BONE_LIMIT_MIN,
                        Glyph::Icon(IconId::CardAngle),
                    ))
                    .label(tr("card.bones.min"))
                    .key(MIN),
                    Cell::new(field(
                        ids::VECTOR_BONE_LIMIT_MAX,
                        Glyph::Icon(IconId::CardAngle),
                    ))
                    .label(tr("card.bones.max"))
                    .key(MAX),
                ],
                y,
            )
        })),
    }
}

pub(super) fn cartao_smart() -> CardDef<'static> {
    let smart = state::current_bone_smart().is_some();
    CardDef {
        section: CARD_SMART,
        title: tr("card.bones.smart"),
        actions: if smart {
            vec![HeadAction {
                id: ids::VECTOR_BONE_SMART_REMOVE,
                icon: IconId::Minus,
                on: false,
            }]
        } else {
            Vec::new()
        },
        add: (!smart).then_some(ids::VECTOR_BONE_SMART_ADD),
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            let Some(sb) = state::current_bone_smart() else {
                return y;
            };
            let y = linha_do_objecto(c, &sb, y);
            let y = linha_da_accao(c, &sb, y);
            c.row(
                &[
                    Cell::new(field(
                        ids::VECTOR_BONE_SMART_FROM,
                        Glyph::Icon(IconId::CardAngle),
                    ))
                    .label(tr("card.bones.from")),
                    Cell::new(field(
                        ids::VECTOR_BONE_SMART_TO,
                        Glyph::Icon(IconId::CardAngle),
                    ))
                    .label(tr("card.bones.to")),
                ],
                y,
            )
        })),
    }
}

pub(super) fn cartao_ik() -> CardDef<'static> {
    let ik = state::current_bone_ik();
    CardDef {
        section: CARD_IK,
        title: tr("card.bones.ik"),
        actions: if ik.is_some() {
            vec![HeadAction {
                id: ids::VECTOR_BONE_IK_REMOVE,
                icon: IconId::Minus,
                on: false,
            }]
        } else {
            vec![HeadAction {
                id: ids::VECTOR_BONE_LOOK_AT,
                icon: IconId::CardLook,
                on: false,
            }]
        },
        add: ik.is_none().then_some(ids::VECTOR_BONE_IK_ADD),
        summary: None,
        body: Some(Box::new(|c: &mut CardCtx<'_>, y| {
            let Some((_, _, _, lado)) = state::current_bone_ik() else {
                return y;
            };
            const MIX: &[NodeId] = &[ids::VECTOR_BONE_IK_MIX];
            const CHAIN: &[NodeId] = &[ids::VECTOR_BONE_IK_CHAIN];
            const SOFT: &[NodeId] = &[ids::VECTOR_BONE_IK_SOFTNESS];
            const AIM: &[NodeId] = &[ids::VECTOR_BONE_IK_OFFSET];
            let mix = Cell::new(field(ids::VECTOR_BONE_IK_MIX, Glyph::Icon(IconId::CardMix)))
                .label(tr("card.bones.mix"))
                .key(MIX);
            if state::current_bone_aim().is_some() {
                return c.row(
                    &[
                        mix,
                        Cell::new(field(
                            ids::VECTOR_BONE_IK_OFFSET,
                            Glyph::Icon(IconId::CardAngle),
                        ))
                        .label(tr("card.bones.aim"))
                        .key(AIM),
                    ],
                    y,
                );
            }
            let y = c.row(
                &[
                    mix,
                    Cell::new(field(
                        ids::VECTOR_BONE_IK_CHAIN,
                        Glyph::Icon(IconId::CardChain),
                    ))
                    .label(tr("card.bones.chain"))
                    .key(CHAIN),
                ],
                y,
            );
            let y = c.row(
                &[Cell::new(field(
                    ids::VECTOR_BONE_IK_SOFTNESS,
                    Glyph::Icon(IconId::CardSoft),
                ))
                .label(tr("card.bones.softness"))
                .key(SOFT)],
                y,
            );
            let auto = crate::section::rotulo_do_auto(state::current_bone_ik_auto_side());
            let textos = [auto.as_str(), "", "", tr("panel.vector.bone.ik.bend.mixed")];
            let icones = [None, Some(IconId::RotateCcw), Some(IconId::RotateCw), None];
            let lados: Vec<Seg<'_>> = (0..ids::VECTOR_BONE_BEND_IDS.len())
                .map(|i| Seg {
                    id: ids::VECTOR_BONE_BEND_IDS[i],
                    icon: icones[i],
                    text: textos[i],
                    on: i == lado,
                })
                .collect();
            c.full(Some(tr("card.bones.bend")), Widget::Choice(&lados), y)
        })),
    }
}
