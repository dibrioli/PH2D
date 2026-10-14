//! ⭐ **O componente `Macro`** (ADR-0186; `docs/Macros/01_plano_onda1.md` §2) — a gravação de um
//! objecto: os passos de cada família, no relógio da macro.
//!
//! É CONFIG (registado, gravado, no undo). Nasce do gesto Gravar ▸ Parar, e por isso não é oferecido
//! na paleta: um `Macro` vazio não tocaria nada. QUANDO e ONDE toca é da Timeline — um clipe com
//! dono, o Autoplay da Animação, o laço do clipe —, não daqui: duas verdades para o mesmo relógio
//! divergiriam.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **Como a trilha do Motion toca.** ⚠️ A ordem é a do seletor do Inspector; acrescentar é no fim.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModoDaMacro {
    /// Cada passo no quadro gravado: o que o artista viu.
    #[default]
    Fiel,
    /// Re-simula da semente depois de cada mudança: o efeito de cada nó (o tutorial nó a nó).
    Explicativo,
}

#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Macro {
    /// O `MacroDoc` codificado (`ph2d_macro::codec::write`). O ficheiro leva a versão PRÓPRIA no
    /// cabeçalho (`PH2DMACRO`, major/minor), como o `TimelineDoc` leva o `DOC_VERSION`: um conteúdo
    /// mais novo muda-a ele, e o `PROJECT_SCHEMA` não se mexe. Vazio = nada gravado.
    pub doc: Vec<u8>,
    pub modo: ModoDaMacro,
}
