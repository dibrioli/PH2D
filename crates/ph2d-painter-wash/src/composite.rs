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
    _p0: f32,
    _p1: f32,
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
    stroke: Option<Stroke>,
}

struct Stroke {
    cw: u32,
    ch: u32,
    gw: u32,
    gh: u32,
    coverage_k: f32,
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
        Self { pipe, bgl, cparams_buf, stroke: None }
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
            inv, coverage_k, _p0: 0.0, _p1: 0.0,
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
            ],
        });
        self.stroke = Some(Stroke { cw, ch, gw, gh, coverage_k, tex, backdrop: backdrop_buf, bg });
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
            coverage_k: s.coverage_k, _p0: 0.0, _p1: 0.0,
        };
        queue.write_buffer(&self.cparams_buf, 0, bytemuck::bytes_of(&cp));
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("wash composite"), timestamp_writes: None });
        pass.set_pipeline(&self.pipe);
        pass.set_bind_group(0, &s.bg, &[]);
        pass.dispatch_workgroups(groups(rw), groups(rh), 1);
    }

    /// The preview texture the sprite renderer samples (premultiplied `Rgba8Unorm`, canvas-res).
    #[must_use]
    pub fn preview_texture(&self) -> Option<&wgpu::Texture> {
        self.stroke.as_ref().map(|s| &s.tex)
    }

    /// Read a FULL-WIDTH row band `[y0, y1)` of the preview texture as packed RGBA8 (the
    /// `(y1-y0)·cw` words `fluid_apply_gpu_composite_rows` expects). The live bake reads only the
    /// wet band each frame (O(stroke), not O(canvas)) — the perf fix vs `read_preview`.
    pub fn read_preview_band(&self, device: &wgpu::Device, queue: &wgpu::Queue, y0: u32, y1: u32) -> Option<Vec<u32>> {
        let s = self.stroke.as_ref()?;
        let y0 = y0.min(s.ch);
        let y1 = y1.clamp(y0, s.ch);
        let rows = y1 - y0;
        if rows == 0 {
            return Some(Vec::new());
        }
        let bpr = (s.cw * 4).div_ceil(256) * 256;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("wash preview band readback"),
            size: u64::from(bpr) * u64::from(rows),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("wash band enc") });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &s.tex,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: y0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(rows) },
            },
            wgpu::Extent3d { width: s.cw, height: rows, depth_or_array_layers: 1 },
        );
        queue.submit([enc.finish()]);
        staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        let data = staging.slice(..).get_mapped_range();
        let mut out = Vec::with_capacity((s.cw * rows) as usize);
        for row in 0..rows {
            let base = (row * bpr) as usize;
            out.extend_from_slice(bytemuck::cast_slice(&data[base..base + (s.cw * 4) as usize]));
        }
        drop(data);
        staging.unmap();
        Some(out)
    }

    /// Read the preview texture back as packed RGBA8 (`cw·ch` words) — test / parity path.
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
