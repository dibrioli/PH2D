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

use crate::fill::FillVertex;
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
    // Fill (T1.6): pipeline de triângulos flat + vertex buffer + contagem.
    fill_pipeline: wgpu::RenderPipeline,
    fill_buf: Option<wgpu::Buffer>,
    fill_count: u32,
    // Depth-buffer para a ordem 2D (GP §2): profundidade por-traço + teste
    // GREATER. Redimensionado sob demanda pra casar o alvo.
    depth_texture: Option<wgpu::Texture>,
    depth_view: Option<wgpu::TextureView>,
    depth_size: (u32, u32),
}

impl FlipRenderer {
    /// Formato do depth-buffer da ordem 2D.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
}

impl FlipRenderer {
    /// Cria o pipeline para o formato de alvo dado. Blend = premultiplicado over
    /// (o fragment emite cor premult).
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-flip bgl"),
            entries: &[
                // camera (uniform) — VERTEX (posições/espessura) + FRAGMENT (o
                // viewport.y para o flip-Y da cobertura analítica em screen-space).
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                    blend: Some(premult_over()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: tri_list(),
            depth_stencil: Some(depth_greater(Self::DEPTH_FORMAT)),
            multisample: no_msaa(),
            multiview_mask: None,
            cache: None,
        });

        // Fill: triângulos flat (vertex buffer FillVertex), mesma câmera (bgl),
        // mesmo depth (profundidade menor → o traço fica por cima).
        let fill_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-flip fill shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/flip_fill.wgsl").into()),
        });
        let fill_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-flip fill pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &fill_shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<FillVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    // pos @0 (offset 0), depth @1 (offset 8), color @2 (offset 16);
                    // o `_pad` (offset 12) é só alinhamento.
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 8,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fill_shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(premult_over()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: tri_list(),
            depth_stencil: Some(depth_greater(Self::DEPTH_FORMAT)),
            multisample: no_msaa(),
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
            fill_pipeline,
            fill_buf: None,
            fill_count: 0,
            depth_texture: None,
            depth_view: None,
            depth_size: (0, 0),
        }
    }

    /// Garante um depth-buffer do tamanho `size` (recria se mudou). Chame antes de
    /// abrir a render pass — a view é o `depth_stencil_attachment` dela.
    pub fn ensure_depth(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.depth_view.is_some() && self.depth_size == size {
            return;
        }
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-flip depth"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth_view = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
        self.depth_texture = Some(tex);
        self.depth_size = size;
    }

    /// A view do depth-buffer atual (após [`Self::ensure_depth`]). `None` antes.
    #[must_use]
    pub fn depth_view(&self) -> Option<&wgpu::TextureView> {
        self.depth_view.as_ref()
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
            self.fill_count = 0;
            self.bind_group = None;
            return;
        }
        // Fill (T1.6): vertex buffer de triângulos flat, se houver.
        if data.fills.is_empty() {
            self.fill_count = 0;
        } else {
            self.fill_buf = Some(vertex_buffer(
                device,
                queue,
                "ph2d-flip fills",
                bytemuck::cast_slice(&data.fills),
            ));
            self.fill_count = data.fills.len() as u32;
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

    /// Codifica os draws (fill ABAIXO, traço por cima) numa render pass já aberta
    /// no alvo (color + o depth desta struct). No-op se o último [`Self::upload`]
    /// não tinha geometria. A ordem 2D vem do depth, mas emitir o fill antes é o
    /// convencional (fill-first).
    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        let Some(bg) = self.bind_group.as_ref() else {
            return;
        };
        pass.set_bind_group(0, bg, &[]);
        // Fill primeiro.
        if let Some(fb) = self.fill_buf.as_ref()
            && self.fill_count > 0
        {
            pass.set_pipeline(&self.fill_pipeline);
            pass.set_vertex_buffer(0, fb.slice(..));
            pass.draw(0..self.fill_count, 0..1);
        }
        // Traço por cima (o bind group segue válido: mesma bgl nos dois pipelines).
        if self.vertex_count > 0 {
            pass.set_pipeline(&self.pipeline);
            pass.draw(0..self.vertex_count, 0..1);
        }
    }
}

/// Blend premultiplicado over (src já vem multiplicado por alpha).
pub(crate) fn premult_over() -> wgpu::BlendState {
    let over = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    };
    wgpu::BlendState {
        color: over,
        alpha: over,
    }
}

/// Depth-state da ordem 2D: escreve + teste GREATER **estrito** — o estado EXATO do
/// GP 2D (`gpencil_cache_utils.cc:449`: `WRITE_DEPTH | BLEND_ALPHA_PREMUL |
/// DEPTH_GREATER`, depth por-STROKE crescente com o sid). Entre traços/fills o sid
/// maior tem depth estritamente maior e ganha; no MESMO depth (o traço sobre si
/// mesmo: quina quebrada, junção, auto-cruzamento) a 2ª face é **descartada, não
/// misturada** → o premult-over nunca acumula ("the stroke cannot overlap itself",
/// `gpencil_vert.glsl` — o default do GP; o modo per-ponto `GP_STROKE_OVERLAP` que
/// deixa acumular é opção de material, não portada). O par obrigatório disto é o
/// `discard` de alpha < 0.001 no fragment: sem ele um fragmento transparente
/// escreveria depth e furaria a geometria sobreposta (o "escamado" histórico).
fn depth_greater(format: wgpu::TextureFormat) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

pub(crate) fn tri_list() -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: None,
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
    }
}

pub(crate) fn no_msaa() -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}

/// Cria um vertex buffer e escreve os bytes (v1: recriado a cada upload).
fn vertex_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    bytes: &[u8],
) -> wgpu::Buffer {
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: bytes.len() as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&buf, 0, bytes);
    buf
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
