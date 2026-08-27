//! **Baking engine VECTORS to a tile** (doc 86 §2, Wave A2) — the shell half of
//! `source.object` for a vector shape.
//!
//! A sprite already IS a tile (its atlas cell); `motion_bridge_objects` publishes
//! it directly. A vector is a curve, and a stamped vector has **two render forms**:
//! it rides `geometry_id` and is drawn CRISP by the vector pass at any zoom
//! (ADR-0154, Part 1) — the default — **and** it is rasterized once into an
//! individual texture (`texture_id`, the LOD tile), the GPU-instanced fallback the
//! shell swaps in above a count threshold (a live-vector grid of 160k stars is a
//! per-frame freeze; the instanced tile scaled to millions before Part 1 removed
//! it). The rasterize half is the FX raster stack's (`fx_live`) machinery verbatim:
//! `draw_path_isolated → VelloPass → an individual texture`, the difference being
//! the DESTINATION (the sprite renderer's `IndividualTextureStore`, not the Vello
//! atlas). One offscreen readback feeds BOTH the tile and the card thumbnail.
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

use crate::render_loop::motion_shape_gen::VecPathStore;
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

/// What a `source.object` VECTOR is on the render side: its LIVE `geometry_id`
/// (a [`VecPathStore`] handle the membrane emits, drawn crisp by the vector pass —
/// ADR-0154's route, reused for objects so a stamped vector stays sharp at ANY
/// zoom instead of the fixed-DPI raster it used to bake) + a full-res tile
/// `texture_id` for the **LOD path** (the drawing rasterized into an individual
/// texture, the pre-Part-1 machinery restored: above a count threshold the crisp
/// live-vector render becomes a freeze — 160k Vello fills/frame — so the shell
/// swaps those instances for this GPU-instanced tile, which scaled to millions
/// before; crispness is kept where it shows, at low counts) + the drawing's WORLD
/// size (so the sink stamps it at its natural size) + a small raster THUMBNAIL for
/// the node-card preview.
struct Baked {
    /// The artist's name for this shape, if any. **Metadata, not the cache key** —
    /// the cache is keyed by [`VecPathId`] (undo/rename-stable), so a rename refreshes
    /// this field without re-storing, and an UNNAMED group child (`None`) still gets a
    /// handle. `objects()` (the individual publish) yields only the named ones.
    name: Option<String>,
    key: BakeKey,
    geometry_id: u32,
    /// The LOD tile: an [`IndividualTextureStore`](ph2d_render) handle for the same
    /// drawing rasterized once at [`BAKE_DPI`]. Refcounted — every store insert pairs
    /// with a `release` (evict / re-bake), so a tile never leaks VRAM.
    texture_id: u32,
    size: [f32; 2],
    /// A mini-render for the source node's card preview (doc 86 A5), downsampled
    /// once on a content change ⇒ cached like everything else here.
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

/// One named vector's LIVE handle, read by the membrane: the `geometry_id` the
/// appearance stream carries + the drawing's world size. (The LOD tile is a separate
/// axis — the membrane always publishes `geometry_id`; the post-cook LOD partition
/// resolves the tile by `geometry_id` via [`ObjectBake::tile_texture_for_gid`], so it
/// does not ride the appearance stream.)
pub(crate) struct ObjectVector {
    pub geometry_id: u32,
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
    /// The `name -> object` map the membrane publishes individually (the picker path).
    /// Read-only — the sync ran at the fx phase; here the membrane only reads results.
    /// Only the NAMED entries are yielded: an unnamed group child has a handle (for the
    /// group stamp) but nothing to type into a node. On a name clash the higher id wins
    /// the `set_external` (id-ascending iteration ⇒ later), as the name-keyed map did.
    pub(crate) fn objects(&self) -> impl Iterator<Item = (&str, ObjectVector)> {
        self.cache.values().filter_map(|b| {
            b.name.as_deref().map(|n| {
                (
                    n,
                    ObjectVector {
                        geometry_id: b.geometry_id,
                        size: b.size,
                    },
                )
            })
        })
    }

    /// The live handle for ONE shape by its [`VecPathId`], or `None` if it isn't stored
    /// (doc 86 §2 A4). A group child that is a vector resolves its appearance through
    /// this — by its drawing id, so an unnamed child still resolves.
    pub(crate) fn vector_for_id(&self, id: VecPathId) -> Option<ObjectVector> {
        self.cache.get(&id).map(|b| ObjectVector {
            geometry_id: b.geometry_id,
            size: b.size,
        })
    }

    /// The LOD tile `texture_id` for a live `geometry_id`, or `None` if that geometry
    /// isn't stored. The LOD partition (`motion_bridge_objects::apply_object_lod`) reads
    /// this: a `VectorInstance` carries a `geometry_id`, and above the count threshold it
    /// is swapped for a GPU-instanced tile quad sampling this texture. O(cache) — a
    /// handful of named shapes, so a linear scan by `geometry_id` is free.
    pub(crate) fn tile_texture_for_gid(&self, geometry_id: u32) -> Option<u32> {
        self.cache
            .values()
            .find(|b| b.geometry_id == geometry_id)
            .map(|b| b.texture_id)
    }

    /// Seed a fake handle under `id` — for headless membrane gates that drive
    /// `resolve_leaf` / `apply_object_lod` without a GPU render. `name` é o que o
    /// `objects()` publica: `None` = um filho de grupo sem nome (que não é
    /// publicado), `Some` = a forma que o artista nomeou.
    #[cfg(test)]
    pub(crate) fn seed_named_for_test(
        &mut self,
        id: VecPathId,
        name: Option<&str>,
        geometry_id: u32,
        texture_id: u32,
        size: [f32; 2],
    ) {
        self.cache.insert(
            id,
            Baked {
                name: name.map(ToString::to_string),
                key: BakeKey {
                    path: VecPath::default(),
                    linear: [0.0; 4],
                    dpi_q: 0,
                },
                geometry_id,
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

    /// O caso comum das gates antigas: um handle sem nome.
    #[cfg(test)]
    pub(crate) fn seed_for_test(
        &mut self,
        id: VecPathId,
        geometry_id: u32,
        texture_id: u32,
        size: [f32; 2],
    ) {
        self.seed_named_for_test(id, None, geometry_id, texture_id, size);
    }

    /// The preview THUMBNAIL for a live `geometry_id`, or `None` (doc 86 A5). The
    /// readout hands it to the source node's snapshot so the card shows a mini-render
    /// of the object instead of a single origin dot. O(cache) — a handful of named
    /// shapes, read once a frame; the gid comes from the stream's own column.
    pub(crate) fn thumbnail_for(
        &self,
        geometry_id: u32,
    ) -> Option<ph2d_panel_motion_graph::PreviewThumb> {
        self.cache
            .values()
            .find(|b| b.geometry_id == geometry_id)
            .map(|b| b.thumb.clone())
    }

    /// Re-store every named vector whose drawing changed; drop any name that vanished.
    /// Runs at the fx phase, where every handle (the vector `store`, `gpu`, `renderer`,
    /// the vector scene + transforms + live geometry) is in hand. Cached by content ⇒ a
    /// static scene bakes once. ⚠️ A `geometry_id` slot in the `store` has no eviction
    /// (the named, session-bounded trade); the LOD **`texture_id` DOES** — every store
    /// insert pairs with a `release` on evict / re-bake, mirroring [`crate::motion_flip_bake`],
    /// so a tile never leaks VRAM.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bake(
        &mut self,
        store: &mut VecPathStore,
        scene: &VecScene,
        map: &VecEntityMap,
        xforms: &VecXforms,
        live: &LiveGeometry,
        gpu: &GpuContext,
        renderer: &mut SpriteRenderer,
        surface_format: wgpu::TextureFormat,
        sim: &SimWorld,
    ) {
        // The drawings to store this frame, keyed by VecPathId → the artist's name
        // (`None` for an unnamed group child): named ∪ group-referenced (see
        // `select_present`). Keying by id — not name — is what lets an unnamed group
        // child get a handle and a rename keep it.
        let world = sim.world();
        let present = select_present(world, map);

        // Drop the ids that vanished (deleted, or an unnamed shape no group references
        // any more). O tile É libertado e a geometria É esquecida — os dois recursos
        // que um bake adquire saem pela mesma porta.
        //
        // ⚠️ **O `store.forget` foi acrescentado em 2026-08-21, e a linha que ele
        // substitui dizia o defeito em voz alta:** *"the store slot goes dead but is
        // not reclaimed (no eviction)"*. A chave do bake inclui a TRANSFORMAÇÃO, então
        // girar ou escalar um objeto deixava um `VecPath` morto **por quadro**. Numa
        // engine cujo laço corre horas, «não reclamado» é «fuga» (Enio, 2026-08-21).
        let gone: Vec<VecPathId> = self
            .cache
            .keys()
            .filter(|id| !present.contains_key(*id))
            .copied()
            .collect();
        for id in gone {
            if let Some(b) = self.cache.remove(&id) {
                renderer.individual_mut().release(b.texture_id);
                store.forget(b.geometry_id);
            }
        }

        // Store the new + changed. A cache hit is a `VecPath`/xform equality — no
        // store push, no thumbnail render.
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
                // re-publishes without a re-store, and continue.
                if let Some(b) = self.cache.get_mut(&id) {
                    b.name = name;
                }
                continue;
            }
            // Changed or new: store the live geometry (→ `geometry_id`), upload the LOD
            // tile (→ `texture_id`) and render the preview thumbnail. Larga os DOIS
            // recursos anteriores DEPOIS do novo bake (a ordem protege o caso em que o
            // bake falha: o antigo ainda está lá para o quadro seguinte re-tentar).
            let old = self.cache.remove(&id);
            let baked = bake_one(
                &mut self.scratch,
                store,
                scene,
                xforms,
                live,
                id,
                gpu,
                renderer,
                surface_format,
                path,
            );
            if let Some(b) = old {
                renderer.individual_mut().release(b.texture_id);
                store.forget(b.geometry_id);
            }
            if let Some((geometry_id, texture_id, size, thumb)) = baked {
                self.cache.insert(
                    id,
                    Baked {
                        name,
                        key,
                        geometry_id,
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

/// The fixed bake "camera": world units → tile pixels, carrying the **Y-flip**
/// that maps world-Y-up to the sprite renderer's top-down texture row order. NOT
/// the live camera (so the tile is zoom-independent), but the SAME
/// `scale_non_uniform(k, -k)` that [`ph2d_render::Camera2d::world_to_screen_affine`]
/// and the Flip bake ([`crate::motion_flip_bake`], which deliberately reuses the
/// frame camera *"so the tile's orientation match"*) apply: Vello renders Y-DOWN
/// and the sprite renderer displays texture row 0 at screen-TOP, so without the
/// `-BAKE_DPI` the baked star points DOWN (the smoke report: *"a estrela no grid
/// fica de cabeça para baixo"*). Named so the upright-tile gate can pin the flip.
fn bake_camera() -> Affine {
    Affine::scale_non_uniform(BAKE_DPI, -BAKE_DPI)
}

/// Store ONE path as a live vector (→ `geometry_id`), upload its full-res tile
/// (→ `texture_id`) and render its preview thumbnail; returns
/// `(geometry_id, texture_id, world size, thumb)`, or `None` if the shape has no
/// drawable bounds or a GPU step failed (skipped, not guessed). The
/// orientation-critical render+readback is [`bake_rgba`]; the live geometry is parked
/// in the `store` the vector [`encode`](crate::render_loop::motion_shape_gen::encode)
/// reads (crisp path), and the SAME readback bytes are uploaded to the sprite
/// renderer's `IndividualTextureStore` (the LOD tile — the GPU-instanced fallback).
/// ⚠️ ONE readback feeds BOTH the thumbnail and the tile; the full-res `rgba` was
/// discarded after Part-1's thumbnail-only bake and is now the tile.
#[allow(clippy::too_many_arguments)]
fn bake_one(
    scratch: &mut Option<VelloPass>,
    store: &mut VecPathStore,
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    surface_format: wgpu::TextureFormat,
    path: &VecPath,
) -> Option<(u32, u32, [f32; 2], ph2d_panel_motion_graph::PreviewThumb)> {
    let (rgba, wpx, hpx, size) = bake_rgba(scratch, scene, xforms, live, id, gpu, surface_format)?;
    // The card thumbnail (doc 86 A5) — one downsample per content change, cached.
    let thumb = thumbnail(&rgba, wpx, hpx);
    // The LOD tile: the full-res straight-RGBA readback uploaded to an individual
    // texture. Refcounted (the caller releases the old one on re-bake / evict).
    let texture_id = renderer.acquire_individual(wpx, hpx, &rgba).ok()?;
    // Park the authored geometry in the store the vector encode reads; the handle
    // is the `geometry_id` the membrane emits, drawn crisp (and honouring its own
    // fill/stroke) by `draw_shape_instance`.
    let geometry_id = store.push(path.clone());
    Some((geometry_id, texture_id, size, thumb))
}

/// Render ONE path into the fixed-DPI tile and read its tightly-packed straight
/// RGBA8 back — the orientation-critical half of [`bake_one`], extracted so the
/// upright-tile gate drives the REAL bake pipeline (ONE door), never a
/// reimplementation. Returns `(rgba, wpx, hpx, world_size)`. Touches no sprite
/// renderer (no upload). The readback is slow, but it runs only on a content
/// change (cached by the caller), so steady state pays nothing.
#[allow(clippy::too_many_arguments)]
fn bake_rgba(
    scratch: &mut Option<VelloPass>,
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    id: VecPathId,
    gpu: &GpuContext,
    surface_format: wgpu::TextureFormat,
) -> Option<(Vec<u8>, u32, u32, [f32; 2])> {
    let camera = bake_camera();
    let (x0, y0, x1, y1) = ph2d_vec_render::path_screen_bounds(scene, xforms, live, id, camera)?;
    let wpx = ((x1 - x0).ceil() as u32).clamp(1, MAX_TILE_SIDE);
    let hpx = ((y1 - y0).ceil() as u32).clamp(1, MAX_TILE_SIDE);

    // Encode the one path, translated so its bbox min corner (the top-left under
    // the Y-flipped camera) lands at the tile origin (0,0) = row 0 = screen-top.
    let mut scratch_scene = VectorScene::new();
    // ⚠️⚠️ **UM OBJECTO DE MOTION ASSADO DE UMA FORMA COM PADRÃO leva a `fallback`, e é DECLARADO.**
    // Este assado alimenta o oleoduto do Motion, que não tem o mapa de ladrilhos do quadro em mãos —
    // a MESMA fronteira da rota de instância (`instance.rs`), e pela mesma razão. ⛔ Não é *"não
    // deu"*: inventar um mapa aqui seria adivinhar de onde ele vem.
    ph2d_vec_render::draw_path_isolated(
        scene,
        xforms,
        live,
        &ph2d_vec_render::PatternTiles::new(),
        id,
        camera,
        Affine::translate((-x0, -y0)),
        &mut scratch_scene,
    );

    // Render offscreen (a dedicated scratch pass, reused + resized across bakes)
    // and read the tightly-packed straight RGBA8 back.
    let pass = match scratch.as_mut() {
        Some(p) => p,
        None => scratch.insert(VelloPass::new(gpu, surface_format, (wpx, hpx)).ok()?),
    };
    let mut rgba = pass
        .render_and_readback(gpu, scratch_scene.inner(), (wpx, hpx))
        .ok()?;
    let want = (wpx * hpx * 4) as usize;
    if rgba.len() < want {
        return None;
    }
    rgba.truncate(want);
    let size = [
        (x1 - x0) as f32 / BAKE_DPI as f32,
        (y1 - y0) as f32 / BAKE_DPI as f32,
    ];
    Some((rgba, wpx, hpx, size))
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
#[path = "motion_object_bake_tests.rs"]
mod tests;
