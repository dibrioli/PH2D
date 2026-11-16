//! ⭐ **A ponte do `Motion Input`** (ciclo 19, doc 128 §3.6) — por quadro, o que cada objecto com
//! o componente publica, e os sinais do runtime, vão para o recurso [`MotionInputs`] que a
//! `ph2d-app-motion` publica para o grafo antes de cozer (no mesmo quadro: a simulação corre antes).
//!
//! - **Valor / Cor**: o campo do componente.
//! - **Vida**: `agora ÷ máximo` do `Health` do objecto From (a mesma leitura das barras).
//! - **Dano**: um DISPARO cada vez que essa razão desce.
//! - **Speed · Grounded · Action · Property** (onda 2): [`crate::motion_input_fonte`]; depois o
//!   Remap (e a curva) e o Smooth, só nos números.
//! - **Sinais**: um sinal do runtime dispara o gatilho com o nome dele.
//!
//! O estado da ponte (a vida do quadro anterior, o cursor dos sinais) é um recurso do mundo que
//! ninguém regista — fora do save e do undo, como o `HealthBarsState`.

use crate::motion_input_fonte::{self as fonte, Fontes};
use bevy_ecs::prelude::Resource;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_ecs::{MotionInput, MotionInputs, MotionSource, Name, SimWorld};
use ph2d_game_dicas::add::MOTION_INPUT;
use ph2d_runtime::{SignalOutbox, SignalReader};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Resource, Default)]
struct Ponte {
    vida_antes: BTreeMap<u64, f32>,
    /// O valor suavizado do quadro anterior, por objecto (o Smooth).
    suave: BTreeMap<u64, f32>,
    leitor: Option<SignalReader>,
}

/// Uma linha deste quadro: `(escritor, nome da entrada, o que publica)`.
enum Publica {
    Valor(Valor),
    Disparo,
    Nada,
}

/// Corre UMA vez por quadro, depois da física (o `HealthNow`) e antes do Motion.
pub fn frame(sim: &mut SimWorld, outbox: &SignalOutbox, fontes: &Fontes<'_>) {
    let world = sim.world_mut();
    ph2d_ecs::motion_input::rotula_entradas(world, &MOTION_INPUT.key.tr());
    let mut ponte = world.remove_resource::<Ponte>().unwrap_or_default();
    // `bool` = a fonte existe neste quadro; sem ela a linha SAI (o `value.input` volta ao número
    // dele), nunca fica presa no último valor.
    let mut linhas: Vec<(Escritor, String, Publica, bool)> = Vec::new();
    let mut vida_agora = BTreeMap::new();
    let mut suave_agora = BTreeMap::new();
    let mut q = world.query::<(bevy_ecs::entity::Entity, &MotionInput)>();
    let lidos: Vec<_> = q.iter(world).map(|(e, m)| (e, m.clone())).collect();
    for (e, m) in lidos {
        let bits = e.to_bits();
        // O escritor fala pelo DONO (o pai, MI-10): é o nome que o artista vê e a ordem de «quem escreve primeiro».
        let dono = ph2d_ecs::motion_input::dono(world, e);
        let escritor = Escritor::Objeto {
            nome: world
                .get::<Name>(dono)
                .map(|n| n.0.clone())
                .unwrap_or_default(),
            bits,
        };
        let alvo = fonte::alvo(world, e, m.from);
        let vida = || {
            alvo.and_then(|a| crate::health_bar_bridge::vida_da_barra(world, a))
                .map(|(agora, max)| agora / max)
        };
        let mut fonte_viva = true;
        let numero = match m.source {
            MotionSource::Value => Some(m.value),
            MotionSource::Health => vida(),
            MotionSource::Speed => alvo.and_then(|a| fonte::velocidade(world, fontes.physics, a)),
            MotionSource::Grounded => alvo.and_then(|a| fonte::no_chao(world, fontes.physics, a)),
            MotionSource::Action => fonte::accao(fontes.accoes, &m.action),
            MotionSource::Property => {
                alvo.and_then(|a| fonte::propriedade(fontes.registry, world, a, &m.property))
            }
            MotionSource::Color | MotionSource::Damage => None,
        };
        let o_que = match m.source {
            MotionSource::Color => Publica::Valor(Valor::Cor(m.color)),
            MotionSource::Damage => match vida() {
                Some(r) => {
                    vida_agora.insert(bits, r);
                    if ponte.vida_antes.get(&bits).is_some_and(|&antes| r < antes) {
                        Publica::Disparo
                    } else {
                        Publica::Nada
                    }
                }
                None => {
                    fonte_viva = false;
                    Publica::Nada
                }
            },
            _ => match numero {
                Some(v) => {
                    let v = fonte::remapeia(&m, v);
                    let v = fonte::suaviza(ponte.suave.get(&bits).copied(), v, fontes.dt, m.smooth);
                    suave_agora.insert(bits, v);
                    Publica::Valor(Valor::Numero(v))
                }
                None => {
                    fonte_viva = false;
                    Publica::Nada
                }
            },
        };
        linhas.push((escritor, m.name.trim().to_string(), o_que, fonte_viva));
    }
    let leitor = ponte.leitor.get_or_insert_with(|| SignalReader::at(outbox));
    let sinais: Vec<String> = outbox.read(leitor).map(|s| s.name.to_string()).collect();

    let mut m = world.get_resource_or_insert_with(MotionInputs::default);
    let mut vivos: BTreeSet<(u64, String)> = BTreeSet::new();
    for (escritor, nome, o_que, fonte_viva) in linhas {
        if let (Escritor::Objeto { bits, .. }, true) = (&escritor, fonte_viva) {
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
    ponte.suave = suave_agora;
    world.insert_resource(ponte);
}

#[cfg(test)]
#[path = "motion_input_bridge_tests.rs"]
mod tests;
