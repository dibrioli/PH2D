//! **COMO o valor conduzido se combina com o canal** — os sete modos, e as
//! guardas de dois deles.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é por RESPONSABILIDADE: o `channel.rs` responde *que canal recebe e como o
//! stream é escrito*, e este responde *que aritmética junta os dois números*.

/// How the driven value combines with the existing channel.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Combine {
    /// `channel + value` — the additive default (matches `motion.step`).
    Add,
    /// `value` — overwrite the channel with the driven value.
    Set,
    /// `channel * value` — scale the existing channel by the value.
    Multiply,
    /// `channel − value` — o simétrico do `Add`.
    ///
    /// ⚠️ **Apendado no índice 3** (folha 06 linha 40; C4D *Blending Mode*). A
    /// composição existia — negar o valor a montante com um `value.math` — e é
    /// exactamente por isso que ele é *ergonomia* e não capacidade: um nó a mais
    /// e um sinal invertido no grafo para dizer «menos».
    Subtract,
    /// `channel / value` — o simétrico do `Multiply`.
    ///
    /// ⚠️ **Um divisor (quase) zero colapsa para `0`, e não para `inf`.** É a mesma
    /// guarda que o `value.math` já tinha, com o mesmo limiar e pelo mesmo motivo:
    /// um `inf` num canal de transform envenena a posição do elemento, e daí em
    /// diante todo `NaN` a jusante vem sem endereço.
    Divide,
    /// `min(channel, value)` — um TECTO sobre o canal.
    Min,
    /// `max(channel, value)` — um PISO sobre o canal.
    Max,
}

/// Abaixo desta magnitude um divisor é tratado como zero (o quociente colapsa
/// para `0`), o gêmeo exacto do `MIN_DIVISOR` do `value.math`.
const MIN_DIVISOR: f32 = 1e-9;

impl Combine {
    pub(crate) fn from_param(v: f32) -> Self {
        match v.round() as i32 {
            1 => Combine::Set,
            2 => Combine::Multiply,
            // ⚠️ Apendados no FIM: `0..2` ficam onde estavam, então todo documento
            // já autorado lê o mesmo modo.
            3 => Combine::Subtract,
            4 => Combine::Divide,
            5 => Combine::Min,
            6 => Combine::Max,
            _ => Combine::Add,
        }
    }
    pub(crate) fn apply(self, channel: f32, value: f32) -> f32 {
        match self {
            Combine::Add => channel + value,
            Combine::Set => value,
            Combine::Multiply => channel * value,
            Combine::Subtract => channel - value,
            Combine::Divide => {
                if value.abs() < MIN_DIVISOR {
                    0.0
                } else {
                    channel / value
                }
            }
            Combine::Min => channel.min(value),
            Combine::Max => channel.max(value),
        }
    }
}
