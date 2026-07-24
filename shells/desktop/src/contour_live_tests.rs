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
/// # O que ele mede, e por que a contagem é o oráculo certo
///
/// O que o artista vê como piscar é um anel a **desaparecer e voltar**: o `linesweeper` não
/// consegue responder em algumas distâncias, o `offset_path` devolve vazio, e o anel some do
/// quadro por um frame. Medido no motor cru com o hexágono do smoke: **70 trocas de contagem**
/// numa varredura de 240 passos de `d`, que é o que um arrasto de slider faz.
///
/// O oráculo é a CONTAGEM de caminhos que o `dispatch` desenharia, e não a geometria: a geometria
/// muda a cada passo *de propósito* (é o efeito a seguir o slider). O que **não** pode acontecer é
/// o número de anéis cair e subir.
///
/// ⚠️ Mutação: tirar o `last_good` do `ensure` (o anel vazio passa direto) ⇒ a contagem oscila ⇒
/// RED. O fixture usa um HEXÁGONO, e é obrigatório: num quadrado o sweep responde em toda
/// distância — a fixture tem de CONTER o fenômeno.
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
