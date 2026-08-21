//! **A LEITURA DE VOLTA de uma textura individual** — irmão do [`crate::individual`], que trata de
//! como o *store* as possui, e do [`crate::individual_entry`], que trata de como elas nascem.
//!
//! ⚠️ **Saiu de lá por medição** (2026-08-21): o `individual.rs` voltou a passar o tecto — **1029**
//! linhas contra 969 — depois de a wave dos 16 bits lhe acrescentar o
//! [`IndividualTextureStore::readback_rgba8`]. A regra registada é **cortar, nunca alargar a
//! allowlist**, e o corte volta a ser por responsabilidade: lá fica *quem possui*, no
//! `individual_entry` *como nascem*, e aqui *como se leem de volta*.
//!
//! # A armadilha que este ficheiro guarda
//!
//! ⚠️ **O passo de linha é do FORMATO, nunca de `width × 4`.** Uma `Rgba16Float` tem 8 bytes por
//! pixel, e um `bytes_per_row` calculado a quatro devolve **metade da imagem** — sem erro de
//! validação, sem aviso: o `copy_texture_to_buffer` aceita, e o defeito aparece adiante como um
//! pânico de índice ou uma imagem cortada. É por isso que o cálculo passa por
//! `format.block_copy_size(None)` e não por uma constante.
//!
//! ⚠️ E é por isso que existe o [`IndividualTextureStore::readback_rgba8`]: ele **normaliza** —
//! quem só sabe 8 bits recebe 8 bits, venha a textura de que formato vier.

use ph2d_gpu::GpuContext;

use crate::individual::{IndividualTextureError, IndividualTextureStore};

impl IndividualTextureStore {
    /// Copy the GPU pixel contents of an entry back into a fresh
    /// `Vec<u8>` (RGBA8, tightly packed `width * height * 4`).
    ///
    /// Submits a one-shot copy-texture-to-buffer + map, blocks on
    /// `device.poll(Wait)`, then strips row padding (`copy_texture_to_buffer`
    /// requires rows aligned to
    /// [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`]). Allocates one staging
    /// `wgpu::Buffer` and one output `Vec<u8>` — fine for one-shot
    /// editor actions (Trim / BG Removal); **not** acceptable in any
    /// per-frame path (HR-3).
    ///
    /// HR-1: stays in `ph2d-render`, never crosses to `ph2d-core`.
    /// HR-13: peak transient memory is `~ width * (padded_row + 4)`
    /// — for a 2k² sprite, ≈ 16 MB staging + 16 MB output.
    pub fn readback(
        &self,
        gpu: &GpuContext,
        id: u32,
    ) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        readback_texture(gpu, &entry.texture, 0, entry.width, entry.height)
    }

    /// **Lê uma entrada SEMPRE como RGBA8 sRGB**, convertendo quando ela é de 16 bits.
    ///
    /// ⚠️ **É esta a porta que o shell usa**, e a razão é o pânico de 2026-08-20: as ferramentas de
    /// imagem trabalham em `Vec<u8>` de 4 bytes por pixel, e entregar-lhes o buffer cru de uma
    /// textura `Rgba16Float` dá **o dobro dos bytes** para as mesmas dimensões — o consumidor
    /// seguinte lê metade da imagem e interpreta pares de bytes como cores.
    ///
    /// A [`Self::readback`] crua fica pública para quem de facto queira os texels (os gates de
    /// mip, a paridade). *A porta que o produto atravessa normaliza; a que os testes usam não.*
    pub fn readback_rgba8(
        &self,
        gpu: &GpuContext,
        id: u32,
    ) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
        let is_16 = self.format(id) == Some(Self::FORMAT_16);
        let (w, h, bytes) = self.readback(gpu, id)?;
        if !is_16 {
            return Ok((w, h, bytes));
        }
        let halves: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|p| u16::from_le_bytes([p[0], p[1]]))
            .collect();
        Ok((w, h, ph2d_color::rgba16_to_rgba8(&halves)))
    }

    /// Read back a specific **mip level** of an entry (level 0 = full res).
    /// Dimensions are `(width >> level).max(1) × (height >> level).max(1)`.
    /// Used by the mip-generation tests to assert the downsample is a correct
    /// LINEAR-light box average; same one-shot staging cost as [`Self::readback`]
    /// (not for any per-frame path).
    pub fn readback_mip(
        &self,
        gpu: &GpuContext,
        id: u32,
        level: u32,
    ) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
        let entry = self
            .entries
            .get(&id)
            .ok_or(IndividualTextureError::NotFound(id))?;
        let w = (entry.width >> level).max(1);
        let h = (entry.height >> level).max(1);
        readback_texture(gpu, &entry.texture, level, w, h)
    }

    /// Replace the pixel contents of an existing entry in place.
    /// Used by the M6 hot-reload bridge when an `AssetId` underlying
    /// an individual sprite changes on disk.
    ///
    /// When `width × height` matches the cached dims, the existing
    /// `wgpu::Texture` is reused (queue.write_texture only); the bind
    /// group survives and the `texture_id` stays stable for SimWorld
    /// references.
    ///
    /// When dims change, the texture/view/bind_group are recreated
    /// against the same id. Sprites referencing the id remain valid;
    /// the next render frame samples the new texture.
    pub fn replace_pixels(
        &mut self,
        gpu: &GpuContext,
        material_bgl: &wgpu::BindGroupLayout,
        id: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        // ⚠️ **Irmão do pânico de 2026-08-20, do lado da ESCRITA.** Ver
        // [`IndividualTextureError::EightBitWriteToSixteenBitTexture`]: aqui não há wgpu a
        // reclamar, e o resultado seria metade de cada linha preenchida em silêncio.
        if self.format(id) == Some(Self::FORMAT_16) {
            return Err(IndividualTextureError::EightBitWriteToSixteenBitTexture { id });
        }
        let expected = (width as usize) * (height as usize) * 4;
        if rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: rgba.len(),
                expected,
            });
        }
        let Some(entry) = self.entries.get_mut(&id) else {
            return Ok(());
        };
        if entry.width == width && entry.height == height {
            crate::individual_entry::write_pixels(gpu, &entry.texture, width, height, rgba);
        } else {
            let refcount = entry.refcount;
            let new_entry = crate::individual_entry::create_entry(
                gpu,
                material_bgl,
                &self.sampler,
                width,
                height,
                rgba,
            );
            let mut new_entry = new_entry;
            new_entry.refcount = refcount;
            *entry = new_entry;
        }
        self.regen_mips(gpu, id);
        Ok(())
    }

    /// Upload a sub-rectangle of pixels into an existing entry's texture,
    /// leaving the rest untouched (`queue.write_texture` with a non-zero
    /// origin + partial extent).
    ///
    /// The Painter dirty-rect composite path uploads only the bounding
    /// box a stroke touched instead of the whole canvas — O(bbox) per
    /// frame instead of O(W×H) (Painter W3 audit item 1a — the GPU end of
    /// the dirty-rect path). The texture id, bind group and dims stay
    /// stable, so SimWorld references and the cached bind group remain
    /// valid; no reallocation occurs.
    ///
    /// `region_rgba` must be tightly packed `width * height * 4` bytes for
    /// the sub-rect ALONE (not the full texture). The region must lie
    /// fully within the entry's current dims. A zero-area region is a
    /// no-op (an empty dirty-rect); an unknown id is a silent no-op
    /// (mirror of [`Self::replace_pixels`]).
    // x/y/w/h is the idiomatic graphics sub-rect form (mirrors
    // `queue.write_texture`'s origin + extent); packing into a struct would
    // break the bridge consumer for no clarity gain.
    #[allow(clippy::too_many_arguments)]
    pub fn replace_pixels_region(
        &mut self,
        gpu: &GpuContext,
        id: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        region_rgba: &[u8],
    ) -> Result<(), IndividualTextureError> {
        let Some(entry) = self.entries.get(&id) else {
            return Ok(());
        };
        // The sub-rect must lie fully within the current texture; a write
        // past the edge would corrupt neighbouring rows or panic in wgpu.
        let in_bounds = x.checked_add(width).is_some_and(|r| r <= entry.width)
            && y.checked_add(height).is_some_and(|b| b <= entry.height);
        if !in_bounds {
            return Err(IndividualTextureError::RegionOutOfBounds {
                x,
                y,
                width,
                height,
                tex_width: entry.width,
                tex_height: entry.height,
            });
        }
        if width == 0 || height == 0 {
            return Ok(()); // empty dirty-rect — nothing to upload
        }
        // ⚠️ **Irmão do pânico de 2026-08-20, do lado da ESCRITA.** Ver
        // [`IndividualTextureError::EightBitWriteToSixteenBitTexture`]: aqui não há wgpu a
        // reclamar, e o resultado seria metade de cada linha preenchida em silêncio.
        if self.format(id) == Some(Self::FORMAT_16) {
            return Err(IndividualTextureError::EightBitWriteToSixteenBitTexture { id });
        }
        let expected = (width as usize) * (height as usize) * 4;
        if region_rgba.len() != expected {
            return Err(IndividualTextureError::PixelLengthMismatch {
                got: region_rgba.len(),
                expected,
            });
        }
        crate::individual_entry::write_pixels_region(
            gpu,
            &entry.texture,
            x,
            y,
            width,
            height,
            region_rgba,
        );
        self.regen_mips(gpu, id);
        Ok(())
    }
}

/// Copy a 2D RGBA8 texture out of GPU memory into a tightly-packed
/// `Vec<u8>`. `copy_texture_to_buffer` requires the destination row
/// pitch to be a multiple of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`]
/// (256 on every backend), so the staging buffer is padded and the
/// output is unpadded row-by-row.
fn readback_texture(
    gpu: &GpuContext,
    texture: &wgpu::Texture,
    mip_level: u32,
    width: u32,
    height: u32,
) -> Result<(u32, u32, Vec<u8>), IndividualTextureError> {
    if width == 0 || height == 0 {
        return Ok((width, height, Vec::new()));
    }
    // ⚠️ **DERIVADO do formato, nunca fixo em 4.** Esta linha era `let bytes_per_pixel: u32 = 4;`
    // e foi a causa de um pânico reportado pelo Enio (2026-08-20): *"RGBA16 + Background Removal =
    // Panic · trim = panic · make square = panic · padding = panic · ETC"*.
    //
    // ⚠️ **O mecanismo MEDIDO, e não o que eu supus.** A primeira redação desta nota dizia que o
    // wgpu **abortava** na validação. Não aborta: com `bytes_per_row = 256` (o alinhamento) e uma
    // linha real de `W × 8`, a validação passa — `256 >= 64` — e a cópia **acontece**. O que se
    // parte é o **desempacotamento**, que retira `W × 4` bytes por linha de uma linha que tem
    // `W × 8`: o buffer volta com **metade** da imagem.
    //
    // O pânico é a jusante, quando um consumidor de 8 bits indexa `w · h · 4` num buffer de
    // `w · h · 2`. *Um erro de stride não falha onde está escrito — falha em toda a gente que o
    // consome, e por isso o sintoma foram NOVE ferramentas ao mesmo tempo.*
    //
    // ⚠️ **A auditoria da W2 não o apanhou**, e a razão é instrutiva: ela varreu quem lê os pixels
    // do `Asset` (um `match` na variante, que o compilador e o `grep` mostram) e este sítio lê da
    // **GPU**, onde a suposição de 8 bits não era um `match` — era uma **constante**.
    // *Uma varredura por forma sintática não vê uma premissa escrita como número.*
    let bytes_per_pixel: u32 = texture
        .format()
        .block_copy_size(None)
        .expect("uma textura de cor tem tamanho de bloco");
    let unpadded_bpr = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let buffer_size = (padded_bpr as u64) * (height as u64);

    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ph2d-render individual readback staging"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ph2d-render individual readback encoder"),
        });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);

    // Map + block-on-wait. `device.poll(PollType::Wait)` drives the
    // queue forward until the buffer's map operation completes. We
    // own the channel both ends so the closure's `Send` bound is
    // satisfied without a runtime.
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("poll: {e}")))?;
    rx.recv()
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("channel: {e}")))?
        .map_err(|e| IndividualTextureError::ReadbackFailed(format!("map_async: {e}")))?;

    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded_bpr as usize) * (height as usize));
    for row in 0..height as usize {
        let start = row * padded_bpr as usize;
        let end = start + unpadded_bpr as usize;
        out.extend_from_slice(&mapped[start..end]);
    }
    drop(mapped);
    staging.unmap();
    Ok((width, height, out))
}
