//! ⭐⭐ **A fileira de VERSÕES do cartão** — um chip por receita da família.
//!
//! # ⛔⛔⛔ O mecanismo de PROPRIEDADES foi ADIADO (Enio, 2026-09-01)
//!
//! Este módulo já teve duas encarnações, e as duas foram recusadas pelo dono:
//!
//! 1. **As chaves no nome** (`Casa {Size=Big}`) — uma gramática dentro do `Name`, com oito funções
//!    a lê-la e a escrevê-la. Custou seis reports com foto: quando o nome manda, renomear deixa de
//!    ser renomear e passa a ser uma operação estrutural.
//! 2. **A propriedade como DADO** + o botão *Salvar Variação…* — o desenho do Figma com o gesto do
//!    XD. *«não ficou bom e não funcionou»* ⇒ **adiado para o fim do plano**, e o código saiu
//!    inteiro. ⚠️ Ele não está comentado nem desligado por bandeira: *meio-feito é pior que não
//!    começar*, e uma feature adiada que fica no fonte é a que volta sozinha.
//!
//! ⇒ o que fica é o que existia **antes das duas** e ninguém recusou (F5 critério 2, 27/08): a
//! família é o conjunto de receitas aparentadas, e o cartão oferece **uma fileira com o nome de
//! cada versão**. É o modelo dos *Prefab Variants* do Unity — derivação, sem eixos.
//!
//! ⚠️ **O nome no chip é RÓTULO, e não mecanismo.** Ninguém o parseia; ele é lido para se mostrar,
//! como a Hierarquia o mostra. *A doença era o nome DECIDIR, nunca o nome aparecer.*

use super::inspector_model_instance::VariantChoice;
use crate::ids;

/// A fileira de versões. ⚠️ O `name` é **vazio** e quem lhe chama *Variant* é o painel (HR-15).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantAxis {
    /// Vazio — ver acima.
    pub name: String,
    /// As versões alcançáveis daqui.
    pub options: Vec<VariantChoice>,
}

/// ⭐ **Uma versão da família, como o cartão precisa de a ver.**
///
/// A shell extrai isto do mundo (as raízes `MasterRoot` aparentadas) e passa-o para cá — é o que
/// mantém esta crate sem ECS.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct VariantMember {
    /// O `StableId` da receita — **a identidade**, que é o que a troca precisa de saber.
    pub master: u64,
    /// O `Name` dela, que é o que o artista lê na Hierarquia.
    pub name: String,
}

/// ⭐⭐ **As versões que esta cópia pode escolher.**
///
/// `members` é a família inteira, **ordenada por `master`** — a ordem é a que o artista vê, e
/// ordenar por id é o que a torna estável entre quadros. `me` é a versão vigente.
///
/// Devolve a fileira e quantas versões ficaram **de fora** do teto da tabela de ids — ⛔ escrito,
/// nunca truncado em silêncio.
///
/// ⚠️ **Com menos de duas versões não há fileira**: um chip único é um controlo que não escolhe
/// nada, e a fileira é derivada — ela aparece e desaparece com a família.
#[must_use]
pub fn axes_for(members: &[VariantMember], me: u64) -> (Vec<VariantAxis>, usize) {
    if members.len() < 2 {
        return (Vec::new(), 0);
    }
    let options: Vec<VariantChoice> = members
        .iter()
        .take(ids::MAX_INSTANCE_AXIS_VALUES)
        .map(|m| VariantChoice {
            master: m.master,
            label: m.name.clone(),
            current: m.master == me,
        })
        .collect();
    let beyond = members.len().saturating_sub(options.len());
    (
        vec![VariantAxis {
            name: String::new(),
            options,
        }],
        beyond,
    )
}

#[cfg(test)]
#[path = "variant_axes_tests.rs"]
mod tests;
