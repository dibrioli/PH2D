//! ⭐⭐ **OS SEIS ENCAIXES** — o vocabulário de *onde um painel PODE estar* (decisões **D1** e
//! **D4**, `docs/UI_New_and_Simple/spec/01_modelo_de_areas.md` §2 e §3).
//!
//! # Por que SEIS, e não os doze do Godot
//!
//! O `editor_dock.h` do Godot tem **12** encaixes: quatro por lado (duas colunas × duas metades),
//! três em baixo, mais o principal. ⛔ **Não os copiamos, e a razão é aritmética:**
//!
//! | colunas por lado | largura | de 1366 (o alvo declarado) |
//! |---|---:|---:|
//! | **1** (308 + 304) | 612 px | **44,8 %** — cabe |
//! | 2 (o modelo deles) | 1224 px | **89,6 %** — ⛔ impossível |
//!
//! ⇒ **uma coluna por lado.** Os 12 pressupõem um monitor de desktop largo.
//! ⚠️ Um encaixe hospeda `0..n` painéis; com `n > 1` são **abas** — *é assim que um encaixe absorve
//! crescimento sem crescer* (spec §2, regra 1).
//!
//! # ⭐ O que isto compra: um gesto que deixa de ser exprimível
//!
//! Um painel de propriedades declara `allowed_slots = {RightTop, RightBottom}` e `can_float =
//! false` — e **nunca chega perto de uma viewport ou de uma régua**, porque não há valor que o
//! exprima. É um `Constraint`, não uma verificação (D1).
//!
//! ⚠️ **Divergência deliberada do Godot:** eles têm `available_layouts` (`VERTICAL | HORIZONTAL |
//! FLOATING`), que descreve a **forma** que o dock aceita. Nós usamos `allowed_slots`, que descreve
//! os **sítios** — com seis encaixes fixos o sítio já implica a forma, e um conjunto de sítios é
//! directamente verificável por um portão.

/// Um dos seis encaixes.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Slot {
    /// Metade de cima da coluna da esquerda.
    LeftTop,
    /// Metade de baixo da coluna da esquerda.
    LeftBottom,
    /// Metade de cima da coluna da direita.
    RightTop,
    /// Metade de baixo da coluna da direita.
    RightBottom,
    /// A faixa de baixo (a linha do tempo, a tira do Flip).
    Bottom,
    /// ⚠️ **O `CENTER` nunca está vazio e nunca é aba de outro encaixe** (spec §2, regra 4).
    Center,
}

impl Slot {
    /// Os seis, na ordem da declaração — a fonte de toda varredura.
    pub const ALL: [Self; 6] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::Bottom,
        Self::Center,
    ];

    /// O bit deste encaixe num [`SlotSet`].
    #[must_use]
    pub const fn bit(self) -> u8 {
        1u8 << (self as u8)
    }
}

/// **Um conjunto de encaixes** — o tipo de `allowed_slots`.
///
/// ⚠️ Um bitset e não um `&[Slot]`: ele tem de ser utilizável numa **constante associada** de trait
/// (`const ALLOWED_SLOTS: SlotSet`), e as operações de conjunto têm de ser `const fn` para o
/// default de cada painel se compor sem código de runtime.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotSet(u8);

impl SlotSet {
    /// O conjunto vazio — ⚠️ um painel que o declare **não tem onde estar**, e há gate.
    pub const NONE: Self = Self(0);
    /// A coluna da esquerda, as duas metades.
    pub const LEFT: Self = Self(Slot::LeftTop.bit() | Slot::LeftBottom.bit());
    /// A coluna da direita, as duas metades.
    pub const RIGHT: Self = Self(Slot::RightTop.bit() | Slot::RightBottom.bit());
    /// As duas colunas.
    pub const SIDES: Self = Self(Self::LEFT.0 | Self::RIGHT.0);
    /// Só a faixa de baixo.
    pub const BOTTOM: Self = Self(Slot::Bottom.bit());
    /// Só o centro.
    pub const CENTER: Self = Self(Slot::Center.bit());
    /// Todos os encaixes que **não** são o centro — o default de um painel.
    pub const ANY_DOCK: Self = Self(Self::SIDES.0 | Self::BOTTOM.0);

    /// O conjunto com um encaixe só.
    #[must_use]
    pub const fn of(slot: Slot) -> Self {
        Self(slot.bit())
    }

    /// A união de dois conjuntos.
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Este encaixe está no conjunto?
    #[must_use]
    pub const fn contains(self, slot: Slot) -> bool {
        self.0 & slot.bit() != 0
    }

    /// Está vazio?
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Os encaixes deste conjunto, na ordem de [`Slot::ALL`].
    pub fn iter(self) -> impl Iterator<Item = Slot> {
        Slot::ALL.into_iter().filter(move |s| self.contains(*s))
    }
}

#[cfg(test)]
#[path = "slot_tests.rs"]
mod tests;
