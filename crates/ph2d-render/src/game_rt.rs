//! `GameRt` — offscreen render target for the game world (sprites,
//! shapes, lights, particles, future material/shader-graph passes).
//!
//! M14.5 introduces the **render-target / viewport pattern** (cf.
//! Bevy `Camera2d.target = RenderTarget::Image`, Godot `SubViewport`,
//! Unity `RenderTexture`, Unreal `SceneColor`): game content writes
//! into this offscreen texture, the editor's Vello chrome writes into
//! its own intermediate, and a [`Compositor`](crate::compositor::Compositor)
//! pass composes both onto the swap chain.
//!
//! Why HDR-ready (`Rgba16Float`):
//! - Linear-light pipeline throughout — no premature gamma clipping
//! - Cores fora de sRGB durante shading (additive lights > 1.0)
//! - AgX tonemap (`tonemap.rs`) controls the LDR transfer at the end
//! - Memory cost ~2× sRGB8 but negligible at editor resolution
//!
//! HR-3 (zero-alloc): the texture is created once and only recreated
//! on resize (`ensure_size`). HR-5 (determinism): no internal RNG or
//! per-frame state — every byte written into this RT is a function of
//! sim inputs.

use ph2d_gpu::GpuContext;

pub struct GameRt {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: (u32, u32),
}

impl GameRt {
    /// Format used by [`GameRt`]. Exposed so that pipelines targeting
    /// this RT can match the color format at construction time.
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(gpu: &GpuContext, size: (u32, u32)) -> Self {
        let (texture, view) = create(&gpu.device, size);
        Self {
            texture,
            view,
            size,
        }
    }

    /// Recreate the texture if dimensions changed. No-op when size
    /// matches the most recent allocation.
    pub fn ensure_size(&mut self, gpu: &GpuContext, size: (u32, u32)) {
        if size == self.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        let (tex, view) = create(&gpu.device, size);
        self.texture = tex;
        self.view = view;
        self.size = size;
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

fn create(device: &wgpu::Device, size: (u32, u32)) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render game RT (Rgba16Float HDR)"),
        size: wgpu::Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: GameRt::FORMAT,
        // RENDER_ATTACHMENT: target of sprite/light/particle passes.
        // TEXTURE_BINDING: sampled by the tonemap + compositor passes.
        // COPY_SRC: enables future capture/screenshot to PNG/EXR.
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_is_rgba16_float() {
        // Pinned: anything that targets `GameRt::FORMAT` (sprite
        // pipeline, light pass, tonemap input sampler) relies on
        // Rgba16Float. A change here without a coordinated audit of
        // every dependent pipeline is a recipe for blank screens at
        // runtime — fail at compile time-ish here instead.
        assert_eq!(GameRt::FORMAT, wgpu::TextureFormat::Rgba16Float);
    }

    /// Try to build a headless GpuContext for tests that need a real
    /// device (texture creation). Returns `None` when no adapter is
    /// available — e.g. Linux CI runners without a GPU or software
    /// rasterizer. Tests use `let Some(gpu) = … else { return; }` so
    /// they pass on adapter-less environments; the assertions run
    /// wherever wgpu can spin up a device (dev machines + Mac CI).
    ///
    /// Cached per test binary: each `request_adapter` + `request_device`
    /// pair costs ~30-50 s on Apple Silicon (cold Metal driver load +
    /// shader compile cache miss). GpuContext is Clone (Arc internals),
    /// so handing out clones is cheap and tests can keep `let Some(gpu)
    /// = …` semantics.
    fn try_headless_gpu() -> Option<GpuContext> {
        use std::sync::OnceLock;
        static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
        SHARED
            .get_or_init(|| {
                let instance = GpuContext::default_instance();
                GpuContext::new(instance, None).ok()
            })
            .clone()
    }

    #[test]
    fn ensure_size_updates_dimensions_on_resize() {
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut rt = GameRt::new(&gpu, (800, 600));
        assert_eq!(rt.size(), (800, 600));
        rt.ensure_size(&gpu, (1024, 768));
        assert_eq!(rt.size(), (1024, 768));
    }

    #[test]
    fn ensure_size_no_op_when_size_unchanged() {
        // No-op path matters: every frame's resize check would
        // otherwise allocate a new texture, blowing HR-3 (zero-alloc
        // hot path) and stalling the GPU on idle reallocations.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut rt = GameRt::new(&gpu, (800, 600));
        rt.ensure_size(&gpu, (800, 600));
        assert_eq!(rt.size(), (800, 600));
    }

    #[test]
    fn ensure_size_rejects_zero_dimension() {
        // wgpu requires width >= 1 and height >= 1 on texture
        // creation. Window-minimize on some platforms reports a
        // (0, 0) inner_size during the transition frame — without
        // this guard `create_texture` panics with a validation error.
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut rt = GameRt::new(&gpu, (800, 600));
        rt.ensure_size(&gpu, (0, 600));
        assert_eq!(rt.size(), (800, 600), "zero width must be rejected");
        rt.ensure_size(&gpu, (800, 0));
        assert_eq!(rt.size(), (800, 600), "zero height must be rejected");
        rt.ensure_size(&gpu, (0, 0));
        assert_eq!(rt.size(), (800, 600), "fully-zero size must be rejected");
    }

    #[test]
    fn ensure_size_handles_repeated_resizes() {
        // Window drag-resize generates a stream of distinct sizes;
        // every one must land in a consistent state. This also
        // exercises the texture's `Drop` (no leak of the previous
        // allocation between resizes).
        let Some(gpu) = try_headless_gpu() else {
            return;
        };
        let mut rt = GameRt::new(&gpu, (100, 100));
        for size in [(200, 200), (400, 300), (200, 200), (1, 1), (1920, 1080)] {
            rt.ensure_size(&gpu, size);
            assert_eq!(rt.size(), size);
        }
    }
}
