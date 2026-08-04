//! **Baking engine VECTORS to a tile** (doc 86 §2, Wave A2) — the shell half of
//! `source.object` for a vector shape.
//!
//! A sprite already IS a tile (its atlas cell); `motion_bridge_objects` publishes
//! it directly. A vector is a curve — to be *stamped* by the `motion.duplicator`
//! it must become a quad, so it is **rasterized once into an offscreen texture**
//! and that texture's id rides the stream like any sprite's. This is the
//! bake-to-tile of doc 86 §2, and the machinery is the FX raster stack's
//! (`fx_live`) verbatim: `draw_path_isolated → VelloPass → an individual
//! texture`, the difference being the DESTINATION (the sprite renderer's
//! `IndividualTextureStore`, not the Vello atlas).
//!
//! ## Three decisions
//!
//! **1. A FIXED bake camera — the tile is camera-independent.** The FX stack
//! bakes in SCREEN space (it is a screen-space effect), so it re-bakes on every
//! zoom. A stamped tile must not: it is baked at a fixed DPI (`Affine::scale`),
//! so the tile is the shape's drawing at a crisp constant resolution, and zoom
//! never touches it. Its world size is the world bbox, so a stamp lands at the
//! shape's natural size.
//!
//! **2. Cached by CONTENT — a static scene bakes once (steady-state free).** The
//! key is the FX-memo pattern (`fx_live_memo::FxKey`): the AUTHORED `VecPath`
//! (LOCAL geometry + style + effects — translation-invariant, ADR-0111) + the
//! LINEAR part of the world transform (a rotate/scale re-bakes; a MOVE does not,
//! since the tile is bbox-normalized) + the DPI. A new `VecPath` field travels
//! into the key by itself. ⚠️ Every `acquire` is paired with a `release`
//! (`IndividualTextureStore` is refcounted): re-bake and vanish both free the old
//! texture, so the tiles don't leak VRAM.
//!
//! **3. LIVE by default (doc 86 §2 tier) — the tile follows the drawing.** The
//! bake re-runs whenever the content-key changes, so editing the shape restamps
//! it; the per-node FREEZE toggle (bake once, never again) is a later step.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_gpu::GpuContext;
use ph2d_render::{SpriteRenderer, VelloPass};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, xform_of};
use ph2d_vector::{Affine, VectorScene};

use crate::vec_entities::VecEntityMap;

/// Bake resolution, pixels per world unit. Fixed (camera-INDEPENDENT) so zoom
/// never re-bakes. 256 px/unit is crisp for a stamped tile at typical sizes;
/// the world size the sink stamps is the bbox, so this is only the tile's inner
/// resolution, not its on-screen scale.
pub(crate) const BAKE_DPI: f64 = 256.0;

/// Cap on a tile side (px) — a VRAM + GPU-limit guard. A shape whose world bbox
/// would exceed this at `BAKE_DPI` is clamped to it (a coarser effective DPI),
/// never refused: a huge tile is a perf choice, not a correctness limit.
pub(crate) const MAX_TILE_SIDE: u32 = 2048;

/// What a baked vector is on the render side: the individual `texture_id` + the
/// tile's WORLD size (so the sink stamps it at the shape's natural size).
struct Baked {
    /// The artist's name for this shape, if any. **Metadata, not the cache key** —
    /// the cache is keyed by [`VecPathId`] (undo/rename-stable), so a rename refreshes
    /// this field without re-baking, and an UNNAMED group child (`None`) still gets a
    /// tile. `tiles()` (the individual publish) yields only the named ones.
    name: Option<String>,
    key: BakeKey,
    texture_id: u32,
    size: [f32; 2],
    /// A mini-render of the tile for the source node's card preview (doc 86 A5),
    /// downsampled once at bake ⇒ cached like everything else here.
    thumb: ph2d_panel_motion_graph::PreviewThumb,
}

/// The change-detector (the `fx_live_memo::FxKey` pattern). Compared by
/// EQUALITY, not hashed, so a new `VecPath` field is caught without touching
/// this struct. `path` is the LOCAL authored geometry (ADR-0111 — moving the
/// shape does not change it); `linear` is the world transform's rotation/scale/
/// skew (translation excluded, because the tile is bbox-normalized).
#[derive(Clone, PartialEq, Debug)]
struct BakeKey {
    path: VecPath,
    linear: [f64; 4],
    dpi_q: u32,
}

/// One named vector's published tile, read by the membrane.
pub(crate) struct BakedTile {
    pub texture_id: u32,
    pub size: [f32; 2],
}

/// The bake cache + its own scratch `VelloPass` (a dedicated offscreen renderer,
/// like `fx_live`'s — never the main frame pass). `Default` is empty (no scratch
/// until the first bake); `Option<VelloPass>` defaults to `None` regardless of
/// whether `VelloPass` is `Default`.
#[derive(Default)]
pub(crate) struct ObjectBake {
    scratch: Option<VelloPass>,
    /// Keyed by the shape's own [`VecPathId`] (undo/rename-stable), NOT its name — so
    /// an unnamed group child gets a tile and a rename doesn't evict it.
    cache: BTreeMap<VecPathId, Baked>,
}

impl ObjectBake {
    /// The `name -> tile` map the membrane publishes individually (the picker path).
    /// Read-only — the bake ran at the fx phase; here the membrane only reads results.
    /// Only the NAMED entries are yielded: an unnamed group child has a tile (for the
    /// group stamp) but nothing to type into a node. On a name clash the higher id wins
    /// the `set_external` (id-ascending iteration ⇒ later), as the name-keyed map did.
    pub(crate) fn tiles(&self) -> impl Iterator<Item = (&str, BakedTile)> {
        self.cache.values().filter_map(|b| {
            b.name.as_deref().map(|n| {
                (
                    n,
                    BakedTile {
                        texture_id: b.texture_id,
                        size: b.size,
                    },
                )
            })
        })
    }

    /// The baked tile for ONE shape by its [`VecPathId`], or `None` if it isn't baked
    /// (doc 86 §2 A4). A group child that is a vector resolves its appearance through
    /// this — by its drawing id, so an unnamed child still resolves.
    pub(crate) fn tile_for_id(&self, id: VecPathId) -> Option<BakedTile> {
        self.cache.get(&id).map(|b| BakedTile {
            texture_id: b.texture_id,
            size: b.size,
        })
    }

    /// Seed a fake tile under `id` — for headless membrane gates that drive
    /// `resolve_leaf` without a GPU bake.
    #[cfg(test)]
    pub(crate) fn seed_for_test(&mut self, id: VecPathId, texture_id: u32, size: [f32; 2]) {
        self.cache.insert(
            id,
            Baked {
                name: None,
                key: BakeKey {
                    path: VecPath::default(),
                    linear: [0.0; 4],
                    dpi_q: 0,
                },
                texture_id,
                size,
                thumb: ph2d_panel_motion_graph::PreviewThumb {
                    rgba: std::sync::Arc::new(Vec::new()),
                    w: 0,
                    h: 0,
                },
            },
        );
    }

    /// The baked-tile THUMBNAIL for `texture_id`, or `None` (doc 86 A5). The membrane
    /// hands it to the source node's snapshot so the card shows a mini-render of the
    /// object it references instead of a single origin dot. O(cache) — a handful of
    /// named shapes, read once a frame; the tid comes from the stream's own column.
    pub(crate) fn thumbnail_for(
        &self,
        texture_id: u32,
    ) -> Option<ph2d_panel_motion_graph::PreviewThumb> {
        self.cache
            .values()
            .find(|b| b.texture_id == texture_id)
            .map(|b| b.thumb.clone())
    }

    /// Re-bake every named vector whose drawing changed; evict + RELEASE the
    /// texture of any name that vanished. Runs at the fx phase, where every
    /// handle (`renderer`, `gpu`, the vector scene + transforms + live geometry)
    /// is in hand. Cached by content ⇒ a static scene bakes once.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bake(
        &mut self,
        scene: &VecScene,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &LiveGeometry,
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        renderer: &mut SpriteRenderer,
        sim: &SimWorld,
    ) {
        // The drawings to tile this frame, keyed by VecPathId → the artist's name
        // (`None` for an unnamed group child): named ∪ group-referenced (see
        // `select_present`). Keying by id — not name — is what lets an unnamed group
        // child get a tile and a rename keep it.
        let world = sim.world();
        let present = select_present(world, map);

        // Evict the ids that vanished (deleted, or an unnamed shape no group references
        // any more), releasing their texture so a tile never leaks VRAM.
        let gone: Vec<VecPathId> = self
            .cache
            .keys()
            .filter(|id| !present.contains_key(*id))
            .copied()
            .collect();
        for id in gone {
            if let Some(b) = self.cache.remove(&id) {
                renderer.individual_mut().release(b.texture_id);
            }
        }

        // Bake the new + changed. A cache hit is a `VecPath`/xform equality — no
        // GPU work, no readback.
        for (id, name) in present {
            let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            let c = xform_of(xforms, id).0;
            let key = BakeKey {
                path: path.clone(),
                linear: [c[0], c[1], c[2], c[3]],
                dpi_q: BAKE_DPI as u32,
            };
            if self.cache.get(&id).is_some_and(|b| b.key == key) {
                // Content unchanged — refresh the (metadata) NAME so a rename
                // re-publishes without a re-bake, and continue.
                if let Some(b) = self.cache.get_mut(&id) {
                    b.name = name;
                }
                continue;
            }
            // Changed or new: release the previous texture (if any) and re-bake.
            let old = self.cache.remove(&id).map(|b| b.texture_id);
            let baked = bake_one(
                &mut self.scratch,
                scene,
                xforms,
                live,
                id,
                gpu,
                surface_format,
                renderer,
            );
            if let Some(t) = old {
                renderer.individual_mut().release(t);
            }
            if let Some((texture_id, size, thumb)) = baked {
                self.cache.insert(
                    id,
                    Baked {
                        name,
                        key,
                        texture_id,
                        size,
                        thumb,
                    },
                );
            }
        }
    }
}

/// The vector drawings to bake this frame, keyed by [`VecPathId`] → the artist's name
/// (`None` for an unnamed group child). A drawing is baked iff it is **named** (the
/// picker path — the artist can type it into a node) OR its entity is inside a **named
/// group** ([`entity_is_in_a_named_group`], so the group stamp has its tile). Unnamed
/// canvas art that no group references is NOT baked ⇒ no wasted VRAM. Pure ECS, so the
/// selection — the load-bearing "which drawings" decision — is a headless gate.
fn select_present(
    world: &ph2d_ecs::World,
    map: &VecEntityMap,
) -> BTreeMap<VecPathId, Option<String>> {
    use crate::render_loop::motion_bridge::entity_is_in_a_named_group;
    let mut present: BTreeMap<VecPathId, Option<String>> = BTreeMap::new();
    for (&id, &bits) in map {
        let entity = Entity::from_bits(bits);
        if world.get_entity(entity).is_err() {
            continue; // stale bits (despawned): its tile is evicted, not baked
        }
        let name = world
            .get::<Name>(entity)
            .map(|n| n.0.clone())
            .filter(|s| !s.trim().is_empty());
        if name.is_none() && !entity_is_in_a_named_group(world, entity) {
            continue; // unnamed AND no group references it — nothing needs its tile
        }
        present.insert(id, name);
    }
    present
}

/// Bake ONE path into a fresh individual texture; returns `(texture_id, world
/// size)`, or `None` if the shape has no drawable bounds or a GPU step failed
/// (skipped, not guessed — the same shape a not-yet-resolvable sprite takes).
#[allow(clippy::too_many_arguments)]
fn bake_one(
    scratch: &mut Option<VelloPass>,
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    gpu: &GpuContext,
    surface_format: wgpu::TextureFormat,
    renderer: &mut SpriteRenderer,
) -> Option<(u32, [f32; 2], ph2d_panel_motion_graph::PreviewThumb)> {
    // A fixed-DPI "camera": world units → tile pixels. NOT the live camera, so
    // the tile is zoom-independent. Bounds honour the stroke half-width + miter.
    let camera = Affine::scale(BAKE_DPI);
    let (x0, y0, x1, y1) = ph2d_vec_render::path_screen_bounds(scene, xforms, live, id, camera)?;
    let wpx = ((x1 - x0).ceil() as u32).clamp(1, MAX_TILE_SIDE);
    let hpx = ((y1 - y0).ceil() as u32).clamp(1, MAX_TILE_SIDE);

    // Encode the one path, translated so its bbox lands at the tile origin.
    let mut scratch_scene = VectorScene::new();
    ph2d_vec_render::draw_path_isolated(
        scene,
        xforms,
        live,
        id,
        camera,
        Affine::translate((-x0, -y0)),
        &mut scratch_scene,
    );

    // Render offscreen (a dedicated scratch pass, reused + resized across bakes)
    // and read the tightly-packed straight RGBA8 back. The readback is slow, but
    // it runs only on a content change (cached), so steady state pays nothing.
    let pass = match scratch.as_mut() {
        Some(p) => p,
        None => scratch.insert(VelloPass::new(gpu, surface_format, (wpx, hpx)).ok()?),
    };
    let rgba = pass
        .render_and_readback(gpu, scratch_scene.inner(), (wpx, hpx))
        .ok()?;
    let want = (wpx * hpx * 4) as usize;
    if rgba.len() < want {
        return None;
    }
    // The card thumbnail (doc 86 A5) comes from these SAME bytes before they are
    // dropped — one downsample per content change, cached with the tile.
    let thumb = thumbnail(&rgba[..want], wpx, hpx);
    // Upload as an individual texture — the SAME raw bytes the FX GPU→GPU copy
    // would move, so the colour behaviour matches; no Sprite is mutated.
    let texture_id = renderer.acquire_individual(wpx, hpx, &rgba[..want]).ok()?;
    let size = [
        (x1 - x0) as f32 / BAKE_DPI as f32,
        (y1 - y0) as f32 / BAKE_DPI as f32,
    ];
    Some((texture_id, size, thumb))
}

/// A small tile side (px) for a card thumbnail — big enough to read the shape,
/// small enough that the bytes ride the snapshot per frame for free (~37 KB at
/// 96²). Shared by the vector (A2) and Flip (A3) bakes.
pub(crate) const THUMB_MAX: u32 = 96;

/// Downsample straight RGBA8 (`w`×`h`) to a card thumbnail (doc 86 A5): at most
/// [`THUMB_MAX`] on its long side, aspect preserved, never upscaled. Box-average in
/// PREMULTIPLIED space (`Σ c·a / Σ a`) so a transparent edge does not bleed a dark
/// halo into the shrunk shape — the premul trap the overlay lesson names (ADR-0120
/// neighbourhood). One pass per bake; the result is cached with the tile.
pub(crate) fn thumbnail(rgba: &[u8], w: u32, h: u32) -> ph2d_panel_motion_graph::PreviewThumb {
    let (w, h) = (w.max(1), h.max(1));
    let long = w.max(h);
    let (tw, th) = if long <= THUMB_MAX {
        (w, h)
    } else {
        let s = THUMB_MAX as f32 / long as f32;
        (
            ((w as f32 * s).round() as u32).max(1),
            ((h as f32 * s).round() as u32).max(1),
        )
    };
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for oy in 0..th {
        let sy0 = oy * h / th;
        let sy1 = ((oy + 1) * h / th).max(sy0 + 1).min(h);
        for ox in 0..tw {
            let sx0 = ox * w / tw;
            let sx1 = ((ox + 1) * w / tw).max(sx0 + 1).min(w);
            let (mut sr, mut sg, mut sb, mut sa, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let i = ((sy * w + sx) * 4) as usize;
                    let a = rgba[i + 3] as u64;
                    sr += rgba[i] as u64 * a;
                    sg += rgba[i + 1] as u64 * a;
                    sb += rgba[i + 2] as u64 * a;
                    sa += a;
                    n += 1;
                }
            }
            let o = ((oy * tw + ox) * 4) as usize;
            // `sa == 0` ⇒ the block was fully transparent; leave the colour at 0 (already
            // zeroed), which is what a transparent thumbnail texel should carry.
            if let (Some(r), Some(g), Some(b)) =
                (sr.checked_div(sa), sg.checked_div(sa), sb.checked_div(sa))
            {
                out[o] = r as u8;
                out[o + 1] = g as u8;
                out[o + 2] = b as u8;
            }
            out[o + 3] = (sa / n.max(1)) as u8;
        }
    }
    ph2d_panel_motion_graph::PreviewThumb {
        rgba: std::sync::Arc::new(out),
        w: tw,
        h: th,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn star() -> VecPath {
        let mut p = ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.45);
        p.fill = Some(ph2d_vec_scene::Paint::solid(ph2d_vec_scene::Rgba8::new(
            255, 170, 40, 255,
        )));
        p
    }

    #[test]
    fn moving_the_shape_does_not_rebake_but_rotating_and_editing_do() {
        // The design decision the cache stands on (doc 86 §2): the tile is the
        // shape's DRAWING at a fixed DPI, bbox-normalized. So a MOVE (translation
        // only) must be a cache hit — the local `VecPath` and the LINEAR part of
        // the transform are unchanged; only the translation moved, and the tile
        // does not carry it. A ROTATE (linear changes) or an EDIT (path changes)
        // re-bakes. A key that folded translation in would re-bake on every drag.
        let base = BakeKey {
            path: star(),
            linear: [1.0, 0.0, 0.0, 1.0],
            dpi_q: 256,
        };
        // A move never touches the local path or the linear coeffs.
        let moved = BakeKey {
            path: star(),
            linear: [1.0, 0.0, 0.0, 1.0],
            dpi_q: 256,
        };
        assert_eq!(base, moved, "a move is a cache hit — no re-bake");
        // A rotate changes the linear part.
        let rotated = BakeKey {
            path: star(),
            linear: [0.0, 1.0, -1.0, 0.0],
            dpi_q: 256,
        };
        assert_ne!(base, rotated, "a rotate re-bakes");
        // An edit changes the authored path.
        let edited = BakeKey {
            path: ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 6, 0.45),
            linear: [1.0, 0.0, 0.0, 1.0],
            dpi_q: 256,
        };
        assert_ne!(base, edited, "editing the shape re-bakes");
    }

    /// **The A5 thumbnail is bounded and keeps aspect** (doc 86 A5). A wide opaque tile
    /// downsamples so its LONG side is `THUMB_MAX`, the 3:1 aspect survives, the bytes are
    /// tightly packed, and an opaque colour comes out unchanged. FALSIFIED by an unbounded
    /// or stretched thumbnail.
    #[test]
    fn the_thumbnail_is_bounded_and_keeps_aspect() {
        let (w, h) = (600u32, 200u32);
        let rgba = vec![255u8; (w * h * 4) as usize]; // opaque white
        let t = thumbnail(&rgba, w, h);
        assert_eq!(t.w.max(t.h), THUMB_MAX, "long side capped at THUMB_MAX");
        assert!(
            (t.w as f32 / t.h as f32 - 3.0).abs() < 0.05,
            "the 3:1 aspect is preserved"
        );
        assert_eq!(
            t.rgba.len(),
            (t.w * t.h * 4) as usize,
            "tightly packed RGBA8"
        );
        assert!(
            t.rgba.chunks(4).all(|p| p == [255, 255, 255, 255]),
            "an opaque solid colour survives the downsample"
        );
    }

    /// **A tile under the cap is never upscaled** (doc 86 A5) — the thumbnail of a small
    /// shape is the tile itself, not a blurry blow-up. FALSIFIED by scaling toward THUMB_MAX.
    #[test]
    fn a_small_tile_is_never_upscaled() {
        let (w, h) = (10u32, 8u32);
        let rgba = vec![128u8; (w * h * 4) as usize];
        let t = thumbnail(&rgba, w, h);
        assert_eq!(
            (t.w, t.h),
            (w, h),
            "under the cap the thumbnail is the tile"
        );
    }

    /// **The downsample does not bleed a dark halo into a transparent edge** (doc 86 A5).
    /// A row of alternating opaque-RED / fully-transparent pixels merges pairwise: the
    /// PREMULTIPLIED average keeps the surviving colour pure red (`Σc·a/Σa = 255`); a naive
    /// STRAIGHT average would pull it toward black (`(255+0)/2 = 127`), the premul trap the
    /// overlay lesson names. FALSIFIED by averaging straight RGBA.
    #[test]
    fn the_downsample_does_not_bleed_a_halo_into_a_transparent_edge() {
        let (w, h) = (THUMB_MAX * 2, 1u32); // 2:1 downsample merges pixel pairs
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        for x in (0..w).step_by(2) {
            let i = (x * 4) as usize;
            rgba[i..i + 4].copy_from_slice(&[255, 0, 0, 255]); // opaque red; odd stays transparent
        }
        let t = thumbnail(&rgba, w, h);
        assert_eq!(t.w, THUMB_MAX, "downsampled 2:1");
        for p in t.rgba.chunks(4) {
            assert_eq!(
                &p[0..3],
                &[255, 0, 0],
                "colour stays pure red — a straight average would darken it to 127"
            );
            assert!(
                (p[3] as i32 - 127).abs() <= 1,
                "alpha is the coverage average of the merged pair"
            );
        }
    }

    #[test]
    fn select_present_bakes_named_and_group_children_but_not_loose_art() {
        // doc 86 §9.6: the bake tiles a vector drawing iff it is NAMED (the picker path)
        // OR sits inside a named group (so the group stamp has its child's tile) — and
        // NOTHING else, so unnamed canvas art never wastes a tile (§0 VRAM). The three
        // rows are the whole decision table; two mutations each break a distinct row.
        use ph2d_ecs::{ChildOf, GroupedChildren};
        let mut sim = SimWorld::new();
        let named = sim.world_mut().spawn((Name::new("Named"),)).id();
        let group = sim
            .world_mut()
            .spawn((Name::new("Group"), GroupedChildren))
            .id();
        let child = sim.world_mut().spawn((ChildOf(group),)).id(); // UNNAMED group child
        let loose = sim.world_mut().spawn(()).id(); // unnamed, no group

        // The map is VecPathId -> entity bits (the same thing `sync` builds).
        let mut map = VecEntityMap::new();
        map.insert(10, named.to_bits());
        map.insert(20, child.to_bits());
        map.insert(30, loose.to_bits());

        let present = select_present(sim.world(), &map);

        assert_eq!(
            present.get(&10),
            Some(&Some("Named".to_string())),
            "a named drawing is tiled, carrying its name"
        );
        // ⚠️ Mutation `if name.is_none()` (drop the group check) SKIPS this — the exact
        // doc-86 item-3 bug (an unnamed group child gets no tile).
        assert_eq!(
            present.get(&20),
            Some(&None),
            "an UNNAMED group child is tiled by its id, with no name"
        );
        // ⚠️ Mutation dropping the `continue` (bake-all) makes this present — a wasted tile.
        assert!(
            !present.contains_key(&30),
            "unnamed canvas art no group references is NOT tiled"
        );
    }

    #[test]
    fn select_present_skips_stale_bits() {
        // A map value whose entity was despawned must not be baked — its tile is evicted,
        // not resurrected. ⚠️ This pins the END-TO-END invariant, not the `get_entity`
        // guard specifically: a despawned entity also has no `Name`, so it falls into the
        // unnamed-AND-no-group skip even without the guard — dropping the guard does NOT
        // falsify this. The guard is robustness (it mirrors `vec_entities::sync`'s own
        // `get_entity(..).is_err()`); it earns its keep the day a Name-independent tiling
        // path appears, which is exactly what this gate would then catch.
        let mut sim = SimWorld::new();
        let live = sim.world_mut().spawn((Name::new("Live"),)).id();
        let dead = sim.world_mut().spawn((Name::new("Dead"),)).id();
        sim.world_mut().despawn(dead);
        let mut map = VecEntityMap::new();
        map.insert(1, live.to_bits());
        map.insert(2, dead.to_bits());
        let present = select_present(sim.world(), &map);
        assert!(present.contains_key(&1), "the live drawing is tiled");
        assert!(
            !present.contains_key(&2),
            "the despawned drawing is skipped"
        );
    }
}
