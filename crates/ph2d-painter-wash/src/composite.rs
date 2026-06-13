//! Canvas-resolution compositor for the minimal watercolor core (ADR-0086 / ADR-0087 §6).
//! Reads the solver's pigment field, glazes it subtractively (Beer–Lambert) over the canvas
//! backdrop, and writes a premultiplied `Rgba8Unorm` preview texture the sprite renderer
//! samples directly (same handoff as the v2 `FluidCompositor`). Mirror of the v2 contract so
//! integration is a drop-in: `begin_stroke` → `encode_composite` → `preview_texture`.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CParams {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    region_ox: u32,
    region_oy: u32,
    region_w: u32,
    region_h: u32,
    inv: f32,
    coverage_k: f32,
    /// 0 = RGB Beer–Lambert (default), 1 = spectral Kubelka–Munk (the optional pigment-mixing mode).
    color_model: u32,
    _p1: f32,
}

/// Packed K–M spectral tables for the composite (mirrors [`crate::km::KmModel`]): the layout the WGSL
/// `km_*` accessors index — `[curves 3·N][absorb PIGMENTS·N][to_rgb 3·N][rgb_absorb PIGMENTS·3]`.
fn pack_km() -> Vec<f32> {
    use crate::km::{KmModel, N, PIGMENTS};
    let km = KmModel::new();
    let curves = km.upsample_basis();
    let absorb = km.pigment_absorbance();
    let to_rgb = km.to_rgb_matrix();
    let rgb_absorb = km.pigment_rgb_absorbance();
    let mut v = Vec::with_capacity(3 * N + PIGMENTS * N + 3 * N + PIGMENTS * 3);
    for row in curves {
        v.extend_from_slice(row);
    }
    for row in absorb {
        v.extend_from_slice(row);
    }
    for row in &to_rgb {
        v.extend_from_slice(row);
    }
    for row in &rgb_absorb {
        v.extend_from_slice(row);
    }
    v
}

const WG: u32 = 8;
fn groups(n: u32) -> u32 {
    n.div_ceil(WG)
}

/// Subtractive canvas compositor → premultiplied preview texture.
pub struct WashCompositor {
    pipe: wgpu::ComputePipeline,
    bgl: wgpu::BindGroupLayout,
    cparams_buf: wgpu::Buffer,
    /// Static spectral K–M tables (binding 4) — built once from [`crate::km::KmModel`].
    km_buf: wgpu::Buffer,
    stroke: Option<Stroke>,
}

struct Stroke {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    coverage_k: f32,
    color_model: u32,
    tex: wgpu::Texture,
    /// Owned only to keep the GPU buffer alive for `bg` (binding 2); never read directly.
    #[allow(dead_code)]
    backdrop: wgpu::Buffer,
    bg: wgpu::BindGroup,
}

impl WashCompositor {
    /// Build the (stroke-independent) pipeline. Call [`WashCompositor::begin_stroke`] to bind a
    /// field + backdrop and allocate the preview texture.
    pub fn new(device: &wgpu::Device) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("wash composite"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader/composite.wgsl").into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("wash composite bgl"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("wash composite layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
        let pipe = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("wash composite pipeline"),
            layout: Some(&layout),
            module: &module,
            entry_point: Some("cs_composite"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let cparams_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash cparams"),
            size: core::mem::size_of::<CParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let km = pack_km();
        let km_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash km tables"),
            size: (km.len() * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: true,
        });
        km_buf.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(&km));
        km_buf.unmap();
        Self { pipe, bgl, cparams_buf, km_buf, stroke: None }
    }

    /// Bind a field (`pig_buf` from [`crate::WashSolver::pig_buffer`]) + canvas `backdrop`
    /// (packed sRGB8 RGBA, `cw·ch` words) and allocate the preview texture. `gw/gh` = grid dims,
    /// `cw/ch` = canvas dims (grid==canvas ⇒ no magnification).
    #[allow(clippy::too_many_arguments)]
    pub fn begin_stroke(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        gw: u32,
        gh: u32,
        cw: u32,
        ch: u32,
        backdrop: &[u32],
        coverage_k: f32,
        color_model: u32,
        pig_buf: &wgpu::Buffer,
    ) {
        assert_eq!(backdrop.len() as u32, cw * ch, "backdrop must be cw·ch words");
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("wash preview_tex"),
            size: wgpu::Extent3d { width: cw, height: ch, depth_or_array_layers: 1 },
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
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        let backdrop_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash backdrop"),
            size: u64::from(cw) * u64::from(ch) * 4,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&backdrop_buf, 0, bytemuck::cast_slice(backdrop));
        let inv = gw as f32 / cw as f32;
        let cp = CParams {
            cw, ch, gw, gh, region_ox: 0, region_oy: 0, region_w: cw, region_h: ch,
            inv, coverage_k, color_model, _p1: 0.0,
        };
        queue.write_buffer(&self.cparams_buf, 0, bytemuck::bytes_of(&cp));
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("wash composite bg"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.cparams_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: pig_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: backdrop_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 4, resource: self.km_buf.as_entire_binding() },
            ],
        });
        self.stroke = Some(Stroke { cw, ch, gw, gh, coverage_k, color_model, tex, backdrop: backdrop_buf, bg });
    }

    /// Append the composite of `region` (canvas-space `(ox,oy,w,h)`) to `enc`.
    pub fn encode_composite(&self, queue: &wgpu::Queue, enc: &mut wgpu::CommandEncoder, region: (u32, u32, u32, u32)) {
        let Some(s) = &self.stroke else { return };
        let rw = region.2.clamp(1, s.cw);
        let rh = region.3.clamp(1, s.ch);
        let cp = CParams {
            cw: s.cw, ch: s.ch, gw: s.gw, gh: s.gh,
            region_ox: region.0.min(s.cw - 1),
            region_oy: region.1.min(s.ch - 1),
            region_w: rw, region_h: rh,
            inv: s.gw as f32 / s.cw as f32,
            coverage_k: s.coverage_k, color_model: s.color_model, _p1: 0.0,
        };
        queue.write_buffer(&self.cparams_buf, 0, bytemuck::bytes_of(&cp));
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash composite"), timestamp_writes: None });
        pass.set_pipeline(&self.pipe);
        pass.set_bind_group(0, &s.bg, &[]);
        pass.dispatch_workgroups(groups(rw), groups(rh), 1);
    }

    /// Flip the colour model on the LIVE stroke without re-seeding — the field is encoding-agnostic
    /// (always pigment concentrations), so a model change is a pure re-render (live transform).
    pub fn set_color_model(&mut self, color_model: u32) {
        if let Some(s) = self.stroke.as_mut() {
            s.color_model = color_model;
        }
    }

    /// The preview texture the sprite renderer samples (premultiplied `Rgba8Unorm`, canvas-res).
    #[must_use]
    pub fn preview_texture(&self) -> Option<&wgpu::Texture> {
        self.stroke.as_ref().map(|s| &s.tex)
    }

    /// Read the preview texture back as packed RGBA8 (`cw·ch` words) — the one-shot finalize bake
    /// (and the parity tests). NOT a per-frame path (the live preview is the zero-readback slot copy).
    pub fn read_preview(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Option<Vec<u32>> {
        let s = self.stroke.as_ref()?;
        let bpr_unpadded = s.cw * 4;
        let bpr = bpr_unpadded.div_ceil(256) * 256; // wgpu copy alignment
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash preview readback"),
            size: u64::from(bpr) * u64::from(s.ch),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash preview readback enc") });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &s.tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(s.ch) },
            },
            wgpu::Extent3d { width: s.cw, height: s.ch, depth_or_array_layers: 1 },
        );
        queue.submit([enc.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = staging.slice(..).get_mapped_range();
        let mut out = Vec::with_capacity((s.cw * s.ch) as usize);
        for row in 0..s.ch {
            let base = (row * bpr) as usize;
            let words: &[u32] = bytemuck::cast_slice(&data[base..base + bpr_unpadded as usize]);
            out.extend_from_slice(words);
        }
        drop(data);
        staging.unmap();
        Some(out)
    }
}
