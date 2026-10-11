//! O quadro dos campos de forma ([`super::frame_fields`]) por COMPORTAMENTO — eram dois gates de
//! TEXTO da shell (`the_armed_shape_latch_is_wired`, `the_shape_fields_are_seeded_by_the_pair`),
//! que liam o `render_frame` porque a decisão vivia lá dentro, atrás de `gfx`. Mudada para aqui (F9),
//! ela corre sem janela, e cada lei passa a ser dita pelo que FAZ.

use super::tests::live_shape;
use super::*;
use crate::state::VecState;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_vector::VectorTool;

const PX: f64 = 0.01;

fn tools() -> ToolRegistry {
    let mut t = ToolRegistry::new();
    t.register(Box::new(VectorTool::default()));
    t
}

fn tool(t: &mut ToolRegistry) -> &mut VectorTool {
    t.tool_by_id_mut(&ToolId::new("vector"))
        .and_then(|x| x.as_any_mut().downcast_mut::<VectorTool>())
        .expect("a ferramenta de vetor")
}

/// O store REGISTADO pelo `populate` do painel (`set_number_value` só alcança um widget que existe).
fn store() -> WidgetStore {
    let mut store = WidgetStore::default();
    <ph2d_panel_vector::VectorPanel as ph2d_editor_core::panel::Panel>::populate(&mut store);
    store
}

fn field(store: &WidgetStore, i: usize) -> Option<f64> {
    store.number_value(ph2d_tool_vector::ids::vector_shape_field_id(i))
}

/// ⭐ **O latch:** o clique no catálogo ACENDE-o (a forma armada manda nos campos mesmo com uma
/// forma viva seleccionada); a seleção que MUDA apaga-o; e, no mesmo quadro, o DESARME corre antes
/// do ARME — o clique que o artista acabou de dar não se perde.
#[test]
fn the_latch_arms_on_the_catalog_click_and_the_selection_change_disarms_it() {
    let (sim, _scene, map, star) =
        live_shape(ShapeKind::Star, ShapeKind::Star.defaults(), [0.0, 0.0]);
    let mut vec = VecState::default();
    vec.entities = map;
    vec.pen.select_many(&[star]);
    vec.draw_config.mode = DrawMode::Shape;
    let (mut t, mut store) = (tools(), store());
    // Sem clique: a forma viva manda (o ciclo Live Shape).
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert!(!vec.shape_armed);
    assert_eq!(field(&store, 0), Some(ShapeKind::Star.defaults()[0]));
    // O clique no Polígono do catálogo: armado, e os campos são os do Polígono.
    tool(&mut t).set_shape(ShapeKind::Polygon);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert!(vec.shape_armed, "o clique no catálogo não acendeu o latch");
    assert_eq!(vec.shape_last_focus, Some((None, ShapeKind::Polygon)));
    // A seleção muda (nada seleccionado): o latch apaga-se.
    vec.pen.select_many(&[]);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert!(!vec.shape_armed, "a seleção mudou e o latch ficou aceso");
    // Os dois no MESMO quadro (outra seleção + um clique): ganha o clique.
    vec.pen.select_many(&[star]);
    tool(&mut t).set_shape(ShapeKind::Polygon);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert!(
        vec.shape_armed,
        "o desarme correu depois do arme e apagou o clique"
    );
}

/// ⚠️ **A memo é o PAR `(alvo, tipo)`:** sem nada selecionado, trocar Star → Polygon no catálogo
/// re-semeia (o `None == None` de 2026-08-01), e os valores vêm do que a TOOL guarda para o tipo.
#[test]
fn the_seed_memo_is_the_pair_and_without_a_target_the_catalog_seeds() {
    let mut vec = VecState::default();
    vec.draw_config.mode = DrawMode::Shape;
    let (mut t, mut store) = (tools(), store());
    let sim = SimWorld::default();
    tool(&mut t).set_shape(ShapeKind::Star);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert_eq!(field(&store, 0), Some(ShapeKind::Star.defaults()[0]));
    tool(&mut t).set_shape(ShapeKind::Polygon);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert_eq!(vec.shape_last_focus, Some((None, ShapeKind::Polygon)));
    let want = tool(&mut t).shape_values(ShapeKind::Polygon)[0];
    assert_eq!(
        field(&store, 0),
        Some(want),
        "o campo ficou com o número da forma anterior"
    );
}

/// ⭐ **F9: a forma mudou POR FORA** (uma alça no canvas) — os números da fila ② acompanham no
/// quadro seguinte, sem trocar de alvo. E um quadro sem mudança NÃO re-semeia (o campo que está a
/// ser arrastado não é atropelado).
#[test]
fn a_change_from_a_canvas_handle_reseeds_the_fields() {
    let (mut sim, mut scene, map, star) =
        live_shape(ShapeKind::Star, ShapeKind::Star.defaults(), [0.0, 0.0]);
    let mut vec = VecState::default();
    vec.entities = map;
    vec.pen.select_many(&[star]);
    let (mut t, mut store) = (tools(), store());
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert_eq!(field(&store, 1), Some(ShapeKind::Star.defaults()[1]));
    // Uma escrita que não veio dos campos (a alça escreve pela MESMA porta).
    let sel = [star];
    assert!(edit_selected_shape(
        &mut sim,
        &mut scene,
        &vec.entities,
        &sel,
        DrawMode::Select,
        false,
        |_, v| {
            v[1] = 0.8;
            true
        }
    ));
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert_eq!(
        field(&store, 1),
        Some(0.8),
        "a fila ② ficou com o número velho"
    );
    // Sem mudança: a semente não corre (um campo a meio de um arrasto escreve no store e não é
    // reposto).
    let id = ph2d_tool_vector::ids::vector_shape_field_id(1);
    store.set_number_value(id, 0.81);
    frame_fields(&mut vec, &sim, &mut t, &mut store, PX);
    assert_eq!(field(&store, 1), Some(0.81), "re-semeou sem a forma mudar");
}
