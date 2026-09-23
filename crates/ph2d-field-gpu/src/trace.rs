//! ⭐⭐⭐ **A MARCHA NO DISPOSITIVO** — o G-buffer do quadro, feito na GPU.
//!
//! # O que vai e o que fica
//!
//! Medido (`docs/Render3d/05` §34), o quadro assente a `1920×1080` reparte-se assim:
//!
//! | | CPU | |
//! |---|---:|---|
//! | traçado | `30,04 ms` | ⇒ **vai** |
//! | sombra directa | `61,73 ms` | ⇒ **vai** |
//! | **pintura** | **`10,87 ms`** | ⇒ **FICA** |
//!
//! ⭐⭐ **A pintura fica, e isso é a decisão importante.** Eu supunha que ela custava `~45 ms` e
//! teria escrito o material em WGSL por causa disso — uma **segunda** implementação do OpenPBR,
//! com tudo o que este repositório sabe sobre duas respostas para a mesma pergunta. Medida, ela
//! custa `10,87 ms` e cabe. *A lei do material continua a viver num sítio só.*
//!
//! # ⚠️ A câmera é a ÚNICA coisa escrita duas vezes, e é deliberado
//!
//! O [`ph2d_field_render::march`] avisa por escrito contra *«duas respostas para «que raio sai
//! daqui?»»*. Mandar os raios prontos seriam `50 MB` por quadro, logo o WGSL reconstrói o
//! `ray_at_plane`. ⇒ ela nasce **com o gate de paridade em cima**: qualquer divergência de câmera
//! move o ponto de acerto em unidades de mundo, e o gate mede exactamente isso.

use crate::trace_leitura::lida;
use crate::trace_uniforme::uniforme_do_pedido;
use crate::trace_wgsl::molde;
use ph2d_field_eval::wgsl::TapeWgsl;

/// O que a marcha do dispositivo devolve, por pixel.
pub struct DeviceGbuffer {
    pub width: u32,
    pub height: u32,
    /// `t` do acerto, ou negativo quando o raio não acertou.
    pub t: Vec<f32>,
    /// A normal em espaço de **VISTA** — a mesma convenção do G-buffer da CPU.
    pub normal: Vec<[f32; 3]>,
    /// ⭐ Quanto de cada lâmpada chega a cada pixel — a [`ph2d_field_render::Shadows`] da CPU.
    ///
    /// ⚠️ **Uma LÂMPADA de cada vez, não um pixel de cada vez:** o bloco `l` ocupa
    /// `l * pixels .. (l+1) * pixels`, que é exactamente a forma que o `Shadows::set_lamp` recebe.
    /// *O formato que o consumidor pede é o formato que se escreve.*
    pub shadow: Vec<f32>,
    /// Quantas lâmpadas há em [`Self::shadow`].
    pub lamps: usize,
    /// ⭐ Quanto do céu chega a cada pixel — a oclusão.
    pub ambient: Vec<f32>,
    /// ⭐⭐⭐ **A luz que a CENA devolve a cada pixel** — o ricochete (`docs/Render3d/08`).
    ///
    /// ⚠️⚠️ **Por ESTE caminho ele vem a ZERO, e é um facto e não um esquecimento:** quem enche o
    /// canal é a passagem do PINTOR, que precisa da tabela de materiais — e este caminho existe
    /// justamente para quando o pintor do dispositivo não corre. *Zero é ausência de luz, que é o
    /// quadro de sempre ao bit.*
    pub bounce: Vec<[f32; 3]>,
    /// ⭐ **O chão com que este quadro foi marchado** — ver [`MarchSetup::ground`]. Ele viaja no
    /// G-buffer porque é ele que diz ao pintor da CPU (o [`DeviceGbuffer::to_cpu`]) de que altura
    /// são os canais de fundo que vêm no [`Self::shadow`].
    pub ground: Option<f32>,
    /// ⭐⭐⭐ **Os pixels de BORDA, re-amostrados no padrão 4-rook** — a `Gbuffer::edges` da CPU.
    ///
    /// ⚠️ Sem eles a silhueta sai serrilhada, e ligar o dispositivo ao produto seria trocar um
    /// defeito por outro. *É por isso que eles vêm na mesma passagem e não «depois».*
    pub edges: Vec<DeviceEdge>,
}

/// Um pixel de borda com as quatro amostras — o espelho da `ph2d_field_render::EdgePixel`.
#[derive(Clone, Copy, Debug)]
pub struct DeviceEdge {
    pub pixel: u32,
    pub hit: [bool; 4],
    pub normal: [[f32; 3]; 4],
}

impl DeviceGbuffer {
    #[must_use]
    pub fn hit(&self, i: usize) -> bool {
        self.t.get(i).is_some_and(|t| *t >= 0.0)
    }
}

pub use crate::trace_lampadas::{MAX_LAMPS, lamps_that_fit};

/// Tudo o que a marcha precisa de saber e que **não** sai da fita — os mesmos números que a
/// [`ph2d_field_render::Scene`] carrega.
#[derive(Clone, Copy, Debug)]
pub struct MarchSetup {
    pub half_extent: f32,
    /// `min(w, h) * 0.5` — o `half` do [`ph2d_field_render::Screen`].
    pub half_px: f32,
    pub target: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    /// O `toward_eye` da base. ⚠️ A direcção do raio na paralela é **`-fwd`**.
    pub fwd: [f32; 3],
    /// `ORTHO_START`, ou a distância ao olho quando há lente.
    pub ortho_start: f32,
    /// `0` = paralela. Caso contrário, a distância do olho ao alvo.
    pub eye_distance: f32,
    pub hit_eps: f32,
    pub normal_eps: f32,
    pub step: f32,
    pub budget: u32,
    pub t_max: f32,
    /// ⭐⭐⭐ **AS LÂMPADAS, no MUNDO** — `n_lamps` entradas válidas. A sombra de cada uma usa a
    /// mesma cerca da CPU: a distância à luz, cortada na saída da bola que contém a peça.
    pub lamps: [[f32; 3]; MAX_LAMPS],
    /// Quantas entradas de [`Self::lamps`] valem. ⛔ Acima de [`MAX_LAMPS`] o chamador cai na CPU.
    pub n_lamps: u32,
    /// O raio da bola que contém a peça, e o centro dela.
    pub ball_center: [f32; 3],
    pub ball_radius: f32,
    /// ⭐ Quantos raios de oclusão por pixel — o [`ph2d_field_render::OCCLUSION_PASSES`].
    pub ao_rays: u32,
    /// O alcance da oclusão em unidades de mundo.
    pub ao_reach: f32,
    /// ⭐⭐⭐ **O CHÃO QUE SÓ RECEBE** (`docs/Render3d/07`) — a altura dele no MUNDO, ou `None`.
    ///
    /// Com ele, um pixel que **falha** a peça e vê o chão guarda nos canais de luz a sombra e o céu
    /// que chegam **ao chão**, e o pintor escurece o fundo por essa razão. ⚠️ `None` é o caminho de
    /// sempre, ao bit: os canais de um pixel de fundo ficam todos a `1,0`.
    pub ground: Option<f32>,
    /// O cosseno abaixo do qual duas normais vizinhas são ARESTA — o `EDGE_COS` da CPU.
    /// ⭐⭐⭐ **A bandeira da W73 — *grosso a mexer, nítido ao assentar*.**
    ///
    /// `false` **salta o segundo despacho inteiro** (a borda re-amostrada) e devolve a lista de
    /// bordas vazia, que é exactamente o que o [`ph2d_field_render::trace_cancellable`] faz na CPU
    /// com o mesmo `antialias`. ⛔ Sem isto, mandar o quadro de MOVIMENTO ao dispositivo punha-o a
    /// pagar um passe que a lei do módulo manda não pagar — *dois motores, uma lei*.
    pub antialias: bool,
    pub edge_cos: f32,
    /// ⭐⭐⭐ **A BORDA MOLE DA SOMBRA** (`docs/Render3d/10` §12) — o raio por CANAL, em PÍXEIS de
    /// ecrã, ou `None`.
    ///
    /// Com ele, o passe que pinta corre as duas passagens separáveis do
    /// [`ph2d_field_render::sss_shadow`] sobre a visibilidade de cada lâmpada, e a closure de
    /// subsuperfície lê a MÉDIA da vizinhança em vez da visibilidade dura deste pixel. ⚠️ `None` é
    /// o caminho de sempre **ao bit**: o passo do buffer não cresce e o pintor sai pelo braço curto
    /// do `mx_direct_sss`.
    ///
    /// ⛔⛔ **Ele é UM raio para o quadro inteiro, e o chamador RECUSA quando a cena tem dois.** A
    /// lei da CPU escolhe o raio **por material** ([`ph2d_field_render::sss_shadow::blur_por_material`]),
    /// e a selecção por pixel pede o dono do ponto dentro do borrão — que arrastaria o campo e a
    /// tabela de donos para um passe que só precisa da normal. *Com dois raios o chamador cai na
    /// CPU, que tem a lei inteira: nenhuma imagem errada, em sítio nenhum.*
    pub mole: Option<[f32; 3]>,
}

/// ⭐⭐⭐ **O TRAÇADOR: o dispositivo, o cache de pipelines e o layout, vivos entre quadros.**
///
/// ⛔⛔ **Ele existe porque a 1.ª sonda mediu a coisa errada.** A [`march`] abre o adaptador, pede
/// o dispositivo e **compila o shader** a cada chamada — e o relógio dela leu **`130 ms` a
/// `640×360`**, isto é, *mais lento que a CPU*. O que ela media era a abertura do dispositivo e a
/// compilação (`6`–`49 ms`, §33), não o quadro.
///
/// ⚠️ *Uma sonda cujo próprio doc diz «é a forma de sonda» ainda assim foi usada como relógio* — e
/// o número saiu convincente e ao contrário.
pub struct Tracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    cache: crate::FieldPipelines,
}

impl Tracer {
    /// `None` sem adaptador.
    #[must_use]
    pub fn new() -> Option<Self> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        // ⭐⭐⭐ **OS LIMITES SÃO OS DA PLACA, e não os mínimos da `wgpu`.**
        //
        // ⛔⛔ O `Limits::default()` é o **piso garantido** da especificação (pensado para a Web), e
        // pedi-lo trava esta máquina no tecto de uma que não é esta. Medido: ele dá
        // `max_storage_buffers_per_shader_stage = 8`, e o passe que pinta com ESCULTURA precisa de
        // `9` — a `wgpu` recusou a criar o layout, numa placa que suporta muito mais.
        //
        // ⚠️ *Nunca deixe o caminho mais lento definir o tecto do mais rápido* (`CLAUDE.md` §0.0).
        // Quem tiver uma placa mais fraca é servido pela mesma linha: ela pede o que a placa TEM.
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("marcha do campo"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        Some(Self {
            device,
            queue,
            cache: crate::FieldPipelines::new(),
        })
    }

    /// Quantos pipelines já foram compilados — o número que um gate de *«um arrasto não
    /// recompila»* observa.
    #[must_use]
    pub fn compiled(&self) -> usize {
        self.cache.compiled()
    }

    /// ⭐ **Um quadro, devolvido como G-BUFFER.** O shader compila-se na primeira estrutura e fica.
    ///
    /// ⚠️ É a porta da PARIDADE e do caminho que ainda pinta na CPU. Quem quer a imagem chama o
    /// [`Self::painted_frame`], que não traz o G-buffer de volta.
    pub fn frame(
        &mut self,
        fita: &TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: MarchSetup,
        width: u32,
        height: u32,
    ) -> DeviceGbuffer {
        match marcha_com(
            &self.device,
            &self.queue,
            &mut self.cache,
            fita,
            sculpts,
            setup,
            width,
            height,
            None,
        ) {
            Saida::Gbuffer(g) => g,
            Saida::Imagem(_) => unreachable!("sem pintor a marcha devolve o G-buffer"),
        }
    }

    /// ⭐⭐⭐ **Um quadro, devolvido como IMAGEM** — RGBA8 pré-multiplicado, pronto para a tela.
    ///
    /// ⛔⛔ **E é aqui que o barramento encolhe:** a `frame` traz `49,8 MB` a `1920×1080` (o centro
    /// e a luz de cada pixel) para a CPU os transformar em `8,3` de imagem. Esta traz os `8,3`, e a
    /// transformação corre onde os dados estão.
    pub fn painted_frame(
        &mut self,
        fita: &TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: MarchSetup,
        pintor: &crate::paint::PaintSetup<'_>,
        width: u32,
        height: u32,
    ) -> Pintado {
        match marcha_com(
            &self.device,
            &self.queue,
            &mut self.cache,
            fita,
            sculpts,
            setup,
            width,
            height,
            Some(pintor),
        ) {
            Saida::Imagem(p) => p,
            Saida::Gbuffer(_) => unreachable!("com pintor a marcha devolve a imagem"),
        }
    }
    /// O dispositivo e a fila — para quem precisa de despachar **outro** passe sobre o MESMO
    /// dispositivo (o arnês de paridade do material, e o passe de sombreamento).
    ///
    /// ⚠️ **Abrir um segundo dispositivo custaria mais do que o trabalho** (medido: `130 ms` para
    /// abrir e compilar, contra `13` da CPU inteira) — e um `Device` não fala com buffers de outro.
    #[must_use]
    pub fn parts(&self) -> (&wgpu::Device, &wgpu::Queue) {
        (&self.device, &self.queue)
    }

    /// ⭐ **O maior buffer que esta placa deixa LIGAR a um shader** — a entrada do
    /// [`lamps_that_fit`].
    ///
    /// ⚠️ **Perguntado à placa e não escrito à mão:** a `wgpu` garante `128 MiB` como mínimo, e uma
    /// placa que ofereça mais fica com mais lâmpadas sem ninguém mexer num número.
    #[must_use]
    pub fn binding_limit(&self) -> u64 {
        self.device.limits().max_storage_buffer_binding_size
    }

    /// ⭐ **Quantos armazéns esta placa deixa um shader ligar de uma vez.**
    ///
    /// ⚠️ **O passe que PINTA precisa de `9`** (seis do grupo `0` e três do grupo `1`), e o piso
    /// garantido da `wgpu` é `8`. Numa placa que fique no piso o quadro cai na CPU — que é a mesma
    /// lei das lâmpadas e da escultura sem grade: *recusar em voz alta em vez de desenhar metade*.
    /// Quantas vezes as grades das esculturas subiram à placa — ver
    /// [`crate::FieldPipelines::grades_enviadas`].
    #[must_use]
    pub fn grades_enviadas(&self) -> usize {
        self.cache.grades_enviadas()
    }

    #[must_use]
    pub fn storage_slots(&self) -> u32 {
        self.device.limits().max_storage_buffers_per_shader_stage
    }

    /// ⭐ **Quanto custa TRAZER `bytes` de volta** — a fase que o `submit` do quadro esconde.
    ///
    /// ⚠️ **É diagnóstico, e não o caminho do produto:** o quadro copia e lê no MESMO `submit` que
    /// despacha o compute, logo um relógio à volta dele mede os dois juntos. Esta porta mede só a
    /// travessia, sobre o mesmo volume de bytes, e é assim que a sonda `frame_budget` separa as
    /// fases sem tocar no que shipa.
    #[must_use]
    pub fn mede_leitura(&self, bytes: u64, corridas: usize) -> f64 {
        let bytes = bytes.max(16);
        let origem = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("origem"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut melhor = f64::INFINITY;
        for _ in 0..corridas {
            let t = std::time::Instant::now();
            let destino = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("leitura"),
                size: bytes,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut enc = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            enc.copy_buffer_to_buffer(&origem, 0, &destino, 0, bytes);
            self.queue.submit([enc.finish()]);
            destino.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            self.device.poll(wgpu::PollType::wait_indefinitely()).ok();
            let dados = destino.slice(..).get_mapped_range();
            std::hint::black_box(dados[0]);
            drop(dados);
            melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
        }
        melhor
    }
}

/// A forma de **SONDA**: abre o dispositivo, corre um quadro e fecha.
///
/// ⛔ **Não a use para medir tempo** — ver [`Tracer`].
#[must_use]
pub fn march(fita: &TapeWgsl, setup: MarchSetup, width: u32, height: u32) -> Option<DeviceGbuffer> {
    Some(Tracer::new()?.frame(fita, &[], setup, width, height))
}

/// O que a marcha entrega — o G-buffer, ou a imagem quando o pintor corre.
///
/// ⚠️ **As duas nunca voltam juntas, e é isso que ela codifica:** com o pintor a correr o G-buffer
/// fica no dispositivo, e trazê-lo «ao lado» seria pagar os `49,8 MB` que este passe existe para
/// não pagar.
pub(crate) enum Saida {
    Gbuffer(DeviceGbuffer),
    Imagem(Pintado),
}

/// ⭐ **O que o pintor entrega** — a imagem, mais a contagem de bordas que o dispositivo escreveu.
///
/// ⚠️ **A contagem vem junto porque ela já voltou**: o número de bordas atravessa o barramento
/// antes do passe que pinta (é ele que diz quantos workgroups despachar). Derivá-la outra vez da
/// imagem seria inventar uma segunda resposta para um facto que já está na mão.
pub struct Pintado {
    /// RGBA8 pré-multiplicado, pronto para a tela.
    pub rgba: Vec<u8>,
    /// Quantos pixels de borda foram re-amostrados — `0` sem anti-serrilhado.
    pub edges: usize,
}

/// ⭐ **Uma entrada de layout, uniforme** — partilhada pelo traçado e pelo pintor.
pub(crate) fn uniforme(b: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// ⭐ **Uma entrada de layout, armazém** — `so_leitura` distingue `read` de `read_write`.
pub(crate) fn armazem(b: u32, so_leitura: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage {
                read_only: so_leitura,
            },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// ⭐⭐⭐ **O layout do grupo `0`** — o uniforme, as constantes e os quatro alvos da marcha.
///
/// ⛔⛔ **Ele é EXPLÍCITO e tem de ser** (ver [`crate::FieldPipelines::entry_with_layout`]): o
/// layout auto-derivado só declara os bindings que **aquela entrada** usa, e a passagem do centro
/// não toca na lista de bordas — o grupo de seis seria recusado em tempo de execução.
///
/// ⚠️ **E o pintor lê o MESMO grupo** ([`crate::paint`]): ele precisa do centro, da luz e da lista
/// de bordas, e uma segunda declaração deles seria a segunda resposta à mesma pergunta.
pub(crate) fn bgl_marcha(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("campo"),
        entries: &[
            uniforme(0),
            armazem(1, true),
            armazem(2, false),
            armazem(3, false),
            armazem(4, false),
            armazem(5, false),
            armazem(6, true),
        ],
    })
}

// O dispositivo, a fila, o cache, a fita, o pedido, a tela e o pintor — sete coisas
// independentes, e uma struct só as renomearia. E o corpo é longo porque são seis bindings,
// dois despachos e duas travessias do barramento.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn marcha_com(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &mut crate::FieldPipelines,
    fita: &TapeWgsl,
    sculpts: &[ph2d_field_eval::device::DeviceSculpt],
    setup: MarchSetup,
    width: u32,
    height: u32,
    pintor: Option<&crate::paint::PaintSetup<'_>>,
) -> Saida {
    let bgl = bgl_marcha(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("campo"),
        bind_group_layouts: &[Some(&bgl)],
        immediate_size: 0,
    });

    // ⭐⭐⭐ **A ORDEM DO `k` É O CONTRATO**, e ela é uma só: a fita da peça, depois os cabeçalhos
    // das esculturas, depois a lei do dono. Cada emissor recebe a origem dele **desta** aritmética,
    // e é por isso que ela vive aqui e não em três sítios.
    let escultura = crate::sculpt::emit(sculpts, fita.consts.len());
    let esculturas = escultura.as_ref().map_or("", |e| e.source.as_str());
    let molde_com_esculturas = molde().replace("{ESCULTURAS}", esculturas);
    // ⭐ **As mesmas leis, para o pintor** (`docs/Render3d/08` §12) — ele marcha o ricochete, e
    // marchar é isto. ⚠️ Elas saem da MESMA substituição: uma segunda chamada ao
    // `crate::sculpt::emit` daria outra aritmética de origens para o mesmo `k`.
    let leis_com_esculturas = crate::trace_wgsl::leis().replace("{ESCULTURAS}", esculturas);
    let p_centro = cache
        .entry_with_layout(
            device,
            &molde_com_esculturas,
            fita,
            "centro_e_luz",
            Some(&layout),
        )
        .clone();
    // ⚠️ **Compilar é o caro** — o pipeline da borda só nasce quando ela vai de facto correr.
    let p_bordas = setup.antialias.then(|| {
        cache
            .entry_with_layout(device, &molde_com_esculturas, fita, "bordas", Some(&layout))
            .clone()
    });

    use wgpu::util::DeviceExt;
    let ub = uniforme_do_pedido(device, setup, width, height);
    // ⭐⭐⭐ **UM vector de constantes para os DOIS passes.** A fita da peça ocupa o princípio; a lei
    // do dono escreve a seguir, e a origem dela é **exactamente** `fita.consts.len()`.
    //
    // ⚠️⚠️ **É por isso que quem a EMITE é este sítio e não o chamador:** a origem que o texto
    // indexa e a ordem com que os vectores se concatenam são a MESMA decisão, e duas respostas
    // pintam cada folha com os números da vizinha **sem erro nenhum**.
    let mut consts = fita.consts.clone();
    if let Some(e) = &escultura {
        consts.extend_from_slice(&e.consts);
    }
    let lei_do_dono = pintor.and_then(|p| p.owners?.to_wgsl(consts.len()));
    if let Some(l) = &lei_do_dono {
        consts.extend_from_slice(&l.consts);
    }
    if consts.is_empty() {
        consts.push(0.0);
    }
    let mut kb_bytes = Vec::with_capacity(consts.len() * 4);
    for c in &consts {
        kb_bytes.extend_from_slice(&c.to_le_bytes());
    }
    let kb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("k"),
        contents: &kb_bytes,
        usage: wgpu::BufferUsages::STORAGE,
    });
    let n = u64::from(width) * u64::from(height);
    let storage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
    let cria = |nome: &str, bytes: u64| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(nome),
            size: bytes.max(16),
            usage: storage,
            mapped_at_creation: false,
        })
    };
    // ⭐⭐⭐ **AS GRADES SOBEM UMA VEZ** — ver [`crate::FieldPipelines::grades`]. Sem escultura é um
    // buffer mínimo, que o layout exige e o shader nunca lê.
    //
    // ⚠️ **Clonado e não emprestado:** um `wgpu::Buffer` é um punho com contagem, e segurar o
    // empréstimo do cache impediria a compilação do pipeline mais abaixo de lhe tocar.
    let b_grades = cache.grades(device, sculpts).clone();
    let b_centro = cria("centro", n * 16);
    // ⭐ O passo é `1 + n_lamps + 6`: o céu, uma visibilidade por lâmpada e o RICOCHETE
    // (`docs/Render3d/08`) — mais **SEIS por lâmpada** quando há BORDA MOLE (o intermediário da
    // passagem horizontal e o resultado, `docs/Render3d/10` §25). ⚠️ Ele é a mesma conta do
    // `passo_da_luz()` do WGSL, e as duas têm de andar juntas: um buffer curto faz o shader
    // escrever fora e a `wgpu` recusa o despacho.
    let passo_luz = u64::from(setup.n_lamps) * (1 + 6 * u64::from(setup.mole.is_some())) + 1 + 6;
    let b_luz = cria("luz", n * passo_luz * 4);
    // ⛔⛔ **O TECTO da lista de bordas era `6 %` e ESTOUROU** — o gate da paridade apanhou-o: na
    // ROSCA a GPU devolveu exactamente `1 296` bordas, que **é** o tecto, contra `1 745` da CPU, e
    // a sobreposição das listas caiu para `72,6 %`.
    //
    // ⚠️ **O `0,5`–`1,2 %` que eu citei é da SILHUETA** (`docs/3DModeling/05`), e a borda deste
    // passe é silhueta **mais VINCO**: uma peça de ranhuras finas é quase toda vinco. Medido, a
    // rosca dá `8,4 %`. ⇒ `25 %`, que é três vezes o pior medido — e o custo é `20 B` por pixel
    // (`41 MB` a `1920×1080`), que a leitura já paga em `~2 ms`.
    //
    // ⚠️ *Um tecto derivado da grandeza ERRADA lê-se como generoso.*
    let max_bordas = (n / 4).max(1024);
    let b_borda = cria("bordas", max_bordas * 5 * 16);
    let b_conta = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("conta"),
        // ⚠️ Quatro palavras e não uma: a cópia de leitura alinha a `16 B`, e um buffer de `4`
        // seria lido fora dos limites.
        contents: &[0u8; 16],
        usage: storage,
    });

    let bind = |_p: &wgpu::ComputePipeline| {
        crate::trace_grupo::grupo_da_marcha(
            device, &bgl, &ub, &kb, &b_centro, &b_luz, &b_conta, &b_borda, &b_grades,
        )
    };
    let bg_centro = bind(&p_centro);
    let bg_bordas = p_bordas.as_ref().map(&bind);

    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    // ⚠️ **DOIS despachos, e a ordem é a lei**: a borda pergunta pelos VIZINHOS, logo o centro tem
    // de estar escrito para toda a imagem antes de ela correr.
    let despachos: Vec<(&wgpu::ComputePipeline, &wgpu::BindGroup)> =
        match (p_bordas.as_ref(), bg_bordas.as_ref()) {
            (Some(p), Some(bg)) => vec![(&p_centro, &bg_centro), (p, bg)],
            _ => vec![(&p_centro, &bg_centro)],
        };
    for (p, bg) in despachos {
        let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cp.set_pipeline(p);
        cp.set_bind_group(0, bg, &[]);
        cp.dispatch_workgroups(width.div_ceil(8), height.div_ceil(8), 1);
    }

    let ler = |enc: &mut wgpu::CommandEncoder, b: &wgpu::Buffer, bytes: u64| {
        let r = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("leitura"),
            size: bytes.max(16),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        enc.copy_buffer_to_buffer(b, 0, &r, 0, bytes.max(16));
        r
    };
    // ⭐⭐⭐ **QUANDO O PINTOR CORRE, O G-BUFFER NÃO ATRAVESSA O BARRAMENTO.** Ele fica no
    // dispositivo, que é onde o passe seguinte o lê — e o que volta é a IMAGEM.
    //
    // Medido a `1920×1080`: o centro e a luz são `49,8 MB` por quadro e a imagem são `8,3`.
    let pinta = pintor.is_some();
    let (r_centro, r_luz) = if pinta {
        (None, None)
    } else {
        (
            Some(ler(&mut enc, &b_centro, n * 16)),
            Some(ler(&mut enc, &b_luz, n * passo_luz * 4)),
        )
    };
    let r_conta = ler(&mut enc, &b_conta, 16);
    queue.submit([enc.finish()]);

    for b in r_centro.iter().chain(r_luz.iter()).chain([&r_conta]) {
        b.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    }
    device.poll(wgpu::PollType::wait_indefinitely()).ok();

    let d_conta = r_conta.slice(..).get_mapped_range();
    let quantas = u32::from_le_bytes([d_conta[0], d_conta[1], d_conta[2], d_conta[3]]) as u64;
    if let Some(pintor) = pintor {
        // ⚠️ **A contagem de bordas tinha de voltar primeiro**, e é isso que este ida-e-volta
        // compra: quantos workgroups o passe da borda precisa é um número que o dispositivo
        // escreveu. *O mesmo ida-e-volta que a leitura da lista já custava, sem a lista.*
        let usadas = if setup.antialias {
            quantas.min(max_bordas)
        } else {
            0
        };
        drop(d_conta);
        #[allow(clippy::cast_possible_truncation)]
        let edges = usadas as usize;
        return Saida::Imagem(Pintado {
            edges,
            rgba: crate::paint::pinta(
                device,
                queue,
                cache,
                pintor,
                lei_do_dono.as_ref(),
                &crate::paint::Alvos {
                    leis: &leis_com_esculturas,
                    fita,
                    bgl: &bgl,
                    grades: &b_grades,
                    setup: &ub,
                    k: &kb,
                    centro: &b_centro,
                    luz: &b_luz,
                    conta: &b_conta,
                    borda: &b_borda,
                },
                width,
                height,
                usadas,
            ),
        });
    }
    let d_centro = r_centro.as_ref().expect("sem pintor o centro volta");
    let d_centro = d_centro.slice(..).get_mapped_range();
    let d_luz = r_luz.as_ref().expect("sem pintor a luz volta");
    let d_luz = d_luz.slice(..).get_mapped_range();

    // ⛔⛔ **A LISTA DE BORDAS LÊ-SE PELO QUE FOI ESCRITO, e não pelo tecto** — e é a diferença
    // entre `35 ms` e o que a máquina de facto faz. O tecto é `25 %` dos pixels (`41 MB` a
    // `1920×1080`) e a ocupação real é `1`–`8 %`: copiar o tecto inteiro a cada quadro era
    // **quase metade** dos `90 MB` de leitura. ⇒ um segundo `submit`, que custa um ida-e-volta e
    // poupa dezenas de megabytes. *Um buffer dimensionado para o pior caso não se lê no pior caso.*
    // ⚠️ **Sem anti-serrilhado não há segunda travessia nenhuma** — nem o `submit`, nem o
    // `poll`, que é um ida-e-volta completo ao dispositivo por quadro.
    let usadas = if setup.antialias {
        quantas.min(max_bordas)
    } else {
        0
    };
    let r_borda = {
        let mut enc2 =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let r = ler(&mut enc2, &b_borda, usadas * 5 * 16);
        if usadas > 0 {
            queue.submit([enc2.finish()]);
            r.slice(..).map_async(wgpu::MapMode::Read, |_| {});
            device.poll(wgpu::PollType::wait_indefinitely()).ok();
        }
        r
    };
    let d_borda = if usadas > 0 {
        Some(r_borda.slice(..).get_mapped_range())
    } else {
        None
    };

    let (t, normal, shadow, ambient, bounce, edges) =
        lida(&d_centro, &d_luz, passo_luz, usadas, d_borda.as_deref());

    drop(d_centro);
    drop(d_luz);
    drop(d_conta);
    drop(d_borda);

    Saida::Gbuffer(DeviceGbuffer {
        width,
        height,
        t,
        normal,
        shadow,
        // ⚠️ **Menos os TRÊS do ricochete** — o passo deixou de ser `1 + n_lamps`
        // (`docs/Render3d/08`), e sem este desconto os canais dele leriam-se como lâmpadas
        // fantasma. *O modo de falha foi o bom: um índice fora do `shadow`, alto e no primeiro
        // quadro.*
        #[allow(clippy::cast_possible_truncation)]
        lamps: (passo_luz - 1 - 6) as usize,
        ambient,
        bounce,
        ground: setup.ground,
        edges,
    })
}

#[cfg(test)]
mod sonda_do_cache_em_disco {
    /// ⏱️⭐⭐⭐ **A PLACA DESTA MÁQUINA OFERECE CACHE DE PIPELINES EM DISCO?**
    ///
    /// A `W9` nomeia o cache em disco como a cura do `1,3 s` que a placa paga **uma vez por
    /// sessão** a compilar o kernel do pintor (`docs/Render3d/03` §W9), e a ranhura dele
    /// (`cache: None` no [`crate::FieldPipelines::entry_with_layout`]) está vazia.
    ///
    /// ⛔⛔ **Mas o doc do `wgpu` avisa que numa plataforma de desktop isto quase não paga:** *«most
    /// desktop GPU drivers will manage their own caches, meaning that little advantage can be
    /// gained from this on those platforms»*. ⇒ *§0.0: antes de construir, meça se a porta existe.*
    ///
    /// Esta sonda imprime três coisas: se o adaptador **anuncia** a feature, a chave que ele daria
    /// a um ficheiro de cache, e o nome do backend — que é o que decide se a advertência se aplica.
    ///
    /// # ⛔⛔⛔ O veredito de VIABILIDADE (2026-09-22), e é por isso que o item DESCEU na fila
    ///
    /// | pergunta | resposta MEDIDA |
    /// |---|---|
    /// | o adaptador oferece? | **SIM** — `NVIDIA GeForce RTX 5060 Ti`, Vulkan, driver `615.71.09`, chave `wgpu_pipeline_cache_vulkan_4318_11524` |
    /// | quanto vale? | `1,3`–`2,8 s`, **uma vez por sessão** (o kernel do pintor; o resto já o paga a fita inerte) |
    /// | quanto custa construir? | uma crate/módulo **isolado com `unsafe`** — o `Device::create_pipeline_cache` é `unsafe` (o blob é entrada não confiável para o driver) e a workspace declara `unsafe_code = "forbid"`. O molde da casa é o [ADR-0116](../../../docs/architecture/decisions/0116-audio-export-opus-isolated-unsafe-crate.md): a crate desce o `forbid` a **`deny`** e só o módulo que toca a ABI o autoriza por escrito, com gate na lista. |
    /// | paga? | ⚠️ **INCERTO, e o fornecedor diz que não:** *«most desktop GPU drivers will manage their own caches, meaning that little advantage can be gained from this on those platforms»*. |
    ///
    /// ⚠️⚠️ **E a evidência desta máquina é MISTA, o que é a razão para não decidir de barato:** em
    /// duas corridas do mesmo processo o pipeline `bordas` foi de `24,70` para `0,28 ms` com o
    /// SPIR-V idêntico — o driver cacheou-o —, e a 1.ª pintura de uma peça leu `1 449` e `1 417 ms`
    /// em dois processos separados, ou seja **o cache do driver não ajudou o pipeline grande**.
    /// *É exactamente ali que a nossa ranhura poderia pagar, e é exactamente ali que não há
    /// medição.*
    ///
    /// ⛔ **A experiência de três vias que decidiria isto NÃO É ESCREVÍVEL nesta árvore** (`A` com
    /// cache vazio · `B` com o blob · `C` sem cache, que é o veredito — *se o `B` for rápido e o
    /// `C` também, quem cacheou foi o driver*): ela precisa de `unsafe`, e o `forbid` da workspace
    /// não se contorna com um `allow`. ⇒ ela é acto de uma crate irmã, no dia em que este item
    /// subir na fila.
    #[test]
    #[ignore = "sonda de diagnóstico: precisa de GPU"]
    fn sonda_a_placa_oferece_cache_de_pipelines() {
        let instance = wgpu::Instance::default();
        let Some(adapter) =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .ok()
        else {
            println!("sem adaptador — saltado");
            return;
        };
        let info = adapter.get_info();
        let tem = adapter.features().contains(wgpu::Features::PIPELINE_CACHE);
        println!("\n  adaptador ····· {} ({:?})", info.name, info.backend);
        println!("  driver ········ {} {}", info.driver, info.driver_info);
        println!("  PIPELINE_CACHE  {}", if tem { "SIM" } else { "NÃO" });
        println!(
            "  chave ········· {}\n",
            wgpu::util::pipeline_cache_key(&info).unwrap_or_else(|| "— (sem chave)".to_string())
        );
    }
}
