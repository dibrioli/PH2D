//! Os gates das ÂNCORAS vivas (plano UI/UX W3).
//!
//! A aritmética de uma regra já está provada no `ph2d_ecs::vec_anchors`. O que só se pode afirmar
//! aqui é a COSTURA: que a régua é a caixa LOCAL da moldura, que a razão local→mundo chega ao
//! deslocamento, que a moldura intocada deixa o mapa **byte-intocado**, que o FLUXO vence, que a
//! pose publicada leva a geometria autorada ao que se DESENHA — e que a régua capturada pela porta
//! de autoria é a MESMA que o passe amostra.

use super::*;
// ⚠️ Privados do AVÔ (`layout_live`): este módulo é descendente dele, então alcança-os pelo
// caminho — e é isso que faz o gate medir o que o produto de facto desenha, e não uma cópia.
use crate::layout_live::{bbox_of, world_of};
use ph2d_ecs::{LayoutDir, Transform, VecFrame, VecLayout};
use ph2d_vec_scene::rectangle;

/// A moldura AGORA mede `w × 40` (canto mínimo na origem); a régua de cada filho dirá 100×40, ou
/// seja *"a moldura cresceu `w − 100` para a direita"*. Cada filho é um quadrado de 10×10 no canto
/// inferior-esquerdo. Devolve `(sim, scene, map, moldura, filhos)`.
fn frame_now(w: f64, n: usize) -> (SimWorld, VecScene, VecEntityMap, Entity, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let frame_id = scene.push_path(rectangle([0.0, 0.0], [w, 40.0]));
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0])))
        .collect();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let frame = Entity::from_bits(map[&frame_id]);
    sim.world_mut()
        .entity_mut(frame)
        .insert(VecFrame { clip: false });
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(frame));
    }
    (sim, scene, map, frame, kids)
}

/// A régua que toda fixture usa: a moldura tinha 100×40 quando a regra foi armada.
const BASE: [f64; 4] = [0.0, 0.0, 100.0, 40.0];

fn anchor(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, min: [f64; 2], max: [f64; 2]) {
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecAnchors {
        min,
        max,
        base: BASE,
    });
}

fn run(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    live: &mut LiveGeometry,
) -> LayoutLive {
    let mut ll = LayoutLive::default();
    ll.recook(scene, sim, map, &VecXforms::default(), live);
    ll
}

fn drawn(live: &LiveGeometry, scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    let items = world_of(scene, &VecXforms::default(), live, id);
    bbox_of(&items).expect("o caminho desenha alguma coisa")
}

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

/// **O CONTROLE, e ele vem primeiro: moldura do tamanho da régua não move nada, e nem paga a
/// cópia.**
///
/// Sem esta metade, um gate que só medisse a moldura CRESCIDA ficaria verde sobre um passe que
/// empurra a arte toda no primeiro frame de qualquer documento.
#[test]
fn an_unresized_frame_leaves_the_map_byte_untouched() {
    let (mut sim, scene, map, _frame, kids) = frame_now(100.0, 1);
    anchor(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert_eq!(ll.anchored(), 0, "nada a colocar");
    assert!(live.is_empty(), "o mapa nao foi sequer tocado");
}

/// **Quem segue a aresta direita anda o crescimento inteiro; quem não tem regra fica.**
///
/// ⚠️ O segundo filho é o **CONTROLE dentro da mesma cena**: uma diferença que apareça nos dois
/// não é da âncora.
#[test]
fn a_right_anchored_child_follows_the_edge_and_a_plain_one_does_not() {
    let (mut sim, scene, map, _frame, kids) = frame_now(160.0, 2);
    anchor(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let (lo, hi) = drawn(&live, &scene, kids[0]);
    assert!(approx(lo[0], 60.0) && approx(hi[0], 70.0), "{lo:?} {hi:?}");
    let (clo, chi) = drawn(&live, &scene, kids[1]);
    assert!(
        approx(clo[0], 0.0) && approx(chi[0], 10.0),
        "o controle nao se move"
    );
}

/// O centrado anda METADE do crescimento.
#[test]
fn a_centred_child_moves_half_the_growth() {
    let (mut sim, scene, map, _frame, kids) = frame_now(160.0, 1);
    anchor(&mut sim, &map, kids[0], [0.5, 0.5], [0.5, 0.5]);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let (lo, _) = drawn(&live, &scene, kids[0]);
    assert!(approx(lo[0], 30.0), "{lo:?}");
}

/// **Esticar é o par de pontas a discordar** — a esquerda fica, a direita leva os 60.
#[test]
fn a_stretched_child_keeps_its_left_edge_and_grows() {
    let (mut sim, scene, map, _frame, kids) = frame_now(160.0, 1);
    anchor(&mut sim, &map, kids[0], [0.0, 0.0], [1.0, 1.0]);
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    let (lo, hi) = drawn(&live, &scene, kids[0]);
    assert!(approx(lo[0], 0.0), "a esquerda fica: {lo:?}");
    assert!(approx(hi[0], 70.0), "a direita leva o crescimento: {hi:?}");
}

/// **O deslocamento é LOCAL, e a razão local→mundo leva-o à tela.**
///
/// ⚠️ Sem esta fixture a razão fica **por provar**: numa moldura sem afim nenhum ela vale `1`, e
/// então *"multiplicar pela razão"* e *"não multiplicar"* dão o mesmo número em todos os outros
/// gates. Aqui a moldura (e o filho, que herda o afim do pai) é vista a 2×, e a régua diz que ela
/// cresceu 60 em unidades LOCAIS — logo o filho tem de andar **120** na tela.
#[test]
fn the_local_delta_reaches_the_screen_through_the_frames_own_ratio() {
    let (mut sim, scene, map, frame_e, kids) = frame_now(160.0, 1);
    anchor(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let frame_id = *scene
        .paths()
        .iter()
        .map(|p| p.id)
        .collect::<Vec<_>>()
        .first()
        .expect("a moldura e' o primeiro caminho");
    let _ = frame_e;
    let twice = ph2d_vec_scene::Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    let mut xf = VecXforms::default();
    xf.insert(frame_id, twice);
    xf.insert(kids[0], twice);
    let mut live = LiveGeometry::new();
    let mut ll = LayoutLive::default();
    ll.recook(&scene, &sim, &map, &xf, &mut live);
    let items = world_of(&scene, &xf, &live, kids[0]);
    let (lo, _) = bbox_of(&items).expect("caixa");
    assert!(
        approx(lo[0], 120.0),
        "o filho pousou em {lo:?} — 60 locais a 2x sao 120 na tela"
    );
}

/// **O FLUXO vence: um filho de moldura que empilha NÃO é ancorado.**
///
/// É a lei do plano — *ou está num fluxo ou está ancorado, nunca os dois* — e ela é honrada aqui
/// (o passe recusa) e no painel (a seção não é oferecida) pela MESMA porta.
#[test]
fn the_flow_wins_over_the_anchor() {
    let (mut sim, scene, map, frame, kids) = frame_now(160.0, 1);
    anchor(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    sim.world_mut().entity_mut(frame).insert(VecLayout {
        dir: LayoutDir::Row,
        ..VecLayout::default()
    });
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert_eq!(ll.anchored(), 0, "a ancora nao tem voto num fluxo");
}

/// **O passe NÃO escreve `Transform`** (ADR-0153) — o undo é por diff do mundo.
#[test]
fn the_anchor_pass_never_writes_the_authored_transform() {
    let (mut sim, scene, map, _frame, kids) = frame_now(160.0, 1);
    anchor(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let e = Entity::from_bits(map[&kids[0]]);
    let before = sim.world().get::<Transform>(e).copied();
    let mut live = LiveGeometry::new();
    run(&sim, &scene, &map, &mut live);
    assert_eq!(sim.world().get::<Transform>(e).copied(), before);
}

/// **A pose publicada leva a geometria AUTORADA ao que se DESENHA — mesmo quando dois braços do
/// passe mexem no mesmo caminho.**
///
/// ⚠️ Este é o gate da COMPOSIÇÃO. Uma moldura ancorada pode ela própria ser um item de fluxo: o
/// fluxo a coloca (e à sub-árvore dela) e a âncora move depois o filho ancorado. A geometria
/// compõe sozinha; a TABELA não compunha, e um `insert` cru guardaria só o último afim — quem
/// **aponta** procuraria a forma a meio caminho de onde ela está.
#[test]
fn the_published_pose_takes_the_authored_geometry_to_what_is_drawn() {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    // A moldura de FORA, que empilha; a de DENTRO, que ancora; e o filho ancorado.
    let outer_id = scene.push_path(rectangle([0.0, 0.0], [400.0, 200.0]));
    let inner_id = scene.push_path(rectangle([0.0, 0.0], [160.0, 40.0]));
    let kid_id = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let outer = Entity::from_bits(map[&outer_id]);
    let inner = Entity::from_bits(map[&inner_id]);
    let kid = Entity::from_bits(map[&kid_id]);
    sim.world_mut()
        .entity_mut(outer)
        .insert(VecFrame { clip: false })
        .insert(VecLayout {
            dir: LayoutDir::Row,
            pad: [8.0, 8.0, 8.0, 8.0],
            ..VecLayout::default()
        });
    sim.world_mut()
        .entity_mut(inner)
        .insert(VecFrame { clip: false })
        .insert(ph2d_ecs::ChildOf(outer));
    sim.world_mut()
        .entity_mut(kid)
        .insert(ph2d_ecs::ChildOf(inner))
        .insert(VecAnchors {
            min: [1.0, 1.0],
            max: [1.0, 1.0],
            base: BASE,
        });
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    let poses = ll.poses();
    let (_, pose) = poses
        .iter()
        .find(|(id, _)| *id == kid_id)
        .expect("o filho ancorado tem pose publicada");
    // A geometria AUTORADA, levada pela pose publicada, tem de dar exactamente o desenho.
    let mut authored = scene
        .paths()
        .iter()
        .find(|p| p.id == kid_id)
        .expect("o caminho")
        .cooked()
        .into_owned();
    ph2d_vec_scene::bake_xform(&mut authored, pose);
    let (plo, phi) = bbox_of(&[authored]).expect("caixa");
    let (dlo, dhi) = drawn(&live, &scene, kid_id);
    assert!(
        approx(plo[0], dlo[0]) && approx(plo[1], dlo[1]) && approx(phi[0], dhi[0]),
        "pose {plo:?}..{phi:?} contra desenho {dlo:?}..{dhi:?}"
    );
}

/// **A âncora não realimenta a MEDIÇÃO do fluxo — e é por isso que ela corre DEPOIS.**
///
/// ⚠️ Uma moldura que não flui é medida pelo fluxo do pai pela **sub-árvore inteira**, e a âncora
/// coloca filhos *lendo a caixa dessa moldura*. Se ela corresse primeiro, a caixa que o fluxo mede
/// passaria a depender de onde a âncora pôs os filhos, que por sua vez depende da caixa — um laço.
/// O oráculo é a moldura interna **colocada exactamente como se o filho não tivesse regra**.
///
/// ⚠️ E a fixture tem de conter o fenómeno: o filho ancorado sai PARA FORA da moldura interna
/// (ele está na aresta direita e a regra manda-o seguir a aresta que cresceu 60). Com ele dentro,
/// a caixa da sub-árvore não muda e as duas ordens dão o MESMO número — foi assim que a primeira
/// versão deste gate sobreviveu à mutação da ordem.
#[test]
fn the_anchor_does_not_feed_back_into_the_flows_measurement() {
    /// Monta a cena e devolve a caixa de mundo da moldura INTERNA depois do passe.
    fn inner_box(anchored: bool) -> ([f64; 2], [f64; 2]) {
        let mut sim = SimWorld::default();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let outer_id = scene.push_path(rectangle([0.0, 0.0], [400.0, 100.0]));
        let inner_id = scene.push_path(rectangle([0.0, 0.0], [160.0, 40.0]));
        // O filho encostado na aresta DIREITA da moldura interna.
        let kid_id = scene.push_path(rectangle([150.0, 0.0], [160.0, 10.0]));
        crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
        let outer = Entity::from_bits(map[&outer_id]);
        let inner = Entity::from_bits(map[&inner_id]);
        let kid = Entity::from_bits(map[&kid_id]);
        sim.world_mut()
            .entity_mut(outer)
            .insert(VecFrame { clip: false })
            .insert(VecLayout {
                dir: LayoutDir::Row,
                ..VecLayout::default()
            });
        sim.world_mut()
            .entity_mut(inner)
            .insert(VecFrame { clip: false })
            .insert(ph2d_ecs::ChildOf(outer))
            // ⚠️ O `grow` é o que torna a MEDIÇÃO observável: sem ele o alvo do fluxo é a caixa
            // medida, o afim é a identidade, e as duas ordens desenham a moldura no mesmo sítio.
            .insert(ph2d_ecs::VecLayoutItem {
                grow: 1.0,
                ..ph2d_ecs::VecLayoutItem::default()
            });
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(inner));
        if anchored {
            sim.world_mut().entity_mut(kid).insert(VecAnchors {
                min: [1.0, 0.0],
                max: [1.0, 0.0],
                base: BASE,
            });
        }
        let mut live = LiveGeometry::new();
        run(&sim, &scene, &map, &mut live);
        drawn(&live, &scene, inner_id)
    }
    let (clo, chi) = inner_box(false);
    let (rlo, rhi) = inner_box(true);
    assert!(
        approx(clo[0], rlo[0]) && approx(chi[0], rhi[0]),
        "a moldura interna foi colocada em {rlo:?}..{rhi:?} com a regra e em {clo:?}..{chi:?} sem \
         ela — a ancora esta' a realimentar a medicao do fluxo"
    );
}

/// **A régua capturada pela porta de AUTORIA é a MESMA que o passe amostra.**
///
/// ⚠️ Duas medições da caixa da moldura — uma que coze a forma viva e outra que não, por exemplo —
/// poriam o filho meio delta ao lado no instante em que a regra é armada, e nada na tela diria
/// porquê. Este gate arma pela porta REAL e exige que, sem redimensionar nada, o passe não mova
/// coisa nenhuma ([[feedback_derived_coordinate_seed_must_match_sample]]).
#[test]
fn arming_a_rule_captures_the_ruler_the_pass_samples() {
    let (mut sim, scene, map, _frame, kids) = frame_now(137.5, 1);
    let sel = vec![kids[0]];
    assert!(crate::vec_anchor_edit::apply_anchor_edit(
        &mut sim,
        &scene,
        &map,
        &sel,
        crate::vec_anchor_edit::AnchorEdit::H([1.0, 1.0]),
    ));
    let mut live = LiveGeometry::new();
    let ll = run(&sim, &scene, &map, &mut live);
    assert_eq!(ll.anchored(), 0, "armar nao pode mover a arte no clique");
    assert!(live.is_empty());
}

/// **`Constrain Left` funciona: armar pelo painel, arrastar a borda ESQUERDA, o filho vai junto.**
///
/// ⚠️ É o gate que faltava, e a falta tem a mesma forma da do redimensionamento: os irmãos acima
/// **injetam** o `VecAnchors` à mão e provam o PASSE; os do `vec_anchor_edit` provam a PORTA de
/// autoria. Nenhum dos dois atravessa a corrente inteira — *clicar o chip → arrastar a alça →
/// o passe honrar* —, e era exactamente aí que o report do Enio vivia (*"Constrain Left e Bottom
/// pararam de funcionar"*): o clique em `Left` apagava a própria regra que pedia, porque `Left`
/// **é** a aresta mínima e o neutro DESTACAVA.
///
/// O oráculo é o que o artista vê: a caixa DESENHADA do filho.
#[test]
fn constrain_left_survives_the_click_and_the_handle() {
    let (mut sim, mut scene, map, frame, kids) = frame_now(100.0, 1);
    let frame_id = sim
        .world()
        .get::<ph2d_ecs::VecPathRef>(frame)
        .expect("a moldura")
        .0;
    // 1. O artista escolhe `Left` (eixo X). ⚠️ Só o PRIMEIRO clique muda o mundo: a regra nasce
    //    colada nas duas arestas mínimas, então pedir `Bottom` a seguir é um no-op honesto — e
    //    exigir `true` dele foi a 1ª versão deste gate a reprovar produto correto.
    assert!(
        crate::vec_anchor_edit::apply_anchor_edit(
            &mut sim,
            &scene,
            &map,
            &[kids[0]],
            crate::vec_anchor_edit::AnchorEdit::H([0.0, 0.0]),
        ),
        "o clique em Left tem de armar a regra"
    );
    let armed = sim
        .world()
        .get::<VecAnchors>(Entity::from_bits(map[&kids[0]]))
        .copied()
        .expect("a regra ficou gravada");
    assert_eq!([armed.min, armed.max], [[0.0, 0.0], [0.0, 0.0]]);
    assert_eq!(armed.base, [0.0, 0.0, 100.0, 40.0], "capturou a regua");
    // 2. A alça arrasta a borda ESQUERDA: o pivô é a borda direita (x = 100), e a moldura
    //    cresce 50 para a esquerda.
    let xf = VecXforms::default();
    let st = crate::vec_frame_resize::begin(&scene, &xf, frame.to_bits(), frame_id, None)
        .expect("armou o redimensionamento");
    crate::vec_frame_resize::apply(&mut sim, &mut scene, &st, [100.0, 20.0], 1.5, 1.0);
    let (flo, _fhi) = scene.path_curve_bbox(frame_id).expect("a moldura");
    assert!(
        approx(flo[0], -50.0),
        "a fixture nao moveu a aresta minima: {flo:?}"
    );
    // 3. O passe honra: o filho anda os mesmos 50 para a esquerda.
    let mut live = LiveGeometry::new();
    let _ = run(&sim, &scene, &map, &mut live);
    let (lo, _hi) = drawn(&live, &scene, kids[0]);
    assert!(
        approx(lo[0], -50.0),
        "o filho nao seguiu a aresta esquerda: x={} (devia ser -50)",
        lo[0]
    );
}
