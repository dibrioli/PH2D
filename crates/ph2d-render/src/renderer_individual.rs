//! A **fachada da loja de texturas individuais**: os métodos do `SpriteRenderer` que emprestam o
//! `GpuContext` e o `material_bgl` à [`IndividualTextureStore`] — adquirir, copiar, ler de volta e
//! substituir pixels —, irmão de `renderer.rs` por tecto de LOC.
//!
//! Corte mecânico: cada método saiu inteiro, verbatim, do `impl SpriteRenderer` de lá; nenhum
//! chamador muda de endereço (são métodos).

use super::*;

impl SpriteRenderer {
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
                .as_chunks::<2>()
                .0
                .iter()
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
}
