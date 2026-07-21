//! **A forma que se VÊ é a que se PEGA** — os gates do pick sobre o Offset vivo
//! (`ph2d_ecs::VecOffset`), módulo irmão de [`super`] pelo teto de LOC.
//!
//! O ADR-0121 separou a curva AUTORADA da COZIDA e o offset vivo (2026-07-21) levou a costura ao
//! nível do OBJETO: o documento guarda o `VecPath` que o artista desenhou e a tela mostra a
//! derivada. Quem desenha já pergunta ao `offset_live` — o hit-test passou a perguntar à MESMA
//! porta, senão a forma fica **pintada num lugar e clicável noutro**, sem erro e sem aviso: o
//! artista clica no meio do que vê e nada acontece.
//!
//! ⚠️ **Os dois sentidos são gates separados de propósito.** Um remendo que apenas SOMASSE a
//! derivada à fonte (união) passa no gate de crescimento e falha no de encolhimento — e o
//! encolhimento é onde mora o modo de falha oposto, *pegar a forma onde a tinta já não está*.
//!
//! Mutação canônica: fazer o `contains_path`/`world_bbox` ignorarem o `live` (o produto de antes
//! de 2026-07-21) tem de SANGRAR os quatro primeiros gates e deixar o de regressão VERDE.

use super::*;
use ph2d_ecs::Transform;
use ph2d_vec_scene::{OffsetSide, VecPath, VecPathId, VecVertex, rectangle};

/// Quadrado de lado 2 centrado na origem, com uma entidade e um id.
fn scene_with_square() -> (SimWorld, VecScene, VecEntityMap, Entity, VecPathId) {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let mut sq = VecPath {
        verts: [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    };
    sq.fill = rectangle([0.0, 0.0], [1.0, 1.0]).fill; // preenchida: tem interior a clicar
    let id = scene.push_path(sq);
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    (sim, scene, map, e, id)
}

/// O offset vivo de `d` já COZIDO, na forma em que o renderer o consome: geometria de MUNDO,
/// endereçada pelo id do caminho. É a mesma chamada do `offset_live::recook` — a fixture não
/// pode inventar a derivada, senão o gate testaria a si mesmo.
fn live_offset(scene: &VecScene, id: VecPathId, d: f64) -> LiveGeometry {
    let src = scene.paths().iter().find(|p| p.id == id).unwrap().clone();
    let out = ph2d_vec_boolean::offset_path(
        &src.cooked(),
        d,
        ph2d_vec_scene::LineJoin::Miter,
        OffsetSide::Both,
    );
    let mut live = LiveGeometry::new();
    live.insert(id, out);
    live
}

/// **O que CRESCEU é clicável onde está desenhado.** Com `d = +1` o quadrado de lado 2 desenha
/// lado 4, então `(1.6, 0)` está a 0,6 unidade FORA da fonte e bem dentro da tinta. Antes, ali
/// não havia nada para o mouse: o pick apalpava a curva autorada.
#[test]
fn the_grown_offset_is_clickable_where_the_paint_is() {
    let (sim, scene, map, e, id) = scene_with_square();
    let live = live_offset(&scene, id, 1.0);
    let vs = VecViewState::default();
    // Pré-condição: a derivada existe (sem isto o gate mediria a fonte e seria verde à toa).
    assert!(
        !live[&id].is_empty(),
        "o offset tem de ter produzido geometria"
    );
    assert_eq!(
        pick_at_world(&sim, &scene, &live, &vs, &map, [1.6, 0.0], 0.0),
        Some(e.to_bits()),
        "clicar DENTRO da forma crescida tem de pegá-la"
    );
    // E o que está fora da tinta segue livre — crescer não é pegar o plano inteiro.
    assert_eq!(
        pick_at_world(&sim, &scene, &live, &vs, &map, [2.4, 0.0], 0.0),
        None,
        "fora da forma desenhada nada é pego"
    );
}

/// **O que ENCOLHEU deixou de ser clicável onde a tinta saiu.** Com `d = −0.6` o quadrado de
/// lado 2 desenha lado 0,8: `(0.8, 0)` está DENTRO da curva autorada e FORA do que se vê.
///
/// É a metade que um remendo de união não tem: somar a derivada à fonte deixa este ponto pego.
#[test]
fn the_inset_offset_is_not_clickable_where_the_paint_left() {
    let (sim, scene, map, e, id) = scene_with_square();
    let live = live_offset(&scene, id, -0.6);
    let vs = VecViewState::default();
    assert!(!live[&id].is_empty(), "a forma encolhida ainda existe");
    assert_eq!(
        pick_at_world(&sim, &scene, &live, &vs, &map, [0.8, 0.0], 0.0),
        None,
        "o vão entre a curva autorada e a tinta encolhida não pertence a ninguém"
    );
    // O miolo, que continua pintado, segue pego — o gate acima não pode ser satisfeito
    // desligando o pick da forma inteira.
    assert_eq!(
        pick_at_world(&sim, &scene, &live, &vs, &map, [0.0, 0.0], 0.0),
        Some(e.to_bits()),
        "o que sobrou de tinta continua clicável"
    );
}

/// **Aniquilada não se pega.** `d` grande e negativo come a forma inteira; o `recook` guarda a
/// entrada VAZIA justamente para dizer *"nada desenhado"* (ausente significaria "use a fonte").
/// O centro, que é o lugar mais óbvio para clicar, não devolve nada.
#[test]
fn an_annihilated_shape_is_clickable_nowhere() {
    let (sim, scene, map, _e, id) = scene_with_square();
    let live = live_offset(&scene, id, -5.0);
    assert!(
        live[&id].is_empty(),
        "pré-condição: o offset tem de ter comido a forma"
    );
    let vs = VecViewState::default();
    for p in [[0.0, 0.0], [0.9, 0.9], [-0.5, 0.2]] {
        assert_eq!(
            pick_at_world(&sim, &scene, &live, &vs, &map, [p[0], p[1]], 0.25),
            None,
            "nada desenhado em {p:?}, nada a pegar"
        );
    }
}

/// **O marquee mede a caixa DESENHADA.** Um retângulo que só encosta na banda crescida pega a
/// forma; e, encolhida, um retângulo sobre o vão que a tinta deixou não pega nada.
#[test]
fn the_marquee_measures_the_drawn_box_not_the_authored_one() {
    let (sim, scene, map, e, id) = scene_with_square();
    let vs = VecViewState::default();
    let grown = live_offset(&scene, id, 1.0);
    assert_eq!(
        pick_in_world_rect(&sim, &scene, &grown, &vs, &map, [1.4, -0.2], [1.8, 0.2]),
        vec![e.to_bits()],
        "a janela cai na banda crescida — a forma está ali"
    );
    let inset = live_offset(&scene, id, -0.6);
    assert!(
        pick_in_world_rect(&sim, &scene, &inset, &vs, &map, [0.6, -0.2], [0.9, 0.2]).is_empty(),
        "a janela cai onde a tinta SAIU — a caixa da fonte a pegaria"
    );
}

/// **REGRESSÃO: sem offset vivo, o pick é o de sempre.** A porta nova é um desvio, não uma
/// reescrita — um caminho sem derivada tem de responder exatamente como respondia (interior sim,
/// fora não), e é isto que mantém honesta a mutação canônica do topo do arquivo.
#[test]
fn without_a_live_offset_the_pick_is_the_source_it_always_was() {
    let (sim, scene, map, e, _id) = scene_with_square();
    let vs = VecViewState::default();
    let none = LiveGeometry::new();
    assert_eq!(
        pick_at_world(&sim, &scene, &none, &vs, &map, [0.5, 0.5], 0.0),
        Some(e.to_bits())
    );
    assert_eq!(
        pick_at_world(&sim, &scene, &none, &vs, &map, [1.6, 0.0], 0.0),
        None
    );
    assert_eq!(
        pick_in_world_rect(&sim, &scene, &none, &vs, &map, [-0.2, -0.2], [0.2, 0.2]),
        vec![e.to_bits()]
    );
}
