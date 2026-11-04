//! **As linhas de tinta dos cartões Fill e Stroke** (02 §2.7) — a pilha e o «Stroke settings». Corte
//! de `cards.rs` por responsabilidade: ali *que cartões há*, aqui *como uma linha de tinta se compõe*.

use crate::cards::{DD_STROKE_ALIGN, HEX_FILL_DE_BASE, HEX_STROKE_DE_BASE, amostra_de, hex_id};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, estilos};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tool_vector::appearance::{self, BasePaint, PaintRow};
use ph2d_tool_vector::ids as vids;
use ph2d_vec_scene::StackItem;

pub(crate) fn field<'a>(id: NodeId, glyph: Glyph<'a>) -> Widget<'a> {
    Widget::Field { id, glyph }
}

pub(crate) fn hex(c: [u8; 4]) -> String {
    format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

/// O que muda entre a linha de uma camada e a de uma tinta de base.
struct Linha<'a> {
    amostra: NodeId,
    /// `None` = misto: as selecionadas discordam.
    cor: Option<[u8; 4]>,
    texto: &'a str,
    /// A caixa do código, onde o código se digita (nem num misto nem num degradê).
    edita: Option<NodeId>,
    solta: Option<NodeId>,
    opacidade: NodeId,
    visivel: bool,
    olho: NodeId,
    tira: NodeId,
}

/// Uma linha da pilha (02 §2.7): `[amostra│código] [opacidade] 👁 –`.
fn compoe(c: &mut CardCtx<'_>, l: &Linha<'_>, rotulos: bool, y: f32) -> f32 {
    let botoes = [
        Seg {
            id: l.olho,
            icon: Some(if l.visivel {
                IconId::Eye
            } else {
                IconId::EyeClosed
            }),
            text: "",
            on: false,
        },
        Seg {
            id: l.tira,
            icon: Some(IconId::Minus),
            text: "",
            on: false,
        },
    ];
    let (k_cor, k_op) = ([l.amostra], [l.opacidade]);
    let mut cor = Cell::new(Widget::Swatch {
        id: l.amostra,
        rgba: l.cor,
        text: l.texto,
        edit: l.edita,
        detach: l.solta,
    });
    let mut op = Cell::new(field(l.opacidade, Glyph::Icon(IconId::CardOpacity)));
    if rotulos {
        cor = cor.label(tr("card.insp.color")).key(&k_cor);
        op = op.label(tr("card.insp.opacity")).key(&k_op);
    }
    c.paint_line(cor, op, Cell::new(Widget::Bare(&botoes)), y)
}

fn layer_row(c: &mut CardCtx<'_>, i: usize, row: &PaintRow, rotulos: bool, y: f32) -> f32 {
    let codigo = hex(row.color);
    // D30: ligada a um estilo, a camada mostra o NOME dele (como o Figma) e a corrente partida.
    let nome = estilos::ligado(i).map(|e| e.name);
    let misto = row.mixed.color;
    let texto = match &nome {
        Some(n) => n.as_str(),
        None if misto => tr("panel.vector.mixed"),
        None if row.ramp.is_some() => tr("card.insp.gradient"),
        None => &codigo,
    };
    let linha = Linha {
        amostra: vids::vector_paint_swatch_id(i),
        cor: (!misto).then_some(row.color),
        texto,
        edita: (nome.is_none() && !misto && row.ramp.is_none()).then(|| hex_id(i)),
        solta: nome.is_some().then(|| estilos::solta_de(i)),
        opacidade: vids::vector_paint_row_opacity_id(i),
        visivel: row.enabled,
        olho: vids::vector_paint_eye_id(i),
        tira: vids::vector_paint_del_id(i),
    };
    compoe(c, &linha, rotulos, y)
}

/// ⭐ **A linha da tinta de BASE** (F44b): a cor que uma forma nasce com. Nada, se ela não a tem.
fn base_row(c: &mut CardCtx<'_>, fill: bool, rotulos: bool, y: f32) -> f32 {
    use ph2d_tool_vector::StyleFields as F;
    let estado = base_de(fill);
    if estado == Some(BasePaint::Absent) {
        return y;
    }
    let snap = appearance::current_vector_style().unwrap_or_default();
    let cor = if fill { snap.fill } else { snap.stroke };
    let misto = !snap.agrees(if fill { F::FILL_RGB } else { F::STROKE_RGB });
    let degrade = fill && appearance::base_fill_ramp().is_some();
    let codigo = hex(cor);
    let i = if fill {
        HEX_FILL_DE_BASE
    } else {
        HEX_STROKE_DE_BASE
    };
    // D30: ligada a um estilo, a base mostra o NOME dele e a corrente partida, como a camada.
    let nome = estilos::ligado(i).map(|e| e.name);
    let texto = match &nome {
        Some(n) => n.as_str(),
        None if misto => tr("panel.vector.mixed"),
        None if degrade => tr("card.insp.gradient"),
        None => &codigo,
    };
    let linha = Linha {
        amostra: amostra_de(i),
        cor: (!misto).then_some(cor),
        texto,
        edita: (nome.is_none() && !misto && !degrade).then(|| hex_id(i)),
        solta: nome.is_some().then(|| estilos::solta_de(i)),
        opacidade: if fill {
            vids::VECTOR_FILL_OPACITY_NUM
        } else {
            vids::VECTOR_STROKE_OPACITY_NUM
        },
        visivel: estado != Some(BasePaint::Hidden),
        olho: if fill {
            vids::VECTOR_FILL_EYE
        } else {
            vids::VECTOR_STROKE_EYE
        },
        tira: if fill {
            vids::VECTOR_FILL_PRESENT
        } else {
            vids::VECTOR_STROKE_PRESENT
        },
    };
    compoe(c, &linha, rotulos, y)
}

/// O estado da tinta de base (`None` = sem resposta: a seleção múltipla, em que a linha fica).
pub(crate) fn base_de(fill: bool) -> Option<BasePaint> {
    appearance::base_paints().map(|[f, s]| if fill { f } else { s })
}

/// As linhas de um cartão, com o TOPO em cima (a inversão da pilha vive aqui e no painel Vector: o
/// documento vai do chão ao topo). Com estruturas diferentes na seleção, só as bases.
pub(crate) fn pilha(c: &mut CardCtx<'_>, fill: bool, y: f32) -> f32 {
    let Some(a) = appearance::current_appearance() else {
        return y;
    };
    let mut y = y;
    let mut rotulos = true;
    for it in a.stack().into_iter().rev() {
        let antes = y;
        match it {
            StackItem::BaseFill if fill => y = base_row(c, true, rotulos, y),
            StackItem::BaseStroke if !fill => y = base_row(c, false, rotulos, y),
            StackItem::Entry(i) if !a.layers_differ => {
                if let Some(row) = a.layers.get(i).filter(|r| r.is_fill == fill) {
                    y = layer_row(c, i, row, rotulos, y);
                }
            }
            _ => {}
        }
        rotulos &= y == antes;
    }
    y
}

/// Só o peso: um traço que é só camada (a posição, as pontas e as quinas dela moram no ⚙ do painel
/// Vector; estes controlos do estilo mexem no traço de BASE).
fn peso_da_camada(c: &mut CardCtx<'_>, i: usize, y: f32) -> f32 {
    let id = vids::vector_paint_row_width_id(i);
    let k = [id];
    c.row(
        &[Cell::new(field(id, Glyph::Icon(IconId::CardWeight)))
            .label(tr("card.insp.weight"))
            .key(&k)],
        y,
    )
}

pub(crate) fn stroke_props(c: &mut CardCtx<'_>, y: f32) -> f32 {
    if base_de(false) == Some(BasePaint::Absent) {
        let camada =
            appearance::current_appearance().and_then(|a| a.layers.iter().position(|r| !r.is_fill));
        return camada.map_or(y, |i| peso_da_camada(c, i, y));
    }
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
