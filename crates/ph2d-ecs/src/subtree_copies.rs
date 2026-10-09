//! ⭐ **As cópias que o EDITOR fez** (doc ObjectsAndComponents 01 §6 e §13).
//!
//! *Duplicate*, *Make*, *Instantiate* e a colocação de um prefab anunciam aqui o par `(original,
//! cópia)` de cada entidade copiada; quem guarda documentos POR OBJECTO fora do mundo — a
//! Animação, na Timeline; os nós, no Motion — lê-o e dá à cópia o seu (escolha do dono, 08/10:
//! clips próprios). Um recurso e não uma chamada: as portas de cópia não conhecem os leitores
//! (ADR-0075).
//!
//! ⭐ **Um LEITOR por consumidor** ([`CopyReader`]): cada um lê cada par UMA vez, e o par sai quando
//! todos o leram. Drenar por um só (o desenho da onda 2) deixava o segundo consumidor sem cópias.
//!
//! ⛔ **A Fábrica não anuncia** (`instantiate_master_many`): as cópias de JOGO nascem e morrem às
//! dezenas, e uma bala que ganhasse clips encheria o documento a cada disparo.

use bevy_ecs::prelude::Resource;
use bevy_ecs::world::World;

use crate::DeepCopy;

/// Quem lê as cópias anunciadas.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CopyReader {
    /// A Animação (os clips de cada objecto).
    Timeline,
    /// O Motion (os nós de cada objecto Motion).
    Motion,
}

impl CopyReader {
    const ALL: [CopyReader; 2] = [CopyReader::Timeline, CopyReader::Motion];

    const fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// Os pares `(bits do original, bits da cópia)` anunciados, com os leitores que já os leram.
#[derive(Resource, Debug, Default)]
pub struct SubtreeCopies(Vec<((u64, u64), u8)>);

/// Anuncia uma cópia do editor.
pub fn announce_copy(world: &mut World, copy: &DeepCopy) {
    world
        .get_resource_or_insert_with(SubtreeCopies::default)
        .0
        .extend(
            copy.entities
                .iter()
                .map(|(s, d)| ((s.to_bits(), d.to_bits()), 0)),
        );
}

/// O que `reader` ainda não leu (vazio sem alocar quando não houve cópia); o par sai quando todos
/// os leitores o leram.
pub fn take_copies(world: &mut World, reader: CopyReader) -> Vec<(u64, u64)> {
    let Some(mut copies) = world.get_resource_mut::<SubtreeCopies>() else {
        return Vec::new();
    };
    if copies.0.is_empty() {
        return Vec::new();
    }
    let all = CopyReader::ALL.iter().fold(0u8, |m, r| m | r.bit());
    let mut out = Vec::new();
    for (pair, read) in &mut copies.0 {
        if *read & reader.bit() == 0 {
            *read |= reader.bit();
            out.push(*pair);
        }
    }
    copies.0.retain(|(_, read)| *read != all);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn announce(world: &mut World, pairs: &[(u64, u64)]) {
        world
            .get_resource_or_insert_with(SubtreeCopies::default)
            .0
            .extend(pairs.iter().map(|p| (*p, 0)));
    }

    /// ⭐ Cada leitor vê cada cópia UMA vez — e o segundo não fica sem ela.
    /// (Mutação: `take_copies` drenar tudo ⇒ o Motion lê vazio — RED.)
    #[test]
    fn each_reader_reads_each_copy_once() {
        let mut world = World::new();
        announce(&mut world, &[(1, 2), (3, 4)]);
        assert_eq!(
            take_copies(&mut world, CopyReader::Timeline),
            vec![(1, 2), (3, 4)]
        );
        assert!(take_copies(&mut world, CopyReader::Timeline).is_empty());
        announce(&mut world, &[(5, 6)]);
        assert_eq!(
            take_copies(&mut world, CopyReader::Motion),
            vec![(1, 2), (3, 4), (5, 6)]
        );
        assert_eq!(take_copies(&mut world, CopyReader::Timeline), vec![(5, 6)]);
        assert!(
            world.resource::<SubtreeCopies>().0.is_empty(),
            "lido por todos, sai"
        );
    }
}
