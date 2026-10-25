//! ⭐ **O passe que dá um id a cada entrada da pilha do Vetor** (F31 §B.3) e publica o espelho
//! [`IdsDoVetor`].
//!
//! Corre a TODO quadro antes da captura do undo (a shell chama-o logo antes do `post_frame_undo`)
//! ⇒ o id nasce no mesmo passo de undo da entrada, e desfazer/refazer devolve o mesmo id.
//!
//! - Entrada com id `NONE`, ou repetido DENTRO do mesmo caminho (tintas e efeitos juntos), recebe um
//!   novo do [`StableIdCounter`] do mundo.
//! - Antes de alocar, o contador reconcilia-se com o maior id da cena INTEIRA: a `VecScene` não está
//!   no mundo, e o `reconcile_at_least` do load não a vê.
//! - Barato quando nada falta: uma varredura sem alocar; o espelho só se reescreve se mudou.

use std::collections::BTreeMap;

use ph2d_ecs::canal_vetor_ids::{IdsDaForma, IdsDoVetor};
use ph2d_ecs::{Entity, ItemId, SimWorld, StableIdCounter, VecFilter, VecPathRef};
use ph2d_vec_scene::{VecPath, VecScene};

/// Os ids da pilha de `p`: tintas e efeitos, na ordem.
fn ids(p: &VecPath) -> impl Iterator<Item = ItemId> + '_ {
    p.paints
        .iter()
        .map(|e| e.id)
        .chain(p.effects.iter().map(|e| e.id))
}

/// `true` se `p` tem um id `NONE` ou repetido.
fn falta(p: &VecPath) -> bool {
    let n = p.paints.len() + p.effects.len();
    ids(p)
        .enumerate()
        .any(|(a, x)| x.is_none() || ids(p).skip(a + 1).take(n).any(|y| y == x))
}

/// Atribui os ids que faltam em `scene`; devolve quantos atribuiu.
pub fn atribuir(sim: &mut SimWorld, scene: &mut VecScene) -> usize {
    if !scene.paths().iter().any(falta) {
        return 0;
    }
    let maior = scene
        .paths()
        .iter()
        .flat_map(ids)
        .map(|i| u64::from(i.0))
        .max()
        .unwrap_or(0);
    let mut contador = sim
        .world_mut()
        .remove_resource::<StableIdCounter>()
        .unwrap_or_default();
    contador.reconcile_at_least(maior.saturating_add(1));
    let mut n = 0;
    for p in scene.paths_mut() {
        if !falta(p) {
            continue;
        }
        let mut vistos: Vec<ItemId> = Vec::new();
        let VecPath {
            paints, effects, ..
        } = p;
        let slots = paints
            .iter_mut()
            .map(|e| &mut e.id)
            .chain(effects.iter_mut().map(|e| &mut e.id));
        for id in slots {
            if id.is_none() || vistos.contains(id) {
                *id = contador.aloca_item();
                n += 1;
            }
            vistos.push(*id);
        }
    }
    sim.world_mut().insert_resource(contador);
    n
}

/// Publica o [`IdsDoVetor`]: uma entrada por entidade com [`VecPathRef`] cujos ids estão todos
/// atribuídos (tintas e efeitos do caminho, filtros do [`VecFilter`]); quem saiu sai do mapa.
pub fn publicar(sim: &mut SimWorld, scene: &VecScene) {
    let mut q = sim
        .world_mut()
        .query::<(Entity, &VecPathRef, Option<&VecFilter>)>();
    let w = sim.world();
    let vivas = q.iter(w).filter_map(|(e, r, f)| {
        let p = scene.path(r.0)?;
        let filtros = f.map_or(&[][..], VecFilter::ids);
        let completa = !ids(p).chain(filtros.iter().copied()).any(ItemId::is_none);
        completa.then_some((e.to_bits(), p, filtros))
    });
    let mut n = 0usize;
    let mesmo = w.get_resource::<IdsDoVetor>().is_some_and(|v| {
        vivas.clone().all(|(bits, p, filtros)| {
            n += 1;
            v.0.get(&bits).is_some_and(|x| {
                x.tintas.iter().copied().eq(p.paints.iter().map(|e| e.id))
                    && x.efeitos.iter().copied().eq(p.effects.iter().map(|e| e.id))
                    && x.filtros == filtros
            })
        }) && v.0.len() == n
    });
    if mesmo {
        return;
    }
    let novo: BTreeMap<u64, IdsDaForma> = vivas
        .map(|(bits, p, filtros)| {
            let forma = IdsDaForma {
                tintas: p.paints.iter().map(|e| e.id).collect(),
                efeitos: p.effects.iter().map(|e| e.id).collect(),
                filtros: filtros.to_vec(),
            };
            (bits, forma)
        })
        .collect();
    sim.world_mut().insert_resource(IdsDoVetor(novo));
}

/// O passe inteiro: atribui e publica. Ver o cabeçalho do módulo.
pub fn passe(sim: &mut SimWorld, scene: &mut VecScene) {
    atribuir(sim, scene);
    publicar(sim, scene);
}

#[cfg(test)]
#[path = "ids_da_pilha_tests.rs"]
mod tests;
