//! As escolhas segmentadas: cada opção escreve a si mesma.

use super::*;
use ph2d_tool_registry::hash_node_id;

/// A que escolha uma opção pertence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grupo {
    Filter,
    Wave,
    Law,
}

#[derive(Clone, Copy)]
pub struct Opcao {
    pub id: NodeId,
    pub chave: &'static str,
    pub grupo: Grupo,
    /// `None` = esta camada não tem esta escolha.
    pub aceso: fn(&Layer) -> Option<bool>,
    /// Não toca no que já está escolhido: clicar de novo em FM não apaga o ratio.
    pub escolhe: fn(&mut Layer),
}

#[must_use]
pub fn opcao(id: NodeId) -> Option<&'static Opcao> {
    OPCOES.iter().find(|o| o.id == id)
}

macro_rules! filtro {
    ($id:literal, $k:literal, $f:ident) => {
        Opcao {
            id: hash_node_id($id),
            chave: $k,
            grupo: Grupo::Filter,
            aceso: |l| ruido(l).map(|n| n.filter == NoiseFilter::$f),
            escolhe: |l| {
                if let Some(n) = ruido_mut(l) {
                    n.filter = NoiseFilter::$f;
                }
            },
        }
    };
}

macro_rules! onda {
    ($id:literal, $k:literal, $w:ident) => {
        Opcao {
            id: hash_node_id($id),
            chave: $k,
            grupo: Grupo::Wave,
            aceso: |l| tom(l).map(|t| matches!(t.wave, Wave::$w)),
            escolhe: |l| {
                if let Some(t) = tom_mut(l) {
                    t.wave = Wave::$w;
                }
            },
        }
    };
}

/// O FM que nasce já se ouve diferente do seno: modulador na oitava, índice 1.
const FM_NOVO: Wave = Wave::Fm {
    ratio: 2.0,
    index: 1.0,
    index_end: 1.0,
};
/// A lei esticada que nasce fica entre a corda (1) e a barra (2).
const ESTICADA_NOVA: ModeLaw = ModeLaw::Stretched { stretch: 1.5 };

pub const OPCOES: &[Opcao] = &[
    filtro!("sfxlab_filter_white", "sfxlab.card.filter.white", White),
    filtro!("sfxlab_filter_low", "sfxlab.card.filter.low", Low),
    filtro!("sfxlab_filter_band", "sfxlab.card.filter.band", Band),
    filtro!("sfxlab_filter_high", "sfxlab.card.filter.high", High),
    onda!("sfxlab_wave_sine", "sfxlab.card.wave.sine", Sine),
    onda!(
        "sfxlab_wave_triangle",
        "sfxlab.card.wave.triangle",
        Triangle
    ),
    onda!("sfxlab_wave_saw", "sfxlab.card.wave.saw", Saw),
    onda!("sfxlab_wave_square", "sfxlab.card.wave.square", Square),
    Opcao {
        id: hash_node_id("sfxlab_wave_fm"),
        chave: "sfxlab.card.wave.fm",
        grupo: Grupo::Wave,
        aceso: |l| tom(l).map(|t| matches!(t.wave, Wave::Fm { .. })),
        escolhe: |l| {
            if let Some(t) = tom_mut(l)
                && !matches!(t.wave, Wave::Fm { .. })
            {
                t.wave = FM_NOVO;
            }
        },
    },
    Opcao {
        id: hash_node_id("sfxlab_law_stretched"),
        chave: "sfxlab.card.law.stretched",
        grupo: Grupo::Law,
        aceso: |l| corpo(l).map(|b| matches!(b.law, ModeLaw::Stretched { .. })),
        escolhe: |l| {
            if let Some(b) = corpo_mut(l)
                && !matches!(b.law, ModeLaw::Stretched { .. })
            {
                b.law = ESTICADA_NOVA;
            }
        },
    },
    Opcao {
        id: hash_node_id("sfxlab_law_bar"),
        chave: "sfxlab.card.law.bar",
        grupo: Grupo::Law,
        aceso: |l| corpo(l).map(|b| b.law == ModeLaw::Bar),
        escolhe: |l| {
            if let Some(b) = corpo_mut(l) {
                b.law = ModeLaw::Bar;
            }
        },
    },
    Opcao {
        id: hash_node_id("sfxlab_law_membrane"),
        chave: "sfxlab.card.law.membrane",
        grupo: Grupo::Law,
        aceso: |l| corpo(l).map(|b| b.law == ModeLaw::Membrane),
        escolhe: |l| {
            if let Some(b) = corpo_mut(l) {
                b.law = ModeLaw::Membrane;
            }
        },
    },
];
