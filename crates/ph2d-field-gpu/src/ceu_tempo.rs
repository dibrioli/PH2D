//! ⭐⭐⭐⭐ **A OCLUSÃO NO TEMPO** — o céu de um ponto calcula-se UMA vez e sobrevive ao movimento
//! (`docs/Render3d/14` §6, o passo `3` da `F1`: *«o quadro anterior é informação; hoje é deitado
//! fora»*; ordem do dono de 2026-09-29: *«o render ainda não está em tempo real — somos uma game
//! engine»*).
//!
//! # A propriedade que torna isto EXACTO e não uma aproximação
//!
//! Os cones da oclusão são direcções FIXAS de MUNDO ([`crate::trace_wgsl`], `direccao_do_cone`), e
//! o céu de um ponto depende do ponto, da normal e do campo — **nunca da câmara**. Enquanto o artista
//! gira a vista a peça não muda, logo o que o quadro anterior somou para um ponto continua CERTO.
//!
//! ⇒ as SOMAS dos cones (`Σ c·vis` e `Σ c`) guardam-se NO MUNDO, numa tabela de hash de células do
//! tamanho de um pixel — a *cache em grelha de hash* dos traçadores de caminhos (a SHaRC e família),
//! aplicada à grandeza que aqui é independente da vista. Cada pixel acha a célula do SEU ponto (e da
//! sua normal, num de `16` baldes); se ela já tem as `CEU_FATIAS` fatias, lê; senão reclama as que
//! faltam com um `atomicAdd` cada e acrescenta-as a uma lista de trabalho — pixels vizinhos na mesma
//! célula repartem as fatias entre si, e a célula fica COMPLETA no mesmo quadro em que apareceu.
//!
//! ⛔⛔ **A 1.ª redacção guardava o histórico no ECRÃ e reprojectava-o**, e foi MEDIDA e recusada: com
//! a rotação a zero ela era exacta (`máx 1` nível), e bastava UM quadro de meio grau para `9 212`
//! canais passarem de `8` níveis — cada salto copia o valor do pixel vizinho, nas sombras de contacto
//! a oclusão muda depressa, e o erro ACUMULA a cada quadro (`16` saltos de `0,5°` erravam mais do que
//! `16` de `3°`). *Um valor recopiado de ecrã para ecrã deriva; um valor guardado no mundo não.* Na
//! tabela o erro está limitado ao tamanho de uma célula, e não cresce.
//!
//! O quadro ASSENTE continua a oclusão inteira e exacta (é ele que a paridade com a CPU mede) e
//! GRAVA as células com a MÉDIA dos seus pixels — logo a primeira rotação já as encontra cheias.
//!
//! # ⭐⭐⭐⭐ Tempo real também no ZOOM (2026-09-30)
//!
//! Três coisas tinham de ser verdade, e nenhuma era: **(1)** o alcance da oclusão vinha da câmara e
//! passou a MUNDO (`ph2d_field_render::occlusion_reach`); **(2)** a [`ChaveDoCeu`] não leva nada da
//! câmara; **(3)** a tabela tem [`ENTRADAS_POR_PIXEL`] entradas por pixel da vista, com o tecto do
//! dispositivo — a `2²¹` fixos ela ENCHIA e o que sobrava era sal. Relógio (`diag_o_ceu_no_tempo`,
//! release, `1920×1080`, carga `8`–`12`, mínimo das medianas de `3` corridas, `0` reinícios):
//!
//! | cena | girar `3°` antes → agora | aproximar `3 %` | afastar `3 %` | sem céu (girar · aproximar) |
//! |---|---|---|---|---|
//! | nó (`28`) | `28,5 → 18,3` | `66,3 → 29,5` | `27,2 → 16,0` | `12,4` · `21,0` |
//! | outras três | `8,0`–`9,2 → 5,8`–`7,2` | `12,3`–`15,7 → 8,1`–`10,6` | `7,8`–`9,8 → 6,6`–`7,3` | `4,4` · `5,9` (a `30`) |
//!
//! ⚠️ A coluna SEM CÉU é o chão: a aproximar, o nó passa de `16,7 ms` mesmo com a oclusão de graça
//! — o que lhe falta ao tempo real ali é a marcha da peça, não o céu.
//!
//! # ⚠️ Os armazéns
//!
//! A tabela vive num GRUPO próprio (`1`), ligado só por estes dois kernels: o grupo `0` tem seis
//! armazéns e aqui sobem para sete — abaixo do piso de oito da `wgpu` —, e o passe que pinta (que
//! já precisa de nove) não é tocado.

/// ⭐⭐⭐⭐ **O que o quadro faz com o histórico da oclusão.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CeuTempo {
    /// O quadro de sempre, **ao bit** — nenhum histórico lido nem escrito.
    #[default]
    Nao,
    /// O quadro ASSENTE: a oclusão inteira, e GRAVA-a no histórico com as fatias todas.
    Grava,
    /// O quadro de MOVIMENTO: herda do quadro anterior e marcha uma fatia só onde falta.
    Acumula,
}

impl CeuTempo {
    /// O código que o `Setup` do WGSL lê — ver `ceu_tempo` na `struct Setup`.
    #[must_use]
    pub(crate) fn codigo(self) -> u32 {
        match self {
            Self::Nao => 0,
            Self::Grava => 1,
            Self::Acumula => 2,
        }
    }
}

/// ⭐ **Em quantas fatias os cones se partem.** Com os `48` de fábrica são `6` cones por fatia — a
/// unidade de trabalho da lista: os pixels de uma célula nova repartem-nas, e cada item da lista
/// marcha a mesma quantidade de cones (sem divergência de célula dentro de um grupo).
pub const CEU_FATIAS: u32 = 8;

/// ⭐ Quantas das [`CEU_FATIAS`] uma célula recebe por quadro de MOVIMENTO — ver o WGSL.
pub const FATIAS_POR_QUADRO: u32 = 4;

/// ⭐⭐⭐⭐ **Em quantos quadros de movimento DEPOIS DO ASSENTE vale o [`FATIAS_POR_QUADRO`]** — ver o
/// WGSL. ⛔ Um orçamento de fatias CONTADAS foi medido e recusado: contar fatias não mede custo (a
/// rosca pede `364 884` num quadro por `1,5 ms`, o nó `54 846` por `5,7 ms`).
pub const QUADROS_COM_TECTO: u32 = 2;

/// O que tem de ser IGUAL para o histórico ser válido: o campo e tudo o que os cones leem.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChaveDoCeu {
    fonte: String,
    consts: Vec<u32>,
    marcha: Vec<u32>,
    longe: Option<crate::longe::Longe>,
    /// Quantas entradas a tabela tem — ver [`entradas_para`]. Mudar de janela muda-as, e a tabela
    /// recomeça pela MESMA porta que uma edição da peça.
    entradas: u32,
    /// ⭐⭐⭐⭐ As LÂMPADAS que a tabela guarda (a posição de cada uma, em bits) — ver
    /// [`palavras_para`]. Mover uma lâmpada muda a sombra de toda célula, e a tabela recomeça.
    lampadas: Vec<u32>,
    /// Quantas palavras uma entrada tem.
    palavras: u32,
}

impl ChaveDoCeu {
    /// ⚠️ **Uma peça com ESCULTURA não é guardada** — a cerca das sondas e do chão: a grade dela
    /// sobe por outro caminho e não entra nesta chave.
    pub(crate) fn de(
        fita: &ph2d_field_eval::wgsl::TapeWgsl,
        sculpts: &[ph2d_field_eval::device::DeviceSculpt],
        setup: &crate::trace::MarchSetup,
        pixels: u64,
        limite: u64,
    ) -> Option<Self> {
        if !sculpts.is_empty() {
            return None;
        }
        let bits = |v: &[f32]| v.iter().map(|f| f.to_bits()).collect::<Vec<u32>>();
        // ⛔⛔ **NADA DA CÂMARA ENTRA AQUI** (2026-09-30). Os dois limiares de PIXEL (`hit_eps`,
        // `normal_eps`) escalam com o zoom e estavam na chave: cada quadro de aproximar recomeçava a
        // tabela e pagava a oclusão inteira — o zoom NÃO estava em tempo real, e o gate que só
        // girava à distância fixa não o podia ver. Eles só deslocam a origem de um cone e a
        // diferença finita da normal na ordem de UM pixel, que é o tamanho da própria célula — o
        // erro que a tabela já aceita. O alcance, que também vinha da câmara, passou a MUNDO
        // (`ph2d_field_render::occlusion_reach`).
        let mut marcha = bits(&[
            setup.step,
            setup.ao_reach,
            setup.ball_center[0],
            setup.ball_center[1],
            setup.ball_center[2],
            setup.ball_radius,
        ]);
        marcha.push(setup.budget);
        marcha.push(setup.ao_rays);
        let palavras = palavras_para(setup.n_lamps);
        let lampadas = if palavras > PALAVRAS_DO_CEU {
            let n = setup.n_lamps as usize;
            setup.lamps[..n].iter().flat_map(|l| bits(l)).collect()
        } else {
            Vec::new()
        };
        Some(Self {
            fonte: fita.source.clone(),
            consts: bits(&fita.consts),
            marcha,
            longe: setup.longe,
            entradas: entradas_para(pixels, limite, palavras),
            lampadas,
            palavras,
        })
    }
}

/// ⭐⭐⭐⭐ **Quantas entradas por PIXEL da vista a tabela tem** — o recurso é a MEMÓRIA da placa
/// (`20 B` por entrada sem lâmpadas ⇒ `80 B` por pixel: `166 MB` a `1920×1080`; `32 B` com até
/// [`LAMPADAS_NA_TABELA`] ⇒ `266 MB`), e o tecto é o do dispositivo ([`entradas_para`]).
///
/// ⛔⛔ **Até 2026-09-30 era uma constante, `2²¹` (`40 MB`), e ela era o SAL da imagem.** Uma célula
/// tem `1`–`2` píxeis e a sondagem é linear em `8` vagas, logo a `1920×1080` a tabela ENCHIA: um
/// pixel sem entrada marcha UMA fatia (`6` cones) só para si e pinta-a — pontos soltos no fundo das
/// fendas. Um zoom põe um nível inteiro de células novas ao lado do de antes, e ali era pior.
/// Medido (`diag_os_gestos_da_camara`, o nó e a rosca, `12` quadros de cada gesto, canais acima de
/// `8` níveis contra a oclusão exacta):
///
/// | entradas | por pixel | girar nó · rosca | aproximar nó · rosca | afastar nó · rosca |
/// |---:|---:|---|---|---|
/// | `2²¹` | `1` | `64`–`131` · `480`–`548` | `1 604`–`1 744` · `1 251`–`1 361` | `55`–`83` · `144`–`187` |
/// | `2²²` | `2` | `11`–`32` · `181`–`239` | `326`–`411` · `464`–`516` | `1`–`34` · `57`–`75` |
/// | **`2²³`** | **`4`** | `11`–`32` · `208`–`214` | `240`–`267` · `417`–`445` | `1` · `51`–`69` |
/// | `2²⁴` | `8` | `11`–`32` · `183`–`192` | `213`–`240` · `346`–`373` | `1` · `51` |
///
/// ⇒ o joelho é `4` por pixel: o dobro disso compra `10`–`15 %` a aproximar e nada a girar.
/// ⛔ Roubar as entradas velhas mais cedo (`VELHA = 16`) foi medido e não ajuda.
pub const ENTRADAS_POR_PIXEL: u64 = 4;

/// ⭐ **As entradas de uma vista de `pixels`**, com o tecto do DISPOSITIVO: o maior buffer que ele
/// deixa ligar a um shader (`limite`, em bytes). Numa placa no piso da `wgpu` (`128 MiB`) uma vista
/// `1920×1080` fica com `6,7 M` em vez de `8,3 M` — o que a tabela acima mede entre `2²²` e `2²³`.
#[must_use]
pub fn entradas_para(pixels: u64, limite: u64, palavras: u32) -> u32 {
    let quer = pixels.saturating_mul(ENTRADAS_POR_PIXEL);
    let cabe = limite / (u64::from(palavras) * 4);
    u32::try_from(quer.min(cabe).max(1 << 16)).unwrap_or(u32::MAX)
}

/// ⭐ **Quanto as vizinhas de uma célula vazia podem discordar para ela as herdar** — em céu (`0..1`).
///
/// Medido (nó de toro a `3°` por quadro, `1920×1080`): herdar sem esta cerca levava o quadro de
/// `21,1` a `16,2 ms` e o pior erro de `13` a `56` níveis, num ponto claro no fundo de uma fenda; a
/// cerca de `0,03`/`0,06`/`0,12` quase não mexe no relógio (`17,4`/`17,0`/`16,4`) nem no pior erro —
/// quem o curou foi exigir DUAS vizinhas. Fica o meio da faixa.
const CONCORDANCIA: f32 = 0.06;

/// ⏱️ Os contadores do quadro de movimento, depois da lista (ver [`crate::cronometro`]): os itens
/// pedidos de céu, de lâmpadas da peça, de lâmpadas do chão, e os que não couberam.
const CONTADORES: u64 = 8;
const ROTULOS_DOS_CONTADORES: [&str; 8] = [
    "n-ceu",
    "n-luz-peca",
    "n-luz-chao",
    "n-transbordo",
    "n-herdou",
    "n-sem-vizinhas",
    "n-discordam",
    "n-do-nivel",
];

/// O tecto de grupos de um despacho numa dimensão (`maxComputeWorkgroupsPerDimension` do piso da `wgpu`).
const MAX_GRUPOS_1D: u32 = 65_535;

/// Quantos quadros sem ser lida uma entrada aguenta antes de outra célula a poder roubar.
pub const VELHA: u32 = 120;

/// As palavras de uma entrada sem lâmpadas: a impressão da chave, `Σ c·vis`, `Σ c` (as duas em ponto
/// fixo), as fatias reclamadas e o quadro em que foi lida pela última vez.
const PALAVRAS_DO_CEU: u32 = 5;

/// ⭐⭐⭐⭐ **Até quantas lâmpadas a tabela guarda a SOMBRA** — a de cada lâmpada num ponto não depende
/// da câmara, logo um giro não a marcha outra vez. Medido no nó de toro (`=28`, `1920×1080`,
/// 2026-09-30): a sombra das lâmpadas era `4,2`–`5,3 ms` do passe da luz num giro, e `~80 %` dela o
/// CHÃO. ⚠️ O recurso deste tecto é a MEMÓRIA: cada par de lâmpadas custa uma palavra por entrada
/// (`8,3 M` entradas a `1920×1080`), e acima dele as lâmpadas voltam a ser marchadas por pixel.
pub const LAMPADAS_NA_TABELA: u32 = 4;

/// ⭐⭐⭐⭐ **A quantas células do chão a peça tem de estar para o chão usar a tabela** — ver o
/// `chao_perto_da_peca` do WGSL (a penumbra perto da peça é mais fina do que a célula).
pub const CHAO_PERTO: f32 = 32.0;

/// ⭐ **As palavras de uma entrada** com `n_lamps` lâmpadas na cena: as do céu, mais uma com a
/// contagem de amostras, mais uma por PAR de lâmpadas (`16` bits cada). ⚠️ A regra é a MESMA do
/// `lampadas_na_tabela` do WGSL.
#[must_use]
pub fn palavras_para(n_lamps: u32) -> u32 {
    if n_lamps == 0 || n_lamps > LAMPADAS_NA_TABELA {
        return PALAVRAS_DO_CEU;
    }
    PALAVRAS_DO_CEU + 1 + n_lamps.div_ceil(2)
}

/// ⭐ **A tabela guardada na placa entre quadros** — e a chave do campo que a encheu.
pub(crate) struct Tabela {
    chave: ChaveDoCeu,
    buffer: wgpu::Buffer,
    quadro: u32,
    /// Quantos quadros de movimento correram desde o último assente (que grava a tabela).
    desde_o_assente: u32,
    /// A lista de trabalho do movimento e quantas vagas ela tem.
    lista: Option<(wgpu::Buffer, u32)>,
    /// Os argumentos do despacho indirecto — um buffer À PARTE, porque a `wgpu` não deixa o mesmo
    /// ser armazém e argumento no mesmo despacho: o `args` escreve-os na lista e uma CÓPIA traz-nos.
    args: Option<wgpu::Buffer>,
}

/// ⭐ **Quantas vezes a tabela recomeçou do zero** — a régua de que uma edição da peça a esquece.
impl crate::FieldPipelines {
    #[must_use]
    pub fn ceu_tempo_reinicios(&self) -> usize {
        self.ceu_tempo_reinicios
    }

    /// ⭐ **Esquece a tabela** — o quadro seguinte recomeça do zero, como depois de uma edição. É a
    /// porta das sondas que repetem uma rotação: sem ela a 2.ª volta encontra as células cheias.
    pub fn esquece_o_ceu(&mut self) {
        self.ceu_tempo = None;
    }
}

/// ⭐ **A tabela guardada serve a esta chave?** — sem ela o quadro não tem o que herdar.
pub(crate) fn herda(cache: &crate::FieldPipelines, chave: &ChaveDoCeu) -> bool {
    cache.ceu_tempo.as_ref().is_some_and(|t| &t.chave == chave)
}

/// ⭐⭐⭐⭐ **Despacha o kernel da tabela neste quadro**, no encoder da marcha e depois da luz.
#[allow(clippy::too_many_arguments)]
pub(crate) fn despacha(
    device: &wgpu::Device,
    enc: &mut wgpu::CommandEncoder,
    cache: &mut crate::FieldPipelines,
    molde: &str,
    fita: &ph2d_field_eval::wgsl::TapeWgsl,
    bgl0: &wgpu::BindGroupLayout,
    bg0: &wgpu::BindGroup,
    chave: ChaveDoCeu,
    setup: &crate::trace::MarchSetup,
    width: u32,
    height: u32,
) {
    use wgpu::util::DeviceExt;
    // A tabela só vale com a MESMA chave — uma edição da peça esquece-a.
    let mut tabela = match cache.ceu_tempo.take() {
        Some(t) if t.chave == chave => t,
        antiga => {
            cache.ceu_tempo_reinicios += 1;
            match antiga {
                // ⚠️ O buffer serve se tiver o MESMO tamanho: limpá-lo no encoder custa menos do que
                // alocar `166 MB`.
                Some(t)
                    if t.chave.entradas == chave.entradas && t.chave.palavras == chave.palavras =>
                {
                    enc.clear_buffer(&t.buffer, 0, None);
                    Tabela {
                        chave,
                        buffer: t.buffer,
                        quadro: 0,
                        desde_o_assente: 0,
                        lista: t.lista,
                        args: t.args,
                    }
                }
                // Sem tabela, ou com outro tamanho: a velha (se houver) é largada aqui.
                _ => Tabela {
                    buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("ceu-tempo"),
                        size: u64::from(chave.entradas) * u64::from(chave.palavras) * 4,
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    }),
                    chave,
                    quadro: 0,
                    desde_o_assente: 0,
                    lista: None,
                    args: None,
                },
            }
        }
    };
    // ⚠️ O quadro começa em `VELHA + 1`: com `0` toda entrada acabada de nascer lia-se velha.
    tabela.quadro = tabela.quadro.max(VELHA + 1).wrapping_add(1);
    tabela.desde_o_assente = if setup.ceu_tempo == CeuTempo::Acumula {
        tabela.desde_o_assente.saturating_add(1)
    } else {
        0
    };
    let por_quadro = if tabela.desde_o_assente <= QUADROS_COM_TECTO {
        FATIAS_POR_QUADRO
    } else {
        CEU_FATIAS
    };
    // ⭐ A UNIDADE das células é uma fracção da bola da peça — um mundo sem escala própria.
    let unidade = (setup.ball_radius * 2f32.powi(-16)).max(f32::MIN_POSITIVE);
    let mut u = Vec::with_capacity(32);
    for v in [tabela.chave.entradas, CEU_FATIAS, tabela.quadro, VELHA] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    // ⭐ As VAGAS da lista: duas fatias por pixel, e nunca mais do que um despacho indirecto alcança.
    let vagas = (width * height).saturating_mul(2).min(MAX_GRUPOS_1D * 64);
    let lista = match tabela.lista.take() {
        Some((b, n)) if n >= vagas => (b, n),
        _ => (
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ceu-tempo-lista"),
                size: (4 + u64::from(vagas) * 2 + CONTADORES) * 4,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            vagas,
        ),
    };
    for f in [unidade, CONCORDANCIA] {
        u.extend_from_slice(&f32::to_le_bytes(f));
    }
    for v in [vagas, tabela.chave.palavras, por_quadro, 0, 0, 0] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    let ub = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("ceu-tempo-tabela"),
        contents: &u,
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let bgl1 = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("ceu-tempo"),
        entries: &[
            crate::trace::uniforme(0),
            crate::trace::armazem(1, false),
            crate::trace::armazem(2, false),
        ],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("ceu-tempo"),
        bind_group_layouts: &[Some(bgl0), Some(&bgl1)],
        immediate_size: 0,
    });
    let fonte = format!(
        "{molde}{}",
        crate::ceu_tempo_wgsl::WGSL.replace("{FATIAS_POR_QUADRO}", &FATIAS_POR_QUADRO.to_string())
    );
    let mut kernel = |nome: &str| {
        cache
            .entry_with_layout(device, &fonte, fita, nome, Some(&layout))
            .clone()
    };
    let bg1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("ceu-tempo"),
        layout: &bgl1,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: tabela.buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: lista.0.as_entire_binding(),
            },
        ],
    });
    let grupos = (width.div_ceil(8), height.div_ceil(8));
    let indirecto = tabela.args.take().unwrap_or_else(|| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ceu-tempo-args"),
            size: 12,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    });
    if setup.ceu_tempo == CeuTempo::Acumula {
        let (pede, args, marcha, le) = (
            kernel("ceu_tempo_pede"),
            kernel("ceu_tempo_args"),
            kernel("ceu_tempo_marcha"),
            kernel("ceu_tempo_le"),
        );
        enc.clear_buffer(&lista.0, 0, Some(16));
        let contadores = (4 + u64::from(vagas) * 2) * 4;
        enc.clear_buffer(&lista.0, contadores, Some(CONTADORES * 4));
        // ⏱️ O relógio das sondas sai do cache enquanto os passes o usam (ver [`crate::cronometro`]).
        let mut crono = cache.cronometro.take();
        let mut passe = |enc: &mut wgpu::CommandEncoder,
                         p: &wgpu::ComputePipeline,
                         rotulo: &'static str,
                         disp: &dyn Fn(&mut wgpu::ComputePass<'_>)| {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceu-tempo"),
                timestamp_writes: crono.as_mut().and_then(|c| c.marca(rotulo)),
            });
            cp.set_pipeline(p);
            cp.set_bind_group(0, bg0, &[]);
            cp.set_bind_group(1, &bg1, &[]);
            disp(&mut cp);
        };
        passe(enc, &pede, "ceu-pede", &|cp| {
            cp.dispatch_workgroups(grupos.0, grupos.1, 1)
        });
        passe(enc, &args, "ceu-args", &|cp| {
            cp.dispatch_workgroups(1, 1, 1)
        });
        enc.copy_buffer_to_buffer(&lista.0, 4, &indirecto, 0, 12);
        passe(enc, &marcha, "ceu-marcha", &|cp| {
            cp.dispatch_workgroups_indirect(&indirecto, 0)
        });
        passe(enc, &le, "ceu-le", &|cp| {
            cp.dispatch_workgroups(grupos.0, grupos.1, 1)
        });
        if let Some(c) = crono.as_mut() {
            c.contadores(enc, &lista.0, contadores, &ROTULOS_DOS_CONTADORES);
        }
        cache.cronometro = crono;
    } else {
        let passes = [
            (kernel("ceu_tempo_zera"), "ceu-zera"),
            (kernel("ceu_tempo_grava"), "ceu-grava"),
        ];
        let mut crono = cache.cronometro.take();
        for (p, rotulo) in &passes {
            let mut cp = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ceu-tempo"),
                timestamp_writes: crono.as_mut().and_then(|c| c.marca(rotulo)),
            });
            cp.set_pipeline(p);
            cp.set_bind_group(0, bg0, &[]);
            cp.set_bind_group(1, &bg1, &[]);
            cp.dispatch_workgroups(grupos.0, grupos.1, 1);
        }
        cache.cronometro = crono;
    }
    tabela.lista = Some(lista);
    tabela.args = Some(indirecto);
    cache.ceu_tempo = Some(tabela);
}
