//! ⭐⭐ **O CONTORNO DE CADA CÓPIA, CALCULADO UMA VEZ** (doc 121 §9.5) — o lado do CPU dos três
//! passes de cálculo de `contorno.wgsl`: os buffers, os pipelines e a capacidade.
//!
//! ⚠️ **A capacidade das arestas é MEDIDA, nunca adivinhada:** quantas arestas uma cópia escreve
//! depende do nível que o shader escolhe para ELA e da caneta no ecrã (o leque de uma junta redonda
//! cresce com o raio), e na rota do dispositivo as cópias nem passam pelo CPU. ⇒ o passe de soma
//! deixa o total no fim da contagem, ele é copiado para um buffer de leitura e lido DOIS quadros
//! depois (mapear exige a submissão anterior), e a capacidade cresce para ele. Até lá, uma cópia que
//! não cabe é desenhada pelo caminho de sempre — a mesma imagem, mais devagar.
//!
//! ⚠️ O tecto da capacidade é o do RECURSO: `max_storage_buffer_binding_size` do dispositivo
//! (uma ligação de armazenamento maior não se pode fazer).

use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use bytemuck::{Pod, Zeroable};
use ph2d_gpu::GpuContext;

/// O uniforme dos três passes (`Contas` no WGSL).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Contas {
    n: u32,
    cap: u32,
    sem_contorno: u32,
    cap_mascaras: u32,
    area_minima_conforme: f32,
    _p: [u32; 3],
}

/// ⭐ doc 121 §9.6 — **a área no ecrã (px², da caixa estimada) a partir da qual uma cópia CONFORME vai
/// pelas arestas no ecrã.** Uma cópia esticada vai sempre (é o caminho que a tira de refazer o eixo
/// em cada pixel); uma conforme pequena já se desenha bem pelos segmentos locais, e o cálculo é
/// pago POR CÓPIA — na escada de `32 768` estrelas pequenas ele custava `+3,9 ms` por zero ganho.
/// ⚠️ O número sai da varredura do doc 121 §9.6.
pub const AREA_MINIMA_CONFORME: f32 = 1024.0;

/// Bytes de UMA aresta (`vec4<f32>`: os dois pontos no ecrã).
const ARESTA: u64 = 16;
/// Arestas por bloco (o mesmo `SEGS_POR_BLOCO` do preenchimento: o shader lê os dois com a mesma
/// constante).
const BLOCO: u64 = crate::SEGS_POR_BLOCO as u64;
/// Arestas por cópia na primeira capacidade, antes de haver total medido. ⚠️ Só a PRIMEIRA: a
/// leitura do total substitui-a dois quadros depois. Uma estrela aguda de cinco pontas com a faixa
/// em todos os vértices escreve `20` (duas por troço); `32` cobre-a com um bloco de folga.
const ARESTAS_POR_COPIA_INICIAL: u64 = 32;
/// Palavras de registo por cópia na primeira capacidade — o mesmo papel do de cima: a leitura do
/// total substitui-o. Uma estrela pequena (`~12` linhas, uma célula, menos de `32` blocos ⇒ `3 + 1`
/// palavras por célula) pede `~48`.
const MASCARAS_POR_COPIA_INICIAL: u64 = 48;
/// Bytes de uma palavra de máscara (`u32`: um bit por bloco).
const PALAVRA: u64 = 4;

// Os estados da leitura do total (um `AtomicU8`, porque o fecho do `map_async` corre noutro sítio).
const LIVRE: u8 = 0;
const COPIADO: u8 = 1;
const MAPEANDO: u8 = 2;
const PRONTO: u8 = 3;

/// Os buffers e os pipelines do contorno.
pub(crate) struct Contorno {
    conta: wgpu::ComputePipeline,
    soma: wgpu::ComputePipeline,
    escreve: wgpu::ComputePipeline,
    /// doc 121 §9.7 — as células, um fio por LINHA, por despacho indirecto.
    celulas: wgpu::ComputePipeline,
    /// O grupo `1` do DESENHO (as quatro leituras).
    pub(crate) leitura: wgpu::BindGroupLayout,
    /// O grupo `2` do CÁLCULO (o uniforme e as cinco escritas).
    escrita: wgpu::BindGroupLayout,
    /// O mesmo grupo SEM o `despacho` (o passe das células, que o lê como argumento indirecto).
    escrita_celulas: wgpu::BindGroupLayout,
    /// O grupo `1` do CÁLCULO, vazio (ver `new`).
    vazio: wgpu::BindGroup,
    contas: wgpu::Buffer,
    contagem: wgpu::Buffer,
    arestas: wgpu::Buffer,
    blocos: wgpu::Buffer,
    copias: wgpu::Buffer,
    caixas: wgpu::Buffer,
    /// As máscaras por linha (doc 121 §9.6): por cópia, por fileira de pixels, um bit por bloco.
    mascaras: wgpu::Buffer,
    /// Os argumentos do despacho indirecto de `cs_celulas` (escritos pelo `cs_soma`).
    despacho: wgpu::Buffer,
    cap_copias: u64,
    cap_arestas: u64,
    cap_mascaras: u64,
    /// O tecto do recurso, em arestas.
    tecto_arestas: u64,
    /// O tecto do recurso, em palavras de máscara.
    tecto_mascaras: u64,
    leitura_total: wgpu::Buffer,
    estado: Arc<AtomicU8>,
    /// O maior total medido (em arestas).
    total_visto: u64,
    /// O maior total medido (em palavras de máscara).
    total_visto_m: u64,
    /// `false` ⇒ nenhuma cópia ganha contorno (o caminho pixel a pixel, para os gates o compararem).
    pub(crate) ligado: bool,
    /// [`AREA_MINIMA_CONFORME`], ou o que um gate pediu.
    pub(crate) area_minima_conforme: f32,
}

fn entrada(
    binding: u32,
    ty: wgpu::BindingType,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

fn armazem(read_only: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage { read_only },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

fn buffer(gpu: &GpuContext, label: &str, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16).next_multiple_of(16),
        usage,
        mapped_at_creation: false,
    })
}

impl Contorno {
    pub(crate) fn new(
        gpu: &GpuContext,
        module: &wgpu::ShaderModule,
        grupo0: &wgpu::BindGroupLayout,
    ) -> Self {
        let device = &gpu.device;
        let vf = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;
        let leitura = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (leitura)"),
            entries: &[
                entrada(0, armazem(true), vf),
                entrada(1, armazem(true), vf),
                entrada(2, armazem(true), vf),
                entrada(3, armazem(true), vf),
                entrada(4, armazem(true), vf),
            ],
        });
        let c = wgpu::ShaderStages::COMPUTE;
        let escritas = [
            entrada(
                0,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                c,
            ),
            entrada(1, armazem(false), c),
            entrada(2, armazem(false), c),
            entrada(3, armazem(false), c),
            entrada(4, armazem(false), c),
            entrada(5, armazem(false), c),
            entrada(6, armazem(false), c),
            entrada(7, armazem(false), c),
        ];
        let escrita = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita)"),
            entries: &escritas,
        });
        // ⚠️ doc 121 §9.7 — o passe das células tem o grupo SEM o `despacho` (ligação `7`): ele é o
        // argumento do despacho indirecto desse passe, e o wgpu recusa o mesmo buffer como escrita e
        // como argumento no mesmo despacho (usos exclusivos no mesmo âmbito).
        let escrita_celulas = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita, celulas)"),
            entries: &escritas[..7],
        });
        // O grupo `1` do cálculo é o do DESENHO e fica vazio: um pipeline só tem de declarar o que
        // os pontos de entrada dele lêem, e os de cálculo não lêem as leituras.
        let vazio = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (vazio)"),
            entries: &[],
        });
        let vazio_grupo = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu contorno (vazio)"),
            layout: &vazio,
            entries: &[],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno"),
            bind_group_layouts: &[Some(grupo0), Some(&vazio), Some(&escrita)],
            immediate_size: 0,
        });
        let pl_celulas = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (celulas)"),
            bind_group_layouts: &[Some(grupo0), Some(&vazio), Some(&escrita_celulas)],
            immediate_size: 0,
        });
        let pipeline_em = |entry: &str, pl: &wgpu::PipelineLayout| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("ph2d-shape-gpu contorno"),
                layout: Some(pl),
                module,
                entry_point: Some(entry),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            })
        };
        let pipeline = |entry: &str| pipeline_em(entry, &pl);
        let tecto_arestas = device.limits().max_storage_buffer_binding_size / ARESTA;
        let tecto_mascaras = device.limits().max_storage_buffer_binding_size / PALAVRA;
        let armazens = wgpu::BufferUsages::STORAGE;
        Self {
            conta: pipeline("cs_conta"),
            soma: pipeline("cs_soma"),
            escreve: pipeline("cs_escreve"),
            celulas: pipeline_em("cs_celulas", &pl_celulas),
            leitura,
            escrita,
            escrita_celulas,
            vazio: vazio_grupo,
            contas: buffer(
                gpu,
                "ph2d-shape-gpu contas",
                std::mem::size_of::<Contas>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            contagem: buffer(
                gpu,
                "ph2d-shape-gpu contagem",
                16,
                armazens | wgpu::BufferUsages::COPY_SRC,
            ),
            arestas: buffer(gpu, "ph2d-shape-gpu arestas", 16, armazens),
            blocos: buffer(gpu, "ph2d-shape-gpu blocos do contorno", 16, armazens),
            copias: buffer(gpu, "ph2d-shape-gpu copias do contorno", 16, armazens),
            caixas: buffer(gpu, "ph2d-shape-gpu caixas do contorno", 16, armazens),
            mascaras: buffer(gpu, "ph2d-shape-gpu mascaras do contorno", 16, armazens),
            despacho: buffer(
                gpu,
                "ph2d-shape-gpu despacho das celulas",
                12,
                armazens | wgpu::BufferUsages::INDIRECT,
            ),
            cap_copias: 0,
            cap_arestas: 0,
            cap_mascaras: 0,
            tecto_arestas,
            tecto_mascaras,
            leitura_total: buffer(
                gpu,
                "ph2d-shape-gpu total do contorno",
                16,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            estado: Arc::new(AtomicU8::new(LIVRE)),
            total_visto: 0,
            total_visto_m: 0,
            ligado: true,
            area_minima_conforme: std::env::var("PH2D_AREA_MINIMA_CONFORME")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(AREA_MINIMA_CONFORME),
        }
    }

    /// A leitura do total: colhe a que ficou pronta, e manda mapear a que foi copiada no quadro
    /// anterior. ⚠️ **Mapear exige a submissão do encoder que copiou** — é por isso que se manda no
    /// quadro SEGUINTE, e por isso o `ShapePass::draw` exige que o encoder dele seja submetido antes
    /// do próximo desenho.
    fn colhe(&mut self) {
        match self.estado.load(Ordering::Acquire) {
            PRONTO => {
                {
                    let dados = self.leitura_total.slice(..).get_mapped_range();
                    let total: u32 = bytemuck::pod_read_unaligned(&dados[..4]);
                    let total_m: u32 = bytemuck::pod_read_unaligned(&dados[4..8]);
                    self.total_visto = self.total_visto.max(u64::from(total));
                    self.total_visto_m = self.total_visto_m.max(u64::from(total_m));
                }
                self.leitura_total.unmap();
                self.estado.store(LIVRE, Ordering::Release);
            }
            COPIADO => {
                self.estado.store(MAPEANDO, Ordering::Release);
                let estado = Arc::clone(&self.estado);
                self.leitura_total
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |r| {
                        // Um mapeamento falhado volta a LIVRE: a capacidade fica a que estava, e a
                        // cópia que não cabe continua pelo caminho de sempre.
                        estado.store(if r.is_ok() { PRONTO } else { LIVRE }, Ordering::Release);
                    });
            }
            _ => {}
        }
    }

    /// Garante os buffers para `n` cópias e para o total medido.
    fn garante(&mut self, gpu: &GpuContext, n: u64) {
        if n > self.cap_copias {
            let cap = n.next_power_of_two();
            let armazens = wgpu::BufferUsages::STORAGE;
            // Três terços de `n + 1`: as arestas, as palavras de máscara e as linhas de ecrã.
            self.contagem = buffer(
                gpu,
                "ph2d-shape-gpu contagem",
                3 * (cap + 1) * 4,
                armazens | wgpu::BufferUsages::COPY_SRC,
            );
            // Três `vec4<u32>` por cópia. `COPY_SRC`: o instrumento `copias_com_contorno` lê-o.
            self.copias = buffer(
                gpu,
                "ph2d-shape-gpu copias do contorno",
                cap * 48,
                armazens | wgpu::BufferUsages::COPY_SRC,
            );
            self.caixas = buffer(gpu, "ph2d-shape-gpu caixas do contorno", cap * 16, armazens);
            self.cap_copias = cap;
        }
        let pedido = self
            .total_visto
            .max(n * ARESTAS_POR_COPIA_INICIAL)
            .min(self.tecto_arestas);
        if pedido > self.cap_arestas {
            let cap = pedido
                .next_power_of_two()
                .min(self.tecto_arestas)
                .next_multiple_of(BLOCO);
            let armazens = wgpu::BufferUsages::STORAGE;
            self.arestas = buffer(gpu, "ph2d-shape-gpu arestas", cap * ARESTA, armazens);
            self.blocos = buffer(
                gpu,
                "ph2d-shape-gpu blocos do contorno",
                // Dois `vec4` por bloco: a caixa e `(y₀, y₈, encadeado, 0)`.
                cap / BLOCO * 32,
                armazens,
            );
            self.cap_arestas = cap;
        }
        let pedido_m = self
            .total_visto_m
            .max(n * MASCARAS_POR_COPIA_INICIAL)
            .min(self.tecto_mascaras);
        if pedido_m > self.cap_mascaras {
            let cap = pedido_m.next_power_of_two().min(self.tecto_mascaras);
            self.mascaras = buffer(
                gpu,
                "ph2d-shape-gpu mascaras do contorno",
                cap * PALAVRA,
                wgpu::BufferUsages::STORAGE,
            );
            self.cap_mascaras = cap;
        }
    }

    /// Codifica os quatro passes para `count` cópias (o grupo `0` é o do desenho).
    pub(crate) fn calcula(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        grupo0: &wgpu::BindGroup,
        count: u32,
    ) {
        self.colhe();
        self.garante(gpu, u64::from(count));
        let contas = Contas {
            n: count,
            cap: u32::try_from(self.cap_arestas).unwrap_or(u32::MAX),
            sem_contorno: u32::from(!self.ligado),
            cap_mascaras: u32::try_from(self.cap_mascaras).unwrap_or(u32::MAX),
            area_minima_conforme: self.area_minima_conforme,
            _p: [0; 3],
        };
        gpu.queue
            .write_buffer(&self.contas, 0, bytemuck::bytes_of(&contas));
        let mut entradas = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: self.contas.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: self.contagem.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: self.arestas.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: self.blocos.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: self.copias.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: self.caixas.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: self.mascaras.as_entire_binding(),
            },
        ];
        // O grupo do passe das células: as mesmas ligações SEM o `despacho`.
        let celulas = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita, celulas)"),
            layout: &self.escrita_celulas,
            entries: &entradas,
        });
        entradas.push(wgpu::BindGroupEntry {
            binding: 7,
            resource: self.despacho.as_entire_binding(),
        });
        let escrita = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita)"),
            layout: &self.escrita,
            entries: &entradas,
        });
        let grupos = count.div_ceil(64);
        let x = grupos.min(65_535);
        let y = grupos.div_ceil(x.max(1));
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ph2d-shape-gpu contorno"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("render.contorno"),
            });
            pass.set_bind_group(0, grupo0, &[]);
            pass.set_bind_group(1, &self.vazio, &[]);
            pass.set_bind_group(2, &escrita, &[]);
            pass.set_pipeline(&self.conta);
            pass.dispatch_workgroups(x, y, 1);
            pass.set_pipeline(&self.soma);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(&self.escreve);
            pass.dispatch_workgroups(x, y, 1);
            // ⭐ doc 121 §9.7 — as células, um fio por linha de ecrã: o número de linhas só existe
            // na placa (o `cs_soma` escreve os grupos), logo o despacho é INDIRECTO.
            pass.set_pipeline(&self.celulas);
            pass.set_bind_group(2, &celulas, &[]);
            pass.dispatch_workgroups_indirect(&self.despacho, 0);
        }
        if self.estado.load(Ordering::Acquire) == LIVRE {
            // Os dois totais: as arestas em `n` e as palavras de máscara em `2n + 1`.
            encoder.copy_buffer_to_buffer(
                &self.contagem,
                u64::from(count) * 4,
                &self.leitura_total,
                0,
                4,
            );
            encoder.copy_buffer_to_buffer(
                &self.contagem,
                (2 * u64::from(count) + 1) * 4,
                &self.leitura_total,
                4,
                4,
            );
            self.estado.store(COPIADO, Ordering::Release);
        }
    }

    /// **Quantas das `n` cópias ganharam contorno** no último cálculo — lido de volta, bloqueando.
    /// Instrumento: um gate e uma sonda que perguntam se o caminho novo CORREU (as duas imagens são
    /// iguais, logo nenhuma régua de pixel o distingue do caminho de sempre).
    pub(crate) fn copias_com_contorno(&self, gpu: &GpuContext, n: u32) -> (u32, u64) {
        let bytes = (u64::from(n) * 48).max(16);
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu contorno (sonda)",
            bytes,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(&self.copias, 0, &leitura, 0, u64::from(n) * 48);
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        let copias: &[[u32; 4]] = bytemuck::cast_slice(&dados[..(u64::from(n) * 48) as usize]);
        // O 1.º de cada trio: blocos do preenchimento, das marcas e do contorno.
        let com = copias
            .iter()
            .step_by(3)
            .filter(|c| c[1] + c[2] + c[3] > 0)
            .count();
        (u32::try_from(com).unwrap_or(u32::MAX), self.cap_arestas)
    }

    /// O grupo `1` do desenho: as leituras do que os passes escreveram.
    pub(crate) fn grupo_de_leitura(&self, gpu: &GpuContext) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu contorno (leitura)"),
            layout: &self.leitura,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.arestas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.blocos.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.copias.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.caixas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.mascaras.as_entire_binding(),
                },
            ],
        })
    }
}
