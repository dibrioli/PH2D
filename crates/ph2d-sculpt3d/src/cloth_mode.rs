//! ⭐⭐⭐ **O MODO DE DEFORMAÇÃO e a ÁREA SIMULADA do pincel de tecido** — os dois
//! selectores que a referência põe no painel dela (espec §8.4) e que aqui
//! existiam **só como variável de ambiente**.
//!
//! ⚠️ **Sem isto o pincel tinha OITO comportamentos e UM alcançável.** A lei da
//! referência implementa os oito desde 2026-09-05 e o artista só chegava ao
//! arrasto — que é exactamente o *controlo inalcançável* que esta casa varre a
//! cada wave, na espécie mais cara: não é um botão morto, é um botão que nunca
//! foi desenhado sobre um motor que já responde.
//!
//! ⚠️ **A ORDEM dos chips é NOSSA**, derivada da ordem em que a espec apresenta
//! os modos (§4.2 os de força, §4.3 os de âncora, §4.5 o Expand), e **não** é um
//! facto do alvo — a espec não a regista. ⛔ Quem a mudar tem de olhar se algum
//! ficheiro guarda o ÍNDICE em vez do nome.

use ph2d_cloth::verlet_gesto::{Area, Modo};

/// **Como o pincel de tecido deforma** (espec §4.2, §4.3, §4.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClothMode {
    /// Empurra na direcção do movimento do cursor, a mesma para todos.
    #[default]
    Drag,
    /// Empurra para DENTRO, ao longo da normal da área, com magnitude `2R`.
    Push,
    /// Puxa cada vértice para o ponto do cursor.
    PinchPoint,
    /// Puxa para a LINHA do traço (descarta a componente ao longo dele).
    PinchPerpendicular,
    /// Empurra ao longo da normal de cada vértice.
    Inflate,
    /// Prende um conjunto fixo de vértices e leva-os com o cursor.
    Grab,
    /// Arrasta o que já pegou, com o centro da queda um passo atrasado.
    SnakeHook,
    /// Muda o REPOUSO: a folha cresce em vez de se deslocar.
    Expand,
}

impl ClothMode {
    /// Os oito, na ordem em que a fileira os desenha.
    pub const ALL: [Self; 8] = [
        Self::Drag,
        Self::Push,
        Self::PinchPoint,
        Self::PinchPerpendicular,
        Self::Inflate,
        Self::Grab,
        Self::SnakeHook,
        Self::Expand,
    ];

    /// O rótulo que aparece no chip (a UI da casa é inglesa — HR/memória).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Drag => "Drag",
            Self::Push => "Push",
            Self::PinchPoint => "Pinch Point",
            Self::PinchPerpendicular => "Pinch Perp",
            Self::Inflate => "Inflate",
            Self::Grab => "Grab",
            Self::SnakeHook => "Snake Hook",
            Self::Expand => "Expand",
        }
    }

    /// A lei correspondente na `ph2d-cloth`.
    ///
    /// ⚠️ **É a ÚNICA porta entre o vocabulário do painel e o da lei**, e é por
    /// isso que ela é um `match` exaustivo: um modo novo de qualquer dos lados
    /// é erro de compilação aqui.
    #[must_use]
    pub fn modo(self) -> Modo {
        match self {
            Self::Drag => Modo::Arrastar,
            Self::Push => Modo::Empurrar,
            Self::PinchPoint => Modo::ApertarPonto,
            Self::PinchPerpendicular => Modo::ApertarLinha,
            Self::Inflate => Modo::Inflar,
            Self::Grab => Modo::Agarrar,
            Self::SnakeHook => Modo::Gancho,
            Self::Expand => Modo::Expandir,
        }
    }

    /// **Este modo RE-APANHA o cursor na superfície a cada passo?** (espec §4.3)
    ///
    /// Os de força sim; o Grab fica no ponto do pen-down e o Snake Hook anda no
    /// plano de profundidade. É a pergunta que a shell faz para escolher o braço
    /// do gesto.
    #[must_use]
    pub fn repica(self) -> bool {
        !matches!(self, Self::Grab | Self::SnakeHook)
    }
}

/// **Que pedaço da malha entra na simulação** (espec §2.1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClothArea {
    /// A esfera fica onde o traço começou, com o raio do 1.º passo.
    Local,
    /// A malha inteira.
    Global,
    /// A esfera segue o cursor, com o raio actual — a omissão dos presets do
    /// alvo (espec §8.2), e por isso a nossa.
    #[default]
    Dynamic,
}

impl ClothArea {
    /// As três, na ordem do painel do alvo (espec §8.4).
    pub const ALL: [Self; 3] = [Self::Local, Self::Global, Self::Dynamic];

    /// O rótulo do chip.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::Global => "Global",
            Self::Dynamic => "Dynamic",
        }
    }

    /// A lei correspondente na `ph2d-cloth`.
    #[must_use]
    pub fn area(self) -> Area {
        match self {
            Self::Local => Area::Local,
            Self::Global => Area::Global,
            Self::Dynamic => Area::Dinamica,
        }
    }
}
