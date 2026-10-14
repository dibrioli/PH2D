//! **O vocabulário da secção MACRO** (ADR-0186) — a gravação de um objecto.
//!
//! O painel não vê a `ph2d-ecs` nem a `ph2d-macro` (ADR-0029): o modo viaja como o ÍNDICE de
//! `ModoDaMacro`, e a gravação como o que a secção mostra dela. A tradução é da
//! `ph2d-app-components`, com gate de ida-e-volta.

/// Quantos modos há — o comprimento de `ModoDaMacro` (gate na `macro_inspector`).
pub const MACRO_MODOS: usize = 2;

/// Índice do modo `Fiel` (o que o artista viu).
pub const MODO_FIEL: u8 = 0;
/// Índice do modo `Explicativo` (o efeito de cada nó).
pub const MODO_EXPLICATIVO: u8 = 1;

/// Uma edição da secção.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroFieldEdit {
    /// O modo, pelo índice de `ModoDaMacro`.
    Modo(u8),
}

/// A gravação, lida do componente.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MacroGravacao {
    /// Nada gravado.
    Vazia,
    /// Cada trilha (a família e quantos passos), e a duração em microssegundos.
    Lida {
        trilhas: Vec<(String, usize)>,
        duracao_us: u64,
    },
    /// O ficheiro não se lê neste build: a chave da frase.
    Ilegivel(&'static str),
}

/// Gravada por uma versão mais nova (outra major do formato).
pub const MACRO_DE_VERSAO_NOVA: &str = "panel.inspector.macro.newer_version";
/// Os bytes não são uma macro, ou estão cortados.
pub const MACRO_ESTRAGADA: &str = "panel.inspector.macro.damaged";

/// O instantâneo da secção.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorMacroInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// O modo, pelo índice de `ModoDaMacro`.
    pub modo: u8,
    pub gravacao: MacroGravacao,
    /// Quantos objectos estão escolhidos (a edição aplica-se ao activo).
    pub selected_count: usize,
}
