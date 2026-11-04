//! ⭐ **Vertex, Connector e Brush em cartões** (fatia G5, `docs/CartoesUI/12_VECTOR_PLANO.md`). Mesmos ids
//! e eventos. As mesmas perguntas de antes: Average só com dois nós, cada rota só com os campos que usa.

use crate::cards_fx::{com_unidade, fila};
use crate::cards_tinta::{campo, par_de_botoes, seg, semeia, spec};
use crate::paint_connector::{NUMBER_FIELDS, current_connector, visible_fields};
use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_i18n::tr;
use ph2d_tool_vector::connector;
use ph2d_tool_vector::ids as vids;
use ph2d_tool_vector::{VertexSel, VertexType};

const PASSO_FINO: f64 = 0.01; // LITERAL-PX-OK: passo do arrasto, no domínio do documento
const SETA_DO_PINCEL: f64 = 0.05; // LITERAL-PX-OK: o passo do chip no `populate_texture_pattern`

/// Com o store MUTÁVEL, antes de pintar: as marcas e as sementes que o corpo antigo dava ao pintar.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for id in [crate::ids::VECTOR_VERT_X, crate::ids::VECTOR_VERT_Y] {
        store.mark_card_field(id, spec(PASSO_FINO, PASSO_FINO, None));
    }
    // O conector mostra o valor EFETIVO, por isso não há fábrica: a semente é a de sempre (`seed`).
    for &(id, field) in NUMBER_FIELDS {
        store.mark_card_field(id, spec(field.step, field.step, None));
    }
    for (id, seta, scrub, fabrica) in [
        (
            ids::VECTOR_BRUSH_SCALE_NUM,
            SETA_DO_PINCEL,
            PASSO_FINO,
            None,
        ),
        (
            ids::VECTOR_BRUSH_SPACING_NUM,
            SETA_DO_PINCEL,
            PASSO_FINO,
            None,
        ),
        (ids::VECTOR_BRUSH_ROTATION_NUM, 1.0, 1.0, Some(0.0)),
        (
            ids::VECTOR_BRUSH_OFFSET_NUM,
            SETA_DO_PINCEL,
            PASSO_FINO,
            Some(0.0),
        ),
    ] {
        store.mark_card_field(id, spec(seta, scrub, fabrica));
    }
    if let Some(b) = state::current_brush() {
        for (id, valor) in [
            (ids::VECTOR_BRUSH_SCALE_NUM, b.scale),
            (ids::VECTOR_BRUSH_SPACING_NUM, b.spacing),
            (ids::VECTOR_BRUSH_ROTATION_NUM, b.rotation_deg),
            (ids::VECTOR_BRUSH_OFFSET_NUM, b.offset),
        ] {
            semeia(store, id, valor, false);
        }
    }
}

/// ⭐ **O cartão Vertex**: o tipo, onde está (a mediana, em mundo), e os gestos sobre a seleção de nós.
pub(crate) fn vertex(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const POS: &[NodeId] = &[crate::ids::VECTOR_VERT_X, crate::ids::VECTOR_VERT_Y];
    let Some(sel) = state::current_vertex_type() else {
        return y;
    };
    // Seleção mista: nenhum aceso, e os três vivos. Retipar é a saída do estado misto.
    let aceso = match sel {
        VertexSel::Uniform(t) => Some(t),
        VertexSel::Mixed => None,
    };
    let tipos = [
        (
            vids::VECTOR_VERT_CORNER,
            "panel.vector.vertex.corner",
            VertexType::Corner,
        ),
        (
            vids::VECTOR_VERT_SMOOTH,
            "panel.vector.vertex.smooth",
            VertexType::Smooth,
        ),
        (
            vids::VECTOR_VERT_SYMMETRIC,
            "panel.vector.vertex.symm",
            VertexType::Symmetric,
        ),
    ]
    .map(|(id, chave, t)| seg(id, tr(chave), aceso == Some(t)));
    let mut y = c.row(&[Cell::new(Widget::Choice(&tipos))], y);
    if state::current_vertex_pos().is_some() {
        y = c.row(
            &[
                Cell::new(campo(POS[0], Glyph::Text(tr("card.glyph.x"))))
                    .label(tr("card.insp.position_m")),
                Cell::new(campo(POS[1], Glyph::Text(tr("card.glyph.y")))).key(POS),
            ],
            y,
        );
    }
    let y = par_de_botoes(
        c,
        (
            crate::ids::VECTOR_VERT_SEL_SUBPATH,
            tr("panel.vector.vertex.select_subpath"),
        ),
        (
            crate::ids::VECTOR_VERT_SEL_SAME,
            tr("panel.vector.vertex.select_same"),
        ),
        y,
    );
    let apagar = (
        crate::ids::VECTOR_VERT_DELETE,
        tr("panel.vector.vertex.delete_node"),
    );
    // Um Average de um nó não tem o que mediar: ficaria vivo e mudo.
    if state::current_vertex_count() >= 2 {
        return par_de_botoes(
            c,
            (
                crate::ids::VECTOR_VERT_AVERAGE,
                tr("panel.vector.vertex.average"),
            ),
            apagar,
            y,
        );
    }
    c.full(
        None,
        Widget::Button {
            id: apagar.0,
            text: apagar.1,
            primary: false,
        },
        y,
    )
}

/// ⭐ **O cartão Connector**: a rota (o botão que cicla, como antes) divide a linha com o 1.º número
/// que ELA usa, e os outros seguem aos pares.
pub(crate) fn connector_card(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(snap) = current_connector() else {
        return y;
    };
    let rota = crate::nomes_do_motor::conector(connector::route_label(f64::from(snap.route)));
    let mut celulas = vec![
        Cell::new(Widget::Button {
            id: vids::VECTOR_CONNECTOR_ROUTE,
            text: rota,
            primary: false,
        })
        .label(crate::nomes_do_motor::conector(connector::ROUTE.label)),
    ];
    celulas.extend(visible_fields(snap.route).iter().map(|(id, field)| {
        let nome = crate::nomes_do_motor::conector(field.label);
        Cell::new(campo(*id, Glyph::Text(corpo::letra_da_grandeza(nome)))).label(nome)
    }));
    celulas.chunks(2).fold(y, |y, par| c.row(par, y))
}

/// ⭐ **O cartão Brush**: a arte (o botão diz se escolhe ou troca), os números dois a dois e a caixa.
pub(crate) fn brush(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(b) = state::current_brush() else {
        return y;
    };
    let arte = if b.has_art {
        crate::art_vocabulary::CHANGE.tr()
    } else {
        crate::art_vocabulary::PICK.tr()
    };
    let y = c.full(
        None,
        Widget::Button {
            id: crate::ids::VECTOR_BRUSH_PICK_SHAPE,
            text: arte,
            primary: false,
        },
        y,
    );
    let vezes = tr("card.vector.unit.times");
    let (tamanho, passo) = (
        com_unidade(tr("panel.vector.brush.size"), vezes),
        com_unidade(tr("panel.vector.brush.spacing"), vezes),
    );
    let rotacao = com_unidade(
        tr("panel.vector.brush.rotation"),
        tr("card.vector.unit.degrees"),
    );
    let desvio = com_unidade(
        tr("panel.vector.brush.offset"),
        tr("card.vector.unit.metres"),
    );
    let y = fila(
        c,
        &[
            (
                tamanho.as_str(),
                ids::VECTOR_BRUSH_SCALE_NUM,
                ids::VECTOR_BRUSH_SCALE,
            ),
            (
                passo.as_str(),
                ids::VECTOR_BRUSH_SPACING_NUM,
                ids::VECTOR_BRUSH_SPACING,
            ),
        ],
        y,
    );
    let y = fila(
        c,
        &[
            (
                rotacao.as_str(),
                ids::VECTOR_BRUSH_ROTATION_NUM,
                ids::VECTOR_BRUSH_ROTATION,
            ),
            (
                desvio.as_str(),
                ids::VECTOR_BRUSH_OFFSET_NUM,
                ids::VECTOR_BRUSH_OFFSET,
            ),
        ],
        y,
    );
    c.check(
        crate::ids::VECTOR_BRUSH_FLIP,
        tr("panel.vector.brush.flip"),
        b.flip,
        y,
    )
}
