//! Gates do **Contour VIVO** — arquivo irmão de `contour_live.rs`.
//!
//! O oráculo é a geometria DERIVADA que o `dispatch` desenharia (`ContourLive::live()`): quantos
//! caminhos, em que ordem, com que cor. As mutações que estes gates existem para matar: o `recook`
//! não ler o componente (0 anéis), a ordem de desenho invertida (a fonte tapa os anéis), a rampa
//! não chegar ao alvo, e o memo re-cozer o prefixo quando só a contagem muda.

use super::{ContourLive, spec_of};
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::{Name, SimWorld, Transform, VecContour, VecPathRef};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecPathId, VecScene, VecVertex, VecXforms};

/// Um quadrado de lado 1 com preenchimento PRETO — a cor de partida da rampa, escolhida para o
/// alvo branco a tornar óbvia.
fn scene() -> (VecScene, SimWorld, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(0, 0, 0, 255))),
        ..VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Shape"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (scene, sim, map, id)
}

fn arm(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, c: VecContour) {
    let e = ph2d_ecs::Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(c);
}

fn fill_of(p: &VecPath) -> [u8; 4] {
    match &p.fill {
        Some(Paint::Solid(c)) => [c.r, c.g, c.b, c.a],
        _ => [0, 0, 0, 0],
    }
}

/// O bbox das ÂNCORAS de um caminho — a régua de "cresceu ou encolheu". ⚠️ Fiável só para o join
/// **Round** (o default): a Miter crava spikes de auto-interseção na erosão que devolvem o bbox ao
/// tamanho da fonte (ver `probe_ring_loops`). Todos os gates aqui usam o default (Round).
fn anchor_bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in &p.verts {
        lo[0] = lo[0].min(v.anchor[0]);
        lo[1] = lo[1].min(v.anchor[1]);
        hi[0] = hi[0].max(v.anchor[0]);
        hi[1] = hi[1].max(v.anchor[1]);
    }
    (lo, hi)
}

/// **N passos produzem N anéis MAIS a fonte**, e a fonte é a ÚLTIMA (por cima) quando os anéis
/// crescem — senão eles a tapariam e o efeito seria um blob da cor do alvo.
///
/// ⚠️ Mutação: inverter a ordem em `assemble` põe a fonte no fundo ⇒ o último caminho deixa de ter
/// a tinta dela ⇒ RED.
#[test]
fn the_rings_stack_behind_the_source_when_they_grow() {
    let (scene, mut sim, map, id) = scene();
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 3,
            d: 0.1,
            to: [255, 255, 255, 255],
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let out = &live.live()[&id];
    assert!(
        out.len() >= 4,
        "3 anéis + a fonte, veio {} caminhos",
        out.len()
    );
    assert_eq!(
        fill_of(out.last().expect("último")),
        [0, 0, 0, 255],
        "a fonte (preta) tem de ser desenhada POR CIMA dos anéis que crescem"
    );
}

/// **A rampa chega ao alvo no último anel** — é o que faz a progressão de cor ser uma progressão
/// e não um degradê que para no meio.
///
/// O oráculo é a COR emitida, e o anel mais distante é o primeiro da lista (a ordem maior→menor).
///
/// ⚠️ Mutação: `ramp_t` devolver `k/steps` sem chegar a 1, ou o `assemble` usar `ring_distance`
/// no lugar dela, tira o último anel do alvo ⇒ RED.
#[test]
fn the_ramp_reaches_the_target_on_the_outermost_ring() {
    let (scene, mut sim, map, id) = scene();
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 4,
            d: 0.08,
            to: [255, 255, 255, 255],
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let out = &live.live()[&id];
    let outer = fill_of(&out[0]);
    assert!(
        outer[0] > 250 && outer[1] > 250 && outer[2] > 250,
        "o anel mais distante devia estar no alvo (branco), está em {outer:?}"
    );
    // E o mais próximo tem de estar mais perto da FONTE que do alvo (a rampa tem duas pontas).
    let inner = fill_of(&out[out.len() - 2]);
    assert!(
        inner[0] < 128,
        "o anel mais próximo devia estar perto da fonte (preta), está em {inner:?}"
    );
}

/// **Acrescentar um passo reusa os anéis já cozidos** — o invariante que torna o slider de passos
/// barato, e a razão inteira de o `d` ser POR PASSO em vez de alcance total.
///
/// O oráculo é a GEOMETRIA: os anéis que já existiam têm de sair **idênticos** depois de a
/// contagem subir. Se a distância dependesse de `steps`, todos se moveriam.
///
/// ⚠️ **O que este gate NÃO prova, e onde isso é provado:** normalizar a distância por `steps`
/// **sobrevive** aqui — porque o memo não é invalidado por uma mudança de contagem, e os anéis
/// viriam do cache na mesma. Eles ficariam parados e STALE. A propriedade é da função pura e está
/// pinada em `ph2d_ecs::vec_contour::tests::the_ring_distance_ignores_the_step_count`. Este aqui
/// prova a outra metade: que o cozimento REUSA em vez de re-cozer.
#[test]
fn adding_a_step_leaves_the_existing_rings_where_they_were() {
    let (scene, mut sim, map, id) = scene();
    let base = VecContour {
        steps: 2,
        d: 0.1,
        to: [255, 255, 255, 255],
        ..VecContour::default()
    };
    arm(&mut sim, &map, id, base);
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    // O anel k=1 é o penúltimo da lista (a fonte é a última, e a ordem é maior→menor).
    let before: Vec<[f64; 2]> = live.live()[&id][live.live()[&id].len() - 2]
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();

    arm(&mut sim, &map, id, VecContour { steps: 5, ..base });
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let after: Vec<[f64; 2]> = live.live()[&id][live.live()[&id].len() - 2]
        .verts
        .iter()
        .map(|v| v.anchor)
        .collect();

    assert_eq!(before.len(), after.len(), "o anel k=1 mudou de forma");
    for (a, b) in before.iter().zip(&after) {
        assert!(
            (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
            "o anel k=1 ANDOU ao subir a contagem ({a:?} -> {b:?}) — a distância está a depender \
             de `steps` e o memo não pode reusar nada"
        );
    }
}

/// **Outer + offset NEGATIVO ENCOLHE para dentro** — o bug que o Enio reportou (*"para negativo,
/// fica como na foto"*, i.e. crescia). A causa era o `offset_ring` escolher o laço do
/// `kurbo::stroke` pelo shoelace, que MENTE numa forma côncava (a estrela CRESCIA ao encolher);
/// curado escolhendo pelo SINAL da área (a erosão sai winding-invertida — `probe_ring_loops`).
///
/// A fonte fica ATRÁS (primeira) quando os anéis encolhem — senão ela os taparia —, e o anel da
/// frente é MENOR que a fonte `[0,1]²`.
///
/// ⚠️ Mutação: `offset_ring` voltar a escolher por `area().abs()` ⇒ a erosão de uma quina Round
/// ainda encolhe, mas o gate irmão da booleana (`the_ring_matches_the_booleana`) pega a estrela.
/// Aqui a mutação que sangra é `signed_dists` do Outer negar o sinal ⇒ os anéis CRESCEM ⇒ RED.
#[test]
fn outer_with_negative_offset_shrinks_inward() {
    let (scene, mut sim, map, id) = scene();
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 3,
            d: -0.1,
            side: 0, // Outer
            to: [255, 255, 255, 255],
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let out = &live.live()[&id];
    assert!(out.len() >= 4, "3 anéis + a fonte, veio {}", out.len());
    assert_eq!(
        fill_of(&out[0]),
        [0, 0, 0, 255],
        "quando os anéis ENCOLHEM a fonte tem de ficar ATRÁS (primeira), senão os tapa"
    );
    let (lo, hi) = anchor_bbox(out.last().expect("anel da frente"));
    assert!(
        lo[0] > 0.0 && lo[1] > 0.0 && hi[0] < 1.0 && hi[1] < 1.0,
        "Outer + `d` negativo devia ENCOLHER para dentro de [0,1]², bbox {lo:?}..{hi:?} \
         (o bug: crescia para fora)"
    );
}

/// **Side:Inner produz anéis PARA DENTRO** — o report do Enio (*"Inner completamente bugado, não
/// aparece nada"*). A causa: `OffsetSide::Inner` move só os FUROS, e a estrela do smoke não tem
/// nenhum ⇒ nada se movia. Curado tornando o `side` do Contour a DIREÇÃO (§ do módulo): Inner é o
/// ESPELHO do Outer, então com `d` positivo os anéis vão para DENTRO (o *Inside* do Corel).
///
/// ⚠️ Mutação: `signed_dists` do Inner devolver `sd` em vez de `-sd` ⇒ os anéis CRESCEM ⇒ RED. E o
/// comportamento ANTIGO (furos) deixaria os anéis do tamanho da fonte ⇒ bbox == `[0,1]²` ⇒ RED.
#[test]
fn inner_side_makes_rings_go_inward() {
    let (scene, mut sim, map, id) = scene();
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 3,
            d: 0.1,
            side: 1, // Inner
            to: [255, 255, 255, 255],
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let out = &live.live()[&id];
    assert!(
        out.len() >= 4,
        "Inner tem de produzir anéis VISÍVEIS (o report: nada aparecia), veio {}",
        out.len()
    );
    let (lo, hi) = anchor_bbox(out.last().expect("anel da frente"));
    assert!(
        lo[0] > 0.0 && lo[1] > 0.0 && hi[0] < 1.0 && hi[1] < 1.0,
        "Inner devia produzir anéis PARA DENTRO de [0,1]², bbox {lo:?}..{hi:?}"
    );
}

/// **Side:Both produz anéis nos DOIS sentidos** — para fora E para dentro, simétrico. É o terceiro
/// modo da direção, distinto de Outer e Inner.
///
/// ⚠️ Mutação: `signed_dists` do Both devolver só `[sd]` ⇒ metade dos anéis (contagem N+1, não
/// 2N+1) e nada para dentro ⇒ RED.
#[test]
fn both_side_makes_rings_on_both_directions() {
    let (scene, mut sim, map, id) = scene();
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 3,
            d: 0.1,
            side: 2, // Both
            to: [255, 255, 255, 255],
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    let out = &live.live()[&id];
    assert!(
        out.len() >= 7,
        "Both devia dar 2N anéis + a fonte (7), veio {}",
        out.len()
    );
    // O de TRÁS (primeiro) é o mais para FORA; o da FRENTE (último) o mais para DENTRO.
    let (back_lo, back_hi) = anchor_bbox(&out[0]);
    let (front_lo, front_hi) = anchor_bbox(out.last().expect("anel da frente"));
    assert!(
        back_hi[0] > 1.0 && back_lo[0] < 0.0,
        "o anel de TRÁS devia estar para FORA de [0,1]², bbox {back_lo:?}..{back_hi:?}"
    );
    assert!(
        front_hi[0] < 1.0 && front_lo[0] > 0.0,
        "o anel da FRENTE devia estar para DENTRO de [0,1]², bbox {front_lo:?}..{front_hi:?}"
    );
}

/// **Sem componente não há anéis** — e a forma volta a desenhar-se sozinha (entrada AUSENTE, que é
/// o que faz o `dispatch` cair na fonte).
#[test]
fn a_shape_without_a_contour_is_left_alone() {
    let (scene, sim, map, id) = scene();
    let mut live = ContourLive::default();
    live.recook(&scene, &sim, &map, &VecXforms::default());
    assert!(spec_of(&sim, &map, id).is_none());
    assert!(
        live.live().get(&id).is_none(),
        "sem contour a entrada tem de estar AUSENTE (presente-e-vazia apagaria a forma)"
    );
}

/// **Arrastar o slider não faz nenhum anel PISCAR** — o gate do report do Enio (*"o efeito não é
/// contínuo, mas dá saltos como se piscasse"*).
///
/// # A contagem de anéis é estável durante o arrasto
///
/// O que o artista via como piscar era um anel a **desaparecer e voltar** — o `linesweeper`
/// panicava em muitas distâncias, o `offset_path` devolvia vazio, e o anel sumia. Isso foi curado
/// na RAIZ pelo offset DIRETO (`offset_ring`), que cobre o caso comum sem tocar o `linesweeper`.
///
/// ⚠️ **Este gate NÃO distingue o mecanismo** — ele prova só que a contagem é estável, e isso é
/// verdade tanto com o offset direto (curado) quanto com a booleana+`last_good` (mascarado). Quem
/// distingue, e prova o FPS, é o irmão `dragging_a_plain_contour_never_calls_the_booleana` (conta
/// as chamadas ao sweep). Este fica como guarda de que a estabilidade sobrevive a qualquer
/// mudança futura. O fixture é um HEXÁGONO de propósito: num quadrado o sweep respondia sempre — a
/// fixture tem de conter o fenômeno que o offset direto agora previne.
#[test]
fn dragging_the_offset_never_makes_a_ring_blink() {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Polygon,
        [1.0, -1.2],
        [3.4, 1.2],
        &[6.0],
    ));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Hex"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    if let Some(p) = scene.path_mut(id) {
        p.fill = Some(Paint::solid(Rgba8::new(0, 0, 0, 255)));
    }
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 6,
            d: 0.0,
            accel: 1.0,
            join: 1, // Round: a quina em que o motor de offset falha mais
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    let xf = VecXforms::new();
    let (mut prev, mut drops) = (usize::MAX, 0);
    for i in 1..=240 {
        let d = f64::from(i) * 0.005;
        let ent = ph2d_ecs::Entity::from_bits(map[&id]);
        if let Some(mut c) = sim.world_mut().get_mut::<VecContour>(ent) {
            c.d = d;
        }
        live.recook(&scene, &sim, &map, &xf);
        let n = live.live().get(&id).map_or(0, Vec::len);
        if prev != usize::MAX && n < prev {
            drops += 1;
        }
        prev = n;
    }
    assert_eq!(
        drops, 0,
        "a contagem de caminhos CAIU {drops} vezes durante o arrasto — é o piscar que o Enio \
         reportou: um anel que o sweep não conseguiu cozer sumiu do quadro em vez de ficar onde \
         estava (a guarda `last_good` do `ensure`)"
    );
}

/// **O arrasto de `d` num contour comum NÃO chama a booleana** — o gate do FPS (report do Enio:
/// *"queda importante de FPS"*), no padrão do ADR-0120 (contar as vezes que o caminho CARO dispara).
///
/// A raiz do FPS era cozer N `offset_path` (sweep booleano, 0,334 ms/anel) por frame de arrasto.
/// O offset DIRETO (`offset_ring`, ~0,0005 ms/anel, 668×) cobre o caso comum — contorno único,
/// Outer — sem tocar o `linesweeper`. Este gate dirige um arrasto de hexágono (N=8, 240 passos de
/// `d`) e exige que a booleana seja chamada **ZERO** vezes.
///
/// ⚠️ É o gate que DISTINGUE o mecanismo, e o irmão anti-blink não distinguia: aquele mede a
/// contagem de anéis, que é estável tanto com o offset direto quanto com a booleana+`last_good`.
/// Mutação: trocar `offset_ring().unwrap_or_else(offset_path)` por `offset_path` direto ⇒ a
/// booleana é chamada 240×8 vezes ⇒ RED. É também por que o custo cai: a booleana não roda.
#[test]
fn dragging_a_plain_contour_never_calls_the_booleana() {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::cook(
        ph2d_vec_scene::ShapeKind::Polygon,
        [1.0, -1.2],
        [3.4, 1.2],
        &[6.0],
    ));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Hex"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    if let Some(p) = scene.path_mut(id) {
        p.fill = Some(Paint::solid(Rgba8::new(0, 0, 0, 255)));
    }
    arm(
        &mut sim,
        &map,
        id,
        VecContour {
            steps: 8,
            d: 0.0,
            accel: 1.0,
            join: 1,
            ..VecContour::default()
        },
    );
    let mut live = ContourLive::default();
    let xf = VecXforms::new();
    let before = ph2d_vec_boolean::__sweep_calls();
    for i in 1..=240 {
        let d = f64::from(i) * 0.005;
        if let Some(mut c) = sim
            .world_mut()
            .get_mut::<VecContour>(ph2d_ecs::Entity::from_bits(map[&id]))
        {
            c.d = d;
        }
        live.recook(&scene, &sim, &map, &xf);
    }
    let calls = ph2d_vec_boolean::__sweep_calls() - before;
    assert_eq!(
        calls, 0,
        "o arrasto de um contour de contorno unico chamou o sweep booleano {calls} vezes — o \
         offset direto devia ter coberto TODOS os aneis (a raiz do FPS e do pisca-pisca)"
    );
}
