//! M14.5 C — individual-texture sprite source.
//!
//! Companion to [`crate::atlas::TextureAtlas`]: while the atlas packs
//! many sprites into one shared 4096² texture and renders them in a
//! single draw call, this store gives each sprite its **own**
//! `wgpu::Texture` at the source's native resolution. The renderer
//! groups consecutive same-texture instances into one draw call each
//! (Godot 4 `RenderingServer` pattern) — a pure-CPU sort step that
//! amortizes well when sprite count stays under a few thousand.
//!
//! ## When to pick this over the atlas
//!
//! - **HD 2D content** (Cuphead-tier) where each sprite is large enough
//!   that packing-and-batching wins nothing over per-sprite textures.
//! - **Procedural / hot-reloaded textures** that change dimensions
//!   between reloads — atlas regrow is more expensive than swapping a
//!   single texture handle.
//! - **Mixed-resolution sprites** where the atlas's Skyline packer
//!   would either waste space or evict on every regrow.
//!
//! For tile-sets, UI icons, and same-shape sprite sheets, prefer the
//! shared atlas — it's still one draw call per frame.
//!
//! ## Lifecycle contract
//!
//! Callers (typically the image-import path in `shells/desktop`) must
//! pair every [`IndividualTextureStore::acquire`] with a
//! [`IndividualTextureStore::release`] when the owning sprite
//! despawns. Refcounting catches the common case where the same
//! `AssetId` is referenced by multiple sprites — the texture is held
//! until the last sprite releases it.
//!
//! HR-5 / ADR-0022: uses `BTreeMap`, not `HashMap`, so the iteration
//! order over textures stays deterministic for tests that count
//! distinct runs in [`crate::renderer::SpriteRenderer`].

use ph2d_gpu::GpuContext;
use std::collections::BTreeMap;

/// One individually-owned texture, with a pre-built bind group sized
/// against the renderer's `material_bgl` so the per-frame batcher can
/// `set_bind_group(1, ...)` without re-creating it.
pub struct IndividualTextureEntry {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
    /// Mip levels in `texture` (`mipgen::mip_levels(width, height)`). The
    /// generator fills `1..mip_count` after each content write.
    pub mip_count: u32,
    /// O formato desta textura — `Rgba8UnormSrgb` (default) ou `Rgba16Float`
    /// (precisão alta, plano `docs/Sprite_projeto/18`).
    ///
    /// ⚠️ **É a chave que escolhe o gerador de mips.** Um `MipGenerator` é construído contra UM
    /// formato; correr o de 8 bits sobre uma textura de 16 é erro de validação do wgpu, não um
    /// resultado errado — mas o modo de falha silencioso está mesmo ao lado, e é por isso que o
    /// formato viaja no entry em vez de ser adivinhado.
    pub format: wgpu::TextureFormat,
    /// Sprites currently referencing this texture. Drops to 0 →
    /// [`IndividualTextureStore::release`] removes the entry and the
    /// `wgpu::Texture` handle drops.
    pub refcount: u32,
    /// **Os bind groups por AMOSTRAGEM** — o gémeo do `atlas_sampler_bgs` do renderer,
    /// chaveado pela `RenderInstance::sampling` (`filter | repeat << 8`).
    ///
    /// ⚠️ **A ausência disto era um defeito de produto, achado em 2026-08-25** (doc 89,
    /// folha 17): o `material_bg` do `renderer_draw` honrava a `sampling` **só para o
    /// átlas** e para toda textura individual chamava `bind_group(id)`, que devolve UM
    /// grupo construído contra o sampler DEFAULT DO PROJECTO. ⇒ o filtro por-nó do
    /// Inspector (§9) estava **inerte em toda textura individual do app** — e uma sprite
    /// promovida a Individual por um `commit_edited_texture` perdia o filtro dela **em
    /// silêncio**. O caso que o expôs é o que o filtro existe para servir: *pixel-art*,
    /// que chega por importação e portanto quase nunca está no átlas partilhado.
    ///
    /// ⚠️ **A cache mora no ENTRY e não no renderer, de propósito**: ela referencia a
    /// `view` desta textura, então tem de morrer exactamente quando ela morre. No
    /// renderer, um `release` seguido de um `acquire` que reciclasse o id devolveria um
    /// grupo a apontar para uma view liberta — e o modo de falha de uma view morta não é
    /// um erro, é o texel errado.
    ///
    /// ⚠️ **`sampling = 0` NÃO entra aqui**: `0` quer dizer *herda o default do projecto*,
    /// que é precisamente o que o [`Self::bind_group`] já é — e que o
    /// [`IndividualTextureStore::set_filter_mode`] reconstrói quando esse default muda.
    /// Um `0` cacheado aqui congelaria o default no valor que ele tinha no 1.º desenho.
    pub sampler_bgs: std::collections::BTreeMap<u32, wgpu::BindGroup>,
}

/// Renderer-side cache of individual sprite textures, keyed by a
/// monotonically-allocated `u32` id (the same value stored in
/// `Sprite::source = SpriteSource::Individual { texture_id }`).
///
/// Id `0` is reserved as the "atlas" sentinel — see
/// [`crate::RenderInstance::ATLAS_TEXTURE_ID`]. The store starts
/// allocation at `1`.
pub struct IndividualTextureStore {
    /// ⚠️ `pub(crate)` porque os irmãos `individual_entry` e `individual_read` os leem — o corte
    /// por LOC partiu o `impl`, não a propriedade: nada fora desta crate lhes toca.
    pub(crate) entries: BTreeMap<u32, IndividualTextureEntry>,
    next_id: u32,
    pub(crate) sampler: wgpu::Sampler,
    /// Regenerates each entry's mip chain after a content write so a minified
    /// (zoomed-out) sprite samples trilinearly instead of undersampling its
    /// antialiased edges into jaggies (2026-06-17 fix).
    ///
    /// ⚠️ **Um gerador por FORMATO, e não um para o store.** Esta nota dizia *"all individual
    /// textures are `Rgba8UnormSrgb`, so one generator serves the whole store"* e deixou de ser
    /// verdade com o plano `docs/Sprite_projeto/18`. Um `MipGenerator` é construído contra um
    /// formato concreto (é ele que define o pipeline do blit), por isso são dois.
    mip_gen: crate::mipgen::MipGenerator,
    /// O irmão de 16 bits do [`Self::mip_gen`]. Construído por omissão junto com o outro: um
    /// pipeline de blit é barato, e construí-lo preguiçosamente exigiria `&mut self` em
    /// [`Self::regen_mips`], que hoje é `&self` e é chamado de sítios que só têm leitura.
    mip_gen_16: crate::mipgen::MipGenerator,
}

/// Errors returned by [`IndividualTextureStore::acquire`] and
/// [`IndividualTextureStore::readback`].
#[derive(Debug)]
pub enum IndividualTextureError {
    PixelLengthMismatch {
        got: usize,
        expected: usize,
    },
    /// `readback`'s requested texture id has no entry in the store.
    NotFound(u32),
    /// A [`IndividualTextureStore::replace_pixels_region`] sub-rect lies
    /// outside the entry's current texture dimensions (a partial write
    /// past the edge would corrupt neighbouring rows or panic in wgpu).
    RegionOutOfBounds {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
    /// **Um caminho de 8 bits foi apontado a uma textura de 16.**
    ///
    /// ⚠️ Existe porque o modo de falha alternativo é **corrupção silenciosa**: escrever bytes com
    /// passo de linha `w × 4` numa textura de `w × 8` não dá erro do wgpu (a validação só exige
    /// `bytes_per_row >= w × block`), preenche metade de cada linha e deixa a outra metade com o
    /// que lá estava. O sintoma seria a imagem esticada ao meio, sem uma palavra.
    ///
    /// Irmão do pânico de 2026-08-20 — o mesmo erro de *stride*, do lado da escrita.
    EightBitWriteToSixteenBitTexture {
        id: u32,
    },
    /// The GPU command queue accepted the copy but the buffer never
    /// finished mapping. Worth surfacing distinctly from a generic
    /// I/O error so the caller can decide whether to retry (device
    /// likely lost — see ADR-0020) or fail loudly.
    ReadbackFailed(String),
    /// A [`IndividualTextureStore::copy_from_texture`] source texture's
    /// dimensions did not match the destination entry's. A `copy_texture_to_
    /// texture` past the edge would be a wgpu validation error, so reject it
    /// at the boundary with a precise diagnostic.
    CopySizeMismatch {
        width: u32,
        height: u32,
        tex_width: u32,
        tex_height: u32,
    },
}

impl std::fmt::Display for IndividualTextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PixelLengthMismatch { got, expected } => write!(
                f,
                "rgba buffer length {got} doesn't match width*height*4 = {expected}"
            ),
            Self::NotFound(id) => write!(f, "no individual texture with id {id}"),
            Self::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "region {width}×{height} at ({x},{y}) exceeds texture {tex_width}×{tex_height}"
            ),
            Self::EightBitWriteToSixteenBitTexture { id } => write!(
                f,
                "refused an 8-bit write to the 16-bit individual texture {id}: the row stride \
                 differs (w*4 vs w*8), so the write would fill half of every row and leave the \
                 rest stale, silently"
            ),
            Self::ReadbackFailed(detail) => write!(f, "GPU readback failed: {detail}"),
            Self::CopySizeMismatch {
                width,
                height,
                tex_width,
                tex_height,
            } => write!(
                f,
                "copy source {width}×{height} doesn't match texture {tex_width}×{tex_height}"
            ),
        }
    }
}

impl std::error::Error for IndividualTextureError {}

impl IndividualTextureStore {
    pub fn new(gpu: &GpuContext) -> Self {
        Self::with_filter(gpu, crate::ImageFilterMode::default())
    }

    /// Build the store with an explicit [`ImageFilterMode`]. The
    /// sampler is the SINGLE canonical sprite sampler
    /// ([`crate::create_sprite_sampler`]) shared with the atlas — so a
    /// sprite baked into an Individual texture (e.g. BG-Removal Apply)
    /// samples identically to an atlas sprite. This fixes the
    /// smooth-preview/pixelated-bake divergence: before, this sampler
    /// hardcoded `Nearest` while the atlas hardcoded `Linear`.
    pub fn with_filter(gpu: &GpuContext, filter: crate::ImageFilterMode) -> Self {
        let sampler = crate::create_sprite_sampler(
            &gpu.device,
            filter,
            "ph2d-render individual texture sampler",
        );
        Self {
            entries: BTreeMap::new(),
            // 1 because 0 is reserved for "shared atlas".
            next_id: 1,
            sampler,
            mip_gen: crate::mipgen::MipGenerator::new(gpu, wgpu::TextureFormat::Rgba8UnormSrgb),
            mip_gen_16: crate::mipgen::MipGenerator::new(gpu, Self::FORMAT_16),
        }
    }

    /// **O formato das texturas de precisão alta.**
    ///
    /// ⚠️ **`Rgba16Float` e não `Rgba16Unorm`, e é medição:** o unorm exige
    /// `Features::TEXTURE_FORMAT_16BIT_NORM`, que o [`ph2d_gpu`] pede **mascarada pelo adapter** —
    /// pode não existir na máquina. O float é baseline do WebGPU, filtrável (logo serve o MESMO
    /// `material_bgl`, que declara `Float { filterable: true }`), e já é a moeda do `GameRt` e do
    /// `fx_stack`.
    pub const FORMAT_16: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    /// Regenerate mip levels `1..` of an entry from its freshly-written level 0.
    /// No-op for an unknown id or a single-level (tiny) texture.
    pub(crate) fn regen_mips(&self, gpu: &GpuContext, id: u32) {
        if let Some(entry) = self.entries.get(&id) {
            // ⚠️ O gerador escolhe-se pelo formato DO ENTRY, nunca por um default. Um `MipGenerator`
            // traz o seu pipeline de blit amarrado a um formato; o errado é erro de validação.
            let generator = if entry.format == Self::FORMAT_16 {
                &self.mip_gen_16
            } else {
                &self.mip_gen
            };
            generator.run(gpu, &entry.texture, entry.mip_count);
        }
    }

    /// Total number of individually-owned textures currently held.
    /// Used by tests and the future Inspector telemetry. Excludes the
    /// shared atlas.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Upload `rgba` (tightly-packed width*height*4 bytes) to a new
    /// individually-owned texture and return its renderer-side
    /// `texture_id`. Refcount starts at 1.
    ///
    /// Errors only on pixel-length mismatch — the GPU side is
    /// fire-and-forget (`queue.write_texture`). Validation issues
    /// (size too large for the device limit) surface at first render.
    pub fn acquire(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<u32, IndividualTextureError> {
        let expected = (width as usize) * (height as usize) * 4;
        if rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: rgba.len(),
                expected,
            });
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("IndividualTextureStore: u32 id space exhausted");
        let entry = crate::individual_entry::create_entry(
            gpu,
            material_bgl,
            &self.sampler,
            width,
            height,
            rgba,
        );
        self.entries.insert(id, entry);
        self.regen_mips(gpu, id);
        Ok(id)
    }

    /// **Irmã de [`Self::acquire`] para a precisão alta** (plano `docs/Sprite_projeto/18`, W2).
    ///
    /// `halves` são `width × height × 4` bits de meio-float em espaço **linear** — o que
    /// [`ph2d_imageio::rgba8_to_rgba16`] produz, e o que a textura [`Self::FORMAT_16`] consome sem
    /// conversão nenhuma.
    ///
    /// ⚠️ **O id sai do MESMO contador do caminho de 8 bits**, de propósito: um `texture_id` é uma
    /// referência do `SpriteSource::Individual`, e dois espaços de id separados fariam duas sprites
    /// de precisões diferentes colidirem no mesmo número. *Números que somam entre caminhos
    /// contam-se uma vez só.*
    ///
    /// ⚠️ O bind group sai do MESMO `material_bgl` — ele declara
    /// `TextureSampleType::Float { filterable: true }`, que `Rgba16Float` satisfaz. É por isso que
    /// esta wave não precisa de um segundo pipeline nem de um ramo no shader.
    pub fn acquire_16(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
        halves: &[u16],
    ) -> Result<u32, IndividualTextureError> {
        let expected = (width as usize) * (height as usize) * 4;
        if halves.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: halves.len(),
                expected,
            });
        }
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("IndividualTextureStore: u32 id space exhausted");
        let entry = crate::individual_entry::create_entry_16(
            gpu,
            material_bgl,
            &self.sampler,
            width,
            height,
            halves,
        );
        self.entries.insert(id, entry);
        self.regen_mips(gpu, id);
        Ok(id)
    }

    /// O formato de uma entrada, ou `None` se o id não existir. A porta pela qual um chamador
    /// descobre a precisão sem alcançar o `wgpu::Texture`.
    #[must_use]
    pub fn format(&self, id: u32) -> Option<wgpu::TextureFormat> {
        self.entries.get(&id).map(|e| e.format)
    }

    /// Allocate an EMPTY individually-owned texture (`width × height`,
    /// `Rgba8UnormSrgb`, `COPY_DST`) and return its `texture_id`. Refcount
    /// starts at 1. Unlike [`Self::acquire`], no pixels are uploaded — the
    /// texture contents are undefined until the caller fills the slot (e.g.
    /// [`Self::copy_from_texture`] from a GPU compositor output the same
    /// frame, before it is ever sampled). The Painter GPU live preview uses
    /// this to avoid a wasted full-canvas zero upload on every resize.
    pub fn acquire_empty(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        width: u32,
        height: u32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("IndividualTextureStore: u32 id space exhausted");
        let entry = crate::individual_entry::create_entry_empty(
            gpu,
            material_bgl,
            &self.sampler,
            width,
            height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        self.entries.insert(id, entry);
        id
    }

    /// Copy a `width × height` source texture into an existing entry's texture
    /// via a GPU texture-to-texture copy — no CPU readback. Used by the Painter
    /// GPU live preview: the layer compositor's straight-sRGB8 `rgba8unorm`
    /// output (after the premultiply blit) is copied byte-for-byte into the
    /// `Rgba8UnormSrgb` preview slot. The two formats are COPY-COMPATIBLE
    /// (`remove_srgb_suffix()` agrees — same texel block; sRGB-ness differs
    /// only at sample time, which is exactly the premultiplied → linear decode
    /// the sprite shader expects). `src` must carry `COPY_SRC`; the entry's
    /// texture already carries `COPY_DST`.
    ///
    /// Errors on an unknown id or a size mismatch. A zero-area copy is a no-op.
    pub fn copy_from_texture(
        &self,
        gpu: &GpuContext,
        id: u32,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if entry.width != width || entry.height != height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width,
                height,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render individual copy_from_texture encoder"),
            });
        // GPU pass profiler (PH2D_FLUID_PROFILE): copies can't carry pass
        // timestamps on Metal, so bracket with empty marker passes. No-op off.
        let prof_span = ph2d_gpu::pass_profiler::copy_span_begin(&mut encoder, "copy.slot");
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        if let Some(t) = prof_span {
            ph2d_gpu::pass_profiler::copy_span_end(&mut encoder, t);
        }
        gpu.queue.submit([encoder.finish()]);
        self.regen_mips(gpu, id);
        Ok(())
    }

    /// GPU→GPU copy of a SUB-RECT of `src` into the same sub-rect of slot `id`,
    /// leaving the rest of the slot untouched. `src_origin` is where the rect
    /// starts in `src`; `dst_x`/`dst_y` where it lands in the slot; `w`/`h` its
    /// size. The dirty-rect sibling of [`Self::copy_from_texture`] — the Painter
    /// E5 live stroke refreshes only the wet envelope of the preview slot instead
    /// of re-copying the whole canvas every frame. The rect must lie within both
    /// textures (caller clamps); an empty rect is a no-op.
    #[allow(clippy::too_many_arguments)]
    pub fn copy_region_from_texture(
        &self,
        gpu: &GpuContext,
        id: u32,
        src: &wgpu::Texture,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if dst_x + w > entry.width || dst_y + h > entry.height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width: dst_x + w,
                height: dst_y + h,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if w == 0 || h == 0 {
            return Ok(());
        }
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render individual copy_region_from_texture encoder"),
            });
        let prof_span = ph2d_gpu::pass_profiler::copy_span_begin(&mut encoder, "copy.region");
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src_x,
                    y: src_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst_x,
                    y: dst_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        if let Some(t) = prof_span {
            ph2d_gpu::pass_profiler::copy_span_end(&mut encoder, t);
        }
        gpu.queue.submit([encoder.finish()]);
        self.regen_mips(gpu, id);
        Ok(())
    }

    /// **Encode-only sibling of [`Self::copy_from_texture`]** (Watercolor v2 R1,
    /// ADR-0085 §2.3-I1): the full-canvas copy is encoded into the caller's `enc`
    /// (NO submit), so the shell folds the fluid sim, composite and this seed copy
    /// into ONE `queue.submit`. Used once to seed the preview slot's backdrop; the
    /// per-frame refresh uses [`Self::encode_copy_region`]. Same validation as the
    /// wrapper; a zero-area copy is a no-op.
    pub fn encode_copy_from_texture(
        &self,
        enc: &mut wgpu::CommandEncoder,
        id: u32,
        src: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if entry.width != width || entry.height != height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width,
                height,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if width == 0 || height == 0 {
            return Ok(());
        }
        let prof_span = ph2d_gpu::pass_profiler::copy_span_begin(enc, "copy.slot");
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        if let Some(t) = prof_span {
            ph2d_gpu::pass_profiler::copy_span_end(enc, t);
        }
        Ok(())
    }

    /// **Encode-only sibling of [`Self::copy_region_from_texture`]** (Watercolor v2 R1,
    /// ADR-0085 §2.3-I1/I2): the dirty-rect copy is encoded into the caller's `enc`
    /// (NO submit), so the per-frame preview refresh joins the single fluid submit AND
    /// only touches the wet rect (no full-canvas bandwidth). The rect must lie within
    /// both textures (caller clamps); an empty rect is a no-op.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_copy_region(
        &self,
        enc: &mut wgpu::CommandEncoder,
        id: u32,
        src: &wgpu::Texture,
        src_x: u32,
        src_y: u32,
        dst_x: u32,
        dst_y: u32,
        w: u32,
        h: u32,
    ) -> Result<(), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        if dst_x + w > entry.width || dst_y + h > entry.height {
            return Err(IndividualTextureError::CopySizeMismatch {
                width: dst_x + w,
                height: dst_y + h,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if w == 0 || h == 0 {
            return Ok(());
        }
        let prof_span = ph2d_gpu::pass_profiler::copy_span_begin(enc, "copy.region");
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: src,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: src_x,
                    y: src_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &entry.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: dst_x,
                    y: dst_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        if let Some(t) = prof_span {
            ph2d_gpu::pass_profiler::copy_span_end(enc, t);
        }
        Ok(())
    }

    /// Increment the refcount for an existing entry. The renderer
    /// uses this when a sprite is duplicated via the M14.6 F context
    /// menu so the source texture survives even if the original
    /// sprite is later deleted.
    pub fn retain(&mut self, id: u32) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.refcount = entry.refcount.saturating_add(1);
        }
    }

    /// Decrement the refcount; drop the entry once it reaches 0.
    /// Returns the post-decrement count, or `None` if the id was
    /// already absent (idempotent for safety).
    pub fn release(&mut self, id: u32) -> Option<u32> {
        let stop = {
            let entry = self.entries.get_mut(&id)?;
            entry.refcount = entry.refcount.saturating_sub(1);
            entry.refcount
        };
        if stop == 0 {
            self.entries.remove(&id);
        }
        Some(stop)
    }

    /// Read access to the pre-built bind group for a texture id.
    /// Returns `None` for ids that were never acquired or have been
    /// fully released — the renderer falls back to "skip this batch"
    /// in either case.
    pub fn bind_group(&self, id: u32) -> Option<&wgpu::BindGroup> {
        self.entries.get(&id).map(|e| &e.bind_group)
    }

    /// Pixel dimensions `(width, height)` of an entry, or `None` for an
    /// unknown id. Used by the extract to convert a sprite's pixel-space
    /// `region_rect` into a UV sub-rect of the (full-unit) texture.
    pub fn dims(&self, id: u32) -> Option<(u32, u32)> {
        self.entries.get(&id).map(|e| (e.width, e.height))
    }
}

// Tests that exercise the GPU paths live alongside `SpriteRenderer`
// in `renderer.rs`'s integration suite (they require a `GpuContext`).
// Pure-Rust tests for refcounting are covered indirectly there as
// well — keeping this module test-light avoids the
// `unsafe { mem::zeroed() }` trick on wgpu handles that wgpu's Drop
// impl would crash on.
