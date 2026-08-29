#![forbid(unsafe_code)]
//! `audio.bands` — **o som como campo de valor por-elemento** (doc 63 §3, o P0
//! `audio.bands`; doc 89 folha 14 §3 item 6, onde ele estava nomeado *"só para o
//! consolidador não o perder na fronteira entre famílias"*).
//!
//! É o **último P0 aberto da conferência inteira**, e o desbloqueador já estava
//! construído: o doc 63 §D6 mediu-o assim — *"`ph2d-audio-spectral` JÁ tem FFT;
//! falta só a ponte"*. A frase é a mesma que a wave do TEXTO escreveu uma família
//! adiante: **era FIAÇÃO, não DSP.**
//!
//! ## A cerca que decide o desenho: a FFT nunca entra no cook
//!
//! Doc 63 §6: *"FFT NUNCA entra no cook. A shell bridge computa bandas por frame
//! via `ph2d-audio-spectral` e publica como INPUT do grafo — determinismo: bandas
//! são função do ARQUIVO + playhead ⇒ scrub-exato."*
//!
//! Este crate não depende de crate de áudio nenhuma, e a contenção é
//! **estrutural, não disciplinar**: sem a dependência ele não CONSEGUE analisar
//! som. O que ele possui é a **LEI** — como o eixo de frequência é cortado em
//! bandas, como um nível vira `0..1` — porque isso é vocabulário de MOTION, não de
//! áudio, e é o que os params do artista descrevem. O shell faz a transformada e
//! chama [`fold`]; os dois lados concordam por chamarem a mesma função.
//!
//! ## Por que ele é um campo de VALOR, e não uma fonte de geometria
//!
//! A referência decide: Cavalry **Sound** mapeia bandas → índices (*"Use Index
//! Context"*), C4D **Sound Effector** é um efetor sobre clones que já existem, e
//! MOPs **Audio** é um falloff. Nenhuma delas *cria* objetos — todas MODULAM os
//! que o artista já distribuiu. Emitindo a coluna `v` (o tipo do
//! `value.instance_field`), este nó entra em `motion.drive`, em todo `field.*`, no
//! `motion.duplicator` e no resto da biblioteca **sem um nó novo a jusante**.
//!
//! **A regra é uma:** o elemento `i` toma a banda `i % count`. Com entrada ligada,
//! `N` vem da geometria (32 caixas ⇒ 32 leituras); sem entrada, `N` **é o próprio
//! `count`** — *as bandas elas mesmas*, que é o analisador de espectro e a
//! cardinalidade natural desta pergunta. ⚠️ Diverge do `value.instance_field`
//! (que degenera para 1) **porque o dado diverge**: existem exatamente `count`
//! bandas, e 1 seria um número que ninguém pediu.
//!
//! ## O suavizado é um filtro sobre a GRAVAÇÃO, nunca sobre a sessão
//!
//! ⚠️ Um analisador comum guarda o valor do quadro anterior e faz um one-pole —
//! e isso é **ESTADO**, que quebra o scrub: voltar a régua daria outro número.
//! Aqui o suavizado é aplicado ao longo das COLUNAS DO ARQUIVO, uma vez, quando a
//! análise é construída. É a mesma disciplina do `motion.emitter` (*"o conjunto
//! vivo é função pura do playhead"*), e é o que torna a promessa do doc 63 —
//! *scrub-exato* — verdadeira por construção em vez de por sorte.

use ph2d_node_registry::{
    NodeRegistry, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// O stream de instâncias — lido pela CONTAGEM (a porta `in`, opcional).
const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// O tipo de VALOR: o campo escalar por-instância (espelho do
/// `value.instance_field`; local de propósito — o vocabulário partilhado é a
/// PORTA, nunca um símbolo partilhado, que é o que mantém isto uma folha).
pub const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// A coluna de saída (a canônica do domínio de valor).
pub const VALUE_COL: &str = "v";

/// **O arquivo é um TEXT PARAM** (doc 32), não um `ParamSpec`: ele mora no
/// `Graph`, ao lado do manifesto e nunca dentro dele — o padrão que deixa um param
/// não-`f32` existir sem tocar o contrato congelado (§6).
///
/// ⚠️ **É um CAMINHO, e a escolha é deliberada.** A alternativa — nomear o clipe
/// que o editor de áudio tem aberto — faria o grafo depender de estado de EDITOR:
/// reabrir o projeto devolveria silêncio, e nada no arquivo salvo diria o que
/// dirigiu a animação. Um caminho é estado de DOCUMENTO (viaja no texto do grafo),
/// e o preço — o arquivo mora fora do projeto, então movê-lo quebra o vínculo — é
/// o *missing footage* que todo DCC tem e sabe nomear.
pub const FILE_KEY: &str = "file";

/// Os nomes dos params. ⚠️ Esta lista e o `MANIFEST` são conferidos um contra o
/// outro por gate: um param que o manifesto declara e a chave de análise não vê
/// fica **inerte depois da primeira vez** — o artista mexe, o cache devolve a
/// análise velha, e nada acusa.
pub mod param {
    pub const COUNT: &str = "count";
    pub const MIN_HZ: &str = "min_hz";
    pub const MAX_HZ: &str = "max_hz";
    pub const SCALE: &str = "scale";
    pub const WEIGHTING: &str = "weighting";
    pub const FLOOR_DB: &str = "floor_db";
    pub const GAIN: &str = "gain";
    pub const SMOOTHING: &str = "smoothing";
    pub const ALL: &[&str] = &[
        COUNT, MIN_HZ, MAX_HZ, SCALE, WEIGHTING, FLOOR_DB, GAIN, SMOOTHING,
    ];
}

/// Como o eixo de frequência é cortado. ⚠️ **APPEND ONLY** — o índice é o que um
/// grafo salvo guarda, e mover um renomeia a escolha de todo documento já autorado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// Larguras iguais em hertz. Honesto e quase sempre errado para o olho: numa
    /// faixa 40 Hz–16 kHz, metade das bandas cai acima de 8 kHz, onde quase nada
    /// da música vive — o analisador fica com um canto vivo e uma planície morta.
    Linear,
    /// Larguras iguais em OITAVAS (o default). É como a altura é ouvida, e é o que
    /// faz cada banda ter mais ou menos a mesma quantidade de música dentro.
    Log,
    /// A escala mel — log acima de ~700 Hz, quase linear abaixo. Distribui mais
    /// resolução nos graves que o log puro, que é onde o corpo de uma batida está.
    Mel,
}

impl Scale {
    pub fn from_index(v: f32) -> Self {
        match v.round() as i32 {
            0 => Self::Linear,
            2 => Self::Mel,
            _ => Self::Log,
        }
    }
}

/// Se o nível é corrigido para como o ouvido pesa a frequência.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weighting {
    /// O que a transformada diz, sem correção — o default, porque é o número
    /// medido e não uma opinião sobre ouvidos.
    None,
    /// Ponderação **A** (IEC 61672): tira dos graves e dos agudos extremos o peso
    /// que o ouvido não lhes dá. É o que impede um bumbo de dominar todas as
    /// barras num analisador de espectro.
    A,
}

impl Weighting {
    pub fn from_index(v: f32) -> Self {
        match v.round() as i32 {
            1 => Self::A,
            _ => Self::None,
        }
    }
}

/// A configuração de análise: tudo o que decide QUE NÚMEROS saem de um arquivo.
///
/// ⚠️ É deliberadamente **todo o conjunto de params** — é isto que o shell usa
/// como chave de cache, e um campo que exista no manifesto e não aqui torna aquele
/// controle inerte assim que a análise é memoizada uma vez.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandSpec {
    pub count: usize,
    pub min_hz: f32,
    pub max_hz: f32,
    pub scale: Scale,
    pub weighting: Weighting,
    pub floor_db: f32,
    pub gain: f32,
    pub smoothing: f32,
}

/// Quantas bandas o painel oferece de arrasto, e o teto que a caixa ainda aceita.
///
/// ⚠️ **O teto é MEDIDO e diz de que recurso ele é** (§0): a análise tem
/// `bins = window/2 + 1` compartimentos, e acima de `bins` bandas duas bandas
/// vizinhas leem o MESMO compartimento — o controle deixa de controlar. Com a
/// janela de 1024 do `ph2d-audio-spectral` são **513**.
pub const MAX_BANDS: usize = 513;

impl BandSpec {
    /// Lê os oito params pela MESMA porta que o `eval` usa. É o que faz a chave
    /// do shell e a do nó serem os mesmos bits.
    pub fn from_params(get: impl Fn(&str) -> f32) -> Self {
        Self {
            count: (get(param::COUNT).round().max(1.0) as usize).min(MAX_BANDS),
            min_hz: get(param::MIN_HZ).max(1.0),
            max_hz: get(param::MAX_HZ).max(2.0),
            scale: Scale::from_index(get(param::SCALE)),
            weighting: Weighting::from_index(get(param::WEIGHTING)),
            floor_db: get(param::FLOOR_DB).min(-1.0),
            gain: get(param::GAIN),
            smoothing: get(param::SMOOTHING).clamp(0.0, 1.0),
        }
    }

    /// A chave de conteúdo da ANÁLISE.
    ///
    /// ⚠️ **Ela não carrega o instante, e isso é o desenho.** A chave nomeia
    /// *quais bandas deste arquivo*, não *quando*; o shell reescreve o mesmo nome
    /// a cada quadro com os valores do playhead, e quem diz ao cook que o valor se
    /// move é o [`Effect::Temporal`] do nó. Uma chave por instante encheria o
    /// canal externo de nomes mortos, um por quadro já cozinhado.
    pub fn key(&self, file: &str) -> String {
        let mut k = String::from("audio");
        for f in [
            self.min_hz,
            self.max_hz,
            self.floor_db,
            self.gain,
            self.smoothing,
        ] {
            k.push(':');
            k.push_str(&f.to_bits().to_string());
        }
        k.push(':');
        k.push_str(&self.count.to_string());
        k.push(':');
        k.push_str(&(self.scale as u8).to_string());
        k.push(':');
        k.push_str(&(self.weighting as u8).to_string());
        // ⚠️ O caminho vai POR ÚLTIMO e sem prefixo de comprimento porque nada o
        // segue: não há um segundo campo de texto com que ele possa forjar uma
        // fronteira (o gate `a_colon_in_the_path_cannot_forge_another_key`).
        k.push(':');
        k.push_str(file);
        k
    }

    /// As `count + 1` fronteiras da faixa, em hertz.
    ///
    /// Porta ÚNICA do corte: quem PINTA uma banda e quem a MEDE perguntam aqui, e
    /// um segundo cálculo desenharia a barra numa frequência e a preencheria com
    /// outra.
    pub fn edges(&self) -> Vec<f32> {
        let (lo, hi) = (self.min_hz.min(self.max_hz), self.max_hz.max(self.min_hz));
        let n = self.count;
        (0..=n)
            .map(|k| {
                let t = k as f32 / n as f32;
                match self.scale {
                    Scale::Linear => lo + (hi - lo) * t,
                    // `lo·(hi/lo)^t` — larguras iguais em oitavas.
                    Scale::Log => lo * (hi / lo).powf(t),
                    Scale::Mel => mel_to_hz(hz_to_mel(lo) + (hz_to_mel(hi) - hz_to_mel(lo)) * t),
                }
            })
            .collect()
    }
}

/// Hertz → mel (O'Shaughnessy 1987, a forma que toda biblioteca de fala usa).
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// A inversa exata da [`hz_to_mel`].
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

/// A ponderação **A** em decibéis, na frequência `hz` (IEC 61672, com o
/// `+2,00 dB` que normaliza a curva a 0 dB em 1 kHz).
pub fn a_weight_db(hz: f32) -> f32 {
    let f2 = hz * hz;
    let num = 12194.0f32.powi(2) * f2 * f2;
    let den = (f2 + 20.6f32.powi(2))
        * ((f2 + 107.7f32.powi(2)) * (f2 + 737.9f32.powi(2))).sqrt()
        * (f2 + 12194.0f32.powi(2));
    if den <= 0.0 {
        return -100.0;
    }
    20.0 * (num / den).log10() + 2.0
}

/// **A LEI** — uma coluna de decibéis por compartimento vira `count` níveis em
/// `0..1`.
///
/// `db` é a coluna da análise (um valor por compartimento, do grave ao Nyquist) e
/// `hz_per_bin` é a **largura** de um compartimento, PUBLICADA por quem fez a
/// transformada. ⚠️ O eixo é recebido, nunca re-derivado de uma taxa de amostragem
/// — este crate não sabe o que é uma taxa de amostragem, e é assim que fica.
///
/// Dentro de uma banda o nível é o **PICO**, não a média: uma banda larga com um
/// harmônico forte e muito silêncio em volta tem a média de um silêncio, e o que o
/// olho procura numa barra é o evento. É a mesma barganha que o espectrograma
/// declara para os pixels dele.
///
/// ⚠️ **Uma banda estreita demais para caber num compartimento ainda lê UM** — sem
/// esse piso, subir a contagem faria as bandas graves (onde a resolução em hertz é
/// mais cara) devolverem zero em silêncio, e o controle pareceria quebrado
/// exactamente onde o artista mais o usa.
pub fn fold(db: &[f32], hz_per_bin: f32, spec: &BandSpec, out: &mut Vec<f32>) {
    out.clear();
    if db.is_empty() || hz_per_bin <= 0.0 {
        out.resize(spec.count, 0.0);
        return;
    }
    let edges = spec.edges();
    let span = -spec.floor_db;
    for k in 0..spec.count {
        let (lo, hi) = (edges[k], edges[k + 1]);
        let b0 = ((lo / hz_per_bin).floor().max(0.0) as usize).min(db.len() - 1);
        let b1 = ((hi / hz_per_bin).ceil().max(0.0) as usize).clamp(b0 + 1, db.len());
        let peak = db[b0..b1].iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let weighted = match spec.weighting {
            Weighting::None => peak,
            // No CENTRO GEOMÉTRICO da banda — a média aritmética de uma banda
            // logarítmica não é o meio dela, e nos graves erra por uma oitava.
            Weighting::A => peak + a_weight_db((lo * hi).sqrt()),
        };
        let t = ((weighted - spec.floor_db) / span).clamp(0.0, 1.0);
        out.push(t * spec.gain);
    }
}

/// O suavizado, aplicado **ao longo das colunas do arquivo** — a metade que torna
/// o scrub exato (ver o doc do módulo).
///
/// Um one-pole causal de coeficiente `smoothing`: a barra sobe na hora e desce
/// devagar, que é o *release* de um analisador. `0` devolve a matriz **ao bit**.
pub fn smooth_over_columns(bands: &mut [f32], count: usize, smoothing: f32) {
    if smoothing <= 0.0 || count == 0 || bands.len() <= count {
        return;
    }
    let a = smoothing.clamp(0.0, 1.0);
    let cols = bands.len() / count;
    for c in 1..cols {
        for k in 0..count {
            let prev = bands[(c - 1) * count + k];
            let cur = &mut bands[c * count + k];
            // Ataque imediato, queda amortecida: o pico do transiente é o que se
            // quer ver, e suavizá-lo nos dois sentidos apaga a batida.
            if *cur < prev {
                *cur = prev * a + *cur * (1.0 - a);
            }
        }
    }
}

/// O contrato estático (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("audio.bands"),
    name: "audio.bands",
    // Opcional: ligada → contagem N; solta → uma leitura por BANDA.
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // ⚠️ **Temporal, e não Pure.** O nó não lê o relógio com as próprias mãos —
    // ele lê um canal externo —, mas aquele canal é reescrito a cada quadro pelo
    // shell. Declarar `Pure` autorizaria o cook a memoizá-lo e as barras
    // congelariam, com tudo verde.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "count",
            default: 16.0,
        },
        ParamSpec {
            name: "min_hz",
            default: 40.0,
        },
        ParamSpec {
            name: "max_hz",
            default: 16_000.0,
        },
        // 0 Linear · 1 Log · 2 Mel.
        ParamSpec {
            name: "scale",
            default: 1.0,
        },
        // 0 None · 1 A.
        ParamSpec {
            name: "weighting",
            default: 0.0,
        },
        ParamSpec {
            name: "floor_db",
            default: -60.0,
        },
        ParamSpec {
            name: "gain",
            default: 1.0,
        },
        ParamSpec {
            name: "smoothing",
            default: 0.35,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct AudioBands;

impl NodeOp for AudioBands {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let spec = BandSpec::from_params(|n| ctx.param(n));
        let file = ctx.text_param(FILE_KEY).unwrap_or("").to_string();
        // Sem arquivo não há análise, e o nó **não adivinha e não falha**: um
        // campo de zeros é a identidade de tudo o que o consome (a mesma política
        // do `source.object` para um objeto ausente).
        let n_in = ctx.input(0).count();
        let levels: Vec<f32> = match ctx.external(&spec.key(&file)).get(VALUE_COL) {
            Some(Column::Scalar(v)) if !v.is_empty() => v.clone(),
            _ => Vec::new(),
        };
        // Solta, a cardinalidade é o número de BANDAS — *as bandas elas mesmas*.
        let n = if n_in > 0 { n_in } else { levels.len().max(1) };
        let v: Vec<f32> = if levels.is_empty() {
            vec![0.0; n]
        } else {
            (0..n).map(|i| levels[i % levels.len()]).collect()
        };
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(v)));
    }
}

/// Os hints do painel. ⚠️ O `file` é um campo de TEXTO — a 4ª lei do doc 88
/// (*todo param é desenhado*) alcança os text params, e sem esta row o caminho
/// seria autorável só por um arquivo salvo à mão.
static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: FILE_KEY,
        label: "Audio File",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        // ⚠️ **Uma ESPÉCIE, nunca uma lista de extensões.** Este crate não depende de crate
        // de áudio nenhuma — é essa cerca estrutural que impede a FFT de entrar no cook —,
        // logo ele **não pode saber** o que este build descodifica. Quem tem os
        // descodificadores é a shell, e é ela que resolve a espécie para a constante canónica.
        widget: ParamWidget::File {
            kind: ph2d_node_registry::FileKind::Audio,
        },
    },
    ParamUiHint {
        param: param::COUNT,
        label: "Bands",
        min: 1.0,
        max: 64.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
    },
    ParamUiHint {
        param: param::MIN_HZ,
        label: "Low",
        min: 20.0,
        max: 2000.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::MAX_HZ,
        label: "High",
        min: 1000.0,
        max: 22_000.0,
        step: 10.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SCALE,
        label: "Scale",
        min: 0.0,
        max: 2.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Linear", "Log", "Mel"],
        },
    },
    ParamUiHint {
        param: param::WEIGHTING,
        label: "Weighting",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["None", "A"],
        },
    },
    ParamUiHint {
        param: param::FLOOR_DB,
        label: "Floor",
        min: -90.0,
        max: -10.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::GAIN,
        label: "Gain",
        min: 0.0,
        max: 4.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: param::SMOOTHING,
        label: "Smoothing",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
];

/// As unidades. `min_hz`/`max_hz` são **hertz** e `floor_db` **decibéis** — dois
/// vocabulários que o artista já lê no editor de áudio, e o mesmo número não pode
/// significar coisas diferentes nos dois painéis.
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: param::MIN_HZ,
        unit: ParamUnit::Hertz,
    },
    ParamUnitDecl {
        param: param::MAX_HZ,
        unit: ParamUnit::Hertz,
    },
    ParamUnitDecl {
        param: param::FLOOR_DB,
        unit: ParamUnit::Decibel,
    },
];

pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(AudioBands))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Audio Bands",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
