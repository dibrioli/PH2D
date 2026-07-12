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
use ph2d_vec_scene::{
    VecPath, VecPathId, VecScene, VecXforms, boundary_hit, round_polyline, smooth_polyline,
    xform_of,
};

use crate::vec_entities::VecEntityMap;

/// O lado por onde cada ponta saiu no frame ANTERIOR (`[start, end]`), por conector.
///
/// É a memória da histerese de [`side_towards`]: sem ela, um alvo parado **em cima da
/// diagonal** da caixa faz os dois lados empatarem, e o menor tremor do arrasto troca a saída
/// a cada quadro. Runtime-only (não vai para o save nem para o undo) — o pior que um cache
/// perdido causa é a linha escolher o lado do zero, uma vez.
pub(crate) type SideCache = BTreeMap<VecPathId, [Option<Dir>; 2]>;

/// A tensão do spline do conector CURVO. `1.0` é a curva Catmull-Rom cheia — o cotovelo vira
/// uma volta redonda, que é o que se espera de "curvo".
const CURVE_TENSION: f64 = 1.0;

/// O quanto o spread pode deslizar ao longo da face, como fração da meia-extensão dela. A saída
/// tem de continuar **na face**: passando disto o ponto escorrega pela quina, e a linha parece
/// brotar do canto da caixa em vez de sair dela.
const SPREAD_FACE_K: f64 = 0.45;

/// O quanto a região de interesse se estende além das duas caixas, em múltiplos do jetty — a
/// folga em que uma forma ainda consegue empurrar a rota.
const ROI_PAD_K: f64 = 3.0;

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
///
/// # O `spread` desliza a ORIGEM do raio, não o resultado
///
/// É o que separa dois conectores no mesmo par de formas. Deslocar o ponto de saída DEPOIS de
/// achá-lo o tiraria do contorno — numa estrela ou numa nuvem ele passaria a flutuar ao lado da
/// forma. Deslizando a origem e disparando o raio de novo, o ponto continua **no contorno, por
/// construção**, seja qual for a forma. É de graça: o `boundary_hit` já parte de um ponto
/// arbitrário.
fn endpoint(
    scene: &VecScene,
    xforms: &VecXforms,
    me: &EndBox,
    other_center: [f64; 2],
    kind: RouteKind,
    prev: Option<Dir>,
    spread: f64,
) -> ([f64; 2], Dir) {
    let c = me.center();
    let (hw, hh) = me.half();
    let d = [other_center[0] - c[0], other_center[1] - c[1]];

    // Uma PORTA fixa (o ímã do Figma) já É um ponto na borda: ela não procura saída — e por
    // isso o spread não a toca. Quem fixou a porta quer a linha exatamente ali.
    if let (Anchor::Port { u, v }, Some(id)) = (me.anchor, me.path)
        && let Some(p) = port_world(scene, xforms, id, f64::from(u), f64::from(v))
    {
        let away = [p[0] - c[0], p[1] - c[1]];
        return (p, side_towards(away, hw, hh, prev));
    }

    let side = side_towards(d, hw, hh, prev);
    let ray = if kind == RouteKind::Straight {
        d
    } else {
        side.vec()
    };
    let n = perp(ray);

    let Some(id) = me.path else {
        // Ponta solta: não há face por onde deslizar, então o spread desloca o ponto direto.
        return ([c[0] + n[0] * spread, c[1] + n[1] * spread], side);
    };

    // O deslize é limitado pela FACE: além disso a saída escorrega pela quina.
    let limit = SPREAD_FACE_K * (n[0].abs() * hw + n[1].abs() * hh);
    let s = spread.clamp(-limit, limit);
    let from = [c[0] + n[0] * s, c[1] + n[1] * s];

    let at = scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .and_then(|p| boundary_hit(p, xform_of(xforms, id).0, from, ray, BOUNDARY_GAP))
        // Forma degenerada (sem contorno fechado: uma reta, um traço da caneta) — o contorno
        // não cruza nada. Cai na caixa, que é o que o draw.io faz SEMPRE.
        .unwrap_or_else(|| bbox_exit(me.bbox, from, ray));
    (at, side)
}

/// A normal unitária de `v` (girada 90° à esquerda). `[0, 0]` para um vetor degenerado — e aí o
/// spread simplesmente não desloca nada, que é o certo: não há face definida.
fn perp(v: [f64; 2]) -> [f64; 2] {
    let l = v[0].hypot(v[1]);
    if l < 1e-12 {
        return [0.0, 0.0];
    }
    [-v[1] / l, v[0] / l]
}

/// O ponto de uma porta `(u, v)` normalizada na caixa LOCAL da forma, levado ao mundo. Local ⇒
/// gira e escala com ela (ADR-0111), de graça.
///
/// **`u` e `v` NÃO são clampados a `[0, 1]`, e isso é a feature.** Um `u = 1.8` é um ponto além
/// da borda direita, medido na régua da própria forma — é assim que a alça de ponta afasta a
/// linha da forma **sem soltar o vínculo** (ver [`crate::connector_handles`]). Clampar aqui
/// (que era o que este código fazia) grudava a ponta de volta na borda e tornava o afastamento
/// impossível de expressar.
fn port_world(
    scene: &VecScene,
    xforms: &VecXforms,
    id: VecPathId,
    u: f64,
    v: f64,
) -> Option<[f64; 2]> {
    let (lo, hi) = scene.path_curve_bbox(id)?;
    let local = [lo[0] + (hi[0] - lo[0]) * u, lo[1] + (hi[1] - lo[1]) * v];
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

/// O jetty automático desta rota.
///
/// **A fórmula mora em [`ph2d_tool_vector::connector::auto_jetty`], não aqui** — e isso não é
/// arrumação. O painel precisa EXIBIR o valor efetivo (o automático, enquanto ninguém fixou
/// nada), então o mesmo número tem dois leitores: quem coze a rota e quem a mostra. Duas cópias
/// da fórmula significam que o painel passa a mentir no dia em que alguém mexer numa delas — é
/// a lição do tempo remapeado, que quebrou três vezes assim
/// ([[feedback_derived_coordinate_seed_must_match_sample]]): **semente e amostra saem da mesma
/// função.**
fn jetty_for(a: &Aabb, b: &Aabb) -> f64 {
    let half = |x: &Aabb| ((x.max[0] - x.min[0]) * 0.5, (x.max[1] - x.min[1]) * 0.5);
    ph2d_tool_vector::connector::auto_jetty(half(a), half(b))
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

/// Escreve a rota **em lugar** no path `id` — id, estilo e (portanto) entidade, seleção e
/// ponta de seta preservados. Ver o detalhe 1 do doc do módulo.
///
/// `radius > 0` arredonda as **dobras** do cotovelo ([`round_polyline`]). As duas PONTAS ficam
/// afiadas de propósito: é do primeiro e do último segmento que sai a tangente que orienta a
/// ponta de seta — arredondá-los faria a seta apontar para o lado.
fn write_route(scene: &mut VecScene, id: VecPathId, pts: &[[f64; 2]], radius: f64, curved: bool) {
    let Some(p) = scene.path_mut(id) else {
        return;
    };
    p.verts.clear();
    if curved {
        // O conector CURVO é a mesma rota, suavizada — o `corner_radius` não se aplica (o
        // spline já não tem quina nenhuma). Ver `RouteKind::Curved`.
        p.verts.extend(smooth_polyline(pts, CURVE_TENSION));
    } else {
        p.verts.extend(round_polyline(pts, radius));
    }
    p.closed = false;
    p.subpaths.clear();
}

/// Uma rota cozida: a polilinha em MUNDO + os lados por onde ela saiu (que viram o `prev` da
/// histerese no próximo frame).
struct Cooked {
    pts: Vec<[f64; 2]>,
    sides: [Option<Dir>; 2],
}

/// **Toda forma que pode barrar uma linha**: a caixa de MUNDO de cada path com contorno
/// fechado que não seja um conector.
///
/// Os dois filtros são a definição de "parede". Um **conector** não é obstáculo — se fosse,
/// o primeiro cruzamento entre duas linhas empurraria todas as outras, e o diagrama
/// desmancharia sozinho a cada traço novo. E um **contorno aberto** (um traço da caneta, uma
/// aresta interna de um cubo) não tem interior: não há o que atravessar. É o mesmo critério
/// do `boundary_hit`, e não por acaso — a borda em que a linha encosta e a parede de que ela
/// desvia têm de ser a MESMA borda.
///
/// Calculado uma vez por frame, não uma vez por conector.
fn shape_boxes(
    sim: &SimWorld,
    scene: &VecScene,
    xforms: &VecXforms,
    map: &VecEntityMap,
) -> Vec<Aabb> {
    let is_connector = |id: VecPathId| {
        map.get(&id).is_some_and(|&bits| {
            sim.world()
                .get::<VecConnector>(Entity::from_bits(bits))
                .is_some()
        })
    };
    let has_interior = |p: &VecPath| {
        (0..p.contour_count()).any(|c| matches!(p.contour(c), Some((v, true)) if v.len() >= 2))
    };
    scene
        .paths()
        .iter()
        .filter(|p| has_interior(p) && !is_connector(p.id))
        .filter_map(|p| {
            let (lo, hi) = scene.path_world_curve_bbox(xforms, p.id)?;
            Some(Aabb::new(lo, hi))
        })
        .collect()
}

/// **Os obstáculos que ESTA rota precisa enxergar.**
///
/// Passar o documento inteiro seria correto e caro: o grafo de visibilidade tem `(2n+3)²` nós,
/// então cada forma do desenho encareceria TODA rota — inclusive a que liga duas caixas
/// vizinhas no canto oposto da tela. A poda por região devolve o custo ao tamanho do problema:
/// numa rota curta sobram duas ou três caixas, e o grafo volta a ter dezenas de nós.
///
/// # A região CRESCE — e é por isso que isto é um ponto fixo, não um filtro
///
/// O filtro ingênuo (pegue quem cruza o corredor entre as duas pontas) tem um furo: uma forma
/// que atravessa a borda do corredor obriga a linha a **contorná-la**, e o contorno vai até a
/// borda OPOSTA dessa forma — que está fora do corredor original, possivelmente colada em
/// OUTRA forma, que o filtro descartou. A linha atravessaria essa segunda forma, e o desvio
/// pareceria simplesmente quebrado.
///
/// Então a seleção é iterativa: quem cruza a região entra, a região engole a caixa de quem
/// entrou, repete. Termina sempre (o conjunto só cresce, e é finito) e converge em uma ou duas
/// passadas num diagrama de verdade. Num diagrama denso ela pega tudo — que é a resposta
/// **certa**, e o preço de estar certo.
fn obstacles_in_play(shapes: &[Aabb], a: Aabb, b: Aabb, pad: f64) -> Vec<Aabb> {
    let mut roi = a.union(b).inflate(pad);
    let mut taken = vec![false; shapes.len()];
    loop {
        let mut grew = false;
        for (i, s) in shapes.iter().enumerate() {
            if !taken[i] && roi.overlaps(*s) {
                taken[i] = true;
                roi = roi.union(s.inflate(pad));
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    shapes
        .iter()
        .zip(&taken)
        .filter_map(|(s, &t)| t.then_some(*s))
        .collect()
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
    shapes: &[Aabb],
) -> Option<Cooked> {
    let (a, b) = (
        end_box(&conn.start, scene, xforms)?,
        end_box(&conn.end, scene, xforms)?,
    );
    // **O curvo ROTEIA como o ortogonal.** A busca é a mesma, os obstáculos são os mesmos; só a
    // escrita da geometria difere. É o que faz o conector curvo desviar de obstáculo de graça,
    // em vez de exigir um segundo roteador.
    let kind = match conn.route {
        ph2d_ecs::RouteKind::Straight => RouteKind::Straight,
        ph2d_ecs::RouteKind::Orthogonal | ph2d_ecs::RouteKind::Curved => RouteKind::Orthogonal,
    };
    let jetty = conn.jetty_or(jetty_for(&a.bbox, &b.bbox));
    let spread = conn.spread_or(ph2d_tool_vector::connector::SPREAD_STEP);
    let (p0, d0) = endpoint(scene, xforms, &a, b.center(), kind, prev[0], spread);
    let (p1, d1) = endpoint(scene, xforms, &b, a.center(), kind, prev[1], -spread);
    // **As duas pontas deslizam para lados OPOSTOS da linha** (`spread` e `−spread`): as normais
    // das duas faces apontam uma contra a outra, então o mesmo sinal as moveria em direções
    // contrárias no mundo — e o conector sairia torto em vez de paralelo.
    let obstacles = obstacles_in_play(shapes, a.bbox, b.bbox, ROI_PAD_K * jetty);
    let pts = route(&RouteInput {
        start: EndSpec { at: p0, dir: d0 },
        end: EndSpec { at: p1, dir: d1 },
        kind,
        jetty,
        obstacles: &obstacles,
        spread,
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

    // As paredes do diagrama — UMA vez por frame, não uma por conector (elas são as mesmas
    // para todos; a poda por rota é que difere).
    let shapes = shape_boxes(sim, scene, xforms, map);

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
        let Some(cooked) = cook(scene, xforms, &conn, prev, &shapes) else {
            continue;
        };
        cache.insert(id, cooked.sides);
        write_route(
            scene,
            id,
            &cooked.pts,
            f64::from(conn.corner_radius),
            conn.route == ph2d_ecs::RouteKind::Curved,
        );

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
