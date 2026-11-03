//! **As linhas de tinta dos cartões Fill e Stroke** (02 §2.7) — a pilha e o «Stroke settings». Corte
//! de `cards.rs` por responsabilidade: ali *que cartões há*, aqui *como uma linha de tinta se compõe*.

use crate::cards::{DD_STROKE_ALIGN, hex_id};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, estilos};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tool_vector::appearance::{self, PaintRow};
use ph2d_tool_vector::ids as vids;

pub(crate) fn field<'a>(id: NodeId, glyph: Glyph<'a>) -> Widget<'a> {
    Widget::Field { id, glyph }
}

pub(crate) fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

/// Uma linha da pilha (02 §2.7): `[amostra│código] [opacidade] 👁 –`. A opacidade é editável na
/// camada ABERTA (a do painel Vector); nas outras, um clique abre-a.
fn layer_row(c: &mut CardCtx<'_>, i: usize, row: &PaintRow, rotulos: bool, y: f32) -> f32 {
    let codigo = hex(row.color);
    let pct = format!("{:.0}", f64::from(row.opacity) * 100.0);
    let aberta = appearance::open_layer_index() == Some(i);
    let opacidade = if aberta {
        field(
            vids::VECTOR_PAINT_OPACITY_NUM,
            Glyph::Icon(IconId::CardOpacity),
        )
    } else {
        Widget::FieldFace {
            id: vids::vector_paint_row_id(i),
            glyph: Glyph::Icon(IconId::CardOpacity),
            text: &pct,
        }
    };
    let olho = [
        Seg {
            id: vids::vector_paint_eye_id(i),
            icon: Some(if row.enabled {
                IconId::Eye
            } else {
                IconId::EyeClosed
            }),
            text: "",
            on: false,
        },
        Seg {
            id: vids::vector_paint_del_id(i),
            icon: Some(IconId::Minus),
            text: "",
            on: false,
        },
    ];
    // D30: ligada a um estilo, a camada mostra o NOME dele (como o Figma) e a corrente partida.
    let nome = estilos::ligado(i).map(|e| e.name);
    let mut cor = Cell::new(Widget::Swatch {
        id: vids::vector_paint_swatch_id(i),
        rgba: Some(row.color),
        text: nome.as_deref().unwrap_or(&codigo),
        edit: nome.is_none().then(|| hex_id(i)),
        detach: nome.is_some().then(|| estilos::solta_de(i)),
    });
    let mut op = Cell::new(opacidade);
    // As marcas são as do painel Vector: a cor desta tinta; a opacidade só na camada ABERTA.
    let k_cor = [vids::vector_paint_swatch_id(i)];
    const K_OP: &[NodeId] = &[vids::VECTOR_PAINT_OPACITY_NUM];
    if rotulos {
        cor = cor.label(tr("card.insp.color")).key(&k_cor);
        op = op.label(tr("card.insp.opacity"));
        if aberta {
            op = op.key(K_OP);
        }
    }
    c.row(&[cor, op, Cell::new(Widget::Bare(&olho))], y)
}

pub(crate) fn pilha(c: &mut CardCtx<'_>, fill: bool, y: f32) -> f32 {
    let Some(a) = appearance::current_appearance() else {
        return y;
    };
    let mut y = y;
    let mut primeira = true;
    for (i, row) in a.layers.iter().enumerate() {
        if row.is_fill == fill {
            y = layer_row(c, i, row, primeira, y);
            primeira = false;
        }
    }
    y
}

pub(crate) fn stroke_props(c: &mut CardCtx<'_>, y: f32) -> f32 {
    use ph2d_tool_vector::{StrokeCap, StrokeJoin};
    use ph2d_vec_scene::StrokeAlign;
    let snap = appearance::current_vector_style().unwrap_or_default();
    let alinhar = [
        (
            vids::VECTOR_ALIGN_CENTRE,
            "card.insp.align_center",
            StrokeAlign::Centre,
        ),
        (
            vids::VECTOR_ALIGN_INNER,
            "card.insp.align_inside",
            StrokeAlign::Inner,
        ),
        (
            vids::VECTOR_ALIGN_OUTER,
            "card.insp.align_outside",
            StrokeAlign::Outer,
        ),
    ]
    .map(|(id, k, a)| Seg {
        id,
        icon: None,
        text: tr(k),
        on: snap.align == a,
    });
    let atual = alinhar.iter().find(|s| s.on).map_or("", |s| s.text);
    const W: &[NodeId] = &[vids::VECTOR_WIDTH_NUM];
    const AL: &[NodeId] = &[
        vids::VECTOR_ALIGN_CENTRE,
        vids::VECTOR_ALIGN_INNER,
        vids::VECTOR_ALIGN_OUTER,
    ];
    const CAP: &[NodeId] = &[
        vids::VECTOR_CAP_BUTT,
        vids::VECTOR_CAP_ROUND,
        vids::VECTOR_CAP_SQUARE,
    ];
    const JOIN: &[NodeId] = &[
        vids::VECTOR_JOIN_MITER,
        vids::VECTOR_JOIN_ROUND,
        vids::VECTOR_JOIN_BEVEL,
    ];
    let y = c.row(
        &[
            Cell::new(Widget::Dropdown {
                id: DD_STROKE_ALIGN,
                text: atual,
                options: &alinhar,
            })
            .label(tr("card.insp.stroke_position"))
            .key(AL),
            Cell::new(field(
                vids::VECTOR_WIDTH_NUM,
                Glyph::Icon(IconId::CardWeight),
            ))
            .label(tr("card.insp.weight"))
            .key(W),
        ],
        y,
    );
    let caps = [
        (vids::VECTOR_CAP_BUTT, IconId::CardCapButt, StrokeCap::Butt),
        (
            vids::VECTOR_CAP_ROUND,
            IconId::CardCapRound,
            StrokeCap::Round,
        ),
        (
            vids::VECTOR_CAP_SQUARE,
            IconId::CardCapSquare,
            StrokeCap::Square,
        ),
    ]
    .map(|(id, icon, v)| Seg {
        id,
        icon: Some(icon),
        text: "",
        on: snap.cap == v,
    });
    let joins = [
        (
            vids::VECTOR_JOIN_MITER,
            IconId::CardJoinMiter,
            StrokeJoin::Miter,
        ),
        (
            vids::VECTOR_JOIN_ROUND,
            IconId::CardJoinRound,
            StrokeJoin::Round,
        ),
        (
            vids::VECTOR_JOIN_BEVEL,
            IconId::CardJoinBevel,
            StrokeJoin::Bevel,
        ),
    ]
    .map(|(id, icon, v)| Seg {
        id,
        icon: Some(icon),
        text: "",
        on: snap.join == v,
    });
    c.row(
        &[
            Cell::new(Widget::Choice(&caps))
                .label(tr("card.insp.cap"))
                .key(CAP),
            Cell::new(Widget::Choice(&joins))
                .label(tr("card.insp.join"))
                .key(JOIN),
        ],
        y,
    )
}
