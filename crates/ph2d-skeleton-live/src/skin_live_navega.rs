//! ⭐⭐⭐ **NAVEGAR O ESQUELETO** — as três perguntas que não são prender nem recozer.
//!
//! ⚠️ **Este irmão nasceu de um TECTO DE LOC vermelho** (2026-09-20, `skin_live.rs` a `708` contra
//! `700`), e o corte é por RESPONSABILIDADE e não pelo tamanho: *«onde acaba uma corrente»*,
//! *«que arte se move com este osso»* e *«que ossos estão acima deste»* são perguntas sobre a
//! FORMA do esqueleto — o ficheiro-pai responde a *«prender»* e *«recozer»*.
//!
//! ⛔ **Os caminhos dos chamadores NÃO mudam:** o pai re-exporta as três, logo
//! `skin_live::chain_to` continua a ser `skin_live::chain_to`. *Um corte de tecto que obrigasse
//! seis chamadores noutra crate a mudar de nome pagaria o tecto com risco de merge.*

use super::{BoneIndex, ossos_da_cena, skeleton_of};
use ph2d_ecs::{ChildOf, Entity, SimWorld};
use ph2d_skeleton_ecs::{Bone, SkinBind};

/// ⭐⭐⭐ **AS PONTAS DE CORRENTE** — os ossos que não têm osso filho, em bits.
///
/// São eles, e só eles, que ganham a alça do *end effector*: em toda outra junta a ponta de um osso
/// **é** a raiz do seguinte, e ali já há uma bolinha com outro verbo.
pub fn chain_ends(sim: &SimWorld) -> Vec<u64> {
    let ossos: Vec<Entity> = ossos_da_cena(sim).into_iter().map(|(e, _)| e).collect();
    let mut out: Vec<u64> = ossos
        .iter()
        .filter(|&&e| {
            sim.world()
                .get::<ph2d_ecs::Children>(e)
                .is_none_or(|f| f.iter().all(|c| sim.world().get::<Bone>(*c).is_none()))
        })
        .map(|e| e.to_bits())
        .collect();
    out.sort_unstable();
    out
}

/// ⭐⭐⭐ **AS IMAGENS PRESAS A ESTE ESQUELETO** — a arte que se move quando este osso se move.
///
/// ⚠️ **Ela existe porque um OSSO não tem silhueta.** O onion mostra o passado e o futuro do que o
/// animador tem na mão, e o que ele tem na mão quando posa é um osso — a coisa que se vê mover é a
/// ARTE. Sem esta porta o onion de um personagem riggado não mostrava nada: a imagem não está
/// animada (quem tem keys são os ossos) e o osso não tem instância de desenho.
///
/// ⚠️ **O critério é o TENDÃO, e não a hierarquia:** prender é uma decisão autorada, e uma imagem
/// pode estar pendurada em qualquer sítio da cena. Um tendão cujo osso não está no índice (apagado)
/// simplesmente não conta, como em todo o resto deste módulo.
///
/// ⛔ **Só IMAGENS.** Uma forma vectorial presa ao mesmo esqueleto é desenhada pelo Vello e não tem
/// `RenderInstance` — o passe que desenha fantasmas é o de sprites, logo ela não pode ser ghostada
/// por aqui. Limite NOMEADO, não esquecimento.
#[must_use]
pub fn skinned_images_of_skeleton(sim: &SimWorld, seed: Entity, index: &BoneIndex) -> Vec<Entity> {
    let ossos: std::collections::BTreeSet<Entity> =
        skeleton_of(sim, Some(seed)).into_iter().collect();
    if ossos.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Entity> = sim
        .world()
        .iter_entities()
        .filter(|er| crate::skin_image::is_skinned_image(sim.world(), er.id()))
        .filter(|er| {
            er.get::<SkinBind>().is_some_and(|s| {
                s.tendons
                    .iter()
                    .any(|t| index.get(&t.bone).is_some_and(|b| ossos.contains(b)))
            })
        })
        .map(|er| er.id())
        .collect();
    // A ordem de `iter_entities` é a dos arquétipos; ordenar deixa a lista determinística entre
    // quadros, que é o que um consumidor de desenho precisa.
    out.sort_by_key(|e| e.to_bits());
    out
}

/// **A CORRENTE que acaba neste osso** — da raiz até ele, na ordem em que a cinemática a resolve.
///
/// ⚠️ Ela sobe enquanto o PAI também for osso, que é a mesma regra do [`skeleton_of`] — parar no
/// primeiro pai não-osso é o que permite pendurar um esqueleto dentro de um grupo sem ele deixar de
/// ser um esqueleto.
pub fn chain_to(sim: &SimWorld, bits: u64) -> Vec<Entity> {
    let mut fila = vec![Entity::from_bits(bits)];
    while let Some(&e) = fila.last() {
        let Some(p) = sim.world().get::<ChildOf>(e).map(ChildOf::parent) else {
            break;
        };
        if sim.world().get::<Bone>(p).is_none() {
            break;
        }
        fila.push(p);
    }
    fila.reverse();
    fila
}
