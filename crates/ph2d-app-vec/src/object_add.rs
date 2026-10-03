//! ⭐ **As formas vetoriais no menu Add de objectos** (spec/06 F1) — as entradas e o nascimento.
//!
//! ⚠️ **A forma nasce pela FERRAMENTA de forma, e não montada à mão:** premir, arrastar e largar
//! o [`ShapeTool`](ph2d_vec_edit::ShapeTool) sobre um quadrado. É o mesmo commit do gesto — o
//! estilo da ferramenta, os valores do painel e o pedido de *nascer viva* — e a forma do menu é
//! indistinguível da desenhada. Uma segunda porta divergiria da primeira no dia em que o gesto
//! mudasse.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::object_add::{AddEntry, AddGroup};
use ph2d_vec_scene::{ShapeKind, VecScene};

/// Rectângulo.
pub const RECTANGLE: AddEntry = AddEntry::new("object_add.vector.rectangle", AddGroup::TwoD);
/// Elipse.
pub const ELLIPSE: AddEntry = AddEntry::new("object_add.vector.ellipse", AddGroup::TwoD);
/// Polígono.
pub const POLYGON: AddEntry = AddEntry::new("object_add.vector.polygon", AddGroup::TwoD);
/// Estrela.
pub const STAR: AddEntry = AddEntry::new("object_add.vector.star", AddGroup::TwoD);
/// O que esta família põe no menu.
pub const ENTRIES: &[AddEntry] = &[RECTANGLE, ELLIPSE, POLYGON, STAR];

/// A fracção da altura visível que o lado de uma forma nova ocupa — a mesma regra do Model
/// (`ph2d_app_field3d::scene::new_shape_size`): uma forma nova tem de ser VISTA, e um tamanho fixo
/// em mundo nasce invisível com zoom longe e tapa a vista com zoom perto.
const SIDE_OF_VIEW: f64 = 0.25;

#[must_use]
fn kind_of(entry: AddEntry) -> Option<ShapeKind> {
    Some(match entry {
        e if e == RECTANGLE => ShapeKind::Rectangle,
        e if e == ELLIPSE => ShapeKind::Ellipse,
        e if e == POLYGON => ShapeKind::Polygon,
        e if e == STAR => ShapeKind::Star,
        _ => return None,
    })
}

/// ⭐ **Cria a forma** desta entrada centrada em `at` (mundo) e devolve os bits da entidade —
/// `None` se a entrada não é desta família.
///
/// `px_to_world` é a escala da vista e `view_px` a altura visível do canvas, em píxeis.
pub fn add(
    entry: AddEntry,
    sim: &mut SimWorld,
    scene: &mut VecScene,
    vec: &mut crate::state::VecState,
    at: [f64; 2],
    px_to_world: f64,
    view_px: f64,
) -> Option<Result<u64, &'static str>> {
    let kind = kind_of(entry)?;
    let half = view_px * SIDE_OF_VIEW * px_to_world * 0.5;
    let values = ph2d_tool_vector::shapes::to_world(kind, &vec.draw_config.values, px_to_world);
    let constraint = ph2d_vec_edit::ShapeConstraint::default();
    vec.shape.on_press(
        scene,
        kind,
        values,
        [at[0] - half, at[1] - half],
        px_to_world,
        constraint,
    );
    vec.shape
        .on_drag(scene, [at[0] + half, at[1] + half], constraint);
    if !vec.shape.on_release(scene) {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    }
    let Some(id) = vec.shape.pending_live() else {
        return Some(Err(ph2d_i18n::tr("object_add.not_born")));
    };
    ph2d_vec_entities::entities::sync(sim, scene, &mut vec.entities);
    crate::shape_live::make_committed_shape_live(sim, scene, &vec.entities, &mut vec.shape, false);
    Some(
        vec.entities
            .get(&id)
            .copied()
            .ok_or_else(|| ph2d_i18n::tr("object_add.not_born")),
    )
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
