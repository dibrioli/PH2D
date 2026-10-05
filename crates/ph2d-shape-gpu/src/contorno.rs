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
/// doc 121 §9.13 — e os dos dois DESENHOS (`desenho` no WGSL): a variante enxuta e a completa.
const DESENHO_ENXUTA: u64 = 36;
const DESENHO_COMPLETO: u64 = 52;
const DESPACHO: u64 = 68;
/// As contagens por cópia na `contagem` (`quinto` no WGSL).
const QUINTOS: u64 = 5;

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
    /// doc 121 §9.15 — o prefixo das arestas ESCRITAS (o despacho do `cs_deposita`).
    soma_escritas: wgpu::ComputePipeline,
    /// doc 121 §9.12 — as células por acumulação, por despachos indirectos: um fio por PIXEL apaga,
    /// um por ARESTA deposita, um por LINHA faz o prefixo do fundo, um por PIXEL varre a célula.
    zera: wgpu::ComputePipeline,
    deposita: wgpu::ComputePipeline,
    fundo: wgpu::ComputePipeline,
    varre: wgpu::ComputePipeline,
    /// doc 121 §9.15 (d) — o `cs_varre` por subgrupo, onde o dispositivo tem `Features::SUBGROUP`.
    varre_sg: Option<wgpu::ComputePipeline>,
    /// `false` ⇒ o `cs_varre` de memória de grupo mesmo com o de subgrupo (os gates comparam os dois).
    pub(crate) subgrupo: bool,
    /// doc 121 §9.15 (c2) — o próximo cálculo MEDE a capacidade antes de desenhar (cena nova).
    pub(crate) medir_ja: bool,
    /// `false` ⇒ só a leitura assíncrona (os gates que medem a mistura do 1.º quadro).
    pub(crate) mede_no_inicio: bool,
    /// doc 121 §9.15 — o `cs_deposita` por aresta ESCRITA (o `override ARESTAS_COMPACTAS` do módulo:
    /// os dois mudam juntos, pelas mesmas constantes).
    compactas: bool,
    /// O grupo do `cs_conta` e do `cs_escreve` (o `override GRUPO_DO_CONTORNO`, §9.17).
    grupo: u32,
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
    /// linha, `[3, 6)` por aresta, `[6, 9)` por pixel de célula; e os dois desenhos de uma cena com
    /// tracejado (`[9, 17)`, doc 121 §9.13).
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
    /// As duas, criadas com as opções de cada uma — o `TRACEJADO` delas e as constantes `extra` (doc 121
    /// §9.16: as da sonda intercalada; vazias no produto).
    pub(crate) fn cria(
        extra: &[(&str, f64)],
        f: impl Fn(wgpu::PipelineCompilationOptions<'_>) -> T,
    ) -> Self {
        let com = |tracejado: f64| {
            let mut c: Vec<(&str, f64)> = extra.to_vec();
            c.push(("TRACEJADO", tracejado));
            c
        };
        let (e, c) = (com(0.0), com(1.0));
        Self {
            enxuta: f(opcoes(&e)),
            completa: f(opcoes(&c)),
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

/// As opções de compilação com as constantes `override` dadas.
fn opcoes<'a>(constants: &'a [(&'a str, f64)]) -> wgpu::PipelineCompilationOptions<'a> {
    wgpu::PipelineCompilationOptions {
        constants,
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
        modulo_subgrupo: Option<&wgpu::ShaderModule>,
        grupo0: &wgpu::BindGroupLayout,
        constantes: &[(&str, f64)],
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
        let pipeline_em =
            |entry: &str, pl: &wgpu::PipelineLayout| compila(entry, pl, opcoes(constantes));
        let pipeline = |entry: &str| pipeline_em(entry, &pl);
        let variantes = |entry: &str| Variantes::cria(constantes, |o| compila(entry, &pl, o));
        let tecto_arestas = device.limits().max_storage_buffer_binding_size / ARESTA;
        let tecto_celulas = device.limits().max_storage_buffer_binding_size / ACUMULA;
        let armazens = wgpu::BufferUsages::STORAGE;
        Self {
            conta: variantes("cs_conta"),
            soma: pipeline("cs_soma"),
            escreve: variantes("cs_escreve"),
            soma_escritas: pipeline("cs_soma_escritas"),
            zera: pipeline_em("cs_zera", &pl_celulas),
            deposita: pipeline_em("cs_deposita", &pl_celulas),
            fundo: pipeline_em("cs_fundo", &pl_celulas),
            varre: pipeline_em("cs_varre", &pl_celulas),
            varre_sg: modulo_subgrupo.map(|m| {
                device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("ph2d-shape-gpu contorno (subgrupo)"),
                    layout: Some(&pl_celulas),
                    module: m,
                    entry_point: Some("cs_varre_sg"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                })
            }),
            subgrupo: true,
            medir_ja: true,
            mede_no_inicio: true,
            compactas: constantes
                .iter()
                .find(|(k, _)| *k == "ARESTAS_COMPACTAS")
                .is_none_or(|(_, v)| *v != 0.0),
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "um grupo"
            )]
            grupo: constantes
                .iter()
                .find(|(k, _)| *k == "GRUPO_DO_CONTORNO")
                .map_or(64, |(_, v)| *v as u32),
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
                DESPACHO,
                armazens | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
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

    /// O `cs_varre` deste quadro: o de subgrupo, se existe e nenhum gate o desligou (§9.15 d).
    fn varre_do_quadro(&self) -> &wgpu::ComputePipeline {
        self.varre_sg
            .as_ref()
            .filter(|_| self.subgrupo)
            .unwrap_or(&self.varre)
    }

    /// O uniforme deste cálculo e os dois grupos `2` — o da escrita e o das células (sem o `despacho`).
    fn prepara(&mut self, gpu: &GpuContext, count: u32) -> (wgpu::BindGroup, wgpu::BindGroup) {
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
        (escrita, celulas)
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
        if std::mem::take(&mut self.medir_ja) && self.ligado && self.mede_no_inicio {
            self.mede_a_capacidade(gpu, grupo0, count, tracejado);
            self.garante(gpu, u64::from(count));
        }
        let (escrita, celulas) = self.prepara(gpu, count);
        let grupos = count.div_ceil(self.grupo);
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
            if self.compactas {
                pass.set_pipeline(&self.soma_escritas);
                pass.dispatch_workgroups(1, 1, 1);
            }
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
                (self.varre_do_quadro(), POR_PIXEL),
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

    /// doc 121 §9.13 — **o desenho de uma cena com tracejado**: as duas variantes, cada uma com as
    /// cópias que o `cs_escreve` lhe deu (todas numa, nenhuma na outra) — uma só desenha, e a ordem da
    /// mistura é a de uma chamada. Os grupos de ligação já postos servem às duas.
    pub(crate) fn desenha_pela_variante_da_placa(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        pipeline: &Variantes<wgpu::RenderPipeline>,
    ) {
        pass.set_pipeline(pipeline.de(false));
        pass.draw_indirect(&self.despacho, DESENHO_ENXUTA);
        pass.set_pipeline(pipeline.de(true));
        pass.draw_indirect(&self.despacho, DESENHO_COMPLETO);
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

#[path = "contorno_sondas.rs"]
mod sondas;

#[path = "contorno_capacidade.rs"]
mod capacidade;

#[cfg(test)]
#[path = "contorno_tests.rs"]
mod tests;
