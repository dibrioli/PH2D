//! T1.2–T1.4 — o pipeline wgpu que rasteriza um `FlipGpuData` numa textura alvo.
//!
//! Um passe simples: bind group com a câmera (uniform) + 3 storage buffers
//! (points/strokes/point_stroke), draw não-instanciado de `point_count * 6`
//! vértices. O vertex shader expande cada ponto num quad (fita); o fragment aplica
//! a máscara de hardness. Ver `shaders/flip.wgsl`.
//!
//! Os buffers vivem em `self` (não podem ser locais: o `encoder` é submetido pelo
//! chamador DEPOIS do `render`). v1 reescreve os buffers a cada chamada — o
//! rebind barato / dirty-flag é o T1.8.

use crate::pack::FlipGpuData;

/// A câmera do passe, no layout do uniform WGSL (80 bytes, align 16).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraRaw {
    /// Mundo → clip (NDC). Para o 2D-ortográfico, `clip = M · (x, y, 0, 1)`.
    pub world_to_clip: [[f32; 4]; 4],
    /// Tamanho do alvo em pixels (screen-space da fita).
    pub viewport: [f32; 2],
    /// Pixels por unidade de mundo (`thickness_px = raio · px_per_world`).
    pub px_per_world: f32,
    pub _pad: f32,
}

impl CameraRaw {
    /// Constrói a câmera a partir do afim mundo→clip, do viewport e do zoom.
    #[must_use]
    pub fn new(world_to_clip: [[f32; 4]; 4], viewport: [f32; 2], px_per_world: f32) -> Self {
        Self {
            world_to_clip,
            viewport,
            px_per_world,
            _pad: 0.0,
        }
    }
}

/// O rasterizador de traço do Flip. Um por (device, formato de alvo).
pub struct FlipRenderer {
    pipeline: wgpu::RenderPipeline,
    bgl: wgpu::BindGroupLayout,
    camera_buf: wgpu::Buffer,
    // Storage buffers + bind group, reconstruídos ao render (v1).
    points_buf: Option<wgpu::Buffer>,
    strokes_buf: Option<wgpu::Buffer>,
    point_stroke_buf: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
    /// Nº de vértices a desenhar no próximo/último `render` (= point_count · 6).
    vertex_count: u32,
}

impl FlipRenderer {
    /// Cria o pipeline para o formato de alvo dado. Blend = premultiplicado over
    /// (o fragment emite cor premult).
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-flip bgl"),
            entries: &[
                // camera (uniform)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                storage_entry(1),
                storage_entry(2),
                storage_entry(3),
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-flip layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-flip shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/flip.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-flip pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState {
                        // Premultiplicado over: src já vem multiplicado por alpha.
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // fitas podem virar CW/CCW conforme a curva
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-flip camera"),
            size: std::mem::size_of::<CameraRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bgl,
            camera_buf,
            points_buf: None,
            strokes_buf: None,
            point_stroke_buf: None,
            bind_group: None,
            vertex_count: 0,
        }
    }

    /// Sobe `data` + `camera` pra GPU e reconstrói o bind group. Chame antes de
    /// [`Self::draw`] (ambos com o mesmo `encoder`/frame).
    pub fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &CameraRaw,
        data: &FlipGpuData,
    ) {
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(camera));
        if data.is_empty() {
            self.vertex_count = 0;
            self.bind_group = None;
            return;
        }
        let points = storage_buffer(
            device,
            queue,
            "ph2d-flip points",
            bytemuck::cast_slice(&data.points),
        );
        let strokes = storage_buffer(
            device,
            queue,
            "ph2d-flip strokes",
            bytemuck::cast_slice(&data.strokes),
        );
        let point_stroke = storage_buffer(
            device,
            queue,
            "ph2d-flip point_stroke",
            bytemuck::cast_slice(&data.point_stroke),
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-flip bind group"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: points.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: strokes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: point_stroke.as_entire_binding(),
                },
            ],
        });

        self.points_buf = Some(points);
        self.strokes_buf = Some(strokes);
        self.point_stroke_buf = Some(point_stroke);
        self.bind_group = Some(bind_group);
        self.vertex_count = data.point_count() as u32 * 6;
    }

    /// Codifica o draw do traço numa render pass já aberta no alvo (o chamador
    /// controla load/clear). No-op se o último [`Self::upload`] não tinha geometria.
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        let Some(bg) = self.bind_group.as_ref() else {
            return;
        };
        if self.vertex_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.draw(0..self.vertex_count, 0..1);
    }
}

/// Uma entrada de bind-group-layout para um storage buffer read-only no vertex.
fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// Cria um storage buffer e escreve os bytes (v1: recriado a cada upload).
fn storage_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    bytes: &[u8],
) -> wgpu::Buffer {
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buf, 0, bytes);
    buf
}
