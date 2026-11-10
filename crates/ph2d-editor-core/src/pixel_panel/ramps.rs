//! P13: as rampas e o ciclo de cor da paleta no vocabulário do chrome (`P13_plano.md` §6). O
//! painel mostra a rampa sob a grelha da paleta, onde a cor está; a família traduz o gesto num
//! `Op` do documento (`ph2d_pixel_ponte::rampas`).

/// Uma rampa do documento como o painel a mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PixelRampInfo {
    pub id: u32,
    /// Os índices da paleta, do escuro para o claro.
    pub indices: Vec<u8>,
    pub cycle: Option<PixelCycle>,
}

/// O ciclo de cor de uma rampa: `period_ms` = a volta inteira.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelCycle {
    pub period_ms: u32,
    pub direction: PixelCycleDirection,
}

impl PixelCycle {
    /// O ciclo que a caixa Cycle liga: um degrau por cada 100 ms (o quadro de fábrica do Pixel).
    #[must_use]
    pub fn default_for(steps: usize) -> Self {
        let n = u32::try_from(steps.max(1)).unwrap_or(u32::MAX);
        Self {
            period_ms: n.saturating_mul(100),
            direction: PixelCycleDirection::Forward,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelCycleDirection {
    #[default]
    Forward,
    Reverse,
    PingPong,
}

impl PixelCycleDirection {
    pub const ALL: [Self; 3] = [Self::Forward, Self::Reverse, Self::PingPong];
}

/// Palette ▸ New Ramp…: a base é a cor primária; os campos inteiros do diálogo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRampConfig {
    pub steps: u8,
    /// Quanto a matiz roda do meio a cada ponta, em graus.
    pub hue_shift_deg: i16,
    /// `0` = croma igual em todo degrau, `100` = croma zero nas pontas.
    pub chroma_pct: u8,
    /// A claridade do degrau mais escuro e do mais claro, em %.
    pub light_pct: [u8; 2],
}

impl Default for PixelRampConfig {
    /// A configuração 1 do oráculo (`coloraide_rampas.json`): 5 degraus, 20°, 0,6, [0,25; 0,92].
    fn default() -> Self {
        Self {
            steps: 5,
            hue_shift_deg: 20,
            chroma_pct: 60,
            light_pct: [25, 92],
        }
    }
}

/// Um gesto sobre as rampas (cada um UM passo do desfazer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PixelRampAction {
    /// *Create* do New Ramp…: as cores geradas entram no fim da paleta e viram uma rampa.
    New(PixelRampConfig),
    /// *Make Ramp*: estes índices (a selecção, em ordem de paleta) viram uma rampa.
    Make(Vec<u8>),
    Remove(u32),
    /// A caixa Cycle, o Period e a Direction da rampa (`None` = sem ciclo).
    Cycle {
        ramp: u32,
        cycle: Option<PixelCycle>,
    },
}
