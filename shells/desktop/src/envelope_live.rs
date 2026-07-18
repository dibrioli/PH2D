//! **O host do Envelope vivo** (ADR-0129) — um **container** cujos filhos têm a geometria deformada
//! por uma gaiola de 4 cantos comum.
//!
//! Espelho do [`crate::morph_live`], e diferente no que deforma: o morph interpola DUAS fontes e
//! cacheia um `Plan`; o envelope deforma N fontes (1..) por um mapa `R2→R2` (a homografia do gesto
//! Quad, [`ph2d_vec_envelope::QuadWarp`]). A entidade que carrega o [`VecEnvelope`] é um **container**
//! (sem `VecPathRef`); cada FILHO tem o `VecPath` da forma **cozida** (deformada) na cena, re-escrita
//! *em lugar* a cada frame.
//!
//! # Um só modelo: o container sempre (`N=1` inclusive)
//!
//! Um envelope de uma forma só é o caso `N=1`; um de várias é o *warp group* do Affinity. Modelar o
//! de-um como path-com-componente e o de-vários como container seriam DOIS modelos do mesmo fato — e
//! recook/gesto/gizmo teriam de perguntar "qual dos dois é este?" em cada sítio. Aqui há um só: o
//! **container**, com 1..N filhos.
//!
//! # A fonte + a gaiola vivem em LOCAL do CONTAINER; a POSE vive no `Transform` (ADR-0111)
//!
//! Ao criar, cada forma-fonte tem a sua geometria COZIDA **assada em MUNDO** e guardada como a fonte
//! do filho; o filho é reparentado sob o container e posto na **identidade** (`Transform::IDENTITY`),
//! e o container nasce na identidade também — então, no nascimento, **container-local == mundo**. O
//! recook desserializa cada fonte, deforma pela gaiola (uma só, comum) e escreve o resultado LOCAL no
//! path de cada filho. Quem leva tudo ao MUNDO é o `Transform` do **container** — e é por isso que o
//! **gizmo de sprite move/gira/escala o envelope inteiro no Select** (Fatia 2): a geometria dos
//! filhos é estável, só a pose do container se move, e o recook **preserva a pose**.
//!
//! ⚠️ **Por que não dobra (ADR §3.3):** o Blend dobra porque a geometria dele depende de FONTES que
//! se movem; a do envelope é função pura de `corners`+fontes FIXOS, então no Select nada a muda e o
//! gizmo age sobre uma bbox estável. Arrastar os cantos (que MUDA a geometria) é Node-only. O
//! `settle_origins` PULA os filhos (têm `ChildOf` — geometria já em mundo) e o container (não está no
//! mapa de paths).

use ph2d_ecs::{
    ChildOf, Entity, Name, RootOrder, SimWorld, Transform, VecEnvelope, VecEnvelopeChild,
};
use ph2d_vec_envelope::{QuadWarp, warp_path};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// A bbox-**união** dos pontos de controle (âncoras + handles, todos os contornos) de várias fontes,
/// como `(origem, tamanho)`. É a bbox de **controle**, não a da curva — ela CONTÉM a curva, que é o
/// domínio-fonte que o `QuadWarp` precisa (a arte cabe no quadrado unitário).
///
/// **Porta única:** o `create` (para semear os cantos em repouso) e o `recook` (para montar o mapa)
/// chamam a MESMA função — se as duas computassem o domínio de formas diferentes, o repouso deixaria
/// de ser a identidade. `None` se as fontes são vazias/degeneradas.
fn union_control_bbox<'a>(paths: impl IntoIterator<Item = &'a VecPath>) -> Option<([f64; 2], [f64; 2])> {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for path in paths {
        for v in path.verts_all() {
            for p in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(p[0]), lo[1].min(p[1])];
                hi = [hi[0].max(p[0]), hi[1].max(p[1])];
            }
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

/// A tolerância de fit (distância de Fréchet), **relativa** ao tamanho do domínio — 0,1% da diagonal
/// da bbox-união. Relativa, e não absoluta, para não ter precipício de escala: o mesmo envelope numa
/// arte grande ou minúscula fita com a mesma fidelidade *percebida*.
fn accuracy(size: [f64; 2]) -> f64 {
    0.001 * size[0].hypot(size[1]).max(1.0)
}

/// **Cria um envelope** sobre as formas `ids` (1..N): assa a geometria COZIDA de cada uma em MUNDO
/// (com o raio de quina resolvido) como fonte do filho, reparenta os filhos sob um **container** novo
/// na identidade, põe cada filho na identidade, e a gaiola nasce em **repouso** (cantos = bbox-união
/// das fontes ⇒ identidade ⇒ as formas não mudam). A POSE fica no `Transform` do container e o recook
/// a preserva — por isso o gizmo move/gira/escala o envelope inteiro no Select (Fatia 2).
///
/// **Síncrono** (≠ morph/blend, que têm `pending`+`upkeep`): o morph/blend CRIAM um path novo (o
/// spine/morfada) que precisa nascer no `sync`; o envelope não cria path nenhum — envolve formas que
/// **já existem** e adiciona só um container SEM path. Então tudo (assar + reparentar + pendurar)
/// acontece aqui, com as entidades que já estão na mão.
///
/// Devolve os bits do container criado (para a seleção/gizmo), ou `None` se nenhuma forma resolveu
/// (ids sumidos) ou o domínio-união é degenerado.
pub(crate) fn create(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    ids: &[VecPathId],
) -> Option<u64> {
    // 1. Assa a geometria COZIDA de cada forma em MUNDO. A pose do filho vira parte da geometria: é
    //    isso que permite ao filho ir para a identidade (a fonte já está no espaço do container).
    //    `baked` = (id, entidade, geometria de mundo) em paralelo, na ordem de `ids`.
    let mut baked: Vec<(VecPathId, Entity, VecPath)> = Vec::new();
    for &id in ids {
        let Some(&bits) = map.get(&id) else { continue };
        let entity = Entity::from_bits(bits);
        if sim.world().get_entity(entity).is_err() {
            continue;
        }
        let Some(src) = scene.paths().iter().find(|p| p.id == id) else {
            continue;
        };
        // A aparência COZIDA em LOCAL do filho (raio de quina resolvido), assada pela pose de MUNDO
        // do filho → geometria de MUNDO. Sem raio, `cooked()` empresta a fonte (custo zero).
        let world_xf = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
            sim, entity,
        ));
        let mut world_path = src.cooked().into_owned();
        ph2d_vec_scene::bake_xform(&mut world_path, &world_xf);
        baked.push((id, entity, world_path));
    }
    if baked.is_empty() {
        return None;
    }
    // 2. A gaiola em repouso = bbox-união (em MUNDO = local do container ao nascer) das fontes.
    let (origin, size) = union_control_bbox(baked.iter().map(|(_, _, p)| p))?;
    let corners = bbox_corners(origin, size);

    // 3. Container novo na identidade + `RootOrder` explícito (nunca `u32::MAX`, senão a árvore o
    //    desempataria por bits de alocação — BUGS #15). Nome como os grupos.
    let order = crate::vec_entities::next_root_order(sim);
    let container = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Envelope"), RootOrder(order)))
        .id();

    // 4. Para cada filho: escreve a geometria de MUNDO no path agora (o frame fica correto ANTES do
    //    recook, que em repouso reescreve o mesmo), guarda a fonte, reparenta sob o container e o põe
    //    na IDENTIDADE — a fonte já é MUNDO (passo 1), então uma pose própria a aplicaria DUAS vezes.
    let mut children: Vec<VecEnvelopeChild> = Vec::with_capacity(baked.len());
    for (id, entity, world_path) in baked {
        let Ok(bytes) = postcard::to_allocvec(&world_path) else {
            continue;
        };
        write_shape(scene, id, world_path);
        if let Ok(mut ce) = sim.world_mut().get_entity_mut(entity) {
            ce.remove::<RootOrder>();
            ce.insert(ChildOf(container));
            ce.insert(Transform::IDENTITY);
        }
        children.push(VecEnvelopeChild { path: id, source: bytes });
    }
    if children.is_empty() {
        // Nenhuma fonte serializou: desfaz o container órfão e desiste.
        if let Ok(c) = sim.world_mut().get_entity_mut(container) {
            c.despawn();
        }
        return None;
    }

    // 5. Pendura o componente no container.
    sim.world_mut()
        .get_entity_mut(container)
        .ok()?
        .insert(VecEnvelope::at_rest(children, corners));
    Some(container.to_bits())
}

/// **O re-cook de todo frame.** Para cada container com um [`VecEnvelope`]: desserializa as fontes
/// LOCAIS, monta UMA gaiola sobre a bbox-união delas (`QuadWarp`) e escreve cada filho deformado **em
/// lugar** no path dele.
///
/// Roda DEPOIS de `vec_entities::sync` e no mesmo lugar do `morph_live::recook`. Toma `&mut SimWorld`
/// para varrer os containers por QUERY (eles não têm path, logo não estão no `VecEntityMap`) — como o
/// `flip_entities` já faz. **A pose NÃO é tocada:** o recook escreve só GEOMETRIA (`scene`), e é o
/// `Transform` do container (via `vec_transform::build`) que a leva ao mundo. É essa pose que o gizmo
/// move no Select (Fatia 2).
pub(crate) fn recook(sim: &mut SimWorld, scene: &mut VecScene) {
    let envs: Vec<VecEnvelope> = {
        let mut q = sim.world_mut().query::<&VecEnvelope>();
        q.iter(sim.world()).cloned().collect()
    };
    for env in envs {
        // Desserializa as fontes; uma corrompida é PULADA (não há o que deformar, e melhor não
        // escrever lixo) — o filho fica com a última geometria boa.
        let mut sources: Vec<(VecPathId, VecPath)> = Vec::new();
        for child in &env.children {
            if let Ok(src) = postcard::from_bytes::<VecPath>(&child.source) {
                sources.push((child.path, src));
            }
        }
        let Some((origin, size)) = union_control_bbox(sources.iter().map(|(_, p)| p)) else {
            continue;
        };
        // Gaiola degenerada (3 cantos colineares): o `QuadWarp` recusa, e as formas **congelam** onde
        // estão — a mesma escolha do morph com uma fonte perdida. Nunca somem, nunca viram lixo.
        let Some(warp) = QuadWarp::new(origin, size, env.corners) else {
            continue;
        };
        let acc = accuracy(size);
        for (id, src) in sources {
            let cooked = warp_path(&src, &warp, acc);
            write_shape(scene, id, cooked);
        }
    }
}

/// Escreve a forma deformada no path `id`, **em lugar** — o estilo (fill/stroke) vem da fonte, que o
/// `warp_path` preserva. Em lugar, e não `push_path`, pela mesma razão do morph: um id novo por frame
/// faria entidade/seleção/gizmo **piscarem**.
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

#[cfg(test)]
#[path = "envelope_live_tests.rs"]
mod tests;
