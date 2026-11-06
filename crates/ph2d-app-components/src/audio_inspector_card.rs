//! ⭐⭐⭐ **A secção AUDIO** (TOP-20 #4, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`crate::timer_inspector`], pela mesma razão dele; saiu da shell na GEF 21 E2.
//!
//! # ⚠️ O snapshot só existe para quem TEM a fonte, as orelhas, ou as duas
//!
//! É o ADR-0166: *o Inspector mostra o que o objecto TEM, e um componente anexa-se pela paleta*.
//!
//! # ⭐⭐ TRÊS coisas são DERIVADAS aqui, e nenhuma delas se podia derivar no painel
//!
//! - **`file_missing`** — o nome não é um som do PROJETO (GEF 21 E2: o caminho antigo que o load
//!   não achou). É a diferença entre *«ainda não escolhi»* e *«o som falta»*.
//! - **`reachable_by_signal`** — varre as tabelas de acção da CENA à procura de um `Play Sound` que
//!   aponte para este objecto. ⚠️ É a metade que a lei pura declara não poder responder: ela sabe
//!   que a fonte não arranca sozinha, e não sabe se alguém a manda arrancar. *Sem ela o painel
//!   gritaria «nada toca isto» sobre uma cena perfeitamente correcta.*
//! - **`is_active_listener`** — com vários ouvintes ganha o de menor identidade, e o painel tem de
//!   dizer qual, senão o artista afina um que ninguém usa.
//!
//! # ⚠️ O PREVIEW e o BROWSE não passam por aqui
//!
//! Eles não escrevem no documento: um toca, o outro abre um diálogo. Quem os serve é o dreno, que
//! tem o dispositivo e a janela. *O que esta porta faz é o commit de um CAMPO.*
//!
//! # ⭐ O campo Sound recebe um NOME, um caminho ou uma queda (GEF 21 E2)
//!
//! Um nome dos sons do projeto fica; um ficheiro que existe é IMPORTADO e o campo fica com o nome
//! dele ([`crate::sound_project`]); o resto fica escrito e o cartão diz que falta.

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    AUDIO_MAX_POLYPHONY, AudioBus, AudioListener2D, AudioSource2D, Entity, Name, PickStrategy,
    ProjectSounds, SignalActions, SignalVerb, SimWorld, SoundEntry, SoundSet, World,
};
use ph2d_editor_core::{
    AudioFieldEdit, InspectorAudioInfo, InspectorAudioSource, InspectorSoundRow, InspectorSoundSet,
    Toast,
};
use ph2d_i18n::{tr, tr_with};

use ph2d_inspector_ordering::queue_set;

const SOURCE: &str = "ph2d::ecs::AudioSource2D";
const SET: &str = "ph2d::ecs::SoundSet";

/// A lista de sons como o cartão a lê: o que ela decide e o que o Pick ainda oferece.
fn sound_set_info(set: Option<&SoundSet>, sons: Option<&ProjectSounds>) -> InspectorSoundSet {
    let padrao = SoundSet::default();
    let s = set.unwrap_or(&padrao);
    let tem = |n: &str| sons.is_some_and(|l| l.get(n).is_some());
    let posicao = |k: PickStrategy| {
        PickStrategy::ALL
            .iter()
            .position(|o| *o == k)
            .and_then(|i| u8::try_from(i).ok())
            .unwrap_or(0)
    };
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
        strategy_tag: posicao(s.strategy),
        sequence_tag: posicao(PickStrategy::Sequence),
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

/// Aplica uma edição da lista de sons. ⚠️ Toda escrita passa pelo `sanitize`.
fn apply_set_edit(
    world: &World,
    entity: Entity,
    src: &AudioSource2D,
    edit: &AudioFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let mut set = world.get::<SoundSet>(entity).cloned().unwrap_or_default();
    let mudou = match edit {
        AudioFieldEdit::SetAdd(name) => add_to_set(&mut set, src, name),
        AudioFieldEdit::SetRemove(i) => {
            let dentro = usize::from(*i) < set.sounds.len();
            if dentro {
                set.sounds.remove(usize::from(*i));
            }
            dentro
        }
        AudioFieldEdit::SetEnabled(i, on) => set
            .sounds
            .get_mut(usize::from(*i))
            .map(|e| e.enabled = *on)
            .is_some(),
        AudioFieldEdit::SetWeight(i, v) => set
            .sounds
            .get_mut(usize::from(*i))
            .map(|e| e.weight = *v)
            .is_some(),
        AudioFieldEdit::SetStrategy(tag) => match PickStrategy::ALL.get(usize::from(*tag)) {
            Some(k) => {
                set.strategy = *k;
                true
            }
            None => false,
        },
        AudioFieldEdit::SetNoRepeat(on) => {
            set.no_repeat = *on;
            true
        }
        AudioFieldEdit::SetPitchJitter(v) => {
            set.pitch_jitter_semitones = *v;
            true
        }
        AudioFieldEdit::SetGainJitter(v) => {
            set.gain_jitter_db = *v;
            true
        }
        _ => false,
    };
    if mudou {
        set.sanitize();
        queue_set(queue, registry, entity.to_bits(), SET, &set);
    }
}

/// Põe `name` na lista. Numa lista vazia o `sound` de sempre entra primeiro, para o objeto não perder o som
/// que já tinha. `false` = o nome é vazio ou já está na lista.
fn add_to_set(set: &mut SoundSet, src: &AudioSource2D, name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() || set.sounds.iter().any(|e| e.name == name) {
        return false;
    }
    let antigo = src.sound.trim();
    if set.sounds.is_empty() && !antigo.is_empty() && antigo != name {
        set.sounds.push(SoundEntry::new(antigo));
    }
    set.sounds.push(SoundEntry::new(name));
    true
}

/// **Alguma linha de `Signal Actions` da cena manda ISTO tocar?**
///
/// ⚠️ **Ela casa por NOME**, como a resolução da tabela: o alvo de uma acção é o `Name`, e uma
/// linha com o alvo **vazio** aponta para o próprio objecto que a carrega.
fn reachable_by_signal(world: &mut World, entity: Entity) -> bool {
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
fn toca(a: &ph2d_ecs::SignalAction) -> bool {
    a.verb == SignalVerb::PlaySound && !a.on.trim().is_empty()
}

/// O snapshot da secção, ou `None` quando o objecto não tem nada de áudio.
pub fn build_audio_info(
    world: &mut World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorAudioInfo> {
    let entity = Entity::from_bits(entity_bits);
    let src = world.get::<AudioSource2D>(entity).cloned();
    let is_listener = world.get::<AudioListener2D>(entity).is_some();
    if src.is_none() && !is_listener {
        return None;
    }
    let listener_count = ph2d_ecs::listener_count(world);
    let is_active_listener = is_listener && ph2d_ecs::listener_of(world) == Some(entity);
    let alcancavel = src.is_some() && reachable_by_signal(world, entity);
    let set = sound_set_info(
        world.get::<SoundSet>(entity),
        world.get_resource::<ProjectSounds>(),
    );
    let source = src.map(|s| InspectorAudioSource {
        // O que toca é a lista quando ela manda; senão, o campo.
        file_missing: if set.decides {
            set.missing_names().next().is_some()
        } else {
            !s.sound.trim().is_empty()
                && world
                    .get_resource::<ProjectSounds>()
                    .is_none_or(|l| l.get(&s.sound).is_none())
        },
        set,
        sound: s.sound,
        volume_db: s.volume_db,
        pitch: s.pitch,
        looping: s.looping,
        autoplay: s.autoplay,
        max_distance: s.max_distance,
        attenuation: s.attenuation,
        non_spatialized_radius: s.non_spatialized_radius,
        panning_strength: s.panning_strength,
        max_polyphony: s.max_polyphony,
        bus_tag: s.bus_tag,
        reachable_by_signal: alcancavel,
    });
    Some(InspectorAudioInfo {
        entity_bits,
        source,
        is_listener,
        listener_count,
        is_active_listener,
        // ⚠️ **Os rótulos saem de `AudioBus::ALL`, que é a fonte** — copiá-los para o painel
        // envelheceria no primeiro barramento novo, e o artista leria o nome errado.
        //
        // ⭐ **E desde 2026-09-19 é a CHAVE que sai do motor, não a palavra** — este é o único
        // sítio onde os quatro barramentos chegam a um pixel, e ele é da shell: o painel não
        // conhece o enum. *Um rótulo que atravessa a fronteira de crate escapa às 30 réguas do
        // HR-15 dos dois lados* (gate `a_fronteira_dos_motores`).
        bus_labels: AudioBus::ALL
            .iter()
            .map(|b| tr(b.label_key()).to_string())
            .collect(),
        selected_count,
    })
}

/// Aplica uma [`AudioFieldEdit`] que toca no DOCUMENTO. Devolve um aviso quando recusa.
///
/// ⚠️ **`Preview`, `StopPreview` e `Browse` devolvem `None` sem escrever nada** — eles não são
/// campos, e quem os serve é o dreno (o Browse volta como `Sound(caminho)`). Ver o doc do módulo.
pub fn apply_audio_edit(
    sim: &mut SimWorld,
    entity_bits: u64,
    edit: &AudioFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Toast> {
    let entity = Entity::from_bits(entity_bits);
    let mut src = sim.world().get::<AudioSource2D>(entity).cloned()?;
    match edit {
        AudioFieldEdit::Preview | AudioFieldEdit::StopPreview | AudioFieldEdit::Browse => {
            return None;
        }
        AudioFieldEdit::SetAdd(_)
        | AudioFieldEdit::SetRemove(_)
        | AudioFieldEdit::SetEnabled(..)
        | AudioFieldEdit::SetWeight(..)
        | AudioFieldEdit::SetStrategy(_)
        | AudioFieldEdit::SetNoRepeat(_)
        | AudioFieldEdit::SetPitchJitter(_)
        | AudioFieldEdit::SetGainJitter(_) => {
            apply_set_edit(sim.world(), entity, &src, edit, queue, registry);
            return None;
        }
        AudioFieldEdit::Sound(text) => match sound_name(sim.world_mut(), text) {
            Ok(name) => {
                // Com a lista a mandar, um som largado ou importado vai para ela: no campo ele não tocaria.
                let world = sim.world();
                let lista_manda = world
                    .get::<SoundSet>(entity)
                    .is_some_and(SoundSet::has_sounds);
                let do_projeto = world
                    .get_resource::<ProjectSounds>()
                    .is_some_and(|l| l.get(&name).is_some());
                if lista_manda && do_projeto {
                    let add = AudioFieldEdit::SetAdd(name);
                    apply_set_edit(world, entity, &src, &add, queue, registry);
                    return None;
                }
                src.sound = name;
            }
            Err(t) => return Some(t),
        },
        AudioFieldEdit::VolumeDb(v) => src.volume_db = *v,
        AudioFieldEdit::Pitch(v) => src.pitch = *v,
        AudioFieldEdit::Looping(on) => src.looping = *on,
        AudioFieldEdit::Autoplay(on) => src.autoplay = *on,
        AudioFieldEdit::MaxDistance(v) => src.max_distance = *v,
        AudioFieldEdit::Attenuation(v) => src.attenuation = *v,
        AudioFieldEdit::Radius(v) => src.non_spatialized_radius = *v,
        AudioFieldEdit::Panning(v) => src.panning_strength = *v,
        AudioFieldEdit::Polyphony(n) => {
            // ⚠️ **Recusa com VOZ acima do teto**, e não em silêncio: o teto é do pool do mixer, e
            // um número que entra saturado sem dizer nada lê-se como o campo estar partido.
            if *n > AUDIO_MAX_POLYPHONY {
                return Some(Toast::warning(tr_with(
                    "shell.inspector_audio.one_source_can_hold_at",
                    &[("AUDIO_MAX_POLYPHONY", &AUDIO_MAX_POLYPHONY)],
                )));
            }
            src.max_polyphony = *n;
        }
        AudioFieldEdit::Bus(tag) => src.bus_tag = *tag,
    }
    // ⭐ A lei dos números é a MESMA do script (`ph2d.set`, doc CodeEditor 08 §3.5).
    super::sanitize_audio_source(&mut src);
    queue_set(queue, registry, entity_bits, SOURCE, &src);
    None
}

/// O que o campo Sound fica a nomear para o `text` escrito, largado ou escolhido — ver o doc do
/// módulo. `Err` = o ficheiro existe e não é som (o aviso diz-lo).
fn sound_name(world: &mut World, text: &str) -> Result<String, Toast> {
    let t = text.trim();
    if t.is_empty()
        || world
            .get_resource::<ProjectSounds>()
            .is_some_and(|l| l.get(t).is_some())
    {
        return Ok(t.to_owned());
    }
    let cwd = std::env::current_dir().ok();
    let Some((file, _)) = ph2d_script::project_scripts::resolve(t, None, cwd.as_deref()) else {
        return Ok(t.to_owned());
    };
    crate::sound_project::import(world, &file).map_err(|e| {
        Toast::warning(tr_with(
            "app.components.audio_inspector.not_a_sound",
            &[("file", &t), ("why", &e)],
        ))
    })
}

#[cfg(test)]
#[path = "audio_inspector_card_tests.rs"]
mod tests;
