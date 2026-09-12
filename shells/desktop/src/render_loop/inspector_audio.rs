//! ⭐⭐⭐ **A secção AUDIO** (TOP-20 #4, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`super::inspector_timer`], pela mesma razão dele.
//!
//! # ⚠️ O snapshot só existe para quem TEM a fonte, as orelhas, ou as duas
//!
//! É o ADR-0166: *o Inspector mostra o que o objecto TEM, e um componente anexa-se pela paleta*.
//!
//! # ⭐⭐ TRÊS coisas são DERIVADAS aqui, e nenhuma delas se podia derivar no painel
//!
//! - **`file_missing`** — pergunta ao DISCO. É a diferença entre *«ainda não escolhi»* e *«alguém
//!   mudou o ficheiro de sítio»*, e sem ela as duas leem-se como o mesmo silêncio.
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

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    AUDIO_MAX_DISTANCE_M, AUDIO_MAX_POLYPHONY, AudioBus, AudioListener2D, AudioSource2D, Entity,
    Name, SignalActions, SignalVerb, SimWorld, World,
};
use ph2d_editor::{AudioFieldEdit, InspectorAudioInfo, InspectorAudioSource, Toast};

use super::inspector_ordering::queue_set;

const SOURCE: &str = "ph2d::ecs::AudioSource2D";

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
pub(super) fn build_audio_info(
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
    let source = src.map(|s| InspectorAudioSource {
        // ⚠️ **O disco responde uma vez por quadro e só para quem tem caminho** — um `exists()`
        // sobre uma string vazia é uma syscall para não dizer nada.
        file_missing: !s.sound.trim().is_empty() && !std::path::Path::new(&s.sound).exists(),
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
        bus_labels: AudioBus::ALL.iter().map(|b| b.label().into()).collect(),
        selected_count,
    })
}

/// Aplica uma [`AudioFieldEdit`] que toca no DOCUMENTO. Devolve um aviso quando recusa.
///
/// ⚠️ **`Preview`, `StopPreview` e `Browse` devolvem `None` sem escrever nada** — eles não são
/// campos, e quem os serve é o dreno. Ver o doc do módulo.
pub(super) fn apply_audio_edit(
    sim: &SimWorld,
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
        AudioFieldEdit::Sound(path) => src.sound = path.trim().to_string(),
        AudioFieldEdit::VolumeDb(v) => src.volume_db = v.clamp(-80.0, 24.0), // CLAMP-OK: a faixa do campo
        AudioFieldEdit::Pitch(v) => src.pitch = v.clamp(0.05, 8.0), // CLAMP-OK: a faixa do campo
        AudioFieldEdit::Looping(on) => src.looping = *on,
        AudioFieldEdit::Autoplay(on) => src.autoplay = *on,
        // ⚠️ **A saturação é do MOTOR** (`AUDIO_MAX_DISTANCE_M`), e mora aqui porque é aqui que o
        // valor entra na cena. Um número acima do teto entra saturado em vez de ficar por explicar.
        AudioFieldEdit::MaxDistance(v) => {
            src.max_distance = v.clamp(0.0, AUDIO_MAX_DISTANCE_M); // CLAMP-OK: o teto é o do motor
        }
        AudioFieldEdit::Attenuation(v) => src.attenuation = v.clamp(0.0, 8.0), // CLAMP-OK: a faixa do campo
        AudioFieldEdit::Radius(v) => {
            src.non_spatialized_radius = v.clamp(0.0, AUDIO_MAX_DISTANCE_M); // CLAMP-OK: o teto é o do motor
        }
        AudioFieldEdit::Panning(v) => src.panning_strength = v.clamp(0.0, 1.0), // CLAMP-OK: fracção
        AudioFieldEdit::Polyphony(n) => {
            // ⚠️ **Recusa com VOZ acima do teto**, e não em silêncio: o teto é do pool do mixer, e
            // um número que entra saturado sem dizer nada lê-se como o campo estar partido.
            if *n > AUDIO_MAX_POLYPHONY {
                return Some(Toast::warning(format!(
                    "One source can hold at most {AUDIO_MAX_POLYPHONY} voices \u{2014} the mixer's \
                     pool is shared with the whole scene."
                )));
            }
            src.max_polyphony = (*n).max(1);
        }
        AudioFieldEdit::Bus(tag) => src.bus_tag = AudioBus::from_tag(*tag).tag(),
    }
    queue_set(queue, registry, entity_bits, SOURCE, &src);
    None
}
