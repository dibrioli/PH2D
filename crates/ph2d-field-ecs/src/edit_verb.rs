//! ⭐⭐⭐ **O VERBO POR FORMA** — quem dobra sobre quem, e com que operação.
//!
//! A lei está escrita uma vez, em [`ph2d_field::fold_verb`]; aqui vive a metade que só a **cena**
//! sabe responder: *quem é a base*, *quem herda*, e *o que o painel pode oferecer antes do clique*.
//!
//! ⚠️ **Arquivo irmão, e não mais linhas no [`crate::edit_tree`]** — ele está em 436 LOC e a regra da
//! casa é *split, nunca allowlist*. O corte aqui é o mesmo do `edit_pose`/`edit_params`: uma
//! pergunta, um ficheiro.
//!
//! ⚠️ **Os predicados desta família são consumidos pelo PAINEL e pela HIERARQUIA**, e é por isso que
//! nenhum deles muta o mundo. É a lição do [`crate::can_wrap`], paga por uma wave inteira em que o
//! gesto de criar grupo ficou inalcançável porque o painel escreveu a **sua** cópia da regra.

use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_field::{FieldError, NodeShape, Op};

use crate::cook::{contributes, kids};
use crate::{FieldNode, FieldVerb};

/// ⭐ **O papel desta forma na receita do grupo dela.**
///
/// ⚠️ **Três estados e não dois**, e o terceiro é o que faz a UI funcionar: *herdado* e *próprio*
/// produzem a mesma operação e **não** são a mesma coisa para quem olha. Um chip aceso por herança
/// diz *«é isto que acontece»*; aceso por escolha diz *«fui eu que pedi»* — e é a diferença entre
/// mudar o padrão do grupo e mudar só esta forma.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VerbRole {
    /// ⭐ **A BASE**: a primeira forma que contribui, a que **semeia** o acumulado. O verbo dela
    /// não é perguntado por ninguém — ver [`ph2d_field::fold_verb`].
    Base,
    /// Dobra com o verbo **do pai**, porque não trouxe nenhum.
    Inherited(Op),
    /// Dobra com o verbo **dela própria**.
    Own(Op),
}

impl VerbRole {
    /// A operação que de facto acontece — `None` só na base, que não dobra sobre nada.
    #[must_use]
    pub fn op(self) -> Option<Op> {
        match self {
            VerbRole::Base => None,
            VerbRole::Inherited(op) | VerbRole::Own(op) => Some(op),
        }
    }

    /// Esta forma **escolheu** o verbo dela? (`false` na base e em quem herda.)
    #[must_use]
    pub fn is_own(self) -> bool {
        matches!(self, VerbRole::Own(_))
    }
}

/// O verbo **autorado** neste nó — `None` quer dizer *«herda o do pai»*. Ver [`crate::FieldVerb`].
#[must_use]
pub fn verb_of(world: &World, entity: Entity) -> Option<Op> {
    world.get::<FieldVerb>(entity).map(|v| v.op)
}

/// ⭐ **Escreve o verbo desta forma** — `None` devolve-a à herança.
///
/// ⚠️ **`None` REMOVE o componente**, e não escreve um verbo «neutro»: a ausência é que significa
/// herança ([`crate::FieldVerb`]), e um valor guardado a fingir de ausente deixaria o nó a
/// discordar do pai em silêncio no dia em que o padrão do grupo mudasse.
///
/// # Errors
/// [`FieldError::BadRoot`] se a entidade não é um nó de modelagem — um verbo pendurado noutra coisa
/// não é lido por ninguém, e escrevê-lo seria estado inalcançável.
pub fn set_verb(world: &mut World, entity: Entity, verb: Option<Op>) -> Result<(), FieldError> {
    if world.get::<FieldNode>(entity).is_none() {
        return Err(FieldError::BadRoot);
    }
    match verb {
        Some(op) => {
            world.entity_mut(entity).insert(FieldVerb { op });
        }
        None => {
            world.entity_mut(entity).remove::<FieldVerb>();
        }
    }
    Ok(())
}

/// ⭐⭐ **O papel desta forma**, ou `None` quando ela não participa de receita nenhuma.
///
/// Devolve `None` para: quem não é nó de modelagem · a **raiz** da peça (não há nada acima com que
/// dobrar) · quem está debaixo de uma **folha** (o cozimento não olha para esses filhos — é a lei do
/// [`crate::promote_leaf_hosts`], e entre o gesto e a derivação do quadro isto pode ser verdade por
/// um quadro) · e quem **não contribui** (escondido, ou um grupo vazio).
///
/// # ⚠️ A BASE é a primeira que CONTRIBUI, não a primeira da lista
///
/// Esconder o primeiro filho promove o segundo a base — porque é exactamente isso que o cozimento
/// faz. A pergunta é respondida pelo [`contributes`], que é a **mesma** função que o `emit` usa;
/// duas cópias divergiriam e o sintoma seria a Hierarquia a dizer `BASE` numa linha que subtrai.
#[must_use]
pub fn verb_role(world: &World, entity: Entity) -> Option<VerbRole> {
    world.get::<FieldNode>(entity)?;
    if !contributes(world, entity) {
        return None;
    }
    let parent = world.get::<ChildOf>(entity)?.0;
    let NodeShape::Combine(parent_op) = world.get::<FieldNode>(parent)?.shape else {
        return None;
    };
    let first = kids(world, parent)
        .into_iter()
        .find(|k| contributes(world, *k))?;
    if first == entity {
        return Some(VerbRole::Base);
    }
    Some(match verb_of(world, entity) {
        Some(op) => VerbRole::Own(op),
        None => VerbRole::Inherited(parent_op),
    })
}
