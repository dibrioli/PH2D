//! A forma vetorial vista pelo **gizmo de sprite** (ADR-0111).
//!
//! Não há gizmo vetorial próprio. Havia — 492 linhas que mutavam a geometria — e
//! ele foi removido. Um path com `Transform` é um objeto como qualquer outro, e a
//! matemática de mover/girar/escalar já existe, testada, em `ph2d-editor-core`.
//!
//! A tradução é exata, não uma imitação. O gizmo enquadra um sprite como
//!
//! ```text
//! centro = translation + R·(anchor ⊙ scale)     meia-extensão = half_intrínseco ⊙ scale
//! ```
//!
//! e uma forma vetorial tem a MESMA forma se lermos `anchor` como o **centro da
//! bbox local da curva** e `half_intrínseco` como a **meia-extensão dessa bbox**.
//! Por isso `opposite_anchor_translation` (que fixa o canto oposto ao escalar) vale
//! sem uma linha nova: ela só depende desses dois números.
//!
//! Aqui moram também o **picking** de canvas — clicar numa forma fora da ferramenta
//! vetorial a seleciona como se fosse um sprite — e o marquee.

use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor::GizmoView;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::{VecPathId, VecScene, VecViewState};

use crate::vec_entities::VecEntityMap;
use crate::vec_transform::{world_transform, xform_of_transform};

/// O `anchor` e o meio-tamanho **intrínsecos** (pré-escala) da forma de `entity`,
/// na linguagem que o gizmo de sprite fala. `None` se a entidade não é um path, ou
/// se o path está vazio.
#[must_use]
pub(crate) fn anchor_half(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
) -> Option<([f32; 2], [f32; 2])> {
    let vp = sim.world().get::<VecPathRef>(entity)?;
    let (lo, hi) = scene.path_curve_bbox(vp.0)?;
    let anchor = [
        ((lo[0] + hi[0]) * 0.5) as f32,
        ((lo[1] + hi[1]) * 0.5) as f32,
    ];
    let half = [
        ((hi[0] - lo[0]) * 0.5) as f32,
        ((hi[1] - lo[1]) * 0.5) as f32,
    ];
    Some((anchor, half))
}

/// A `GizmoView` de uma forma vetorial — o mesmo `bbox_world` + `pivot` + `rotation`
/// que um sprite publica, para que `paint_sprite_gizmo` desenhe e registre as alças.
///
/// A pose vem do `SimWorld` (`Transform` ∘ cadeia de pais), e não do `PresentWorld`:
/// um path não é extraído para lá — ele não tem `RenderInstance`.
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
) -> Option<GizmoView> {
    let (anchor, half_intrinsic) = anchor_half(sim, scene, entity)?;
    let wt = world_transform(sim, entity);
    let (sx, sy) = (wt.scale.x, wt.scale.y);
    let half = [
        (half_intrinsic[0] * sx).abs(),
        (half_intrinsic[1] * sy).abs(),
    ];
    // Invariante idêntica à do sprite: quad center = pivot + R·(anchor ⊙ scale).
    let (ax, ay) = (anchor[0] * sx, anchor[1] * sy);
    // T1.3.5 cross-OS bit-identical.
    let (sin_r, cos_r) = libm::sincosf(wt.rotation);
    let cx = wt.translation.x + ax * cos_r - ay * sin_r;
    let cy = wt.translation.y + ax * sin_r + ay * cos_r;
    Some(GizmoView {
        bbox_min_world: [cx - half[0], cy - half[1]],
        bbox_max_world: [cx + half[0], cy + half[1]],
        pivot_world: [wt.translation.x, wt.translation.y],
        pivot_tool_active,
        rotation: wt.rotation,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
        cursor_screen: Some(last_pointer),
    })
}

/// O ponto de mundo `p` cai DENTRO da forma de `entity`?
///
/// É o análogo vetorial de `pick_sprite_at_world` para uma entidade já conhecida —
/// o que decide se um Down no interior da forma inicia um move (e não um pan).
#[must_use]
pub(crate) fn contains_world(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    p: [f32; 2],
) -> bool {
    let Some(vp) = sim.world().get::<VecPathRef>(entity) else {
        return false;
    };
    contains_path(sim, scene, entity, vp.0, p)
}

/// `p` (mundo) está dentro do path `id`, cuja entidade é `entity`?
fn contains_path(
    sim: &SimWorld,
    scene: &VecScene,
    entity: Entity,
    id: VecPathId,
    p: [f32; 2],
) -> bool {
    let x = xform_of_transform(world_transform(sim, entity));
    let Some(inv) = x.inverse() else {
        return false; // forma colapsada: sem interior
    };
    scene.path_contains_point(id, inv.apply([f64::from(p[0]), f64::from(p[1])]))
}

/// Toda forma vetorial sob `p` (mundo), **do topo para o fundo** — a lista que o
/// clique-cíclico do canvas consome. Escondida ou travada não entra, como um sprite
/// não entraria.
///
/// `paths` está em ordem de z (fundo → topo), então varre-se ao contrário.
#[must_use]
pub(crate) fn pick_all_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    p: [f32; 2],
) -> Vec<u64> {
    let mut out = Vec::new();
    for path in scene.paths().iter().rev() {
        if !view_state.is_pickable(path.id) {
            continue;
        }
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_ok() && contains_path(sim, scene, e, path.id, p) {
            out.push(bits);
        }
    }
    out
}

/// A forma mais ao topo sob `p`, ou `None`. Conveniência dos testes: o canvas
/// consome a lista inteira ([`pick_all_at_world`]) para poder ciclar entre
/// sobreposições.
#[cfg(test)]
#[must_use]
fn pick_at_world(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    p: [f32; 2],
) -> Option<u64> {
    pick_all_at_world(sim, scene, view_state, map, p)
        .into_iter()
        .next()
}

/// Toda forma vetorial cuja bbox de mundo intersecta o retângulo — o marquee.
#[must_use]
pub(crate) fn pick_in_world_rect(
    sim: &SimWorld,
    scene: &VecScene,
    view_state: &VecViewState,
    map: &VecEntityMap,
    rect_min: [f32; 2],
    rect_max: [f32; 2],
) -> Vec<u64> {
    let mut out = Vec::new();
    for path in scene.paths() {
        if !view_state.is_pickable(path.id) {
            continue;
        }
        let Some(&bits) = map.get(&path.id) else {
            continue;
        };
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        let Some((lo, hi)) = scene.path_curve_bbox(path.id) else {
            continue;
        };
        let x = xform_of_transform(world_transform(sim, e));
        // Os 4 cantos do bbox LOCAL sobem ao mundo; uma forma girada dá um
        // quadrilátero, e o bbox dele é o que se compara com o marquee.
        let corners = [
            x.apply(lo),
            x.apply([hi[0], lo[1]]),
            x.apply(hi),
            x.apply([lo[0], hi[1]]),
        ];
        let (mut wlo, mut whi) = (corners[0], corners[0]);
        for c in &corners[1..] {
            wlo = [wlo[0].min(c[0]), wlo[1].min(c[1])];
            whi = [whi[0].max(c[0]), whi[1].max(c[1])];
        }
        let overlaps = whi[0] >= f64::from(rect_min[0])
            && wlo[0] <= f64::from(rect_max[0])
            && whi[1] >= f64::from(rect_min[1])
            && wlo[1] <= f64::from(rect_max[1]);
        if overlaps {
            out.push(bits);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    use ph2d_vec_scene::rectangle;

    fn scene_with_square() -> (SimWorld, VecScene, VecEntityMap, Entity) {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, VecPathRef(id)))
            .id();
        map.insert(id, e.to_bits());
        (sim, scene, map, e)
    }

    /// O gizmo lê a forma como um sprite: `anchor` = centro da bbox local,
    /// `half` = meia-extensão dela. Um quadrado [-1,1]² centrado na origem.
    #[test]
    fn a_path_reports_its_local_bbox_as_a_sprite_anchor_and_half() {
        let (sim, scene, _, e) = scene_with_square();
        let (anchor, half) = anchor_half(&sim, &scene, e).unwrap();
        assert_eq!(anchor, [0.0, 0.0]);
        assert_eq!(half, [1.0, 1.0]);
    }

    /// A `GizmoView` acompanha o `Transform`: transladar move a caixa, escalar a
    /// cresce, e o pivô continua sendo a origem da entidade.
    #[test]
    fn the_gizmo_box_follows_the_transform_of_the_entity() {
        let (mut sim, scene, _, e) = scene_with_square();
        let cam = Camera2d::default();
        let ws = WindowSize {
            width: 800,
            height: 600,
        };
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: Vec2::new(10.0, 5.0),
            scale: Vec2::new(3.0, 3.0),
            ..Transform::IDENTITY
        });
        let v = view(&sim, &scene, e, &cam, ws, (0.0, 0.0), false).unwrap();
        assert_eq!(v.pivot_world, [10.0, 5.0]);
        assert_eq!(v.bbox_min_world, [7.0, 2.0], "10±3, 5±3");
        assert_eq!(v.bbox_max_world, [13.0, 8.0]);
    }

    /// O picking respeita o `Transform`: o interior está onde a forma é DESENHADA,
    /// não onde ela é guardada.
    #[test]
    fn picking_finds_the_shape_where_the_transform_puts_it() {
        let (mut sim, scene, map, e) = scene_with_square();
        let vs = VecViewState::default();
        assert!(pick_at_world(&sim, &scene, &vs, &map, [0.0, 0.0]).is_some());

        sim.world_mut().entity_mut(e).insert(Transform {
            translation: Vec2::new(50.0, 0.0),
            ..Transform::IDENTITY
        });
        assert_eq!(
            pick_at_world(&sim, &scene, &vs, &map, [0.0, 0.0]),
            None,
            "a origem ficou vazia"
        );
        assert_eq!(
            pick_at_world(&sim, &scene, &vs, &map, [50.0, 0.0]),
            Some(e.to_bits()),
            "a forma está onde o transform a pôs"
        );
    }

    /// Travada ou escondida não é selecionável no canvas — como um sprite.
    #[test]
    fn a_hidden_or_locked_shape_is_not_pickable() {
        let (sim, scene, map, _) = scene_with_square();
        let id = scene.paths()[0].id;
        let hidden = VecViewState {
            hidden: vec![id],
            locked: Vec::new(),
        };
        assert_eq!(pick_at_world(&sim, &scene, &hidden, &map, [0.0, 0.0]), None);
        let locked = VecViewState {
            hidden: Vec::new(),
            locked: vec![id],
        };
        assert_eq!(pick_at_world(&sim, &scene, &locked, &map, [0.0, 0.0]), None);
    }

    /// O marquee pega a forma pela bbox de MUNDO.
    #[test]
    fn the_marquee_selects_a_translated_shape_by_its_world_bbox() {
        let (mut sim, scene, map, e) = scene_with_square();
        let vs = VecViewState::default();
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: Vec2::new(20.0, 20.0),
            ..Transform::IDENTITY
        });
        assert!(pick_in_world_rect(&sim, &scene, &vs, &map, [-5.0, -5.0], [5.0, 5.0]).is_empty());
        assert_eq!(
            pick_in_world_rect(&sim, &scene, &vs, &map, [15.0, 15.0], [25.0, 25.0]),
            vec![e.to_bits()]
        );
    }
}
