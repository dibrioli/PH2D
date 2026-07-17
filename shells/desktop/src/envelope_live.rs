//! **O host do Envelope vivo** (ADR-0123, Fatia B) — a forma deformada por uma gaiola de 4 cantos.
//!
//! Espelho do [`crate::morph_live`], e mais simples: o morph interpola DUAS fontes e cacheia um
//! `Plan`; o envelope deforma UMA fonte por um mapa `R2→R2` (a homografia do gesto Quad,
//! [`ph2d_vec_envelope::QuadWarp`]). A entidade que carrega o [`VecEnvelope`] tem o `VecPath` da
//! forma **cozida** (deformada) na cena, re-escrita *em lugar* a cada frame.
//!
//! # A fonte é CONGELADA no componente, e por isso o recook não precisa de xforms
//!
//! No morph, as fontes são OUTRAS formas da cena, e o recook as re-assa pelos afins DESTE frame
//! (elas se movem). No envelope, a fonte é a **própria** forma, autorada e imutável — assada em
//! MUNDO na criação e guardada nos bytes do [`VecEnvelope`]. O recook a desserializa, deforma pela
//! gaiola e escreve o resultado. Nada a re-assar por frame ⇒ **sem `VecXforms` no recook**.
//!
//! ⚠️ **Consequência declarada:** mover o objeto-envelope inteiro (gizmo) não está nesta fatia — a
//! fonte está congelada em mundo e a entidade vive na identidade (como o morph). Mover o conjunto,
//! e arrastar os cantos da gaiola (modo Node), são a fatia seguinte. Aqui a gaiola é **autorada**
//! (a cena de smoke a define) e a forma é **estática e correta**.

use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecEnvelope};
use ph2d_vec_envelope::{QuadWarp, warp_path};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene, VecXforms, bake_xform, xform_of};

use crate::vec_entities::VecEntityMap;

/// A bbox dos pontos de controle de um path (âncoras + handles, todos os contornos), como
/// `(origem, tamanho)`.
///
/// É a bbox de **controle**, não a da curva — ela CONTÉM a curva, que é o que o domínio-fonte do
/// envelope precisa (a arte cabe no quadrado unitário). E, em **repouso** (cantos = cantos desta
/// bbox), o `QuadWarp` é a identidade **independentemente** de a bbox ser justa ou folgada — então
/// a folga só afeta como a deformação se distribui quando um canto é puxado, nunca o repouso.
fn control_bbox(path: &VecPath) -> Option<([f64; 2], [f64; 2])> {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            lo = [lo[0].min(p[0]), lo[1].min(p[1])];
            hi = [hi[0].max(p[0]), hi[1].max(p[1])];
        }
    }
    (lo[0].is_finite() && hi[0] > lo[0] && hi[1] > lo[1])
        .then_some((lo, [hi[0] - lo[0], hi[1] - lo[1]]))
}

/// Os 4 cantos de uma bbox `(origem, tamanho)` na ordem `[BL, BR, TR, TL]` — a que o `QuadWarp`
/// espera, e a mesma em que `VecEnvelope::corners` é lido.
pub(crate) fn bbox_corners(origin: [f64; 2], size: [f64; 2]) -> [[f64; 2]; 4] {
    let [ox, oy] = origin;
    let [w, h] = size;
    [[ox, oy], [ox + w, oy], [ox + w, oy + h], [ox, oy + h]]
}

/// **Cria um envelope** sobre a forma `id`: assa a geometria dela em MUNDO (a aparência COZIDA, com
/// o raio de quina resolvido), guarda-a como fonte, e nasce em **repouso** (cantos = cantos do bbox
/// da fonte ⇒ identidade ⇒ a forma não muda).
///
/// Devolve `(id, componente)` para a fila `pending` — a entidade da forma já existe, mas o
/// `attach` acontece no `upkeep` (depois do `sync`), por simetria com o morph e para o caso de a
/// forma ter acabado de nascer.
///
/// `None` se a forma sumiu ou é degenerada (sem bbox).
pub(crate) fn create(
    scene: &VecScene,
    xforms: &VecXforms,
    id: VecPathId,
) -> Option<(VecPathId, VecEnvelope)> {
    let mut src = scene.paths().iter().find(|p| p.id == id)?.clone();
    bake_xform(&mut src, &xform_of(xforms, id));
    // A aparência COZIDA (raio de quina resolvido): é o que o mundo vê, e é o que o envelope
    // deforma. Sem raio, `cooked()` é a própria fonte emprestada (custo zero).
    let source = src.cooked().into_owned();
    let (origin, size) = control_bbox(&source)?;
    let corners = bbox_corners(origin, size);
    let bytes = postcard::to_allocvec(&source).ok()?;
    Some((id, VecEnvelope::at_rest(bytes, corners)))
}

/// A tolerância de fit (distância de Fréchet), **relativa** ao tamanho da fonte — 0,1% da diagonal
/// do bbox. Relativa, e não absoluta, para não ter precipício de escala: o mesmo envelope numa arte
/// grande ou minúscula fita com a mesma fidelidade *percebida*. É a grandeza que o Illustrator
/// vende como "Fidelity", só que aqui é fit adaptativo, não inserção por contagem.
fn accuracy(size: [f64; 2]) -> f64 {
    0.001 * size[0].hypot(size[1]).max(1.0)
}

/// **O re-cook de todo frame.** Para cada entidade com um [`VecEnvelope`]: desserializa a fonte,
/// deforma-a pela gaiola (`QuadWarp`), e escreve a forma **em lugar**.
///
/// Roda DEPOIS de `vec_entities::sync` (a entidade existe) e depois de `vec_transform::build`, no
/// mesmo lugar do `morph_live::recook` — mas sem os xforms (a fonte já é MUNDO).
pub(crate) fn recook(sim: &mut SimWorld, scene: &mut VecScene, map: &VecEntityMap) {
    let log = std::env::var("PH2D_ENVELOPE_LOG").is_ok();
    let envs: Vec<(VecPathId, Entity, VecEnvelope)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let env = sim.world().get::<VecEnvelope>(e)?.clone();
            Some((id, e, env))
        })
        .collect();
    if log && !envs.is_empty() {
        eprintln!("[envelope] recook: {} envelope(s) no mapa", envs.len());
    }

    for (id, entity, env) in envs {
        let Ok(source) = postcard::from_bytes::<VecPath>(&env.source) else {
            continue; // fonte corrompida: não há o que deformar, e melhor não escrever lixo
        };
        let Some((origin, size)) = control_bbox(&source) else {
            continue;
        };
        // Gaiola degenerada (3 cantos colineares): o `QuadWarp` recusa, e a forma **congela** onde
        // está — a mesma escolha do morph com uma fonte perdida. Nunca some, nunca vira lixo.
        let Some(warp) = QuadWarp::new(origin, size, env.corners) else {
            if log {
                eprintln!(
                    "[envelope] id={id}: gaiola degenerada, CONGELA (corners={:?})",
                    env.corners
                );
            }
            continue;
        };
        let cooked = warp_path(&source, &warp, accuracy(size));
        if log {
            eprintln!(
                "[envelope] id={id}: deformou {} -> {} verts | bbox origin={origin:?} size={size:?} | corners={:?}",
                source.verts.len(),
                cooked.verts.len(),
                env.corners
            );
        }
        write_shape(scene, id, cooked);

        // Vive na IDENTIDADE: a geometria acima é MUNDO. É a mesma regra do morph, e é ela que põe o
        // componente no filtro do `settle_origins` (senão o pivô seria assentado e a forma sairia
        // deslocada da posição que ela descreve).
        if sim
            .world()
            .get::<Transform>(entity)
            .is_some_and(|t| *t != Transform::IDENTITY)
            && let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
        {
            *t = Transform::IDENTITY;
        }
    }
}

/// Escreve a forma deformada no path `id`, **em lugar** — o estilo (fill/stroke) vem da fonte, que
/// o `warp_path` preserva. Em lugar, e não `push_path`, pela mesma razão do morph: um id novo por
/// frame faria entidade/seleção/gizmo **piscarem**.
fn write_shape(scene: &mut VecScene, id: VecPathId, cooked: VecPath) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    p.verts = cooked.verts;
    p.subpaths = cooked.subpaths;
    p.closed = cooked.closed;
    p.fill_rule = cooked.fill_rule;
    p.fill = cooked.fill;
    p.stroke = cooked.stroke;
}

/// Pendura (ou atualiza) o [`VecEnvelope`] na entidade do path `id` — espelho de
/// `morph_live::attach`. Idempotente.
pub(crate) fn attach(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    env: &VecEnvelope,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecEnvelope>(entity) == Some(env) {
        return true;
    }
    let first = sim.world().get::<VecEnvelope>(entity).is_none();
    let Ok(mut e) = sim.world_mut().get_entity_mut(entity) else {
        return false;
    };
    e.insert(env.clone());
    if first {
        e.insert(Name::new(format!("Envelope {id}")));
    }
    true
}

/// Drena a fila `pending` (o envelope recém-criado, à espera da entidade nascer no `sync`) —
/// espelho de `morph_live::upkeep`. Roda entre o `sync` e o [`recook`], **antes** do `settle`:
/// sem o componente pendurado antes do `settle`, o path seria assentado como um path comum e o
/// recook do frame seguinte sairia deslocado.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    pending: &mut Option<(VecPathId, VecEnvelope)>,
) {
    if let Some((id, env)) = pending.as_ref() {
        let gone = !scene.paths().iter().any(|p| p.id == *id);
        if gone || attach(sim, map, *id, env) {
            *pending = None;
        }
    }
}

#[cfg(test)]
#[path = "envelope_live_tests.rs"]
mod tests;
