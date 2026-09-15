//! M14.7 A — sprite hit-testing in world space.
//!
//! Two operations the editor needs once the user starts manipulating
//! sprites directly on the canvas:
//!
//! - [`pick_sprite_at_world`] — given a cursor mapped through
//!   [`Camera2d::screen_to_world`](crate::camera::Camera2d::screen_to_world)
//!   to world coordinates, return the topmost sprite whose
//!   axis-aligned bbox contains the point.
//! - [`selection_bbox_world`] — given the sim-entity bits the editor
//!   stored as its current selection, recover the world-space rect
//!   the gizmo painter (M14.7 B) draws handles on.
//!
//! Both functions operate on **PresentWorld** because that's where
//! `GlobalTransform` lives (per ADR-0021). The host already mirrors
//! `SimRef` from sim → present every frame; that back-pointer is how
//! we surface a stable `entity_bits` to the editor without exposing a
//! `bevy_ecs::Entity` across the ADR-0021 / HR-8 boundary.
//!
//! ## Rotation / scale / skew handling
//!
//! Picking honours the full `RenderInstance.basis` (the 2×2 world
//! linear map, ADR-0070-amendment-4): [`pick_sprite_at_world`] inverts
//! the basis to bring the cursor into the sprite's local frame and
//! tests the local quad, so rotated / scaled / skewed sprites pick
//! exactly where they render. The rect / gizmo-box paths use the exact
//! parallelogram AABB. A degenerate basis (det ≈ 0) is unpickable.
//!
//! ## Top-most resolution
//!
//! Multiple sprites can overlap the same world point. With no Z field
//! on `Sprite` today we approximate "top-most" by **last hit in
//! iteration order** — bevy_ecs walks each archetype in insertion
//! order, so within a single archetype the latest spawn wins. Cross-
//! archetype the order is implementation-defined, but the editor's
//! demo content tends to share one archetype (Transform + Sprite +
//! optional Name), so this is good enough until a real Z/layer field
//! lands. Once `Sprite` carries Z the tiebreak switches to that.

use crate::SpriteMesh;
use crate::sprite::RenderInstance;
use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, SimRef};

/// World-space axis-aligned bounding box. Min/max in meters.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldBbox {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl WorldBbox {
    /// `(center_x, center_y, half_width, half_height)` derived from
    /// min/max. Used by the gizmo painter to position handles.
    pub fn center_half(&self) -> ([f32; 2], [f32; 2]) {
        let cx = (self.min[0] + self.max[0]) * 0.5;
        let cy = (self.min[1] + self.max[1]) * 0.5;
        let hw = (self.max[0] - self.min[0]) * 0.5;
        let hh = (self.max[1] - self.min[1]) * 0.5;
        ([cx, cy], [hw, hh])
    }

    /// True when `point` lies inside (or on the boundary of) the box.
    pub fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
    }
}

// ─── basis geometry helpers (ADR-0070-amendment-4) ──────────────────
//
// `RenderInstance.basis` is the 2x2 world linear map (rotation + scale +
// skew), column-major `[col0.x, col0.y, col1.x, col1.y]`, i.e. matrix
// `M = [[b0, b2], [b1, b3]]`. `size`/`anchor` are now LOCAL, so picking
// inverts `M` to test the world cursor against the local quad, and uses
// the exact parallelogram AABB for the rubber-band / gizmo box.

/// Map a WORLD-space delta (cursor − pivot) into the sprite's LOCAL
/// frame by inverting `basis`. `None` when the basis is degenerate.
#[inline]
fn world_delta_to_local(basis: [f32; 4], dx: f32, dy: f32) -> Option<(f32, f32)> {
    let [b0, b1, b2, b3] = basis;
    let det = b0 * b3 - b1 * b2;
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    Some(((b3 * dx - b2 * dy) * inv, (-b1 * dx + b0 * dy) * inv))
}

/// Map a LOCAL point through `basis` to a WORLD delta from the pivot.
#[inline]
fn basis_apply(basis: [f32; 4], x: f32, y: f32) -> (f32, f32) {
    let [b0, b1, b2, b3] = basis;
    (b0 * x + b2 * y, b1 * x + b3 * y)
}

/// World-space AABB half-extents of a centered local quad (half-sizes
/// `half_w`/`half_h`) under `basis` — the exact bbox of the transformed
/// parallelogram (sum of absolute column contributions).
#[inline]
fn world_aabb_half_extents(basis: [f32; 4], half_w: f32, half_h: f32) -> (f32, f32) {
    let [b0, b1, b2, b3] = basis;
    (
        b0.abs() * half_w + b2.abs() * half_h,
        b1.abs() * half_w + b3.abs() * half_h,
    )
}

// ─── what an instance DRAWS: the quad, or the mesh of a skinned sprite ─────
//
// ⭐⭐ Plano `docs/Skeleton/03`, W3: a sprite drawn as a [`SpriteMesh`] (an image bound to a
// skeleton) is drawn where its POSED triangles are, not where its quad rests — so every pick path
// and every box below asks the renderer's own rule (`sprite_mesh::drawn_mesh`) which of the two it
// is. ⛔ A second rule here would make a sprite pickable where nothing is drawn.

/// Does the LOCAL point (relative to the pivot) land on what this instance draws?
fn covers_local(ri: &RenderInstance, mesh: Option<&SpriteMesh>, local: (f32, f32)) -> bool {
    if let Some(m) = crate::sprite_mesh::drawn_mesh(mesh, ri.size) {
        return crate::sprite_mesh::covers(m, [local.0, local.1]);
    }
    // The quad center sits at `anchor` in the local frame — `world_pos` is the pivot, not
    // necessarily the center — so the bbox spans `[anchor - half, anchor + half]`. anchor [0,0]
    // (every legacy sprite) collapses to the original centered test.
    (local.0 - ri.anchor[0]).abs() <= ri.size[0] * 0.5
        && (local.1 - ri.anchor[1]).abs() <= ri.size[1] * 0.5
}

/// The exact world AABB of what this instance draws — the posed mesh's vertices through the basis,
/// or the transformed quad (rotation + scale + skew), centered on `pivot + basis·anchor`.
fn drawn_world_bbox(pos: [f32; 2], ri: &RenderInstance, mesh: Option<&SpriteMesh>) -> WorldBbox {
    if let Some(m) = crate::sprite_mesh::drawn_mesh(mesh, ri.size) {
        let mut b = WorldBbox {
            min: [f32::INFINITY; 2],
            max: [f32::NEG_INFINITY; 2],
        };
        for t in m.triangles() {
            for p in crate::sprite_mesh::corners(m, t) {
                let (wx, wy) = basis_apply(ri.basis, p[0], p[1]);
                let w = [pos[0] + wx, pos[1] + wy];
                b.min = [b.min[0].min(w[0]), b.min[1].min(w[1])];
                b.max = [b.max[0].max(w[0]), b.max[1].max(w[1])];
            }
        }
        return b;
    }
    let (half_w, half_h) = world_aabb_half_extents(ri.basis, ri.size[0] * 0.5, ri.size[1] * 0.5);
    let (ax, ay) = basis_apply(ri.basis, ri.anchor[0], ri.anchor[1]);
    let (cx, cy) = (pos[0] + ax, pos[1] + ay);
    WorldBbox {
        min: [cx - half_w, cy - half_h],
        max: [cx + half_w, cy + half_h],
    }
}

/// Return every sprite whose DRAWN shape (its oriented quad, or its posed mesh) contains
/// `world_pos`, ordered top → bottom. Top = last in archetype iteration order
/// (= most recently spawned within an archetype, which matches the
/// "topmost = last hit" heuristic used by [`pick_sprite_at_world`]).
///
/// Used by the alternate-click cycling UX (M14.7 polish 19.3): the
/// host caches this list per click position and walks it on every
/// other click so the user can step down the stack at a single spot
/// without losing their cursor location.
pub fn pick_sprites_at_world(present: &mut World, world_pos: [f32; 2]) -> Vec<u64> {
    let mut hits: Vec<u64> = Vec::new();
    let mut q = present.query::<(
        &SimRef,
        &GlobalTransform,
        &RenderInstance,
        Option<&SpriteMesh>,
    )>();
    for (sim_ref, gt, ri, mesh) in q.iter(present) {
        let pos = gt.translation();
        // Invert the basis to bring the world cursor into the sprite's local frame, so the
        // test honours rotation + scale + skew.
        let Some(local) =
            world_delta_to_local(ri.basis, world_pos[0] - pos.x, world_pos[1] - pos.y)
        else {
            continue;
        };
        if covers_local(ri, mesh, local) {
            hits.push(sim_ref.0.to_bits());
        }
    }
    // Reverse → top-of-pile (last spawned) ends up at index 0.
    hits.reverse();
    hits
}

/// Fase 0f: return every sprite whose DRAWN world bbox intersects the rect
/// spanning `rect_min`..`rect_max`. Used by the canvas rubber-band
/// box-select gesture. The bbox is the exact AABB of the transformed
/// parallelogram (rotation + scale + skew via the basis), or of the posed mesh.
/// The order of returned bits matches archetype iteration; the caller deduplicates.
pub fn pick_sprites_in_world_rect(
    present: &mut World,
    rect_min: [f32; 2],
    rect_max: [f32; 2],
) -> Vec<u64> {
    let mut hits: Vec<u64> = Vec::new();
    let mut q = present.query::<(
        &SimRef,
        &GlobalTransform,
        &RenderInstance,
        Option<&SpriteMesh>,
    )>();
    for (sim_ref, gt, ri, mesh) in q.iter(present) {
        let pos = gt.translation();
        let b = drawn_world_bbox([pos.x, pos.y], ri, mesh);
        if b.max[0] >= rect_min[0]
            && b.min[0] <= rect_max[0]
            && b.max[1] >= rect_min[1]
            && b.min[1] <= rect_max[1]
        {
            hits.push(sim_ref.0.to_bits());
        }
    }
    hits
}

/// Return the sim-entity bits of the topmost sprite whose DRAWN shape contains
/// `world_pos`. Returns `None` when no sprite covers the
/// point (e.g. the user clicked empty canvas) — the host treats this
/// as a deselect.
///
/// Walks every `(SimRef, GlobalTransform, RenderInstance)` triple in
/// `present`. RenderInstance carries the already-extracted size and
/// matches exactly what the renderer painted on the screen, so a
/// click that visually lands on the sprite is guaranteed to pick.
pub fn pick_sprite_at_world(present: &mut World, world_pos: [f32; 2]) -> Option<u64> {
    let mut best: Option<u64> = None;
    let mut q = present.query::<(
        &SimRef,
        &GlobalTransform,
        &RenderInstance,
        Option<&SpriteMesh>,
    )>();
    for (sim_ref, gt, ri, mesh) in q.iter(present) {
        let pos = gt.translation();
        // Invert the 2x2 basis to bring the cursor delta into the sprite's
        // local frame so the test matches what the user sees — honouring
        // rotation + scale + skew (ADR-0070-amendment-4). A degenerate basis
        // (det~0) can't be hit.
        let Some(local) =
            world_delta_to_local(ri.basis, world_pos[0] - pos.x, world_pos[1] - pos.y)
        else {
            continue;
        };
        if covers_local(ri, mesh, local) {
            // Last hit wins — within an archetype bevy_ecs walks in
            // insertion order, so the most recently spawned sprite
            // overrides earlier ones (intuitive "top of the pile").
            best = Some(sim_ref.0.to_bits());
        }
    }
    best
}

/// Map a WORLD-space point to the selected sprite's texture **UV** (`0..1`), honouring
/// the full `RenderInstance.basis` (rotation + scale + skew) + `anchor` — so a click on a
/// MOVED / ROTATED / SCALED sprite lands on the texel that visually sits under the cursor
/// (the same inversion `pick_sprite_at_world` uses, so input and render agree exactly).
///
/// `None` when the entity is absent, the basis is degenerate, the size is non-positive, OR
/// the point is outside the quad (`u`/`v` ∉ `[0, 1)` — exclusive high side, matching the
/// painter's texel grid: `u = 1.0` would map one texel past the last column). UV (0,0) is
/// the top-left of the texture (world +Y up ⇒ `v` grows downward), matching the renderer's
/// texture orientation. `world_pos` is the cursor mapped through
/// [`Camera2d::screen_to_world`](crate::camera::Camera2d::screen_to_world).
///
/// ⭐⭐ **A skinned sprite** (drawn as a [`SpriteMesh`], plan `docs/Skeleton/03` W3): the UV is the
/// REST UV under the posed mesh, and a point off the mesh is `None` — its rest quad is not drawn.
pub fn sprite_world_to_uv(
    present: &mut World,
    sim_entity_bits: u64,
    world_pos: [f32; 2],
) -> Option<(f32, f32)> {
    let UvHit {
        uv: (u, v),
        on_mesh,
        ..
    } = uv_query(present, sim_entity_bits, world_pos, [0.0, 0.0])?;
    let drawn_here = on_mesh.unwrap_or(true);
    // matched the entity, but the cursor is off what it draws
    (drawn_here && (0.0..1.0).contains(&u) && (0.0..1.0).contains(&v)).then_some((u, v))
}

/// Like [`sprite_world_to_uv`] but WITHOUT the `[0, 1)` quad-containment gate:
/// returns the UV even when the cursor sits OFF the sprite quad (`u`/`v` may be
/// negative or `≥ 1`). Still `None` on an absent entity, a degenerate basis, or a
/// non-positive size.
///
/// The Painter uses this so the WHOLE sprite stays active for painting regardless
/// of where the quad edge falls relative to the viewport: a stroke that reaches /
/// crosses the edge keeps the wash simulating instead of dropping the segment
/// (the old `[0,1)` gate made an edge-grazing or off-screen drag `None`, which
/// broke the stroke and stalled the live wet field). Out-of-quad samples deposit
/// nothing harmful downstream — the dab envelope clamps to the canvas grid and
/// the splat's radius cutoff rejects a dab whose footprint misses every cell.
///
/// ⚠️ **A skinned sprite, off its posed mesh, is answered by the REST quad law** — a named limit
/// (plan `docs/Skeleton/03` W3): a stroke that leaves the posed silhouette is mapped as if the image
/// rested. On the mesh it is the rest UV under the posed triangle.
pub fn sprite_world_to_uv_unclamped(
    present: &mut World,
    sim_entity_bits: u64,
    world_pos: [f32; 2],
) -> Option<(f32, f32)> {
    uv_query(present, sim_entity_bits, world_pos, [0.0, 0.0]).map(|h| h.uv)
}

/// ⭐⭐⭐ **O que uma PORTA DE CANVAS deve fazer sobre uma sprite desenhada como MALHA.**
///
/// ⛔⛔ **A lei existia em duas portas e o consumidor usava uma TERCEIRA** (medido 2026-09-14): o
/// [`sprite_world_to_uv`] e o [`sprite_world_to_uv_unclamped`] conhecem a malha desde a W3 do plano
/// `docs/Skeleton/03` — e tinham **zero** chamadores de produto. O Painter mapeia o ponteiro pelo
/// afim do QUAD DE REPOUSO (`ph2d_sprite_screen::sprite_image_to_screen_affine`), que não sabe o que
/// é uma malha, logo numa arte presa e DOBRADA a pincelada cai deslocada exactamente pela
/// deformação — em toda a arte, não só fora dela. *Duas portas com a lei certa e ninguém as
/// chamava.*
///
/// A resposta é um enum de três estados porque o chamador tem três coisas diferentes a fazer, e
/// nenhuma delas é «um `Option` com um `if` ao lado»:
///
/// - [`Self::Quad`] — **não é uma malha**: a lei do chamador continua **intocada** (e para o Painter
///   isso importa: o afim dele carrega a grelha da folha, o *Repeat Image* e a margem do gizmo de
///   deformação, que esta porta não conhece).
/// - [`Self::Use`] — a UV de REPOUSO do texel sob o ponto, lida do triângulo posado.
/// - [`Self::Refuse`] — fora da arte desenhada, e o gesto está a COMEÇAR: um traço não nasce sobre
///   um quad que não se desenha. ⚠️ Com o gesto **já aberto** a porta devolve [`Self::Use`] com a UV
///   do quad de repouso — é a lei que o `_unclamped` já escrevia, e é o que deixa uma pincelada
///   sair da silhueta sem se partir.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MeshUv {
    /// A sprite não é desenhada como malha — o chamador fica com a lei dele.
    Quad,
    /// A UV de repouso a usar, com a **deformação local** ali ([`crate::MeshWarp`]: a `2×2`
    /// adimensional, identidade em repouso, **mais a curvatura** da dobra) — o que o pincel precisa
    /// para sair redondo no ECRÃ e não na textura.
    Use {
        u: f32,
        v: f32,
        warp: crate::MeshWarp,
    },
    /// Fora da arte desenhada, num gesto que COMEÇA.
    Refuse,
}

/// Ver [`MeshUv`]. `starting` = este é o primeiro ponto do gesto (um `Down`).
///
/// ⭐⭐⭐ **`footprint_uv` é o RAIO do dab em UV de repouso, por eixo** — e não um detalhe de
/// afinação: a deformação devolvida é a que a malha faz **sobre esse disco**, não a do ponto. Um
/// dab que cabe num triângulo recebe a deformação daquele triângulo **ao bit**; um que se estende
/// por vários recebe o melhor afim sobre o que ele cobre, que é o que o impede de sair menos
/// redondo do que sairia sem correcção nenhuma. A lei e a medição vivem na
/// [`crate::sprite_mesh_warp`]. ⛔ `[0, 0]` = *«não vou pintar»* — é o que as portas de apontar
/// passam, e ali a resposta é a de um ponto.
pub fn mesh_uv(
    present: &mut World,
    sim_entity_bits: u64,
    world_pos: [f32; 2],
    starting: bool,
    footprint_uv: [f32; 2],
) -> MeshUv {
    let Some(UvHit {
        uv: (u, v),
        on_mesh,
        warp,
    }) = uv_query(present, sim_entity_bits, world_pos, footprint_uv)
    else {
        return MeshUv::Quad;
    };
    match on_mesh {
        None => MeshUv::Quad,
        Some(true) => MeshUv::Use { u, v, warp },
        Some(false) if starting => MeshUv::Refuse,
        // ⚠️ Fora da malha a deformação é a do QUAD (identidade): o traço já aberto continua com a
        // forma que o artista pediu, que é o que o deixa sair da silhueta sem se partir.
        Some(false) => MeshUv::Use {
            u,
            v,
            warp: crate::MeshWarp::rest(),
        },
    }
}

/// A resposta de [`uv_query`] — nomeada porque são TRÊS coisas e um tuplo delas lê-se por posição.
struct UvHit {
    uv: (f32, f32),
    /// `Some(false)` = fora da malha desenhada (respondido pela lei do quad); `None` num quad simples.
    on_mesh: Option<bool>,
    /// A deformação local ali (identidade fora de uma malha ou em repouso).
    warp: crate::MeshWarp,
}

/// The UV under `world_pos` on sprite `sim_entity_bits`, and — for a skinned sprite — whether the
/// point is ON its drawn mesh (`Some(false)` = off it, answered by the rest quad law). `None` in
/// the second slot for a plain quad.
fn uv_query(
    present: &mut World,
    sim_entity_bits: u64,
    world_pos: [f32; 2],
    footprint_uv: [f32; 2],
) -> Option<UvHit> {
    let mut q = present.query::<(
        &SimRef,
        &GlobalTransform,
        &RenderInstance,
        Option<&SpriteMesh>,
    )>();
    for (sim_ref, gt, ri, mesh) in q.iter(present) {
        if sim_ref.0.to_bits() != sim_entity_bits {
            continue;
        }
        let pos = gt.translation();
        // Invert the 2×2 basis to bring the world cursor into the sprite's LOCAL frame —
        // this is what makes rotation / scale / skew correct (vs the old translation-only
        // AABB). Degenerate basis (det≈0) ⇒ unpaintable.
        let (local_dx, local_dy) =
            world_delta_to_local(ri.basis, world_pos[0] - pos.x, world_pos[1] - pos.y)?;
        let (sw, sh) = (ri.size[0], ri.size[1]);
        if sw <= 0.0 || sh <= 0.0 {
            return None;
        }
        let malha = crate::sprite_mesh::drawn_mesh(mesh, ri.size);
        if let Some(m) = malha
            && let Some(uv) = crate::sprite_mesh::uv_under(m, [local_dx, local_dy])
        {
            let warp =
                crate::sprite_mesh_warp::warp_over(m, [local_dx, local_dy], ri.size, footprint_uv)
                    .unwrap_or(crate::MeshWarp::rest());
            return Some(UvHit {
                uv: (uv[0], uv[1]),
                on_mesh: Some(true),
                warp,
            });
        }
        // The quad center sits at `anchor` in the local frame. `u` grows with local +X
        // (right edge), `v` with local −Y (so the top edge = `v=0`, matching the texture +
        // the old axis-aligned mapping for an un-rotated sprite).
        let u = (local_dx - ri.anchor[0]) / sw + 0.5;
        let v = 0.5 - (local_dy - ri.anchor[1]) / sh;
        return Some(UvHit {
            uv: (u, v),
            on_mesh: malha.map(|_| false),
            warp: crate::MeshWarp::rest(),
        });
    }
    None
}

/// Look up the world-space bbox of the sprite currently selected by
/// the editor — the box of what it DRAWS (its transformed quad, or the posed
/// mesh of a skinned sprite). Returns `None` when the entity no longer exists in
/// PresentWorld (e.g. it was despawned this frame) — callers treat
/// that as "no selection to draw the gizmo over".
pub fn selection_bbox_world(present: &mut World, sim_entity_bits: u64) -> Option<WorldBbox> {
    let mut q = present.query::<(
        &SimRef,
        &GlobalTransform,
        &RenderInstance,
        Option<&SpriteMesh>,
    )>();
    for (sim_ref, gt, ri, mesh) in q.iter(present) {
        if sim_ref.0.to_bits() == sim_entity_bits {
            let pos = gt.translation();
            return Some(drawn_world_bbox([pos.x, pos.y], ri, mesh));
        }
    }
    None
}

/// ⭐⭐ **The world box around every sprite AS DRAWN** — each instance's quad (anchor and basis
/// included) or posed mesh. `None` for a scene with no sprite. It is what *View All* frames.
///
/// ⚠️ **It lives here, beside the picking boxes, and not in the shell:** the shell's View All used to
/// rebuild a pivot-centred quad by hand, which ignored the anchor, the basis and (since the W3 of
/// plan `docs/Skeleton/03`) the mesh — a second box law that disagreed with the gizmo's.
pub fn scene_sprites_bbox_world(present: &mut World) -> Option<WorldBbox> {
    let mut q = present.query::<(&GlobalTransform, &RenderInstance, Option<&SpriteMesh>)>();
    let mut all: Option<WorldBbox> = None;
    for (gt, ri, mesh) in q.iter(present) {
        let pos = gt.translation();
        let b = drawn_world_bbox([pos.x, pos.y], ri, mesh);
        all = Some(match all {
            None => b,
            Some(a) => WorldBbox {
                min: [a.min[0].min(b.min[0]), a.min[1].min(b.min[1])],
                max: [a.max[0].max(b.max[0]), a.max[1].max(b.max[1])],
            },
        });
    }
    all
}

/// ⭐⭐⭐ **A ARTE COMO ELA É DESENHADA, para quem desenha POR CIMA dela** — o gémeo da
/// [`mesh_uv`], na direcção que faltava.
///
/// ⛔⛔ **A [`mesh_uv`] curou quem APONTA e deixou quem PINTA CHROME a mapear pelo quad de
/// repouso** (medido 2026-09-15): o editor de curva do Painter resolve o clique pela malha desde a
/// wave anterior e continuava a desenhar os pontos de controlo pelo afim do quad — *o artista
/// clicava num sítio e o ponto aparecia noutro*. ⚠️ **Um controlo desenhado por um mapa e agarrado
/// por outro é um controlo morto sob o dedo**, que é a espécie do §5.0 que nenhuma sonda vê.
///
/// ⚠️ **Ela é READ-ONLY de propósito** (`&World`, não `&mut`): quem desenha o quadro já tem o mundo
/// de apresentação emprestado, e um `query::<…>()` por chamada **aloca** as tabelas de arquétipo
/// dele (a nota do [`ph2d_ecs::PresentWorld::sweep`] mediu 107 blocos / 10 quadros por isso). Aqui
/// atravessa-se o mundo uma vez, guarda-se a malha EMPRESTADA, e cada ponto é uma pergunta sem
/// alocação nenhuma.
///
/// ⚠️ **O custo por ponto é o número de TRIÂNGULOS** (varrimento linear, como a pergunta inversa):
/// é o preço que a `Smooth` paga, e o consumidor de hoje (o editor de curva) pergunta por dezenas
/// de pontos enquanto uma curva está a ser editada. Se um consumidor de MILHARES aparecer, a cura é
/// um índice por UV construído aqui — e não uma segunda cópia desta álgebra.
#[derive(Clone, Copy)]
pub struct DrawnMesh<'a> {
    mesh: &'a SpriteMesh,
    basis: [f32; 4],
    pos: [f32; 2],
}

impl DrawnMesh<'_> {
    /// **UV de repouso → MUNDO**, pelo triângulo POSADO que contém aquele texel. `None` fora da
    /// malha desenhada — e aí o chamador fica com a lei do quad, exactamente como a
    /// [`MeshUv::Quad`] do outro lado.
    #[must_use]
    pub fn world_at_uv(&self, uv: [f32; 2]) -> Option<[f32; 2]> {
        let l = crate::sprite_mesh::local_at_uv(self.mesh, uv)?;
        let (dx, dy) = basis_apply(self.basis, l[0], l[1]);
        Some([self.pos[0] + dx, self.pos[1] + dy])
    }
}

/// ⭐⭐ **A INSTÂNCIA desta sprite e a malha dela, para quem quer desenhar OUTRA COISA no mesmo
/// sítio** — a tinta da máscara de protecção da Remoção de fundo é a primeira.
///
/// ⛔⛔ **Ela nasceu porque a alternativa está MEDIDA e REFUTADA:** desenhar a tinta por cima da
/// arte dobrada com **um recorte do Vello por triângulo** deixa costuras (`10 580` px fora da barra
/// numa arte translúcida, e dilatar os recortes **piora** — a faixa sobreposta compõe-se duas
/// vezes: `55 978` px) e, pior, os buffers do Vello são de tamanho FIXO e o que os estoura **degrada
/// em silêncio** — que foi exactamente o *«Smooth bugado quebrando a forma»* deste módulo.
/// ⇒ a tinta entra no passe de SPRITES como uma instância a mais, com a MESMA malha: sem costuras
/// por construção (a regra de canto do rasterizador dá cada centro de pixel a UM triângulo).
///
/// `None` quando a entidade não é uma sprite desenhada. A malha é `None` num quad simples.
#[must_use]
pub fn drawn_instance_of(
    present: &World,
    sim_entity_bits: u64,
) -> Option<(&RenderInstance, Option<&SpriteMesh>)> {
    for e in present.iter_entities() {
        let Some(sim_ref) = e.get::<SimRef>() else {
            continue;
        };
        if sim_ref.0.to_bits() != sim_entity_bits {
            continue;
        }
        let ri = e.get::<RenderInstance>()?;
        return Some((
            ri,
            crate::sprite_mesh::drawn_mesh(e.get::<SpriteMesh>(), ri.size),
        ));
    }
    None
}

/// A malha POSADA desta sprite da simulação, ou `None` quando ela não é desenhada como malha (e aí
/// o chamador fica com o afim do quad de repouso, intocado). Ver [`DrawnMesh`].
#[must_use]
pub fn drawn_mesh_of(present: &World, sim_entity_bits: u64) -> Option<DrawnMesh<'_>> {
    for e in present.iter_entities() {
        let Some(sim_ref) = e.get::<SimRef>() else {
            continue;
        };
        if sim_ref.0.to_bits() != sim_entity_bits {
            continue;
        }
        let (Some(gt), Some(ri)) = (e.get::<GlobalTransform>(), e.get::<RenderInstance>()) else {
            return None;
        };
        // ⚠️ A MESMA pergunta que o passe de sprites faz (`drawn_mesh`): uma malha que este mapa
        // lesse e o desenho recusasse punha o chrome sobre uma dobra que não está no ecrã.
        let mesh = crate::sprite_mesh::drawn_mesh(e.get::<SpriteMesh>(), ri.size)?;
        let pos = gt.translation();
        return Some(DrawnMesh {
            mesh,
            basis: ri.basis,
            pos: [pos.x, pos.y],
        });
    }
    None
}

#[cfg(test)]
#[path = "picking_tests.rs"]
mod tests;
