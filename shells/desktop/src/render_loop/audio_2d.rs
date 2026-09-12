//! ⭐⭐⭐ **A PONTE DO SOM DE CENA** — onde um objecto do mundo deixa de ser mudo (TOP-20 #4).
//!
//! # A fronteira, e porque ela é exactamente esta
//!
//! - A **lei** vive no [`ph2d_ecs::audio_2d`]: quem são as orelhas, quanto se ouve, de que lado.
//!   Ela é pura e não conhece o dispositivo.
//! - O **livro das vozes** vive no [`ph2d_app_audio::scene`]: o que está a soar agora, chaveado por
//!   `StableId`, ao lado do `cpal`.
//! - Este ficheiro é a **costura**: lê o mundo, chama a lei, e manda o livro tocar.
//!
//! É a mesma repartição do `timer_tick` (a lei devolve factos, a ponte publica-os), com uma junta a
//! mais porque uma voz é um recurso do dispositivo e um sinal não é.
//!
//! # ⚠️ «Começar a tocar» é uma ARESTA, e a aresta é o NASCIMENTO
//!
//! É a lei que o `Timer` pagou com um report do dono (*«nada do smoke funciona»*): a condição do
//! `autoplay` **não** pode ser *«não está a tocar»*, porque um som de uma vez só que acaba satisfaz
//! isso e renasceria a cada quadro. A condição é *«este objecto não existia no livro»*, e quem a
//! responde é o [`ph2d_app_audio::scene::SceneAudio::take_birth`].
//!
//! # ⚠️ O que este passe NÃO faz
//!
//! ⛔ **Ele não escreve na cena.** Nenhum componente registado é tocado aqui, logo **não há passo de
//! undo nenhum a declarar** e o `preview_drive` não entra na assinatura — ao contrário do
//! `signal_actions::apply`, que escreve `Visibility`. *Som é saída; ele lê o documento e não o
//! muda.*

use std::collections::BTreeSet;

use ph2d_ecs::{AudioSource2D, Entity, SimWorld, StableId};

use ph2d_audio::AudioEngine;

use ph2d_app_audio::AudioSystem;
use ph2d_app_audio::scene::SceneAudio;

/// O que o quadro fez com o som — o que o smoke imprime.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct AudioSceneReport {
    /// Quantas fontes nasceram com `autoplay` e arrancaram neste quadro.
    pub(crate) started: usize,
    /// Quantas vozes de cena estão vivas depois deste quadro.
    pub(crate) live: usize,
    /// Quantas fontes a cena tem.
    pub(crate) sources: usize,
    /// Quantos ouvintes a cena tem. ⚠️ **`0` é informação, não erro** — o som toca sem posição.
    pub(crate) listeners: usize,
}

/// **Onde estão as orelhas**, em coordenadas de mundo — ou `None` se a cena não tiver ouvinte.
fn ears(sim: &mut SimWorld) -> Option<[f32; 2]> {
    let e = ph2d_ecs::listener_of(sim.world_mut())?;
    let t = ph2d_ecs::world_transform(sim.world(), e)?;
    Some([t.translation.x, t.translation.y])
}

/// A posição de mundo de uma entidade.
fn position_of(sim: &SimWorld, e: Entity) -> Option<[f32; 2]> {
    let t = ph2d_ecs::world_transform(sim.world(), e)?;
    Some([t.translation.x, t.translation.y])
}

/// ⭐⭐ **O passe de um quadro** — nasce, segue e retira.
///
/// ⚠️ **A ordem das quatro metades é load-bearing:**
/// 1. **retirar** o que acabou (senão a polifonia conta vozes que já não soam);
/// 2. **esquecer** quem saiu da cena — e calá-lo (um `Ctrl+Z` sobre um objecto que soava deixaria
///    um som órfão que ninguém pode parar);
/// 3. **nascer** o que é novo com `autoplay`;
/// 4. **seguir** — reescrever ganho e pan de tudo o que está vivo.
///
/// ⚠️ **O passo 4 corre para TODAS as fontes, e não só para as que nasceram agora**: é ele que faz
/// o som andar com o objecto. Sem ele o pan ficaria congelado no instante do disparo, que é o
/// defeito que a espacialização existe para não ter.
pub(crate) fn update(sim: &mut SimWorld, audio: Option<&mut AudioSystem>) -> AudioSceneReport {
    let Some(audio) = audio else {
        return AudioSceneReport::default();
    };
    let (livro, motor) = audio.scene_parts();
    update_with(sim, livro, motor)
}

/// ⭐⭐ **A PORTA que um gate alcança** — o mesmo passe, com o livro e o motor em vez do
/// dispositivo.
///
/// ⚠️ **Ela existe porque o `AudioSystem` precisa de uma placa de som e um gate não tem uma.** O
/// `AudioEngine::new` devolve o par controlo/renderer sem tocar no `cpal`, então tudo o que este
/// passe DECIDE — a aresta do nascimento, a polifonia, quem é calado ao sair da cena — é medível
/// sem dispositivo nenhum. ⛔ Sem esta porta, a única forma de o testar seria reimplementá-lo no
/// teste, que é a segunda resposta à mesma pergunta.
pub(crate) fn update_with(
    sim: &mut SimWorld,
    livro: &mut SceneAudio,
    motor: &mut AudioEngine,
) -> AudioSceneReport {
    let listeners = ph2d_ecs::listener_count(sim.world_mut());
    let ouvinte = ears(sim);

    // As fontes, na ordem da IDENTIDADE — nunca a da query (a ordem dos arquétipos muda quando um
    // componente é inserido, e um som que troca de dono não se reproduz).
    let mut fontes: Vec<(u64, Entity, AudioSource2D)> = sim
        .world_mut()
        .query::<(Entity, &AudioSource2D, &StableId)>()
        .iter(sim.world())
        .map(|(e, s, id)| (id.0, e, s.clone()))
        .collect();
    fontes.sort_unstable_by_key(|(id, _, _)| *id);
    let mut started = 0;

    livro.retire();
    let vivos: BTreeSet<u64> = fontes.iter().map(|(id, _, _)| *id).collect();
    livro.forget_absent(motor, &vivos);

    for (id, entity, cfg) in &fontes {
        let Some(pos) = position_of(sim, *entity) else {
            continue;
        };
        let sp = ph2d_ecs::spatialize(pos, ouvinte, cfg);
        // ⚠️ **A aresta primeiro**: `take_birth` marca-o como nascido mesmo quando o `autoplay`
        // está desligado — senão uma fonte muda ficaria «por nascer» para sempre e ligar o
        // `autoplay` a meio da sessão fá-la-ia arrancar sem ninguém pedir.
        let nasceu = livro.take_birth(*id);
        if nasceu && cfg.autoplay && livro.play(motor, *id, cfg, sp) {
            started += 1;
        }
        livro.update(motor, *id, sp);
    }
    AudioSceneReport {
        started,
        live: livro.live_voices(),
        sources: fontes.len(),
        listeners,
    }
}

/// **TOCA** o som de um alvo — o verbo `PlaySound` da tabela de acções.
///
/// ⚠️ **A posição é lida AGORA**, e não a do último quadro: um som disparado por um sinal nasce de
/// onde o objecto está no instante do disparo.
pub(crate) fn play_target(
    sim: &mut SimWorld,
    audio: Option<&mut AudioSystem>,
    target: Entity,
) -> bool {
    let Some(audio) = audio else {
        return false;
    };
    let (livro, motor) = audio.scene_parts();
    play_target_with(sim, livro, motor, target)
}

/// A porta do [`play_target`] que um gate alcança — ver o doc do [`update_with`].
pub(crate) fn play_target_with(
    sim: &mut SimWorld,
    livro: &mut SceneAudio,
    motor: &mut AudioEngine,
    target: Entity,
) -> bool {
    let Some(cfg) = sim.world().get::<AudioSource2D>(target).cloned() else {
        return false;
    };
    let Some(id) = sim.world().get::<StableId>(target).map(|s| s.0) else {
        return false;
    };
    let Some(pos) = position_of(sim, target) else {
        return false;
    };
    let ouvinte = ears(sim);
    let sp = ph2d_ecs::spatialize(pos, ouvinte, &cfg);
    livro.play(motor, id, &cfg, sp)
}

/// **CALA** o som de um alvo — o verbo `StopSound`.
pub(crate) fn stop_target(
    sim: &mut SimWorld,
    audio: Option<&mut AudioSystem>,
    target: Entity,
) -> bool {
    let Some(audio) = audio else {
        return false;
    };
    let (livro, motor) = audio.scene_parts();
    stop_target_with(sim, livro, motor, target)
}

/// A porta do [`stop_target`] que um gate alcança — ver o doc do [`update_with`].
pub(crate) fn stop_target_with(
    sim: &SimWorld,
    livro: &mut SceneAudio,
    motor: &AudioEngine,
    target: Entity,
) -> bool {
    let Some(id) = sim.world().get::<StableId>(target).map(|s| s.0) else {
        return false;
    };
    livro.stop(motor, id)
}

#[cfg(test)]
#[path = "audio_2d_tests.rs"]
mod tests;
