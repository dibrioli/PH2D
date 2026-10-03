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
    cap_celulas: u32,
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
// ⛔ doc 121 §9.12 — **as células NÃO têm palpite de fábrica por cópia**: só a capacidade MEDIDA (dois
// quadros depois; até lá as cópias vão pelo caminho de sempre, a mesma imagem). Um palpite entra no
// `max` e nunca mais sai: com `16` por cópia (`528 B` cada) a `=127` densa ficava com `132 MB` e a
// escada de `32 768` com `264 MB`, pedidas pelo PALPITE e não pela cena (medido no app, W5 de 03/10).
/// Pixels de uma célula (`PIXELS_DA_CELULA` no WGSL).
const PIXELS_DA_CELULA: u64 = 32;
/// Bytes de uma célula em cada buffer (doc 121 §9.12): o REGISTO (os três fundos e a regra) e a
/// ACUMULAÇÃO (três famílias × um depósito por pixel; o `cs_varre` grava a cobertura acabada no lugar
/// do depósito do preenchimento — sem buffer próprio, `528 → 400 B` por célula).
const REGISTO: u64 = 4 * 4;
const ACUMULA: u64 = 3 * PIXELS_DA_CELULA * 4;

/// A capacidade das células para `n` pedidas: o múltiplo seguinte de um OITAVO do degrau de potência
/// de dois abaixo de `n` (doc 121 §9.12). A potência de dois pedia até o DOBRO do medido (`107 520 →
/// 131 072` na `=127` densa); assim sobra menos de `1/8`. Só as células: crescem só por medição.
fn ao_oitavo_do_degrau(n: u64) -> u64 {
    n.next_multiple_of(1 << n.max(1).ilog2().saturating_sub(3))
}

/// Onde estão, no buffer do despacho, os argumentos de um fio por LINHA, por ARESTA e por PIXEL de
/// célula (`despacha` no WGSL).
const POR_LINHA: u64 = 0;
const POR_ARESTA: u64 = 12;
const POR_PIXEL: u64 = 24;

// Os estados da leitura do total (um `AtomicU8`, porque o fecho do `map_async` corre noutro sítio).
const LIVRE: u8 = 0;
const COPIADO: u8 = 1;
const MAPEANDO: u8 = 2;
const PRONTO: u8 = 3;

/// Os buffers e os pipelines do contorno.
pub(crate) struct Contorno {
    /// A contagem e a escrita percorrem o eixo: têm as duas variantes do tracejado ([`Variantes`]).
    conta: Variantes<wgpu::ComputePipeline>,
    soma: wgpu::ComputePipeline,
    escreve: Variantes<wgpu::ComputePipeline>,
    /// doc 121 §9.12 — as células por acumulação, por despachos indirectos: um fio por PIXEL apaga,
    /// um por ARESTA deposita, um por LINHA faz o prefixo do fundo, um por PIXEL varre a célula.
    zera: wgpu::ComputePipeline,
    deposita: wgpu::ComputePipeline,
    fundo: wgpu::ComputePipeline,
    varre: wgpu::ComputePipeline,
    /// O grupo `1` do DESENHO (as três leituras).
    pub(crate) leitura: wgpu::BindGroupLayout,
    /// O grupo `2` do CÁLCULO (o uniforme e as cinco escritas).
    escrita: wgpu::BindGroupLayout,
    /// O mesmo grupo SEM o `despacho` (os passes das células, que o lêem como argumento indirecto).
    escrita_celulas: wgpu::BindGroupLayout,
    /// O grupo `1` do CÁLCULO, vazio (ver `new`).
    vazio: wgpu::BindGroup,
    contas: wgpu::Buffer,
    contagem: wgpu::Buffer,
    arestas: wgpu::Buffer,
    copias: wgpu::Buffer,
    caixas: wgpu::Buffer,
    /// Os registos de célula (doc 121 §9.12): por cópia, por fileira de pixels, por célula, os três
    /// fundos e a regra.
    celulas_buf: wgpu::Buffer,
    /// A acumulação das células (doc 121 §9.12); a 1.ª palavra de cada pixel acaba com a cobertura
    /// que o desenho lê.
    acumula: wgpu::Buffer,
    /// Os argumentos dos despachos indirectos das células (escritos pelo `cs_soma`): `[0, 3)` por
    /// linha, `[3, 6)` por aresta, `[6, 9)` por pixel de célula.
    despacho: wgpu::Buffer,
    cap_copias: u64,
    cap_arestas: u64,
    cap_celulas: u64,
    /// O tecto do recurso, em arestas.
    tecto_arestas: u64,
    /// O tecto do recurso, em células (o do maior buffer delas, a acumulação).
    tecto_celulas: u64,
    leitura_total: wgpu::Buffer,
    estado: Arc<AtomicU8>,
    /// O maior total medido (em arestas).
    total_visto: u64,
    /// O maior total medido (em células).
    total_visto_m: u64,
    /// As cópias do último cálculo (onde está o total das células na contagem).
    ultimo_n: u32,
    /// O tecto que um gate impõe às células (`ShapePass::limita_as_celulas`) — sem ele, o do recurso.
    pub(crate) celulas_no_maximo: u64,
    /// `false` ⇒ nenhuma cópia ganha contorno (o caminho pixel a pixel, para os gates o compararem).
    pub(crate) ligado: bool,
    /// [`AREA_MINIMA_CONFORME`], ou o que um gate pediu.
    pub(crate) area_minima_conforme: f32,
    /// `PH2D_FLUID_PROFILE=1` ⇒ cada crescimento das células diz quanto passaram a ocupar (doc 121
    /// §9.12: a memória delas na cena do app é medida, não estimada).
    relata: bool,
}

/// doc 121 §9.10 — um pipeline nas duas variantes do `override TRACEJADO` (`shape.wgsl`): a ENXUTA,
/// sem o ramo do tracejado, e a COMPLETA. Inline, o ramo dobra os registos de quem o tem (iGPU:
/// fragmento `56 → 128` VGPRs, `18 → 8` ondas por SIMD) mesmo quando nenhum troço é tracejado.
pub(crate) struct Variantes<T> {
    enxuta: T,
    completa: T,
}

impl<T> Variantes<T> {
    /// As duas, criadas com as opções de cada uma ([`opcoes`]).
    pub(crate) fn cria(f: impl Fn(wgpu::PipelineCompilationOptions<'static>) -> T) -> Self {
        Self {
            enxuta: f(opcoes(false)),
            completa: f(opcoes(true)),
        }
    }

    pub(crate) fn de(&self, tracejado: bool) -> &T {
        if tracejado {
            &self.completa
        } else {
            &self.enxuta
        }
    }
}

/// As opções de compilação de uma variante: o valor do `override TRACEJADO`.
fn opcoes(tracejado: bool) -> wgpu::PipelineCompilationOptions<'static> {
    const ENXUTA: &[(&str, f64)] = &[("TRACEJADO", 0.0)];
    const COMPLETA: &[(&str, f64)] = &[("TRACEJADO", 1.0)];
    wgpu::PipelineCompilationOptions {
        constants: if tracejado { COMPLETA } else { ENXUTA },
        ..Default::default()
    }
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
            entrada(4, armazem(false), c),
            entrada(5, armazem(false), c),
            entrada(6, armazem(false), c),
            entrada(8, armazem(false), c),
            entrada(7, armazem(false), c),
        ];
        let escrita = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita)"),
            entries: &escritas,
        });
        // ⚠️ doc 121 §9.7 — os passes das células têm o grupo SEM o `despacho` (ligação `7`, a ÚLTIMA
        // da lista): ele é o argumento do despacho indirecto deles, e o wgpu recusa o mesmo buffer como
        // escrita e como argumento no mesmo despacho (usos exclusivos no mesmo âmbito).
        let escrita_celulas = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ph2d-shape-gpu contorno (escrita, celulas)"),
            entries: &escritas[..escritas.len() - 1],
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
        let compila =
            |entry: &str,
             pl: &wgpu::PipelineLayout,
             compilation_options: wgpu::PipelineCompilationOptions<'_>| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ph2d-shape-gpu contorno"),
                    layout: Some(pl),
                    module,
                    entry_point: Some(entry),
                    compilation_options,
                    cache: None,
                })
            };
        let pipeline_em = |entry: &str, pl: &wgpu::PipelineLayout| {
            compila(entry, pl, wgpu::PipelineCompilationOptions::default())
        };
        let pipeline = |entry: &str| pipeline_em(entry, &pl);
        let variantes = |entry: &str| Variantes::cria(|o| compila(entry, &pl, o));
        let tecto_arestas = device.limits().max_storage_buffer_binding_size / ARESTA;
        let tecto_celulas = device.limits().max_storage_buffer_binding_size / ACUMULA;
        let armazens = wgpu::BufferUsages::STORAGE;
        Self {
            conta: variantes("cs_conta"),
            soma: pipeline("cs_soma"),
            escreve: variantes("cs_escreve"),
            zera: pipeline_em("cs_zera", &pl_celulas),
            deposita: pipeline_em("cs_deposita", &pl_celulas),
            fundo: pipeline_em("cs_fundo", &pl_celulas),
            varre: pipeline_em("cs_varre", &pl_celulas),
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
            copias: buffer(gpu, "ph2d-shape-gpu copias do contorno", 16, armazens),
            caixas: buffer(gpu, "ph2d-shape-gpu caixas do contorno", 16, armazens),
            celulas_buf: buffer(gpu, "ph2d-shape-gpu celulas do contorno", 16, armazens),
            acumula: buffer(gpu, "ph2d-shape-gpu acumulacao das celulas", 16, armazens),
            despacho: buffer(
                gpu,
                "ph2d-shape-gpu despacho das celulas",
                36,
                armazens | wgpu::BufferUsages::INDIRECT,
            ),
            cap_copias: 0,
            cap_arestas: 0,
            cap_celulas: 0,
            tecto_arestas,
            tecto_celulas,
            leitura_total: buffer(
                gpu,
                "ph2d-shape-gpu total do contorno",
                16,
                wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            ),
            estado: Arc::new(AtomicU8::new(LIVRE)),
            total_visto: 0,
            total_visto_m: 0,
            ultimo_n: 0,
            celulas_no_maximo: u64::MAX,
            ligado: true,
            area_minima_conforme: std::env::var("PH2D_AREA_MINIMA_CONFORME")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(AREA_MINIMA_CONFORME),
            relata: std::env::var("PH2D_FLUID_PROFILE").is_ok_and(|v| v != "0"),
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
            // Três terços de `n + 1`: as arestas, as células e as linhas de ecrã.
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
            self.cap_arestas = cap;
        }
        let tecto_m = self.tecto_celulas.min(self.celulas_no_maximo);
        let pedido_m = self.total_visto_m.min(tecto_m);
        if pedido_m > self.cap_celulas {
            let cap = ao_oitavo_do_degrau(pedido_m).min(tecto_m);
            let armazens = wgpu::BufferUsages::STORAGE;
            self.celulas_buf = buffer(gpu, "ph2d-shape-gpu celulas do contorno", cap * REGISTO, armazens);
            self.acumula = buffer(gpu, "ph2d-shape-gpu acumulacao das celulas", cap * ACUMULA, armazens);
            self.cap_celulas = cap;
            if self.relata {
                let mb = cap * (REGISTO + ACUMULA) / (1024 * 1024);
                eprintln!(
                    "[formas] celulas: capacidade {cap} ({mb} MB) para {pedido_m} pedidas por {n} copias"
                );
            }
        }
    }

    /// Codifica os quatro passes para `count` cópias (o grupo `0` é o do desenho), na variante do
    /// eixo carregado (`tracejado`, [`Variantes`]).
    pub(crate) fn calcula(
        &mut self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        grupo0: &wgpu::BindGroup,
        count: u32,
        tracejado: bool,
    ) {
        self.colhe();
        self.garante(gpu, u64::from(count));
        let contas = Contas {
            n: count,
            cap: u32::try_from(self.cap_arestas).unwrap_or(u32::MAX),
            sem_contorno: u32::from(!self.ligado),
            cap_celulas: u32::try_from(self.cap_celulas).unwrap_or(u32::MAX),
            area_minima_conforme: self.area_minima_conforme,
            _p: [0; 3],
        };
        self.ultimo_n = count;
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
                binding: 4,
                resource: self.copias.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: self.caixas.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: self.celulas_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: self.acumula.as_entire_binding(),
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
        // Três passes de cálculo, cada um com o seu relógio no perfilador (`PH2D_FLUID_PROFILE=1`):
        // a contagem, a escrita das arestas (um fio por CÓPIA) e as células.
        let passe = |encoder: &mut wgpu::CommandEncoder, relogio: &'static str| {
            let mut pass = encoder
                .begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(relogio),
                    timestamp_writes: ph2d_gpu::pass_profiler::compute_writes(relogio),
                })
                .forget_lifetime();
            pass.set_bind_group(0, grupo0, &[]);
            pass.set_bind_group(1, &self.vazio, &[]);
            pass.set_bind_group(2, &escrita, &[]);
            pass
        };
        {
            let mut pass = passe(encoder, "render.contorno.conta");
            pass.set_pipeline(self.conta.de(tracejado));
            pass.dispatch_workgroups(x, y, 1);
            pass.set_pipeline(&self.soma);
            pass.dispatch_workgroups(1, 1, 1);
        }
        {
            let mut pass = passe(encoder, "render.contorno.escreve");
            pass.set_pipeline(self.escreve.de(tracejado));
            pass.dispatch_workgroups(x, y, 1);
        }
        {
            let mut pass = passe(encoder, "render.contorno.celulas");
            // ⭐ doc 121 §9.12 — as células por acumulação: os números de linhas, de arestas e de
            // células só existem na placa (o `cs_soma` escreve os grupos), logo os despachos são
            // INDIRECTOS.
            pass.set_bind_group(2, &celulas, &[]);
            for (pipeline, args) in [
                (&self.zera, POR_PIXEL),
                (&self.deposita, POR_ARESTA),
                (&self.fundo, POR_LINHA),
                (&self.varre, POR_PIXEL),
            ] {
                pass.set_pipeline(pipeline);
                pass.dispatch_workgroups_indirect(&self.despacho, args);
            }
        }
        if self.estado.load(Ordering::Acquire) == LIVRE {
            // Os dois totais: as arestas em `n` e as células em `2n + 1`.
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

    /// **Quantas células o último cálculo PEDIU**, e a capacidade delas — lido de volta, bloqueando
    /// (doc 121 §9.12). Pedido acima da capacidade ⇒ alguma cópia não coube e foi desenhada pelo
    /// caminho de sempre. Instrumento de gates e sondas.
    pub(crate) fn celulas_do_ultimo_quadro(&self, gpu: &GpuContext) -> (u64, u64) {
        let leitura = buffer(
            gpu,
            "ph2d-shape-gpu celulas (sonda)",
            16,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        enc.copy_buffer_to_buffer(
            &self.contagem,
            (2 * u64::from(self.ultimo_n) + 1) * 4,
            &leitura,
            0,
            4,
        );
        gpu.queue.submit([enc.finish()]);
        leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let dados = leitura.slice(..).get_mapped_range();
        let pedido: u32 = bytemuck::pod_read_unaligned(&dados[..4]);
        (u64::from(pedido), self.cap_celulas)
    }

    /// O grupo `1` do desenho: as leituras do que os passes escreveram.
    pub(crate) fn grupo_de_leitura(&self, gpu: &GpuContext) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-shape-gpu contorno (leitura)"),
            layout: &self.leitura,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.copias.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.caixas.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.acumula.as_entire_binding(),
                },
            ],
        })
    }
}

#[cfg(test)]
#[path = "contorno_tests.rs"]
mod tests;
