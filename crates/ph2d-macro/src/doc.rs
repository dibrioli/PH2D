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
    /// Por tempo NÃO decrescente (o codec e o gravador recusam o contrário).
    pub events: Vec<Event>,
}

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
