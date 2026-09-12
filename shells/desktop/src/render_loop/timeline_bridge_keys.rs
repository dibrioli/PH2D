//! **A AUTORIA de uma chave** — irmão de [`super::timeline_bridge`] por teto de LOC (HR-18, 600),
//! e o corte é por RESPONSABILIDADE: lá mora o que corre **a cada quadro** (o relógio, os intents,
//! o apply, os sinais); aqui o que responde *«que chave é que este K insere, e onde?»*.
//!
//! ⚠️ **O ficheiro pai estava EXACTAMENTE no teto** (600/600) quando a wave da pré-visualização
//! lhe acrescentou quatro linhas — e quem toca num ficheiro que está no limite é quem paga o
//! corte. *A cura de um teto estourado é o corte para um irmão, nunca uma isenção.*
//!
//! ⚠️ **Os endereços não mudaram:** o pai re-exporta tudo o que saiu daqui
//! (`pub(crate) use timeline_bridge_keys::*`), então nenhum chamador precisou de mexer. Um corte
//! que obriga vinte ficheiros a mudar de `use` é um corte que ninguém volta a fazer.

use ph2d_ecs::World;
use ph2d_timeline::{PropKind, TimelineState};

/// Sample a bound property's CURRENT value from the scene, for a K-insert
/// keyframe (capture-the-pose). Transform properties read the entity's
/// `Transform`; opacity reads `Sprite.tint[3]`.
pub(crate) fn sample_prop_value(
    world: &World,
    entity_bits: u64,
    prop: PropKind,
) -> Option<ph2d_anim::AnimValue> {
    use ph2d_anim::AnimValue::Float;
    use ph2d_ecs::{Entity, Transform};
    let e = Entity::from_bits(entity_bits);
    let xf = || world.get::<Transform>(e);
    Some(match prop {
        PropKind::TranslationX => Float(xf()?.translation.x),
        PropKind::TranslationY => Float(xf()?.translation.y),
        PropKind::Rotation => Float(xf()?.rotation),
        PropKind::ScaleX => Float(xf()?.scale.x),
        PropKind::ScaleY => Float(xf()?.scale.y),
        // ⭐ **A opacidade tem DOIS substratos**, e a entidade é um ou o outro, nunca os dois: uma
        // sprite guarda-a no `tint[3]`; um caminho vetorial não tem campo nenhum para ela e a
        // recebe pela projecção do quadro (`ph2d_ecs::VecDrivenStyle` → `BoundStyle::alpha`).
        //
        // ⚠️ **Num vetor ainda não conduzido a resposta é `1.0`, não `None`.** Esta função semeia
        // o valor que a tecla K grava: recusar aqui faria a 1.ª chave de um fade nascer sem valor,
        // e devolver `0.0` faria toda track nova começar invisível.
        PropKind::Opacity if world.get::<ph2d_ecs::VecPathRef>(e).is_some() => Float(
            world
                .get::<ph2d_ecs::VecDrivenStyle>(e)
                .and_then(|d| d.alpha)
                .unwrap_or(1.0),
        ),
        PropKind::Opacity => Float(world.get::<ph2d_render::Sprite>(e)?.tint[3]),
        // The morph `t`: unlike the clock below, this IS a scene value, so K captures it the same
        // way it captures a pose — the artist parks the slider where the shape looks right and
        // presses K. (No `VecMorph` on the entity ⇒ nothing to capture, and the `?` refuses.)
        PropKind::Morph => Float(world.get::<ph2d_ecs::VecMorph>(e)?.t),
        // The timeline's own clock has no scene value to sample — the K flow
        // seeds it through `key_value_for` instead.
        PropKind::TimeRemap => return None,
        // Position is not a scalar this function can sample, and that is the
        // channel's shape rather than a gap: capturing one means ADDING AN ANCHOR at
        // the object's current place, which moves the geometry and therefore rewrites
        // the distance every later key holds (ADR-0141 §2). That is an edit to a
        // trajectory, not a read of a number, so it goes through the path's own door
        // — which is what the authoring slice builds. Refusing here keeps this
        // function honest about what it is.
        PropKind::Position => return None,
        // **Os parâmetros de um joint são valores de CENA**, como o `t` do Morph:
        // o artista afina o servo no Inspector até a máquina parecer certa e
        // aperta K. (Sem `PhysicsJoint` na entidade não há o que capturar, e o
        // `?` recusa — que é o mesmo que o Morph faz num sprite comum.)
        PropKind::JointMotorTarget => {
            Float(world.get::<ph2d_physics_ecs::PhysicsJoint>(e)?.motor_target)
        }
        PropKind::JointMotorSpeed => {
            Float(world.get::<ph2d_physics_ecs::PhysicsJoint>(e)?.motor_speed)
        }
        PropKind::JointRestLength => {
            Float(world.get::<ph2d_physics_ecs::PhysicsJoint>(e)?.rest_length)
        }
        PropKind::JointMaxLength => {
            Float(world.get::<ph2d_physics_ecs::PhysicsJoint>(e)?.max_length)
        }
    })
}

/// The value a K-inserted key carries for `prop` at playhead `t_secs`: scene
/// properties sample the live world ([`sample_prop_value`]); **Time Remap** has
/// no scene value — a new key lands ON the clock the entity ALREADY plays
/// ([`ph2d_timeline::remapped_time`]: identity on an empty track, on-curve
/// between keys, slope-1 extrapolation past them, Hold's freeze respected), so
/// K never bends the remap. Seeding through any OTHER transform than the one
/// the apply samples with re-creates the freeze: `tr.sample` flat-clamps past
/// the last key, so K@0 then K@2 laid down a FLAT map = every track of the
/// entity frozen at source 0 (the 2026-07-11 "Time nullifies the animation").
pub(crate) fn key_value_for(
    world: &World,
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    t_secs: f64,
) -> Option<ph2d_anim::AnimValue> {
    if prop == PropKind::TimeRemap {
        let source = ph2d_timeline::remapped_time(&timeline.doc, entity, t_secs);
        return Some(ph2d_anim::AnimValue::Float(source as f32));
    }
    // The live pose is what the animator SEES. Under a clip stack it is a blend,
    // so the number the track must hold to produce it is the blend's inverse —
    // never the pose itself, which would land the object somewhere else the moment
    // the stack re-evaluated. `None` = the active clip cannot express this pose,
    // and K refuses (ADR-0115 R9).
    let ph2d_anim::AnimValue::Float(pose) = sample_prop_value(world, entity, prop)? else {
        return None;
    };
    // `Err` = the active clip cannot express this pose (a lane above overrides it, or a
    // pure/non-linear expression drives it — ADR-0152 W5); K refuses. The refusal REASON is
    // surfaced by the manual-K path (`key_home` -> `KeyRefusal::message`), not this value door.
    let stored =
        ph2d_timeline::key_value_in_active_clip(&timeline.doc, entity, prop, pose, t_secs).ok()?;
    Some(ph2d_anim::AnimValue::Float(stored))
}

/// The track time a K-inserted key lands at, for playhead `t_secs`: scene
/// props key at the entity's own clock ([`ph2d_timeline::remapped_time`]) —
/// tracks are authored in SOURCE time, the same time the apply samples them
/// at, so under a Time Remap the captured pose stays visible at the playhead
/// instead of landing at an invisible time and snapping back. The Time track
/// itself keys at the playhead (the map lives in playhead time). Identity /
/// no remap: `t_secs` unchanged.
pub(crate) fn key_insert_time(
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    t_secs: f64,
) -> Option<ph2d_anim::RationalTime> {
    let t = if prop == PropKind::TimeRemap {
        t_secs
    } else {
        // `key_time`, not `remapped_time`: under a clip stack the strip's map
        // composes on top of the entity's clock, and it can have NO answer (the
        // clip is playing twice at this instant, or not at all). Refuse rather
        // than drop the key at a time the animator never looked at.
        ph2d_timeline::key_time(&timeline.doc, entity, t_secs)?
    };
    Some(ph2d_anim::RationalTime::from_seconds(t))
}

/// **K in the Keys/solo view** — the `(value, time)` a key captures when the scene
/// is showing the active clip soloed at clip time `clip_t`.
///
/// It never REFUSES (returns `Some` whenever the pose can be read): in solo there is
/// no stack to override the clip or to play it twice, so "key it here" always names
/// one place. That is the whole point of editing a clip in isolation.
///
/// The VALUE is the live pose stored DIRECTLY — with the clip soloed, the pose you
/// see IS the clip's value (no blend to invert). The TIME is the entity's own clip
/// clock (`remapped_time`, the active clip's Time Remap), so a Time-Remapped object
/// keys where its retime puts it. Both come through the SAME door the solo apply and
/// the Arrange-side K read, so a soloed pose and the key that captures it agree.
///
/// A Time Remap track keys ON its own curve (the retime value at `clip_t`), exactly
/// as the Arrange-side K does — that half is clock-agnostic.
pub(crate) fn key_authoring_solo(
    world: &World,
    timeline: &TimelineState,
    entity: u64,
    prop: PropKind,
    clip_t: f64,
) -> Option<(ph2d_anim::AnimValue, ph2d_anim::RationalTime)> {
    let doc = &timeline.doc;
    if prop == PropKind::TimeRemap {
        let source = ph2d_timeline::remapped_time(doc, entity, clip_t);
        return Some((
            ph2d_anim::AnimValue::Float(source as f32),
            ph2d_anim::RationalTime::from_seconds(clip_t),
        ));
    }
    let value = sample_prop_value(world, entity, prop)?; // the raw pose = what you see
    Some((value, solo_key_time(timeline, entity, clip_t)))
}

/// **Onde uma key autorada na vista Keys/solo aterrissa**, no relógio do clip.
///
/// A metade do TEMPO do [`key_authoring_solo`], extraída porque ganhou um segundo
/// chamador: o K do modo Path, que não tem valor a amostrar (a âncora é que produz o
/// número) mas precisa EXATAMENTE do mesmo instante. Duas cópias desta composição
/// divergem, e o sintoma é a pose a saltar de volta
/// ([[feedback_derived_coordinate_seed_must_match_sample]]).
///
/// A duração autorada do clip corta o relógio ANTES do remap — a mesma composição que o
/// `apply_active_clip` roda. Além do corte a pose que se vê está congelada em
/// `curve(corte)`, então o K captura NA fronteira (o quadro visível); keyar no tempo cru
/// aterrissaria num instante que o apply nunca amostra.
pub(crate) fn solo_key_time(
    timeline: &TimelineState,
    entity: u64,
    clip_t: f64,
) -> ph2d_anim::RationalTime {
    let doc = &timeline.doc;
    let clip_t = doc.clip_cut(doc.active_index(), clip_t);
    ph2d_anim::RationalTime::from_seconds(ph2d_timeline::remapped_time(doc, entity, clip_t))
}

/// **Onde o K pode ancorar uma trajetória** — `Ok(instante)` na aba Keys, `Err(motivo)`
/// fora dela (Enio, 2026-07-31: *"Path editável apenas em Keys: Clips"*).
///
/// ⚠️ **A pergunta é feita AQUI e não no chamador**, e é o que torna a regra impossível de
/// esquecer: a função lê o `keys_mode` que o próprio `TimelineState` carrega, então o K não
/// tem um booleano a passar (nem a passar errado). É a metade de AUTORIA da mesma lei que o
/// overlay aplica no `active_path` para o desenho e as alças — o mesmo booleano, duas
/// superfícies.
///
/// ⚠️ E o `Err` não é um `None`: um gesto que não faz nada e não diz nada é indistinguível
/// de uma ferramenta quebrada, então o chamador **fala** ([`ph2d_timeline::KeyRefusal`]).
/// Sob uma pilha a pose é a composição das strips e o clip ativo não é o que o animador
/// escolheu; ancorar ali reescreveria a distância de todas as keys de um clip que a aba nem
/// nomeia. Era o `key_insert_time` que atendia este caso — e ele responde a outra pergunta
/// (*onde esta STRIP toca?*), que para geometria de clip não é a pergunta certa.
pub(crate) fn path_key_time(
    timeline: &TimelineState,
    entity: u64,
    clip_t: f64,
) -> Result<ph2d_anim::RationalTime, ph2d_timeline::KeyRefusal> {
    if !timeline.keys_mode {
        return Err(ph2d_timeline::KeyRefusal::PathNeedsKeysTab);
    }
    Ok(solo_key_time(timeline, entity, clip_t))
}

/// The default interpolation for a freshly inserted key (a gentle ease).
pub(crate) fn default_interp() -> ph2d_anim::Interp {
    ph2d_anim::Interp::Eased(ph2d_anim::Easing::new(
        ph2d_anim::EasingFamily::Cubic,
        ph2d_anim::EasingMode::InOut,
    ))
}

/// ⭐⭐⭐ **O NÚMERO QUE CADA ROW MOSTRA** — report do Enio, 2026-09-04: *"o painel não mostra as
/// propriedades animadas (os números não mudam em tempo real com a animação)"*.
///
/// Mora AQUI, ao lado do [`sample_prop_value`], porque é o quarto consumidor da mesma pergunta —
/// *quanto vale esta propriedade no mundo?* — e as outras três já a fazem por esta porta (a semente
/// do `rest`, a tecla K, o auto-key). Uma segunda leitura noutro ficheiro seria a superfície pela
/// qual o readout e a chave que o K grava passam a discordar.
///
/// ⚠️ **É o MUNDO e não a curva**, e a diferença é o defeito irmão deste report: no mesmo dia uma
/// forma filtrada ficou opaca com a curva a dizer `0`. Um readout amostrado da curva teria escrito
/// `0,00` e concordado com o defeito.
pub(crate) fn publish_track_values(view: &mut ph2d_timeline::TimelineViewSnapshot, world: &World) {
    let ph2d_timeline::TimelineViewSnapshot { tracks, values, .. } = view;
    values.publish(tracks, |entity, prop| {
        match sample_prop_value(world, entity, prop) {
            Some(ph2d_anim::AnimValue::Float(v)) => Some(v),
            // Um canal sem escalar de mundo (`TimeRemap`, `Position`) ou um objecto que morreu:
            // a row fica SEM número, nunca com um zero (ver `TrackValues::get`).
            _ => None,
        }
    });
}
