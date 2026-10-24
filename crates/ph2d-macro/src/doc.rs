//! O documento de uma macro: trilhas de eventos com tempo.

/// Tempo desde o início da gravação, em microssegundos.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacroTime(pub u64);

impl MacroTime {
    pub const ZERO: MacroTime = MacroTime(0);

    #[must_use]
    pub const fn from_ms(ms: u64) -> Self {
        MacroTime(ms * 1_000)
    }

    /// Segundos do relógio da macro (o `ph2d_ecs::MacroTempo` que a Timeline escreve) → µs. Antes do
    /// início é o início.
    #[must_use]
    pub fn from_secs_f64(segundos: f64) -> Self {
        MacroTime((segundos.max(0.0) * 1e6).round() as u64)
    }
}

/// O quadro de referência da gravação (Corel Painter): as famílias guardam posições
/// RELATIVAS a ele, para a mesma macro tocar noutro tamanho.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RefRect {
    pub w: f32,
    pub h: f32,
}

/// Um passo gravado. A `payload` é da família dona da trilha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub t: MacroTime,
    pub payload: Vec<u8>,
}

/// Uma trilha: os passos de UMA família sobre UM alvo, por ordem de tempo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    /// A chave da família (`"painter"`, `"motion"`, …) — quem sabe ler a `payload`.
    pub family: String,
    /// `StableId` do objeto alvo; `0` = nenhum.
    pub target: u64,
    /// O estado inicial que a família restaura antes do 1.º passo (MyPaint, Corel «Initial State»).
    pub initial: Vec<u8>,
    /// Por tempo NÃO decrescente (o codec e o gravador recusam o contrário). A GRAVAÇÃO: nunca muda.
    pub events: Vec<Event>,
    /// As edições das etapas ([`crate::etapa`]): a trilha que toca é [`Track::tocavel`].
    pub edicoes: Vec<crate::etapa::EdicaoDeEtapa>,
    /// **O objecto da trilha nasce DENTRO do objecto Macro** (W18, `docs/Macros/16_W18_plano.md` §2.4).
    /// `None` = o alvo é o próprio objecto que tem a `Macro` (as gravações de antes da W18).
    pub nasce: Option<Nascimento>,
    /// **Esta trilha continua o objecto da trilha `k`** (W19: o artista voltou a uma imagem já gravada):
    /// não faz nascer nada, e o alvo é o dela. Exclusivo com [`Self::nasce`].
    pub continua: Option<u32>,
}

/// **A ficha de nascimento** do objecto de uma trilha. O tamanho e o conteúdo inicial NÃO estão aqui:
/// vivem no estado inicial da família ([`Track::initial`]) — uma verdade só.
#[derive(Clone, Debug, PartialEq)]
pub struct Nascimento {
    /// Quando nasce, no relógio da macro.
    pub inicio: MacroTime,
    pub nome: String,
    /// A transformada LOCAL ao pai: `[tx, ty, rotação, sx, sy, skew_x, skew_y]`.
    pub local: [f32; 7],
    /// O índice da trilha cujo objecto é o pai; `None` = o objecto Macro.
    pub pai: Option<u32>,
}

impl Eq for Nascimento {}

#[derive(Clone, Debug, PartialEq)]
pub struct MacroDoc {
    pub reference: RefRect,
    pub tracks: Vec<Track>,
}

impl MacroDoc {
    #[must_use]
    pub fn new(reference: RefRect) -> Self {
        MacroDoc {
            reference,
            tracks: Vec::new(),
        }
    }

    /// O tempo do último passo de todas as trilhas.
    #[must_use]
    pub fn duration(&self) -> MacroTime {
        self.tracks
            .iter()
            .filter_map(|t| t.events.last().map(|e| e.t))
            .max()
            .unwrap_or(MacroTime::ZERO)
    }

    /// `true` se cada trilha está por tempo não decrescente.
    #[must_use]
    pub fn is_time_ordered(&self) -> bool {
        self.tracks
            .iter()
            .all(|tr| tr.events.windows(2).all(|w| w[0].t <= w[1].t))
    }
}
