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
    /// ⭐ Quanto da lâmpada chega a cada pixel — a [`ph2d_field_render::Shadows`] da CPU.
    pub shadow: Vec<f32>,
    /// ⭐ Quanto do céu chega a cada pixel — a oclusão.
    pub ambient: Vec<f32>,
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
    /// A lâmpada, no MUNDO. A sombra usa a mesma cerca da CPU: a distância à luz, cortada na saída
    /// da bola que contém a peça.
    pub lamp: [f32; 3],
    /// O raio da bola que contém a peça, e o centro dela.
    pub ball_center: [f32; 3],
    pub ball_radius: f32,
    /// ⭐ Quantos raios de oclusão por pixel — o [`ph2d_field_render::OCCLUSION_PASSES`].
    pub ao_rays: u32,
    /// O alcance da oclusão em unidades de mundo.
    pub ao_reach: f32,
    /// O cosseno abaixo do qual duas normais vizinhas são ARESTA — o `EDGE_COS` da CPU.
    /// ⭐⭐⭐ **A bandeira da W73 — *grosso a mexer, nítido ao assentar*.**
    ///
    /// `false` **salta o segundo despacho inteiro** (a borda re-amostrada) e devolve a lista de
    /// bordas vazia, que é exactamente o que o [`ph2d_field_render::trace_cancellable`] faz na CPU
    /// com o mesmo `antialias`. ⛔ Sem isto, mandar o quadro de MOVIMENTO ao dispositivo punha-o a
    /// pagar um passe que a lei do módulo manda não pagar — *dois motores, uma lei*.
    pub antialias: bool,
    pub edge_cos: f32,
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
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("marcha do campo"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
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
        setup: MarchSetup,
        width: u32,
        height: u32,
    ) -> DeviceGbuffer {
        match marcha_com(
            &self.device,
            &self.queue,
            &mut self.cache,
            fita,
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
    Some(Tracer::new()?.frame(fita, setup, width, height))
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

    let p_centro = cache
        .entry_with_layout(device, &molde(), fita, "centro_e_luz", Some(&layout))
        .clone();
    // ⚠️ **Compilar é o caro** — o pipeline da borda só nasce quando ela vai de facto correr.
    let p_bordas = setup.antialias.then(|| {
        cache
            .entry_with_layout(device, &molde(), fita, "bordas", Some(&layout))
            .clone()
    });

    // O uniforme, campo a campo — a mesma ordem da `struct Setup`. ⚠️ Um `vec3` alinha a 16 B.
    let mut u: Vec<u8> = Vec::with_capacity(160);
    for v in [width, height, setup.budget, setup.ao_rays] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    for f in [
        setup.half_extent,
        setup.half_px,
        setup.ortho_start,
        setup.eye_distance,
        setup.hit_eps,
        setup.normal_eps,
        setup.step,
        setup.t_max,
        setup.ball_radius,
        setup.ao_reach,
        setup.edge_cos,
        0.0,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    for v in [
        setup.target,
        setup.right,
        setup.up,
        setup.fwd,
        setup.lamp,
        setup.ball_center,
    ] {
        for f in v {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes()); // o padding do `vec3`
    }

    use wgpu::util::DeviceExt;
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("setup"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    // ⭐⭐⭐ **UM vector de constantes para os DOIS passes.** A fita da peça ocupa o princípio; a lei
    // do dono escreve a seguir, e a origem dela é **exactamente** `fita.consts.len()`.
    //
    // ⚠️⚠️ **É por isso que quem a EMITE é este sítio e não o chamador:** a origem que o texto
    // indexa e a ordem com que os vectores se concatenam são a MESMA decisão, e duas respostas
    // pintam cada folha com os números da vizinha **sem erro nenhum**.
    let lei_do_dono = pintor.and_then(|p| p.owners?.to_wgsl(fita.consts.len()));
    let mut consts = fita.consts.clone();
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
    let b_centro = cria("centro", n * 16);
    let b_luz = cria("luz", n * 8);
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
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: kb.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: b_centro.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: b_luz.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: b_conta.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: b_borda.as_entire_binding(),
                },
            ],
        })
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
            Some(ler(&mut enc, &b_luz, n * 8)),
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
                    bgl: &bgl,
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

    let f4 = |q: &[u8; 16], o: usize| f32::from_le_bytes([q[o], q[o + 1], q[o + 2], q[o + 3]]);
    let mut t = Vec::with_capacity(d_centro.len() / 16);
    let mut normal = Vec::with_capacity(t.capacity());
    for q in d_centro.as_chunks::<16>().0 {
        t.push(f4(q, 0));
        normal.push([f4(q, 4), f4(q, 8), f4(q, 12)]);
    }
    let mut shadow = Vec::with_capacity(t.len());
    let mut ambient = Vec::with_capacity(t.len());
    for q in d_luz.as_chunks::<8>().0 {
        shadow.push(f32::from_le_bytes([q[0], q[1], q[2], q[3]]));
        ambient.push(f32::from_le_bytes([q[4], q[5], q[6], q[7]]));
    }
    let vazio: [u8; 0] = [];
    let quads = d_borda.as_deref().unwrap_or(&vazio).as_chunks::<16>().0;
    #[allow(clippy::cast_possible_truncation)]
    let usadas = usadas as usize;
    let mut edges = Vec::with_capacity(usadas);
    for slot in 0..usadas.min(quads.len() / 5) {
        let cabeca = &quads[slot * 5];
        let pixel = u32::from_le_bytes([cabeca[0], cabeca[1], cabeca[2], cabeca[3]]);
        let mut hit = [false; 4];
        let mut nrm = [[0.0f32; 3]; 4];
        for j in 0..4 {
            let q = &quads[slot * 5 + 1 + j];
            hit[j] = f4(q, 0) >= 0.0;
            nrm[j] = [f4(q, 4), f4(q, 8), f4(q, 12)];
        }
        edges.push(DeviceEdge {
            pixel,
            hit,
            normal: nrm,
        });
    }
    // ⚠️ **Ordenada por pixel**, como a `Gbuffer::edges` da CPU promete — a ordem da lista aqui é
    // a de chegada dos workgroups, que é arbitrária.
    edges.sort_by_key(|e| e.pixel);

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
        ambient,
        edges,
    })
}
