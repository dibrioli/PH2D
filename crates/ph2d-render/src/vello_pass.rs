//! Vello compute-rasterizer pass (M11 widget paint).
//!
//! Renders a [`vello::Scene`] (built upstream by `ph2d-editor` from the
//! current widget tree) into an intermediate `Rgba8Unorm` storage
//! texture, then blits that texture onto the surface frame view via
//! [`wgpu::util::TextureBlitter`]. This compositing pattern is the
//! one Vello docs recommend (`render_to_texture` doc §1) and avoids
//! the GPU vendor pessimization of compute-writes-to-surface.
//!
//! Pass ordering on the desktop shell:
//!     [sprite_pass] writes the surface view (loads clear, draws sprites)
//!     [vello_pass] reads its own intermediate, writes the surface view
//! The blitter overdraws the sprite output where the Vello scene has
//! non-transparent pixels — widgets sit on top of game content.
//!
//! Resize handling: the intermediate texture is sized to the surface
//! and rebuilt only when surface dimensions change. Vello-side state
//! (Renderer + shaders) is build-once.

use ph2d_gpu::GpuContext;
use vello::peniko::Color;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use wgpu::util::{TextureBlitter, TextureBlitterBuilder};

pub struct VelloPass {
    renderer: Renderer,
    /// Intermediate Rgba8Unorm storage texture; Vello compute writes
    /// here. Sized at the surface dimensions. Recreated on resize.
    intermediate: wgpu::Texture,
    intermediate_view: wgpu::TextureView,
    /// Blitter samples `intermediate` and draws into the surface view.
    blitter: TextureBlitter,
    last_size: (u32, u32),
    surface_format: wgpu::TextureFormat,
}

impl VelloPass {
    /// Build the pass. `surface_format` is the format of the wgpu
    /// surface frame view we'll blit into (e.g. Bgra8UnormSrgb on
    /// desktop, Rgba8UnormSrgb on web).
    pub fn new(
        gpu: &GpuContext,
        surface_format: wgpu::TextureFormat,
        initial_size: (u32, u32),
    ) -> Result<Self, String> {
        let renderer = Renderer::new(
            &gpu.device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|e| format!("vello::Renderer::new: {e}"))?;

        let (intermediate, intermediate_view) = create_intermediate(&gpu.device, initial_size);
        // Pre-multiplied alpha blending: the intermediate texture has
        // a transparent background (Color::TRANSPARENT in render()) so
        // sprites underneath stay visible where the editor scene is
        // empty. Default `TextureBlitter::new` uses a no-blend
        // pipeline (overdraw) which paints transparent pixels as
        // black — hiding everything.
        let blitter = TextureBlitterBuilder::new(&gpu.device, surface_format)
            .blend_state(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING)
            .build();

        Ok(Self {
            renderer,
            intermediate,
            intermediate_view,
            blitter,
            last_size: initial_size,
            surface_format,
        })
    }

    /// Recreate the intermediate texture if the surface was resized.
    /// Cheap when no-op (just a tuple compare).
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.last_size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let (tex, view) = create_intermediate(&gpu.device, size);
        self.intermediate = tex;
        self.intermediate_view = view;
        self.last_size = size;
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Intermediate texture view. Bound by the
    /// [`Compositor`](crate::Compositor) as its "UI layer" input.
    pub fn intermediate_view(&self) -> &wgpu::TextureView {
        &self.intermediate_view
    }

    /// Underlying intermediate texture. Exposed so the
    /// [`Compositor`](crate::Compositor) can create extra views —
    /// `wgpu::TextureView` is not `Clone` in wgpu 28.
    pub fn intermediate_texture(&self) -> &wgpu::Texture {
        &self.intermediate
    }

    /// Register a GPU `texture` as a Vello image and return its stable [`ImageData`] handle
    /// (the vector FX, plano 24). Drawing that handle in a `Scene` samples the texture DIRECTLY —
    /// no CPU upload, no re-upload per frame. The texture must be `Rgba8Unorm` + `COPY_SRC`
    /// (Vello copies it into its atlas each render). Keep the texture alive; re-blur into it in
    /// place and Vello sees the fresh pixels behind the SAME id (zero atlas-id churn).
    pub fn register_texture(&mut self, texture: wgpu::Texture) -> vello::peniko::ImageData {
        self.renderer.register_texture(texture)
    }

    /// Unregister a texture previously registered via [`Self::register_texture`] (the filter was
    /// removed / the shape deleted) — frees the atlas slot.
    pub fn unregister_texture(&mut self, image: vello::peniko::ImageData) {
        self.renderer.unregister_texture(image);
    }

    /// M14.5: render the Vello `scene` into the intermediate texture
    /// **without** the final blit onto a surface view. The intermediate
    /// stays available via [`intermediate_view`](Self::intermediate_view)
    /// so a downstream compositor can read it as a texture binding.
    ///
    /// `prefer_msaa` selects between `AaConfig::Msaa16` (more samples
    /// per pixel; smoother glyph edges in unhinted text renderings) and
    /// the default `AaConfig::Area` (analytical coverage; smoother for
    /// thin vector strokes). Shells read the current text-rendering
    /// preset's `prefer_msaa` flag before calling.
    pub fn render_to_intermediate(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        size: (u32, u32),
        bg_color: Color,
        prefer_msaa: bool,
    ) -> Result<(), String> {
        self.ensure_size(gpu, size);
        let params = RenderParams {
            base_color: bg_color,
            width: self.last_size.0,
            height: self.last_size.1,
            // Default: `Area` analytical coverage. MSAA16 produced
            // visible stippling on thin (1-1.5 px) vector strokes at
            // near-axis angles — `Area` integrates coverage analytically
            // per pixel, smoother for thin strokes and cheaper.
            // CrispHeavyPlus opts INTO `Msaa16` because at body sizes
            // with `hint=off`, more samples per pixel smooths the glyph
            // edges noticeably without re-introducing stipple (glyphs
            // aren't 1-1.5 px strokes at near-axis angles — the
            // problem case is vectors). Renderer is initialized with
            // `AaSupport::all()` so both are available.
            antialiasing_method: if prefer_msaa {
                AaConfig::Msaa16
            } else {
                AaConfig::Area
            },
        };
        self.renderer
            .render_to_texture(
                &gpu.device,
                &gpu.queue,
                scene,
                &self.intermediate_view,
                &params,
            )
            .map_err(|e| format!("vello render_to_texture: {e}"))
    }

    /// Synchronously read the sRGB-encoded RGBA byte at `(x, y)`
    /// from the most-recently-rendered intermediate texture.
    /// Returns `None` if the coords are out of bounds or the GPU
    /// readback fails. Used by the eyedropper to sample the rendered
    /// scene at the user's click position.
    ///
    /// Note on color space: although the intermediate texture is
    /// declared `Rgba8Unorm` (linear), Vello writes sRGB-encoded
    /// bytes directly into it (peniko's `Color::from_rgba8` is
    /// sRGB). So the bytes we read back are already in the same
    /// encoding `ColorValue::from_rgba8` expects — no conversion
    /// needed. Doing an extra linear→sRGB pass here would lighten
    /// midtones. See `docs/UI_Bugs/README.md` §4.3.
    ///
    /// Blocks the calling thread until the GPU finishes the copy and
    /// the staging buffer maps — typically a few ms. Eyedropper picks
    /// are rare (one per click in pick mode), so the latency hit is
    /// fine; do NOT call this per-frame.
    pub fn read_pixel(&self, gpu: &GpuContext, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.last_size.0 || y >= self.last_size.1 {
            return None;
        }
        // wgpu requires `bytes_per_row` to be a multiple of 256 for
        // texture-to-buffer copies. We only need 4 bytes (one pixel)
        // but allocate a full 256-byte row.
        const BYTES_PER_ROW: u32 = 256;
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render eyedropper readback"),
            size: BYTES_PER_ROW as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render eyedropper encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.intermediate,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(BYTES_PER_ROW),
                    rows_per_image: Some(1),
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        gpu.queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        rx.recv().ok()?.ok()?;
        let view = slice.get_mapped_range();
        let pixel = [view[0], view[1], view[2], view[3]];
        drop(view);
        buffer.unmap();
        Some(pixel)
    }

    /// Render `scene` into the intermediate at `size` **on a transparent
    /// background** and read the whole region back to a straight-RGBA8 `Vec`
    /// (`size.0 * size.1 * 4` bytes, sRGB-encoded like [`read_pixel`]).
    ///
    /// This is the FX producer's rasterizer (plano 24): the shell renders ONE
    /// isolated vector shape into a scratch [`VelloPass`] sized to the shape's
    /// screen bbox, reads it back, and blurs/tints it on the CPU. It is the
    /// full-region sibling of [`read_pixel`] — same `copy_texture_to_buffer`
    /// dance, same synchronous map, but a whole texture instead of one texel.
    ///
    /// wgpu requires `bytes_per_row` to be a multiple of 256, so the staging
    /// buffer is padded per row and the padding is stripped on the way out —
    /// the returned `Vec` is tightly packed (`width * 4` per row).
    ///
    /// Blocks the calling thread until the copy lands and the buffer maps
    /// (a few ms). Do NOT call this per-frame per shape without a budget — the
    /// producer sizes the scratch to the shape, not the surface, for exactly
    /// this reason.
    pub fn render_and_readback(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        size: (u32, u32),
    ) -> Result<Vec<u8>, String> {
        if size.0 == 0 || size.1 == 0 {
            return Ok(Vec::new());
        }
        self.render_to_intermediate(gpu, scene, size, Color::TRANSPARENT, false)?;
        let (w, h) = self.last_size;
        // 256-byte row alignment for texture→buffer copies.
        let unpadded = w * 4;
        let padded = unpadded.div_ceil(256) * 256;
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render fx readback"),
            size: (padded as u64) * (h as u64),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render fx readback encoder"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.intermediate,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
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
        gpu.queue.submit([encoder.finish()]);

        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("fx readback poll: {e}"))?;
        rx.recv()
            .map_err(|e| format!("fx readback recv: {e}"))?
            .map_err(|e| format!("fx readback map: {e}"))?;
        let view = slice.get_mapped_range();
        // Strip the per-row padding into a tightly-packed straight-RGBA8 buffer.
        let mut out = Vec::with_capacity((unpadded as usize) * (h as usize));
        for row in 0..h as usize {
            let start = row * padded as usize;
            out.extend_from_slice(&view[start..start + unpadded as usize]);
        }
        drop(view);
        buffer.unmap();
        Ok(out)
    }

    /// Render `scene` into the intermediate then blit onto `target`.
    /// `target` must match the surface_format passed at construction.
    /// `bg_color` is the Vello clear before drawing; pass transparent
    /// (`Color::TRANSPARENT`) to overlay onto the existing surface
    /// pixels (sprite content stays visible where the scene is empty).
    pub fn render(
        &mut self,
        gpu: &GpuContext,
        scene: &Scene,
        target: &wgpu::TextureView,
        size: (u32, u32),
        bg_color: Color,
    ) -> Result<(), String> {
        self.ensure_size(gpu, size);
        let params = RenderParams {
            base_color: bg_color,
            width: self.last_size.0,
            height: self.last_size.1,
            // M14.5 round 8: `Area` analytical coverage. MSAA16
            // produced visible stippling on thin (1-1.5 px) strokes
            // at near-axis angles — the 16 fixed sample positions
            // hit-or-miss the stroke in patterns that read as
            // pixelation. `Area` integrates coverage analytically
            // per pixel, smoother for thin strokes and cheaper
            // than MSAA16. The earlier "Area looks low-rez" note
            // pre-dated the gamma-correct compositor + glyph-snap
            // fixes; in the current pipeline Area wins.
            antialiasing_method: AaConfig::Area,
        };
        self.renderer
            .render_to_texture(
                &gpu.device,
                &gpu.queue,
                scene,
                &self.intermediate_view,
                &params,
            )
            .map_err(|e| format!("vello render_to_texture: {e}"))?;

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render vello blit"),
            });
        self.blitter
            .copy(&gpu.device, &mut encoder, &self.intermediate_view, target);
        gpu.queue.submit([encoder.finish()]);
        Ok(())
    }
}

fn create_intermediate(
    device: &wgpu::Device,
    size: (u32, u32),
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render vello intermediate"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        // Vello requires Rgba8Unorm + STORAGE_BINDING; we add
        // TEXTURE_BINDING so the compositor can `textureLoad`
        // individual super-samples, and COPY_SRC so the eyedropper
        // can copy a single pixel back to a CPU-mappable buffer for
        // color readback. No sRGB view sibling — STORAGE_BINDING
        // forbids that combination per wgpu validation.
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
