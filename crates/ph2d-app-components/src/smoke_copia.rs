//! ⛔⛔ **A CÓPIA que a fábrica faria de um molde, pelo nome** — só para os gates das cenas.
//!
//! ⚠️ Partilhada pelas duas cenas da vida (`=1` e `=2`): escrita duas vezes, uma delas deixaria de
//! passar pela porta de cópia do produto no dia em que a outra mudasse.

use ph2d_ecs::MasterRoot;
use ph2d_ecs::{Name, SimWorld};

/// ⛔⛔ **A CÓPIA que a fábrica faria de um molde, pelo nome** — pela porta de cópia do PRODUTO
/// ([`crate::instantiate::instantiate_master_many`]) e com o registo que o produto monta.
///
/// ⚠️ **Nasceu do smoke do dono** (*«ninguém sumiu ao levar muitos tiros»*): a 1.ª redacção destes
/// gates lia a RECEITA do mundo e montava o alvo à mão — e a cópia profunda leva só os componentes
/// REGISTADOS, que a `Health`/`Damage` não eram. *Uma fixtura que monta à mão o que o produto COPIA
/// mede outro programa.* ⇒ os números saem da CÓPIA, e uma cópia sem vida faz o gate reprovar alto.
pub(crate) fn copia(sim: &mut SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let molde = {
        let w = sim.world_mut();
        let mut q =
            w.query_filtered::<(ph2d_ecs::Entity, &Name), bevy_ecs::query::With<MasterRoot>>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == nome)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("o molde «{nome}» nao esta' na cena"))
    };
    let registo = crate::component_registry_for_tests::registo();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    crate::instantiate::instantiate_master_many(
        sim,
        &registo,
        molde,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
        1,
    )
    .unwrap_or_else(|e| panic!("«{nome}» recusou a cópia: {e:?}"))[0]
}
