//! Texture atlas — single GPU texture holding many sprites at native
//! resolution, with a Skyline rect packer choosing each sprite's
//! (x, y) placement at insert time.
//!
//! ## Module layout
//!
//! This module is split by responsibility:
//! - [`region`] — pure data types ([`AtlasRegion`] + UV math,
//!   [`AtlasInsertError`]).
//! - [`gpu_ops`] — raw `wgpu` texture ops (upload / create / clear /
//!   readback).
//! - [`demo`] — golden-angle HSV demo-tile synthesis.
//! - this file — the [`TextureAtlas`] orchestrator (packing, free-list,
//!   regrow, mip regen) plus the atlas constants and the public
//!   re-exports.
//!
//! ## Why dynamic packing
//!
//! M5/M6 shipped a placeholder atlas (a 4×4 grid of 64×64 cells) so
//! the demo could render the HSV dummy tiles. M14.4c reused that
//! grid for image import, which forced every imported PNG into a
//! 64×64 cell — sources lost > 99 % of their pixels and any non-
//! square aspect ratio got stretched to a square. The plan all
//! along (see the comment that lived at the top of this file
//! before M14.4d) was for M6 to swap the grid for real PNG loading;
//! the grid-shaped placeholder leaked into M14.4c and never got
//! removed. This module is the swap.
//!
//! Each insert reserves a region of the atlas large enough for the
//! source's native dimensions. Sprites are looked up by `u32` key
//! (the demo seeds keys 0..15 for its HSV tiles; the importer
//! allocates `next_atlas_key()` keys 16, 17, …). Imported sprites
//! keep their full source resolution up to the atlas size limit
//! ([`ATLAS_DEFAULT_SIZE_PX`]).
//!
//! ## Algorithm
//!
//! [`rect_packer`] under the hood (Skyline heuristic). Cheap (O(n)
//! per insert against n existing skyline nodes) and packs well for
//! sprites of similar height, which is the common case for pixel-
//! art and small UI glyphs. We don't bother with rotation — Skyline
//! handles axis-aligned packs and rotated atlases complicate the
//! UV math downstream for negligible packing gain.
//!
//! ## Growth
//!
//! The atlas is born at [`ATLAS_DEFAULT_SIZE_PX`] and doubles IN PLACE up to
//! [`ATLAS_MAX_SIZE_PX`] ([`TextureAtlas::insert_crescendo`], `cresce.rs`): every
//! region keeps its pixel rect, only the normalized UV changes (readers resolve it
//! per frame). The white tile sits at the origin from birth, so its
//! [`WHITE_TILE_UV`] never changes.
//!
//! ## Mipmaps (2026-06-18 minification fix, Phase 2)
//!
//! The atlas texture carries a full mip chain ([`crate::mipgen`]) so a
//! minified (zoomed-out) sprite samples trilinearly instead of
//! undersampling its antialiased edges into jaggies — the same fix the
//! individual-texture store got in Phase 1, now extended to the SHARED
//! atlas where committed paint strokes and imported images live.
//!
//! Because the atlas packs many regions into ONE texture, a naïve
//! whole-texture downsample would bleed neighbouring regions (and the
//! never-written garbage between them) into each region's edge mips.
//! Two guards bound that bleed to sub-pixel: (1) [`AtlasRegion::uv`]'s
//! half-texel inset keeps level-0 sampling strictly inside each region,
//! and (2) the texture is CLEARED to transparent at creation so edge
//! bleed at higher mips fades toward transparent (correct for a
//! premultiplied sprite edge) instead of pulling in stale garbage. Full
//! gutter padding (needed only for tightly-packed tile-atlases zoomed
//! out past mip ~4) is a documented follow-up; the painter case (large
//! canvas + a few imports with soft/transparent borders) is the benign
//! one.

mod cresce;
mod demo;
mod fabrica;
mod gpu_ops;
mod hero_sheet;
mod region;
mod renasce;

pub use fabrica::{
    BACKGROUND_TEXELS_PER_M, CLOUDS_TILE, CLOUDS_TILE_KEY, COIN_FRAME_MS, COIN_SHEET,
    COIN_SHEET_KEY, DIRT_TILE, DIRT_TILE_KEY, FACTORY_ART, FLAG_FRAME_MS, FLAG_SHEET,
    FLAG_SHEET_KEY, FactoryArt, GROUND_TILE, GROUND_TILE_KEY, HERO_ART, HILLS_TILE, HILLS_TILE_KEY,
    PLATFORM_TILE, PLATFORM_TILE_KEY, factory_image, factory_pixels,
};

/// A imagem `size` repetida `copies` (`[colunas, linhas]`) vezes, lado a lado — o que
/// [`TextureAtlas::insert_wrapped`] guarda à volta de uma imagem que repete.
#[must_use]
pub fn wrap_image(
    texels: &[ph2d_color::SrgbRgba],
    size: [u32; 2],
    copies: [u32; 2],
) -> Vec<ph2d_color::SrgbRgba> {
    let [w, h] = size.map(|v| v as usize);
    let [cx, cy] = copies.map(|c| c.max(1) as usize);
    let linha = w;
    let mut out = Vec::with_capacity(linha * cx * h * cy);
    for y in 0..h * cy {
        let fonte = &texels[(y % h) * linha..(y % h + 1) * linha];
        for _ in 0..cx {
            out.extend_from_slice(fonte);
        }
    }
    out
}
pub use hero_sheet::{
    HERO_SHEET_CELL_PX, HERO_SHEET_COLS, HERO_SHEET_KEY, HERO_SHEET_ROWS,
    HERO_SHEET_SOLE_BELOW_CENTRE, HERO_SHEET_TAGS, HeroTag, hero_sheet_pixels,
};
pub use region::{AtlasInsertError, AtlasRegion};

use demo::make_hsv_tile;
use gpu_ops::readback_atlas_texture;
use ph2d_gpu::GpuContext;
// BTreeMap (not HashMap): HR-5 / ADR-0022 forbid unordered maps in
// any path that contributes to a deterministic snapshot. Atlas
// lookups aren't snapshot-relevant (per-frame UV resolution only),
// but the project's lint blocks `std::collections::HashMap`
// project-wide. BTreeMap is fine here — region count stays small
// (≤ a few hundred sprites in practice) so the log-N cost vs a hash
// map is dwarfed by the GPU work that follows the lookup.
use std::collections::BTreeMap;

/// Side an atlas is BORN with (square, RGBA8 + mips ≈ 21 MiB). It grows in place up to
/// [`ATLAS_MAX_SIZE_PX`] when content arrives. Pre-allocating the maximum cost 341 MiB of the
/// card for 16 demo tiles (F36 §2 table; F37 §1 measured the drop).
pub const ATLAS_DEFAULT_SIZE_PX: u32 = 2048;

/// Largest side the atlas grows to — the wgpu Default tier's `max_texture_dimension_2d`
/// on every backend we target (8192² RGBA8 + mips ≈ 341 MiB).
pub const ATLAS_MAX_SIZE_PX: u32 = 8192;

/// Pixel side length of each tile in the demo HSV atlas. Used by
/// [`TextureAtlas::dummy`] only; arbitrary-sized sprites can live
/// in the atlas alongside the demo tiles via [`TextureAtlas::insert`].
pub const DEMO_TILE_PX: u32 = 64;

/// Number of HSV demo tiles seeded by [`TextureAtlas::dummy`]. Keys
/// `0..DEMO_TILE_COUNT` are reserved for demo content; user imports
/// should start their key allocator at [`FIRST_IMPORT_KEY`].
pub const DEMO_TILE_COUNT: u32 = 16;

/// First key the importer is free to allocate for user sprites.
/// Sits past the demo-tile range so the demo's HSV grid stays
/// intact while imports are spawned and despawned.
pub const FIRST_IMPORT_KEY: u32 = 16;

/// Reserved key of the opaque **white** tile ([`TextureAtlas::insert_white_tile`]).
/// A sentinel at the top of the key space, so it can never collide with the
/// importer's allocator (which counts up from [`FIRST_IMPORT_KEY`]) and needs
/// no shared numbering to be bumped.
///
/// A quad sampling this tile shows its `tint` **exactly** — the shader
/// multiplies tint by the texel. Instances that carry no `uv_rect` of their own
/// (today: every Motion node stream, which has no framing node yet) must sample
/// it, or the artist's authored colour comes out multiplied by whatever image
/// happened to sit at the fallback tile.
pub const WHITE_TILE_KEY: u32 = u32::MAX;

/// UV of the white tile that holds at EVERY atlas size: the tile is born at the
/// origin, and `[0, 0, k, k]` with `k = ½ tile / max side` stays inside its 64 px for
/// any side ≤ [`ATLAS_MAX_SIZE_PX`] (the clamp-to-edge sampler keeps the `0` edge white).
/// Callers that cache it (Motion, particles, health bars, the macro track) stay right
/// when the atlas grows.
pub const WHITE_TILE_UV: [f32; 4] = {
    let k = (DEMO_TILE_PX / 2) as f32 / ATLAS_MAX_SIZE_PX as f32;
    [0.0, 0.0, k, k]
};

// Compile-time guard on the sentinel: it must sit past every key the demo seeds
// and every key the importer can allocate, or the white tile would be silently
// overwritten by a user sprite.
const _: () = assert!(WHITE_TILE_KEY > FIRST_IMPORT_KEY && WHITE_TILE_KEY > DEMO_TILE_COUNT);

/// Reserved key of the white **disc** tile: a quad of size `[d, d]` sampling it
/// draws a disc of diameter `d` in its `tint` — the drawing of a
/// `ColliderShape::Ball` of radius `d / 2` (plano 30 §25, A: the drawing is the body).
pub const DISC_TILE_KEY: u32 = u32::MAX - 1;

/// Side of the disc tile: the edge is anti-aliased over one texel, and the smoke
/// scenes draw a `0,5–0,9 m` disc at `~60–110 px` (plano 30 §25), so `256` keeps the
/// edge sharp under a `2×` zoom.
pub const DISC_TILE_PX: u32 = 256;

const _: () = assert!(DISC_TILE_KEY > FIRST_IMPORT_KEY && DISC_TILE_KEY != WHITE_TILE_KEY);

/// The disc tile's straight-alpha texels, row by row: white, with coverage `1` inside a disc
/// whose edge sits half a texel inside the tile (the sampler never reaches past
/// it) and a one-texel linear ramp across the edge.
#[must_use]
pub fn disc_tile_pixels(side: u32) -> Vec<ph2d_color::SrgbRgba> {
    let c = side as f32 / 2.0;
    let r = c - 0.5;
    let mut px = Vec::with_capacity((side * side) as usize);
    for y in 0..side {
        for x in 0..side {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let a = (r - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0);
            px.push(ph2d_color::SrgbRgba::new(
                0xff,
                0xff,
                0xff,
                (a * 255.0).round() as u8,
            ));
        }
    }
    px
}

pub struct TextureAtlas {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size_px: u32,
    packer: rect_packer::DensePacker,
    /// O lugar EMPACOTADO de cada chave (o que o empacotador e a *free list* contam).
    regions: BTreeMap<u32, AtlasRegion>,
    /// ⭐ As chaves guardadas ENVOLVIDAS pelas cópias de si mesmas ([`Self::insert_wrapped`]): as cópias
    /// `[colunas, linhas]` e a região da imagem (a célula do meio) — o que [`Self::region`] responde. É o
    /// *Extrude Edges* da Unity (plano GEF 13 D1): o mip e o filtro de uma imagem que repete leem a própria
    /// imagem à volta dela, e o objecto não sabe que as cópias existem.
    wraps: BTreeMap<u32, ([u32; 2], AtlasRegion)>,
    /// Atlas V2 (M14.4f): free-list of regions made available by
    /// [`Self::remove`] (and by replace-with-different-size, which
    /// would otherwise leak the slot). Indexed by `(width, height)`
    /// so an `insert` of matching dimensions can reuse a slot without
    /// touching the Skyline packer. BTreeMap (not HashMap) per HR-5.
    free_slots: BTreeMap<(u32, u32), Vec<AtlasRegion>>,
    /// Hard cap for [`Self::cresce`]. Read from
    /// `gpu.device.limits().max_texture_dimension_2d` at construction
    /// so weaker adapters (mobile/WebGL) don't try to allocate
    /// textures their hardware refuses. Capped at 8192 even when the
    /// adapter exposes 16384 — packer + bind-group cost grows
    /// quadratically with side length.
    max_size_px: u32,
    /// Regenerates the atlas mip chain after each content write so a
    /// minified (zoomed-out) sprite samples trilinearly instead of
    /// aliasing its antialiased edges (2026-06-18 Phase 2). The atlas is
    /// `Rgba8UnormSrgb`, so one generator serves the whole texture; a
    /// full regen costs ~one fullscreen blit per level (≈0.5 ms at
    /// 8192²) and only fires on insert / replace / regrow — never per
    /// frame (the painter live preview uses individual textures).
    mip_gen: crate::mipgen::MipGenerator,
    /// ⭐⭐ **QUANTAS VEZES OS PIXELS DESTE ATLAS MUDARAM** — o relógio de invalidação para quem
    /// mantém uma cópia em CPU dele.
    ///
    /// ⛔⛔ Ele existe por causa do achado §2.5 da auditoria do `source.lsystem`: a leitura de
    /// pixels da folha lia o atlas **INTEIRO** (`8192² × 4 = 268 MB`) e guardava-o **pela vida do
    /// processo**, sem nunca o invalidar — *pintar na folha servia pixels velhos para sempre*.
    /// Um cache de bytes de GPU não pode ser invalidado por tempo nem por tamanho: ele tem de ser
    /// invalidado por **mudança**, e só o dono da textura sabe quando ela muda.
    ///
    /// ⚠️ Incrementado no [`Self::regen_mips`], que é o funil por onde **toda** escrita de
    /// conteúdo passa (o `insert`, a substituição dentro dele e o `cresce`) — pendurá-lo
    /// em cada mutador seria a lista que envelhece no seguinte.
    ///
    /// ⛔ **O [`Self::remove`] NÃO o move, e está certo**: ele devolve o slot à *free list* e
    /// **não toca num pixel**. Uma cópia em CPU tirada antes dele continua exacta; o que a
    /// invalida é o `insert` que vier ocupar aquele slot, e esse passa pelo funil. *Um relógio de
    /// invalidação conta MUDANÇAS DE CONTEÚDO, não mudanças de contabilidade.*
    epoch: u64,
    /// `Some` enquanto a textura é uma que a placa RECUSOU ao nascer (F36): os bytes de cada região
    /// viva, para [`Self::refaz_se_recusada`] a refazer no fim do episódio sem memória.
    fonte: Option<BTreeMap<u32, Vec<u8>>>,
}

impl TextureAtlas {
    /// Build an empty atlas of `size_px × size_px` RGBA8 (sRGB).
    /// Texture is allocated up-front; bytes start as garbage and
    /// only the regions touched by [`Self::insert`] /
    /// [`Self::update_region`] are deterministic.
    pub fn new(gpu: &GpuContext, size_px: u32) -> Self {
        Self::with_filter(gpu, size_px, crate::ImageFilterMode::default())
    }

    /// Build an empty atlas using an explicit [`ImageFilterMode`].
    /// [`Self::new`] defers to this with the project default. The
    /// sampler is the SINGLE canonical sprite sampler
    /// ([`crate::create_sprite_sampler`]) — atlas and individual
    /// textures share the exact same descriptor so they never diverge
    /// (the old hardcoded `Linear` vs `Nearest` bug).
    ///
    /// [`ImageFilterMode`]: crate::ImageFilterMode
    pub fn with_filter(gpu: &GpuContext, size_px: u32, filter: crate::ImageFilterMode) -> Self {
        let adapter_cap = gpu.device.limits().max_texture_dimension_2d;
        let max_size_px = adapter_cap.min(ATLAS_MAX_SIZE_PX);
        let size_px = size_px.max(1).min(max_size_px);
        // The clear (inside `nasce_textura`) keeps untouched packing space transparent so the gap
        // between regions downsamples cleanly (see module doc §Mipmaps).
        let (texture, view, recusada) = renasce::nasce_textura(gpu, size_px);
        let sampler =
            crate::create_sprite_sampler(&gpu.device, filter, "ph2d-render atlas sampler");
        let mut atlas = Self {
            texture,
            view,
            sampler,
            size_px,
            packer: rect_packer::DensePacker::new(size_px as i32, size_px as i32),
            regions: BTreeMap::new(),
            wraps: BTreeMap::new(),
            free_slots: BTreeMap::new(),
            max_size_px,
            mip_gen: crate::mipgen::MipGenerator::new(gpu, wgpu::TextureFormat::Rgba8UnormSrgb),
            epoch: 0,
            fonte: recusada.then(BTreeMap::new),
        };
        // The white tile takes the origin before anything else ([`WHITE_TILE_UV`]); its
        // insert also fills the (transparent) mip chain. An atlas too small for it only
        // gets the mip chain.
        if atlas.insert_white_tile(gpu).is_err() {
            atlas.regen_mips(gpu);
        }
        atlas
    }

    /// Regenerate the atlas mip chain (levels `1..`) from level 0 after a
    /// content write. Cheap enough to run on every insert/replace; never
    /// called per frame.
    fn regen_mips(&mut self, gpu: &GpuContext) {
        // ⚠️ **O contador vive AQUI**, e não em cada mutador — ver [`Self::epoch`].
        self.epoch = self.epoch.wrapping_add(1);
        self.mip_gen.run(
            gpu,
            &self.texture,
            crate::mipgen::mip_levels(self.size_px, self.size_px),
        );
    }

    /// **O relógio de mudança dos pixels** — ver [`Self::epoch`]. Quem guarda uma cópia em CPU
    /// deste atlas compara-o e deita a cópia fora quando ele anda.
    #[must_use]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Maximum side length [`Self::cresce`] will allow.
    /// Caller-visible so the shell can pre-emptively refuse imports
    /// larger than this without waiting for the atlas to fail.
    pub fn max_size_px(&self) -> u32 {
        self.max_size_px
    }

    /// Recreate the atlas sampler for a new [`ImageFilterMode`]. The
    /// texture and packed regions are untouched — only how they're
    /// SAMPLED changes — but any bind group referencing the old
    /// `sampler` must be rebuilt afterward (the renderer does this via
    /// `rebuild_material_bind_group`). Cheap: one sampler allocation.
    ///
    /// [`ImageFilterMode`]: crate::ImageFilterMode
    pub fn set_filter_mode(&mut self, gpu: &GpuContext, filter: crate::ImageFilterMode) {
        self.sampler =
            crate::create_sprite_sampler(&gpu.device, filter, "ph2d-render atlas sampler");
    }

    /// Build the demo atlas — empty atlas + 16 HSV-tinted 64×64
    /// tiles inserted at keys `0..16`. Used by the 1000-sprite
    /// demo and by tests that need a populated atlas.
    pub fn dummy(gpu: &GpuContext) -> Self {
        let mut atlas = Self::new(gpu, ATLAS_DEFAULT_SIZE_PX);
        for i in 0..DEMO_TILE_COUNT {
            let pixels = make_hsv_tile(i, DEMO_TILE_PX);
            atlas
                .insert(gpu, i, DEMO_TILE_PX, DEMO_TILE_PX, &pixels)
                .expect("demo HSV tiles always fit into a fresh 4096² atlas");
        }
        atlas
    }

    /// Insert (or refresh) the reserved opaque **white** tile at
    /// [`WHITE_TILE_KEY`], returning its UV rect for instances that carry no
    /// `uv_rect` of their own. A quad sampling it renders its `tint` exactly.
    ///
    /// Sized like a demo tile rather than 1×1: the sampler filters at region
    /// borders, so a one-texel region would blend with whatever the packer put
    /// next to it. Idempotent (a second call is `insert`'s replace path). ⚠️ Not public: a renderer
    /// takes the reserved tiles TOGETHER ([`Self::insert_reserved_tiles`]), or a disc sprite would
    /// sample whatever sat at its key.
    fn insert_white_tile(&mut self, gpu: &GpuContext) -> Result<[f32; 4], AtlasInsertError> {
        let px = vec![0xff; (DEMO_TILE_PX * DEMO_TILE_PX * 4) as usize];
        self.insert(gpu, WHITE_TILE_KEY, DEMO_TILE_PX, DEMO_TILE_PX, &px)
            .map(|r| {
                if (r.x, r.y) == (0, 0) {
                    WHITE_TILE_UV
                } else {
                    r.uv(self.size_px)
                }
            })
    }

    /// The reserved tiles a renderer needs before any sprite draws: the
    /// [`WHITE_TILE_KEY`] square, the [`DISC_TILE_KEY`] disc and the [`FACTORY_ART`] (the
    /// [`HERO_SHEET_KEY`] sheet and the platformer's tiles). Returns the white tile's UV rect (see
    /// [`Self::insert_white_tile`]).
    pub fn insert_reserved_tiles(
        &mut self,
        gpu: &GpuContext,
    ) -> Result<[f32; 4], AtlasInsertError> {
        let px = disc_tile_pixels(DISC_TILE_PX);
        let bytes: &[u8] = bytemuck::cast_slice(&px);
        self.insert_crescendo(gpu, DISC_TILE_KEY, DISC_TILE_PX, DISC_TILE_PX, bytes)?;
        for art in &FACTORY_ART {
            let [w, h] = art.image_px();
            if art.tile {
                self.insert_wrapped(gpu, art.key, w, h, factory_image(art), art.grid)?;
            } else {
                self.insert_crescendo(gpu, art.key, w, h, factory_image(art))?;
            }
        }
        self.insert_white_tile(gpu)
    }

    /// ⭐ Guarda a imagem `w × h` de `key` envolvida pelas cópias de si mesma (`copies` = `[colunas, linhas]`,
    /// ímpares) e responde por ela a região da cópia do MEIO. Ver [`Self::wraps`].
    pub fn insert_wrapped(
        &mut self,
        gpu: &GpuContext,
        key: u32,
        w: u32,
        h: u32,
        rgba: &[u8],
        copies: [u32; 2],
    ) -> Result<AtlasRegion, AtlasInsertError> {
        let [cx, cy] = copies.map(|c| c.max(1));
        let tira = wrap_image(bytemuck::cast_slice(rgba), [w, h], [cx, cy]);
        let fora = self.insert_crescendo(gpu, key, w * cx, h * cy, bytemuck::cast_slice(&tira))?;
        let dentro = AtlasRegion {
            x: fora.x + w * (cx / 2),
            y: fora.y + h * (cy / 2),
            w,
            h,
        };
        self.wraps.insert(key, ([cx, cy], dentro));
        Ok(dentro)
    }

    /// Pack an `(width × height)` source at native resolution into
    /// the atlas under `key` and upload the bytes. Returns the
    /// resulting region's pixel rect on success.
    ///
    /// `rgba` must be tightly-packed `width * height * 4` bytes
    /// (no row padding — wgpu requires `bytes_per_row` aligned to
    /// 256, which we handle internally if needed).
    ///
    /// If `key` was already inserted this is treated as a *replace*
    /// — the existing region is reused (no re-pack) and only the
    /// pixels are rewritten. If the replace dimensions DIFFER from
    /// the original, M14.4f releases the old slot into `free_slots`
    /// and falls through to a fresh pack attempt — the slot will be
    /// reclaimed by the next matching-size insert.
    pub fn insert(
        &mut self,
        gpu: &GpuContext,
        key: u32,
        width: u32,
        height: u32,
        rgba: &[u8],
    ) -> Result<AtlasRegion, AtlasInsertError> {
        if width == 0 || height == 0 {
            return Err(AtlasInsertError::AtlasFull { width, height });
        }
        if width > self.size_px || height > self.size_px {
            return Err(AtlasInsertError::SourceTooLarge {
                width,
                height,
                atlas: self.size_px,
            });
        }
        assert_eq!(
            rgba.len() as u32,
            width * height * 4,
            "insert: rgba buffer must be width*height*4 = {} bytes (got {})",
            width * height * 4,
            rgba.len()
        );
        // Uma escrita simples é a imagem inteira: deixa de haver cópias à volta.
        self.wraps.remove(&key);

        // Replace path: same key + same size → rewrite pixels in
        // the existing region. M14.4f: when sizes DIFFER we now
        // release the old slot into `free_slots` (rather than
        // leaking it) and fall through to the fresh-insert flow
        // below, which may consume that same slot if dimensions
        // match a later sprite.
        if let Some(existing) = self.regions.remove(&key) {
            if existing.w == width && existing.h == height {
                self.sobe(gpu, key, existing, rgba);
                self.regions.insert(key, existing);
                self.regen_mips(gpu);
                return Ok(existing);
            }
            self.free_slots
                .entry((existing.w, existing.h))
                .or_default()
                .push(existing);
        }

        // Fast path: a previously-freed slot of identical dimensions
        // can host this sprite without consulting the Skyline packer.
        // This is what makes `despawn → re-import` cycles survive
        // long sessions without saturating the packer.
        if let Some(bucket) = self.free_slots.get_mut(&(width, height))
            && let Some(region) = bucket.pop()
        {
            if bucket.is_empty() {
                self.free_slots.remove(&(width, height));
            }
            self.sobe(gpu, key, region, rgba);
            self.regions.insert(key, region);
            self.regen_mips(gpu);
            return Ok(region);
        }

        // Fresh insert — ask the packer for a free slot. `false`
        // disables rotation (we don't want rotated UVs upstream).
        let rect = self
            .packer
            .pack(width as i32, height as i32, false)
            .ok_or(AtlasInsertError::AtlasFull { width, height })?;
        let region = AtlasRegion {
            x: rect.x as u32,
            y: rect.y as u32,
            w: width,
            h: height,
        };
        self.sobe(gpu, key, region, rgba);
        self.regions.insert(key, region);
        self.regen_mips(gpu);
        Ok(region)
    }

    /// Release `key`'s region back into the free-list. The next
    /// [`Self::insert`] of matching dimensions will reuse the slot
    /// (skipping the Skyline packer). Returns the freed region for
    /// the caller's inspection, or `None` if `key` wasn't inserted.
    ///
    /// **Refcount contract**: each `key` is currently owned by
    /// exactly one sprite (the M14.4d `next_import_cell` allocator
    /// guarantees uniqueness). Calling `remove` while another sprite
    /// still references the key is undefined behavior at the user
    /// level (the still-referenced sprite will sample garbage / a
    /// future unrelated sprite's pixels). Refcount tracking lands
    /// when M14.5 introduces prefab/clone semantics that can share
    /// keys.
    pub fn remove(&mut self, key: u32) -> Option<AtlasRegion> {
        self.wraps.remove(&key);
        let region = self.regions.remove(&key)?;
        if let Some(fonte) = &mut self.fonte {
            fonte.remove(&key);
        }
        self.free_slots
            .entry((region.w, region.h))
            .or_default()
            .push(region);
        Some(region)
    }

    /// Free-list size (for tests + diagnostics). Number of regions
    /// currently available for matching-size reuse.
    pub fn free_slot_count(&self) -> usize {
        self.free_slots.values().map(Vec::len).sum()
    }

    /// Lookup the reserved region for `key` (returned by an earlier
    /// [`Self::insert`]). `None` when the key was never inserted.
    ///
    /// ⚠️ A região da IMAGEM: numa chave envolvida, a cópia do meio ([`Self::insert_wrapped`]).
    pub fn region(&self, key: u32) -> Option<AtlasRegion> {
        match self.wraps.get(&key) {
            Some((_, dentro)) => Some(*dentro),
            None => self.regions.get(&key).copied(),
        }
    }

    /// Lookup the UV rectangle for `key` (sugar over
    /// `region(key).map(|r| r.uv(size_px))`). Returns a
    /// zero-sized UV at the origin when `key` is missing, so the
    /// extract path can emit a draw without an early-return branch
    /// — the sprite renders as a 1-texel sample that is visually
    /// inert and easy to spot in screenshots if it ever happens.
    pub fn region_uv(&self, key: u32) -> [f32; 4] {
        match self.region(key) {
            Some(r) => r.uv(self.size_px),
            None => [0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Source pixel dimensions `(w, h)` of `key`'s packed region — i.e.
    /// the original image size (the atlas packs at native resolution, no
    /// scaling). `None` for an unknown key. Used by the extract to convert
    /// a sprite's pixel-space `region_rect` into an atlas-UV sub-rect.
    pub fn region_px(&self, key: u32) -> Option<(u32, u32)> {
        self.region(key).map(|r| (r.w, r.h))
    }

    /// Number of reserved regions. Used by tests; not perf-critical.
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Read a whole **mip level** of the atlas texture back into a tightly
    /// packed `Vec<u8>` (RGBA8). Level 0 is full res; level `n` is
    /// `(size_px >> n).max(1)` square. One-shot staging copy (same cost
    /// model as the individual store's readback) — for tests and
    /// diagnostics only, never a per-frame path. Returns `(w, h, bytes)`.
    pub fn readback_mip(&self, gpu: &GpuContext, level: u32) -> (u32, u32, Vec<u8>) {
        let side = (self.size_px >> level).max(1);
        let bytes = readback_atlas_texture(gpu, &self.texture, level, side, side);
        (side, side, bytes)
    }
}

#[cfg(test)]
mod tests;
