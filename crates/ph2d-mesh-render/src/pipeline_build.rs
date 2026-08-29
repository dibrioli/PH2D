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
                // ⚠️ **O FRAGMENT também**, e é o `wire_cull` que o exige: quem
                // decide se uma linha de costas some é o fragmento (o vértice não
                // pode descartar um primitivo), então a pergunta *"esta peça é um
                // sólido?"* tem de estar visível de lá.
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // **A VISIBILIDADE DE TELA** (`crate::ssao`) — grupo 2 do passe de cor.
        //
        // ⚠️ Grupo próprio e não uma quarta entrada do grupo 0 pela razão de
        // sempre, a FREQUÊNCIA: o grupo 0 é um bind por frame com três uniforms
        // estáveis, e isto é uma TEXTURA que é recriada a cada resize. Juntá-los
        // faria toda mudança de tamanho de janela reconstruir o bind da câmera.
        //
        // ⚠️ E sem SAMPLER, de propósito: o barro lê `textureLoad` com as
        // coordenadas inteiras do próprio fragmento — a correspondência é 1:1 com
        // a tela, então filtrar seria interpolar uma medição consigo mesma.
        let ao_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh ao bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });

        // ---- GRUPO 3: as TABELAS ESTÁVEIS (hoje, a LUT do SSS) ----
        //
        // ⚠️ **Grupo próprio, e o critério é o mesmo do grupo 2: a FREQUÊNCIA.**
        // O 0 são uniforms por frame, o 1 é por objeto, o 2 é uma textura
        // recriada a cada resize — e isto é uma tabela assada **uma vez na vida
        // do processo**. Pendurá-la no 2 faria toda mudança de tamanho de janela
        // reconstruir o bind de uma textura que não mudou.
        //
        // ⚠️ E **COM sampler**, ao contrário do AO: aqui a textura é uma FUNÇÃO
        // tabelada e a consulta cai entre os nós, então a interpolação é o que
        // torna 128 linhas suficientes (gate `the_table_is_fine_enough`). No AO
        // a correspondência é 1:1 com a tela e filtrar seria interpolar uma
        // medição consigo mesma.
        let sss_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh sss bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // A IMAGEM DO MATCAP — no grupo do SSS porque os quatro grupos
                // que o wgpu garante já estavam ocupados, e porque o sampler
                // deste grupo (linear + `ClampToEdge`) é exatamente o que ela
                // quer. Ver o doc do binding no `mesh.wgsl`.
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        // A tabela nasce VAZIA e é preenchida no primeiro `render`.
        //
        // ⚠️ **Porque o `new` não tem `queue`** — a mesma restrição que fez o
        // canal de AO guardar oclusão em vez de visibilidade. Lá a inversão
        // resolveu de graça; aqui não há inversão que produza uma tabela de
        // Penner, então a saída é o `ensure_sss_lut`, irmão exato do
        // `ensure_depth`/`ensure_ssao`. Vinte chamadores do `new` continuam sem
        // mudar de assinatura.
        //
        // ⚠️ E enquanto ela está zerada o barro renderiza IGUAL, porque o default
        // de `sss_strength` é **0**: o `mix` nem consulta a tabela.
        let sss_lut = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh sss lut"),
            size: wgpu::Extent3d {
                width: crate::sss::LUT_SIZE,
                height: crate::sss::LUT_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // ⚠️ **`ClampToEdge` nos dois eixos, e ele é load-bearing:** o eixo `t`
        // SATURA no teto da tabela (`sss::T_MAX`), e é o clamp que transforma
        // *"pediu mais espalhamento do que a tabela representa"* em *"o controle
        // parou de responder"* em vez de num wrap para o outro extremo — que
        // desenharia uma superfície muito curva como se fosse plana.
        let sss_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("ph2d-mesh sss sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        // A imagem do matcap nasce VAZIA, e o `ensure_matcap` a preenche no
        // primeiro `render` — o irmão exato do `ensure_sss_lut`, e pela mesma
        // restrição (o `new` não tem `queue`).
        //
        // ⚠️ **`Rgba8UnormSrgb`, e o `Srgb` é o modelo inteiro:** os PNGs guardam
        // sRGB e o shader quer LINEAR; este formato faz o hardware desfazer a
        // transferência na leitura, de graça e com a curva certa (a com joelho,
        // não um `x^2.2`). Trocar por `Rgba8Unorm` deixaria toda escultura
        // clara demais, sem erro nenhum.
        let matcap_tex = crate::pipeline::matcap_texture(device, crate::matcap::MATCAPS[0].side);
        let sss_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh sss bind"),
            layout: &sss_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &sss_lut.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sss_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &matcap_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-mesh layout"),
            bind_group_layouts: &[Some(&bgl), Some(&obj_bgl), Some(&ao_bgl), Some(&sss_bgl)],
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
        /// O AO ASSADO por vértice — buffer próprio pela razão da curvatura, e
        /// por uma a mais: ele é o canal que muda MENOS de todos (só num bake
        /// explícito), então empacotá-lo com qualquer vizinho faria o upload
        /// dele viajar de carona em toda mudança de forma.
        const AO: [wgpu::VertexAttribute; 1] = f32_attr(4);
        /// A CURVATURA DE MUNDO por vértice (`1/comprimento`) — o eixo da tabela
        /// do SSS. Buffer próprio pela razão da irmã adimensional ao lado, e não
        /// empacotada COM ela apesar de mudarem no mesmo instante: um `vec2` num
        /// buffer só é o layout certo se as duas forem sempre lidas juntas, e o
        /// Cavity lê uma sem a outra em todo frame com o SSS desligado.
        const CURVW: [wgpu::VertexAttribute; 1] = f32_attr(5);
        /// A ESPESSURA assada por vértice — buffer próprio pela razão do AO, que
        /// é a irmã dele em tudo: os dois nascem do MESMO bake, mudam só nele, e
        /// empacotá-los juntos economizaria um buffer para pagar com um upload
        /// de canal que ninguém mexeu em toda troca de forma.
        const THICK: [wgpu::VertexAttribute; 1] = f32_attr(6);
        /// O PREVIEW do padrão do pincel — o canal **transiente** que mostra,
        /// no barro, o que o próximo traço vai depositar.
        ///
        /// ⚠️ **Irmão da máscara e não dela:** os dois são `f32` por vértice e
        /// pintam um tinto, mas a máscara é AUTORADA (ela protege) e este é
        /// DERIVADO do pincel vivo. Colapsá-los faria o preview apagar a
        /// proteção que o artista pintou — e restaurá-la depois seria uma
        /// promessa que um `return` esquecido quebra em silêncio.
        const PREVIEW: [wgpu::VertexAttribute; 1] = f32_attr(7);
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
            /// A entrada de VÉRTICE. Só o passe de arestas a troca — ver o
            /// `WIRE_DEPTH_NUDGE` no shader.
            vs: &'a str,
            entry: &'a str,
            format: wgpu::TextureFormat,
            /// O SEGUNDO alvo, quando o fragment escreve dois — hoje só o
            /// G-buffer, que doa a normal **e** a oclusão de forma.
            ///
            /// ⚠️ `Option` e não um formato com sentinela: *"este passe escreve
            /// um alvo só"* não é um formato, e um sentinela obrigaria os três
            /// chamadores a saber disso.
            second: Option<wgpu::TextureFormat>,
            topology: wgpu::PrimitiveTopology,
            blend: Option<wgpu::BlendState>,
            bias: wgpu::DepthBiasState,
        }
        let make = |v: Variant<'_>| {
            let Variant {
                label,
                vs,
                entry,
                format,
                second,
                topology,
                blend,
                bias,
            } = v;
            let targets = [
                Some(wgpu::ColorTargetState {
                    format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                second.map(|format| wgpu::ColorTargetState {
                    format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ];
            // Um alvo ausente é a LISTA mais curta, não um `None` no meio dela: o
            // wgpu lê `targets[i]` como *"o attachment i existe e está
            // desabilitado"*, e um buraco declarado aqui exigiria um attachment
            // vazio em toda render pass que usasse o pipeline.
            let targets = &targets[..if second.is_some() { 2 } else { 1 }];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(vs),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[
                        vec3_buffer(&POS),
                        vec3_buffer(&NRM),
                        f32_buffer(&MASK),
                        f32_buffer(&CURV),
                        f32_buffer(&AO),
                        f32_buffer(&CURVW),
                        f32_buffer(&THICK),
                        f32_buffer(&PREVIEW),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    // Opaco nos passes de FORMA: a escultura é sólida, e um blend
                    // ali só serviria para esconder um erro de profundidade atrás
                    // de uma mistura. O passe de ARESTAS é o oposto — ele anota a
                    // forma sem a apagar.
                    targets,
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
                    depth_write_enabled: Some(true),
                    // `Less` com limpeza em 1.0: profundidade 3D comum. (O Flip usa
                    // `Greater` porque a ordem dele é 2D por-traço, outra pergunta.)
                    depth_compare: Some(wgpu::CompareFunction::Less),
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
        // ⛔ **O viés da ARESTA é ZERO — ele nunca alcançou uma linha.** O WebGPU
        // exige viés nulo fora de topologia de triângulos, e o `wgpu` 29 passou a
        // VALIDÁ-LO (o 28 ignorava calado). O que faz a aresta ganhar da face é a
        // `WIRE_DEPTH_NUDGE` no shader — o mecanismo, a medição e o preço deste
        // campo morto estão lá, ao lado dela.
        const WIRE: wgpu::DepthBiasState = SOLID;
        let pipeline = make(Variant {
            label: "ph2d-mesh pipeline",
            vs: "vs_main",
            entry: "fs_main",
            format: target_format,
            second: None,
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: None,
            bias: SOLID,
        });
        let gbuffer_pipeline = make(Variant {
            label: "ph2d-mesh gbuffer",
            vs: "vs_main",
            entry: "fs_gbuffer",
            format: Self::GBUFFER_FORMAT,
            second: Some(Self::OCCLUSION_FORMAT),
            topology: wgpu::PrimitiveTopology::TriangleList,
            blend: None,
            bias: SOLID,
        });
        let wire_pipeline = make(Variant {
            label: "ph2d-mesh wire",
            vs: "vs_wire",
            entry: "fs_wire",
            format: target_format,
            second: None,
            topology: wgpu::PrimitiveTopology::LineList,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            bias: WIRE,
        });

        // **O 1×1 VAZIO** — *"ninguém mediu oclusão nesta vista"*.
        //
        // ⚠️ **É por causa deste fallback que o canal guarda OCLUSÃO e não
        // visibilidade.** Uma textura recém-criada nasce zerada, e ela precisa de
        // um conteúdo que signifique *nada aqui escurece nada*: com visibilidade,
        // zero quer dizer *tudo é sombra* e a peça sairia preta; com oclusão, zero
        // é exatamente a resposta certa — e sai **de graça**, sem escrever um
        // texel, sem `queue` no construtor, e sem que os vinte chamadores do
        // `new` mudem de assinatura por causa de um pixel.
        //
        // ⚠️ A mesma inversão faz o `textureLoad` FORA DOS LIMITES cair do lado
        // seguro: WGSL devolve zero, que aqui é *não oclui*. O shader clampa a
        // coordenada mesmo assim — depender do comportamento de borda seria uma
        // regra invisível, e ela é a diferença entre uma peça normal e uma peça
        // preta.
        let empty = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh ao empty"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::SSAO_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let ao_white_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh ao empty bind"),
            layout: &ao_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &empty.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        // ---- O PASSE de tela cheia do AO (`shaders/ssao.wgsl`) ----
        let ssao_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-mesh ssao bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // A PROFUNDIDADE, como textura de leitura.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // As NORMAIS — o mesmo G-buffer que a doação produz.
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let ssao_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-mesh ssao params"),
            size: crate::ssao::SsaoRaw::SIZE as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ssao_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-mesh ssao shader"),
            source: wgpu::ShaderSource::Wgsl(super::SSAO_WGSL.into()),
        });
        let ssao_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ph2d-mesh ssao pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ph2d-mesh ssao layout"),
                    bind_group_layouts: &[Some(&ssao_bgl)],
                    immediate_size: 0,
                }),
            ),
            vertex: wgpu::VertexState {
                module: &ssao_shader,
                entry_point: Some("vs_fullscreen"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                // Sem buffer nenhum: as coordenadas do triângulo saem do índice.
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_shader,
                entry_point: Some("fs_ssao"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: Self::SSAO_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::RED,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // Um triângulo de tela cheia não tem lado de trás a descartar, e o
                // sentido dele depende de como os índices caem — descartar aqui
                // seria uma tela preta por uma convenção invisível.
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // ⚠️ **Sem depth-stencil**: este passe LÊ a profundidade pelo bind
            // group, e anexá-la aqui a poria em escrita e leitura ao mesmo tempo.
            depth_stencil: None,
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
            gbuffer_pipeline,
            wire_pipeline,
            ssao_bgl,
            ssao_uniform,
            ssao_pipeline,
            ao_bgl,
            ao_white_bind,
            ssao: None,
            ssao_fresh: false,
            sss_lut,
            sss_bind,
            sss_lut_ready: false,
            matcap_tex,
            matcap_ready: None,
            // Guardados porque a imagem do matcap muda de LADO entre fontes
            // (512 do Blender, 749 do SculptGL), e trocar o tamanho de uma
            // textura exige recriá-la — e o bind group que aponta para ela.
            sss_bgl,
            sss_sampler,
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
            scratch_preview: Vec::new(),
            scratch_ao: Vec::new(),
            scratch_thickness: Vec::new(),
        }
    }
}
