//! ⭐⭐⭐ **A MARCHA NO DISPOSITIVO** — o G-buffer do quadro, feito na GPU.
//!
//! Ela serve o MATCAP (o pintor de [`crate::matcap`] lê o `centro` e as bordas sem os trazer à
//! CPU) e a leitura de volta que as paridades com a CPU medem ([`Tracer::frame`]). ⚠️ A luz, o céu,
//! o chão e o pintor de material do Render traçado viviam aqui e saíram em 03/10: o modo Render
//! desenha por malha (`ph2d-mesh-forward`).
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
    /// O cosseno abaixo do qual duas normais vizinhas são ARESTA — o `EDGE_COS` da CPU.
    /// ⭐⭐⭐ **A bandeira da W73 — *grosso a mexer, nítido ao assentar*.**
    ///
    /// `false` **salta o segundo despacho inteiro** (a borda re-amostrada) e devolve a lista de
    /// bordas vazia, que é exactamente o que o [`ph2d_field_render::trace_cancellable`] faz na CPU
    /// com o mesmo `antialias`. ⛔ Sem isto, mandar o quadro de MOVIMENTO ao dispositivo punha-o a
    /// pagar um passe que a lei do módulo manda não pagar — *dois motores, uma lei*.
    pub antialias: bool,
    pub edge_cos: f32,
    /// ⭐⭐⭐⭐ **A GRADE DE LONGE** — ver [`crate::longe`]: o raio atravessa o vazio por uma grade
    /// assada na placa e toca a peça pela árvore exacta. `None` é a marcha de sempre, **ao bit**.
    pub longe: Option<crate::longe::Longe>,
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
            // ⏱️ Só as sondas pedem o relógio por passe — ver [`crate::cronometro`].
            required_features: crate::cronometro::feature(&adapter),
            required_limits: adapter.limits(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }))
        .ok()?;
        let mut cache = crate::FieldPipelines::new();
        cache.cronometro = crate::cronometro::Cronometro::novo(&device, &queue);
        Some(Self {
            device,
            queue,
            cache,
        })
    }

    /// ⏱️ **O relatório do relógio por passe** — por rótulo, a média em ms e quantas vezes correu,
    /// desde o último relatório. Vazio fora das sondas (`PH2D_GPU_CRONOMETRO=1`).
    pub fn cronometro_relatorio(&mut self) -> Vec<(&'static str, f64, u32)> {
        self.cache
            .cronometro
            .as_mut()
            .map(crate::cronometro::Cronometro::relatorio)
            .unwrap_or_default()
    }

    /// Quantos pipelines já foram compilados — o número que um gate de *«um arrasto não
    /// recompila»* observa.
    #[must_use]
    pub fn compiled(&self) -> usize {
        self.cache.compiled()
    }

    /// Ver [`crate::FieldPipelines::rodadas_de_compilacao`].
    #[must_use]
    pub fn rodadas_de_compilacao(&self) -> usize {
        self.cache.rodadas_de_compilacao()
    }

    /// Ver [`crate::FieldPipelines::entradas_compiladas`].
    #[must_use]
    pub fn entradas_compiladas(&self) -> Vec<String> {
        self.cache.entradas_compiladas()
    }

    /// ⭐ **Um quadro, devolvido como G-BUFFER.** O shader compila-se na primeira estrutura e fica.
    ///
    /// ⚠️ É a porta da PARIDADE com a CPU. Quem quer a imagem chama o [`Self::matcap_frame`], que
    /// não traz o G-buffer de volta.
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
            Pintura::Nenhuma,
        ) {
            Saida::Gbuffer(g) => g,
            Saida::Imagem(_) => unreachable!("sem pintor a marcha devolve o G-buffer"),
        }
    }

    /// ⭐⭐⭐ **Um quadro em MATCAP, devolvido como IMAGEM** — o modo de **omissão** do modelador.
    ///
    /// ⛔⛔ **E a alternativa está medida e é PIOR:** marchar aqui e sombrear o matcap na CPU pede o
    /// G-buffer de volta (`49,8 MB` a `1920×1080`, `119`–`123 ms`) contra `90,17` da CPU inteira.
    /// *O ganho não é a marcha estar na placa — é a IMAGEM não atravessar o barramento.*
    pub fn matcap_frame(
        &mut self,
        fita: &TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: MarchSetup,
        mc: &crate::matcap::MatcapSetup<'_>,
        width: u32,
        height: u32,
    ) -> Pintado {
        let antes = self.cache.compilado_ms();
        match marcha_com(
            &self.device,
            &self.queue,
            &mut self.cache,
            fita,
            sculpts,
            setup,
            width,
            height,
            Pintura::Matcap(mc),
        ) {
            Saida::Imagem(mut p) => {
                p.compilado_ms = self.cache.compilado_ms() - antes;
                p
            }
            Saida::Gbuffer(_) => unreachable!("com matcap a marcha devolve a imagem"),
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

/// ⭐⭐⭐ **OS ALVOS DA MARCHA, vivos no dispositivo** — o que o pintor de Matcap liga no grupo `0`
/// para ler o `centro` e as bordas sem os trazer de volta.
pub(crate) struct Alvos<'a> {
    pub fita: &'a ph2d_field_eval::wgsl::TapeWgsl,
    pub bgl: &'a wgpu::BindGroupLayout,
    pub grades: &'a wgpu::Buffer,
    pub setup: &'a wgpu::Buffer,
    pub k: &'a wgpu::Buffer,
    pub centro: &'a wgpu::Buffer,
    pub conta: &'a wgpu::Buffer,
    pub borda: &'a wgpu::Buffer,
}

/// ⭐⭐⭐ **O QUE ESTE QUADRO DEVOLVE** — o G-buffer (a paridade) ou a imagem do Matcap.
pub(crate) enum Pintura<'a> {
    /// O G-buffer volta para a CPU — a porta da PARIDADE.
    Nenhuma,
    /// ⭐⭐⭐ **A luz do OLHO** — ver [`crate::matcap`], e é este o modo de **omissão** do modelador.
    Matcap(&'a crate::matcap::MatcapSetup<'a>),
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
    /// ⏱️⭐⭐⭐ **Quanto deste quadro foi COMPILAÇÃO** — ver [`crate::FieldPipelines::compilado_ms`].
    /// Quem mede o quadro para decidir o tamanho do seguinte desconta-o.
    pub compilado_ms: f64,
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

/// ⭐⭐⭐ **O layout do grupo `0`** — o uniforme, as constantes e os alvos da marcha.
///
/// ⛔⛔ **Ele é EXPLÍCITO e tem de ser** (ver [`crate::FieldPipelines::entry_with_layout`]): o
/// layout auto-derivado só declara os bindings que **aquela entrada** usa, e a passagem do centro
/// não toca na lista de bordas — o grupo de seis seria recusado em tempo de execução.
///
/// ⚠️ **E o pintor de Matcap lê o MESMO grupo** ([`crate::matcap`]): ele precisa do centro e da
/// lista de bordas, e uma segunda declaração deles seria a segunda resposta à mesma pergunta.
pub(crate) fn bgl_marcha(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("campo"),
        entries: &entradas_da_marcha(),
    })
}

/// ⭐⭐⭐ **AS SEIS ENTRADAS DO GRUPO `0`, numa lista NOMEADA** — para que a contagem de armazéns
/// deste repo possa ser **CONTADA** em vez de escrita à mão (*«número que soma se CONTA, nunca se
/// escolhe»*, `CLAUDE.md` §5.0). ⚠️ O binding `3` era a LUZ do Render traçado, que saiu em 03/10;
/// os outros números ficaram para o texto do pintor de Matcap não mudar.
#[must_use]
pub(crate) fn entradas_da_marcha() -> [wgpu::BindGroupLayoutEntry; 6] {
    [
        uniforme(0),
        armazem(1, true),
        armazem(2, false),
        armazem(4, false),
        armazem(5, false),
        armazem(6, true),
    ]
}

/// ⭐ **Quantos ARMAZÉNS uma lista de entradas liga** — a régua que faz o número ser contado.
///
/// ⚠️ Ela conta `Buffer { ty: Storage, .. }` e mais nada: um uniforme e uma textura não gastam a
/// ranhura que o `max_storage_buffers_per_shader_stage` limita.
#[must_use]
pub(crate) fn conta_armazens(entradas: &[wgpu::BindGroupLayoutEntry]) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    let n = entradas
        .iter()
        .filter(|e| {
            matches!(
                e.ty,
                wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { .. },
                    ..
                }
            )
        })
        .count() as u32;
    n
}

/// ⭐ **O fluxo de um quadro vive no irmão** — ver o cabeçalho do [`super::trace_marcha_com`].
#[path = "trace_marcha_com.rs"]
mod trace_marcha_com;
use trace_marcha_com::marcha_com;

/// ⏱️ A sonda do cache de pipelines vive no irmão — ver [`super::trace_sonda_cache`].
#[cfg(test)]
#[path = "trace_sonda_cache.rs"]
mod trace_sonda_cache;
