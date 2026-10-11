//! ⭐⭐ **O cartão lateral leva o que nenhuma alça move** (F10, pedido do dono de 11/10: *«se algum
//! parâmetro não pode ser colocado no gizmo então precisamos criar um modal que aparece na
//! lateral»*). Genérico sobre o descritor: para TODA forma do catálogo, o cartão é a conta
//! `fields_without_handle`, com os MESMOS ids da fila ② — e só existe com as alças à vista.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::ToolBars;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_panel_vector::bars;
use ph2d_tool_vector::ids::{vector_shape_choice_id, vector_shape_field_id};
use ph2d_tool_vector::shape_handles::fields_without_handle;
use ph2d_tool_vector::shapes::{FieldUnit, SHAPES};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};
use ph2d_vec_scene::ShapeKind;

fn barras(mode: DrawMode, live: Option<ShapeKind>) -> ToolBars {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(usize::from(live.is_some()));
    ph2d_panel_vector::set_current_shape_focus(live);
    bars::tool_bars(crate::host_aberto().store(), None, Vec::new())
}

fn ids(v: &[ToolRailEntry]) -> Vec<NodeId> {
    v.iter().filter_map(ToolRailEntry::node_id).collect()
}

/// O id que o campo `i` de `kind` pinta (o slot numérico ou o gémeo da escolha).
fn id_do_campo(kind: ShapeKind, i: usize) -> NodeId {
    let d = SHAPES.iter().find(|d| d.kind == kind).expect("no catálogo");
    match d.fields[i].unit {
        FieldUnit::Choice(_) => vector_shape_choice_id(i),
        _ => vector_shape_field_id(i),
    }
}

#[test]
fn the_card_is_every_field_no_handle_moves_with_the_row_ids() {
    let mut com_cartao = 0;
    for d in SHAPES {
        let b = barras(DrawMode::Shape, Some(d.kind));
        let esperado: Vec<NodeId> = fields_without_handle(d.kind)
            .map(|i| id_do_campo(d.kind, i))
            .collect();
        assert_eq!(
            ids(&b.card),
            esperado,
            "{:?}: o cartão não é a conta",
            d.kind
        );
        let fila: Vec<NodeId> = ids(&b.row_lead)
            .into_iter()
            .chain(ids(&b.row_tail))
            .collect();
        for id in ids(&b.card) {
            assert!(
                fila.contains(&id),
                "{:?}: {id:?} no cartão e fora da fila ②",
                d.kind
            );
        }
        com_cartao += usize::from(!b.card.is_empty());
    }
    assert!(
        com_cartao >= 3,
        "controlo: só {com_cartao} formas têm cartão"
    );
}

#[test]
fn the_card_and_the_handles_are_born_of_the_same_predicate() {
    let kind = SHAPES
        .iter()
        .map(|d| d.kind)
        .find(|&k| fields_without_handle(k).next().is_some())
        .expect("controlo: alguma forma tem campo sem alça");
    let mut vistos = 0;
    for &mode in DrawMode::ALL {
        for live in [None, Some(kind)] {
            let b = barras(mode, live);
            let shown = bars::shape_handles_shown(mode, live);
            assert_eq!(!b.card.is_empty(), shown, "{mode:?} live={live:?}");
            vistos += usize::from(shown);
        }
    }
    assert_eq!(vistos, 1, "só o modo Forma com forma viva mostra as alças");
}
