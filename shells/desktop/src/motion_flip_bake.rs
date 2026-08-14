//! **Baking engine FLIP objects to a tile** (doc 86 §2, Wave A3) — the shell half
//! of `source.object` for a Flip animation object. Sibling of `motion_object_bake`
//! (the vector bake); the `source.object` node is UNCHANGED (media-agnostic — only
//! the membrane learns a new medium).
//!
//! A sprite already IS a tile; a vector shape has three offscreen primitives in
//! `ph2d-vec-render` (`path_screen_bounds`/`draw_path_isolated`/`render_and_readback`),
//! so A2 reused them. **Flip has none of the three** — its render path is entirely
//! GPU→GPU (rasterize → resolve → 22-mode compositor → blit into `game_rt`). So this
//! wave builds a bounds-at-frame and an offscreen readback, driving the **same** raster
//! and compositor the frame pass drives (`FlipCompose::stage_layer` and the
//! `LayerCompositor`), so the baked tile is byte-for-byte what the Flip drawn directly
//! would be. Nothing diverges: **one engine, one state.**
//!
//! ## Three decisions
//!
//! **1. The whole OBJECT, at the current frame.** A Flip object is a STACK of layers,
//! each with its own drawing at the current playhead, composed with per-layer
//! blend/opacity. "The object the artist named" is all of them composed — the same
//! reading A1 (the whole sprite) and A2 (the whole shape) take. So the bake drives the
//! per-layer stage → inject → composite exactly as `flip_pass::composite_layers` does,
//! then reads the compositor's straight-`Rgba8Unorm` output back instead of blitting it
//! to `game_rt`.
//!
//! **2. SCRATCH renderers, not the frame pass's.** The bake owns its own
//! `FlipRenderer`/`FlipCompose`/`LayerCompositor` (like A2's scratch `VelloPass`),
//! never the persistent frame-pass ones. Using those with a bake-DPI camera + a
//! tile-sized viewport would thrash their per-frame frescura memo every frame
//! (bake-size ↔ window-size). The bake camera is reused from the frame pass
//! (`camera_raw`/`fold_model`) so the tile's orientation + thickness convention match
//! the screen exactly — a hand-rolled matrix is a second door that flips Y or mis-scales
//! the line. The trail engine (walk vs raster) is armed from the SAME `new_engine_armed`,
//! so `PH2D_FLIP_NEW_ENGINE=0` doesn't make the tile disagree with the screen.
//!
//! **3. Cached by CONTENT, LIVE tier — the tile follows the drawing AND the frame.**
//! The key hashes the RESOLVED content at the current frame (each visible layer's
//! drawing geometry + pose + blend/opacity/depth, folded with the object's LINEAR
//! transform + the DPI). A static hold across frames keeps the same key ⇒ no re-bake;
//! a frame change that swaps the drawing or moves a pose, an edit, or a rotate/scale
//! re-bakes. The object's world TRANSLATION is excluded (the tile is bbox-normalized,
//! so dragging the object never re-bakes — A2's rule). Every `acquire` pairs with a
//! `release` (refcounted `IndividualTextureStore`), so tiles don't leak VRAM.

use std::collections::BTreeMap;

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, Name, SimWorld};
use ph2d_flip::{FlipDoc, FlipObject, FlipObjectId, Frame};
use ph2d_flip_render::{FlipCompose, FlipRenderer, pack_drawing};
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_render::layer_compositor::{
    LayerCompositor, LayerOp, LayerPixelProvider, LayerPixels, Region,
};
use ph2d_render::{Camera2d, SpriteRenderer};
use ph2d_vec_scene::Xform;

use crate::flip_entities::FlipEntityMap;
use crate::flip_transform::{art_to_world, build as build_flip_models};
// ⚠️ The tile resolution + cap are SHARED with the vector bake — one number for
// "how crisp a stamped tile is" (a Flip tile and a vector tile of the same world
// size have the same inner resolution). A second const would drift.
use crate::motion_object_bake::{BAKE_DPI, MAX_TILE_SIDE};
// ⚠️ Reused from the frame pass, NOT re-derived: the camera convention (Y direction +
// the world-unit thickness ruler, an Enio decision) has exactly one home.
use crate::render_loop::flip_pass::camera::{camera_raw, fold_model};
use crate::render_loop::flip_pass::new_engine_armed;

/// The scratch offscreen HDR format that `FlipRenderer` draws into and `FlipCompose`
/// resolves from. The bake never blits to `game_rt`, so the `FlipCompose` blit target
/// format is irrelevant — this constant only feeds the (unused) blit pipeline; the
/// real work resolves to the compositor's straight `Rgba8Unorm` output.
const SCRATCH_HDR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// What a baked Flip object is on the render side: the individual `texture_id` + the
/// tile's WORLD size (natural stamp size), keyed by a content hash of the resolved frame.
struct FlipBaked {
    /// The artist's name, if any. **Metadata, not the cache key** — the cache is keyed by
    /// [`FlipObjectId`] (undo/rename-stable); an unnamed group child (`None`) still gets a tile.
    name: Option<String>,
    key: u64,
    texture_id: u32,
    size: [f32; 2],
    /// A mini-render of the composed tile for the node-card preview (doc 86 A5), cached.
    thumb: ph2d_panel_motion_graph::PreviewThumb,
}

/// One named Flip object's tile: baked `texture_id` + world size (a Flip stays a
/// RASTER tile — no live vector path, unlike a `source.object` vector, ADR-0154).
pub(crate) struct FlipTile {
    pub texture_id: u32,
    pub size: [f32; 2],
}

/// The Flip bake cache + its scratch renderers (own `FlipRenderer`/`FlipCompose`/
/// `LayerCompositor`, never the frame pass's). `Default` is empty until the first bake.
#[derive(Default)]
pub(crate) struct FlipObjectBake {
    render: Option<FlipRenderer>,
    compose: Option<FlipCompose>,
    compositor: Option<LayerCompositor>,
    /// The zeroed pixels the compositor's `LayerPixelProvider` reports for every key
    /// (version 0 — the injected slices are what's really composed; never uploaded).
    dummy: Vec<u8>,
    /// Keyed por `(objeto, QUADRO RESOLVIDO)` — o id é undo/rename-stable (um filho de
    /// grupo sem nome ganha tile e um rename não o despeja), e o quadro é o que torna
    /// um mesmo objeto em dois tempos duas tiles.
    ///
    /// ⚠️ **A chave é o QUADRO, não o offset, e é isso que fecha o perigo do param
    /// dirigido.** Um `time_offset` pode ser dirigido por fio (doc 58) — um `value.lfo`
    /// nele varreria offsets contínuos, e uma chave por offset cunharia uma tile por
    /// QUADRO DE APP, com a cache nunca acertando. Chaveando pelo quadro, dois offsets
    /// que caem no mesmo desenho **são a mesma tile**, e o conjunto vivo é limitado
    /// pelo que o grafo pede neste quadro (o despejo abaixo remove o resto) — ou seja
    /// pela contagem de nós, que é uma coisa que o artista põe na tela com a mão.
    cache: BTreeMap<(FlipObjectId, Frame), FlipBaked>,
    /// `(objeto, BITS do offset) -> quadro`, reconstruído a cada bake: a tradução de
    /// *o que o grafo pediu* para *que tile responde*. Não é uma segunda cache — é a
    /// resolução, e ela existe porque o pedido chega em segundos e a tile mora num
    /// quadro.
    asked: BTreeMap<(FlipObjectId, u32), Frame>,
}

impl FlipObjectBake {
    /// Re-bake every named Flip object whose resolved frame changed; evict + RELEASE
    /// the texture of any name that vanished. Runs at the fx phase, where the doc, the
    /// entity map, the playhead, the GPU + the sprite renderer are in hand. Cached by
    /// content ⇒ a static hold (same drawing/pose across frames) bakes once.
    ///
    /// `shifts` são os offsets de tempo que o grafo pede neste quadro, em segundos.
    /// O chamador SEMPRE inclui `0.0` (o nome cru), e os demais vêm dos
    /// `source.object` do documento — então o conjunto de tiles vivas é o que a cena
    /// de fato nomeia, e o despejo abaixo devolve a VRAM de tudo o mais.
    #[allow(clippy::too_many_arguments)] // o 8º é `shifts`, e cada um é um dono distinto
    pub(crate) fn bake(
        &mut self,
        flip: &FlipDoc,
        map: &FlipEntityMap,
        playhead: &Playhead,
        shifts: &[f32],
        gpu: &GpuContext,
        renderer: &mut SpriteRenderer,
        sim: &SimWorld,
    ) {
        // Object world transforms (ADR-0111 — the gizmo moves/rotates/scales the Flip
        // object via its entity `Transform`; the render folds it in). Same source the
        // frame pass uses (`flip_transform::build`).
        let models = build_flip_models(sim, map);

        // The objects to tile this frame, keyed by FlipObjectId → the artist's name
        // (`None` for an unnamed group child): named ∪ group-referenced (`select_present`).
        // Keying by id — not name — is what lets an unnamed group child get a tile and a
        // rename keep it.
        let world = sim.world();
        let present = select_present(world, map);

        // A resolução do quadro é reconstruída do zero: ela descreve o que o grafo
        // pede AGORA, e é ela que decide o que sobrevive ao despejo abaixo.
        self.asked.clear();
        for &oid in present.keys() {
            let Some(obj) = flip.objects().iter().find(|o| o.id == oid) else {
                continue;
            };
            for sh in shifts {
                self.asked.insert(
                    (oid, sh.to_bits()),
                    obj.frame_at_shifted(playhead, f64::from(*sh)),
                );
            }
        }

        // Despeja o que ninguém pede mais: o objeto que sumiu (deletado, ou um filho de
        // grupo sem nome que nenhum grupo referencia) E o QUADRO que nenhum offset
        // resolve neste quadro do app — é esta segunda metade que impede um offset
        // dirigido por fio de acumular uma tile por quadro visitado.
        let wanted: std::collections::BTreeSet<(FlipObjectId, Frame)> =
            self.asked.iter().map(|((o, _), f)| (*o, *f)).collect();
        let gone: Vec<(FlipObjectId, Frame)> = self
            .cache
            .keys()
            .filter(|k| !wanted.contains(*k))
            .copied()
            .collect();
        for k in gone {
            if let Some(b) = self.cache.remove(&k) {
                renderer.individual_mut().release(b.texture_id);
            }
        }

        // Bake the new + changed. A cache hit is a content-hash equality — no GPU work.
        for (oid, name) in present {
            let Some(obj) = flip.objects().iter().find(|o| o.id == oid) else {
                continue;
            };
            let model = models
                .iter()
                .find(|(id, _)| *id == oid)
                .map_or(Xform::IDENTITY, |(_, x)| *x);
            // Um objeto, um quadro por offset pedido — e dois offsets que caem no MESMO
            // desenho visitam a mesma entrada, então o segundo é um acerto de cache.
            let frames: std::collections::BTreeSet<Frame> = shifts
                .iter()
                .map(|sh| obj.frame_at_shifted(playhead, f64::from(*sh)))
                .collect();
            for frame in frames {
                let ck = (oid, frame);
                let key = content_key(obj, &model, frame);
                if self.cache.get(&ck).is_some_and(|b| b.key == key) {
                    // Content unchanged — refresh the (metadata) NAME so a rename
                    // re-publishes without a re-bake, and continue.
                    if let Some(b) = self.cache.get_mut(&ck) {
                        b.name.clone_from(&name);
                    }
                    continue;
                }
                let old = self.cache.remove(&ck).map(|b| b.texture_id);
                let baked = self.bake_one(obj, &model, frame, gpu, renderer);
                if let Some(t) = old {
                    renderer.individual_mut().release(t);
                }
                if let Some((texture_id, size, thumb)) = baked {
                    self.cache.insert(
                        ck,
                        FlipBaked {
                            name: name.clone(),
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

    /// Bake ONE object at the current frame into a fresh individual texture; returns
    /// `(texture_id, world size)`, or `None` if the object has no drawable geometry at
    /// this frame or a GPU step failed (skipped, not guessed).
    fn bake_one(
        &mut self,
        obj: &FlipObject,
        model: &Xform,
        frame: Frame,
        gpu: &GpuContext,
        renderer: &mut SpriteRenderer,
    ) -> Option<(u32, [f32; 2], ph2d_panel_motion_graph::PreviewThumb)> {
        // The visible layers WITH geometry at this frame, bottom-to-top. Each carries
        // its LOCAL→world afine (object model ∘ layer pose) and its blend/opacity + a
        // per-layer compositor key. No parallax (a template has no camera pan).
        let mut layers: Vec<BakeLayer<'_>> = Vec::new();
        for layer in obj.layers() {
            if !layer.visible {
                continue;
            }
            let Some(did) = layer.drawing_at_cycled(frame) else {
                continue;
            };
            let Some(drawing) = obj.drawing(did) else {
                continue;
            };
            if drawing.strokes.is_empty() {
                continue;
            }
            layers.push(BakeLayer {
                drawing,
                lm: art_to_world(model, layer.pose_at_cycled(frame)),
                blend: layer.blend.to_u8(),
                opacity: layer.opacity,
                // Distinct per layer within this one object (layer ids are unique),
                // which is all the compositor needs for a single object's composite.
                key: u64::from(layer.id.0),
            });
        }
        if layers.is_empty() {
            return None;
        }

        // WORLD bbox of the strokes (post model+pose), inflated by the max stroke
        // half-width (thickness is world units — the frame-pass convention). The bbox
        // is framed by the bake camera, so the object's world position is normalized
        // out automatically (the tile is a template, stamped at each point's P).
        let mut min = [f32::INFINITY; 2];
        let mut max = [f32::NEG_INFINITY; 2];
        let mut half_w = 0.0f32;
        for l in &layers {
            for stroke in &l.drawing.strokes {
                for w in stroke.widths() {
                    half_w = half_w.max(*w * 0.5);
                }
                for p in stroke.positions() {
                    let wp = xform_point(&l.lm, *p);
                    min[0] = min[0].min(wp[0]);
                    min[1] = min[1].min(wp[1]);
                    max[0] = max[0].max(wp[0]);
                    max[1] = max[1].max(wp[1]);
                }
            }
        }
        if !min[0].is_finite() || !max[0].is_finite() {
            return None;
        }
        let (x0, y0) = (min[0] - half_w, min[1] - half_w);
        let (x1, y1) = (max[0] + half_w, max[1] + half_w);
        let bw = (x1 - x0).max(1e-4);
        let bh = (y1 - y0).max(1e-4);
        let wpx = tile_side(bw);
        let hpx = tile_side(bh);

        // The base camera framing the bbox — reused from the frame pass so Y and the
        // thickness ruler match the screen (a `Camera2d` centred on the bbox, height =
        // bbox height, viewport = the tile ⇒ px_per_world == BAKE_DPI by construction).
        let win = WindowSize {
            width: wpx,
            height: hpx,
        };
        let bake_cam = Camera2d::new([(x0 + x1) * 0.5, (y0 + y1) * 0.5], bh);
        let base = camera_raw(&bake_cam, win);

        // Scratch renderers (lazy). Arm the SAME trail engine the frame pass uses, so
        // the tile can't disagree with the on-screen Flip under `PH2D_FLIP_NEW_ENGINE`.
        let render = self
            .render
            .get_or_insert_with(|| FlipRenderer::new(&gpu.device, SCRATCH_HDR));
        let compose = self
            .compose
            .get_or_insert_with(|| FlipCompose::new(&gpu.device, SCRATCH_HDR));
        let compositor = self
            .compositor
            .get_or_insert_with(|| LayerCompositor::new(gpu));
        compose.set_walk_engine(&gpu.device, new_engine_armed());

        // The op-list (bottom-to-top), built before the loop because inject/composite
        // dimension by it — the SAME shape `composite_layers` builds.
        let ops: Vec<LayerOp> = layers
            .iter()
            .map(|l| LayerOp::Layer {
                mask: None,
                clipping: false,
                key: l.key,
                blend_mode: l.blend,
                opacity: l.opacity,
            })
            .collect();

        // Stage + inject each layer. `stage_layer` rasterizes the layer's drawing (its
        // LOCAL geometry, the `model` folded into the camera) and resolves to a straight
        // slice; `inject_slice_from_texture` copies it (GPU→GPU) into the compositor.
        for l in &layers {
            let data = pack_drawing(l.drawing);
            let layer_cam = if l.lm.is_identity() {
                base
            } else {
                fold_model(&base, &l.lm)
            };
            let slice = compose.stage_layer(
                &gpu.device,
                &gpu.queue,
                render,
                &layer_cam,
                &data,
                (wpx, hpx),
            );
            if let Err(e) = compositor.inject_slice_from_texture(
                gpu,
                &ops,
                l.key,
                slice,
                wpx,
                hpx,
                (0, 0, wpx, hpx),
                0,
            ) {
                eprintln!("[motion.flip bake] inject falhou: {e}");
                return None;
            }
        }

        // Composite (blend/opacity per-layer) → the compositor's straight Rgba8Unorm
        // output, then read it back to CPU RGBA (never blitted to game_rt).
        let need = (wpx as usize) * (hpx as usize) * 4;
        if self.dummy.len() != need {
            self.dummy.resize(need, 0);
        }
        let provider = DummyProvider {
            pixels: &self.dummy,
        };
        if let Err(e) = compositor.composite(gpu, &ops, &provider, wpx, hpx, Region::full(wpx, hpx))
        {
            eprintln!("[motion.flip bake] composite falhou: {e}");
            return None;
        }
        let out = compositor.output_texture()?;
        let rgba = readback(gpu, out, wpx, hpx);
        if rgba.len() < need {
            return None;
        }
        // The card thumbnail (doc 86 A5) from the SAME composed bytes, via the shared
        // downsampler the vector bake uses — one twin, one look.
        let thumb = crate::motion_object_bake::thumbnail(&rgba[..need], wpx, hpx);
        let texture_id = renderer.acquire_individual(wpx, hpx, &rgba[..need]).ok()?;
        Some((texture_id, [bw, bh], thumb))
    }
}

/// The Flip objects to bake this frame, keyed by [`FlipObjectId`] → the artist's name
/// (`None` for an unnamed group child). Baked iff **named** (the picker path) OR inside a
/// **named group** ([`entity_is_in_a_named_group`], so the group stamp has its tile);
/// unnamed canvas objects no group references are NOT baked ⇒ no wasted VRAM. Pure ECS,
/// so the selection is a headless gate. The twin of `motion_object_bake::select_present`.
fn select_present(
    world: &ph2d_ecs::World,
    map: &FlipEntityMap,
) -> BTreeMap<FlipObjectId, Option<String>> {
    use crate::render_loop::motion_bridge::entity_is_in_a_named_group;
    let mut present: BTreeMap<FlipObjectId, Option<String>> = BTreeMap::new();
    for (&oid, &bits) in map {
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
        present.insert(oid, name);
    }
    present
}

/// One visible layer prepared for the composite drive.
struct BakeLayer<'a> {
    drawing: &'a ph2d_flip::FlipDrawing,
    lm: Xform,
    blend: u8,
    opacity: f32,
    key: u64,
}

/// The content hash keying the LIVE cache (doc 86 §2). Hashes the RESOLVED frame — each
/// visible layer's drawing geometry, its blend/opacity/depth, and its afine folded with
/// the object's LINEAR transform (translation excluded ⇒ dragging the object never
/// re-bakes, the tile being bbox-normalized) — plus the DPI. A static hold keeps the
/// key; a frame change that swaps the drawing or moves a pose, an edit, or a rotate/
/// scale changes it.
fn content_key(obj: &FlipObject, model: &Xform, frame: Frame) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    BAKE_DPI.to_bits().hash(&mut h);
    // The object's LINEAR part only (translation zeroed): a MOVE is a cache hit.
    let [a, b, c, d, _, _] = model.0;
    let linear = Xform([a, b, c, d, 0.0, 0.0]);
    for layer in obj.layers() {
        if !layer.visible {
            continue;
        }
        let Some(did) = layer.drawing_at_cycled(frame) else {
            continue;
        };
        let Some(drawing) = obj.drawing(did) else {
            continue;
        };
        if drawing.strokes.is_empty() {
            continue;
        }
        did.0.hash(&mut h);
        layer.blend.to_u8().hash(&mut h);
        layer.opacity.to_bits().hash(&mut h);
        layer.depth.to_bits().hash(&mut h);
        // The layer's afine WITHOUT the object translation — captures the object's
        // linear transform + the full authored layer pose (a pose translation is part
        // of the object's internal composition, so it DOES re-bake).
        for v in art_to_world(&linear, layer.pose_at_cycled(frame)).0 {
            v.to_bits().hash(&mut h);
        }
        for stroke in &drawing.strokes {
            for p in stroke.positions() {
                p.x.to_bits().hash(&mut h);
                p.y.to_bits().hash(&mut h);
            }
            for w in stroke.widths() {
                w.to_bits().hash(&mut h);
            }
        }
    }
    h.finish()
}

/// The tile side in pixels for a world span, at `BAKE_DPI`, clamped to the VRAM/GPU cap
/// (a huge tile is a perf choice, not a correctness limit — clamped, never refused).
fn tile_side(world_span: f32) -> u32 {
    ((f64::from(world_span) * BAKE_DPI).ceil()).clamp(1.0, f64::from(MAX_TILE_SIDE)) as u32
}

/// Apply a `local → world` afine to a point. Matches `fold_model`'s interpretation of
/// `Xform.0 = [a,b,c,d,e,f]` (col-major `[[a,b,0,0],[c,d,0,0],[0,0,1,0],[e,f,0,1]]`), so
/// the bbox is computed in the SAME space the render places the geometry.
fn xform_point(m: &Xform, p: Vec2) -> [f32; 2] {
    let [a, b, c, d, e, f] = m.0;
    let (px, py) = (f64::from(p.x), f64::from(p.y));
    [(a * px + c * py + e) as f32, (b * px + d * py + f) as f32]
}

/// Provider that returns the zeroed dummy for ANY key at version `0` — the SAME version
/// `inject_slice_from_texture` records, so the compositor finds every slice "clean" and
/// never uploads the transparent dummy over the injected art (the frame pass's trick).
struct DummyProvider<'a> {
    pixels: &'a [u8],
}

impl LayerPixelProvider for DummyProvider<'_> {
    fn layer_pixels(&self, _key: u64) -> Option<LayerPixels<'_>> {
        Some(LayerPixels {
            version: 0,
            rgba8: self.pixels,
            dirty: None,
        })
    }
}

/// Read a straight `Rgba8Unorm` texture back to CPU RGBA (`w·h·4`, row padding stripped)
/// — the standard `copy_texture_to_buffer` + `map_async` + poll dance (mirrors
/// `fx_dump::readback`). Slow, but it runs only on a content change (cached).
fn readback(gpu: &GpuContext, tex: &wgpu::Texture, w: u32, h: u32) -> Vec<u8> {
    let unpadded = w * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("motion flip bake readback"),
        size: u64::from(padded) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let _ = rx.recv();
    let view = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    for row in 0..h as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&view[s..s + unpadded as usize]);
    }
    drop(view);
    staging.unmap();
    out
}

// Gates live in a sibling FILHO (via `#[path]`) so the parent stays under the shell
// LOC cap; `use super::*` there reaches the private `content_key`/`bake_one`.
// A metade que se CONSULTA (quais tiles existem, e qual responde a este pedido) mora
// num irmão: o pai é *como uma tile é PRODUZIDA* (o bake na GPU, os scratch renderers,
// a chave de conteúdo) e o filho *como se PEDE por uma*. Os dois são `impl
// FlipObjectBake` — a divisão é de assunto, não de tipo.
#[path = "motion_flip_bake_query.rs"]
mod query;

#[cfg(test)]
#[path = "motion_flip_bake_tests.rs"]
mod tests;
