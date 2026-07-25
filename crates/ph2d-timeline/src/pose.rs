//! **A pose de uma entidade num instante — a PORTA ÚNICA** que dá pose ao apply e ao
//! onion (ADR-0142).
//!
//! [`apply_active_clip`](crate::apply) ESCREVE a pose no mundo; [`pose_at`] a devolve
//! sem tocar o mundo (o onion precisa da pose de `t±k` sem mover o objeto vivo). As duas
//! passam pelo MESMO [`set_transform_field`] — a mesma aritmética, um destino cada — para
//! que não exista uma 2ª derivação da pose a divergir (a doença
//! [[feedback_derived_coordinate_seed_must_match_sample]] que este módulo pagou 3×). Um
//! gate de equivalência prova `pose_at == { apply; read Transform }` campo a campo.

use ph2d_anim::{AnimValue, AttributeEvaluator};
use ph2d_ecs::{Entity, Transform, World};

use crate::binding::TargetBinding;
use crate::prop::PropKind;
use crate::sprite::SpriteProp;
use crate::{AutoOrient, TimelineDoc, remapped_time};

/// Sobrepõe um valor resolvido `(binding, valor)` num `Transform` — a **porta única** por
/// onde o apply (mutando o mundo) e o [`pose_at`] (num Transform de rascunho) escrevem
/// uma pose. Só os canais que são pose (Position + os cinco de sprite-transform); Morph e
/// Opacity vivem noutros componentes e não movem um `Transform`.
pub(crate) fn set_transform_field(
    xf: &mut Transform,
    b: &TargetBinding,
    v: AnimValue,
    orient: bool,
) {
    // O único canal cujo valor não é uma coordenada (ADR-0141): é uma DISTÂNCIA ao longo
    // do caminho, e virá um ponto quando se perguntar à trajetória onde ela está.
    if b.prop == PropKind::Position {
        let (Some(path), AnimValue::Float(s)) = (b.path.as_ref(), v) else {
            return;
        };
        let Some(sample) = path.at(f64::from(s)) else {
            return;
        };
        xf.translation.x = sample.point[0];
        xf.translation.y = sample.point[1];
        // **Auto-orient** (ADR-0141 §6): o objeto encara a tangente do caminho.
        //
        // ⚠️ **Numa CÚSPIDE não se escreve nada, e é assim que o ângulo se SEGURA.**
        // `tangent_at` devolve `None` onde a velocidade da curva é zero — inventar uma
        // direção ali produz o pico solto. Não escrever deixa a `rotation` como estava,
        // que É "segurar o último ângulo válido", sem estado nenhum a guardar/invalidar.
        // E este canal não tem o bug do AE (*"flips when stopping motion"*) por
        // construção: o ângulo vem da GEOMETRIA da curva, não do vetor velocidade.
        if orient && let Some(t) = sample.tangent {
            xf.rotation = libm::atan2f(t[1], t[0]);
        }
        return;
    }
    if let Some(sp) = b.prop.as_sprite_transform() {
        let AnimValue::Float(f) = v else { return };
        match sp {
            SpriteProp::TranslationX => xf.translation.x = f,
            SpriteProp::TranslationY => xf.translation.y = f,
            SpriteProp::Rotation => xf.rotation = f,
            SpriteProp::ScaleX => xf.scale.x = f,
            SpriteProp::ScaleY => xf.scale.y = f,
        }
    }
}

/// **A pose que `entity` teria no instante `clip_t` do clip, SEM tocar o mundo.**
///
/// Parte do `Transform` VIVO (os campos que nenhuma track dirige ficam como estão — o que
/// o apply também faz) e sobrepõe cada binding da entidade, amostrado no relógio DELA.
/// Espelha [`apply_active_clip`](crate::apply) passo a passo (o corte de duração do clip,
/// o `remapped_time`, o skip de track vazia, a decisão de auto-orient) exceto por não
/// escrever — é o que o gate de equivalência pina.
///
/// `None` quando a entidade não existe ou não tem `Transform`. Zero-alloc.
#[must_use]
pub fn pose_at(world: &World, doc: &TimelineDoc, entity: u64, clip_t: f64) -> Option<Transform> {
    let e = Entity::try_from_bits(entity)?;
    let mut xf = *world.get::<Transform>(e)?;
    // A duração autorada do clip corta o próprio relógio (Enio, 2026-07-23) — o mesmo
    // corte que o apply faz, senão um fantasma além do fim leria um tempo que o objeto
    // vivo nunca alcança.
    let clip_t = doc.clip_cut(doc.active_index(), clip_t);
    // O relógio da entidade (Time Remap) é o mesmo para todos os bindings dela.
    let t_entity = remapped_time(doc, entity, clip_t);
    for b in doc.bindings() {
        if b.entity != entity || b.missing || b.prop == PropKind::TimeRemap {
            continue;
        }
        // Track vazia não força pose default (um binding recém-criado).
        let Some(v) = doc
            .active_clip()
            .track(b.target)
            .and_then(|tr| (!tr.is_empty()).then(|| tr.sample(t_entity)))
        else {
            continue;
        };
        let orient =
            b.prop == PropKind::Position && doc.auto_orient(entity) == AutoOrient::Active;
        set_transform_field(&mut xf, b, v, orient);
    }
    Some(xf)
}

/// **Toda entidade que a timeline dirige** (o alvo de um binding vivo com track não-vazia)
/// — o conjunto que o onion pode ghostar. Ordenado e sem repetição: um objeto com X e Y
/// keyados aparece UMA vez.
#[must_use]
pub fn animated_entities(doc: &TimelineDoc) -> Vec<u64> {
    let mut out: Vec<u64> = doc
        .bindings()
        .iter()
        .filter(|b| {
            !b.missing
                && b.prop != PropKind::TimeRemap
                && doc
                    .active_clip()
                    .track(b.target)
                    .is_some_and(|tr| !tr.is_empty())
        })
        .map(|b| b.entity)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

#[cfg(test)]
#[path = "pose_tests.rs"]
mod pose_tests;
