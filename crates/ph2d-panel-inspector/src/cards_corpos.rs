//! **Que cartões o Inspector mostra e o corpo de cada um** — corte de `cards.rs` por responsabilidade
//! (ids, semente e commits ficam lá; a linha de tinta em `cards_pilha`).

use crate::cards::{
    CARD_APPEARANCE, CARD_FILL, CARD_MATERIAL, CARD_OBJECT, CARD_ORDERING, CARD_STROKE,
    CARD_TRANSFORM, DD_BLEND, SCALE_LOCK, graus, scale_locked,
};
use crate::cards_pilha::{field, pilha, stroke_props};
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget, estilos};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tool_vector::appearance::{self, Appearance, BasePaint};
use ph2d_tool_vector::ids as vids;
use ph2d_vec_scene::StackItem;

/// ⭐ Os cartões do Inspector, pelo estado publicado (os que não valem não aparecem).
pub(crate) fn cards(store: &WidgetStore) -> Vec<CardDef<'static>> {
    let mut out = Vec::new();
    if crate::state::current_inspector_transform().is_none() {
        return out;
    }
    // O olho do objecto publicado: o de um preso não é a caixa da seleção.
    let visivel = crate::state::current_inspector_visibility().is_none_or(|v| v.mixed || v.visible);
    out.push(objeto(store, visivel));
    out.extend(crate::cards_objeto::do_topo());
    out.push(transform());
    if let Some(a) = appearance::current_appearance() {
        out.push(aparencia(&a, visivel));
        out.extend(tintas(&a));
    }
    out.extend(ordering());
    out.extend(material());
    out
}

fn objeto(store: &WidgetStore, visivel: bool) -> CardDef<'static> {
    let vector = appearance::current_appearance();
    // O cadeado é o da linha da Hierarquia (o MESMO id; sem linha, não há cadeado).
    let cadeado = crate::state::current_inspector_transform()
        .and_then(|t| store.hierarchy_row_of_entity(t.entity_bits));
    let imagem = crate::state::current_inspector_sprite().is_some();
    CardDef {
        section: CARD_OBJECT,
        title: "",
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let tipo = if vector.is_some() {
                IconId::CardPath
            } else if imagem {
                IconId::Image
            } else {
                IconId::Layers
            };
            let mut olho = vec![Seg {
                id: ids::INSP_VISIBILITY_CHECK,
                icon: Some(if visivel {
                    IconId::Eye
                } else {
                    IconId::EyeClosed
                }),
                text: "",
                on: false,
            }];
            if let Some((linha, trancado)) = cadeado {
                olho.push(Seg {
                    id: core_ids::hier_lock_companion(linha),
                    icon: Some(if trancado {
                        IconId::LockKeyhole
                    } else {
                        IconId::LockKeyholeOpen
                    }),
                    text: "",
                    on: trancado,
                });
            }
            let y = c.row(
                &[
                    Cell::new(Widget::Badge(tipo)),
                    Cell::new(Widget::Text {
                        id: core_ids::INSP_ENTITY_NAME,
                        placeholder: tr("card.insp.name"),
                    }),
                    Cell::new(Widget::Bare(&olho)),
                ],
                y,
            );
            crate::cards_objeto::frase_da_selecao(c, y)
        })),
    }
}

fn transform() -> CardDef<'static> {
    // Rodar e inverter são verbos da forma vetorial: em outro objecto seriam um clique sem efeito.
    let vetor = appearance::current_appearance().is_some();
    CardDef {
        section: CARD_TRANSFORM,
        title: tr("card.insp.transform"),
        actions: vec![HeadAction {
            id: ids::INSP_TRANSFORM_RESET,
            icon: IconId::Reset,
            on: false,
        }],
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            const POS: &[NodeId] = &[ids::INSP_TRANSFORM_POS_X, ids::INSP_TRANSFORM_POS_Y];
            const ROT: &[NodeId] = &[ids::INSP_TRANSFORM_ROT];
            const SCALE: &[NodeId] = &[ids::INSP_TRANSFORM_SCALE_X, ids::INSP_TRANSFORM_SCALE_Y];
            const SKEW: &[NodeId] = &[ids::INSP_TRANSFORM_SKEW_X, ids::INSP_TRANSFORM_SKEW_Y];
            let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
            let (pos, rot, skew) = (
                match crate::state::current_display_unit() {
                    DisplayUnit::Meters => tr("card.insp.position_m"),
                    DisplayUnit::Pixels => tr("card.insp.position_px"),
                },
                if graus() {
                    tr("card.insp.rotation_deg")
                } else {
                    tr("card.insp.rotation_rad")
                },
                if graus() {
                    tr("card.insp.skew_deg")
                } else {
                    tr("card.insp.skew_rad")
                },
            );
            // O par X/Y partilha o rótulo do grupo e UMA marca (02 §3).
            let y = c.row(
                &[
                    Cell::new(field(ids::INSP_TRANSFORM_POS_X, Glyph::Text(gx))).label(pos),
                    Cell::new(field(ids::INSP_TRANSFORM_POS_Y, Glyph::Text(gy))).key(POS),
                ],
                y,
            );
            let giros = [
                Seg {
                    id: vids::VECTOR_ARRANGE_ROTATE_CW,
                    icon: Some(IconId::RotateCw),
                    text: "",
                    on: false,
                },
                Seg {
                    id: vids::VECTOR_ARRANGE_FLIP_H,
                    icon: Some(IconId::FlipH),
                    text: "",
                    on: false,
                },
                Seg {
                    id: vids::VECTOR_ARRANGE_FLIP_V,
                    icon: Some(IconId::FlipV),
                    text: "",
                    on: false,
                },
            ];
            let rotacao = Cell::new(field(
                ids::INSP_TRANSFORM_ROT,
                Glyph::Icon(IconId::CardAngle),
            ))
            .label(rot)
            .key(ROT);
            let y = if vetor {
                c.row(&[rotacao, Cell::new(Widget::Bare(&giros))], y)
            } else {
                c.row(&[rotacao], y)
            };
            let corrente = [Seg {
                id: SCALE_LOCK,
                icon: Some(IconId::Link),
                text: "",
                on: scale_locked(),
            }];
            let y = c.row(
                &[
                    Cell::new(field(ids::INSP_TRANSFORM_SCALE_X, Glyph::Text(gx)))
                        .label(tr("card.insp.scale")),
                    Cell::new(field(ids::INSP_TRANSFORM_SCALE_Y, Glyph::Text(gy))).key(SCALE),
                    Cell::new(Widget::Bare(&corrente)),
                ],
                y,
            );
            c.row(
                &[
                    Cell::new(field(ids::INSP_TRANSFORM_SKEW_X, Glyph::Text(gx))).label(skew),
                    Cell::new(field(ids::INSP_TRANSFORM_SKEW_Y, Glyph::Text(gy))).key(SKEW),
                ],
                y,
            )
        })),
    }
}

fn aparencia(a: &Appearance, visivel: bool) -> CardDef<'static> {
    let modos: Vec<ph2d_vec_scene::BlendMode> = ph2d_vec_render::blend::offered().collect();
    let opcoes: Vec<Seg<'static>> = modos
        .iter()
        .enumerate()
        .map(|(i, m)| Seg {
            id: vids::vector_obj_blend_option_id(i),
            icon: None,
            text: tr(m.label_key()),
            on: *m == a.blend,
        })
        .collect();
    let atual = if a.mixed_blend {
        tr("panel.vector.mixed")
    } else {
        tr(a.blend.label_key())
    };
    CardDef {
        section: CARD_APPEARANCE,
        title: tr("card.insp.appearance"),
        // O olho do objeto, também aqui (maquete C; Figma: o olho da secção Layer).
        actions: vec![HeadAction {
            id: ids::INSP_VISIBILITY_CHECK,
            icon: if visivel {
                IconId::Eye
            } else {
                IconId::EyeClosed
            },
            on: false,
        }],
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            const OP: &[NodeId] = &[vids::VECTOR_OBJ_OPACITY_NUM];
            const BL: &[NodeId] = &[vids::VECTOR_OBJ_BLEND];
            c.row(
                &[
                    Cell::new(field(
                        vids::VECTOR_OBJ_OPACITY_NUM,
                        Glyph::Icon(IconId::CardOpacity),
                    ))
                    .label(tr("card.insp.opacity"))
                    .key(OP),
                    Cell::new(Widget::Dropdown {
                        id: DD_BLEND,
                        text: atual,
                        options: &opcoes,
                    })
                    .label(tr("card.insp.blend"))
                    .key(BL),
                ],
                y,
            )
        })),
    }
}

/// A linha de cima de um cartão de tinta: a primeira que `cards_pilha::pilha` pinta. Uma base ausente
/// não tem linha, e com estruturas diferentes na seleção só as bases as têm.
fn linha_de_cima(a: &Appearance, fill: bool, base: Option<BasePaint>) -> Option<usize> {
    let tem_base = base != Some(BasePaint::Absent);
    a.stack().into_iter().rev().find_map(|it| match it {
        StackItem::BaseFill if fill && tem_base => Some(ph2d_vec_scene::LINHA_BASE_FILL),
        StackItem::BaseStroke if !fill && tem_base => Some(ph2d_vec_scene::LINHA_BASE_STROKE),
        StackItem::Entry(i)
            if !a.layers_differ && a.layers.get(i).is_some_and(|r| r.is_fill == fill) =>
        {
            Some(i)
        }
        _ => None,
    })
}

/// Os cartões Fill e Stroke.
fn tintas(a: &Appearance) -> Vec<CardDef<'static>> {
    let mut out = Vec::with_capacity(2);
    for (section, title, add, present, fill) in [
        (
            CARD_FILL,
            "card.insp.fill",
            vids::VECTOR_PAINT_ADD_FILL,
            vids::VECTOR_FILL_PRESENT,
            true,
        ),
        (
            CARD_STROKE,
            "card.insp.stroke",
            vids::VECTOR_PAINT_ADD_STROKE,
            vids::VECTOR_STROKE_PRESENT,
            false,
        ),
    ] {
        let base = crate::cards_pilha::base_de(fill);
        let tem = base != Some(BasePaint::Absent) || a.layers.iter().any(|r| r.is_fill == fill);
        // O «+»: a tinta de base, se a forma não a tem; senão uma camada nova (enquanto houver lugar).
        let add = if base == Some(BasePaint::Absent) {
            Some(present)
        } else {
            (a.layers.len() < ph2d_vec_scene::MAX_PAINT_LAYERS).then_some(add)
        };
        // D28: o ⠿ abre os estilos de cor para a camada aberta deste tipo (senão a linha de cima).
        let camada = appearance::open_layer_index()
            .filter(|i| a.layers.get(*i).is_some_and(|r| r.is_fill == fill))
            .or_else(|| linha_de_cima(a, fill, base));
        let mut actions = Vec::new();
        if let Some(i) = camada {
            actions.push(HeadAction {
                id: estilos::declare(section, i),
                icon: IconId::CardStyles,
                on: estilos::aberta_em(section),
            });
        }
        if let Some(id) = add.filter(|_| tem) {
            actions.push(HeadAction {
                id,
                icon: IconId::Add,
                on: false,
            });
        }
        out.push(CardDef {
            section,
            title: tr(title),
            actions,
            add: add.filter(|_| !tem),
            summary: None,
            body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                let y = pilha(c, fill, y);
                if fill { y } else { stroke_props(c, y) }
            })),
        });
    }
    out
}

fn ordering() -> Option<CardDef<'static>> {
    let o = crate::state::current_inspector_ordering()?;
    let resumo = Some(format!("{} {}", tr("card.glyph.z"), o.z_index.unwrap_or(0)));
    Some(CardDef {
        section: CARD_ORDERING,
        title: tr("card.insp.ordering"),
        actions: Vec::new(),
        add: None,
        summary: resumo,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let y = c.full(
                Some(tr("card.insp.z_index")),
                field(ids::INSP_ORDER_Z_INDEX, Glyph::Text(tr("card.glyph.z"))),
                y,
            );
            crate::cards_objeto::resto_da_ordem(c, y, &o)
        })),
    })
}

fn material() -> Option<CardDef<'static>> {
    let b = crate::state::current_inspector_blend()?;
    let on = usize::from(b.blend_tag);
    let mixed = b.mixed.blend;
    Some(CardDef {
        section: CARD_MATERIAL,
        title: tr("card.insp.material"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            let segs: Vec<Seg<'_>> = ids::INSP_SAMPLE_BLEND
                .iter()
                .zip(crate::sections::BLEND_LABELS)
                .enumerate()
                .map(|(i, (&id, k))| Seg {
                    id,
                    icon: None,
                    text: k.tr(),
                    on: !mixed && i == on,
                })
                .collect();
            c.full(Some(tr("card.insp.mode")), Widget::Choice(&segs), y)
        })),
    })
}
