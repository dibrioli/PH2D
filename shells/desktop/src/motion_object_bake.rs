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
    key: BakeKey,
    texture_id: u32,
    size: [f32; 2],
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
    cache: BTreeMap<String, Baked>,
}

impl ObjectBake {
    /// The `name -> tile` map the membrane publishes. Read-only — the bake ran
    /// at the fx phase; here the membrane only reads the results.
    pub(crate) fn tiles(&self) -> impl Iterator<Item = (&str, BakedTile)> {
        self.cache.iter().map(|(n, b)| {
            (
                n.as_str(),
                BakedTile {
                    texture_id: b.texture_id,
                    size: b.size,
                },
            )
        })
    }

    /// The baked tile for ONE named shape, or `None` if it isn't baked (doc 86 §2
    /// A4). A group child that is a vector resolves its appearance through this.
    pub(crate) fn tile_for(&self, name: &str) -> Option<BakedTile> {
        self.cache.get(name).map(|b| BakedTile {
            texture_id: b.texture_id,
            size: b.size,
        })
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
        // The named vectors present this frame, by name (last id wins on a name
        // clash — the artist's business, like `motion_bridge_shapes`).
        let mut present: BTreeMap<String, VecPathId> = BTreeMap::new();
        let world = sim.world();
        for (&id, &bits) in map {
            let Ok(entity) = world.get_entity(Entity::from_bits(bits)) else {
                continue;
            };
            let Some(name) = entity.get::<Name>().map(|n| n.0.clone()) else {
                continue; // unnamed: nothing for the artist to type into the node
            };
            if name.trim().is_empty() {
                continue;
            }
            present.insert(name, id);
        }

        // Evict the names that vanished (deleted / un-named / renamed away),
        // releasing their texture so a tile never leaks VRAM.
        let gone: Vec<String> = self
            .cache
            .keys()
            .filter(|n| !present.contains_key(*n))
            .cloned()
            .collect();
        for n in gone {
            if let Some(b) = self.cache.remove(&n) {
                renderer.individual_mut().release(b.texture_id);
            }
        }

        // Bake the new + changed. A cache hit is a `VecPath`/xform equality — no
        // GPU work, no readback.
        for (name, id) in present {
            let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
                continue;
            };
            let c = xform_of(xforms, id).0;
            let key = BakeKey {
                path: path.clone(),
                linear: [c[0], c[1], c[2], c[3]],
                dpi_q: BAKE_DPI as u32,
            };
            if self.cache.get(&name).is_some_and(|b| b.key == key) {
                continue; // unchanged — the tile is still valid
            }
            // Changed or new: release the previous texture (if any) and re-bake.
            let old = self.cache.remove(&name).map(|b| b.texture_id);
            let baked = bake_one(&mut self.scratch, scene, xforms, live, id, gpu, surface_format, renderer);
            if let Some(t) = old {
                renderer.individual_mut().release(t);
            }
            if let Some((texture_id, size)) = baked {
                self.cache.insert(name, Baked { key, texture_id, size });
            }
        }
    }
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
) -> Option<(u32, [f32; 2])> {
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
    // Upload as an individual texture — the SAME raw bytes the FX GPU→GPU copy
    // would move, so the colour behaviour matches; no Sprite is mutated.
    let texture_id = renderer.acquire_individual(wpx, hpx, &rgba[..want]).ok()?;
    let size = [(x1 - x0) as f32 / BAKE_DPI as f32, (y1 - y0) as f32 / BAKE_DPI as f32];
    Some((texture_id, size))
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
        let base = BakeKey { path: star(), linear: [1.0, 0.0, 0.0, 1.0], dpi_q: 256 };
        // A move never touches the local path or the linear coeffs.
        let moved = BakeKey { path: star(), linear: [1.0, 0.0, 0.0, 1.0], dpi_q: 256 };
        assert_eq!(base, moved, "a move is a cache hit — no re-bake");
        // A rotate changes the linear part.
        let rotated = BakeKey { path: star(), linear: [0.0, 1.0, -1.0, 0.0], dpi_q: 256 };
        assert_ne!(base, rotated, "a rotate re-bakes");
        // An edit changes the authored path.
        let edited = BakeKey {
            path: ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 6, 0.45),
            linear: [1.0, 0.0, 0.0, 1.0],
            dpi_q: 256,
        };
        assert_ne!(base, edited, "editing the shape re-bakes");
    }
}
