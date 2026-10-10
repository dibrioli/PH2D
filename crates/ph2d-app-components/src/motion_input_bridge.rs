//! ⭐ **A ponte do `Motion Input`** (ciclo 19, doc 128 §3.6) — por quadro, o que cada objecto com
//! o componente publica, e os sinais do runtime, vão para o recurso [`MotionInputs`] que a
//! `ph2d-app-motion` publica para o grafo antes de cozer (no mesmo quadro: a simulação corre antes).
//!
//! - **Valor / Cor**: o campo do componente.
//! - **Vida**: `agora ÷ máximo` do `Health` do próprio objecto (a mesma leitura das barras).
//! - **Dano**: um DISPARO cada vez que essa razão desce.
//! - **Sinais**: um sinal do runtime dispara o gatilho com o nome dele.
//!
//! O estado da ponte (a vida do quadro anterior, o cursor dos sinais) é um recurso do mundo que
//! ninguém regista — fora do save e do undo, como o `HealthBarsState`.

use bevy_ecs::prelude::Resource;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_ecs::{MotionInput, MotionInputs, MotionSource, Name, SimWorld};
use ph2d_runtime::{SignalOutbox, SignalReader};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Resource, Default)]
struct Ponte {
    vida_antes: BTreeMap<u64, f32>,
    leitor: Option<SignalReader>,
}

/// Uma linha deste quadro: `(escritor, nome da entrada, o que publica)`.
enum Publica {
    Valor(Valor),
    Disparo,
    Nada,
}

/// Corre UMA vez por quadro, depois da física (o `HealthNow`) e antes do Motion.
pub fn frame(sim: &mut SimWorld, outbox: &SignalOutbox) {
    let world = sim.world_mut();
    let mut ponte = world.remove_resource::<Ponte>().unwrap_or_default();
    let mut linhas: Vec<(Escritor, String, Publica)> = Vec::new();
    let mut vida_agora = BTreeMap::new();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &MotionInput, Option<&Name>)>();
    let lidos: Vec<_> = q
        .iter(world)
        .map(|(e, m, n)| (e, m.clone(), n.map(|n| n.0.clone()).unwrap_or_default()))
        .collect();
    for (e, m, nome_obj) in lidos {
        let bits = e.to_bits();
        let escritor = Escritor::Objeto {
            nome: nome_obj,
            bits,
        };
        let vida =
            || crate::health_bar_bridge::vida_da_barra(world, e).map(|(agora, max)| agora / max);
        let o_que = match m.source {
            MotionSource::Value => Publica::Valor(Valor::Numero(m.value)),
            MotionSource::Color => Publica::Valor(Valor::Cor(m.color)),
            MotionSource::Health => {
                vida().map_or(Publica::Nada, |r| Publica::Valor(Valor::Numero(r)))
            }
            MotionSource::Damage => match vida() {
                Some(r) => {
                    vida_agora.insert(bits, r);
                    if ponte.vida_antes.get(&bits).is_some_and(|&antes| r < antes) {
                        Publica::Disparo
                    } else {
                        Publica::Nada
                    }
                }
                None => Publica::Nada,
            },
        };
        linhas.push((escritor, m.name.trim().to_string(), o_que));
    }
    let leitor = ponte.leitor.get_or_insert_with(|| SignalReader::at(outbox));
    let sinais: Vec<String> = outbox.read(leitor).map(|s| s.name.to_string()).collect();

    let mut m = world.get_resource_or_insert_with(MotionInputs::default);
    let mut vivos: BTreeSet<(u64, String)> = BTreeSet::new();
    for (escritor, nome, o_que) in linhas {
        if let Escritor::Objeto { bits, .. } = &escritor {
            vivos.insert((*bits, nome.clone()));
        }
        match o_que {
            Publica::Valor(v) => m.set(&nome, escritor, v),
            Publica::Disparo => m.fire(&nome, escritor),
            Publica::Nada => {}
        }
    }
    m.retem_objetos(|_, bits, nome| vivos.contains(&(bits, nome.to_string())));
    for s in sinais {
        m.fire(&s, Escritor::Sinal);
    }
    ponte.vida_antes = vida_agora;
    world.insert_resource(ponte);
}

#[cfg(test)]
#[path = "motion_input_bridge_tests.rs"]
mod tests;
