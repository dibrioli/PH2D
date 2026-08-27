//! SpriteRenderer — owns the pipeline + per-frame buffers and renders
//! one batch (one atlas) of sprites into a [`ph2d_gpu::FrameTarget`].
//!
//! M5 scope is one batch (one atlas). Multi-atlas grouping + sort by
//! z lands in M6 with the asset pipeline. The render method:
//!   1. Pulls every `RenderInstance` from the [`PresentWorld`] into a
//!      scratch `Vec` (no allocation if capacity already sufficient).
//!   2. Uploads via `Queue::write_buffer` into the dynamic instance
//!      buffer.
//!   3. Writes the camera uniform.
//!   4. Issues a single instanced triangle-strip draw (4 vertices,
//!      N instances).

use crate::atlas::TextureAtlas;
use crate::camera::{Camera2d, CameraUniform};
use crate::individual::{IndividualTextureError, IndividualTextureStore};
use crate::instance_buffer::InstanceBuffer;
use crate::pipeline::SpritePipeline;
use crate::sprite::{QuadVertex, RenderInstance};
use ph2d_ecs::PresentWorld;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;

/// One contiguous run of instances that share the same `texture_id`
/// in the sorted scratch buffer. Reused frame-to-frame to keep
/// `render()` allocation-free (HR-3). `start` and `end` index into the
/// instance buffer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct DrawRun {
    pub(crate) texture_id: u32,
    /// Packed per-node sampling key (`RenderInstance::sampling`). For
    /// atlas runs the renderer binds the matching cached sampler
    /// (W3.T3.11); individual textures keep the global store sampler.
    pub(crate) sampling: u32,
    pub(crate) start: u32,
    pub(crate) end: u32,
    /// ClipChildren stencil group (`RenderInstance::clip_group`). `0` =
    /// the normal pass; a non-zero id batches this run into one clip
    /// span (ADR-0070-amendment-7). Part of the run key so a clip-group's
    /// runs never merge with the surrounding normal sprites.
    pub(crate) clip_group: u32,
    /// Clip role of this run (`RenderInstance::CLIP_ROLE_*`): `0` member,
    /// `1` ClipOnly mask source, `2` ClipAndDraw mask source. Part of the
    /// run key so the single mask-source instance always forms its own
    /// run (drawn with the stencil-mark pipeline), separate from members.
    pub(crate) clip_role: u8,
    /// Mask role (`RenderInstance::MASK_ROLE_*`): `0` none, `1` Mask2D
    /// source, `2` responder VisibleInside, `3` responder VisibleOutside.
    /// Part of the run key so mask sources / inside / outside each form
    /// their own runs (drawn with the mark / test / test-outside pipeline).
    pub(crate) mask_role: u8,
    /// Blend-mode tag (`BlendMode::tag()`, `0..5`) read from
    /// `RenderInstance::flip_uv` bits 5-7 (§10). Part of the run key so a
    /// run binds the matching blend pipeline; index `0` (Mix) is the
    /// zero-regression default. Only honored by the normal pass.
    pub(crate) blend: u8,
}

pub struct SpriteRenderer {
    gpu: GpuContext,
    pipeline: SpritePipeline,
    atlas: TextureAtlas,
    /// M14.5 C: per-sprite individually-owned textures. Reuses the
    /// pipeline's `material_bgl` so each entry's bind group can drop
    /// straight into `set_bind_group(1, ...)` at draw time.
    individual: IndividualTextureStore,
    /// KTX2 Fase 2 (W2.T4): cooked GPU-compressed textures. Lazily built on
    /// the first cooked sprite, bound through the SAME `material_bgl`. Its
    /// `texture_id`s live in the [`RenderInstance::COOKED_TEXTURE_ID_BIT`]
    /// namespace so the draw loop routes them here, not to `individual`.
    cooked: crate::cooked_texture::CookedTextureStore,
    instance_buffer: InstanceBuffer,
    quad_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    material_bind_group: wgpu::BindGroup,
    /// W3.T3.11 per-node sampling: atlas bind groups keyed by the packed
    /// `sampling` value, built lazily (atlas texture + the sampler for
    /// that filter/repeat). `BTreeMap` (ADR-0022; the key is `u32`, not
    /// `Entity`). Cleared when `set_filter_mode` rebuilds the atlas view.
    atlas_sampler_bgs: std::collections::BTreeMap<u32, wgpu::BindGroup>,
    /// Reused scratch to avoid per-frame allocation when sprite count
    /// is stable (typical case).
    scratch: Vec<RenderInstance>,
    /// Reused run buffer for the M14.5 C batching pass. Each entry is
    /// one draw call's worth of instances; the renderer walks them in
    /// order, swaps bind group 1 per run, and emits the draw.
    runs: Vec<DrawRun>,
    /// Lazily-allocated `Stencil8` attachment for the ClipChildren clip
    /// pass (W3 §8). `None` until the first frame that contains a clip
    /// group — scenes with no clip never pay for it (zero-regression).
    /// Re-created when the target size changes.
    clip_stencil: Option<ClipStencil>,
    /// **O sub-retângulo que o último `draw_scratch` REALMENTE aplicou** — não o que lhe
    /// pediram. Os dois divergem quando o quadro tem clip/máscara
    /// (`scene_viewport.filter(|_| !has_clip && !has_mask)`), e a divergência é decidida por
    /// CONTEÚDO do quadro, então nenhum chamador a pode prever.
    ///
    /// ⚠️ Existe porque uma sonda de 2026-08-25 imprimiu o valor **pedido** e concluiu que o
    /// passe o honrava — sobre um defeito visível. *Uma sonda que lê o argumento em vez do
    /// efeito responde por quem a escreveu, não pelo produto.* Só diagnóstico: nada no
    /// desenho o lê.
    applied_subrect: Option<[f32; 4]>,
}

/// The `Stencil8` texture + its view + the size it was allocated for.
/// Cached on [`SpriteRenderer`] and reused across frames (HR-3); only
/// re-created when the render target's size changes.
struct ClipStencil {
    view: wgpu::TextureView,
    size: (u32, u32),
}

impl SpriteRenderer {
    pub fn new(
        gpu: GpuContext,
        color_format: wgpu::TextureFormat,
        atlas: TextureAtlas,
        initial_instance_capacity: u32,
    ) -> Self {
        let pipeline = SpritePipeline::new(&gpu, color_format);

        let quad_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render quad vbo"),
            size: std::mem::size_of_val(&QuadVertex::QUAD_STRIP) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(
            &quad_buffer,
            0,
            bytemuck::cast_slice(&QuadVertex::QUAD_STRIP),
        );

        let camera_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render camera ubo"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let frame_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render frame bg"),
            layout: &pipeline.frame_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let material_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render material bg"),
            layout: &pipeline.material_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&atlas.sampler),
                },
            ],
        });

        let instance_buffer = InstanceBuffer::new(&gpu, initial_instance_capacity);
        // The individual store must agree with the atlas on filter
        // mode. The caller built `atlas` (possibly via
        // `TextureAtlas::with_filter`); seed the store from the SAME
        // descriptor so both samplers match from frame 0. Subsequent
        // changes go through `set_filter_mode` which drives both.
        let individual = IndividualTextureStore::new(&gpu);

        Self {
            gpu,
            pipeline,
            atlas,
            individual,
            // Built lazily on the first cooked sprite (W2.T4) — a scene
            // with none never allocates the compressed pipeline.
            cooked: crate::cooked_texture::CookedTextureStore::new(),
            instance_buffer,
            quad_buffer,
            camera_buffer,
            frame_bind_group,
            material_bind_group,
            atlas_sampler_bgs: std::collections::BTreeMap::new(),
            scratch: Vec::with_capacity(initial_instance_capacity as usize),
            // Real workloads keep distinct textures small (typically
            // 1 atlas + a few individuals); 16 is a reasonable seed.
            runs: Vec::with_capacity(16),
            // Allocated lazily on the first clipped frame.
            clip_stencil: None,
            applied_subrect: None,
        }
    }

    /// O sub-retângulo que o ÚLTIMO desenho aplicou de facto (ver
    /// [`Self::applied_subrect`] no campo). `None` = janela cheia — que é o certo fora do
    /// split **e** o sintoma quando um clip/máscara o derruba dentro dele.
    #[must_use]
    pub fn applied_subrect(&self) -> Option<[f32; 4]> {
        self.applied_subrect
    }

    /// Read access to the individual-texture store. Hosts call
    /// `acquire`/`release` directly on this; the renderer itself only
    /// reads `bind_group()` at draw time.
    pub fn individual(&self) -> &IndividualTextureStore {
        &self.individual
    }

    /// Maximum 2D texture dimension supported by the active adapter
    /// (`wgpu::Limits::max_texture_dimension_2d`). Image-edit actions
    /// (Trim, Make Square, future BG Removal) should cap their output
    /// dims against this before calling [`Self::acquire_individual`]
    /// so the user gets a clean toast instead of a deferred device-loss
    /// on the first render that touches the oversize texture.
    pub fn max_texture_dimension_2d(&self) -> u32 {
        self.gpu.device.limits().max_texture_dimension_2d
    }

    /// Which KTX2 compression families the active adapter can sample
    /// (KTX2 Fase 2, W2.T1.5). The loader uses this to pick the richest
    /// cooked tier the GPU supports and fall back to uncompressed when a
    /// family is missing. Captures `device.features()` into a
    /// [`crate::ktx2_format::CompressionFeatureSet`].
    pub fn detect_supported_compressions(&self) -> crate::ktx2_format::CompressionFeatureSet {
        crate::ktx2_format::CompressionFeatureSet::from_features(self.gpu.device.features())
    }

    /// The richest cooked [`ph2d_asset::TierIndex`] this device can sample,
    /// derived from [`Self::detect_supported_compressions`] (W2.T4). This is
    /// the **preferred** tier the loader feeds into
    /// [`ph2d_asset::TierIndex::fallback_ladder`]; it's cheap (one device
    /// feature query) so the loader can call it per upload pass without
    /// caching. On an Apple-Silicon Mac this resolves to `Mobile` (ASTC, no
    /// BC); on a desktop BC adapter, `Desktop`; on a bare WebGPU adapter,
    /// `Constrained` (uncompressed RGBA8).
    #[must_use]
    pub fn active_device_tier(&self) -> ph2d_asset::TierIndex {
        self.detect_supported_compressions().best_tier()
    }

    /// Read access to the cooked-texture store (W2.T4). The extract phase
    /// uses [`CookedTextureStore::texture_id`](crate::cooked_texture::CookedTextureStore::texture_id)
    /// to resolve a sprite's `logical_id` to the cached `texture_id`.
    #[must_use]
    pub fn cooked(&self) -> &crate::cooked_texture::CookedTextureStore {
        &self.cooked
    }

    /// Ensure the cooked KTX2 texture for `logical_id` is decoded, uploaded,
    /// and cached, returning its `texture_id` (in the cooked namespace). The
    /// W2.T4 loader resolves `logical_id + tier → (asset_id, blob)` upstream
    /// (it owns the asset DB + logical map) and hands the bytes here; this
    /// method owns only the GPU upload + cache. Idempotent — a repeat call
    /// for an already-uploaded `logical_id`/`asset_id` is two map lookups.
    ///
    /// # Errors
    /// [`CookedTextureError`]: a corrupt KTX2 blob, or a format this device
    /// cannot sample (the loader should descend its fallback ladder) /
    /// otherwise un-uploadable. Nothing is cached on error.
    pub fn ensure_cooked_texture(
        &mut self,
        logical_id: ph2d_asset::LogicalTextureId,
        asset_id: ph2d_asset::AssetId,
        ktx2_blob: &[u8],
    ) -> Result<u32, crate::cooked_texture::CookedTextureError> {
        self.cooked.ensure(
            &self.gpu,
            &self.pipeline.material_bgl,
            logical_id,
            asset_id,
            ktx2_blob,
        )
    }

    /// The cooked `texture_id` a `logical_id` resolved to, or `None` if it
    /// was never [`ensure_cooked_texture`](Self::ensure_cooked_texture)d.
    /// The extract phase stamps this into the sprite's `RenderInstance`.
    #[must_use]
    pub fn cooked_texture_id(&self, logical_id: ph2d_asset::LogicalTextureId) -> Option<u32> {
        self.cooked.texture_id(logical_id)
    }

    /// Map `logical_id` to the magenta missing-texture placeholder (W2.T4 plan
    /// addendum) so a sprite whose cooked artifact can't be resolved renders
    /// visibly magenta instead of invisibly. The loader calls this after its
    /// fallback ladder is exhausted. Returns the placeholder `texture_id`, or
    /// the error if even the placeholder fails to build (caller stays invisible).
    pub fn mark_cooked_missing(
        &mut self,
        logical_id: ph2d_asset::LogicalTextureId,
    ) -> Result<u32, crate::cooked_texture::CookedTextureError> {
        self.cooked
            .mark_missing(&self.gpu, &self.pipeline.material_bgl, logical_id)
    }

    /// Mutable handle to the individual-texture store. The host's
    /// image-import path acquires textures here when the user
    /// selects the Individual source strategy for a sprite (M14.5
    /// inspector — separate phase).
    pub fn individual_mut(&mut self) -> &mut IndividualTextureStore {
        &mut self.individual
    }

    /// Convenience for the import path: acquire an individual texture
    /// from raw RGBA bytes and return the renderer-side `texture_id`
    /// the caller stamps into `SpriteSource::Individual`.
    pub fn acquire_individual(
        &mut self,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<u32, IndividualTextureError> {
        self.individual
            .acquire(&self.gpu, &self.pipeline.material_bgl, width, height, rgba)
    }

    /// **Irmã de [`Self::acquire_individual`] para a precisão alta** (plano
    /// `docs/Sprite_projeto/18`). `halves` são meio-float **linear**, tal como
    /// [`ph2d_imageio::rgba8_to_rgba16`] os produz.
    ///
    /// ⚠️ **Uma sprite de 16 bits NÃO pode viver no atlas** — ele é uma textura só, com um formato
    /// só. Quem promove uma sprite a 16 bits promove-a também a `Individual`, e é a UI que tem de
    /// o dizer **antes** da conversão, não depois (plano 18 §3.3).
    pub fn acquire_individual_16(
        &mut self,
        width: u32,
        height: u32,
        halves: &[u16],
    ) -> Result<u32, IndividualTextureError> {
        self.individual.acquire_16(
            &self.gpu,
            &self.pipeline.material_bgl,
            width,
            height,
            halves,
        )
    }

    /// O formato GPU de uma textura individual, ou `None` se o id não existir.
    ///
    /// A porta pela qual a chrome descobre a precisão **real** de uma sprite — a linha `Format` do
    /// Inspector tem de dizer o que a textura É, e não o que alguém pediu que fosse. Foi
    /// exactamente essa diferença que o plano 17 §5 removeu de lá.
    #[must_use]
    pub fn individual_format(&self, texture_id: u32) -> Option<wgpu::TextureFormat> {
        self.individual.format(texture_id)
    }

    /// Borrow the renderer's [`GpuContext`] (cheap `Arc`-backed handles).
    /// Tool preview bridges that own a GPU compute pass (the Painter live
    /// preview's `LayerCompositor` + `PreviewPremul`) need it to build and
    /// drive their own pipelines without the renderer re-exporting every wgpu
    /// primitive.
    #[must_use]
    pub fn gpu(&self) -> &GpuContext {
        &self.gpu
    }

    /// Acquire an EMPTY individual texture slot (`width × height`) — no pixel
    /// upload — and return its `texture_id`. The caller fills it the same frame
    /// via [`Self::copy_texture_into_individual`] before it is sampled. The
    /// Painter GPU live preview uses this so a resize doesn't pay a wasted
    /// full-canvas zero upload. Wrapper over
    /// [`IndividualTextureStore::acquire_empty`].
    pub fn acquire_individual_empty(&mut self, width: u32, height: u32) -> u32 {
        self.individual
            .acquire_empty(&self.gpu, &self.pipeline.material_bgl, width, height)
    }

    /// Copy a GPU source texture into an existing individual slot (no CPU
    /// readback). Wrapper over [`IndividualTextureStore::copy_from_texture`] —
    /// the Painter GPU preview blits the premultiplied compositor output
    /// straight into the preview slot.
    pub fn copy_texture_into_individual(
        &mut self,
        texture_id: u32,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        self.individual
            .copy_from_texture(&self.gpu, texture_id, src, width, height)
    }

    /// Copy a SUB-RECT of a GPU source texture into the same sub-rect of an
    /// individual slot (no CPU readback). Dirty-rect sibling of
    /// [`Self::copy_texture_into_individual`] — the Painter E5 live stroke
    /// refreshes only the wet envelope of the preview slot per frame. Wrapper over
    /// [`IndividualTextureStore::copy_region_from_texture`].
    #[allow(clippy::too_many_arguments)]
    pub fn copy_texture_region_into_individual(
        &mut self,
        texture_id: u32,
        src: &wgpu::Texture,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        self.individual.copy_region_from_texture(
            &self.gpu, texture_id, src, src_x, src_y, dst_x, dst_y, width, height,
        )
    }

    /// Encode a full-canvas copy into an individual slot using a CALLER-OWNED
    /// encoder (no submit). Watercolor v2 R1 (ADR-0085 §2.3-I1) seed path: the
    /// shell folds the fluid sim + composite + this copy into one `queue.submit`.
    /// Wrapper over [`IndividualTextureStore::encode_copy_from_texture`].
    pub fn encode_copy_into_individual(
        &self,
        enc: &mut wgpu::CommandEncoder,
        texture_id: u32,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        self.individual
            .encode_copy_from_texture(enc, texture_id, src, width, height)
    }

    /// Encode a dirty-rect copy into an individual slot using a CALLER-OWNED
    /// encoder (no submit). Watercolor v2 R1 (ADR-0085 §2.3-I1/I2) per-frame
    /// refresh: joins the single fluid submit and touches only the wet rect.
    /// Wrapper over [`IndividualTextureStore::encode_copy_region`].
    #[allow(clippy::too_many_arguments)]
    pub fn encode_copy_region_into_individual(
        &self,
        enc: &mut wgpu::CommandEncoder,
        texture_id: u32,
        src: &wgpu::Texture,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        self.individual.encode_copy_region(
            enc, texture_id, src, src_x, src_y, dst_x, dst_y, width, height,
        )
    }

    /// Convenience for the image-edit path: copy an individual
    /// texture's GPU contents back to a `Vec<u8>` (RGBA8, tightly
    /// packed). Used by Trim Transparency / Background Removal when
    /// the source sprite is already on an individual texture and the
    /// shell needs the current pixels to feed the next edit.
    ///
    /// One-shot, blocking — see [`IndividualTextureStore::readback`]
    /// for the cost model. Not for per-frame use.
    pub fn readback_individual(
        &self,
        texture_id: u32,
    ) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
        // ⚠️ **`readback_rgba8` e não `readback`.** Esta é a porta do SHELL, e as ferramentas de
        // imagem a jusante trabalham em `Vec<u8>` de 4 bytes por pixel. Devolver o buffer cru de
        // uma textura de 16 bits daria o **dobro** dos bytes para as mesmas dimensões e o
        // consumidor leria pares de bytes como cores (pânico reportado em 2026-08-20).
        self.individual.readback_rgba8(&self.gpu, texture_id)
    }

    /// **Os texels CRUS de uma textura de 16 bits**, ou `None` se ela não for de 16 bits.
    ///
    /// ⚠️ Serve as ferramentas que **não calculam valor de pixel nenhum** — trim, make-square,
    /// padding: elas só copiam e preenchem, por isso a precisão pode atravessá-las **exacta**. Sem
    /// esta porta a única leitura era a normalizada para 8 bits, e essas três destruíam 16 bits por
    /// nada (Enio, 2026-08-20: *"após aplicar algumas das tools a sprite volta para RGBA8"*).
    pub fn readback_individual_16(&self, texture_id: u32) -> Option<(u32, u32, Vec<u16>)> {
        if self.individual.format(texture_id) != Some(IndividualTextureStore::FORMAT_16) {
            return None;
        }
        let (w, h, bytes) = self.individual.readback(&self.gpu, texture_id).ok()?;
        Some((
            w,
            h,
            bytes
                .chunks_exact(2)
                .map(|p| u16::from_le_bytes([p[0], p[1]]))
                .collect(),
        ))
    }

    /// Convenience wrapper around [`IndividualTextureStore::replace_pixels`]
    /// that hides the renderer-internal `GpuContext` + `material_bgl`
    /// from callers. Used by tool live-preview bridges (BG-Removal,
    /// 2026-05-26) that own a transient texture slot and refresh its
    /// contents whenever the CPU-side preview cache produces a new
    /// frame. Mirrors the `acquire_individual` ergonomics.
    pub fn replace_individual_pixels(
        &mut self,
        texture_id: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        self.individual.replace_pixels(
            &self.gpu,
            &self.pipeline.material_bgl,
            texture_id,
            width,
            height,
            rgba,
        )
    }

    /// Upload only a sub-rectangle into an existing individual texture,
    /// leaving the rest untouched. Renderer-side wrapper over
    /// [`IndividualTextureStore::replace_pixels_region`] — the dirty-rect
    /// path (Painter stroke preview) uploads just the stamp's bbox instead
    /// of the whole canvas. No bind-group rebuild (dims are unchanged), so
    /// `material_bgl` is not needed. `region_rgba` is the tightly-packed
    /// `width * height * 4` bytes for the sub-rect alone; the region must lie
    /// within the texture's current dims.
    // x/y/w/h sub-rect form mirrors the store method + `write_texture`.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_individual_pixels_region(
        &mut self,
        texture_id: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        region_rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        self.individual.replace_pixels_region(
            &self.gpu,
            texture_id,
            x,
            y,
            width,
            height,
            region_rgba,
        )
    }

    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    /// **A view e as dimensões por trás de um `texture_id`** — a porta ÚNICA para quem precisa
    /// de LIGAR a textura de uma sprite em vez de a desenhar (doc 89 folha 11).
    ///
    /// ⚠️ **Ela existe porque um `texture_id` responde a outra pergunta.** O passe de sprites
    /// resolve um id para o **bind group** que o material dele quer (`@group(1)`), e é isso que
    /// as três lojas expõem hoje. Um passe de TELA — o composite do halo, com a máscara de
    /// sujidade — tem um layout próprio, então o que ele precisa é da `TextureView` crua, para
    /// pôr no bind group DELE. Era essa a peça que a célula da folha 11 chamou de *"a fiação
    /// cara"*; a outra metade que ela precificava (resolver as três variantes de
    /// `SpriteSource`) já tinha sido construída pela folha 14.
    ///
    /// ⚠️ **UMA porta com três braços, e não três chamadas do lado do consumidor**: o
    /// espaço de ids é uma convenção deste crate (`ATLAS_TEXTURE_ID`, o bit dos assados, e os
    /// individuais no resto), e um consumidor que a reimplemente é a segunda resposta que
    /// diverge quando um quarto tipo de loja nascer. As dimensões vêm juntas porque quem liga
    /// uma imagem a um passe de tela precisa **sempre** do aspecto dela, e pedi-las noutra
    /// chamada seria a segunda oportunidade de as ir buscar à loja errada.
    ///
    /// `None` para um id que nenhuma loja conhece (uma textura ainda a carregar, ou já
    /// libertada) — *não adivinha e não falha*.
    #[must_use]
    pub fn texture_view_and_dims(&self, texture_id: u32) -> Option<(&wgpu::TextureView, u32, u32)> {
        if texture_id == RenderInstance::ATLAS_TEXTURE_ID {
            let n = self.atlas.size_px;
            return Some((&self.atlas.view, n, n));
        }
        if RenderInstance::is_cooked_texture_id(texture_id) {
            let (w, h) = self.cooked.dims(texture_id)?;
            return Some((self.cooked.view(texture_id)?, w, h));
        }
        let (w, h) = self.individual.dims(texture_id)?;
        Some((self.individual.view(texture_id)?, w, h))
    }

    /// Insert a freshly-decoded source image into the renderer's
    /// atlas at native resolution, returning the packed region on
    /// success. Wraps [`TextureAtlas::insert`] so callers (the
    /// image-import path in `shells/desktop`) don't need to thread
    /// the [`GpuContext`] themselves.
    ///
    /// `rgba` must be tightly-packed `width * height * 4` bytes.
    /// Errors when the source is bigger than the atlas, or when
    /// the packer's skyline is exhausted. See
    /// [`AtlasInsertError`](crate::atlas::AtlasInsertError).
    pub fn insert_atlas_sprite(
        &mut self,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<crate::atlas::AtlasRegion, crate::atlas::AtlasInsertError> {
        self.atlas.insert(&self.gpu, key, width, height, rgba)
    }

    /// Atlas V2 (M14.4f): insert with automatic regrow on
    /// `AtlasFull`. The atlas doubles its texture (capped at
    /// `atlas.max_size_px()`), re-packs every existing region using
    /// `fetch_pixels(key)` to recover the source bytes, rebuilds the
    /// material bind group to point at the new texture, then retries
    /// the original insert.
    ///
    /// `fetch_pixels` is the caller's hook into wherever it stores
    /// the asset source bytes (typically `AssetDb::get(asset_id) →
    /// Asset::ImageRgba8 { pixels, ... }`). Returning `None` for a
    /// key drops that region from the post-regrow atlas; surviving
    /// keys keep their old indices so render instances need no
    /// patching.
    ///
    /// Errors when the atlas is already at its cap, or when the
    /// caller's source dimensions exceed `max_size_px`.
    pub fn insert_atlas_sprite_with_regrow<F>(
        &mut self,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
        fetch_pixels: F,
    ) -> Result<crate::atlas::AtlasRegion, crate::atlas::AtlasInsertError>
    where
        F: Fn(u32) -> Option<Vec<u8>>,
    {
        match self.atlas.insert(&self.gpu, key, width, height, rgba) {
            Ok(region) => Ok(region),
            Err(crate::atlas::AtlasInsertError::AtlasFull { .. }) => {
                let cap = self.atlas.max_size_px();
                let target = (self.atlas.size_px.saturating_mul(2)).min(cap);
                if target <= self.atlas.size_px {
                    // Already at cap — surface the original error so
                    // the shell can toast something actionable.
                    return Err(crate::atlas::AtlasInsertError::AtlasFull { width, height });
                }
                self.atlas.regrow_inplace(&self.gpu, target, fetch_pixels)?;
                self.rebuild_material_bind_group();
                // Retry — pure path now (no further regrow needed at
                // this granularity; the source that triggered full
                // necessarily fits in a 2× atlas if it fit pre-regrow
                // at all).
                self.atlas.insert(&self.gpu, key, width, height, rgba)
            }
            Err(other) => Err(other),
        }
    }

    /// Release `key`'s atlas slot back into the free-list. The next
    /// `insert_atlas_sprite` of the same dimensions will reuse the
    /// slot without re-packing. Returns the freed region for the
    /// caller's diagnostics, or `None` if `key` wasn't inserted.
    pub fn remove_atlas_sprite(&mut self, key: u32) -> Option<crate::atlas::AtlasRegion> {
        self.atlas.remove(key)
    }

    /// Rebuild the material bind group against the current
    /// `atlas.view`. Called internally after
    /// [`Self::insert_atlas_sprite_with_regrow`] re-creates the
    /// atlas texture; if a future path mutates the atlas externally
    /// (e.g. hot-reload swapping the whole texture) it can call this
    /// to keep the renderer pointing at fresh pixels.
    pub fn rebuild_material_bind_group(&mut self) {
        self.material_bind_group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render material bg (regrown)"),
                layout: &self.pipeline.material_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.atlas.sampler),
                    },
                ],
            });
        // The per-sampling atlas bind groups reference the (now stale)
        // old atlas view — drop them so they rebuild against the new
        // view on the next render (W3.T3.11).
        self.atlas_sampler_bgs.clear();
    }

    /// Switch the global sprite-sampling mode at runtime. Recreates the
    /// atlas sampler + the individual-store samplers, then rebuilds the
    /// bind groups that referenced the old samplers (the atlas's
    /// `material_bind_group` and every individual entry's bind group).
    ///
    /// No texture is re-uploaded — only how the existing pixels are
    /// sampled changes — so this is cheap (a handful of sampler +
    /// bind-group allocations) and safe to call from the Settings menu
    /// handler on every toggle.
    pub fn set_filter_mode(&mut self, mode: crate::ImageFilterMode) {
        self.atlas.set_filter_mode(&self.gpu, mode);
        self.individual
            .set_filter_mode(&self.gpu, &self.pipeline.material_bgl, mode);
        // The shared-atlas material bind group baked in the old atlas
        // sampler; rebuild it against the freshly-created one.
        self.rebuild_material_bind_group();
    }

    /// Render every `RenderInstance` in `present` into `target`.
    /// Loads with `clear_color` (single pass; M6+ may compose multiple).
    ///
    /// M14.5: `target` is now a generic `&wgpu::TextureView` so the
    /// caller can route the output into either the swap chain (legacy
    /// fixture demo) or an offscreen [`GameRt`](crate::GameRt) (live
    /// editor mode with compositor pass). The pipeline's color format
    /// (`color_format` passed to `SpritePipeline::new`) MUST match
    /// whatever this view points at — mismatch is a wgpu validation
    /// error caught at first draw.
    /// Lazily build + cache the atlas bind group for a packed `sampling`
    /// key (W3.T3.11): the atlas texture + a sampler with the resolved
    /// filter/repeat. The bind group keeps the sampler alive internally,
    /// so the local handle can drop.
    fn ensure_atlas_sampler_bg(&mut self, sampling: u32) {
        if self.atlas_sampler_bgs.contains_key(&sampling) {
            return;
        }
        let (filter_tag, repeat_tag) = RenderInstance::unpack_sampling(sampling);
        let sampler =
            crate::image_filter::sampler_from_tags(&self.gpu.device, filter_tag, repeat_tag);
        let bg = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ph2d-render atlas per-sampling bg"),
                layout: &self.pipeline.material_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.atlas.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
        self.atlas_sampler_bgs.insert(sampling, bg);
    }

    /// Ensure the `Stencil8` clip attachment exists at `size`, (re)creating
    /// it on first use or a size change (HR-3: reused otherwise). The
    /// `TextureView` keeps the texture alive, so the texture handle can drop.
    fn ensure_clip_stencil(&mut self, size: (u32, u32)) {
        if self.clip_stencil.as_ref().map(|s| s.size) == Some(size) {
            return;
        }
        let tex = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-render clip stencil"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::pipeline::STENCIL_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.clip_stencil = Some(ClipStencil { view, size });
    }
}

#[path = "renderer_draw.rs"]
mod draw;

/// Walk the sorted instance slice and emit one [`DrawRun`] per maximal
/// contiguous run of the same `(texture_id, sampling)` pair. Reuses
/// `runs`'s capacity. `scratch` MUST already be sorted so those pairs
/// are contiguous (the caller sorts by `(z_order, texture_id,
/// sampling)`).
fn compute_runs(scratch: &[RenderInstance], runs: &mut Vec<DrawRun>) {
    runs.clear();
    if scratch.is_empty() {
        return;
    }
    // Key also on `(clip_group, clip_role)` so a clip-group's instances
    // never merge into a neighbouring normal run, and the single mask
    // source forms its own run apart from its members (W3 §8).
    let key = |i: &RenderInstance| {
        (
            i.texture_id,
            i.sampling,
            i.clip_group,
            RenderInstance::clip_role(i.clip_meta),
            RenderInstance::mask_role(i.clip_meta),
            RenderInstance::unpack_blend(i.flip_uv),
        )
    };
    let emit = |runs: &mut Vec<DrawRun>, k: (u32, u32, u32, u8, u8, u8), start: u32, end: u32| {
        runs.push(DrawRun {
            texture_id: k.0,
            sampling: k.1,
            start,
            end,
            clip_group: k.2,
            clip_role: k.3,
            mask_role: k.4,
            blend: k.5,
        });
    };
    let mut start = 0u32;
    let mut current = key(&scratch[0]);
    for (i, inst) in scratch.iter().enumerate().skip(1) {
        if key(inst) != current {
            emit(runs, current, start, i as u32);
            current = key(inst);
            start = i as u32;
        }
    }
    emit(runs, current, start, scratch.len() as u32);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(texture_id: u32) -> RenderInstance {
        RenderInstance {
            world_pos: [0.0, 0.0],
            size: [1.0, 1.0],
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            basis: RenderInstance::IDENTITY_BASIS,
            texture_id,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            z_order: 0,
            sampling: 0,
            uv_xform: RenderInstance::IDENTITY_UV_XFORM,
            clip_group: RenderInstance::CLIP_GROUP_NONE,
            clip_meta: 0,
            sub_order: 0,
        }
    }

    #[test]
    fn compute_runs_empty_input_emits_no_runs() {
        let mut runs = Vec::new();
        compute_runs(&[], &mut runs);
        assert!(runs.is_empty());
    }

    #[test]
    fn compute_runs_groups_consecutive_same_texture() {
        // Pre-sorted: [0, 0, 0, 7, 7, 12]
        let scratch = [inst(0), inst(0), inst(0), inst(7), inst(7), inst(12)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(
            runs,
            vec![
                DrawRun {
                    texture_id: 0,
                    sampling: 0,
                    start: 0,
                    end: 3,
                    clip_group: 0,
                    clip_role: 0,
                    mask_role: 0,
                    blend: 0,
                },
                DrawRun {
                    texture_id: 7,
                    sampling: 0,
                    start: 3,
                    end: 5,
                    clip_group: 0,
                    clip_role: 0,
                    mask_role: 0,
                    blend: 0,
                },
                DrawRun {
                    texture_id: 12,
                    sampling: 0,
                    start: 5,
                    end: 6,
                    clip_group: 0,
                    clip_role: 0,
                    mask_role: 0,
                    blend: 0,
                },
            ]
        );
    }

    #[test]
    fn compute_runs_singleton_per_instance() {
        // All distinct textures → N runs of 1.
        let scratch = [inst(1), inst(2), inst(3)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(runs.len(), 3);
        for r in &runs {
            assert_eq!(r.end - r.start, 1);
        }
    }

    #[test]
    fn compute_runs_reuses_vec_capacity() {
        // First call grows the vec; second call with a different
        // shape must NOT allocate again (HR-3 alloc-free hot path).
        let mut runs = Vec::with_capacity(8);
        let cap = runs.capacity();
        compute_runs(&[inst(0), inst(0)], &mut runs);
        compute_runs(&[inst(5), inst(6), inst(7), inst(7)], &mut runs);
        assert!(runs.capacity() >= cap, "capacity must not shrink");
        assert_eq!(runs.len(), 3);
    }

    #[test]
    fn compute_runs_atlas_constant() {
        let scratch = [inst(RenderInstance::ATLAS_TEXTURE_ID)];
        let mut runs = Vec::new();
        compute_runs(&scratch, &mut runs);
        assert_eq!(runs[0].texture_id, RenderInstance::ATLAS_TEXTURE_ID);
        assert_eq!(runs[0].texture_id, 0);
    }
}
