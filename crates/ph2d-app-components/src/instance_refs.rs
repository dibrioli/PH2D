//! ⭐ **UMA tabela: cada referência DECLARADA tem aqui quem a reescreve** (ADR-0164 / F4.2).
//!
//! Uma referência guardada por identidade (`PhysicsJoint.body_a`, a corda de uma `PulleyWheel`,
//! o elo `InstanceOf.master`) não sobrevive a uma cópia: ela continua a apontar para o
//! **original**. É por isso que hoje *a junta de uma cópia prende os corpos do mestre*.
//!
//! # Porque isto é uma TABELA conferida, e não um `match` escrito à mão
//!
//! O descritor de componente já declara quais campos são referência
//! ([`ph2d_component_desc::RefKind`]) — foi para isto que a declaração existe. Se a lista dos
//! remapeadores vivesse solta ao lado dela, teríamos **duas respostas à mesma pergunta**, e a que
//! envelhece é a que o artista sente: um campo declarado sem remapeador é uma junta que prende no
//! sítio errado, calada.
//!
//! ⇒ o censo de dois lados em baixo: **todo campo `RefKind::Object` tem remapeador** *e* **todo
//! remapeador nomeia um campo declarado**. Declarar uma referência nova sem escrever quem a
//! reescreve **reprova a suíte**.
//!
//! ⚠️ **A shell é o único sítio onde isto pode viver:** o descritor está numa crate-folha que não
//! vê tipo nenhum, e as crates que possuem os campos (`ph2d-physics-ecs`, `ph2d-ecs`) não se
//! veem umas às outras. Cada uma escreve o remapeador **dela**; aqui só se juntam.

use ph2d_ecs::{Entity, World};
use std::collections::BTreeMap;

/// Reescreve as referências deste componente nas entidades dadas; devolve quantas mexeu.
type Remap = fn(&mut World, &[Entity], &BTreeMap<u64, u64>) -> usize;

/// **A tabela.** Uma linha por componente que declara uma referência a objeto.
pub(crate) const REMAPPERS: &[(&str, Remap)] = &[
    ("ph2d::ecs::InstanceOf", ph2d_ecs::remap_instance_of),
    (
        "ph2d::physics::PhysicsJoint",
        ph2d_physics_ecs::remap_joint_refs,
    ),
    (
        "ph2d::physics::PulleyWheel",
        ph2d_physics_ecs::remap_wheel_refs,
    ),
];

/// **Reescreve TODA referência a objeto** das entidades dadas através do mapa
/// `StableId do original → StableId da cópia`. Devolve quantos componentes mudaram.
///
/// ⚠️ Referência cujo alvo está **fora** do mapa fica como está — é o que mantém uma instância
/// pendurada no gancho do cenário de que o mestre pendia.
pub(crate) fn remap_object_refs(
    world: &mut World,
    entities: &[Entity],
    by_id: &BTreeMap<u64, u64>,
) -> usize {
    remap_object_refs_except(world, entities, by_id, &[])
}

/// **Este componente guarda referências a objeto?** — a pergunta que o sync faz antes de comparar
/// bytes.
///
/// ⚠️ Para quem carrega referência, `bytes(mestre) == bytes(instância)` **não** é a pergunta certa:
/// os dois lados nomeiam corpos diferentes de propósito, então a igualdade é *falsa* mesmo quando
/// nada mudou. Ver o sync, onde essa distinção vira duas rotas.
#[must_use]
pub(crate) fn carries_object_ref(canonical_name: &str) -> bool {
    REMAPPERS.iter().any(|(n, _)| *n == canonical_name)
}

/// ⭐⭐ **O mesmo, saltando os componentes nomeados** — e a lei que o exige é uma só:
///
/// > **O que não PROPAGA não se REMAPEIA.**
///
/// O remap existe para reescrever referências que acabaram de chegar **do mestre**. Uma referência
/// que o passe não trouxe não é uma referência ao mestre, e reescrevê-la é corrupção.
///
/// ⚠️ **Isto foi MEDIDO, não previsto** (F4.3): o sync remapeava tudo, e o `InstanceOf.master` da
/// raiz de uma instância É a identidade do mestre — logo uma chave do mapa. O primeiro passe
/// reescrevia-o para a identidade da **própria instância**, e a partir do segundo quadro a
/// instância dizia-se instância de si mesma: o sync deixava de a encontrar e **nada mais
/// propagava**. Calado, e com todos os gates verdes — porque nenhum deles corria o passe DUAS
/// vezes antes de medir. *A mutação que sobreviveu foi o que o revelou.*
pub(crate) fn remap_object_refs_except(
    world: &mut World,
    entities: &[Entity],
    by_id: &BTreeMap<u64, u64>,
    skip: &[&str],
) -> usize {
    REMAPPERS
        .iter()
        .filter(|(name, _)| !skip.contains(name))
        .map(|(_, f)| f(world, entities, by_id))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::REMAPPERS;
    use ph2d_component_desc::RefKind;

    /// Os componentes que DECLARAM ao menos uma referência a objeto.
    fn declared() -> Vec<&'static str> {
        ph2d_component_desc::all()
            .filter(|d| d.fields.iter().any(|f| f.is_ref == Some(RefKind::Object)))
            .map(|d| d.canonical_name)
            .collect()
    }

    /// ⭐ **Metade 1 — toda referência declarada tem quem a reescreva.**
    ///
    /// (Mutação: apagar uma linha da tabela ⇒ RED nomeando o componente órfão.)
    #[test]
    fn every_declared_object_ref_has_a_remapper() {
        for name in declared() {
            assert!(
                REMAPPERS.iter().any(|(n, _)| *n == name),
                "{name} declara uma referencia a objeto e NINGUEM a remapeia — a copia dele \
                 apontaria para o original, calada"
            );
        }
    }

    /// ⭐ **Metade 2 — todo remapeador nomeia uma referência declarada.**
    ///
    /// Sem esta metade a tabela apodrece do outro lado: um campo que deixe de ser referência (ou
    /// um componente renomeado) deixa aqui uma função que varre entidades por nada.
    #[test]
    fn every_remapper_names_a_declared_object_ref() {
        let declared = declared();
        for (name, _) in REMAPPERS {
            assert!(
                declared.contains(name),
                "o remapeador de {name} nao tem campo declarado `RefKind::Object` no catalogo"
            );
        }
    }

    /// ⚠️ **O controle positivo.** Com o catálogo vazio de referências as duas metades acima
    /// passariam por vacuidade — e é precisamente o estado em que este repo estava até a F4.2.
    #[test]
    fn the_census_is_not_vacuous() {
        assert!(
            declared().len() >= 3,
            "so' {} componentes declaram referencia — o censo acima esta' a medir quase nada",
            declared().len()
        );
        assert_eq!(declared().len(), REMAPPERS.len());
    }

    /// ⚠️ **As duas metades da declaração andam juntas:** `kind == Ref` ⟺ `is_ref.is_some()`.
    ///
    /// Descrever uma referência como `Int` poria um chip numérico a pedir a identidade de um
    /// corpo; declarar `Ref` sem dizer o que ela aponta deixaria o remap sem alvo.
    #[test]
    fn a_ref_field_declares_both_halves() {
        use ph2d_component_desc::FieldKind;
        for d in ph2d_component_desc::all() {
            for f in d.fields {
                assert_eq!(
                    f.kind == FieldKind::Ref,
                    f.is_ref.is_some(),
                    "{}.{} declara metade de uma referencia ({:?} / {:?})",
                    d.canonical_name,
                    f.name,
                    f.kind,
                    f.is_ref
                );
            }
        }
    }
}
