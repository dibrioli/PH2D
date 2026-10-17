//! As LEITURAS da luz do quadro (o custo, o nível, a fina engrossada) — ficheiro irmão do `quadro.rs`
//! pelo tecto de 700 LOC (W5j).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_gpu::{Config, Grade, Medida, Solver, SondaDeMemoria, TemposMs};

/// A configuração da vista, a grade da fluência, a região da janela nela e a janela.
type Onde<'a> = (&'a Config, Grade, ([u32; 2], [u32; 2]), (u32, u32));

/// ⭐ W5u — INSTRUMENTO `PH2D_LIGHT2D_CUSTO=1`: o número do Inspector partido por passe (a vista e o
/// entorno), a cada 60 leituras — o custo que o dono vê no app inteiro, não o da régua.
/// ⭐ W5v: e ONDE — a janela, a grade das cascatas, a fina (texels finos por texel, a captura, a região da
/// janela na grade da fluência) e a aritmética: o que uma régua tem de reproduzir.
/// ⭐ W5w: e o MÍNIMO e a MEDIANA das 60 leituras (a amostra solta é ruído); `PH2D_LIGHT2D_RENASCE=<k>` faz o
/// solver da vista renascer a cada `k` janelas no MESMO processo (a `c2` do High varia entre processos e
/// não dentro: a régua do regime de alocação). Devolve se o solver renasce agora.
pub(super) fn relatar_custo(
    janela: &mut JanelaDeCusto,
    t: &TemposMs,
    entorno: Option<&TemposMs>,
    onde: Onde<'_>,
    n: u32,
    gpu: &ph2d_gpu::GpuContext,
    solvers: (&Solver, Option<&Solver>),
) -> bool {
    if std::env::var_os("PH2D_LIGHT2D_CUSTO").is_none() {
        return false;
    }
    let total = t.total() + entorno.map_or(0.0, TemposMs::total);
    janela.total.push(total);
    janela.c2.push(t.c2_total());
    janela.fina.push(t.c3_fina);
    // ⭐ W5y: `PH2D_LIGHT2D_SERIE=1` — cada leitura (a c2 muda DENTRO de uma geração: a janela esconde quando).
    if std::env::var_os("PH2D_LIGHT2D_SERIE").is_some() {
        // E o alocador da placa (o processo inteiro): um degrau da c2 casa com uma alocação de fora?
        let (n_aloc, mib, blocos) = gpu.device.generate_allocator_report().map_or((0, 0, 0), |r| {
            (
                r.allocations.len(),
                r.total_allocated_bytes >> 20,
                r.blocks.len(),
            )
        });
        eprintln!(
            "[luz2d] série g{} w{} · c2 {:.3} · total {total:.3} · aloc {n_aloc} {mib} MiB {blocos} blocos",
            janela.geracao,
            janela.n,
            t.c2_total()
        );
    }
    if !n.is_multiple_of(60) {
        return false;
    }
    let (total_min, total_med) = min_e_mediana(&mut janela.total);
    let (c2_min, c2_med) = min_e_mediana(&mut janela.c2);
    let (fina_min, fina_med) = min_e_mediana(&mut janela.fina);
    eprintln!(
        "[luz2d] janela g{} w{} · {} leituras · total min {total_min:.2} med {total_med:.2} · c2 min \
         {c2_min:.2} med {c2_med:.2} · fina min {fina_min:.2} med {fina_med:.2}",
        janela.geracao,
        janela.n,
        janela.c2.len(),
    );
    // ⭐ W5y: `PH2D_LIGHT2D_FASES=1` — as etapas da holográfica da vista (somadas nos níveis), por janela.
    if let Some(f) = solvers.0.fases_da_hrc(gpu) {
        let mut etapas: Vec<(&str, f64)> = Vec::new();
        for (nome, _, ms, _) in f {
            match etapas.iter_mut().find(|e| e.0 == nome) {
                Some(e) => e.1 += ms,
                None => etapas.push((nome, ms)),
            }
        }
        let soma: f64 = etapas.iter().map(|e| e.1).sum();
        let partes: Vec<String> = etapas.iter().map(|(n, ms)| format!("{n} {ms:.3}")).collect();
        eprintln!(
            "[luz2d] fases g{} w{} · Σ {soma:.3} · {}",
            janela.geracao,
            janela.n,
            partes.join(" · ")
        );
    }
    if janela.n == 0 && std::env::var_os("PH2D_LIGHT2D_MEMORIA").is_some() {
        relatar_memoria(janela, gpu, solvers);
    }
    janela.total.clear();
    janela.c2.clear();
    janela.fina.clear();
    janela.n += 1;
    let renasce = std::env::var("PH2D_LIGHT2D_RENASCE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
        .is_some_and(|k| k > 0 && janela.n.is_multiple_of(k));
    if renasce {
        janela.geracao += 1;
        janela.n = 0;
    }
    let (cfg, fluencia, (i, f), alvo) = onde;
    eprintln!(
        "[luz2d] onde · janela {}×{} · grade {}×{} · fina ×{} captura ×{} · fluência {}×{} região {}×{} · {:?}",
        alvo.0,
        alvo.1,
        cfg.grade.largura,
        cfg.grade.altura,
        cfg.fina,
        cfg.captura,
        fluencia.largura,
        fluencia.altura,
        f[0] - i[0],
        f[1] - i[1],
        cfg.aritmetica,
    );
    let grade = (cfg.grade.largura, cfg.grade.altura);
    eprintln!(
        "[luz2d] custo {:.2} ms · grade {}×{} · c0 {:.2} c1 {:.2} c2 {:.2} vis {:.2} fina {:.2} c0_fina {:.2} \
         preenche {:.2} másc {:.2} rebote {:.2}+{:.2} miolo {:.2} · entorno {:.2}",
        total,
        grade.0,
        grade.1,
        t.c0,
        t.c1,
        t.c2_total(),
        t.c3_visibilidade,
        t.c3_fina,
        t.c0_fina,
        t.c3_preenche,
        t.c3_mascaras,
        t.c0_rebote,
        t.c3_rebote,
        t.c3_miolo,
        entorno.map_or(0.0, TemposMs::total),
    );
    renasce
}

/// As leituras da janela corrente do instrumento ([`relatar_custo`]): a geração do solver e a janela nela.
#[derive(Default)]
pub(super) struct JanelaDeCusto {
    total: Vec<f64>,
    c2: Vec<f64>,
    fina: Vec<f64>,
    geracao: u32,
    n: u32,
    sonda: Option<SondaDeMemoria>,
}

/// ⭐ W5x — `PH2D_LIGHT2D_MEMORIA=1` (com o `CUSTO`): a BANDA de ler cada buffer grande da holográfica da
/// vista, sozinho (1× por geração, na 1.ª janela): os zeros em páginas comprimíveis leem-se a ~4× a banda da DRAM, fora delas à banda da DRAM —
/// o regime da `c2` do High é a soma destes (handoff W5x). ⭐ W5y: e as texturas da lista e os ladrilhos, da vista
/// e do entorno (emprestados: não mudam a alocação; `=lista` dispensa os clones da holográfica).
fn relatar_memoria(
    janela: &mut JanelaDeCusto,
    gpu: &ph2d_gpu::GpuContext,
    (vista, entorno): (&Solver, Option<&Solver>),
) {
    if janela.sonda.is_none() {
        janela.sonda = SondaDeMemoria::nova(gpu);
    }
    let Some(sonda) = janela.sonda.as_ref() else {
        return;
    };
    let g = janela.geracao;
    // ⭐ W5y — CALIBRAÇÃO: texturas NOVAS de zeros (64 MiB) — a sonda de texturas vê a compressão?
    let zeros: Vec<(&'static str, u64, f64)> = [
        ("zeros_rg32", wgpu::TextureFormat::Rg32Uint, 8_u32),
        ("zeros_rgba32", wgpu::TextureFormat::Rgba32Uint, 16),
    ]
    .into_iter()
    .filter_map(|(nome, format, texel)| {
        let lado = gpu.device.limits().max_texture_dimension_2d;
        let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("luz2d sonda calibração"),
            size: wgpu::Extent3d {
                width: lado,
                height: ((64 << 20) / (texel * lado)).min(lado),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        sonda.ms_textura(&t, 3).map(|(ms, bytes)| (nome, bytes, ms))
    })
    .collect();
    linha_de_memoria(g, "calibração", "texturas", zeros.into_iter());
    for (quem, s) in [("vista", Some(vista)), ("entorno", entorno)] {
        let Some(s) = s else { continue };
        let hrc = s
            .buffers_da_holografica()
            .iter()
            .map(|(nome, b)| (*nome, b.size(), sonda.ms(b, Medida::Ler, 3)));
        linha_de_memoria(g, quem, "hrc", hrc);
        // ⭐ W5y: aos SALTOS (4096 × 256 fios × 16 leituras de 16 B): a subida e a descida leem espalhado.
        let saltos = s.buffers_da_holografica().iter().map(|(nome, b)| {
            (*nome, 4096 * 256 * 16 * 16, sonda.ms(b, Medida::Saltar, 3))
        });
        linha_de_memoria(g, quem, "saltos", saltos);
        // ⭐ W5y: as texturas da lista (≥ 1 MiB; as menores cabem no L2) e os ladrilhos.
        let (texturas, ladrilhos) = s.recursos_da_lista();
        let outros = texturas
            .filter_map(|(nome, t)| sonda.ms_textura(t, 3).map(|(ms, bytes)| (nome, bytes, ms)))
            .filter(|(_, bytes, _)| *bytes >= 1 << 20)
            .chain(ladrilhos.map(|(nome, b)| (nome, b.size(), sonda.ms(b, Medida::Ler, 3))));
        linha_de_memoria(g, quem, "lista", outros);
    }
}

/// `[luz2d] memória g<g> · <quem> <grupo> · Σler <ms> ms · <nome> <GB/s>/<MiB> …`.
fn linha_de_memoria(
    g: u32,
    quem: &str,
    grupo: &str,
    medidas: impl Iterator<Item = (&'static str, u64, f64)>,
) {
    let mut soma = 0.0;
    let partes: Vec<String> = medidas
        .map(|(nome, bytes, ms)| {
            soma += ms;
            let curto = nome
                .trim_start_matches("luz2d/buf/hrc_")
                .trim_start_matches("luz2d/buf/")
                .trim_start_matches("luz2d/tex/");
            format!(
                "{curto} {:.0}/{:.1}",
                bytes as f64 / (ms * 1e-3) / 1e9,
                bytes as f64 / f64::from(1 << 20)
            )
        })
        .collect();
    eprintln!(
        "[luz2d] memória g{g} · {quem} {grupo} · Σler {soma:.3} ms · GB/s/MiB: {}",
        partes.join(" · ")
    );
}

/// ⭐ W5y — INSTRUMENTO `PH2D_LIGHT2D_SOZINHA=1|2`: a luz corre com a placa ociosa (1: espera antes e
/// depois do envio dela; 2: e o entorno num envio próprio). O carimbo de um passe conta o tempo de parede,
/// o trabalho que corre AO LADO incluído. 0 sem a variável (e sem espera).
pub(super) fn sozinha(gpu: &ph2d_gpu::GpuContext) -> u8 {
    let n = std::env::var("PH2D_LIGHT2D_SOZINHA")
        .ok()
        .and_then(|v| v.trim().parse::<u8>().ok())
        .unwrap_or(0);
    if n > 0 {
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    n
}

/// Envia `enc`, espera a placa e devolve um codificador novo.
pub(super) fn enviar_e_esperar(
    gpu: &ph2d_gpu::GpuContext,
    enc: wgpu::CommandEncoder,
) -> wgpu::CommandEncoder {
    gpu.queue.submit([enc.finish()]);
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    gpu.device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("luz2d quadro do editor"),
        })
}

fn min_e_mediana(v: &mut [f64]) -> (f64, f64) {
    if v.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

impl super::LuzDoQuadro {
    /// ⭐ W5e: quantas vezes a fina está engrossada neste quadro (0 = a do nível) e se foi pelo
    /// dispositivo (o aviso do Inspector).
    #[must_use]
    pub fn fina_engrossada(&self) -> (u32, bool) {
        let pelos_tetos = self
            .fina_dos_tetos
            .and_then(|(_, f)| f)
            .is_some_and(|f| f > 0);
        let k = (self.degrau.captura_px / self.base.captura_px)
            .log2()
            .round() as u32;
        (k, pelos_tetos)
    }

    /// ⭐ W5e: quantas vezes o solver da vista foi montado (alocado) — uma grade nova, um nível novo ou
    /// uma recusa que se tenta de novo; nada cabendo abaixo da recusa, o quadro não monta.
    #[must_use]
    pub fn montagens(&self) -> u32 {
        self.montagens
    }

    /// O último custo de placa da grade atual (ms), com ou sem o «Automático»; `None` antes da 1.ª
    /// leitura dela ou sem `TIMESTAMP_QUERY` (aí o automático fica no teto).
    #[must_use]
    pub fn custo_ms(&self) -> Option<f64> {
        self.custo_ms
    }

    /// Quantas leituras de custo o «Automático» já colheu — o governador lê CONTINUAMENTE, não uma
    /// vez.
    #[must_use]
    pub fn leituras_de_custo(&self) -> u32 {
        self.leituras
    }

    /// O nível em uso e se é automático (`None` = degrau fixo).
    #[must_use]
    pub fn qualidade(&self) -> Option<(NivelDaLuz, bool)> {
        let p = self.preferencia?;
        Some((
            if p.automatica {
                self.governador.nivel
            } else {
                p.nivel
            },
            p.automatica,
        ))
    }
}
