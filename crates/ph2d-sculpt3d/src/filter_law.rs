//! ⭐⭐⭐ **A LEI QUE UM ARRASTO DE FILTRO APLICA** — a união das nove leis de
//! malha (W9) com os cinco tipos do filtro de tecido (espec §7).
//!
//! # Por que uma UNIÃO, e não um segundo modo armado
//!
//! ⚠️ **Os dois filtros são o MESMO gesto para quem usa o app:** armar, arrastar
//! na horizontal, e uma lei corre sobre a peça inteira, com **um** passo de undo.
//! O que muda é a lei — que é exactamente o que um selector escolhe.
//!
//! ⛔ **Um terceiro modo armado seria o caminho errado, e o código já o dizia:**
//! o doc do `arm_filter` do shell regista que a exclusão entre o filtro e o
//! transform vive em duas portas *«porque um `enum` obrigaria a reescrever os
//! cinco leitores do `transform_arm`»* — com um terceiro, a exclusão passa a ser
//! três pares e o argumento inverte-se. E o artista ganharia um botão a mais para
//! dizer a mesma coisa que o selector já diz.
//!
//! ⚠️ **A diferença de GESTO entre as duas famílias é real e fica na LEI**, não
//! no modo: as de malha refazem **um** passo a partir da pose congelada (voltar
//! com o dedo desfaz) e as de tecido **acumulam** uma simulação (voltar com o
//! dedo empurra ao contrário). É a espec §7 contra a §1, e o
//! [`crate::stroke_cloth_filter`] mede-a.

use crate::{ClothFilterKind, FilterKind};

/// **A lei escolhida no selector do filtro.**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterLaw {
    /// Uma das nove leis de malha da W9 — um passo a partir da pose congelada.
    Mesh(FilterKind),
    /// Um dos cinco tipos do filtro de tecido — uma simulação que acumula.
    Cloth(ClothFilterKind),
}

impl Default for FilterLaw {
    fn default() -> Self {
        Self::Mesh(FilterKind::ALL[0])
    }
}

impl FilterLaw {
    /// **Todas, de malha primeiro.**
    ///
    /// ⚠️ **É DERIVADA das duas listas** (`FilterKind::ALL` e
    /// `ClothFilterKind::ALL`), nunca escrita à mão: uma lei nova de qualquer dos
    /// lados entra aqui sozinha. ⛔ A contagem NÃO se escreve num doc — conte-a
    /// nas duas fontes.
    #[must_use]
    pub fn all() -> Vec<Self> {
        FilterKind::ALL
            .into_iter()
            .map(Self::Mesh)
            .chain(ClothFilterKind::ALL.into_iter().map(Self::Cloth))
            .collect()
    }

    /// O rótulo do chip — o da lei, sem prefixo de família.
    ///
    /// ⚠️ **Duas famílias têm rótulos REPETIDOS** (*Inflate* e *Scale* existem
    /// dos dois lados, e são leis diferentes): é a fileira que os separa, e é
    /// por isso que elas são **duas rows** no painel e não uma de catorze chips.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Mesh(k) => k.label(),
            Self::Cloth(k) => k.label(),
        }
    }

    /// ⭐ **A CHAVE do rótulo**, delegada à família — o irmão de [`label`](Self::label).
    ///
    /// ⚠️ Ela delega **pela mesma porta** que o `label` acima: a chave de uma lei de malha é a do
    /// `FilterKind` e a de uma de tecido é a do `ClothFilterKind`. *Escrever aqui uma terceira
    /// tabela poria o chip a dizer uma coisa e o censo a medir outra.*
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Mesh(k) => k.label_key(),
            Self::Cloth(k) => k.label_key(),
        }
    }

    /// **Esta lei é de TECIDO?** — a pergunta que o driver do gesto faz, e ela
    /// tem dois leitores (o `begin` e o `at`), o que a torna uma porta.
    #[must_use]
    pub fn is_cloth(self) -> bool {
        matches!(self, Self::Cloth(_))
    }

    /// O tipo de tecido, se for um.
    #[must_use]
    pub fn cloth(self) -> Option<ClothFilterKind> {
        match self {
            Self::Cloth(k) => Some(k),
            Self::Mesh(_) => None,
        }
    }

    /// A lei de malha, se for uma.
    #[must_use]
    pub fn mesh(self) -> Option<FilterKind> {
        match self {
            Self::Mesh(k) => Some(k),
            Self::Cloth(_) => None,
        }
    }
}
