//! **Os factos do cartão «Audio»**: a lista de sons como o cartão a lê e a edita (SFX Lab W1), e a pergunta
//! «alguma linha de `Signal Actions` manda isto tocar?». Tudo é função do mundo; o commit passa pela fila.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    Entity, Name, PickStrategy, ProjectSounds, SignalAction, SignalActions, SignalVerb, SoundSet,
    World,
};
use ph2d_editor_core::{AudioFieldEdit, InspectorSoundRow, InspectorSoundSet};
use ph2d_i18n::tr;
use ph2d_inspector_ordering::queue_set;

const SET: &str = "ph2d::ecs::SoundSet";

/// A lista de sons como o cartão a lê: o que ela decide e o que o Pick ainda oferece.
#[must_use]
pub fn sound_set_info(set: Option<&SoundSet>, sons: Option<&ProjectSounds>) -> InspectorSoundSet {
    let padrao = SoundSet::default();
    let s = set.unwrap_or(&padrao);
    let tem = |n: &str| sons.is_some_and(|l| l.get(n).is_some());
    InspectorSoundSet {
        rows: s
            .sounds
            .iter()
            .map(|e| InspectorSoundRow {
                name: e.name.clone(),
                weight: e.weight,
                enabled: e.enabled,
                missing: !tem(&e.name),
            })
            .collect(),
        decides: s.has_sounds(),
        strategy_tag: s.strategy.index(),
        sequence_tag: PickStrategy::Sequence.index(),
        no_repeat: s.no_repeat,
        pitch_jitter_st: s.pitch_jitter_semitones,
        gain_jitter_db: s.gain_jitter_db,
        mode_labels: PickStrategy::ALL
            .iter()
            .map(|k| tr(k.label_key()).to_string())
            .collect(),
        offer: sons.map_or_else(Vec::new, |l| {
            l.iter()
                .map(|(n, _)| n)
                .filter(|n| !s.sounds.iter().any(|e| e.name == *n))
                .map(str::to_owned)
                .collect()
        }),
        project_sounds: sons.map_or(0, ProjectSounds::len),
    }
}

/// Aplica uma edição da lista de sons; `sound` é o campo Sound da fonte. ⚠️ Toda escrita passa pelo `sanitize`.
pub fn apply_set_edit(
    world: &World,
    entity: Entity,
    sound: &str,
    edit: &AudioFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let mut set = world.get::<SoundSet>(entity).cloned().unwrap_or_default();
    let mudou = match edit {
        AudioFieldEdit::SetAdd(name) => set.add_keeping(sound, name),
        AudioFieldEdit::SetRemove(i) => set.remove_at(usize::from(*i)),
        AudioFieldEdit::SetEnabled(i, on) => set.set_enabled(usize::from(*i), *on),
        AudioFieldEdit::SetWeight(i, v) => set.set_weight(usize::from(*i), *v),
        AudioFieldEdit::SetStrategy(tag) => set.set_strategy_at(usize::from(*tag)),
        AudioFieldEdit::SetNoRepeat(on) => set.set_no_repeat(*on),
        AudioFieldEdit::SetPitchJitter(v) => set.set_pitch_jitter(*v),
        AudioFieldEdit::SetGainJitter(v) => set.set_gain_jitter(*v),
        _ => false,
    };
    if mudou {
        set.sanitize();
        queue_set(queue, registry, entity.to_bits(), SET, &set);
    }
}

/// Com a lista a mandar, um som do projeto largado ou importado vai para ela: no campo ele não tocaria.
/// `true` = foi para a lista (nada mais se grava).
pub fn send_to_list(
    world: &World,
    entity: Entity,
    sound: &str,
    name: &str,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> bool {
    let lista_manda = world
        .get::<SoundSet>(entity)
        .is_some_and(SoundSet::has_sounds);
    let do_projeto = world
        .get_resource::<ProjectSounds>()
        .is_some_and(|l| l.get(name).is_some());
    if lista_manda && do_projeto {
        let add = AudioFieldEdit::SetAdd(name.to_owned());
        apply_set_edit(world, entity, sound, &add, queue, registry);
    }
    lista_manda && do_projeto
}

/// **Alguma linha de `Signal Actions` da cena manda ISTO tocar?**
///
/// ⚠️ **Ela casa por NOME**, como a resolução da tabela: o alvo de uma acção é o `Name`, e uma
/// linha com o alvo **vazio** aponta para o próprio objecto que a carrega.
pub fn reachable_by_signal(world: &mut World, entity: Entity) -> bool {
    let Some(nome) = world.get::<Name>(entity).map(|n| n.0.clone()) else {
        // Sem nome, só uma linha do PRÓPRIO objecto (alvo vazio) o pode alcançar.
        return world
            .get::<SignalActions>(entity)
            .is_some_and(|t| t.0.iter().any(|a| toca(a) && a.target.trim().is_empty()));
    };
    world
        .query::<(Entity, &SignalActions)>()
        .iter(world)
        .any(|(dono, tabela)| {
            tabela.0.iter().any(|a| {
                toca(a)
                    && if a.target.trim().is_empty() {
                        dono == entity
                    } else {
                        a.target == nome
                    }
            })
        })
}

/// Esta linha manda tocar? ⚠️ **Só o `PlaySound`** — `StopSound` cala, e contá-lo faria o painel
/// dizer que um objecto que alguém só sabe CALAR chega a soar.
fn toca(a: &SignalAction) -> bool {
    a.verb == SignalVerb::PlaySound && !a.on.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sem mundo: o que o cartão oferece e quem manda saem só da lista e dos sons do projeto.
    #[test]
    fn the_list_info_offers_what_is_left() {
        let mut l = ProjectSounds::default();
        l.add("a.wav", vec![1]);
        l.add("b.wav", vec![2]);
        let set = SoundSet {
            sounds: vec![ph2d_ecs::SoundEntry::new("a.wav")],
            ..SoundSet::default()
        };
        let i = sound_set_info(Some(&set), Some(&l));
        assert_eq!(i.offer, ["b.wav"]);
        assert!(i.decides);
    }
}
