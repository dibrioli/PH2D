//! **A pose de uma entidade num instante — a PORTA ÚNICA** que dá pose ao apply e ao
//! onion (ADR-0142).
//!
//! [`apply_active_clip`](crate::apply) ESCREVE a pose no mundo; [`pose_at`] a devolve
//! sem tocar o mundo (o onion precisa da pose de `t±k` sem mover o objeto vivo). As duas
//! passam pelo MESMO [`set_transform_field`] — a mesma aritmética, um destino cada — para
//! que não exista uma 2ª derivação da pose a divergir (a doença
//! [[feedback_derived_coordinate_seed_must_match_sample]] que este módulo pagou 3×). Um
//! gate de equivalência prova `pose_at == { apply; read Transform }` campo a campo.

use ph2d_anim::AnimValue;
use ph2d_ecs::{Entity, Transform, World};

use crate::binding::TargetBinding;
use crate::prop::PropKind;
use crate::sprite::SpriteProp;
use crate::{AutoOrient, TimelineDoc, remapped_time};

/// Sobrepõe um valor resolvido `(binding, valor)` num `Transform` — a **porta única** por
/// onde o apply (mutando o mundo) e o [`pose_at`] (num Transform de rascunho) escrevem
/// uma pose. Só os canais que são pose (Position + os cinco de sprite-transform); Morph e
/// Opacity vivem noutros componentes e não movem um `Transform`.
/// **Escreve uma POSE de trajetória já resolvida como PONTO** — o destino do
/// [`crate::stack_eval::sample_stack_point`].
///
/// ⚠️ **O auto-orient PROJETA em vez de pedir um segundo blend.** A tangente é uma
/// grandeza sobre UMA curva, e depois de compor pontos não há mais distância; projetar o
/// ponto de volta na trajetória que dirige devolve exatamente a distância que ele ocupa —
/// exato fora do cruzamento (o ponto ESTÁ na curva) e a leitura honesta dentro dele. A
/// alternativa seria rodar o blend uma terceira vez em distância, o que faria a pose e o
/// ângulo saírem de duas composições diferentes: a doença
/// [[feedback_derived_coordinate_seed_must_match_sample]], mais uma vez.
pub(crate) fn set_transform_point(
    xf: &mut Transform,
    path: Option<&crate::MotionPath>,
    p: [f32; 2],
    orient: bool,
) {
    xf.translation.x = p[0];
    xf.translation.y = p[1];
    if orient
        && let Some(path) = path
        && let Some(d) = path.project(p)
        && let Some(t) = path.at(d).and_then(|s| s.tangent)
    {
        xf.rotation = libm::atan2f(t[1], t[0]);
    }
}

pub(crate) fn set_transform_field(
    xf: &mut Transform,
    b: &TargetBinding,
    path: Option<&crate::MotionPath>,
    v: AnimValue,
    orient: bool,
) {
    // O único canal cujo valor não é uma coordenada (ADR-0141): é uma DISTÂNCIA ao longo
    // do caminho, e virá um ponto quando se perguntar à trajetória onde ela está.
    if b.prop == PropKind::Position {
        // ⚠️ A trajetória é PASSADA, nunca lida do binding: ela mora no CLIP desde
        // 2026-07-30 (`NamedClip::paths`), e quem a resolve é quem sabe qual clip está a
        // dirigir — [`crate::TimelineDoc::path_for`], a porta única.
        let (Some(path), AnimValue::Float(s)) = (path, v) else {
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
    // **Um `LinkFrame` DEGENERADO** (ADR-0152 W6, §3): os prop-links (`Nome.prop`) resolvem à
    // pose VIVA da fonte, semeada do mundo. Uma expressão LOCAL (time/wiggle) o ignora e o
    // fantasma é EXATO; um prop-link entre objetos é APROXIMADO — lê a pose da fonte NESTE
    // playhead, não no tempo `t_entity` do fantasma, porque o onion não tem o grafo cross-time
    // (limitação declarada). Semear do mundo pós-apply mantém `pose_at == {apply; read}` para
    // canal keyado E expressão local, o que o gate de equivalência pina.
    let mut links = crate::frame_solve::LinkFrame {
        // ⚠️ A metade BOUND do mapa de nomes: o fantasma recebe `&World` de dentro do
        // extract, e os prop-links dele já são aproximados por construção (leem a fonte no
        // playhead vivo, não no tempo do fantasma). Alcançar uma fonte NÃO-bindada exigiria
        // empurrar `&mut` pelo caminho de render para deixar uma aproximação um pouco
        // menos aproximada.
        names: crate::frame_solve::build_names_bound(world, doc),
        ..Default::default()
    };
    crate::frame_solve::seed_links(world, doc, &mut links.links);
    for b in doc.bindings() {
        if b.entity != entity || b.missing || b.prop == PropKind::TimeRemap {
            continue;
        }
        // A MESMA porta que o apply lê (`solo_source_value`): amostra keyada OU a expressão
        // per-clip sobre ela, para que um fantasma dirigido case com o que o objeto mostra.
        // Track vazia sem expressão devolve `None` (esparsidade) — um binding recém-criado não
        // força pose default.
        let Some(v) = crate::stack_eval::solo_source_value(
            doc,
            doc.active_index(),
            b.target,
            t_entity,
            b.rest.unwrap_or(0.0),
            &links,
        ) else {
            continue;
        };
        let orient = b.prop == PropKind::Position && doc.auto_orient(entity) == AutoOrient::Active;
        set_transform_field(
            &mut xf,
            b,
            doc.path_for(b.target),
            AnimValue::Float(v),
            orient,
        );
    }
    Some(xf)
}

/// **Os instantes de keyframe de uma entidade**, em segundos, ordenados e sem repetição —
/// a UNIÃO dos tempos de key de todas as tracks dela (exceto o Time Remap, que é o
/// relógio, não uma pose). É o que o onion em modo *Keys* ghosta: as poses AUTORADAS
/// vizinhas, o modelo pose-a-pose do animador.
///
/// ⚠️ Tempos da TRACK (tempo-fonte). Sem Time Remap, tempo-fonte == tempo de clip, e o
/// onion amostra a pose neles diretamente; sob remap a vizinhança-por-key é aproximada
/// (a inversa `fonte→clip` não é única — a mesma razão pela qual o bake recusa instante
/// ambíguo). Uma coluna de keys `X`+`Y` no mesmo tempo conta UMA vez.
#[must_use]
pub fn entity_key_times(doc: &TimelineDoc, entity: u64) -> Vec<f64> {
    let clip = doc.active_clip();
    let mut out: Vec<f64> = doc
        .bindings()
        .iter()
        .filter(|b| b.entity == entity && !b.missing && b.prop != PropKind::TimeRemap)
        .filter_map(|b| clip.track(b.target))
        .flat_map(|tr| tr.keys().iter().map(|k| k.t.to_seconds()))
        .collect();
    out.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // Dedup de colunas alinhadas (X e Y keyados no mesmo tempo são UMA pose-fantasma).
    out.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    out
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
