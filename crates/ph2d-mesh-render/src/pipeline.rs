//! O passe wgpu que põe a malha na tela.
//!
//! ⚠️ **Dois vertex buffers, não um interleaved — e é a decisão que paga.** A
//! `Mesh` guarda posição e normal em vetores SEPARADOS (SoA), e intercalar para
//! subir custaria uma cópia da malha inteira a cada upload. Com um buffer por
//! atributo, o upload é `write_buffer` direto sobre o slice que já existe, e o
//! caminho que a W2 vai querer — *um dab mexeu em posições, a cor não mudou* —
//! sobe **só o que mudou** em vez de reintercalar tudo.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use ph2d_mesh::Mesh;
use wgpu::util::DeviceExt as _;

use crate::camera::Camera3d;

/// O uniform da câmera. `mat4x4` alinha em 16 B, então não há padding a declarar.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraRaw {
    view_proj: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
}

/// Os buffers da malha no device.
struct MeshGpu {
    positions: wgpu::Buffer,
    normals: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
    vert_capacity: usize,
    index_capacity: usize,
}

/// O renderizador da malha.
pub struct MeshRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    depth: Option<wgpu::TextureView>,
    depth_size: (u32, u32),
    mesh: Option<MeshGpu>,
    scratch_indices: Vec<[u32; 3]>,
}

impl MeshRenderer {
    /// O formato do depth-buffer da cena 3D.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    #[must_use]
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh camera"),
            size: size_of::<CameraRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh bind"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh layout"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
        });

        // Os atributos têm de viver tanto quanto o descritor, então são `const`
        // e não temporários de uma closure — um slice emprestado de dentro de um
        // construtor morre antes de o pipeline ser criado.
        const fn vec3_attr(location: u32) -> [wgpu::VertexAttribute; 1] {
            [wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: location,
            }]
        }
        const POS: [wgpu::VertexAttribute; 1] = vec3_attr(0);
        const NRM: [wgpu::VertexAttribute; 1] = vec3_attr(1);
        let vec3_buffer = |attrs: &'static [wgpu::VertexAttribute]| wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: attrs,
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[vec3_buffer(&POS), vec3_buffer(&NRM)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    // Opaco: a escultura é sólida, e um blend aqui só serviria
                    // para esconder um erro de profundidade atrás de uma mistura.
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // ⚠️ **Sem culling, de propósito.** Uma escultura em progresso é
                // frequentemente uma casca aberta, e um OBJ de terceiro chega com
                // winding misto; descartar a face de trás transformaria isso num
                // buraco, que é indistinguível de geometria faltando. O shader
                // vira a normal para o olho, então o verso acende como frente.
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                // `Less` com limpeza em 1.0: profundidade 3D comum. (O Flip usa
                // `Greater` porque a ordem dele é 2D por-traço, outra pergunta.)
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            uniform,
            bind,
            depth: None,
            depth_size: (0, 0),
            mesh: None,
            scratch_indices: Vec::new(),
        }
    }

    /// Sobe a malha. Reusa os buffers quando cabem — um dab que só move
    /// vértices não realoca nada.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, mesh: &Mesh) {
        mesh.triangle_indices(&mut self.scratch_indices);
        let verts = mesh.vert_count();
        let idx = self.scratch_indices.len();
        let index_count = (idx * 3) as u32;

        let fits = self
            .mesh
            .as_ref()
            .is_some_and(|g| g.vert_capacity >= verts && g.index_capacity >= idx);

        if fits {
            let g = self.mesh.as_mut().expect("acabou de ser conferido");
            queue.write_buffer(&g.positions, 0, bytemuck::cast_slice(mesh.positions()));
            queue.write_buffer(&g.normals, 0, bytemuck::cast_slice(mesh.normals()));
            queue.write_buffer(&g.indices, 0, bytemuck::cast_slice(&self.scratch_indices));
            g.index_count = index_count;
            return;
        }

        let vb = |label, data: &[u8]| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: data,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        };
        self.mesh = Some(MeshGpu {
            positions: vb("ph2d-mesh pos", bytemuck::cast_slice(mesh.positions())),
            normals: vb("ph2d-mesh nrm", bytemuck::cast_slice(mesh.normals())),
            indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("ph2d-mesh idx"),
                contents: bytemuck::cast_slice(&self.scratch_indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }),
            index_count,
            vert_capacity: verts,
            index_capacity: idx,
        });
    }

    /// Há geometria para desenhar?
    #[must_use]
    pub fn has_mesh(&self) -> bool {
        self.mesh.as_ref().is_some_and(|g| g.index_count > 0)
    }

    /// Garante o depth-buffer do tamanho pedido (recria se mudou).
    pub fn ensure_depth(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.depth.is_some() && self.depth_size == size {
            return;
        }
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh depth"),
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
        self.depth = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
        self.depth_size = size;
    }

    /// Desenha a malha em `color_view`, PRESERVANDO o que já está lá
    /// (`LoadOp::Load`) — a cena 2D fica por baixo. No-op sem geometria.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        camera: &Camera3d,
        size: (u32, u32),
    ) {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);

        let aspect = size.0 as f32 / size.1 as f32;
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&CameraRaw {
                view_proj: camera.view_proj(aspect).to_cols_array_2d(),
                view: camera.view().to_cols_array_2d(),
            }),
        );

        let g = self.mesh.as_ref().expect("has_mesh acabou de confirmar");
        let depth = self.depth.as_ref().expect("ensure_depth acabou de rodar");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-mesh pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        pass.set_vertex_buffer(0, g.positions.slice(..));
        pass.set_vertex_buffer(1, g.normals.slice(..));
        pass.set_index_buffer(g.indices.slice(..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..g.index_count, 0, 0..1);
    }
}

/// A matriz que o uniform carrega, exposta para o gate poder afirmar o que sobe
/// ao device sem abrir uma render pass.
#[must_use]
pub fn camera_uniform_bytes(camera: &Camera3d, aspect: f32) -> [u8; 128] {
    let raw = CameraRaw {
        view_proj: camera.view_proj(aspect).to_cols_array_2d(),
        view: camera.view().to_cols_array_2d(),
    };
    let mut out = [0u8; 128];
    out.copy_from_slice(bytemuck::bytes_of(&raw));
    out
}

/// A matriz de vista-projeção que o shader recebe, reconstruída dos bytes.
/// Existe para o gate provar que a coluna-major do `glam` é a coluna-major que
/// o WGSL espera — trocar isso transpõe a cena e nada avisa.
#[must_use]
pub fn view_proj_from_bytes(bytes: &[u8; 128]) -> Mat4 {
    let mut cols = [0f32; 16];
    for (i, c) in cols.iter_mut().enumerate() {
        *c = f32::from_ne_bytes([
            bytes[i * 4],
            bytes[i * 4 + 1],
            bytes[i * 4 + 2],
            bytes[i * 4 + 3],
        ]);
    }
    Mat4::from_cols_array(&cols)
}
