//! **As marcas por cima dos nós** (o overlay de edição do vetor): o anel do nó SOLDADO (plano 39) e
//! os nós do EDIT SHAPE (F7e). Saíram da shell (`fase_vector_edit_overlay`), que é composição.

use crate::state::VecState;
use ph2d_tokens::Theme;
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState, VecXforms};
use ph2d_vector::{Affine, VectorScene};

/// O caminho cujas alças de Bézier o overlay de nós desenha. Com os nós do Edit Shape à vista,
/// nenhum: as alças genéricas da forma afinada seriam um 2.º conjunto por cima dos nós dela.
#[must_use]
pub fn node_overlay_selected(vec: &VecState) -> Option<VecPathId> {
    match vec.pencil.shape_edit_state() {
        ph2d_vec_edit::pencil::ShapeEditState::Open => None,
        _ => vec.pen.selected(),
    }
}

/// ⭐⭐⭐ **A marca do nó soldado** (report do Enio, 2026-09-01: *«as linhas não compartilham o mesmo
/// nó»* — e ele não tinha como ver: duas pontas coincidentes e duas a um pixel pintam o mesmo
/// quadrado). A LEI é a do arrasto (`welded_nodes`); aqui só o que se vê — um caminho escondido não
/// mostra marca —, pela MESMA projecção do overlay (`overlay_transform`). Depois, os nós do Edit
/// Shape, em mundo pela câmara.
pub fn draw(
    vec: &VecState,
    vec_scene: &VecScene,
    vec_view: &VecViewState,
    vec_xf: &VecXforms,
    cam_affine: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let marcas: Vec<[f64; 2]> = vec
        .pen
        .welded_nodes(vec_scene)
        .into_iter()
        .filter(|(id, _)| !vec_view.is_hidden(*id))
        .filter_map(|(id, i)| {
            let v = vec_scene.path(id)?.vert(i)?;
            let t = ph2d_vec_render::overlay_transform(vec_view, vec_xf, id, cam_affine);
            let p = t * ph2d_vector::Point::new(v.anchor[0], v.anchor[1]);
            Some([p.x, p.y])
        })
        .collect();
    ph2d_vec_render::draw_weld_marks(&marcas, theme, target);
    let (nodes, grabbed) = vec.pencil.shape_edit_nodes();
    ph2d_vec_render::draw_shape_nodes(&nodes, grabbed, cam_affine, theme, target);
}
