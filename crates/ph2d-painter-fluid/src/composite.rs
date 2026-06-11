//! GPU wet-field composite (ADR-0049 W15.3) — the per-frame K–M glaze on the GPU,
//! removing the pigment readback stall. Band-for-band mirror of
//! [`ph2d_painter_brush::wet_composite::composite_wet_field_cpu`] (the parity
//! ground truth; the gate proves they agree on a real device).
//!
//! The compositor is built once per device. Inputs each composite: the GPU-resident
//! low-res [`PIG_CH`]-channel pigment field (ADR-0080: mass-weighted K/S + err + mass),
//! the canvas-res backdrop (RGBA8), and the constant spectral basis (uploaded once via
//! [`ph2d_painter_brush::pigment_mix::spectral_basis`]). The pigment colour/opacity are
//! reduced PER-PIXEL from the field in the shader (no per-stroke brush). Output is canvas-res RGBA8
//! — to a storage buffer (the readback / first-milestone path here) or, in the
//! shell integration, a preview texture (zero readback).

use ph2d_painter_brush::diffusion::{PIG_CH, WetCell};
use ph2d_painter_brush::pigment_mix::{SPECTRAL_BANDS, spectral_basis};
use ph2d_painter_brush::wet_composite::{WET_COMPOSITE_SS, composite_canvas_region};

/// The GPU composite shader source (mirror of the CPU `wet_composite`). Embedded so
/// a dev-test validates it through naga before any GPU init.
pub const COMPOSITE_WGSL: &str = include_str!("shader/composite.wgsl");

/// The WGSL `U` uniform, byte-for-byte (48 B). `#[repr(C)]` Pod for a zero-copy
/// `write_buffer` (HR-3). ADR-0080: pigment colour/opacity are now PER-PIXEL (reduced
/// from the field), so the per-stroke `pcol`/`color_sum` uniforms are gone.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuU {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    inv: f32,
    coverage_k: f32,
    ss: u32,
    origin_x: u32,
    origin_y: u32,
    end_x: u32,
    end_y: u32,
    /// Wet-paper sheen flag (1 = ON) — consumed ONLY by the preview-texture
    /// kernels (`cs_premul_tex`/`cs_straight_tex`); `cs_composite` ignores it,
    /// so `out_buf` (the bake source) is never sheened. 0 ⇒ byte-identical.
    wet_sheen: u32,
}

/// The WGSL `Coeffs` storage struct, byte-for-byte (960 B). The flattened constant
/// spectral basis (`base[7][24]` + `m[3][24]`); the brush `ks`/`err` are now per-pixel
/// (ADR-0080), reduced from the field in the shader, so they're no longer uploaded.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuCoeffs {
    base: [f32; 168], // 7 * 24
    m: [f32; 72],     // 3 * 24
}

impl GpuCoeffs {
    /// Pack the constant spectral basis for the shader (constant per device).
    fn build() -> Self {
        // Pinned: the shader hard-codes NB=24 (and the flatten strides below).
        assert_eq!(
            SPECTRAL_BANDS, 24,
            "composite.wgsl assumes 24 spectral bands"
        );
        let (base, m) = spectral_basis();
        let mut out = Self {
            base: [0.0; 168],
            m: [0.0; 72],
        };
        for (k, row) in base.iter().enumerate() {
            out.base[k * 24..k * 24 + 24].copy_from_slice(row);
        }
        for (c, row) in m.iter().enumerate() {
            out.m[c * 24..c * 24 + 24].copy_from_slice(row);
        }
        out
    }
}

/// Persistent per-stroke GPU state for the fast path — buffers + bind group reused
/// across frames so the hot loop NEVER allocates or re-uploads the canvas. The
/// backdrop + coeffs are constant for a stroke (uploaded once in
/// [`FluidCompositor::begin_stroke`]); only the 64-byte `params` (the region) is
/// written per frame.
struct CompositeState {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    scale: u32,
    coverage_k: f32,
    ss: u32,
    params: wgpu::Buffer,
    coeffs: wgpu::Buffer,
    out: wgpu::Buffer,
    backdrop: wgpu::Buffer,
    /// ADR-0084 paper-reveal — the session's original canvas content (canvas-res RGBA8, like
    /// `backdrop`); the shader lerps lifted pixels back toward it (never toward transparency).
    paper: wgpu::Buffer,
    staging: wgpu::Buffer,
    /// Owned all-zero `lifted_frac` (ADR-0084) bound at binding 5 when `begin_stroke` is called
    /// without a live lift buffer — kept here so it outlives the `bind` it backs. Sized to `gw·gh`.
    dormant_lifted: wgpu::Buffer,
    /// Owned all-zero water bound at binding 7 when `begin_stroke` is called without a live
    /// solver water buffer (same dormant-zero pattern as `dormant_lifted`): zero water ⇒
    /// `wet = 0` ⇒ the wet-paper sheen is a no-op. Sized to `gw·gh`.
    dormant_water: wgpu::Buffer,
    bind: wgpu::BindGroup,
    /// E4 — the PREMULTIPLIED live-preview texture (canvas-res rgba8unorm). `cs_premul_init`
    /// fills it with the premultiplied backdrop at `begin_stroke`; `cs_premul_tex` overwrites the
    /// composited region each [`FluidCompositor::composite_frame_to_texture`]. The sprite renderer
    /// samples it directly (`TEXTURE_BINDING`) — no staging readback, no CPU premultiply,
    /// no re-upload. Recreated on canvas resize like the other canvas-res resources.
    preview_tex: wgpu::Texture,
    /// The group(1) bind for the premul passes (just the storage view of `preview_tex`).
    preview_bind: wgpu::BindGroup,
    /// **E5 (ADR-0078 S2 multi-layer)** — the STRAIGHT-alpha live texture
    /// (canvas-res `rgba8unorm`, `COPY_SRC`) the shell injects into the GPU
    /// layer compositor's slice cache for the active layer (its slices are
    /// straight; premul happens at the end of the layer chain). LAZY: only a
    /// non-trivial GPU-representable stack pays the VRAM — created (+ seeded
    /// with the straight backdrop via `cs_straight_init`) on the first
    /// [`FluidCompositor::composite_frame_to_straight_texture`] of a stroke,
    /// and dropped at every `begin_stroke` (the backdrop changed).
    straight: Option<(wgpu::Texture, wgpu::BindGroup)>,
}

/// An in-flight async readback (ADR-0078 S2 — pipelined composite). The band copy is
/// submitted + `map_async`'d WITHOUT a per-frame `device.poll(wait)` (which drained the
/// whole GPU queue, ~2.6 ms/frame — the measured 250→140 FPS stall); the result is
/// read the NEXT frame once a non-blocking `poll(Poll)` has fired its callback.
struct PendingReadback {
    rx: std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    band_bytes: u64,
    rect: (u32, u32, u32, u32),
}

/// The live GPU compositor: one compute pipeline + bind-group layout (built once per
/// device) + persistent per-stroke buffers ([`CompositeState`]) for the hot loop.
/// The one-shot `composite_*` methods (per-call buffers) are the unit-test / CPU
/// convenience; the shell drives [`Self::begin_stroke`] + [`Self::composite_frame_pipelined`].
pub struct FluidCompositor {
    pipeline: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    /// E4 — group(1) layout (the write-only preview storage texture) for the premul passes.
    tex_bgl: wgpu::BindGroupLayout,
    /// E4 — `cs_premul_tex`: out_buf region → premultiplied preview texture.
    premul_tex_pipeline: wgpu::ComputePipeline,
    /// E4 — `cs_premul_init`: backdrop → premultiplied preview texture (stroke start).
    premul_init_pipeline: wgpu::ComputePipeline,
    /// E5 — `cs_straight_tex`: out_buf region → STRAIGHT-alpha texture (the layer-
    /// compositor injection source; same group(1) layout, separate bind group).
    straight_tex_pipeline: wgpu::ComputePipeline,
    /// E5 — `cs_straight_init`: backdrop → straight texture (lazy first-frame seed).
    straight_init_pipeline: wgpu::ComputePipeline,
    state: Option<CompositeState>,
    /// The previous frame's readback, awaiting a non-blocking poll (pipelined path).
    pending: Option<PendingReadback>,
    /// Wet-paper sheen toggle for the preview-texture passes ([`Self::set_wet_sheen`]).
    /// VIEW-ONLY: `cs_composite`/`out_buf` (the bake source) never see it. Default
    /// `false` ⇒ byte-identical output (proven by `wet_sheen_off_is_byte_identical`).
    wet_sheen: bool,
}

impl FluidCompositor {
    /// Build the pipeline + bind-group layout on `device`.
    #[must_use]
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-painter-fluid composite"),
            source: wgpu::ShaderSource::Wgsl(COMPOSITE_WGSL.into()),
        });
        let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage(1, true),  // pig_in (read)
                storage(2, true),  // backdrop (read)
                storage(3, false), // out_buf (read_write)
                storage(4, true),  // coeffs (read)
                storage(5, true),  // lifted_frac (read — ADR-0084)
                storage(6, true),  // paper_canvas (read — ADR-0084 paper-reveal)
                storage(7, true),  // water (read — wet-paper sheen, preview-tex passes only)
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("ph2d-painter-fluid composite pipeline"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cs_composite"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        // E4 — the premul-to-texture passes live in the SAME module but bind the preview
        // storage texture as group(1), so the long-lived group(0) buffer layout (and every
        // existing bind group built against it) is untouched; the premul pipelines reuse
        // the per-stroke group(0) bind as-is.
        let tex_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite preview-tex bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            }],
        });
        let premul_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-painter-fluid composite premul layout"),
            bind_group_layouts: &[&bgl, &tex_bgl],
            immediate_size: 0,
        });
        let premul_pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&premul_layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let premul_tex_pipeline = premul_pipeline("cs_premul_tex");
        let premul_init_pipeline = premul_pipeline("cs_premul_init");
        // E5 — same group(0)+group(1) layouts (the straight texture binds the same
        // group(1) storage-texture slot through its own bind group at dispatch).
        let straight_tex_pipeline = premul_pipeline("cs_straight_tex");
        let straight_init_pipeline = premul_pipeline("cs_straight_init");
        Self {
            pipeline,
            bgl,
            tex_bgl,
            premul_tex_pipeline,
            premul_init_pipeline,
            straight_tex_pipeline,
            straight_init_pipeline,
            state: None,
            pending: None,
            wet_sheen: false,
        }
    }

    /// Toggle the wet-paper sheen for the preview-texture passes (the bridge sets it
    /// each frame from the tool's `show_wet`). VIEW-ONLY: only `cs_premul_tex` /
    /// `cs_straight_tex` consume the flag — the canonical `out_buf` composite (and so
    /// the readback bake into `canvas_rgba`) stays sheen-free, which is what makes the
    /// wash "dry lighter" like real watercolor. `false` ⇒ byte-identical output.
    pub fn set_wet_sheen(&mut self, on: bool) {
        self.wet_sheen = on;
    }

    /// Create an all-zero `array<f32>` buffer sized to `gw·gh` for a dormant per-cell field —
    /// the backdrop-lift accumulator (ADR-0084) or the wet-sheen water: a fresh STORAGE buffer
    /// reads as zero ⇒ `lf = 0` / `wet = 0` everywhere ⇒ byte-identical output. Returned owned
    /// so the caller keeps it alive alongside the bind group it backs.
    fn make_dormant_field(device: &wgpu::Device, gw: u32, gh: u32, label: &str) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: ((gw as u64) * (gh as u64) * 4).max(4),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        })
    }

    /// **Fast-path stroke setup (call once per stroke / when the backdrop or brush
    /// changes).** (Re)allocates the persistent canvas buffers only on a size change,
    /// uploads the backdrop + the amortised brush coeffs ONCE, and rebuilds the bind
    /// group against the solver's resident `pigment_buf`. After this, [`Self::composite_frame`]
    /// runs each frame writing only the 64-byte region uniform — no per-frame
    /// allocation, no canvas re-upload (the W15.3 hot-loop perf fix).
    #[allow(clippy::too_many_arguments)]
    pub fn begin_stroke(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        ss: u32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        // ADR-0084 paper-reveal — the session's original canvas content (`cw·ch·4` RGBA8, same
        // layout as `backdrop_rgba`); the shader lerps lifted pixels back toward it. Callers that
        // don't lift pass the BACKDROP slice (`mix(b, b, lf) = b` ⇒ exact no-op regardless of `lf`).
        paper_rgba: &[u8],
        // ADR-0084 — the solver's `lifted_frac_buffer()` so the compositor reveals the paper
        // where dry paint was lifted. `None` ⇒ bind the owned all-zero dormant buffer (the
        // backdrop-lift branch is inert ⇒ byte-identical to the pre-ADR-0084 output).
        lifted_frac_buf: Option<&wgpu::Buffer>,
        // Wet-paper sheen — the solver's `water_buffer()` (`gw·gh` f32) so the preview-texture
        // passes can darken wet regions + brighten the meniscus. `None` ⇒ owned all-zero dormant
        // buffer ⇒ `wet = 0` ⇒ sheen no-op (same pattern as `lifted_frac_buf`).
        water_buf: Option<&wgpu::Buffer>,
    ) {
        // Drain any in-flight pipelined readback from a PRIOR stroke (its map may still
        // hold `staging`): complete + unmap it before this stroke reuses the buffer, so
        // the primed (sync) first frame can map it cleanly. Discard the stale band.
        if self.pending.take().is_some() {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(st) = self.state.as_ref() {
                st.staging.unmap();
            }
        }
        let canvas_bytes = (cw as usize * ch as usize * 4) as u64;
        // (Re)create the canvas-sized buffers only when the size changes.
        let resized = self.state.as_ref().map(|s| (s.cw, s.ch)) != Some((cw, ch));
        if resized {
            let params = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite params (persistent)"),
                size: core::mem::size_of::<GpuU>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let coeffs = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite coeffs (persistent)"),
                size: core::mem::size_of::<GpuCoeffs>() as u64,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let backdrop = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite backdrop (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let paper = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite paper (persistent, ADR-0084)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let out = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite out (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let staging = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("composite readback (persistent)"),
                size: canvas_bytes,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            // E4 — the premultiplied preview texture (canvas-res). STORAGE_BINDING for the
            // premul passes, TEXTURE_BINDING so the sprite renderer samples it directly
            // (step 2 wiring), COPY_SRC for the parity-test readback, COPY_DST kept so the
            // shell can patch it from CPU bytes if it ever needs to (cheap to grant now).
            let preview_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("composite preview texture (premultiplied, E4)"),
                size: wgpu::Extent3d {
                    width: cw,
                    height: ch,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let preview_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite preview-tex bg (E4)"),
                layout: &self.tex_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &preview_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                }],
            });
            // Owned dormant lifted_frac (ADR-0084) — used when no live lift buffer is bound below.
            let dormant_lifted =
                Self::make_dormant_field(device, gw, gh, "composite dormant lifted_frac");
            let lifted_res = lifted_frac_buf
                .unwrap_or(&dormant_lifted)
                .as_entire_binding();
            // Owned dormant water (wet-paper sheen) — used when no live water buffer is bound.
            let dormant_water = Self::make_dormant_field(device, gw, gh, "composite dormant water");
            let water_res = water_buf.unwrap_or(&dormant_water).as_entire_binding();
            // Placeholder bind; rebuilt below (needs pigment_buf).
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite bg (persistent)"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: pigment_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: backdrop.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: out.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: coeffs.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: lifted_res,
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: paper.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: water_res,
                    },
                ],
            });
            self.state = Some(CompositeState {
                cw,
                ch,
                gw,
                gh,
                scale,
                coverage_k,
                ss,
                params,
                coeffs,
                out,
                backdrop,
                paper,
                staging,
                dormant_lifted,
                dormant_water,
                bind,
                preview_tex,
                preview_bind,
                straight: None,
            });
        }
        // SAFETY of unwrap: `state` is Some after the resize branch (or already was).
        let st = self.state.as_mut().expect("composite state set");
        // E5 — the straight texture is seeded from the STROKE's backdrop, which is
        // about to change: drop it so the next straight-texture frame (if any)
        // lazily recreates + re-seeds it. Also frees the VRAM between strokes.
        st.straight = None;
        st.gw = gw;
        st.gh = gh;
        st.scale = scale;
        st.coverage_k = coverage_k;
        st.ss = ss;
        // Backdrop + paper + coeffs are constant for the stroke → upload once here.
        queue.write_buffer(&st.backdrop, 0, backdrop_rgba);
        queue.write_buffer(&st.paper, 0, paper_rgba);
        queue.write_buffer(&st.coeffs, 0, bytemuck::bytes_of(&GpuCoeffs::build()));
        // Rebuild the bind group each stroke so it tracks the current resident pig_a
        // (cheap — just resource references; the canvas buffers are reused).
        if !resized {
            // The grid dims can change without a canvas resize (the dormant fields are sized to
            // `gw·gh`), so re-create them when the size no longer matches before binding.
            if st.dormant_lifted.size() != ((gw as u64) * (gh as u64) * 4).max(4) {
                st.dormant_lifted =
                    Self::make_dormant_field(device, gw, gh, "composite dormant lifted_frac");
            }
            if st.dormant_water.size() != ((gw as u64) * (gh as u64) * 4).max(4) {
                st.dormant_water =
                    Self::make_dormant_field(device, gw, gh, "composite dormant water");
            }
            let lifted_res = lifted_frac_buf
                .unwrap_or(&st.dormant_lifted)
                .as_entire_binding();
            let water_res = water_buf.unwrap_or(&st.dormant_water).as_entire_binding();
            st.bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite bg (persistent)"),
                layout: &self.bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: st.params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: pigment_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: st.backdrop.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: st.out.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: st.coeffs.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: lifted_res,
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: st.paper.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: water_res,
                    },
                ],
            });
        }
        // E4 — seed the preview texture with the PREMULTIPLIED backdrop, entirely on the
        // GPU: one full-canvas `cs_premul_init` dispatch per stroke (origin 0 → end cw/ch,
        // reading the `backdrop` buffer just uploaded above). The per-frame composite
        // regions only cover the wet band, so pixels never composited must already show
        // the backdrop when the renderer samples the texture. Clobbering the region
        // uniform here is safe — every `composite_frame*` call rewrites it first.
        let uni = GpuU {
            cw,
            ch,
            gw,
            gh,
            inv: 1.0 / scale as f32,
            coverage_k,
            ss,
            origin_x: 0,
            origin_y: 0,
            end_x: cw,
            end_y: ch,
            wet_sheen: 0,
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite preview init (E4)"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite preview init pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_init"),
            });
            pass.set_pipeline(&self.premul_init_pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.set_bind_group(1, &st.preview_bind, &[]);
            pass.dispatch_workgroups(cw.div_ceil(8), ch.div_ceil(8), 1);
        }
        queue.submit([enc.finish()]);
    }

    /// **Fast-path per-frame composite (after [`Self::begin_stroke`]).** Writes only
    /// the 64-byte region uniform, dispatches over the bbox, and reads back the wet
    /// row band. Reuses the persistent buffers/bind — zero per-frame allocation, zero
    /// canvas re-upload. Returns `(band, rect)` like [`Self::composite_buffer_rows`];
    /// `(vec![], (0,0,0,0))` when the region is empty.
    #[must_use]
    pub fn composite_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_region: (u32, u32, u32, u32),
    ) -> (Vec<u8>, (u32, u32, u32, u32)) {
        // Drain any in-flight PIPELINED readback first: it left `staging` mapped, and
        // this sync path re-maps `staging` below — re-mapping an already-mapped buffer
        // is a wgpu validation error that ABORTS the process (the crash when a sync
        // bake — e.g. the undo flush — followed pipelined drying frames). Complete +
        // unmap it before reuse; the stale band is superseded by this composite.
        if self.pending.take().is_some() {
            let _ = device.poll(wgpu::PollType::wait_indefinitely());
            if let Some(st) = self.state.as_ref() {
                st.staging.unmap();
            }
        }
        let Some(st) = self.state.as_ref() else {
            return (Vec::new(), (0, 0, 0, 0));
        };
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region(grid_region, st.scale, st.cw, st.ch);
        if py_hi <= py_lo || px_hi <= px_lo {
            return (Vec::new(), (0, 0, 0, 0));
        }
        let uni = GpuU {
            cw: st.cw,
            ch: st.ch,
            gw: st.gw,
            gh: st.gh,
            inv: 1.0 / st.scale as f32,
            coverage_k: st.coverage_k,
            ss: st.ss,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            wet_sheen: 0,
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));

        let row_bytes = u64::from(st.cw) * 4;
        let band_off = u64::from(py_lo) * row_bytes;
        let band_bytes = u64::from(py_hi - py_lo) * row_bytes;
        let (rw, rh) = (px_hi - px_lo, py_hi - py_lo);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite frame"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite frame pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_sync"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&st.out, band_off, &st.staging, 0, band_bytes);
        queue.submit([enc.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        st.staging
            .slice(0..band_bytes)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let rows = st.staging.slice(0..band_bytes).get_mapped_range().to_vec();
        st.staging.unmap();
        (rows, (px_lo, py_lo, px_hi, py_hi))
    }

    /// **Pipelined per-frame composite (ADR-0078 S2 — no per-frame `device.poll(wait)`).**
    /// Like [`Self::composite_frame`] but the band readback is ASYNC: this frame's
    /// composite + copy are submitted and `map_async`'d without blocking, and the band
    /// RETURNED is the PREVIOUS frame's (read after a non-blocking `poll(Poll)` fired
    /// its callback). That removes the ~2.6 ms/frame stall where the synchronous poll
    /// drained the entire GPU queue (incl. the main UI render) — the measured 250→140
    /// FPS regression. The preview lags one frame (≈4 ms at 250 FPS, imperceptible);
    /// the field is frozen for several frames before it dries + drops, so the final
    /// state is always blitted before the drop (no pen-up special-case needed).
    /// Returns `(vec![], (0,0,0,0))` on the first frame / when the prior map isn't ready.
    #[must_use]
    pub fn composite_frame_pipelined(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_region: (u32, u32, u32, u32),
    ) -> (Vec<u8>, (u32, u32, u32, u32)) {
        // Fire any completed map callbacks WITHOUT blocking (the key vs the old path).
        let _ = device.poll(wgpu::PollType::Poll);
        // Read the prior frame's band if its map has completed (1-frame-late).
        let mut result = (Vec::new(), (0, 0, 0, 0));
        if let Some(p) = self.pending.take() {
            match p.rx.try_recv() {
                Ok(Ok(())) => {
                    if let Some(st) = self.state.as_ref() {
                        let rows = st
                            .staging
                            .slice(0..p.band_bytes)
                            .get_mapped_range()
                            .to_vec();
                        st.staging.unmap();
                        result = (rows, p.rect);
                    }
                }
                Ok(Err(_)) => {
                    // Map failed — unmap defensively; staging is free to reuse.
                    if let Some(st) = self.state.as_ref() {
                        st.staging.unmap();
                    }
                }
                Err(_) => {
                    // GPU not done yet (rare) — keep waiting, don't reuse staging.
                    self.pending = Some(p);
                    return result;
                }
            }
        }
        // Submit THIS frame's composite + copy + async map (staging is free now).
        let Some(st) = self.state.as_ref() else {
            return result;
        };
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region(grid_region, st.scale, st.cw, st.ch);
        if py_hi <= py_lo || px_hi <= px_lo {
            return result;
        }
        let uni = GpuU {
            cw: st.cw,
            ch: st.ch,
            gw: st.gw,
            gh: st.gh,
            inv: 1.0 / st.scale as f32,
            coverage_k: st.coverage_k,
            ss: st.ss,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            wet_sheen: 0,
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));
        let row_bytes = u64::from(st.cw) * 4;
        let band_off = u64::from(py_lo) * row_bytes;
        let band_bytes = u64::from(py_hi - py_lo) * row_bytes;
        let (rw, rh) = (px_hi - px_lo, py_hi - py_lo);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite frame (pipelined)"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite frame pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_pipe"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        enc.copy_buffer_to_buffer(&st.out, band_off, &st.staging, 0, band_bytes);
        queue.submit([enc.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        st.staging
            .slice(0..band_bytes)
            .map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
        self.pending = Some(PendingReadback {
            rx,
            band_bytes,
            rect: (px_lo, py_lo, px_hi, py_hi),
        });
        result
    }

    /// **E4 — per-frame composite straight into the premultiplied preview texture
    /// (after [`Self::begin_stroke`]).** Same region dispatch as
    /// [`Self::composite_frame_pipelined`] (`cs_composite` over the wet bbox) but with
    /// NO staging copy, NO `map_async`, NO readback: a second dispatch
    /// (`cs_premul_tex`, same region, same pass — WebGPU's implicit inter-dispatch
    /// sync) premultiplies the fresh straight-sRGB8 `out_buf` bytes into
    /// [`Self::preview_texture`] with the EXACT byte semantics of the shell's CPU
    /// `premultiply_rgba8` (`rgb' = (rgb·a + 127)/255` on the sRGB-encoded bytes).
    /// The sprite renderer samples the texture directly — the GPU→CPU→GPU round-trip
    /// is gone; CPU readback remains only for the pen-up bake (the untouched
    /// readback entry points). Returns the composited canvas rect
    /// `(px_lo, py_lo, px_hi, py_hi)`, or `None` when there's no stroke state / the
    /// region is empty (nothing was dispatched).
    pub fn composite_frame_to_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_region: (u32, u32, u32, u32),
    ) -> Option<(u32, u32, u32, u32)> {
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite frame (to texture, E4)"),
        });
        let rect = self.encode_frame_to_texture(queue, &mut enc, grid_region);
        queue.submit([enc.finish()]);
        rect
    }

    /// **Encode-only sibling of [`Self::composite_frame_to_texture`]** (Watercolor v2
    /// R1, ADR-0085 §2.3-I1): the `cs_composite` + `cs_premul_tex` passes are encoded
    /// into the caller's `enc` (NO submit), so the shell folds the sim, this composite
    /// and the preview-slot copy into ONE `queue.submit`. Same byte semantics as the
    /// wrapper. Returns the composited canvas rect, or `None` when there's no stroke
    /// state / the region is empty (nothing encoded).
    pub fn encode_frame_to_texture(
        &mut self,
        queue: &wgpu::Queue,
        enc: &mut wgpu::CommandEncoder,
        grid_region: (u32, u32, u32, u32),
    ) -> Option<(u32, u32, u32, u32)> {
        let st = self.state.as_ref()?;
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region(grid_region, st.scale, st.cw, st.ch);
        if py_hi <= py_lo || px_hi <= px_lo {
            return None;
        }
        let uni = GpuU {
            cw: st.cw,
            ch: st.ch,
            gw: st.gw,
            gh: st.gh,
            inv: 1.0 / st.scale as f32,
            coverage_k: st.coverage_k,
            ss: st.ss,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            wet_sheen: u32::from(self.wet_sheen),
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));
        let (rw, rh) = (px_hi - px_lo, py_hi - py_lo);
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite frame to-texture pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_tex"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
            pass.set_pipeline(&self.premul_tex_pipeline);
            pass.set_bind_group(1, &st.preview_bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        Some((px_lo, py_lo, px_hi, py_hi))
    }

    /// E4 — the premultiplied live-preview texture (canvas-res `Rgba8Unorm`,
    /// `TEXTURE_BINDING` for direct sampling), or `None` before [`Self::begin_stroke`] /
    /// after [`Self::end_stroke`]. Holds the premultiplied backdrop where no frame
    /// region ever composited.
    pub fn preview_texture(&self) -> Option<&wgpu::Texture> {
        self.state.as_ref().map(|st| &st.preview_tex)
    }

    /// **E5 — per-frame composite into the STRAIGHT-alpha texture (after
    /// [`Self::begin_stroke`]).** Same shape as [`Self::composite_frame_to_texture`]
    /// (one pass: `cs_composite` over the wet bbox, then a texture store of the SAME
    /// region — WebGPU's implicit inter-dispatch sync) but the store is
    /// `cs_straight_tex`: the raw straight-sRGB8 `out_buf` bytes, NO premultiply —
    /// the GPU layer compositor consumes straight slices (premul happens once at the
    /// end of the layer chain). The texture is created lazily on the first call of a
    /// stroke and seeded full-canvas with the straight backdrop (`cs_straight_init`,
    /// its own submit so the region uniform can differ), so out-of-region texels hold
    /// the active layer's pre-stroke pixels. Returns the composited canvas rect, or
    /// `None` when there's no stroke state / the region is empty.
    pub fn composite_frame_to_straight_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_region: (u32, u32, u32, u32),
    ) -> Option<(u32, u32, u32, u32)> {
        // Disjoint field borrows: `state` is &mut, the pipelines are read-only.
        let st = self.state.as_mut()?;
        let (px_lo, py_lo, px_hi, py_hi) =
            composite_canvas_region(grid_region, st.scale, st.cw, st.ch);
        if py_hi <= py_lo || px_hi <= px_lo {
            return None;
        }
        // Lazy create + full-canvas seed (first straight frame of the stroke).
        if st.straight.is_none() {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("composite straight texture (E5)"),
                size: wgpu::Extent3d {
                    width: st.cw,
                    height: st.ch,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("composite straight-tex bg (E5)"),
                layout: &self.tex_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                }],
            });
            // Seed the FULL canvas with the straight backdrop. Separate submit: the
            // region uniform must cover the whole canvas here but only the wet rect
            // below, and `write_buffer` orders against subsequently submitted work.
            let uni = GpuU {
                cw: st.cw,
                ch: st.ch,
                gw: st.gw,
                gh: st.gh,
                inv: 1.0 / st.scale as f32,
                coverage_k: st.coverage_k,
                ss: st.ss,
                origin_x: 0,
                origin_y: 0,
                end_x: st.cw,
                end_y: st.ch,
                wet_sheen: 0,
            };
            queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("composite straight init (E5)"),
            });
            {
                let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("composite straight init pass"),
                    timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_init"),
                });
                pass.set_pipeline(&self.straight_init_pipeline);
                pass.set_bind_group(0, &st.bind, &[]);
                pass.set_bind_group(1, &bind, &[]);
                pass.dispatch_workgroups(st.cw.div_ceil(8), st.ch.div_ceil(8), 1);
            }
            queue.submit([enc.finish()]);
            st.straight = Some((tex, bind));
        }
        let (_, straight_bind) = st.straight.as_ref().expect("just ensured");
        let uni = GpuU {
            cw: st.cw,
            ch: st.ch,
            gw: st.gw,
            gh: st.gh,
            inv: 1.0 / st.scale as f32,
            coverage_k: st.coverage_k,
            ss: st.ss,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            wet_sheen: u32::from(self.wet_sheen),
        };
        queue.write_buffer(&st.params, 0, bytemuck::bytes_of(&uni));
        let (rw, rh) = (px_hi - px_lo, py_hi - py_lo);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite frame (to straight texture, E5)"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite frame to-straight-texture pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_straight"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &st.bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
            pass.set_pipeline(&self.straight_tex_pipeline);
            pass.set_bind_group(1, straight_bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        queue.submit([enc.finish()]);
        Some((px_lo, py_lo, px_hi, py_hi))
    }

    /// E5 — the straight-alpha live texture (canvas-res `Rgba8Unorm`, `COPY_SRC`
    /// for the layer-compositor slice injection), or `None` before the first
    /// [`Self::composite_frame_to_straight_texture`] of the current stroke.
    pub fn straight_texture(&self) -> Option<&wgpu::Texture> {
        self.state
            .as_ref()
            .and_then(|st| st.straight.as_ref())
            .map(|(tex, _)| tex)
    }

    /// Drop the persistent stroke state (frees the canvas buffers) — call when the
    /// field is gone so VRAM isn't held between strokes.
    pub fn end_stroke(&mut self) {
        self.state = None;
        self.pending = None;
    }

    /// One-shot composite to an RGBA8 byte buffer, uploading a CPU pigment field
    /// (the unit-test / CPU-fallback convenience). For the per-frame GPU path use
    /// [`Self::composite_buffer`] with the solver's resident pigment buffer (no
    /// upload, no readback of pigment). `pigment` is the low-res field (`gw*gh`, xyz
    /// mass); `backdrop_rgba` is canvas-res (`cw*ch*4`); `grid_region` scopes work.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_to_rgba(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment: &[WetCell],
        backdrop_rgba: &[u8],
        grid_region: (u32, u32, u32, u32),
    ) -> Vec<u8> {
        let pig_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite pig_in (upload)"),
            size: (pigment.len() * PIG_CH * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Planar (SoA) upload to match the shader's `pidx` layout (ADR-0085) — the
        // production path reads the solver's already-planar `total`; this one-shot
        // test/convenience path transposes the cell-major `pigment` the same way.
        queue.write_buffer(&pig_buf, 0, bytemuck::cast_slice(&crate::solver::pack_soa(pigment)));
        self.composite_buffer(
            device,
            queue,
            gw,
            gh,
            cw,
            ch,
            scale,
            coverage_k,
            &pig_buf,
            backdrop_rgba,
            grid_region,
        )
    }

    /// Composite reading an EXTERNAL pigment buffer (`array<vec4<f32>>`, `gw*gh`) —
    /// bind the solver's [`crate::FluidSolver::pigment_buffer`] here to composite the
    /// GPU-resident bloomed pigment with NO pigment readback (the W15.3 stall fix).
    /// Returns the full canvas RGBA8 (pixels outside the padded region equal the
    /// backdrop, matching the CPU reference's untouched-pixel invariant).
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_buffer(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        grid_region: (u32, u32, u32, u32),
    ) -> Vec<u8> {
        let npix = (cw as usize) * (ch as usize);
        let canvas_bytes = (npix * 4) as u64;
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        let (params_buf, coeffs_buf, backdrop_buf, paper_buf, dormant_lifted, out_buf, bind) = self
            .build_buffers(
                device,
                queue,
                gw,
                gh,
                cw,
                ch,
                scale,
                coverage_k,
                pigment_buf,
                backdrop_rgba,
                // ADR-0084 paper-reveal: the one-shot path binds the backdrop as the paper —
                // `mix(b, b, lf) = b`, an exact no-op regardless of `lf`.
                backdrop_rgba,
                grid_region,
                None,
            );
        let _keep = (
            params_buf,
            coeffs_buf,
            backdrop_buf,
            paper_buf,
            dormant_lifted,
        );

        let (rw, rh) = (px_hi.saturating_sub(px_lo), py_hi.saturating_sub(py_lo));
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite enc"),
        });
        if rw > 0 && rh > 0 {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_buffer"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite readback"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(&out_buf, 0, &staging, 0, canvas_bytes);
        queue.submit([enc.finish()]);

        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        staging.unmap();
        out
    }

    /// Like [`Self::composite_buffer`] but reads back ONLY the contiguous wet
    /// **row band** `[py_lo, py_hi)` (full width, a single contiguous copy) — the
    /// per-frame shell path. Returns `(band, (px_lo, py_lo, px_hi, py_hi))`: the
    /// full-width band bytes `(py_hi-py_lo)*cw*4` plus the composited canvas **rect**.
    /// The caller blits ONLY the rect's columns `[px_lo, px_hi)` of each band row
    /// over `canvas_rgba` — outside the rect the pigment is frozen and MUST be left
    /// untouched (a full-width blit would erase parts of the stroke that share rows
    /// with the active wet front). Returns `(vec![], (0,0,0,0))` when empty.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn composite_buffer_rows(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        grid_region: (u32, u32, u32, u32),
    ) -> (Vec<u8>, (u32, u32, u32, u32)) {
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        if py_hi <= py_lo || px_hi <= px_lo || cw == 0 {
            return (Vec::new(), (0, 0, 0, 0));
        }
        let row_bytes = (cw * 4) as u64;
        let band_off = u64::from(py_lo) * row_bytes;
        let band_bytes = u64::from(py_hi - py_lo) * row_bytes;

        let (params_buf, coeffs_buf, backdrop_buf, paper_buf, dormant_lifted, out_buf, bind) = self
            .build_buffers(
                device,
                queue,
                gw,
                gh,
                cw,
                ch,
                scale,
                coverage_k,
                pigment_buf,
                backdrop_rgba,
                // ADR-0084 paper-reveal: the one-shot path binds the backdrop as the paper —
                // `mix(b, b, lf) = b`, an exact no-op regardless of `lf`.
                backdrop_rgba,
                grid_region,
                None,
            );
        let (rw, rh) = (px_hi.saturating_sub(px_lo), py_hi.saturating_sub(py_lo));

        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("composite rows enc"),
        });
        if rw > 0 && rh > 0 {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("composite rows pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("fluid.comp_rows"),
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind, &[]);
            pass.dispatch_workgroups(rw.div_ceil(8), rh.div_ceil(8), 1);
        }
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite rows readback"),
            size: band_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(&out_buf, band_off, &staging, 0, band_bytes);
        queue.submit([enc.finish()]);
        // Keep the per-composite buffers alive until the GPU finishes (they back the
        // dispatch + the copy). `_keep` drops after the readback poll below.
        let _keep = (
            params_buf,
            coeffs_buf,
            backdrop_buf,
            paper_buf,
            dormant_lifted,
            out_buf,
        );

        let (tx, rx) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().expect("map channel").expect("mapped");
        let mapped = staging.slice(..).get_mapped_range();
        let rows = mapped.to_vec();
        drop(mapped);
        staging.unmap();
        (rows, (px_lo, py_lo, px_hi, py_hi))
    }

    /// Build the per-composite buffers + bind group (shared by the readback paths). `lifted_frac_buf`
    /// (ADR-0084) is the live lift accumulator, or `None` for the dormant all-zero buffer (returned
    /// in the tuple so the caller keeps it alive alongside the bind group it backs). `paper_rgba`
    /// (ADR-0084 paper-reveal) is the session's original canvas content; the one-shot callers pass
    /// the backdrop (`mix(b, b, lf) = b` ⇒ exact no-op regardless of `lf`).
    #[allow(clippy::too_many_arguments)]
    fn build_buffers(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        scale: u32,
        coverage_k: f32,
        pigment_buf: &wgpu::Buffer,
        backdrop_rgba: &[u8],
        paper_rgba: &[u8],
        grid_region: (u32, u32, u32, u32),
        lifted_frac_buf: Option<&wgpu::Buffer>,
    ) -> (
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::Buffer,
        wgpu::BindGroup,
    ) {
        let canvas_bytes = (cw as usize * ch as usize * 4) as u64;
        let (px_lo, py_lo, px_hi, py_hi) = composite_canvas_region(grid_region, scale, cw, ch);
        let uni = GpuU {
            cw,
            ch,
            gw,
            gh,
            inv: 1.0 / scale as f32,
            coverage_k,
            ss: WET_COMPOSITE_SS,
            origin_x: px_lo,
            origin_y: py_lo,
            end_x: px_hi,
            end_y: py_hi,
            wet_sheen: 0,
        };
        let coeffs = GpuCoeffs::build();
        let params_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite params"),
            size: core::mem::size_of::<GpuU>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&params_buf, 0, bytemuck::bytes_of(&uni));
        let coeffs_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite coeffs"),
            size: core::mem::size_of::<GpuCoeffs>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&coeffs_buf, 0, bytemuck::bytes_of(&coeffs));
        let backdrop_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite backdrop"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&backdrop_buf, 0, backdrop_rgba);
        let paper_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite paper (ADR-0084)"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&paper_buf, 0, paper_rgba);
        let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("composite out"),
            size: canvas_bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        queue.write_buffer(&out_buf, 0, backdrop_rgba);
        // ADR-0084 — the live lift accumulator, or an owned all-zero dormant buffer (returned so it
        // outlives the bind group). A fresh STORAGE buffer reads as zero ⇒ `lf = 0` ⇒ no-op.
        let dormant_lifted =
            Self::make_dormant_field(device, gw, gh, "composite dormant lifted_frac");
        let lifted_res = lifted_frac_buf
            .unwrap_or(&dormant_lifted)
            .as_entire_binding();
        // Wet-sheen water (binding 7): the one-shot readback paths never dispatch the
        // preview-texture kernels (the only readers), so the all-zero dormant field is
        // re-bound here — legal (two read-only bindings of one buffer) and inert.
        let water_res = dormant_lifted.as_entire_binding();
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pigment_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: backdrop_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: out_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: coeffs_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lifted_res,
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: paper_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: water_res,
                },
            ],
        });
        (
            params_buf,
            coeffs_buf,
            backdrop_buf,
            paper_buf,
            dormant_lifted,
            out_buf,
            bind,
        )
    }
}
