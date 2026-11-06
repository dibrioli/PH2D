//! **Marcador em quadro**: «ao entrar neste passo desta animação, grita este sinal». É o que sincroniza um
//! passo com o pé no chão: a linha de `Signal Actions` que ouve o sinal toca o som.
//!
//! Componente irmão de [`crate::SpriteAnimations`] pelo postcard posicional: arquivo antigo não tem o blob e
//! fica sem marcadores. O passo é a posição dentro da animação, a mesma com intervalo ou com lista de quadros.
//!
//! Num ping-pong o passo do meio é atravessado duas vezes por ciclo, e o marcador grita nas duas.

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use serde::{Deserialize, Serialize};

use crate::sprite_anim_frames::advance_frames_visiting;
use crate::{
    AnimOutcome, AnimationTag, SimComponent, SpriteAnimFrames, SpriteAnimations, SpriteAnimator,
    SpriteGrid, entry_cell, tag_cells,
};

/// Um marcador.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameMarker {
    /// O nome da tag dona; renomear a tag reescreve-o ([`SpriteAnimMarkers::rename_tag`]).
    pub tag: String,
    /// A posição dentro da animação, a partir de `0`.
    pub step: u32,
    /// O sinal. Vazio = calado.
    pub signal: String,
}

/// **Os marcadores de uma sprite.** Um marcador cuja tag sumiu fica inerte, sem erro.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpriteAnimMarkers(pub Vec<FrameMarker>);

impl SimComponent for SpriteAnimMarkers {}

impl SpriteAnimMarkers {
    /// A tag `old` passou a chamar-se `new`.
    pub fn rename_tag(&mut self, old: &str, new: &str) {
        for m in &mut self.0 {
            if m.tag == old {
                m.tag = new.to_owned();
            }
        }
    }

    /// Os marcadores desta tag, na ordem da lista.
    pub fn of_tag<'a>(&'a self, tag: &'a str) -> impl Iterator<Item = &'a FrameMarker> {
        self.0.iter().filter(move |m| m.tag == tag)
    }

    /// O marcador `i` DESTA tag, como posição na lista inteira.
    fn position(&self, tag: &str, i: usize) -> Option<usize> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, m)| m.tag == tag)
            .nth(i)
            .map(|(j, _)| j)
    }

    /// Acrescenta um marcador calado no primeiro dos `steps` quadros ainda sem marcador: dois marcadores no
    /// mesmo quadro não são o que se quer.
    pub fn add(&mut self, tag: &str, steps: u32) {
        let livre = (0..steps.max(1))
            .find(|s| !self.of_tag(tag).any(|m| m.step == *s))
            .unwrap_or(0);
        self.0.push(FrameMarker {
            tag: tag.to_owned(),
            step: livre,
            signal: String::new(),
        });
    }

    /// Muda o passo do marcador `i` desta tag, com teto no último dos `steps`. `None` = marcador fora.
    pub fn set_step(&mut self, tag: &str, i: usize, step: u32, steps: u32) -> Option<()> {
        let j = self.position(tag, i)?;
        self.0[j].step = step.min(steps.saturating_sub(1));
        Some(())
    }

    /// Muda o sinal do marcador `i` desta tag. `None` = marcador fora.
    pub fn set_signal(&mut self, tag: &str, i: usize, signal: String) -> Option<()> {
        let j = self.position(tag, i)?;
        self.0[j].signal = signal;
        Some(())
    }

    /// Tira o marcador `i` desta tag. `None` = marcador fora.
    pub fn remove_at(&mut self, tag: &str, i: usize) -> Option<()> {
        let j = self.position(tag, i)?;
        self.0.remove(j);
        Some(())
    }

    /// Tira todos os marcadores desta tag. `true` = havia algum.
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let antes = self.0.len();
        self.0.retain(|m| m.tag != tag);
        self.0.len() != antes
    }
}

/// Quantos quadros a animação `name` toca na grelha de hoje (`0` = não resolve).
#[must_use]
pub fn anim_steps(world: &World, entity: Entity, name: &str) -> u32 {
    let cells = world
        .get::<SpriteGrid>(entity)
        .map_or(1, |g| g.cells().max(1));
    world
        .get::<SpriteAnimations>(entity)
        .and_then(|l| l.get(name))
        .and_then(|t| {
            let frames = world.get::<SpriteAnimFrames>(entity);
            tag_cells(t, frames.and_then(|f| f.get(name)), cells)
        })
        .map_or(0, |seq| u32::try_from(seq.len()).unwrap_or(u32::MAX))
}

/// **[`crate::advance_frames`] dizendo os sinais dos marcadores atravessados**, na ordem em que aconteceram.
///
/// O primeiro passo de uma animação não é «entrado» por ninguém: ela já está lá quando começa. O começo
/// reconhece-se sem guardar um bit: relógio em zero, nenhum ciclo fechado, tocando e na célula de entrada. Na
/// volta de um ciclo o `repeat_count` já subiu, então um laço que cruza o quadro exatamente na volta conta uma vez.
#[allow(clippy::too_many_arguments)]
pub fn advance_frames_marked(
    animator: &mut SpriteAnimator,
    tag: &AnimationTag,
    list: Option<&[u32]>,
    frame: &mut u32,
    step: &mut u32,
    cells: u32,
    dt_ticks: u64,
    markers: Option<&SpriteAnimMarkers>,
    fired: &mut Vec<String>,
) -> AnimOutcome {
    let mut grita = |passo: u32| {
        if let Some(m) = markers {
            fired.extend(
                m.of_tag(&tag.name)
                    .filter(|m| m.step == passo && !m.signal.is_empty())
                    .map(|m| m.signal.clone()),
            );
        }
    };
    let comeca = animator.playing
        && animator.speed_q16 != 0
        && dt_ticks > 0
        && animator.elapsed_ticks == 0
        && animator.repeat_count == 0;
    if comeca
        && let Some((celula, passo)) = entry_cell(animator, tag, list, cells)
        && celula == *frame
    {
        grita(passo);
    }
    advance_frames_visiting(
        animator, tag, list, frame, step, cells, dt_ticks, &mut grita,
    )
}

#[cfg(test)]
#[path = "sprite_anim_markers_tests.rs"]
mod tests;
