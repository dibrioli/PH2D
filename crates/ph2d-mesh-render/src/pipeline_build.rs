//! **COMO o passe é MONTADO** — layouts, atributos de vértice e os dois
//! pipelines.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados do
//! [`MeshRenderer`]; o corte é *como este renderizador é construído* (aqui)
//! contra *o que ele FAZ com a malha* (lá). São assuntos diferentes e crescem
//! por motivos diferentes: este arquivo cresce quando o pipeline ganha um
//! estado, o outro quando a cena ganha um objeto.

use super::{CameraRaw, MESH_WGSL, MeshRenderer};
use crate::lighting::RigRaw;
use crate::shade::ShadeRaw;

impl MeshRenderer {
    #[must_use]
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh bgl"),
            entries: &[
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
                // O RIG. Buffer separado do da câmera de propósito: são duas
                // frequências (a câmera muda a cada arrasto, o rig quando o
                // artista abre o card) e, sobretudo, o gate do layout coluna-major
                // da câmera continua olhando exatamente os mesmos 128 bytes que
                // olhava antes desta wave.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // **AS OPÇÕES DE SOMBREAMENTO** (hoje: a cavidade). Terceira
                // entrada do grupo 0 e não um campo apendado ao rig, apesar de o
                // `RigRaw` ter padding sobrando: uma cavidade não é uma lâmpada, e
                // aquele struct é o espelho do `Lamp` do passe de luz da tinta —
                // enfiar um knob de barro nele faria a próxima wave que sincronizar
                // os dois herdar um campo que o outro lado não tem.
                //
                // A FREQUÊNCIA é a que justifica o grupo: câmera, rig e opções são
                // todos da CENA (uma escrita por frame). O `Object` é o grupo 1
                // porque ele é por-desenho.
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh camera"),
            size: size_of::<CameraRaw>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rig_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh rig"),
            size: RigRaw::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shade_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh shade"),
            size: ShadeRaw::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh bind"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rig_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: shade_uniform.as_entire_binding(),
                },
            ],
        });

        let obj_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh object bgl"),
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

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh layout"),
            bind_group_layouts: &[&bgl, &obj_bgl],
            immediate_size: 0,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh shader"),
            source: wgpu::ShaderSource::Wgsl(MESH_WGSL.into()),
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
        const fn f32_attr(location: u32) -> [wgpu::VertexAttribute; 1] {
            [wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 0,
                shader_location: location,
            }]
        }
        const MASK: [wgpu::VertexAttribute; 1] = f32_attr(2);
        /// A CURVATURA por vértice (`ph2d_mesh::curvature`) — buffer próprio, e
        /// não um segundo canal empacotado com a máscara, porque as duas mudam em
        /// momentos diferentes: a máscara quando o artista a pinta, a curvatura
        /// em TODO dab. Juntá-las faria um upload incremental de forma reenviar
        /// a autoria que ninguém tocou.
        const CURV: [wgpu::VertexAttribute; 1] = f32_attr(3);
        // Irmão do `vec3_buffer`, e uma CLOSURE pela mesma razão que ele: o
        // `make` abaixo é chamado duas vezes (a cena e o G-buffer), e um valor
        // capturado por move faria dele um `FnOnce`.
        let f32_buffer = |attrs: &'static [wgpu::VertexAttribute]| wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: attrs,
        };
        let vec3_buffer = |attrs: &'static [wgpu::VertexAttribute]| wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: attrs,
        };

        // **O QUE VARIA entre os pipelines** — e só isto.
        //
        // ⚠️ Um struct de variação e não três descritores: o que os pipelines têm
        // em COMUM é o que precisa de proteção (o layout, os quatro buffers de
        // vértice, `cull_mode: None`, o formato do depth), e uma segunda cópia do
        // descritor seria uma segunda resposta a *"como esta malha é
        // rasterizada"* — o dia em que alguém trocasse o culling num e não no
        // outro, o G-buffer passaria a descrever uma silhueta que a tela não
        // mostra. Os campos abaixo são exatamente os que os três **têm** de
        // decidir por conta.
        struct Variant<'a> {
            label: &'a str,
            entry: &'a str,
            format: wgpu::TextureFormat,
            topology: wgpu::PrimitiveTopology,
            blend: Option<wgpu::BlendState>,
            bias: wgpu::DepthBiasState,
        }
        let make = |v: Variant<'_>| {
            let Variant {
                label,
                entry,
                format,
                topology,
                blend,
                bias,
            } = v;
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[
                        vec3_buffer(&POS),
                        vec3_buffer(&NRM),
                        f32_buffer(&MASK),
                        f32_buffer(&CURV),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        // Opaco nos passes de FORMA: a escultura é sólida, e um
                        // blend ali só serviria para esconder um erro de
                        // profundidade atrás de uma mistura. O passe de ARESTAS é
                        // o oposto — ele anota a forma sem a apagar.
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
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
                    // ⚠️ **As arestas ESCREVEM profundidade como as faces.** Não
                    // escrever deixaria a malha do outro lado da peça atravessar
                    // a superfície da frente, e o wireframe viraria um emaranhado
                    // que não diz de que lado está o quê.
                    depth_write_enabled: true,
                    // `Less` com limpeza em 1.0: profundidade 3D comum. (O Flip usa
                    // `Greater` porque a ordem dele é 2D por-traço, outra pergunta.)
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias,
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            })
        };
        const SOLID: wgpu::DepthBiasState = wgpu::DepthBiasState {
            constant: 0,
            slope_scale: 0.0,
            clamp: 0.0,
        };
        // ⚠️ **O viés NEGATIVO é o que faz a aresta ganhar da face.** Uma linha
        // desenhada sobre o triângulo que a contém é COPLANAR com ele, e uma
        // comparação de profundidade entre dois números que a rasterização
        // computou por caminhos diferentes é uma moeda: o resultado é o
        // z-fighting clássico, a malha piscando em faixas. Puxar a linha para
        // perto do olho (`Less` com profundidade 0 = perto ⇒ viés negativo) a
        // tira do empate. O termo de INCLINAÇÃO é o que cobre a face quase de
        // perfil, onde a profundidade varia muito dentro de um pixel e um viés
        // constante não alcança.
        const WIRE: wgpu::DepthBiasState = wgpu::DepthBiasState {
            constant: -4,
            slope_scale: -1.0,
            clamp: 0.0,
        };
        let pipeline = make(Variant {
            label: "ph2d-mesh pipeline",
            entry: "fs_main",
            format: target_format,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: None,
            bias: SOLID,
        });
        let gbuffer_pipeline = make(Variant {
            label: "ph2d-mesh gbuffer",
            entry: "fs_gbuffer",
            format: Self::GBUFFER_FORMAT,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: None,
            bias: SOLID,
        });
        let wire_pipeline = make(Variant {
            label: "ph2d-mesh wire",
            entry: "fs_wire",
            format: target_format,
            topology: wgpu::PrimitiveTopology::LineList,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            bias: WIRE,
        });

        Self {
            pipeline,
            gbuffer_pipeline,
            wire_pipeline,
            uniform,
            rig_uniform,
            shade_uniform,
            bind,
            obj_bgl,
            depth: None,
            depth_size: (0, 0),
            slots: Vec::new(),
            poses: Vec::new(),
            scratch_indices: Vec::new(),
            scratch_indices_flat: Vec::new(),
            scratch_moved: Vec::new(),
            scratch_runs: Vec::new(),
            scratch_masks: Vec::new(),
        }
    }
}
