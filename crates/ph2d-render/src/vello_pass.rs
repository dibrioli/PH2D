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
    /// here. Recreated on resize.
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
        // TEXTURE_BINDING so the blitter can sample it back out.
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
