//! **O overlay de EDIÇÃO** — as âncoras e os handles que o modo Node manipula.
//!
//! Irmão do [`super`] pelo teto de 700 LOC, cortado por assunto: lá o desenho da ARTE, aqui o dos
//! controles que a editam.

use ph2d_vec_scene::{VecPathId, VecViewState, VecXforms};
use ph2d_vector::{Affine, Point, VectorScene};

use super::*;

/// **Onde as ÂNCORAS desta forma vão** — a porta única do overlay de edição.
///
/// `camera ∘ pose ∘ transform`: o transform leva a curva LOCAL ao mundo, a **pose** é o que o AUTO
/// LAYOUT lhe deu neste frame (identidade quando ninguém a colocou), e a câmera leva à tela.
///
/// ⚠️ Sem a pose as âncoras ficam onde a forma foi AUTORADA enquanto o desenho está onde a moldura
/// a pôs — Enio, 2026-08-02: *"os Path das formas aparecem no lugar de origem"*. Ela entra DEPOIS
/// do transform porque age sobre a geometria já posta no mundo.
///
/// ⚠️ **Só o LAYOUT entra aqui, e é uma distinção de natureza:** ele é uma pose (`translate ∘
/// scale`), então as âncoras viajam com a forma. Um Offset ou um Pattern MUDAM a curva, e as
/// âncoras deles ficam na fonte — que é o que o modo Node edita (a convenção do
/// `inkscape:original-d`).
#[must_use]
pub fn overlay_transform(
    view: &VecViewState,
    xforms: &VecXforms,
    id: VecPathId,
    camera: Affine,
) -> Affine {
    camera
        * Affine::new(view.layout_pose(id).0)
        * Affine::new(ph2d_vec_scene::xform_of(xforms, id).0)
}

/// Desenha os **gizmos de edição** por cima da cena (screen-space, tamanho
/// constante em px): quadradinho em cada âncora; e — só no path `selected` — as
/// linhas âncora→handle + bolinhas nos handles dos vértices suaves. Cores
/// hardcoded de scaffold (Fase 1); migram p/ tokens no chrome do cutover (Fase R).
#[allow(clippy::too_many_arguments)]
pub fn draw_overlays(
    scene: &VecScene,
    view: &VecViewState,
    selected: Option<VecPathId>,
    selected_paths: &[VecPathId],
    selected_verts: &[usize],
    xforms: &VecXforms,
    camera: Affine,
    target: &mut VectorScene,
) {
    for path in scene.paths() {
        if view.is_hidden(path.id) {
            continue; // um path escondido não mostra âncoras
        }
        let transform = overlay_transform(view, xforms, path.id, camera);
        let is_sel = Some(path.id) == selected;
        // Any path in the OBJECT selection set is highlighted; the primary also shows
        // its Bézier handles + per-vertex picks.
        let in_set = selected_paths.contains(&path.id);
        if is_sel {
            for (i, v) in path.verts_all().enumerate() {
                let a = transform * Point::new(v.anchor[0], v.anchor[1]);
                // Draw each handle at its DISPLAY position: a real (offset) handle
                // as-is, and a Smooth/Symmetric point's zero-length ("invisible")
                // handle as a grabbable GHOST stub along the smooth tangent — without
                // touching geometry (the curve only changes when the user drags it).
                // A straight Corner's zero handle stays hidden (`ghost_handle` = None).
                for out in [false, true] {
                    let real = if out { v.out_handle } else { v.in_handle };
                    let (dx, dy) = (real[0] - v.anchor[0], real[1] - v.anchor[1]);
                    let is_ghost = dx * dx + dy * dy <= 1e-18;
                    let Some(h) = ph2d_vec_scene::ghost_handle(path, i, out) else {
                        continue;
                    };
                    let hp = transform * Point::new(h[0], h[1]);
                    let mut line = BezPath::new();
                    line.move_to(a);
                    line.line_to(hp);
                    // Ghost stubs are dimmer + hollow so the user reads them as
                    // "suggested, not yet affecting the curve".
                    let line_a = if is_ghost { 120 } else { 200 };
                    target.inner_mut().stroke(
                        &Stroke::new(1.0),
                        Affine::IDENTITY,
                        &Brush::Solid(Color::from_rgba8(120, 190, 230, line_a)),
                        None,
                        &line,
                    );
                    if is_ghost {
                        target.inner_mut().stroke(
                            &Stroke::new(1.2),
                            Affine::IDENTITY,
                            &Brush::Solid(Color::from_rgba8(120, 190, 230, 220)),
                            None,
                            &Circle::new(hp, 3.5),
                        );
                    } else {
                        target.inner_mut().fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            &Brush::Solid(Color::from_rgba8(120, 190, 230, 255)),
                            None,
                            &Circle::new(hp, 3.5),
                        );
                    }
                }
            }
        }
        // Flat index across contours — the same space `hit_test` / `selected_verts`
        // address, so a hole's anchors pick exactly like the outer contour's.
        for (i, v) in path.verts_all().enumerate() {
            let a = transform * Point::new(v.anchor[0], v.anchor[1]);
            // A vertex in the multi-selection (selected path only) is drawn bigger
            // + cyan; other anchors of the selected path are orange; other paths gray.
            let picked = is_sel && selected_verts.contains(&i);
            let s = if picked { 4.5 } else { 3.5 };
            let col = if picked {
                Color::from_rgba8(90, 200, 235, 255) // ciano = vértice selecionado (grupo)
            } else if in_set {
                Color::from_rgba8(250, 180, 90, 255) // laranja = path na seleção de objeto
            } else {
                Color::from_rgba8(230, 230, 235, 220)
            };
            target.fill_rect(Rect::new(a.x - s, a.y - s, a.x + s, a.y + s), col);
        }
    }
}
