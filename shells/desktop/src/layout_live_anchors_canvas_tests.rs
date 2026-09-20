//! Os gates das ÂNCORAS de um **canvas de HUD** (TOP-20 #20).
//!
//! ⚠️ A aritmética da regra já está provada no `ph2d_ecs::vec_anchors`, e a da caixa efectiva no
//! `ph2d_hud`. O que só se pode afirmar AQUI é a COSTURA: que a moldura de um filho de canvas é a
//! caixa **efectiva** (e não a de referência), que sem banda o mapa fica **byte-intocado**, que sem
//! câmera nada ancora.
//!
//! ⚠️ **A SELECÇÃO não se prova aqui** — *quem* um canvas ancora e *contra que rectângulo* vive em
//! [`ph2d_app_components::hud_anchors`], com gates próprios que não precisam de cena nem de device.
//! Estes medem o que se DESENHA, que é a metade que só a shell tem.

use super::*;
use crate::layout_live::{bbox_of, world_of};
use ph2d_app_components::hud_bridge::View;
use ph2d_ecs::{Fit, UiCanvas};
use ph2d_vec_scene::rectangle;

/// A referência de fábrica da casa, centrada na origem — que é o idioma do canvas.
const REF: [f64; 4] = [-16.0, -9.0, 16.0, 9.0];

/// Um canvas com `n` filhos quadrados de `10×10` no canto mínimo. O canvas **não é um caminho**:
/// ele é uma raiz com `UiCanvas`, que é o que a cena do HUD monta.
fn canvas_com_filhos(n: usize) -> (SimWorld, VecScene, VecEntityMap, Entity, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let kids: Vec<VecPathId> = (0..n)
        .map(|_| scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0])))
        .collect();
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let root = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                // ⚠️ **`Expand` e não `Keep`:** desde a correcção do oráculo (bloco L4) só o
                // `Expand` cresce a caixa — com `Keep` estes gates mediriam um delta de `0,0` e
                // ficariam verdes a afirmar nada.
                fit: Fit::Expand,
            },
        ))
        .id();
    for k in &kids {
        let kid = Entity::from_bits(map[k]);
        sim.world_mut()
            .entity_mut(kid)
            .insert(ph2d_ecs::ChildOf(root));
    }
    (sim, scene, map, root, kids)
}

fn prende(sim: &mut SimWorld, map: &VecEntityMap, id: VecPathId, min: [f64; 2], max: [f64; 2]) {
    let e = Entity::from_bits(map[&id]);
    sim.world_mut().entity_mut(e).insert(VecAnchors {
        min,
        max,
        base: REF,
    });
}

fn corre(
    sim: &SimWorld,
    scene: &VecScene,
    map: &VecEntityMap,
    live: &mut LiveGeometry,
    vista: Option<View>,
) -> LayoutLive {
    let mut ll = LayoutLive {
        vista,
        ..LayoutLive::default()
    };
    ll.recook(
        scene,
        sim,
        map,
        &VecXforms::default(),
        live,
        crate::vec_bindings::TokenCtx::factory(),
    );
    ll
}

fn vista(hw: f32, hh: f32) -> View {
    View {
        center: [0.0, 0.0],
        half: [hw, hh],
    }
}

fn desenhado(live: &LiveGeometry, scene: &VecScene, id: VecPathId) -> ([f64; 2], [f64; 2]) {
    let items = world_of(scene, &VecXforms::default(), live, id);
    bbox_of(&items).expect("o caminho desenha alguma coisa")
}

/// ⭐⭐⭐ **A frase inteira da wave: um filho preso à direita CHEGA à borda real.**
///
/// A conta, fechada à mão: referência `32×18` numa vista de `42×18` ⇒ `Keep` dá escala `1,0` e uma
/// banda de `(42 − 32)/2 = 5`. A caixa efectiva é `[−21, 21]`, logo `moved = −5` e `grew = 10` ⇒ um
/// filho com `min = max = 1` anda **`+5`**.
///
/// **Mutação que deve sangrar:** usar a caixa de REFERÊNCIA no lugar da efectiva.
#[test]
fn um_filho_preso_a_direita_anda_a_banda() {
    let (mut sim, scene, map, _root, kids) = canvas_com_filhos(1);
    prende(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let mut live = LiveGeometry::default();
    let ll = corre(&sim, &scene, &map, &mut live, Some(vista(21.0, 9.0)));

    let (lo, hi) = desenhado(&live, &scene, kids[0]);
    assert!(
        (lo[0] - 5.0).abs() < 0.01 && (hi[0] - 15.0).abs() < 0.01,
        "o filho desenhou em x = [{}, {}] e a banda pedia [5, 15]",
        lo[0],
        hi[0]
    );
    // ⚠️ E em `y` NÃO se mexeu: naquela vista a banda vertical é zero, e uma implementação que
    // crescesse os dois eixos passaria a metade de cima deste gate.
    assert!(
        (lo[1] - 0.0).abs() < 0.01,
        "o filho andou em y ({}) numa vista sem banda vertical",
        lo[1]
    );
    assert_eq!(ll.anchored, 1, "o canvas nao ancorou ninguem");
}

/// ⭐⭐ **O NEUTRO é byte-intocado** — no aspecto da própria caixa não há banda, o afim é a
/// identidade e o passe **nem paga a cópia da geometria**.
///
/// ⚠️ É esta metade que torna a wave barata: sem ela, toda cena de HUD já autorada mudaria de
/// imagem no dia em que isto shipasse.
#[test]
fn sem_banda_o_mapa_fica_byte_intocado() {
    let (mut sim, scene, map, _root, kids) = canvas_com_filhos(1);
    prende(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let mut live = LiveGeometry::default();
    let ll = corre(&sim, &scene, &map, &mut live, Some(vista(16.0, 9.0)));
    assert!(
        live.is_empty(),
        "o passe escreveu no mapa sem haver banda nenhuma"
    );
    assert_eq!(ll.anchored, 0, "contou uma ancoragem que nao aconteceu");
}

/// ⭐⭐ **Um filho ESTICÁVEL cresce com a banda** — `min = 0`, `max = 1`.
///
/// ⚠️ A mesma lei do pino, e é isso que se está a afirmar: o `anchor_kid` não foi duplicado, logo
/// o esticão sai de graça. Com `moved = −5` e `grew = 10`: a aresta mínima anda `−5` e a máxima
/// `+5` ⇒ o quadrado de `[0, 10]` passa a `[−5, 15]`.
#[test]
fn um_filho_esticavel_cresce_com_a_banda() {
    let (mut sim, scene, map, _root, kids) = canvas_com_filhos(1);
    prende(&mut sim, &map, kids[0], [0.0, 0.0], [1.0, 1.0]);
    let mut live = LiveGeometry::default();
    corre(&sim, &scene, &map, &mut live, Some(vista(21.0, 9.0)));
    let (lo, hi) = desenhado(&live, &scene, kids[0]);
    assert!(
        (lo[0] + 5.0).abs() < 0.01 && (hi[0] - 15.0).abs() < 0.01,
        "o esticao deu x = [{}, {}] e a conta pede [−5, 15]",
        lo[0],
        hi[0]
    );
}

/// ⭐⭐ **E com a raiz a ESCALAR, o deslocamento atravessa a escala.**
///
/// ⚠️⚠️ **Este gate existe porque uma mutação SOBREVIVEU:** a fixtura das irmãs está no ponto
/// NEUTRO da escala (`Keep` a `42×18` dá `s = 1,0`), logo trocar a escala por `1` não mudava nada
/// e a régua não media o que dizia medir. *Um corpus no ponto NEUTRO de um knob não testa esse
/// knob* — a lei que esta casa já pagou no L-System, no apagador e no `Accumulate`.
///
/// A conta: vista de `84×36` ⇒ `fx = 2,625`, `fy = 2,0`, `Keep` dá `s = 2,0`; a banda de mundo é
/// `(84 − 64)/2 = 10` e em local `5`. O delta local de `5` atravessa a escala ⇒ o quadrado de
/// `[0, 10]` vai para **`[10, 20]`**.
#[test]
fn com_a_raiz_a_escalar_o_deslocamento_atravessa_a_escala() {
    let (mut sim, scene, map, _root, kids) = canvas_com_filhos(1);
    prende(&mut sim, &map, kids[0], [1.0, 1.0], [1.0, 1.0]);
    let mut live = LiveGeometry::default();
    corre(&sim, &scene, &map, &mut live, Some(vista(42.0, 18.0)));
    let (lo, hi) = desenhado(&live, &scene, kids[0]);
    assert!(
        (lo[0] - 10.0).abs() < 0.01 && (hi[0] - 20.0).abs() < 0.01,
        "com escala 2 o filho desenhou em [{}, {}] e a conta pede [10, 20]",
        lo[0],
        hi[0]
    );
}
