//! **Impasto light pass** — the relief made visible, on the GPU.
//!
//! # Why it exists
//!
//! The Painter's light pass was CPU-only, and that had a cost far past the pass itself: the GPU preview's
//! eligibility gate bailed the moment `impasto_visible()`, so **a document with any relief on it
//! composited its entire layer stack on the CPU** — every blend mode, every adjustment, every layer, every
//! frame. The shading was the reason, and the shading is the cheap part.
//!
//! This is the sibling of [`crate::preview_premul`] and sits in the same place in the chain: the
//! [`crate::layer_compositor::LayerCompositor`] writes a straight-sRGB8 composite, this lights it, and the
//! premultiply blit takes it from there.
//!
//! # What crosses the seam
//!
//! Only the OPTICS port. The composed relief — which layers, in which z-order, folded by which composite
//! mode, with the live stroke merged and the glass ceiling applied — is materialised once on the CPU
//! (`ph2d_tool_painter::tool::paint::impasto_gpu`) and arrives here as three finished planes. The shader
//! re-implements a normal, four lamps and a BRDF, and re-implements none of the fold.
//!
//! That is the whole risk budget: a shader that re-derived the fold would be a second answer to *"how do
//! layers of paint stack"*, and two answers to one question drift — here, in the one place where nobody
//! can read a number back out. A bounded port can be pinned against the function it ports; a second fold
//! could only be pinned against a screenshot.
//!
//! # Parity
//!
//! Runtime output is **not** bit-identical across backends — the same caveat the layer compositor carries
//! (`layer_compositor::mod` §"Parity"): `sqrt` is correctly rounded but a backend is free to contract
//! `a * b + c` into an FMA, which is *more* accurate and therefore different. So the policy here is the
//! project's established one:
//!
//! - the shader's **literals** are pinned bit-identical to the Rust constants by a CPU-only gate
//!   (`impasto_light_shader_constants_match_the_cpu_pass`) — no device required, so it runs everywhere;
//! - the two **structural** contracts are pinned EXACTLY, because they are early-outs and exactly
//!   reproducible: flat paint comes back byte-identical, and bare paper is untouched;
//! - **runtime** agreement with `apply_impasto_light` is pinned within a documented byte epsilon by an
//!   `#[ignore]`d GPU gate, reconciled against the canonical CPU pass itself rather than a
//!   re-implementation of it.
//!
//! The store is quantised explicitly in the shader so the one *avoidable* divergence — Rust rounding half
//! away from zero, WGSL free to round half to even — does not cost a byte on every .5 boundary.

use ph2d_gpu::GpuContext;

pub(crate) const IMPASTO_LIGHT_WGSL: &str = include_str!("shaders/impasto_light.wgsl");

/// Workgroup edge (mirrors `@workgroup_size(8, 8, 1)` in the shader).
const WORKGROUP_EDGE: u32 = 8;

/// Lamps the shader's uniform holds. Mirrors `impasto_rig::MAX_LIGHTS`; pinned by
/// `impasto_light_shader_constants_match_the_cpu_pass`.
pub const IMPASTO_MAX_LIGHTS: usize = 4;

/// One resolved lamp, as the pass consumes it: direction, half-vector, and `intensity × colour` already
/// multiplied together.
///
/// Resolved by the caller, never here. The direction is built from the artist's whole-degree azimuth and
/// elevation through the Painter's shared 1°-step rotor — the same table the brush's Jitter Rotate turns
/// by — and a second rotor computed on this side would disagree in the last bits with the one the rest of
/// the app uses.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ImpastoLamp {
    pub dir: [f32; 3],
    pub half: [f32; 3],
    pub tint: [f32; 3],
}

/// Everything one light dispatch needs. Borrowed — the caller owns the planes, which it rebuilds from the
/// layer store.
pub struct ImpastoLightInput<'a> {
    /// Canvas dimensions. The planes are canvas-sized so the shader's clamp IS the canvas clamp: the
    /// normal is a central difference, and a region-sized plane would clamp at the region's edge instead,
    /// drawing a seam along a rectangle nobody could explain.
    pub width: u32,
    pub height: u32,
    /// The region to light, in canvas coords. Texels outside keep whatever the destination already held —
    /// the same persistent-output contract the compositor's region dispatch runs on, which means the
    /// caller owes this pass a full-canvas dispatch before its first partial one.
    pub region: crate::layer_compositor::Region,
    /// **Where the plane buffers below go**, in canvas coords — the window the CPU folded this frame.
    ///
    /// The plane textures are canvas-sized and PERSISTENT, so a window leaves every texel outside it
    /// holding what the last upload put there. That is what lets the fold shrink from the canvas to the
    /// dirty rect (measured: 202 ms → 2,8 ms at 4096²) while the shader's central difference still clamps
    /// to the CANVAS — the reason the planes were canvas-sized in the first place, and a property of the
    /// texture, not of the upload.
    ///
    /// A window is only sound when nothing outside it changed. The pass cannot know that and does not
    /// guess: it knows only whether it has ever had a FULL window for this canvas ([`Self::region`]'s
    /// sibling question, answered by [`ImpastoLightPass::planes_seeded`]), and refuses a partial upload
    /// before that with [`ImpastoLightError::PlanesNotSeeded`].
    pub plane_region: crate::layer_compositor::Region,
    /// Composed height, post-ceiling: `plane_region.w × plane_region.h` floats.
    pub relief: &'a [f32],
    /// Composed coverage: `plane_region.w × plane_region.h` bytes.
    pub cover: &'a [u8],
    /// `[shine, roughness, metallic, wax]` per texel of `plane_region`.
    pub mat0: &'a [u8],
    /// `[wax_r, wax_g, wax_b, _]` per texel of `plane_region`.
    pub mat1: &'a [u8],
    /// The lit lamps (1..=[`IMPASTO_MAX_LIGHTS`]). Empty is a caller bug — the CPU seam returns no planes
    /// at all when every lamp is off, which is how an unlit canvas stays byte-identical instead of being
    /// divided by a zero flat response.
    pub lamps: &'a [ImpastoLamp],
    /// The specular table, row-major `rough_levels × lut_width`. Uploaded as EXACT floats, which is what
    /// keeps `pow` off the GPU entirely: the CPU bakes this once per process and the shader only indexes
    /// it, so the one transcendental in the model cannot diverge between the two paths at all.
    pub spec_lut: &'a [f32],
    pub lut_width: u32,
    pub rough_levels: u32,
}

impl ImpastoLightInput<'_> {
    /// Refuse a mis-shaped request BEFORE a single GPU resource is touched.
    ///
    /// A method on the input rather than a check inside `run`, so the gates exercise the very predicate
    /// the pass applies. A test carrying its own copy of these rules would be a second answer to *"is this
    /// request well formed"*, and the copy would go on passing after the real one changed.
    ///
    /// # Errors
    ///
    /// [`ImpastoLightError`] naming the first thing that does not fit.
    pub fn check(&self) -> Result<(), ImpastoLightError> {
        // The planes are sized by the WINDOW now, not by the canvas. Checking them against the canvas
        // would pass a full upload and reject every partial one — and checking nothing would let a short
        // buffer reach `write_texture`, where the failure is a driver error instead of a named refusal.
        let n = (self.plane_region.w as usize) * (self.plane_region.h as usize);
        if self.width == 0 || self.height == 0 || self.region.w == 0 || self.region.h == 0 {
            return Err(ImpastoLightError::EmptyExtent);
        }
        if self.plane_region.w == 0
            || self.plane_region.h == 0
            || self.plane_region.x + self.plane_region.w > self.width
            || self.plane_region.y + self.plane_region.h > self.height
        {
            return Err(ImpastoLightError::PlaneSize);
        }
        if self.relief.len() != n
            || self.cover.len() != n
            || self.mat0.len() != n * 4
            || self.mat1.len() != n * 4
        {
            return Err(ImpastoLightError::PlaneSize);
        }
        if self.lamps.is_empty() || self.lamps.len() > IMPASTO_MAX_LIGHTS {
            return Err(ImpastoLightError::LampCount);
        }
        if self.lut_width == 0
            || self.rough_levels == 0
            || self.spec_lut.len() != (self.lut_width as usize) * (self.rough_levels as usize)
        {
            return Err(ImpastoLightError::LutSize);
        }
        Ok(())
    }
}

/// Why a dispatch was refused. Every arm is a shape mismatch the caller can fix; none is a device error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpastoLightError {
    /// A zero-area canvas or region.
    EmptyExtent,
    /// A plane is not `plane_region.w × plane_region.h` (times its texel size), or the window does not
    /// fit the canvas.
    PlaneSize,
    /// A PARTIAL plane upload arrived before this canvas ever had a full one.
    ///
    /// The texels outside the window would be whatever the texture was born with — zeros — so the pass
    /// would light most of the painting as if it were flat. Refused rather than drawn, and the caller's
    /// fix is to ask [`ImpastoLightPass::planes_seeded`] first and fold the whole canvas when it says no.
    PlanesNotSeeded,
    /// No lamp, or more than [`IMPASTO_MAX_LIGHTS`].
    LampCount,
    /// The specular table is not `rough_levels × lut_width`.
    LutSize,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LampRaw {
    dir: [f32; 4],
    hlf: [f32; 4],
    tint: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    lamps: [LampRaw; IMPASTO_MAX_LIGHTS],
    n: u32,
    ox: u32,
    oy: u32,
    rw: u32,
    rh: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
}

/// A canvas-sized plane the shader reads.
struct Plane {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

/// Every canvas-sized resource, rebuilt together when the canvas resizes — one dimension check instead of
/// five that could disagree.
struct Canvas {
    width: u32,
    height: u32,
    /// Has this canvas ever had a FULL plane upload? Born `false` with the textures, which is the honest
    /// state: they hold zeros, and zeros are a flat painting, not the artist's.
    ///
    /// Kept HERE and not on the pass because it is a fact about these textures — resize rebuilds them and
    /// the answer goes back to `false` for free, where a flag one level up would survive the rebuild and
    /// claim a fresh texture was seeded.
    planes_seeded: bool,
    relief: Plane,
    cover: Plane,
    mat0: Plane,
    mat1: Plane,
    out: Plane,
}

/// GPU pass that lights a composited canvas from its relief. One per painter session, held by the shell
/// bridge alongside the [`crate::layer_compositor::LayerCompositor`].
pub struct ImpastoLightPass {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    canvas: Option<Canvas>,
    lut: Option<(Plane, u32, u32)>,
    globals: wgpu::Buffer,
}

impl ImpastoLightPass {
    /// Build the compute pipeline. Cheap — no GPU textures until the first [`Self::run`].
    #[must_use]
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ph2d-render impasto_light shader"),
                source: wgpu::ShaderSource::Wgsl(IMPASTO_LIGHT_WGSL.into()),
            });
        let sampled = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let bgl = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ph2d-render impasto_light bgl"),
                entries: &[
                    sampled(0), // composited, UNLIT
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    sampled(2), // relief
                    sampled(3), // cover
                    sampled(4), // mat0
                    sampled(5), // mat1
                    sampled(6), // spec LUT
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ph2d-render impasto_light layout"),
                bind_group_layouts: &[&bgl],
                immediate_size: 0,
            });
        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-render impasto_light pipeline"),
                layout: Some(&layout),
                module: &shader,
                entry_point: Some("cs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });
        let globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render impasto_light globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            bgl,
            canvas: None,
            lut: None,
            globals,
        }
    }

    /// The lit output of the last [`Self::run`] (`rgba8unorm`). `None` before the first run.
    #[must_use]
    pub fn output_texture(&self) -> Option<&wgpu::Texture> {
        self.canvas.as_ref().map(|c| &c.out.texture)
    }

    /// Light `src` into the internal output and return it. Encodes + submits one compute dispatch.
    ///
    /// # Errors
    ///
    /// [`ImpastoLightError`] for a mis-shaped request. Every arm is a caller bug caught before a single
    /// GPU resource is touched — a mis-sized plane would otherwise read another canvas's pixels and light
    /// the painting from a relief that is not on it.
    pub fn run(
        &mut self,
        gpu: &GpuContext,
        src: &wgpu::Texture,
        input: &ImpastoLightInput<'_>,
    ) -> Result<&wgpu::Texture, ImpastoLightError> {
        input.check()?;
        let (w, h) = (input.width, input.height);
        self.ensure_canvas(gpu, w, h);
        // Asked AFTER `ensure_canvas`, because a resize rebuilt the textures and un-seeded them: asking
        // first would answer about the canvas the artist just left.
        let full = input.plane_region.w == w && input.plane_region.h == h;
        if !full && !self.planes_seeded(w, h) {
            return Err(ImpastoLightError::PlanesNotSeeded);
        }
        self.ensure_lut(gpu, input);
        self.write_planes(gpu, input);
        if full && let Some(c) = self.canvas.as_mut() {
            c.planes_seeded = true;
        }
        self.write_globals(gpu, input);

        let canvas = self.canvas.as_ref().expect("just ensured");
        let lut = &self.lut.as_ref().expect("just ensured").0;
        let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-render impasto_light bg"),
            layout: &self.bgl,
            entries: &[
                bind(0, &src_view),
                bind(1, &canvas.out.view),
                bind(2, &canvas.relief.view),
                bind(3, &canvas.cover.view),
                bind(4, &canvas.mat0.view),
                bind(5, &canvas.mat1.view),
                bind(6, &lut.view),
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.globals.as_entire_binding(),
                },
            ],
        });
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ph2d-render impasto_light encoder"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-render impasto_light pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.impasto_light"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                input.region.w.div_ceil(WORKGROUP_EDGE),
                input.region.h.div_ceil(WORKGROUP_EDGE),
                1,
            );
        }
        gpu.queue.submit([encoder.finish()]);
        Ok(&self.canvas.as_ref().expect("just ensured").out.texture)
    }

    /// Rebuild every canvas-sized resource when the canvas resizes. They are rebuilt TOGETHER on one
    /// dimension check — five independent checks would be five chances for two planes to disagree about
    /// how big the painting is.
    fn ensure_canvas(&mut self, gpu: &GpuContext, w: u32, h: u32) {
        if let Some(c) = &self.canvas
            && c.width == w
            && c.height == h
        {
            return;
        }
        use wgpu::TextureFormat as F;
        use wgpu::TextureUsages as U;
        let read = U::TEXTURE_BINDING | U::COPY_DST;
        self.canvas = Some(Canvas {
            width: w,
            height: h,
            planes_seeded: false,
            relief: plane(gpu, "relief", w, h, F::R32Float, read),
            cover: plane(gpu, "cover", w, h, F::R8Unorm, read),
            mat0: plane(gpu, "mat0", w, h, F::Rgba8Unorm, read),
            mat1: plane(gpu, "mat1", w, h, F::Rgba8Unorm, read),
            out: plane(
                gpu,
                "out",
                w,
                h,
                F::Rgba8Unorm,
                U::STORAGE_BINDING | U::COPY_SRC | U::TEXTURE_BINDING,
            ),
        });
    }

    /// Upload the specular table — once per process, in practice: it is a pure function of nothing but
    /// itself, so the dimension check is a content check too.
    fn ensure_lut(&mut self, gpu: &GpuContext, input: &ImpastoLightInput<'_>) {
        let (w, h) = (input.lut_width, input.rough_levels);
        if let Some((_, lw, lh)) = &self.lut
            && *lw == w
            && *lh == h
        {
            return;
        }
        let p = plane(
            gpu,
            "spec_lut",
            w,
            h,
            wgpu::TextureFormat::R32Float,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        // The LUT is always whole — it is a pure function of nothing but itself, uploaded once per
        // process. It shares `write_plane` because it is the same `write_texture`, not because it is a
        // plane of the painting.
        write_plane(
            gpu,
            &p,
            bytemuck::cast_slice(input.spec_lut),
            w * 4,
            crate::layer_compositor::Region::full(w, h),
        );
        self.lut = Some((p, w, h));
    }

    fn write_planes(&self, gpu: &GpuContext, input: &ImpastoLightInput<'_>) {
        let Some(c) = &self.canvas else { return };
        let r = input.plane_region;
        write_plane(
            gpu,
            &c.relief,
            bytemuck::cast_slice(input.relief),
            r.w * 4,
            r,
        );
        write_plane(gpu, &c.cover, input.cover, r.w, r);
        write_plane(gpu, &c.mat0, input.mat0, r.w * 4, r);
        write_plane(gpu, &c.mat1, input.mat1, r.w * 4, r);
    }

    /// **Do the plane textures for this canvas hold the artist's relief, everywhere?**
    ///
    /// The question a caller must ask before folding only a window. `false` on a fresh or resized canvas
    /// — the textures are zeros there, and lighting from zeros draws the painting flat outside the window.
    ///
    /// The pass answers it because the pass owns the textures. A caller keeping its own "have I seeded
    /// it?" flag would be a second answer to a question about somebody else's state, and it would go on
    /// saying yes through the resize that threw the textures away.
    #[must_use]
    pub fn planes_seeded(&self, width: u32, height: u32) -> bool {
        self.canvas
            .as_ref()
            .is_some_and(|c| c.planes_seeded && c.width == width && c.height == height)
    }

    fn write_globals(&self, gpu: &GpuContext, input: &ImpastoLightInput<'_>) {
        let mut g = Globals {
            lamps: [LampRaw {
                dir: [0.0; 4],
                hlf: [0.0; 4],
                tint: [0.0; 4],
            }; IMPASTO_MAX_LIGHTS],
            n: input.lamps.len() as u32,
            ox: input.region.x,
            oy: input.region.y,
            rw: input.region.w,
            rh: input.region.h,
            pad0: 0,
            pad1: 0,
            pad2: 0,
        };
        for (slot, l) in g.lamps.iter_mut().zip(input.lamps) {
            slot.dir = [l.dir[0], l.dir[1], l.dir[2], 0.0];
            slot.hlf = [l.half[0], l.half[1], l.half[2], 0.0];
            slot.tint = [l.tint[0], l.tint[1], l.tint[2], 0.0];
        }
        gpu.queue
            .write_buffer(&self.globals, 0, bytemuck::bytes_of(&g));
    }
}

fn bind(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

fn plane(
    gpu: &GpuContext,
    what: &str,
    w: u32,
    h: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> Plane {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("ph2d-render impasto_light {what}")),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Plane { texture, view }
}

/// Upload one plane's WINDOW. `bytes` is exactly `region.w × region.h` texels, tightly packed, and lands
/// at the window's origin — the texels outside keep what the last upload put there, which is what makes
/// the canvas-sized texture survive a rect-sized fold.
fn write_plane(
    gpu: &GpuContext,
    p: &Plane,
    bytes: &[u8],
    row: u32,
    region: crate::layer_compositor::Region,
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &p.texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: region.x,
                y: region.y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(row),
            rows_per_image: Some(region.h),
        },
        wgpu::Extent3d {
            width: region.w,
            height: region.h,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(test)]
#[path = "impasto_light_tests.rs"]
mod tests;
