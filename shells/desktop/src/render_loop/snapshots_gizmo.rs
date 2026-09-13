//! **O gizmo da seleção: a caixa de uma sprite e a vista GLOBAL** — os dois pedaços do passe do gizmo do [`super`]
//! (`snapshots`) que não precisam do resto dele. Filho por `#[path]` (OBRA 3 da `line/render-bodies`): o `build_view`
//! chama [`sprite_view`] no ramo da sprite, e a `publish` chama [`global_view`] no sítio do bloco.

use super::*;

/// A vista global (a união da seleção com o recuo das alças, ou a do arrasto global em curso) e a supressão do
/// gizmo de sprite sob o Transform do Deform.
pub(super) fn global_view(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    last_pointer: (f32, f32),
    suppress_sprite_gizmo: bool,
) {
    // Onda 2 polish: while a Global gizmo drag is alive, derive the
    // global view from the cached `global_view_start` snapshot +
    // primary's transform deltas. This is what makes the global gizmo
    // **rotate visually** during a Global Rotate (and scale rigidly
    // during a Global Scale) instead of being the axis-aligned union
    // of rotated sprites — that union grows under rotation, which
    // would make the gizmo "balloon" rather than rotate.
    let global_from_drag = if let (Some(start), Some(drag)) = (
        hero.gizmo.global_view_start.as_ref().copied(),
        hero.gizmo.drag.as_ref().copied(),
    ) && matches!(drag.target, ph2d_editor_core::GizmoTarget::Global)
    {
        let primary_entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let world = sim.world();
        let (delta_rot, factor_x, factor_y) =
            if let Some(t) = world.get::<Transform>(primary_entity) {
                let dr = t.rotation - drag.start_transform.rotation;
                let fx = if drag.start_transform.scale[0].abs() > f32::EPSILON {
                    t.scale.x / drag.start_transform.scale[0]
                } else {
                    1.0
                };
                let fy = if drag.start_transform.scale[1].abs() > f32::EPSILON {
                    t.scale.y / drag.start_transform.scale[1]
                } else {
                    1.0
                };
                (dr, fx, fy)
            } else {
                (0.0, 1.0, 1.0)
            };
        let cx_s = (start.bbox_min_world[0] + start.bbox_max_world[0]) * 0.5;
        let cy_s = (start.bbox_min_world[1] + start.bbox_max_world[1]) * 0.5;
        let hw_s = (start.bbox_max_world[0] - start.bbox_min_world[0]) * 0.5;
        let hh_s = (start.bbox_max_world[1] - start.bbox_min_world[1]) * 0.5;
        // Onda 2 hotfix: global drags (Scale + Rotate) PIVOT around the
        // start centre. The primary's translation shifts as a side
        // effect of the rotation/scale, but the gizmo's centre stays
        // at the original pivot — using the primary's delta_translation
        // here was making the gizmo drift away from the sprites it
        // covers (smoke: "o desenho do gizmo não rotaciona corretamente
        // em seu centro causando um drift entre as sprites e o
        // desenho do gizmo"). Global has no Translate handle (we
        // dropped BBOX_INTERIOR for keyed gizmos), so this branch only
        // sees Scale + Rotate.
        let new_cx = cx_s;
        let new_cy = cy_s;
        let new_hw = hw_s * factor_x.abs();
        let new_hh = hh_s * factor_y.abs();
        Some(ph2d_editor_core::GizmoView {
            bbox_min_world: [new_cx - new_hw, new_cy - new_hh],
            bbox_max_world: [new_cx + new_hw, new_cy + new_hh],
            pivot_world: [new_cx, new_cy],
            pivot_tool_active: false,
            rotation: delta_rot,
            camera_center: start.camera_center,
            camera_height_world: start.camera_height_world,
            window_w: start.window_w,
            window_h: start.window_h,
            canvas: start.canvas,
            cursor_screen: Some(last_pointer),
        })
    } else {
        None
    };
    // Onda 2: global view = union of every selected sprite's bbox,
    // EXPANDED by a fixed screen offset so the global gizmo's handles
    // sit clear of the individual gizmos' handles (Enio: "o gizmo da
    // multiseleção com offset em relação aos gizmos individuais para
    // não conflitar as alças de manipulação"). 32 px in screen space,
    // converted to world units at the current zoom so the offset
    // tracks the zoom level — handles stay one handle-size + a gap
    // outside the individuals at any scale.
    // Conta VIEWS, não bits selecionados: uma seleção de 1 sprite + 1 path
    // vetorial (ADR-0110) tem `selected_len() == 2` mas uma view só, e o gizmo
    // global desenharia — deslocado 32 px — em volta de um sprite sozinho.
    let painted_views = usize::from(hero.gizmo.view.is_some()) + hero.gizmo.extra_views.len();
    hero.gizmo.global_view = if let Some(v) = global_from_drag {
        Some(v)
    } else if painted_views > 1 {
        let primary = hero.gizmo.view.as_ref();
        let mut iter = primary
            .into_iter()
            .chain(hero.gizmo.extra_views.iter().map(|(_, v)| v));
        iter.next().map(|first| {
            let mut min_x = first.bbox_min_world[0];
            let mut min_y = first.bbox_min_world[1];
            let mut max_x = first.bbox_max_world[0];
            let mut max_y = first.bbox_max_world[1];
            for v in iter {
                min_x = min_x.min(v.bbox_min_world[0]);
                min_y = min_y.min(v.bbox_min_world[1]);
                max_x = max_x.max(v.bbox_max_world[0]);
                max_y = max_y.max(v.bbox_max_world[1]);
            }
            let pixel_to_world = first.camera_height_world / first.window_h.max(1.0);
            let offset_world = 32.0 * pixel_to_world;
            ph2d_editor_core::GizmoView {
                bbox_min_world: [min_x - offset_world, min_y - offset_world],
                bbox_max_world: [max_x + offset_world, max_y + offset_world],
                pivot_world: [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5],
                pivot_tool_active: false,
                rotation: 0.0,
                camera_center: first.camera_center,
                camera_height_world: first.camera_height_world,
                window_w: first.window_w,
                window_h: first.window_h,
                canvas: first.canvas,
                cursor_screen: first.cursor_screen,
            }
        })
    } else {
        None
    };
    // While the Painter's Deform **Transform** gizmo is live (Uniform / Free / Distort / Warp), the
    // SPRITE gizmo is fully suppressed — view, extras and the global union. On a whole-image transform
    // both gizmos put their corner squares on the SAME screen corners, and a near-corner Down grabbed
    // the sprite's scale handle instead of the deform's (Enio 2026-07-04: "inative o gizmo da sprite
    // para as quatro ferramentas de Transform"). No view ⇒ nothing painted ⇒ no handle registered in
    // the hit index ⇒ every corner click reaches the deform gizmo.
    if suppress_sprite_gizmo {
        hero.gizmo.view = None;
        hero.gizmo.extra_views.clear();
        hero.gizmo.global_view = None;
    }
}

/// A caixa de uma SPRITE: o `GlobalTransform` do espelho no `present`, e a caixa da folha aberta quando ela está.
#[allow(clippy::too_many_arguments)]
pub(super) fn sprite_view(
    bits: u64,
    sim_entity: ph2d_ecs::Entity,
    sim: &SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
    gizmo_ppm: f32,
    sheet_gizmo_bits: Option<u64>,
    tool_preview_bits: &[Option<u64>],
) -> Option<ph2d_editor_core::GizmoView> {
    let sprite = sim.world().get::<Sprite>(sim_entity)?;
    let mut q = present
        .world_mut()
        .query::<(&SimRef, &ph2d_ecs::GlobalTransform)>();
    let gt = q.iter(present.world()).find_map(|(sref, gt)| {
        if sref.0 == sim_entity {
            Some(*gt)
        } else {
            None
        }
    })?;
    let affine = gt.affine();
    let col0_x = affine[0];
    let col0_y = affine[1];
    let col1_x = affine[2];
    let col1_y = affine[3];
    let scale_x = (col0_x * col0_x + col0_y * col0_y).sqrt();
    let scale_y = (col1_x * col1_x + col1_y * col1_y).sqrt();
    let rotation = col0_y.atan2(col0_x);
    let p = gt.translation();
    // **COM A FOLHA ABERTA, A CAIXA ENVOLVE A FOLHA** (Enio, 2026-08-23: *«o gizmo da
    // sprite deve englobar todas as células»*). A escolha e os números vivem em
    // `sheet_grid_overlay::gizmo_box`, que é onde eles têm gate — aqui só se aplica a
    // escala e a rotação, como sempre.
    let (eff_anchor, half) = crate::render_loop::sheet_grid_overlay::gizmo_box(
        sprite,
        sim.world().get::<ph2d_ecs::SpriteGrid>(sim_entity).copied(),
        gizmo_ppm,
        sheet_gizmo_bits == Some(bits),
        crate::render_loop::sim_extract_sheet::is_tool_previewed(tool_preview_bits, sim_entity),
    );
    let half_w = half[0] * scale_x;
    let half_h = half[1] * scale_y;
    let ax = eff_anchor[0] * scale_x;
    let ay = eff_anchor[1] * scale_y;
    // T1.3.5 cross-OS bit-identical.
    let (sin_r, cos_r) = libm::sincosf(rotation);
    let cx = p.x + ax * cos_r - ay * sin_r;
    let cy = p.y + ax * sin_r + ay * cos_r;
    Some(ph2d_editor_core::GizmoView {
        bbox_min_world: [cx - half_w, cy - half_h],
        bbox_max_world: [cx + half_w, cy + half_h],
        pivot_world: [p.x, p.y],
        pivot_tool_active,
        rotation,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor_core::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
        cursor_screen: Some(last_pointer),
    })
}
