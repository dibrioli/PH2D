//! **O PASSE** — o pipeline, os buffers de geometria e a chamada de desenho.

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;

use crate::geometry::{GeometryRecord, ShapeGeometry};

/// A projecção mundo → pixel do alvo: `pixel = lin · mundo + t`, com `lin = [[a, c], [b, d]]`
/// guardado como `(a, b, c, d)` — os mesmos coeficientes de um `Affine` do kurbo.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ShapeView {
    pub lin: [f32; 4],
    pub t: [f32; 2],
    /// O tamanho do alvo em pixels.
    pub alvo: [f32; 2],
}

/// Uma CÓPIA — a pose de uma [`ph2d_eval_motion`]-`VectorInstance` na forma que a placa lê.
///
/// ⚠️ `geometry` é o HANDLE da geometria (o `geometry_id` do stream), não um índice: o passe
/// resolve-o contra os handles ordenados que [`ShapePass::set_geometries`] carregou.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct ShapeInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub basis: [f32; 4],
    pub anchor: [f32; 2],
    pub geometry: u32,
    pub _pad: u32,
    /// A cor do preenchimento, não pré-multiplicada.
    pub tint: [f32; 4],
}

/// As cópias a desenhar: o buffer (de [`ShapeInstance`]) e quantas.
///
/// ⚠️ O buffer é de quem chama: o do próprio passe ([`ShapePass::uploaded`], a rota da CPU) ou um do
/// cozimento na placa, que as desenha sem nunca as ler de volta.
#[derive(Clone, Copy)]
pub struct Copias<'a> {
    pub buffer: &'a wgpu::Buffer,
    pub count: u32,
}

/// O passe: um pipeline por formato de alvo, e as geometrias carregadas.
pub struct ShapePass {
    /// As duas variantes do tracejado (doc 121 §9.10, [`crate::contorno::Variantes`]).
    pipeline: crate::contorno::Variantes<wgpu::RenderPipeline>,
    /// Algum troço do eixo carregado é tracejado ⇒ a variante COMPLETA (o mesmo buffer que o shader lê).
    tracejado: bool,
    layout: wgpu::BindGroupLayout,
    view_buf: wgpu::Buffer,
    records: wgpu::Buffer,
    segs: wgpu::Buffer,
    handles: wgpu::Buffer,
    /// Os itens do eixo do traço de todas as geometrias ([`crate::eixo`]).
    eixo: wgpu::Buffer,
    /// Os blocos de segmentos ([`crate::blocos`]), um por `SEGS_POR_BLOCO` de `segs`.
    blocos: wgpu::Buffer,
    instances: Option<wgpu::Buffer>,
    /// As geometrias carregadas, pela ordem dos handles — o que a comparação de [`Self::set_geometries`] lê.
    carregadas: Vec<u32>,
    /// O contorno de cada cópia calculado uma vez (doc 121 §9.5).
    contorno: crate::contorno::Contorno,
    /// O grupo de ligação e as cópias do último desenho — o que o [`Self::redesenha`] repete.
    ultimo: Option<(wgpu::BindGroup, u32)>,
}

/// O shader: o desenho e os passes de cálculo do contorno num MÓDULO só — os dois lêem a mesma
/// `copia_de` e a mesma geometria do traço (`bissectriz`), e escritas duas vezes elas divergiriam.
const SHADER: &str = concat!(include_str!("shape.wgsl"), include_str!("contorno.wgsl"));
/// doc 121 §9.15 (d) — o `cs_varre` por subgrupo, num módulo à parte: um módulo com operações de
/// subgrupo não valida num dispositivo sem `Features::SUBGROUP`.
const SUBGRUPO: &str = include_str!("contorno_subgrupo.wgsl");

fn storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        // ⚠️ COMPUTE também: os passes do contorno (`contorno.wgsl`) lêem as cópias e o eixo.
        visibility: wgpu::ShaderStages::VERTEX
            | wgpu::ShaderStages::FRAGMENT
            | wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn buffer_com(
    gpu: &GpuContext,
    label: &str,
    bytes: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    // ⚠️ Um buffer de armazenamento vazio não se pode ligar: o mínimo é uma palavra.
    let size = (bytes.len() as u64).max(16).next_multiple_of(4);
    let b = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    if !bytes.is_empty() {
        gpu.queue.write_buffer(&b, 0, bytes);
    }
    b
}

/// doc 121 §9.16 — uma VARIANTE do passe, para a sonda intercalada medir pedaços lado a lado no MESMO
/// processo: as constantes `override` do contorno ([`ShapePass::com_constantes`]) e as duas portas de
/// execução. A omissão é o produto.
#[derive(Clone, Debug, Default)]
pub struct VarianteDoPasse {
    pub constantes: Vec<(&'static str, f64)>,
    pub sem_subgrupo: bool,
    pub sem_medida_no_inicio: bool,
}

impl ShapePass {
    /// O passe da `variante` (a omissão é [`Self::new`]).
    #[must_use]
    pub fn da_variante(gpu: &GpuContext, format: wgpu::TextureFormat, v: &VarianteDoPasse) -> Self {
        let mut p = Self::com_constantes(gpu, format, &v.constantes);
        p.com_subgrupo(!v.sem_subgrupo);
        p.mede_a_capacidade_no_inicio(!v.sem_medida_no_inicio);
        p
    }

    /// Um passe que desenha para alvos de `format`, com a mistura pré-multiplicada «por cima»
    /// (a do Vello: `Mix::Normal` + `Compose::SrcOver`).
    #[must_use]
    pub fn new(gpu: &GpuContext, format: wgpu::TextureFormat) -> Self {
        Self::com_constantes(gpu, format, &[])
    }

    /// O mesmo passe com constantes `override` do módulo do contorno (doc 121 §9.16) — a porta da
    /// sonda intercalada, que liga e desliga pedaços ao criar o pipeline, sem recompilar. Hoje:
    /// `AJUSTE_NA_CONTAGEM` · `TOTAL_NO_PERCURSO` · `ARESTAS_COMPACTAS` · `JUNTA_UMA_POR_TROCO` (`1` ou `0`).
    #[must_use]
    pub fn com_constantes(
        gpu: &GpuContext,
        format: wgpu::TextureFormat,
        constantes: &[(&str, f64)],
    ) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ph2d-shape-gpu"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT
                        | wgpu::ShaderStages::COMPUTE,
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
                storage_entry(4),
                storage_entry(5),
                storage_entry(6),
            ],
        });
        let modulo_subgrupo = device
            .features()
            .contains(wgpu::Features::SUBGROUP)
            .then(|| {
                device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("ph2d-shape-gpu (subgrupo)"),
                    source: wgpu::ShaderSource::Wgsl(format!("{SHADER}{SUBGRUPO}").into()),
                })
            });
        let contorno = crate::contorno::Contorno::new(
            gpu,
            &module,
            modulo_subgrupo.as_ref(),
            &layout,
            constantes,
        );
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu"),
            bind_group_layouts: &[Some(&layout), Some(&contorno.leitura)],
            immediate_size: 0,
        });
        let pipeline = crate::contorno::Variantes::cria(&[], |compilation_options| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("ph2d-shape-gpu"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs_main"),
                    compilation_options: compilation_options.clone(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_main"),
                    compilation_options,
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        });
        let view_buf = buffer_com(
            gpu,
            "ph2d-shape-gpu view",
            bytemuck::bytes_of(&ShapeView::zeroed()),
            wgpu::BufferUsages::UNIFORM,
        );
        let vazio = |l| buffer_com(gpu, l, &[], wgpu::BufferUsages::STORAGE);
        Self {
            pipeline,
            tracejado: false,
            layout,
            view_buf,
            records: vazio("ph2d-shape-gpu records"),
            segs: vazio("ph2d-shape-gpu segs"),
            handles: vazio("ph2d-shape-gpu handles"),
            eixo: vazio("ph2d-shape-gpu eixo"),
            blocos: vazio("ph2d-shape-gpu blocos"),
            instances: None,
            carregadas: Vec::new(),
            contorno,
            ultimo: None,
        }
    }

    /// **As geometrias carregadas têm algum troço tracejado** — o passe desenha com a variante
    /// COMPLETA; sem nenhum, com a ENXUTA (doc 121 §9.10). Instrumento de gates.
    #[must_use]
    pub fn usa_o_tracejado(&self) -> bool {
        self.tracejado
    }

    /// **Quantas das `n` cópias do último desenho ganharam o contorno calculado**, e a capacidade
    /// de arestas — lido de volta da placa (bloqueia). Instrumento de gates e sondas.
    #[must_use]
    pub fn copias_com_contorno(&self, gpu: &GpuContext, n: u32) -> (u32, u64) {
        self.contorno.copias_com_contorno(gpu, n)
    }

    /// **As arestas do último desenho: `(reservadas, escritas, do contorno)`** (doc 121 §9.15) — lido
    /// de volta da placa (bloqueia). Instrumento de sondas.
    #[must_use]
    pub fn arestas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64, u64) {
        self.contorno.arestas_do_ultimo_quadro(gpu)
    }

    /// **As cópias do último desenho por variante `(enxuta, completa)`** numa cena com tracejado (doc
    /// 121 §9.13): a placa põe-nas todas na enxuta e passa-as à completa quando uma cópia tracejada e
    /// visível fica sem células. Lido de volta (bloqueia). Instrumento de gates.
    #[must_use]
    pub fn copias_por_variante(&self, gpu: &GpuContext) -> (u32, u32) {
        self.contorno.copias_por_variante(gpu)
    }

    /// **Quantas das células em uso o último desenho TOCOU, e quantas usou** (doc 121 §9.13, a
    /// alavanca da variante esparsa) — lido de volta da placa (bloqueia). Instrumento de sondas.
    #[must_use]
    pub fn celulas_tocadas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        self.contorno.celulas_tocadas_do_ultimo_quadro(gpu)
    }

    /// **Quantas células o último desenho pediu, e a capacidade delas** (doc 121 §9.12) — lido de
    /// volta da placa (bloqueia). Pedido acima da capacidade ⇒ alguma cópia foi desenhada pelo
    /// caminho de sempre. Instrumento de gates e sondas.
    #[must_use]
    pub fn celulas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        self.contorno.celulas_do_ultimo_quadro(gpu)
    }

    /// Um TECTO para a capacidade das células — a porta pela qual um gate faz cópias transbordarem
    /// e mede que o recurso por cópia desenha a mesma imagem ao lado das que ficam nas células.
    pub fn limita_as_celulas(&mut self, celulas: u64) {
        self.contorno.celulas_no_maximo = celulas;
    }

    /// `false` ⇒ nenhuma cópia ganha o contorno calculado e o traço sai pixel a pixel do eixo, como
    /// antes do doc 121 §9.5 — a porta pela qual os gates comparam os dois caminhos.
    pub fn com_contorno(&mut self, ligado: bool) {
        self.contorno.ligado = ligado;
    }

    /// `false` ⇒ o prefixo das células pela memória de grupo mesmo onde há o de subgrupo (doc 121
    /// §9.15 d) — a porta pela qual um gate compara os dois, byte a byte.
    pub fn com_subgrupo(&mut self, ligado: bool) {
        self.contorno.subgrupo = ligado;
    }

    /// `false` ⇒ uma cena nova espera dois quadros pela capacidade medida, pixel a pixel (doc 121
    /// §9.15 c2) — a porta dos gates que medem a mistura dos dois caminhos no 1.º quadro.
    pub fn mede_a_capacidade_no_inicio(&mut self, ligado: bool) {
        self.contorno.mede_no_inicio = ligado;
    }

    /// O dispositivo tem o prefixo por subgrupo (instrumento de gates: o controlo de que os dois
    /// caminhos existem).
    #[must_use]
    pub fn tem_subgrupo(&self) -> bool {
        self.contorno.tem_subgrupo()
    }

    /// A área no ecrã (px²) a partir da qual uma cópia CONFORME vai pelas arestas no ecrã (doc 121
    /// §9.6). `0` ⇒ TODAS as cópias — é como os gates medem o caminho novo nas formas pequenas.
    pub fn area_minima_conforme(&mut self, px2: f32) {
        self.contorno.area_minima_conforme = px2;
    }

    /// Carrega as geometrias deste quadro. ⚠️ **Só reconstrói se o CONJUNTO de handles mudou** —
    /// uma geometria é imutável sob o seu handle (o `VecPathStore` nunca recicla um), logo o
    /// mesmo conjunto é o mesmo conteúdo.
    pub fn set_geometries<'g>(
        &mut self,
        gpu: &GpuContext,
        geos: impl IntoIterator<Item = (u32, &'g ShapeGeometry)>,
    ) {
        let mut v: Vec<(u32, &ShapeGeometry)> = geos.into_iter().collect();
        v.sort_by_key(|(h, _)| *h);
        v.dedup_by_key(|(h, _)| *h);
        let chaves: Vec<u32> = v.iter().map(|(h, _)| *h).collect();
        if chaves == self.carregadas {
            return;
        }
        // doc 121 §9.15 (c2) — outra cena: o próximo cálculo mede a capacidade antes de desenhar.
        self.contorno.medir_ja = true;
        let mut records: Vec<GeometryRecord> = Vec::with_capacity(v.len());
        let mut segs: Vec<[f32; 4]> = Vec::new();
        let mut eixo: Vec<crate::EixoItem> = Vec::new();
        let mut blocos: Vec<crate::BlocoDeSegmentos> = Vec::new();
        for (_, g) in &v {
            // ⚠️ O bloco `k` é `segs[8k..8k+8]` só se cada geometria começar num múltiplo de 8 —
            // e começa, porque cada trecho dela foi completado ([`crate::blocos`]).
            debug_assert!(segs.len().is_multiple_of(crate::SEGS_POR_BLOCO));
            let base = u32::try_from(segs.len()).expect("segmentos cabem em u32");
            let base_eixo = u32::try_from(eixo.len()).expect("itens do eixo cabem em u32");
            let mut r = g.record;
            for rg in &mut r.ranges {
                rg[0] += base;
                rg[2] += base;
            }
            for e in &mut r.eixo {
                e[0] += base_eixo;
            }
            records.push(r);
            segs.extend_from_slice(&g.segments);
            blocos.extend_from_slice(&g.blocos);
            eixo.extend_from_slice(&g.eixo);
        }
        // ⛔ **Um item a mais quando não há eixo nenhum:** o mínimo do buffer é `16` bytes e um
        // item tem `48` — uma ligação mais curta que UM elemento do `array<Eixo>` reprova a
        // validação do desenho, e o passe não pintava um pixel numa cena só de preenchimentos
        // (medido na 1.ª corrida da W4: `0 px` contra `77 296`). Nenhum registo o conta.
        if eixo.is_empty() {
            eixo.push(crate::EixoItem::default());
        }
        self.tracejado = eixo.iter().any(crate::EixoItem::tracejado);
        self.eixo = buffer_com(
            gpu,
            "ph2d-shape-gpu eixo",
            bytemuck::cast_slice(&eixo),
            wgpu::BufferUsages::STORAGE,
        );
        if blocos.is_empty() {
            blocos.push(crate::BlocoDeSegmentos::default());
        }
        self.blocos = buffer_com(
            gpu,
            "ph2d-shape-gpu blocos",
            bytemuck::cast_slice(&blocos),
            wgpu::BufferUsages::STORAGE,
        );
        self.records = buffer_com(
            gpu,
            "ph2d-shape-gpu records",
            bytemuck::cast_slice(&records),
            wgpu::BufferUsages::STORAGE,
        );
        self.segs = buffer_com(
            gpu,
            "ph2d-shape-gpu segs",
            bytemuck::cast_slice(&segs),
            wgpu::BufferUsages::STORAGE,
        );
        // ⛔⛔ **O enchimento do buffer é `u32::MAX` e nunca zero.** Um buffer de armazenamento tem
        // mínimo de 16 bytes, e o shader faz uma BUSCA BINÁRIA sobre `arrayLength` — com um handle
        // só, os três lugares de enchimento a zero deixavam a lista `[7, 0, 0, 0]` DESORDENADA, a
        // busca nunca achava o `7` e o passe não desenhava um pixel (medido na 1.ª corrida da
        // paridade: `0 px` contra `77 296` do Vello). O máximo mantém a ordem, e nenhum handle vivo o
        // alcança (o `VecPathStore` conta a partir de `1`).
        let mut ordenados = chaves.clone();
        while ordenados.len() < 4 {
            ordenados.push(u32::MAX);
        }
        self.handles = buffer_com(
            gpu,
            "ph2d-shape-gpu handles",
            bytemuck::cast_slice(&ordenados),
            wgpu::BufferUsages::STORAGE,
        );
        self.carregadas = chaves;
    }

    /// Carrega as cópias da rota da CPU num buffer do passe (que cresce e se reaproveita).
    pub fn upload_instances(&mut self, gpu: &GpuContext, insts: &[ShapeInstance]) {
        let bytes: &[u8] = bytemuck::cast_slice(insts);
        let precisa = (bytes.len() as u64).max(64);
        if self.instances.as_ref().is_none_or(|b| b.size() < precisa) {
            self.instances = Some(gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-shape-gpu instances"),
                size: precisa.next_power_of_two(),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if !bytes.is_empty() {
            gpu.queue.write_buffer(
                self.instances.as_ref().expect("acabou de ser garantido"),
                0,
                bytes,
            );
        }
    }

    /// O buffer que [`Self::upload_instances`] encheu — para passar a [`Self::draw`].
    #[must_use]
    pub fn uploaded(&self) -> Option<&wgpu::Buffer> {
        self.instances.as_ref()
    }

    /// Desenha `count` cópias de `instances` sobre `target`, por cima do que lá está (ou depois
    /// de o limpar, conforme `load`).
    ///
    /// ⚠️ **O `encoder` tem de ser SUBMETIDO antes do próximo `draw`:** o total de arestas do
    /// contorno é copiado nele para um buffer de leitura que o desenho seguinte manda mapear
    /// (`contorno.rs`), e mapear um buffer cuja cópia não foi submetida é um erro de validação.
    pub fn draw(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        view: ShapeView,
        copias: Copias<'_>,
    ) {
        let Copias {
            buffer: instances,
            count,
        } = copias;
        gpu.queue
            .write_buffer(&self.view_buf, 0, bytemuck::bytes_of(&view));
        let bg = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.view_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.records.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.segs.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.handles.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.eixo.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.blocos.as_entire_binding(),
                },
            ],
        });
        let desenha = count > 0 && !self.carregadas.is_empty();
        if desenha {
            self.contorno
                .calcula(gpu, encoder, &bg, count, self.tracejado);
        }
        self.ultimo = desenha.then(|| (bg.clone(), count));
        self.passe_de_desenho(gpu, encoder, target, load, desenha.then_some((&bg, count)));
    }

    /// ⭐ doc 121 §9.14 (c) — **o último desenho outra vez, noutro alvo**, por cima do que lá está: as
    /// mesmas cópias e as células que o cálculo desse desenho deixou, sem as recalcular. É o halo do
    /// `fx.glow` (o alvo `Rgba16Float` do brilho: o HDR do `tint` sobrevive). ⚠️ O alvo tem de ter o
    /// tamanho do último (o `alvo` da vista é o dele), e o encoder do último desenho já submetido.
    pub fn redesenha(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        if let Some((bg, count)) = &self.ultimo {
            self.passe_de_desenho(gpu, encoder, target, wgpu::LoadOp::Load, Some((bg, *count)));
        }
    }

    /// O passe de desenho sobre `target`; `None` só limpa (ou carrega) o alvo.
    fn passe_de_desenho(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        load: wgpu::LoadOp<wgpu::Color>,
        copias: Option<(&wgpu::BindGroup, u32)>,
    ) {
        let leitura = self.contorno.grupo_de_leitura(gpu);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-shape-gpu"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            // O relógio da placa do perfilador (`PH2D_FLUID_PROFILE=1`) — `None` com ele desligado.
            timestamp_writes: ph2d_gpu::pass_profiler::render_writes("render.formas"),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let Some((bg, count)) = copias else {
            return;
        };
        pass.set_bind_group(0, bg, &[]);
        pass.set_bind_group(1, &leitura, &[]);
        if self.tracejado {
            // doc 121 §9.13 — a placa escolhe: a completa só num quadro com uma cópia tracejada
            // desenhada pixel a pixel.
            self.contorno
                .desenha_pela_variante_da_placa(&mut pass, &self.pipeline);
        } else {
            pass.set_pipeline(self.pipeline.de(false));
            pass.draw(0..6, 0..count);
        }
    }
}
