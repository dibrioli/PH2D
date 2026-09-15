//! ⭐⭐⭐ **QUEM LÊ O TECLADO — uma porta, e não uma resposta por metade do gesto.**
//!
//! # ⛔⛔ Por que esta porta existe (report do dono, 2026-09-15: *«nada se move»*)
//!
//! A entrega do dedo ao mundo é escrita em **duas** metades, e elas correm em quadros diferentes
//! do mesmo tique:
//!
//! 1. a **ENTREGA** — a shell resolve o Input Map e chama
//!    [`crate::PhysicsBridge::set_player_input`] para cada player da cena (e é a contagem que ela
//!    devolve que decide se este tique descreve uma CORRIDA que a fita grava);
//! 2. o **REPLAY** — [`crate::PhysicsBridge`] pergunta à fita o que o dedo fez naquele tique e
//!    reinstala-o ([`super::bridge::tape`]).
//!
//! Quando o TOP-20 #13 trouxe o segundo controlador, só a metade **2** aprendeu a conhecê-lo. A
//! metade **1** continuou a varrer só o [`PlatformPlayer`] ⇒ o mover de vista de cima nunca
//! recebia entrada, e — porque a contagem dava `0` — a fita **também nunca gravava**, matando o
//! caminho do replay pela mesma razão. Os 24 gates da wave entravam pelo canal interno e ficavam
//! todos verdes.
//!
//! ⇒ *a pergunta passa a ter UMA porta, e a próxima variante é uma linha aqui.*

use crate::components::{PlatformPlayer, TopDownPlayer};
use bevy_ecs::prelude::{Entity, With, World};

/// **Esta entidade é dirigida pelo teclado?**
///
/// ⚠️ `default_controls = false` num mover de vista de cima responde `false` **de propósito**: é
/// isso que o torna um motor PURO, obediente a quem lhe escrever a intenção pelo canal
/// [`crate::PhysicsBridge::set_player_input`].
///
/// ⚠️ Um [`PlatformPlayer`] não tem o interruptor, e a ausência é histórica — a lei dele é anterior
/// ao knob. ⛔ Acrescentá-lo aqui seria inventar um campo que o componente não tem.
#[must_use]
pub fn reads_the_keyboard(world: &World, entity: Entity) -> bool {
    if world.get::<PlatformPlayer>(entity).is_some() {
        return true;
    }
    world
        .get::<TopDownPlayer>(entity)
        .is_some_and(|c| c.default_controls)
}

/// **Visita toda entidade do mundo que lê o teclado.** A mesma lei do irmão acima, do lado de quem
/// tem de varrer em vez de perguntar.
///
/// ⚠️ **Um visitante e não um `Vec`**: isto corre uma vez por quadro no caminho quente, e devolver
/// uma lista poria uma alocação por quadro num sítio onde a casa tem gate a proibi-lo (HR-3).
///
/// ⚠️ **Dois passes e não um `Or<…>`**, e a segunda metade SALTA quem já tem
/// [`PlatformPlayer`]: com os dois movers na mesma entidade o de plataforma ganha (é a lei da
/// ponte), e visitar duas vezes faria a contagem de players do chamador mentir.
pub fn for_each_keyboard_driven(world: &World, mut visit: impl FnMut(Entity)) {
    if let Some(mut q) = world.try_query_filtered::<Entity, With<PlatformPlayer>>() {
        for e in q.iter(world) {
            visit(e);
        }
    }
    if let Some(mut q) = world.try_query::<(Entity, &TopDownPlayer)>() {
        for (e, cfg) in q.iter(world) {
            if cfg.default_controls && world.get::<PlatformPlayer>(e).is_none() {
                visit(e);
            }
        }
    }
}

#[cfg(test)]
#[path = "keyboard_driven_tests.rs"]
mod tests;
