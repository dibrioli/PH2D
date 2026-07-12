//! **Conectores vivos** — a linha que gruda em duas formas e as SEGUE.
//!
//! Espelho exato do padrão da *Live Shape* ([`crate::vec_shape_live`]): o componente
//! [`VecConnector`] guarda a **relação** (a quem cada ponta se prende, e como a rota é
//! desenhada) e a geometria é uma **função pura** dela, re-cozida a cada frame. Ninguém
//! "move" um conector: move-se uma forma, e a linha se refaz.
//!
//! Consequência de graça (a mesma da Live Shape): **undo e save cobrem o conector sem uma
//! linha a mais** — os dois capturam o mundo ECS + a cena vetorial, e o `VecConnector` está
//! registrado no `ComponentRegistry`.
//!
//! # Os três detalhes que decidem se isto funciona
//!
//! 1. **Escreve a polilinha EM LUGAR** (`scene.path_mut(id)`), preservando o id. Remover e
//!    re-empurrar o path daria um id novo a cada frame — e a entidade, a seleção e o gizmo
//!    piscariam junto (o invariante documentado da Live Shape).
//! 2. **O lado de saída lembra do frame anterior** ([`SideCache`]). Sem o `prev` da
//!    histerese, um alvo passando pela diagonal da caixa faz a linha piscar entre sair pelo
//!    topo e sair pela lateral, quadro sim, quadro não.
//! 3. **O conector vive na IDENTIDADE.** A geometria que escrevemos aqui é MUNDO; se a
//!    entidade tivesse pose, o afim a aplicaria por cima e a rota sairia deslocada. Por isso
//!    `vec_transform::settle_origins` o **pula** (como já pula as formas vivas) e este
//!    módulo devolve o `Transform` à identidade se alguém o mexeu — o que torna o conector
//!    **não-arrastável pelo gizmo**, que é o comportamento certo: arrastar um conector não
//!    quer dizer nada; o que se move são as pontas dele.

use std::collections::BTreeMap;

use ph2d_ecs::{Anchor, ConnectorEnd, Entity, Name, SimWorld, Transform, VecConnector};
use ph2d_vec_connect::{Aabb, Dir, EndSpec, RouteInput, RouteKind, route, side_towards};
use ph2d_vec_scene::{VecPathId, VecScene, VecVertex, VecXforms, boundary_hit, xform_of};

use crate::vec_entities::VecEntityMap;

/// O lado por onde cada ponta saiu no frame ANTERIOR (`[start, end]`), por conector.
///
/// É a memória da histerese de [`side_towards`]: sem ela, um alvo parado **em cima da
/// diagonal** da caixa faz os dois lados empatarem, e o menor tremor do arrasto troca a saída
/// a cada quadro. Runtime-only (não vai para o save nem para o undo) — o pior que um cache
/// perdido causa é a linha escolher o lado do zero, uma vez.
pub(crate) type SideCache = BTreeMap<VecPathId, [Option<Dir>; 2]>;

/// O **jetty** (o quanto a linha avança reto antes de poder dobrar), como fração da menor
/// meia-extensão da maior das duas caixas. Relativo, e não em unidades fixas: no PH2D uma
/// forma tem ~1-2 unidades de mundo, mas nada impede uma de ter 50 — um jetty absoluto ficaria
/// invisível numa e gigante na outra.
const JETTY_K: f64 = 0.35;
/// Piso/teto do jetty (duas pontas soltas não têm caixa de onde derivá-lo).
const JETTY_MIN: f64 = 0.08;
const JETTY_MAX: f64 = 1.0;

/// Afastamento perpendicular por `parallel_index` — dois conectores no MESMO par de formas
/// não podem se sobrepor (o segundo sumiria exatamente por baixo do primeiro, e o usuário
/// juraria que ele não foi criado).
const SPREAD_STEP: f64 = 0.35;

/// A folga entre a forma e a ponta da linha. **Zero**: a linha ENCOSTA no contorno — é o que
/// faz a ponta de seta apontar para a forma, e não para perto dela. (O recuo visual da seta é
/// do render, que já a insere pelo `Marker::inset`.)
const BOUNDARY_GAP: f64 = 0.0;

/// Uma ponta resolvida: a caixa de MUNDO que ela ocupa (um ponto, se está solta), o path a que
/// se prende (se se prende a algum) e como ela acha a saída.
struct EndBox {
    bbox: Aabb,
    path: Option<VecPathId>,
    anchor: Anchor,
}

impl EndBox {
    fn center(&self) -> [f64; 2] {
        self.bbox.center()
    }
    /// Meias-extensões da caixa (`0` para uma ponta solta — [`side_towards`] clampa).
    fn half(&self) -> (f64, f64) {
        (
            (self.bbox.max[0] - self.bbox.min[0]) * 0.5,
            (self.bbox.max[1] - self.bbox.min[1]) * 0.5,
        )
    }
}

/// A caixa de MUNDO de uma ponta. `None` ⇒ **o alvo sumiu** (o objeto foi apagado) — o
/// chamador congela a ponta onde ela estava, em vez de deixar a linha voar para a origem.
fn end_box(end: &ConnectorEnd, scene: &VecScene, xforms: &VecXforms) -> Option<EndBox> {
    match *end {
        ConnectorEnd::Free { at } => Some(EndBox {
            bbox: Aabb::new(at, at),
            path: None,
            anchor: Anchor::Floating,
        }),
        ConnectorEnd::Bound { target, anchor } => {
            let (lo, hi) = scene.path_world_curve_bbox(xforms, target)?;
            Some(EndBox {
                bbox: Aabb::new(lo, hi),
                path: Some(target),
                anchor,
            })
        }
    }
}

/// **Onde a linha encosta, e por que lado ela sai.**
///
/// O lado vem do quadrante da diagonal da CAIXA (com a histerese de `prev`); o ponto vem do
/// contorno REAL da forma (`boundary_hit`) — não da bbox dela, que é o que o draw.io faz para
/// um stencil qualquer. Numa estrela a diferença é a linha sair da ponta em vez de flutuar
/// dentro do vale.
///
/// O raio: para a rota **reta**, do centro da forma rumo ao centro da outra (é por ali que a
/// linha vai passar); para a **ortogonal**, ao longo do eixo escolhido (a linha sai
/// perpendicular à face, e é essa a aparência de um fluxograma).
fn endpoint(
    scene: &VecScene,
    xforms: &VecXforms,
    me: &EndBox,
    other_center: [f64; 2],
    kind: RouteKind,
    prev: Option<Dir>,
) -> ([f64; 2], Dir) {
    let c = me.center();
    let (hw, hh) = me.half();
    let d = [other_center[0] - c[0], other_center[1] - c[1]];

    // Uma PORTA fixa (o ímã do Figma) já É um ponto na borda: ela não procura saída, e a
    // face por onde sai é aquela em que ela mesma se apoia.
    if let (Anchor::Port { u, v }, Some(id)) = (me.anchor, me.path)
        && let Some(p) = port_world(scene, xforms, id, f64::from(u), f64::from(v))
    {
        let away = [p[0] - c[0], p[1] - c[1]];
        return (p, side_towards(away, hw, hh, prev));
    }

    let side = side_towards(d, hw, hh, prev);
    let Some(id) = me.path else {
        return (c, side); // ponta solta: o ponto É ela mesma
    };
    let ray = if kind == RouteKind::Straight {
        d
    } else {
        side.vec()
    };
    let at = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| boundary_hit(p, xform_of(xforms, id).0, c, ray, BOUNDARY_GAP))
        // Forma degenerada (sem contorno fechado: uma reta, um traço da caneta) — o contorno
        // não cruza nada. Cai na caixa, que é o que o draw.io faz SEMPRE.
        .unwrap_or_else(|| bbox_exit(me.bbox, c, ray));
    (at, side)
}

/// O ponto de uma porta `(u, v)` normalizada na caixa LOCAL da forma, levado ao mundo. Local ⇒
/// gira e escala com ela (ADR-0111), de graça.
fn port_world(
    scene: &VecScene,
    xforms: &VecXforms,
    id: VecPathId,
    u: f64,
    v: f64,
) -> Option<[f64; 2]> {
    let (lo, hi) = scene.path_curve_bbox(id)?;
    let local = [
        lo[0] + (hi[0] - lo[0]) * u.clamp(0.0, 1.0),
        lo[1] + (hi[1] - lo[1]) * v.clamp(0.0, 1.0),
    ];
    Some(xform_of(xforms, id).apply(local))
}

/// Onde o raio `from + t·dir` SAI da caixa (o slab de maior `t`). O fallback do
/// `boundary_hit`: sem contorno fechado não há borda de verdade, e a caixa é a melhor
/// aproximação — nunca o centro, que deixaria a linha por baixo da forma.
fn bbox_exit(b: Aabb, from: [f64; 2], dir: [f64; 2]) -> [f64; 2] {
    let mut t = f64::INFINITY;
    for i in 0..2 {
        if dir[i].abs() > 1e-12 {
            let edge = if dir[i] > 0.0 { b.max[i] } else { b.min[i] };
            let tt = (edge - from[i]) / dir[i];
            if tt >= 0.0 {
                t = t.min(tt);
            }
        }
    }
    if !t.is_finite() {
        return from;
    }
    [from[0] + dir[0] * t, from[1] + dir[1] * t]
}

/// O jetty desta rota — ver [`JETTY_K`].
fn jetty_for(a: &Aabb, b: &Aabb) -> f64 {
    let smallest_half = |x: &Aabb| {
        let hw = (x.max[0] - x.min[0]) * 0.5;
        let hh = (x.max[1] - x.min[1]) * 0.5;
        hw.min(hh)
    };
    (JETTY_K * smallest_half(a).max(smallest_half(b))).clamp(JETTY_MIN, JETTY_MAX)
}

/// As pontas da polilinha ATUAL do conector (é onde elas "estavam" — a memória que uma ponta
/// órfã usa para congelar no lugar certo).
fn current_endpoints(scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| Some((p.verts.first()?.anchor, p.verts.last()?.anchor)))
        .unwrap_or(([0.0, 0.0], [0.0, 0.0]))
}

/// Escreve a polilinha **em lugar** no path `id` — id, estilo e (portanto) entidade,
/// seleção e ponta de seta preservados. Ver o detalhe 1 do doc do módulo.
fn write_polyline(scene: &mut VecScene, id: VecPathId, pts: &[[f64; 2]]) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    p.verts.clear();
    p.verts.extend(pts.iter().map(|&a| VecVertex::corner(a)));
    p.closed = false;
    p.subpaths.clear();
}

/// Uma rota cozida: a polilinha em MUNDO + os lados por onde ela saiu (que viram o `prev` da
/// histerese no próximo frame).
struct Cooked {
    pts: Vec<[f64; 2]>,
    sides: [Option<Dir>; 2],
}

/// A rota de `conn`, em MUNDO. Função TOTAL (o roteador é total): sempre uma polilinha
/// não-vazia, mesmo com as caixas sobrepostas ou as duas pontas soltas no mesmo ponto.
///
/// `None` só quando um dos alvos SUMIU — e aí o chamador já congelou a ponta antes.
fn cook(
    scene: &VecScene,
    xforms: &VecXforms,
    conn: &VecConnector,
    prev: [Option<Dir>; 2],
) -> Option<Cooked> {
    let (a, b) = (
        end_box(&conn.start, scene, xforms)?,
        end_box(&conn.end, scene, xforms)?,
    );
    let kind = match conn.route {
        ph2d_ecs::RouteKind::Straight => RouteKind::Straight,
        ph2d_ecs::RouteKind::Orthogonal => RouteKind::Orthogonal,
    };
    let (p0, d0) = endpoint(scene, xforms, &a, b.center(), kind, prev[0]);
    let (p1, d1) = endpoint(scene, xforms, &b, a.center(), kind, prev[1]);
    // Os obstáculos de hoje são as duas caixas terminais. Quando o desvio das OUTRAS formas
    // chegar, é este slice que cresce — o roteador não muda (é a aposta do ADR do módulo).
    let obstacles: Vec<Aabb> = [&a, &b]
        .iter()
        .filter(|e| e.path.is_some())
        .map(|e| e.bbox)
        .collect();
    let pts = route(&RouteInput {
        start: EndSpec { at: p0, dir: d0 },
        end: EndSpec { at: p1, dir: d1 },
        kind,
        jetty: jetty_for(&a.bbox, &b.bbox),
        obstacles: &obstacles,
        spread: f64::from(conn.parallel_index) * SPREAD_STEP,
        // Fonte e destino na MESMA forma: não há rota a buscar — é um laço, e ele é
        // construído, não roteado.
        self_loop: conn.is_self_loop().then_some(a.bbox),
    });
    Some(Cooked {
        pts,
        sides: [Some(d0), Some(d1)],
    })
}

/// **O re-cook de todo frame.** Para cada entidade com um [`VecConnector`]: resolve as duas
/// pontas, rota, e escreve a polilinha no `VecPath` dela — em lugar.
///
/// Roda DEPOIS de `vec_entities::sync` (a entidade existe) e depois de `vec_transform::build`
/// (os afins das formas-alvo já são os deste frame), e ANTES do render.
pub(crate) fn recook(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    xforms: &VecXforms,
    cache: &mut SideCache,
) {
    let conns: Vec<(VecPathId, Entity, VecConnector)> = map
        .iter()
        .filter_map(|(&id, &bits)| {
            let e = Entity::from_bits(bits);
            let c = sim.world().get::<VecConnector>(e)?.clone();
            Some((id, e, c))
        })
        .collect();
    if conns.is_empty() {
        cache.clear();
        return;
    }
    cache.retain(|id, _| conns.iter().any(|(c, _, _)| c == id));

    for (id, entity, mut conn) in conns {
        // 1. O alvo sumiu (o objeto foi apagado) ⇒ a ponta CONGELA onde estava. Apagar o
        //    conector junto destruiria trabalho; deixá-lo apontando para um id morto faria a
        //    linha sumir. Congelar é o que o draw.io faz — e o único que preserva o desenho.
        let (was_a, was_b) = current_endpoints(scene, id);
        let alive = |t: u64| scene.paths().iter().any(|p| p.id == t);
        if conn.freeze_missing(alive, was_a, was_b)
            && let Ok(mut e) = sim.world_mut().get_entity_mut(entity)
        {
            e.insert(conn.clone());
        }

        // 2. A rota. Depois do congelamento, as duas pontas SEMPRE resolvem.
        let prev = cache.get(&id).copied().unwrap_or([None; 2]);
        let Some(cooked) = cook(scene, xforms, &conn, prev) else {
            continue;
        };
        cache.insert(id, cooked.sides);
        write_polyline(scene, id, &cooked.pts);

        // 3. O conector vive na IDENTIDADE (detalhe 3 do doc): a geometria acima é MUNDO, e
        //    uma pose por cima a deslocaria. Devolver a identidade é o que torna o gizmo
        //    inócuo sobre ele — arrastar um conector não quer dizer nada.
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

/// Pendura (ou atualiza) o [`VecConnector`] na entidade do path `id`.
///
/// O gesto de canvas só conhece o **path** (ele empurra a linha na cena); a entidade nasce no
/// `vec_entities::sync` do frame. Este é o ponto onde os dois se encontram — espelho de
/// `vec_shape_live::make_committed_shape_live`. Idempotente: se o componente já é igual, não
/// escreve (senão o change-tick da ECS marcaria a entidade suja a cada frame do arrasto).
///
/// `true` se a entidade existia e o componente está lá.
pub(crate) fn attach(
    sim: &mut SimWorld,
    map: &VecEntityMap,
    id: VecPathId,
    conn: &VecConnector,
) -> bool {
    let Some(&bits) = map.get(&id) else {
        return false;
    };
    let entity = Entity::from_bits(bits);
    if sim.world().get::<VecConnector>(entity) == Some(conn) {
        return true;
    }
    let first = sim.world().get::<VecConnector>(entity).is_none();
    let Ok(mut e) = sim.world_mut().get_entity_mut(entity) else {
        return false;
    };
    e.insert(conn.clone());
    if first {
        // O nome que a Hierarquia mostra. "Path 7" para uma linha que ninguém desenhou seria
        // uma mentira — e é pelo nome que o usuário a acha na árvore.
        e.insert(Name::new(format!("Connector {id}")));
    }
    true
}

/// Pendura o componente do conector **em gesto** (o preview é o conector de verdade) e o do
/// recém-**fechado** (que espera a entidade dele nascer).
///
/// Roda entre o `vec_entities::sync` (a entidade existe) e o [`recook`] (que a lê). O
/// `pending` é uma fila de um item: um clique rápido fecha o gesto ANTES de qualquer sync, e
/// sem ele a linha ficaria na cena sem `VecConnector` — um traço inerte que não segue ninguém.
pub(crate) fn upkeep(
    sim: &mut SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    drag: Option<(VecPathId, &VecConnector)>,
    pending: &mut Option<(VecPathId, VecConnector)>,
) {
    if let Some((id, conn)) = drag {
        attach(sim, map, id, conn);
    }
    if let Some((id, conn)) = pending.as_ref() {
        // Ou a entidade chegou (attach), ou o path sumiu (undo/delete no mesmo frame): nos
        // dois casos a fila esvazia — nunca fica um item eterno tentando.
        let gone = !scene.paths().iter().any(|p| p.id == *id);
        if gone || attach(sim, map, *id, conn) {
            *pending = None;
        }
    }
}

#[cfg(test)]
#[path = "connector_live_tests.rs"]
mod tests;
