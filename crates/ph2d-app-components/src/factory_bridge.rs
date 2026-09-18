//! ⭐⭐⭐ **A PONTE da FÁBRICA e da MORTE** (TOP-20 #11 e #12) — os factos que a lei pura devolveu
//! viram objectos no mundo, e vice-versa.
//!
//! # A fronteira, e porque ela é esta
//!
//! `ph2d_ecs::tick_factories` decide **quem nasce e onde**, e `tick_lifetimes`/`reap_outside`
//! decidem **quem morre** — as três são puras sobre o mundo e não lhe tocam. Aqui é onde o mundo
//! muda, e é a mesma fronteira que o `SignalActions` já usa: *a lei devolve factos, a ponte
//! aplica-os.*
//!
//! ⚠️ **E é por isso que ela vive nesta crate e não na `ph2d-ecs`:** instanciar uma receita passa
//! por [`crate::instantiate::instantiate_master_many`], que sabe de documentos possuídos, de
//! variantes e de remapeamento de referências — coisas da FAMÍLIA, não do ECS.
//!
//! # ⚠️ A morte é ADIADA, e o dreno é UM
//!
//! O oráculo (Godot) mediu que um nó pedido para morrer continua **válido, na árvore e em toda
//! consulta** até ao fim do quadro. Aqui é igual por construção: as duas leis de morte produzem
//! factos e este dreno corre uma vez, depois de todos os produtores. ⚠️ **E ele DEDUPLICA** — uma
//! bala pode morrer de velha e por sair do ecrã no mesmo quadro, e `despawn` duas vezes na mesma
//! entidade é um pânico.

use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{Birth, Death, Entity, SimWorld, Spawned, StableId, Transform, entity_of_stable_id};
use std::collections::BTreeSet;

/// O que a ponte pôs no mundo neste quadro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BirthReport {
    /// Quantas cópias nasceram.
    pub nasceram: usize,
    /// Quantos pedidos a porta recusou — a receita não é um mestre, ou já não existe.
    ///
    /// ⚠️ **Contados e não silenciados**: o painel di-lo ao artista, e o log do smoke imprime-o.
    /// Um pedido recusado em silêncio lê-se como *«a fábrica está partida»*.
    pub recusadas: usize,
}

/// ⭐⭐ **Põe no mundo as cópias que a lei pediu.**
///
/// # ⚠️ Os pedidos vêm AGRUPADOS por receita, e é isso que torna a porta em lote alcançável
///
/// `tick_factories` devolve os nascimentos pela ordem da identidade da fábrica, logo os de uma
/// mesma fábrica são **contíguos**. Este laço fecha cada grupo e chama
/// [`crate::instantiate::instantiate_master_many`] **uma vez por grupo** — sem isso o lote não
/// serviria de nada, porque o preço que ele corta é por CHAMADA (medido: 256 cópias numa cena de
/// 10 000 custam `133,8 ms` uma a uma contra `2,6 ms` em lote).
///
/// ⚠️ **A pose é escrita DEPOIS de instanciar**, no `Transform` da raiz: a cópia nasce com a pose
/// autorada da receita, e é a fábrica que decide onde ela aterra.
pub fn apply_births(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    births: &[Birth],
    tick: u64,
) -> BirthReport {
    let mut out = BirthReport::default();
    let mut i = 0usize;
    while i < births.len() {
        let j = births[i + 1..]
            .iter()
            .position(|b| b.master != births[i].master || b.factory != births[i].factory)
            .map_or(births.len(), |k| i + 1 + k);
        let grupo = &births[i..j];
        i = j;
        let quem = grupo[0].factory;
        let by = sim.world().get::<StableId>(quem).map_or(0, |s| s.0);
        let Some(mestre) = entity_of_stable_id(sim.world_mut(), StableId(grupo[0].master)) else {
            out.recusadas += grupo.len();
            continue;
        };
        let n = u32::try_from(grupo.len()).unwrap_or(u32::MAX);
        let Ok(copias) = crate::instantiate::instantiate_master_many(
            sim,
            registry,
            mestre,
            // ⚠️ **Na raiz da cena, e não debaixo da fábrica**: uma cópia filha herdaria a pose
            // dela e andaria com ela, e uma fábrica que se move arrastaria o que já nasceu — o
            // defeito que o emissor do Motion mediu e nomeou (*«arrastar o emissor arrastava o
            // penacho inteiro»*).
            None,
            docs,
            crate::instantiate::ArtLink::Own,
            n,
        ) else {
            out.recusadas += grupo.len();
            continue;
        };
        for (copia, pedido) in copias.into_iter().zip(grupo) {
            if let Some(mut t) = sim.world_mut().get_mut::<Transform>(copia) {
                t.translation.x = pedido.at[0];
                t.translation.y = pedido.at[1];
                // ⭐⭐ **E a MIRA, quando a fábrica a pediu** (o gatilho, 2026-09-18).
                //
                // ⚠️ **`Option` e não um `f32` com um neutro:** `0` é um ângulo legítimo (apontar
                // para a direita), logo um sentinela aqui tornaria essa mira inexprimível. O
                // caminho de omissão é `None` ⇒ a cópia fica com a rotação do MOLDE, byte a byte
                // como antes desta wave.
                if let Some(aim) = pedido.aim {
                    t.rotation = aim;
                }
            }
            sim.world_mut().entity_mut(copia).insert(Spawned {
                by,
                born_tick: tick,
            });
            out.nasceram += 1;
        }
    }
    out
}

/// ⭐⭐ **Tira do mundo quem morreu** — o dreno único do fim do quadro.
///
/// ⚠️ **Deduplica**, e a razão é concreta: as duas leis de morte correm no mesmo quadro e uma bala
/// que se esgota ao sair do ecrã aparece nas duas listas. ⛔ `despawn` na mesma entidade duas vezes
/// entra em pânico.
///
/// ⚠️ **Despawn RECURSIVO** (o do `bevy_ecs`): uma cópia é uma subárvore, e deixar os filhos para
/// trás daria órfãos sem `ChildOf` que a captura voltaria a ver como raízes do documento.
pub fn apply_deaths(sim: &mut SimWorld, deaths: &[Death]) -> usize {
    let unicos: BTreeSet<Entity> = deaths.iter().map(|d| d.entity).collect();
    let mut n = 0;
    for e in unicos {
        if sim.world().get_entity(e).is_ok() {
            sim.world_mut().entity_mut(e).despawn();
            n += 1;
        }
    }
    n
}

/// **Varre tudo o que nasceu numa corrida** — o que rebobinar faz.
///
/// ⚠️ **Ela existe porque o `Ctrl+Z` e o abrir um projecto já a fazem DE GRAÇA**: os dois
/// reconstroem o mundo a partir de um `WorldSnapshot`, e as cópias não estão lá. O que sobra é o
/// gesto que **não** passa por um restore — rebobinar o relógio —, e é para ele que esta porta
/// existe.
pub fn sweep_spawned(sim: &mut SimWorld) -> usize {
    let mut q = sim.world_mut().query::<(Entity, &Spawned)>();
    let todos: Vec<Entity> = q.iter(sim.world()).map(|(e, _)| e).collect();
    let mut n = 0;
    for e in todos {
        if sim.world().get_entity(e).is_ok() {
            sim.world_mut().entity_mut(e).despawn();
            n += 1;
        }
    }
    n
}

#[cfg(test)]
#[path = "factory_bridge_tests.rs"]
mod tests;
