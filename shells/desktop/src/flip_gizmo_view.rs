//! O objeto Flip visto pelo **gizmo de sprite** (ADR-0111 parity, espelho de
//! [`crate::vec_gizmo_view`]).
//!
//! Não há gizmo Flip próprio. Um objeto com `Transform` é um objeto como qualquer
//! outro, e a matemática de mover/girar/escalar já existe, testada, em
//! `ph2d-editor-core`. A tradução é exata: o gizmo enquadra um sprite como
//!
//! ```text
//! centro = translation + R·(anchor ⊙ scale)     meia-extensão = half_intrínseco ⊙ scale
//! ```
//!
//! e um objeto Flip tem a MESMA forma se lermos `anchor` como o **centro da bbox
//! local da arte** e `half_intrínseco` como a **meia-extensão dessa bbox**.
//!
//! Aqui moram também o **picking** de canvas — clicar na arte fora da tool Flip a
//! seleciona como um sprite — e o marquee. A arte é uma nuvem de traços; o hit é
//! por proximidade de traço (não há "interior" como numa forma vetorial fechada).

use ph2d_ecs::{Entity, FlipObjectRef, SimWorld};
use ph2d_editor::GizmoView;
use ph2d_flip::{FlipDoc, FlipObjectId};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

use crate::flip_entities::FlipEntityMap;
use crate::flip_transform::object_xform;
use crate::vec_transform::world_transform;

/// Raio de captura do traço, em pixels de tela — a arte Flip é pega por proximidade
/// (uma nuvem de linhas não tem interior). Igual ao vetor.
const STROKE_HIT_PX: f64 = 8.0;

/// `STROKE_HIT_PX` convertido a world-units no zoom atual.
#[must_use]
pub(crate) fn stroke_hit_r(camera: &Camera2d, window_size: WindowSize) -> f64 {
    let w0 = camera.screen_to_world((0.0, 0.0), window_size);
    let w1 = camera.screen_to_world((1.0, 0.0), window_size);
    let px = ((f64::from(w1[0] - w0[0])).powi(2) + (f64::from(w1[1] - w0[1])).powi(2)).sqrt();
    STROKE_HIT_PX * px
}

/// O `FlipObjectId` que a entidade referencia, se for um objeto Flip.
fn object_of(sim: &SimWorld, entity: Entity) -> Option<FlipObjectId> {
    sim.world()
        .get::<FlipObjectRef>(entity)
        .map(|r| FlipObjectId(r.0))
}

/// O `anchor` e o meio-tamanho **intrínsecos** (pré-escala) do objeto de `entity`,
/// na linguagem que o gizmo de sprite fala. `None` se a entidade não é um objeto
/// Flip, ou se o objeto não tem arte.
#[must_use]
pub(crate) fn anchor_half(
    sim: &SimWorld,
    doc: &FlipDoc,
    entity: Entity,
) -> Option<([f32; 2], [f32; 2])> {
    let oid = object_of(sim, entity)?;
    let (lo, hi) = doc.object(oid)?.geometry_bbox()?;
    let anchor = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    let half = [(hi[0] - lo[0]) * 0.5, (hi[1] - lo[1]) * 0.5];
    Some((anchor, half))
}

/// A `GizmoView` de um objeto Flip — o mesmo `bbox_world` + `pivot` + `rotation` que
/// um sprite publica, para que `paint_sprite_gizmo` desenhe e registre as alças.
/// A pose vem do `SimWorld` (`Transform` ∘ cadeia de pais).
#[must_use]
pub(crate) fn view(
    sim: &SimWorld,
    doc: &FlipDoc,
    entity: Entity,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    pivot_tool_active: bool,
) -> Option<GizmoView> {
    let (anchor, half_intrinsic) = anchor_half(sim, doc, entity)?;
    let wt = world_transform(sim, entity);
    let (sx, sy) = (wt.scale.x, wt.scale.y);
    let half = [(half_intrinsic[0] * sx).abs(), (half_intrinsic[1] * sy).abs()];
    // Invariante idêntica à do sprite: quad center = pivot + R·(anchor ⊙ scale).
    let (ax, ay) = (anchor[0] * sx, anchor[1] * sy);
    let (sin_r, cos_r) = libm::sincosf(wt.rotation); // T1.3.5 bit-idêntico cross-OS
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

/// O ponto de mundo `p` pega a arte do objeto de `entity` (a ≤ `stroke_hit_r` de
/// algum traço)? A geometria é LOCAL, então `p` desce ao espaço local pelo afim
/// inverso e o raio (world) converte pela escala.
#[must_use]
pub(crate) fn contains_world(
    sim: &SimWorld,
    doc: &FlipDoc,
    entity: Entity,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    let Some(oid) = object_of(sim, entity) else {
        return false;
    };
    contains_object(sim, doc, entity, oid, p, stroke_hit_r)
}

fn contains_object(
    sim: &SimWorld,
    doc: &FlipDoc,
    entity: Entity,
    oid: FlipObjectId,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> bool {
    let x = object_xform(sim, entity);
    let Some(inv) = x.inverse() else {
        return false; // objeto colapsado
    };
    let local = inv.apply([f64::from(p[0]), f64::from(p[1])]);
    let scale = x.mean_scale();
    // O raio é WORLD; o teste é LOCAL → converte o raio pela escala do objeto.
    let r_local = if scale > 0.0 { stroke_hit_r / scale } else { stroke_hit_r };
    let r2 = r_local * r_local;
    let Some(obj) = doc.object(oid) else {
        return false;
    };
    for d in obj.drawings() {
        for s in &d.strokes {
            let pos = s.positions();
            for w in pos.windows(2) {
                let d2 = seg_dist2(
                    local,
                    [f64::from(w[0].x), f64::from(w[0].y)],
                    [f64::from(w[1].x), f64::from(w[1].y)],
                );
                if d2 <= r2 {
                    return true;
                }
            }
            // Um traço de 1 ponto (toque) ainda pega no próprio ponto.
            if pos.len() == 1 {
                let dx = local[0] - f64::from(pos[0].x);
                let dy = local[1] - f64::from(pos[0].y);
                if dx * dx + dy * dy <= r2 {
                    return true;
                }
            }
        }
    }
    false
}

/// Distância² de `p` ao segmento `a`→`b` (tudo no mesmo espaço).
fn seg_dist2(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1];
    let t = if len2 > 0.0 {
        ((ap[0] * ab[0] + ap[1] * ab[1]) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let c = [a[0] + t * ab[0], a[1] + t * ab[1]];
    let d = [p[0] - c[0], p[1] - c[1]];
    d[0] * d[0] + d[1] * d[1]
}

/// Todo objeto Flip sob `p` (mundo), do topo para o fundo — a lista que o clique-
/// cíclico do canvas consome. Escondido ou travado não entra.
#[must_use]
pub(crate) fn pick_all_at_world(
    sim: &SimWorld,
    doc: &FlipDoc,
    map: &FlipEntityMap,
    p: [f32; 2],
    stroke_hit_r: f64,
) -> Vec<u64> {
    let mut out = Vec::new();
    // `map` é BTree por id; a ordem de z real é a da árvore, mas o ciclo de clique
    // só precisa de TODOS os hits (o dispatch já ordena/rotaciona). Varre em ordem
    // estável de id.
    for (&oid, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        if is_hidden_or_locked(sim, e) {
            continue;
        }
        if contains_object(sim, doc, e, oid, p, stroke_hit_r) {
            out.push(bits);
        }
    }
    out
}

/// Todo objeto Flip cuja bbox de MUNDO intersecta o retângulo — o marquee.
#[must_use]
pub(crate) fn pick_in_world_rect(
    sim: &SimWorld,
    doc: &FlipDoc,
    map: &FlipEntityMap,
    rect_min: [f32; 2],
    rect_max: [f32; 2],
) -> Vec<u64> {
    let mut out = Vec::new();
    for (&oid, &bits) in map {
        let e = Entity::from_bits(bits);
        if sim.world().get_entity(e).is_err() {
            continue;
        }
        if is_hidden_or_locked(sim, e) {
            continue;
        }
        let Some((lo, hi)) = doc.object(oid).and_then(|o| o.geometry_bbox()) else {
            continue;
        };
        let x = object_xform(sim, e);
        // Os 4 cantos do bbox LOCAL sobem ao mundo; o bbox do quadrilátero é o que
        // se compara com o marquee.
        let corners = [
            x.apply([f64::from(lo[0]), f64::from(lo[1])]),
            x.apply([f64::from(hi[0]), f64::from(lo[1])]),
            x.apply([f64::from(hi[0]), f64::from(hi[1])]),
            x.apply([f64::from(lo[0]), f64::from(hi[1])]),
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

/// Escondido/travado herdado pela cadeia da Hierarquia — como um sprite, não é
/// selecionável no canvas. Trava = `is_locked_for_edit` (o mesmo predicado do gizmo
/// de sprite: `Locked` no próprio ou `GroupedChildren` num ancestral); escondido =
/// `Visibility.hidden` no próprio ou em qualquer ancestral (esconder um grupo
/// esconde os filhos).
fn is_hidden_or_locked(sim: &SimWorld, entity: Entity) -> bool {
    let w = sim.world();
    if ph2d_ecs::is_locked_for_edit(w, entity) {
        return true;
    }
    let mut cur = Some(entity);
    for _ in 0..64 {
        let Some(e) = cur else { return false };
        if w.get::<ph2d_ecs::Visibility>(e).is_some_and(|v| v.hidden) {
            return true;
        }
        cur = w.get::<ph2d_ecs::ChildOf>(e).map(|c| c.parent());
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{Name, Transform};
    use ph2d_flip::{FlipStroke, Hold, KeyKind};

    fn doc_with_segment(
        a: [f32; 2],
        b: [f32; 2],
    ) -> (FlipDoc, SimWorld, FlipEntityMap, FlipObjectId, Entity) {
        let mut doc = FlipDoc::new();
        let oid = doc.push_object("Obj");
        let obj = doc.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj.insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe).unwrap();
        let mut s = FlipStroke::new();
        s.push_default(ph2d_core::Vec2::new(a[0], a[1]));
        s.push_default(ph2d_core::Vec2::new(b[0], b[1]));
        obj.drawing_mut(d).unwrap().strokes.push(s);
        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Obj"), FlipObjectRef(oid.0)))
            .id();
        let mut map = FlipEntityMap::new();
        map.insert(oid, e.to_bits());
        (doc, sim, map, oid, e)
    }

    /// O gizmo lê o objeto como um sprite: `anchor` = centro da bbox local, `half`
    /// = meia-extensão. Traço de (10,20) a (30,40): centro (20,30), meia (10,10).
    #[test]
    fn an_object_reports_its_local_bbox_as_a_sprite_anchor_and_half() {
        let (doc, sim, _, _, e) = doc_with_segment([10.0, 20.0], [30.0, 40.0]);
        let (anchor, half) = anchor_half(&sim, &doc, e).unwrap();
        assert_eq!(anchor, [20.0, 30.0]);
        assert_eq!(half, [10.0, 10.0]);
    }

    /// A `GizmoView` acompanha o `Transform`: transladar move a caixa, escalar a
    /// cresce, e o pivô é a origem da entidade.
    #[test]
    fn the_gizmo_box_follows_the_transform() {
        let (doc, mut sim, _, _, e) = doc_with_segment([-1.0, -1.0], [1.0, 1.0]);
        let cam = Camera2d::default();
        let ws = WindowSize { width: 800, height: 600 };
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: ph2d_core::Vec2::new(10.0, 5.0),
            scale: ph2d_core::Vec2::new(3.0, 3.0),
            ..Transform::IDENTITY
        });
        let v = view(&sim, &doc, e, &cam, ws, (0.0, 0.0), false).unwrap();
        assert_eq!(v.pivot_world, [10.0, 5.0]);
        assert_eq!(v.bbox_min_world, [7.0, 2.0], "10±3, 5±3");
        assert_eq!(v.bbox_max_world, [13.0, 8.0]);
    }

    /// O picking respeita o `Transform`: a arte está onde é DESENHADA, não onde é
    /// guardada. Um clique EM CIMA do traço pega; longe, não.
    #[test]
    fn picking_finds_the_art_where_the_transform_puts_it() {
        let (mut doc, mut sim, map, oid, e) = doc_with_segment([0.0, 0.0], [10.0, 0.0]);
        // Assenta o pivô (centro (5,0)) → geometria local, Transform.translation=(5,0).
        crate::flip_transform::settle_origins(&mut sim, &mut doc, &map, None);
        // Clique sobre o traço, no mundo.
        assert!(!pick_all_at_world(&sim, &doc, &map, [5.0, 0.05], 1.0).is_empty());
        // Move o objeto +100 em x: a origem do traço fica vazia.
        let _ = oid;
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: ph2d_core::Vec2::new(105.0, 0.0),
            ..Transform::IDENTITY
        });
        assert!(pick_all_at_world(&sim, &doc, &map, [5.0, 0.05], 1.0).is_empty());
        assert!(!pick_all_at_world(&sim, &doc, &map, [105.0, 0.05], 1.0).is_empty());
    }

    /// O marquee pega o objeto pela bbox de MUNDO.
    #[test]
    fn the_marquee_selects_a_translated_object_by_its_world_bbox() {
        let (mut doc, mut sim, map, _oid, e) = doc_with_segment([-1.0, -1.0], [1.0, 1.0]);
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: ph2d_core::Vec2::new(20.0, 20.0),
            ..Transform::IDENTITY
        });
        assert!(pick_in_world_rect(&sim, &doc, &map, [-5.0, -5.0], [5.0, 5.0]).is_empty());
        assert_eq!(
            pick_in_world_rect(&sim, &doc, &map, [15.0, 15.0], [25.0, 25.0]),
            vec![e.to_bits()]
        );
    }
}
