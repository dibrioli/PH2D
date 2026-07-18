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
    ChildOf, Entity, EnvelopeKind, Name, RootOrder, SimWorld, Transform, VecEnvelope,
    VecEnvelopeChild,
};
use ph2d_vec_envelope::{CoonsWarp, QuadWarp, rest_edges, warp_path};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

use crate::vec_entities::VecEntityMap;

/// A bbox-**união** dos pontos de controle (âncoras + handles, todos os contornos) de várias fontes,
/// como `(origem, tamanho)`. É a bbox de **controle**, não a da curva — ela CONTÉM a curva, que é o
/// domínio-fonte que o `QuadWarp` precisa (a arte cabe no quadrado unitário).
///
/// **Porta única:** o `create` (para semear os cantos em repouso) e o `recook` (para montar o mapa)
/// chamam a MESMA função — se as duas computassem o domínio de formas diferentes, o repouso deixaria
/// de ser a identidade. `None` se as fontes são vazias/degeneradas.
fn union_control_bbox<'a>(
    paths: impl IntoIterator<Item = &'a VecPath>,
) -> Option<([f64; 2], [f64; 2])> {
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
        let world_xf = crate::vec_transform::xform_of_transform(
            crate::vec_transform::world_transform(sim, entity),
        );
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
        .spawn((
            Transform::default(),
            Name::new("Envelope"),
            RootOrder(order),
        ))
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
        children.push(VecEnvelopeChild {
            path: id,
            source: bytes,
        });
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
        .insert(VecEnvelope::at_rest(
            children,
            corners,
            rest_edges(&corners),
        ));
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
        let acc = accuracy(size);
        // O gesto escolhe o MAPA, e só isso — a espinha (desserializar · deformar · escrever) é a
        // mesma. Gaiola degenerada (3 cantos colineares no Quad): o mapa recusa, e as formas
        // **congelam** onde estão — a mesma escolha do morph com uma fonte perdida. Nunca somem,
        // nunca viram lixo.
        match env.kind {
            EnvelopeKind::Perspective => {
                let Some(warp) = QuadWarp::new(origin, size, env.corners) else {
                    continue;
                };
                cook_children(scene, sources, &warp, acc);
            }
            EnvelopeKind::Mesh => {
                let Some(warp) = CoonsWarp::new(origin, size, env.corners, &env.edges) else {
                    continue;
                };
                cook_children(scene, sources, &warp, acc);
            }
        }
    }
}

/// Deforma cada fonte pelo `warp` e escreve o resultado no path do filho.
///
/// Genérica no mapa de propósito: é a **porta única** por onde os dois gestos passam, então nada que
/// dependa de *qual* mapa é (a tolerância, a escrita em lugar, o estilo preservado) pode divergir
/// entre eles. Trocar de gesto muda uma linha do `match` acima, e mais nada.
fn cook_children(
    scene: &mut VecScene,
    sources: Vec<(VecPathId, VecPath)>,
    warp: &impl ph2d_vec_envelope::Warp,
    acc: f64,
) {
    for (id, src) in sources {
        write_shape(scene, id, warp_path(&src, warp, acc));
    }
}

/// **Carimba um preset de gaiola** (ADR-0129 Fatia C) no envelope `bits`: Arc / Flag / Wave / …
/// com força `bend ∈ [-1, 1]`. `true` se algo foi escrito.
///
/// **O preset REESCREVE a gaiola inteira** — cantos de volta ao repouso, lados envergados pela
/// tabela — e põe o gesto em `Mesh` (com lados retos não há preset nenhum a exprimir; ver
/// `ph2d_vec_envelope::preset` sobre por que "Arc com 4 cantos" é um trapézio). É o *Reset with
/// Warp* do Illustrator: pedir um arco depois de puxar um canto dá um arco, não um arco torto.
///
/// O preset é escrito no **quadrado unitário** e mapeado aqui para o retângulo-fonte por um afim de
/// escala. ⚠️ A escala é **não-uniforme**, então a barriga deixa de ser exatamente perpendicular ao
/// lado — e isso é o que se quer: o preset acompanha a proporção da arte, em vez de ficar circular
/// numa forma achatada.
pub(crate) fn apply_preset(
    sim: &mut SimWorld,
    bits: u64,
    warp: ph2d_ecs::EnvelopeWarp,
    bend: f64,
) -> bool {
    let entity = Entity::from_bits(bits);
    let Some((origin, size)) = sim.world().get::<VecEnvelope>(entity).and_then(rest_domain) else {
        return false;
    };
    // A tabela do componente é a FORMA (±1); a amplitude é do motor, e é ela que carrega a garantia
    // de não-dobra. Multiplicar aqui mantém as duas metades onde elas pertencem.
    let bows: ph2d_vec_envelope::EdgeBows =
        warp.bows().map(|s| s.map(|v| v * ph2d_vec_envelope::AMP));
    let (uc, ue) = ph2d_vec_envelope::preset_cage(&bows, bend);
    let to_rect = |p: [f64; 2]| [origin[0] + p[0] * size[0], origin[1] + p[1] * size[1]];

    let Some(mut env) = sim.world_mut().get_mut::<VecEnvelope>(entity) else {
        return false;
    };
    env.corners = std::array::from_fn(|i| to_rect(uc[i]));
    env.edges = std::array::from_fn(|i| std::array::from_fn(|j| to_rect(ue[i][j])));
    env.kind = EnvelopeKind::Mesh;
    env.warp = Some(warp);
    env.bend = bend.clamp(-1.0, 1.0);
    true
}

/// O retângulo-fonte (repouso) de um envelope: a bbox-união de controle das **fontes autoradas**.
///
/// Passa pelo MESMO [`union_control_bbox`] que o `create` e o `recook` — é o que garante que um
/// preset com `bend = 0` devolva exatamente a gaiola em repouso, e não uma quase-igual.
fn rest_domain(env: &VecEnvelope) -> Option<([f64; 2], [f64; 2])> {
    let sources: Vec<VecPath> = env
        .children
        .iter()
        .filter_map(|c| postcard::from_bytes(&c.source).ok())
        .collect();
    union_control_bbox(sources.iter())
}

/// Teto de profundidade da caminhada de ancestrais (defesa contra árvore corrompida).
const MAX_DEPTH: usize = 64;

/// **O container de Envelope que governa `bits`** — ele mesmo, ou o ancestral mais próximo que
/// carrega um [`VecEnvelope`]. `None` se nada na cadeia é um envelope.
///
/// **Porta única** da pergunta *"de quem é este envelope?"*: a seleção
/// ([`crate::vec_selection`], que decide selecionar-só-o-container) e o [`dissolve`] a fazem por
/// AQUI. Duas cópias divergiriam — e a que esquecesse de aninhar deixaria um envelope inalcançável.
#[must_use]
pub(crate) fn container_of(sim: &SimWorld, bits: u64) -> Option<u64> {
    let w = sim.world();
    let mut cur = Entity::from_bits(bits);
    for _ in 0..MAX_DEPTH {
        if w.get::<VecEnvelope>(cur).is_some() {
            return Some(cur.to_bits());
        }
        cur = w.get::<ph2d_ecs::ChildOf>(cur)?.parent();
    }
    None
}

/// O container que **todos** os bits compartilham, ou `None` — se a lista está vazia, se algum bit
/// não vive sob um envelope, ou se há dois envelopes diferentes (a seleção não é "um envelope").
#[must_use]
pub(crate) fn sole_container(sim: &SimWorld, selected: &[u64]) -> Option<u64> {
    let mut found: Option<u64> = None;
    for &b in selected {
        let c = container_of(sim, b)?;
        match found {
            None => found = Some(c),
            Some(f) if f == c => {}
            Some(_) => return None,
        }
    }
    found
}

/// O que sobra no path de cada filho quando a gaiola se dissolve ([`dissolve`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Keep {
    /// **Expand:** a geometria DEFORMADA (a que está na tela) vira a forma definitiva. A deformação
    /// deixa de ser uma relação viva e passa a ser o desenho.
    Deformed,
    /// **Release:** a fonte AUTORADA volta — a deformação é desfeita e o artista recupera o original.
    Authored,
}

/// **Dissolve** os envelopes tocados pela seleção: cada filho vira uma forma comum de novo e o
/// container morre. `Keep` decide QUAL geometria fica (Expand = a deformada · Release = a autorada).
///
/// **A pose do container é ASSADA na geometria de cada filho** — o inverso exato do que o [`create`]
/// fez na entrada. É o que mantém a arte onde ela está quando o envelope foi movido/girado/escalado
/// pelo gizmo: o filho volta a ser raiz **em MUNDO, na identidade**, que é o estado de uma forma
/// recém-desenhada — e o `settle_origins` do frame seguinte lhe dá o pivô no centro, de graça.
/// (Transferir a pose do container para cada filho seria a alternativa; ela só é limpa com UM filho,
/// e teria de decidir o que fazer com a rotação/escala compartilhada de N.)
///
/// A seleção passa às formas libertadas — o artista fica com o material na mão e a seleção não
/// aponta para um container morto. `true` se algum envelope foi dissolvido.
pub(crate) fn dissolve(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    pen: &mut ph2d_vec_edit::PenTool,
    keep: Keep,
) -> bool {
    // Os containers tocados pela seleção (dedup — vários filhos do mesmo envelope dão UM container).
    let mut containers: Vec<u64> = Vec::new();
    for id in pen.selected_paths() {
        if let Some(&bits) = map.get(id)
            && let Some(c) = container_of(sim, bits)
            && !containers.contains(&c)
        {
            containers.push(c);
        }
    }
    if containers.is_empty() {
        return false;
    }

    let mut freed: Vec<VecPathId> = Vec::new();
    for c in containers {
        let entity = Entity::from_bits(c);
        let Some(env) = sim.world().get::<VecEnvelope>(entity).cloned() else {
            continue;
        };
        // A pose do container, a assar na geometria de cada filho.
        let world_xf = crate::vec_transform::xform_of_transform(
            crate::vec_transform::world_transform(sim, entity),
        );
        for child in &env.children {
            // Expand fica com o que está no path (o deformado que o recook escreveu); Release
            // ressuscita a fonte autorada. Nos dois casos a pose do container entra na geometria.
            let geom = match keep {
                Keep::Deformed => scene.paths().iter().find(|p| p.id == child.path).cloned(),
                Keep::Authored => postcard::from_bytes::<VecPath>(&child.source).ok(),
            };
            let Some(mut geom) = geom else { continue };
            ph2d_vec_scene::bake_xform(&mut geom, &world_xf);
            write_shape(scene, child.path, geom);
            // O filho volta a ser RAIZ: sem pai, com ordem própria, na identidade (a geometria já é
            // mundo). É o estado de uma forma recém-desenhada — o `settle` lhe dá o pivô depois.
            if let Some(&bits) = map.get(&child.path) {
                let order = crate::vec_entities::next_root_order(sim);
                if let Ok(mut ce) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
                    ce.remove::<ph2d_ecs::ChildOf>();
                    ce.insert(RootOrder(order));
                    ce.insert(Transform::IDENTITY);
                }
            }
            freed.push(child.path);
        }
        // O container morre com a gaiola. Sem filhos e sem path, ele não é nada.
        if let Ok(ce) = sim.world_mut().get_entity_mut(entity) {
            ce.despawn();
        }
    }
    if !freed.is_empty() {
        pen.select_many(&freed);
    }
    !freed.is_empty()
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
