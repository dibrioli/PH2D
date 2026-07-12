//! **Rótulos vivos** — o texto que pertence a uma forma (ou a um conector) e a SEGUE.
//!
//! Espelho exato do [`crate::connector_live`]: o componente [`VecLabel`] guarda a **relação**
//! (de quem o rótulo é, e onde em relação a ele) e a **pose** é uma função pura dela, re-derivada
//! a cada frame. Ninguém "sincroniza" um rótulo: move-se a forma, e o texto vai junto.
//!
//! Consequência de graça (a mesma do conector): **undo e save o cobrem sem uma linha a mais** —
//! os dois capturam o mundo ECS, e o `VecLabel` está registrado no `ComponentRegistry`.
//!
//! # Os três detalhes que decidem se isto funciona
//!
//! 1. **A âncora de um CONECTOR é o meio da rota POR COMPRIMENTO DE ARCO**, não o vértice do
//!    meio da polilinha — que numa rota em "L" é exatamente a QUINA, e o rótulo pousaria em
//!    cima do cotovelo (ver [`arclength_midpoint`]).
//!
//! 2. **O passe roda DEPOIS do `connector_live::recook`** (senão a polilinha do conector ainda
//!    é a do frame anterior, e o rótulo fica um frame atrás da linha) e DEPOIS do
//!    `vec_transform::build` (os afins das formas-alvo já são os deste frame).
//!
//! 3. **A pose que escrevemos fica em CACHE** ([`LabelPoses`]). É o que distingue "o hospedeiro
//!    se moveu" de "o USUÁRIO arrastou o rótulo": no frame seguinte, um `Transform` diferente do
//!    que gravamos só pode ter vindo do gizmo ⇒ o deslocamento é **absorvido no offset** em vez
//!    de brigar com ele.
//!
//!    Sem essa distinção caímos num dos dois bugs, e não há meio-termo: quem SEMPRE dirige faz o
//!    rótulo saltar de volta ao centro no instante em que o usuário o solta; quem SEMPRE absorve
//!    o desprende da forma (o offset engoliria o movimento do hospedeiro, e o alvo nunca mudaria).

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, Name, SimWorld, Transform, VecConnector, VecLabel};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, xform_of};

use crate::vec_entities::VecEntityMap;

/// A pose que ESTE passe escreveu no frame anterior, por rótulo. Runtime-only (não vai para o
/// save nem para o undo): o pior que um cache perdido causa é um frame de absorção idempotente
/// — o estado restaurado é auto-consistente (`centro = âncora + offset`), então re-absorver
/// devolve o MESMO offset.
pub(crate) type LabelPoses = BTreeMap<VecPathId, Transform>;

/// Amostras por segmento cúbico ao achatar a rota de um conector. O mesmo número que as bboxes
/// de curva do documento usam — a rota é quase sempre uma polilinha (handles coincidentes), e aí
/// a amostragem é exata de graça.
const CURVE_SAMPLES: usize = 16;

/// Ponto de uma cúbica de Bézier em `t`.
fn cubic(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0],
        a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1],
    ]
}

/// A rota do path `id` achatada em MUNDO — o primeiro contorno, amostrado. Vazia se o path sumiu.
fn polyline_world(scene: &VecScene, xforms: &VecXforms, id: VecPathId) -> Vec<[f64; 2]> {
    let Some(path) = scene.paths().iter().find(|p| p.id == id) else {
        return Vec::new();
    };
    let Some((verts, closed)) = path.contour(0) else {
        return Vec::new();
    };
    let n = verts.len();
    if n == 0 {
        return Vec::new();
    }
    let x = xform_of(xforms, id);
    let segs = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(segs * CURVE_SAMPLES + 1);
    out.push(x.apply(verts[0].anchor));
    for s in 0..segs {
        let a = &verts[s];
        let b = &verts[(s + 1) % n];
        for j in 1..=CURVE_SAMPLES {
            let t = j as f64 / CURVE_SAMPLES as f64;
            out.push(x.apply(cubic(a.anchor, a.out_handle, b.in_handle, b.anchor, t)));
        }
    }
    out
}

/// **O meio da polilinha por COMPRIMENTO DE ARCO** — o ponto que divide a linha em duas metades
/// de mesmo comprimento.
///
/// A alternativa preguiçosa (o vértice do meio, ou o ponto médio dos extremos) parece a mesma
/// coisa numa reta e está errada em tudo mais. Numa rota ortogonal em "L" — o caso COMUM de um
/// fluxograma — o vértice do meio é exatamente a **quina**, e o rótulo pousa em cima do cotovelo;
/// e se o "L" tem um braço curto e um longo, o ponto médio dos extremos nem cai sobre a linha.
/// Por arco, o rótulo cai onde qualquer um apontaria com o dedo ao dizer "no meio da seta".
fn arclength_midpoint(pts: &[[f64; 2]]) -> Option<[f64; 2]> {
    let first = *pts.first()?;
    let seg = |a: [f64; 2], b: [f64; 2]| (b[0] - a[0]).hypot(b[1] - a[1]);
    let total: f64 = pts.windows(2).map(|w| seg(w[0], w[1])).sum();
    if total <= 1e-12 {
        return Some(first); // rota degenerada (as duas pontas no mesmo ponto)
    }
    let half = total * 0.5;
    let mut acc = 0.0;
    for w in pts.windows(2) {
        let d = seg(w[0], w[1]);
        if acc + d >= half {
            let t = (half - acc) / d;
            return Some([
                w[0][0] + (w[1][0] - w[0][0]) * t,
                w[0][1] + (w[1][1] - w[0][1]) * t,
            ]);
        }
        acc += d;
    }
    pts.last().copied()
}

/// **A âncora do hospedeiro** — o ponto de que o rótulo pende. `None` ⇒ o hospedeiro SUMIU (foi
/// apagado), e o chamador dissolve o vínculo.
///
/// Uma FORMA ancora no centro da bbox de MUNDO da curva dela (o mesmo `path_world_curve_bbox` do
/// gizmo e do align — a bbox local de toda forma assentada está centrada na origem, ADR-0112, e
/// compará-las localmente poria todos os rótulos no mesmo lugar). Um CONECTOR ancora no meio da
/// rota, por arco.
fn anchor_of(
    sim: &SimWorld,
    scene: &VecScene,
    xforms: &VecXforms,
    map: &VecEntityMap,
    host: VecPathId,
) -> Option<[f64; 2]> {
    let is_connector = map.get(&host).is_some_and(|&bits| {
        sim.world()
            .get::<VecConnector>(Entity::from_bits(bits))
            .is_some()
    });
    if is_connector {
        return arclength_midpoint(&polyline_world(scene, xforms, host));
    }
    let (lo, hi) = scene.path_world_curve_bbox(xforms, host)?;
    Some([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// O centro da bbox de MUNDO do próprio rótulo — o ponto que o passe leva até o alvo.
///
/// **É a bbox, não a translação**, e é isso que torna o passe robusto: seja qual for o pivô do
/// texto (a origem do `Transform` de um objeto de texto fica onde ele foi criado), o que se vê
/// centrado na forma é a CAIXA das letras.
fn world_centre(scene: &VecScene, xforms: &VecXforms, id: VecPathId) -> Option<[f64; 2]> {
    let (lo, hi) = scene.path_world_curve_bbox(xforms, id)?;
    Some([(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5])
}

/// **O passe de todo frame.** Para cada entidade com um [`VecLabel`]: resolve a âncora do
/// hospedeiro, decide entre ABSORVER (o usuário arrastou) e DIRIGIR (o hospedeiro se moveu), e
/// escreve o `Transform` do rótulo.
///
/// `editing` é o path da sessão de texto ABERTA, e ele é uma exceção necessária: enquanto se
/// digita, `vec_text_object::upsert_text_shape` reescreve a pose do texto todo frame
/// (`origin + center`, e o `center` muda a cada letra). Sem esta guarda o passe leria essa
/// escrita como um arrasto do usuário e absorveria o crescimento da palavra no offset — o rótulo
/// se afastaria da forma a cada tecla.
///
/// `xforms` é atualizado no lugar para os rótulos que se moveram: o render deste frame lê o mapa
/// DEPOIS de nós, e sem isso o texto apareceria um frame atrás da própria pose.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    xforms: &mut VecXforms,
    editing: Option<VecPathId>,
    cache: &mut LabelPoses,
) {
    let labels: Vec<(VecPathId, Entity, VecLabel)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let l = *sim.world().get::<VecLabel>(e)?;
            Some((id, e, l))
        })
        .collect();
    if labels.is_empty() {
        cache.clear();
        return;
    }
    cache.retain(|id, _| labels.iter().any(|(l, _, _)| l == id));

    for (id, entity, mut label) in labels {
        // 1. O hospedeiro sumiu (o objeto foi apagado) ⇒ o rótulo vira um TEXTO COMUM: só o
        //    vínculo morre. Apagá-lo junto destruiria trabalho do usuário (ele digitou aquilo),
        //    e mantê-lo apontando para um id morto o deixaria sem âncora — congelado num limbo
        //    que nada mais move. É a mesma filosofia do `freeze_missing` do conector.
        let Some(anchor) = anchor_of(sim, scene, xforms, map, label.host) else {
            if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
                e.remove::<VecLabel>();
            }
            cache.remove(&id);
            continue;
        };
        let Some(centre) = world_centre(scene, xforms, id) else {
            continue; // rótulo sem geometria (string vazia): nada a posicionar ainda
        };
        let now = sim
            .world()
            .get::<Transform>(entity)
            .copied()
            .unwrap_or(Transform::IDENTITY);

        // 2. A pose MUDOU em relação à que NÓS escrevemos? Só o gizmo pode ter feito isso ⇒
        //    ABSORVE o deslocamento no offset (detalhe 3 do doc do módulo).
        if cache.get(&id).is_some_and(|c| *c != now) && editing != Some(id) {
            label.absorb(centre, anchor);
            if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
                e.insert(label);
            }
        }

        // 3. DIRIGE a pose a partir de `âncora + offset`. Depois de absorver isto é um no-op
        //    exato (`absorb` é o inverso de `target`) — o rótulo NÃO salta de volta.
        let target = label.target(anchor);
        translate_pose(sim, entity, [target[0] - centre[0], target[1] - centre[1]]);

        // 4. Cacheia o que FICOU gravado (não o que pretendíamos): é contra este valor que o
        //    frame seguinte compara, então ele tem de ser o do mundo, arredondamento de `f32`
        //    incluso — senão todo frame pareceria um arrasto.
        let after = sim
            .world()
            .get::<Transform>(entity)
            .copied()
            .unwrap_or(Transform::IDENTITY);
        cache.insert(id, after);
        let x = crate::vec_transform::xform_of_transform(crate::vec_transform::world_transform(
            sim, entity,
        ));
        if x.is_identity() {
            xforms.remove(&id);
        } else {
            xforms.insert(id, x);
        }
    }
}

/// Anda com a pose de `entity` um `delta` de MUNDO.
///
/// A translação de um `Transform` vive no espaço do PAI, não no mundo — um rótulo filho de um
/// grupo girado receberia o delta torto se o escrevêssemos cru. Desfazer o afim do pai é o mesmo
/// que `vec_transform::move_origin_to` faz, e pela mesma razão.
///
/// # A pose é `f32`, e o `delta` nunca chega exatamente a zero
///
/// A translação é `f32`; a âncora, a bbox e o delta são `f64`. O ida-e-volta deixa um resíduo da
/// ordem do ulp (`~1e-7` numa coordenada de ~10), então "já está no lugar" jamais aparece como
/// `delta == 0` — e um epsilon fixo estaria errado, porque o ulp de `f32` CRESCE com a
/// magnitude. A guarda certa é a única exata: converta para `f32` e compare com o que já está lá.
/// Sem ela, `get_mut` reescreveria o mesmo número todo frame e marcaria a entidade suja para
/// sempre (é a mesma cautela do `connector_live`, que só devolve a identidade se ela não estiver
/// lá).
fn translate_pose(sim: &mut SimWorld, entity: Entity, delta: [f64; 2]) {
    let parent = ph2d_ecs::parent_world_transform(sim.world(), entity);
    let d = crate::vec_transform::xform_of_transform(parent)
        .inverse()
        .map_or(delta, |inv| inv.apply_vec(delta));
    let Some(t) = sim.world().get::<Transform>(entity).copied() else {
        return;
    };
    let next = ph2d_core::Vec2::new(
        (f64::from(t.translation.x) + d[0]) as f32,
        (f64::from(t.translation.y) + d[1]) as f32,
    );
    if next == t.translation {
        return; // a pose já É esta em `f32`: escrever só sujaria o change-tick
    }
    if let Some(mut m) = sim.world_mut().get_mut::<Transform>(entity) {
        m.translation = next;
    }
}

/// Pendura o [`VecLabel`] na entidade do path `id` e lhe dá um nome que o usuário reconheça na
/// Hierarquia (`"<hospedeiro> Label"`). Espelho de `connector_live::attach`.
///
/// **NÃO sobrescreve um vínculo existente**: o offset ali dentro pode ser o arrasto que o usuário
/// acabou de dar, e re-anexar o zeraria.
///
/// `true` se a entidade existia e o componente está lá.
pub(crate) fn attach(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    host: VecPathId,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecLabel>(entity).is_some() {
        return true;
    }
    // O nome do hospedeiro, para o rótulo se chamar "Process Label" e não "Path 12" — é pelo
    // nome que o usuário o acha na árvore.
    let base = map
        .get(&host)
        .and_then(|&b| sim.world().get::<Name>(Entity::from_bits(b)))
        .map_or_else(|| format!("Path {host}"), |n| n.0.clone());
    let Ok(mut e) = sim.world_mut().get_entity_mut(entity) else {
        return false;
    };
    e.insert(VecLabel::on(host));
    e.insert(Name::new(format!("{base} Label")));
    true
}

/// Pendura o vínculo no rótulo **recém-nascido** — o texto cuja sessão o duplo-clique acabou de
/// abrir sobre uma forma.
///
/// A fila de um item existe porque um rótulo nasce **vazio**: sem geometria não há `VecPath`,
/// sem path não há entidade, e sem entidade não há onde pendurar o componente. O vínculo só pode
/// ser feito quando a primeira letra materializa o objeto — e é aqui, todo frame, que os dois se
/// encontram (espelho do `vec_connect_pending`).
///
/// A sessão terminar (Esc, troca de modo, clique fora) **desarma** a fila: um texto que o usuário
/// abandonou vazio nunca chega a existir, e o arm não pode sobrar para grudar no PRÓXIMO texto.
pub(crate) fn upkeep_pending(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    pending: &mut Option<VecPathId>,
    editing: Option<VecPathId>,
    session_alive: bool,
) {
    if !session_alive {
        *pending = None;
        return;
    }
    let (Some(host), Some(id)) = (*pending, editing) else {
        return; // ainda sem geometria: o texto está vazio
    };
    if attach(sim, map, id, host) {
        *pending = None;
    }
}

/// O rótulo de `host`, se ele já tem um: `(path, entidade, params do texto)`.
///
/// A busca é pelo `host` do vínculo, e **não** pelo que está sob o cursor: o usuário dá o
/// duplo-clique na FORMA, e o rótulo dela pode estar deslocado (ele foi arrastado) — procurar
/// "um texto aqui" acharia nada e criaria um segundo rótulo por cima do primeiro.
fn label_of(
    sim: &SimWorld,
    map: &VecEntityMap,
    host: VecPathId,
) -> Option<(VecPathId, Entity, ph2d_ecs::VecTextParams)> {
    map.iter().find_map(|(&id, &bits)| {
        let e = Entity::from_bits(bits);
        if sim.world().get::<VecLabel>(e)?.host != host {
            return None;
        }
        match sim.world().get::<ph2d_ecs::VecShape>(e) {
            Some(ph2d_ecs::VecShape::Text(p)) => Some((id, e, p.clone())),
            _ => None,
        }
    })
}

/// O que o duplo-clique achou: um rótulo que já existe (com a sessão dele reconstruída) ou um
/// hospedeiro nu (e a âncora onde o rótulo novo nasce).
///
/// A sessão vai numa `Box`: ela é uma ordem de grandeza maior que a outra variante (`VecTextEdit`
/// carrega a string, a família e os eixos), e um `enum` do tamanho da maior seria copiado inteiro
/// no caminho comum — o do duplo-clique que não achou rótulo nenhum.
enum LabelHit {
    Reopen(VecPathId, Box<crate::vec_text::VecTextEdit>),
    New(VecPathId, [f64; 2]),
}

impl crate::app_state::App {
    /// **Duplo-clique numa FORMA ou num CONECTOR, no modo Select ⇒ o RÓTULO dele.** Se não há,
    /// nasce um (vazio, com a sessão de digitação aberta); se há, entra na edição do existente.
    /// É o gesto do draw.io/Figma, e o primeiro que o usuário tenta.
    ///
    /// Chamado pelo `vec_text_double_click` **depois** de o duplo-clique falhar em achar um
    /// TEXTO sob o cursor — a ordem importa: o rótulo fica POR CIMA da forma, então quem clica
    /// nele quer editá-lo (o caminho do texto), e quem clica na forma quer o rótulo dela (este).
    ///
    /// `true` = a sessão está aberta (o chamador consome a pressão).
    pub(crate) fn vec_label_double_click_at(&mut self, world: [f64; 2]) -> bool {
        /// O mesmo raio de captura do clique de seleção.
        const HIT_PX: f64 = 10.0;
        let tol = HIT_PX * self.vec_px_to_world();
        // Fase de LEITURA (o `gfx` fica emprestado). Sai daqui a decisão pronta, e o borrow
        // morre antes de qualquer mutação — `vec_set_draw_mode` precisa do `&mut self`.
        let hit = {
            let Some(gfx) = self.gfx.as_ref() else {
                return false;
            };
            let Some(host) = self.vec_pen.path_at(&gfx.vec_scene, world, tol) else {
                return false; // o duplo-clique caiu no vazio
            };
            if let Some((lid, entity, params)) = label_of(&gfx.sim, &self.vec_entities, host) {
                let edit = crate::vec_text_reopen::reopen_text_session(
                    &gfx.sim,
                    lid,
                    entity,
                    &params,
                    &gfx.vec_scene,
                );
                LabelHit::Reopen(lid, Box::new(edit))
            } else {
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                let Some(anchor) =
                    anchor_of(&gfx.sim, &gfx.vec_scene, &xf, &self.vec_entities, host)
                else {
                    return false;
                };
                LabelHit::New(host, anchor)
            }
        };
        match hit {
            LabelHit::Reopen(lid, edit) => {
                self.vec_pen.select_many(&[lid]);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Text);
                self.vec_text_edit = Some(*edit);
                self.vec_text_regen();
            }
            LabelHit::New(host, anchor) => {
                // O rótulo herda o Style ATIVO do painel, como qualquer texto.
                let (fill, stroke) =
                    crate::vec_glyph::resolve_style(&self.vec_pen.style(), self.vec_px_to_world());
                self.vec_text_edit = Some(crate::vec_text::VecTextEdit {
                    // A âncora é só a semente do caret na 1ª letra: quem manda na pose é o
                    // [`upkeep`], que a re-deriva do hospedeiro a cada frame.
                    origin: anchor,
                    size: self.vec_text_size,
                    weight: self.vec_text_weight,
                    line_height: self.vec_text_line_height,
                    tracking: self.vec_text_tracking,
                    align: self.vec_text_align,
                    extra_axes: self.vec_text_extra_axes.clone(),
                    family: self.vec_text_family.clone(),
                    fill,
                    stroke,
                    text: String::new(),
                    id: None,
                    center: [0.0, 0.0],
                });
                // O vínculo espera a 1ª letra criar o objeto (ver [`upkeep_pending`]).
                self.vec_label_pending = Some(host);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Text);
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "label_live_tests.rs"]
mod tests;
