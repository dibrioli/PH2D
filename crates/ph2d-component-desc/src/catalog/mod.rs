//! **O catálogo** — um descritor por tipo registado, cortado por FAMÍLIA.
//!
//! # Por que por família, e não uma tabela só
//!
//! Isolamento (DIRETRIZ §1.5.2.1). Uma linha que acrescenta um componente de física apende
//! em [`physics`] e não encosta em mais nada; a de vetor apende em [`vector`]. Uma tabela
//! central de 108 linhas seria o sítio em que **toda** linha escreve — a superfície de
//! colisão que se manda projetar para fora. E o teto de 700 LOC por arquivo já a proibiria.
//!
//! # A ordem DENTRO de cada família é lei
//!
//! Cada `&[ComponentDesc]` está **ordenado por `canonical_name`**, e [`desc_for`] faz busca
//! binária dentro de cada família. Há gate (`the_catalog_is_sorted_and_unique`): fora de
//! ordem, a busca devolve `None` para um tipo que existe — e um descritor que não é
//! encontrado lê-se exatamente como um descritor que não existe.
//!
//! # ⚠️ Este catálogo não é a fonte da VERDADE sobre o que existe
//!
//! O `ComponentRegistry` é. Este é side-metadata **sobre** o que lá está, e a ligação entre
//! os dois é uma string. O censo de dois lados (na shell, que é quem tem o registo completo)
//! é o que impede a deriva; sem ele, esta pasta apodrece em silêncio.

use crate::ComponentDesc;

pub mod bridges;
pub mod core;
pub mod field;
pub mod image;
pub mod physics;
pub mod script;
pub mod vector;

/// As famílias, cada uma ordenada por `canonical_name`.
///
/// ⛔ **Fonte única da iteração.** Acrescentar uma família é acrescentar uma linha aqui e um
/// módulo ao lado — nunca uma segunda lista noutro sítio.
const FAMILIES: &[&[ComponentDesc]] = &[
    bridges::DESCS,
    core::DESCS,
    field::DESCS,
    image::DESCS,
    physics::DESCS,
    script::DESCS,
    vector::DESCS,
];

/// Todos os descritores, família a família.
///
/// A ordem é *dentro* de cada família, não global — quem precisa de ordem total ordena o que
/// recebe. (A paleta agrupa por [`crate::ComponentCategory`], então nunca precisa dela.)
pub fn all() -> impl Iterator<Item = &'static ComponentDesc> {
    FAMILIES.iter().copied().flatten()
}

/// O descritor de um tipo, pelo nome canónico do `ComponentRegistry`.
///
/// Busca binária dentro de cada família (7 famílias × log₂(≤32) ≈ 35 comparações no pior
/// caso). ⚠️ **Não faça isto virar linear por conveniência:** o Inspector chama-o por
/// componente presente por quadro, e o custo de uma decisão de UI tem de ser invisível.
#[must_use]
pub fn desc_for(canonical_name: &str) -> Option<&'static ComponentDesc> {
    for family in FAMILIES {
        if let Ok(i) = family.binary_search_by_key(&canonical_name, |d| d.canonical_name) {
            return Some(&family[i]);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Attach, ObjectKind};
    use std::collections::BTreeSet;

    /// **Cada família ordenada, e nenhum nome repetido em todo o catálogo.**
    ///
    /// As duas metades da mesma lei: a ordem é o que faz [`desc_for`] achar, e a unicidade é
    /// o que faz a resposta ser UMA. ⚠️ Um nome duplicado em duas famílias não é erro de
    /// compilação e não faz `desc_for` falhar — ele devolve a primeira, em silêncio.
    #[test]
    fn the_catalog_is_sorted_and_unique() {
        for family in FAMILIES {
            for pair in family.windows(2) {
                assert!(
                    pair[0].canonical_name < pair[1].canonical_name,
                    "catalogo fora de ordem: '{}' vem depois de '{}' — desc_for faz busca \
                     binaria e devolveria None para um tipo que existe",
                    pair[0].canonical_name,
                    pair[1].canonical_name,
                );
            }
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for d in all() {
            assert!(
                seen.insert(d.canonical_name),
                "nome duplicado no catalogo: '{}' — desc_for devolveria a primeira em silencio",
                d.canonical_name,
            );
        }
    }

    /// **Todo `Authored` alcança algum objeto.**
    ///
    /// `applies_to` vazio é a forma de escrever "morto" sem o dizer: o componente existe, o
    /// censo do registo encontra-o, e a paleta nunca o oferece a ninguém. ⛔ Se um componente
    /// não deve ser oferecido, isso escreve-se [`Attach::Machinery`] — que é uma AFIRMAÇÃO,
    /// e o revisor lê-a.
    #[test]
    fn no_authored_component_is_unreachable() {
        for d in all() {
            if let Attach::Authored { applies_to } = d.attach {
                assert!(
                    !applies_to.is_empty(),
                    "'{}' e Authored com applies_to VAZIO — inalcancavel em todo objeto. \
                     Se e maquina, declare Attach::Machinery.",
                    d.canonical_name,
                );
            }
        }
    }

    /// **Todo tipo de objeto tem alguma coisa para lhe acrescentar.**
    ///
    /// A cobertura do outro lado: se um `ObjectKind` novo entrar no vocabulário e ninguém
    /// declarar `applies_to` para ele, a paleta abre **vazia** naquele objeto — e o artista
    /// lê isso como o botão estar partido, não como uma ausência de desenho.
    #[test]
    fn every_object_kind_has_something_to_offer() {
        for kind in ObjectKind::ALL {
            let n = all().filter(|d| d.is_offered_to(kind)).count();
            assert!(
                n > 0,
                "nenhum componente e oferecido a {:?} — a paleta abriria vazia nesse objeto",
                kind,
            );
        }
    }

    /// **O marcador de cada `ObjectKind` está no catálogo.**
    ///
    /// O `ObjectKind` deriva-se por PRESENÇA de um componente; se o nome do marcador não
    /// existir aqui, ou ele foi renomeado (e a derivação passou a nunca disparar), ou o
    /// vocabulário ganhou uma variante que ninguém ligou a nada.
    #[test]
    fn every_object_kind_marker_is_a_real_component() {
        for kind in ObjectKind::ALL {
            if let Some(marker) = kind.marker() {
                assert!(
                    desc_for(marker).is_some(),
                    "o marcador de {:?} e '{}', que nao esta no catalogo — a derivacao do \
                     tipo de objeto nunca dispararia",
                    kind,
                    marker,
                );
            }
        }
    }

    /// **Os `field_id` de um tipo são únicos e crescentes.**
    ///
    /// Crescentes porque a tabela é lida por humanos e por `ComponentDesc::field`; únicos
    /// porque dois campos com o mesmo id fazem um override alvejar o outro — que é
    /// exatamente o defeito que o id declarado existe para impedir.
    #[test]
    fn field_ids_are_unique_and_ascending_within_a_type() {
        for d in all() {
            for pair in d.fields.windows(2) {
                assert!(
                    pair[0].field_id < pair[1].field_id,
                    "'{}': field_id fora de ordem ou repetido ({} depois de {})",
                    d.canonical_name,
                    pair[1].field_id,
                    pair[0].field_id,
                );
            }
        }
    }

    /// **Um marcador de tamanho zero não tem campos; um tipo com campos não é marcador.**
    ///
    /// `FieldKind::Marker` diz *"a presença É o valor"*. Descrevê-lo com campos ao lado é
    /// afirmar as duas coisas ao mesmo tempo, e o Inspector teria de escolher uma.
    #[test]
    fn a_marker_field_is_the_only_field_of_its_type() {
        for d in all() {
            let markers = d
                .fields
                .iter()
                .filter(|f| f.kind == crate::FieldKind::Marker)
                .count();
            if markers > 0 {
                assert_eq!(
                    d.fields.len(),
                    1,
                    "'{}' mistura FieldKind::Marker com outros campos — a presenca e o valor, \
                     ou nao e",
                    d.canonical_name,
                );
            }
        }
    }
}
