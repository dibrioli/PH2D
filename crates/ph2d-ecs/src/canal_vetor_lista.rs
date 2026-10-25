//! A LISTA de um campo do Vetor ([`super`]): o endereço por identidade das entradas (F31 §A).

use super::{
    EFEITO_LIGADO, EFEITO_PARAM, FILTRO_TIPO, FILTRO_ULTIMO, PILHA_DILATA_JUNCAO, PILHA_LIGADA,
    PINCEL_ESPELHO, TINTA_PREENCH,
};

/// ⭐ **De que LISTA da forma um campo é** — a porta única do endereço por identidade (F31 §A): a
/// chave de uma entrada leva o `ItemId` dela (`FieldKey::do_item`), nunca a posição.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Lista {
    /// A forma, o contorno, a base, o texto, os componentes vivos: endereço por `i`/`j`.
    Nenhuma,
    /// Uma entrada da pilha de pinturas (`TINTA_*`, `TRACO_*`, `PINCEL_*`, `PILHA_*`); `i` 0 sem
    /// item = a tinta de base.
    Tinta,
    /// Um efeito da pilha (`EFEITO_*`).
    Efeito,
    /// Um filtro da forma (`FILTRO_*`, `VecFilter`).
    Filtro,
}

impl Lista {
    /// A posição na lista que a chave POSICIONAL (v23) com este `i` endereçava — `None` = a base
    /// (tinta `i` 0) ou um campo sem lista.
    #[must_use]
    pub fn posicao_antiga(self, i: u16) -> Option<usize> {
        match self {
            Self::Nenhuma => None,
            Self::Tinta => usize::from(i).checked_sub(1),
            Self::Efeito | Self::Filtro => Some(usize::from(i)),
        }
    }

    /// O `i` que a chave posicional dava à posição `pos` (o inverso de [`Self::posicao_antiga`]) —
    /// só para o NOME da linha, que continua a ler-se como antes.
    #[must_use]
    pub fn i_antigo(self, pos: usize) -> usize {
        match self {
            Self::Tinta => pos + 1,
            _ => pos,
        }
    }
}

/// A lista do campo `campo` (ver [`Lista`]).
#[must_use]
pub const fn lista(campo: u16) -> Lista {
    match campo {
        TINTA_PREENCH..=PINCEL_ESPELHO | PILHA_LIGADA..=PILHA_DILATA_JUNCAO => Lista::Tinta,
        EFEITO_LIGADO | EFEITO_PARAM => Lista::Efeito,
        FILTRO_TIPO..=FILTRO_ULTIMO => Lista::Filtro,
        _ => Lista::Nenhuma,
    }
}
