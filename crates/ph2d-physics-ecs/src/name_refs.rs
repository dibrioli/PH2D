//! **A resolução NOME → IDENTIDADE das referências da física** (ADR-0164 F1).
//!
//! # As duas perguntas que esta função separa
//!
//! Uma junta guarda *"a que corpo eu prendo"*. Até esta wave a resposta era o **hash do
//! nome** (`stable_name_id`), resolvido **por dispatch** pela ponte — e daí vinham os dois
//! defeitos que o `name.rs` já documentava:
//!
//! - **Renomear desligava a junta.** O hash mudava e a ponte deixava de achar o corpo, calada.
//! - **Copiar prendia no ORIGINAL.** A cópia recebe o nome `" (1)"`, o hash muda, e a junta
//!   copiada continua a nomear o mestre — o defeito que a wave da instância existe para curar.
//!
//! Hoje a junta guarda o [`ph2d_ecs::StableId`], que ninguém edita. Mas **autorar** por nome
//! continua certo: um humano aponta *"prende ao Poste"*, não *"prende ao 47"*. Esta função é a
//! costura entre as duas — ela corre **uma vez**, depois de a cena estar montada, e troca cada
//! hash de nome pela identidade do objeto que tem aquele nome.
//!
//! # Dois consumidores, e nenhum deles é «cada cena»
//!
//! 1. **O roteador das cenas de smoke** (`physics_smoke.rs`), depois do `match`. ⚠️ Ali e não
//!    em cada cena: 35 cenas que tivessem de se lembrar da chamada seriam 35 sítios onde
//!    esquecê-la dá uma junta que **não prende e não avisa**.
//! 2. **A migração v95 → v96**, depois do restore — os ficheiros antigos guardam hashes.
//!
//! ⚠️ **É idempotente na prática**: um `StableId` é um inteiro pequeno e sequencial (1, 2, 3…)
//! e o mapa só contém hashes FNV-1a de nomes, que são valores grandes. Correr duas vezes não
//! encontra nada na segunda.

use bevy_ecs::prelude::{Entity, World};
use ph2d_ecs::{Name, StableId, stable_name_id};
use std::collections::BTreeMap;

/// Quantas referências foram traduzidas.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ResolvedRefs {
    pub joints: usize,
    pub wheels: usize,
}

impl ResolvedRefs {
    #[must_use]
    pub const fn total(self) -> usize {
        self.joints + self.wheels
    }
}

/// **Troca todo hash-de-nome guardado numa referência da física pela identidade do objeto.**
///
/// Ver o cabeçalho do módulo. Devolve a contagem, para que quem chama possa dizer no log o que
/// aconteceu — um número zero num ficheiro que tinha juntas é o sinal de que a tradução não
/// encontrou os nomes.
pub fn resolve_body_names(world: &mut World) -> ResolvedRefs {
    ph2d_ecs::assign_missing_stable_ids(world);

    // hash-do-nome → identidade. ⚠️ Construído do MUNDO e não do registo: o que importa é
    // quem existe agora, e um nome que já não está em ninguém simplesmente não traduz (a
    // referência fica como estava, apontando para nada — que é o que ela já fazia).
    let by_name_hash: BTreeMap<u64, u64> = {
        let mut q = world.query::<(&Name, &StableId)>();
        q.iter(world)
            .map(|(n, s)| (stable_name_id(n.as_str()), s.0))
            .collect()
    };
    if by_name_hash.is_empty() {
        return ResolvedRefs::default();
    }

    let mut out = ResolvedRefs::default();

    // As juntas.
    let joints: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, bevy_ecs::prelude::With<crate::PhysicsJoint>>();
        q.iter(world).collect()
    };
    for e in joints {
        let Some(mut j) = world.get_mut::<crate::PhysicsJoint>(e) else {
            continue;
        };
        let mut hit = false;
        if let Some(&id) = by_name_hash.get(&j.body_a) {
            j.body_a = id;
            hit = true;
        }
        if let Some(&id) = by_name_hash.get(&j.body_b) {
            j.body_b = id;
            hit = true;
        }
        if hit {
            out.joints += 1;
        }
    }

    // As roldanas — `rope` (a corda a que ela pertence) e `body` (o corpo em que ela é
    // montada, `0` quando ela vive no cenário).
    let wheels: Vec<Entity> = {
        let mut q = world.query_filtered::<Entity, bevy_ecs::prelude::With<crate::PulleyWheel>>();
        q.iter(world).collect()
    };
    for e in wheels {
        let Some(mut w) = world.get_mut::<crate::PulleyWheel>(e) else {
            continue;
        };
        let mut hit = false;
        if let Some(&id) = by_name_hash.get(&w.rope) {
            w.rope = id;
            hit = true;
        }
        if w.body != 0
            && let Some(&id) = by_name_hash.get(&w.body)
        {
            w.body = id;
            hit = true;
        }
        if hit {
            out.wheels += 1;
        }
    }

    out
}

#[cfg(test)]
#[path = "name_refs_tests.rs"]
mod tests;
