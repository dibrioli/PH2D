//! Os gates do gizmo de canvas dos deformadores de quadrilátero.
//!
//! ⚠️ A lei central é **SEED = SAMPLE**: onde a alça é desenhada e o que o arrasto
//! escreve têm de ser inversos exactos. Um round-trip que não fecha é o gizmo a
//! escorregar do cursor, e é o defeito que esta casa já pagou em três gizmos.

use super::*;

fn unit_box() -> WarpBox {
    WarpBox {
        lo: [0.0, 0.0],
        hi: [4.0, 2.0],
    }
}

/// Uma porta de param que devolve o que a tabela disser, e `0` para o resto.
fn params(pairs: &'static [(&'static str, f32)]) -> impl Fn(&str) -> f32 {
    move |n| pairs.iter().find(|(k, _)| *k == n).map_or(0.0, |(_, v)| *v)
}

/// **OS DOIS NÓS TÊM GIZMO, E MAIS NENHUM.**
///
/// ⚠️ A metade negativa é a que interessa: a ausência é o default seguro, e um nó que
/// não está na tabela não pode ter hit-region nenhuma no canvas.
#[test]
fn exactly_the_two_quad_deformers_have_a_gizmo() {
    let four = spec_for(NodeTypeId::of("motion.four_point_warp")).expect("o Corner Pin tem");
    assert!(!four.has_tangents, "o Corner Pin não tem tangentes");
    let bez = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("o Bezier tem");
    assert!(bez.has_tangents, "o Bezier tem as oito");
    for other in [
        "motion.transform",
        "motion.grid",
        "motion.spline_wrap",
        "field.box",
    ] {
        assert!(
            spec_for(NodeTypeId::of(other)).is_none(),
            "`{other}` não pode ter alças"
        );
    }
}

/// **NO NEUTRO, AS QUATRO ALÇAS SÃO OS CANTOS DA CAIXA.**
#[test]
fn at_rest_the_corner_handles_sit_on_the_box_corners() {
    let spec = spec_for(NodeTypeId::of("motion.four_point_warp")).expect("spec");
    let b = unit_box();
    let (hs, n) = handles(spec, b, 1.0, &params(&[]));
    assert_eq!(n, 4, "o Corner Pin oferece quatro alças");
    // TL, TR, BR, BL sobre a caixa `[0,0]..[4,2]`.
    let want = [[0.0, 2.0], [4.0, 2.0], [4.0, 0.0], [0.0, 0.0]];
    for (i, w) in want.iter().enumerate() {
        assert!(
            (hs[i].world[0] - w[0]).abs() < 1e-5 && (hs[i].world[1] - w[1]).abs() < 1e-5,
            "alça {i}: {:?} vs {w:?}",
            hs[i].world
        );
    }
}

/// **O BEZIER OFERECE DOZE, E AS OITO TANGENTES NASCEM NOS TERÇOS.**
#[test]
fn the_bezier_offers_twelve_handles_with_the_tangents_at_the_thirds() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let (hs, n) = handles(spec, unit_box(), 1.0, &params(&[]));
    assert_eq!(n, MAX_HANDLES, "quatro cantos + oito tangentes");
    // A primeira tangente do topo: um terço de TL → TR, ou seja x = 4/3, y = 2.
    let t = hs[4];
    assert!(matches!(t.kind, WarpHandleKind::Tangent(_, 0)));
    assert!(
        (t.world[0] - 4.0 / 3.0).abs() < 1e-5 && (t.world[1] - 2.0).abs() < 1e-5,
        "a tangente nasce no terço: {:?}",
        t.world
    );
}

/// **UM CANTO MOVIDO LEVA AS TANGENTES DELE JUNTO — NO CORNER PIN.**
///
/// ⚠️ É o que faz o contorno desenhado ser o quadrilátero REAL. Sem isto o overlay
/// desenharia as arestas do quad de origem enquanto os cantos já estão noutro sítio —
/// uma moldura que mente sobre a figura que ela envolve.
#[test]
fn the_corner_pin_outline_follows_its_moved_corners() {
    let spec = spec_for(NodeTypeId::of("motion.four_point_warp")).expect("spec");
    let b = boundary(
        spec,
        unit_box(),
        1.0,
        &params(&[("tr_dx", 2.0), ("tr_dy", 1.0)]),
    );
    // TR foi para (6, 3); as tangentes do TOPO têm de estar nos terços de TL → TR.
    assert!((b.corner[TR][0] - 6.0).abs() < 1e-5 && (b.corner[TR][1] - 3.0).abs() < 1e-5);
    let want = [0.0 + (6.0 - 0.0) / 3.0, 2.0 + (3.0 - 2.0) / 3.0];
    assert!(
        (b.tangent[TOP][0][0] - want[0]).abs() < 1e-5
            && (b.tangent[TOP][0][1] - want[1]).abs() < 1e-5,
        "a tangente segue o canto: {:?} vs {want:?}",
        b.tangent[TOP][0]
    );
    // E o CONTORNO passa por lá: com arestas rectas, o ponto do meio do topo é o
    // ponto médio dos dois cantos.
    let ring = outline(&b);
    let mid = ring[OUTLINE_SEGMENTS / 2];
    let want_mid = [3.0, 2.5];
    assert!(
        (mid[0] - want_mid[0]).abs() < 1e-4 && (mid[1] - want_mid[1]).abs() < 1e-4,
        "meio do topo: {mid:?} vs {want_mid:?}"
    );
}

/// **SEED = SAMPLE: arrastar uma alça pelo delta escreve o param que a repõe ali.**
///
/// ⚠️ **O gate central deste módulo.** Ele fecha o ciclo pelo produto: lê a alça, aplica
/// a edição que o arrasto geraria, relê a alça, e exige que ela tenha ido EXACTAMENTE
/// para onde o dedo estava. Um gizmo cujo writeback não é o inverso da semente escorrega
/// do cursor, e o defeito só aparece com o `warp` fora de `1`.
#[test]
fn dragging_a_handle_writes_the_param_that_puts_it_under_the_finger() {
    for warp in [1.0f32, 0.5, 2.0, -1.0] {
        let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
        let b = unit_box();
        let (hs, _) = handles(spec, b, warp, &params(&[]));
        let h = hs[1]; // TR
        let delta = [0.7f32, -0.3];
        let target = [h.world[0] + delta[0], h.world[1] + delta[1]];
        let e = edits(&h, [0.0, 0.0], delta, warp).expect("warp não-nulo tem inverso");
        // Reconstrói a porta de param com o que o arrasto escreveu.
        let written: Vec<(String, f32)> = e.iter().map(|(k, v)| ((*k).to_string(), *v)).collect();
        let port = move |n: &str| {
            written
                .iter()
                .find(|(k, _)| k == n)
                .map_or(0.0, |(_, v)| *v)
        };
        let (after, _) = handles(spec, b, warp, &port);
        assert!(
            (after[1].world[0] - target[0]).abs() < 1e-4
                && (after[1].world[1] - target[1]).abs() < 1e-4,
            "warp {warp}: a alça foi para {:?}, o dedo estava em {target:?}",
            after[1].world
        );
    }
}

/// **UM `warp` NULO NÃO TEM INVERSO, E O ARRASTO RECUSA.**
///
/// ⚠️ Com a porta a zero o nó não aplica offset nenhum, então nenhum valor de param
/// põe a alça noutro sítio — dividir daria um infinito que envenenaria o documento.
/// Recusar é o único comportamento honesto.
#[test]
fn a_zero_warp_has_no_inverse_and_the_drag_declines() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let (hs, _) = handles(spec, unit_box(), 1.0, &params(&[]));
    assert!(edits(&hs[0], [0.0, 0.0], [1.0, 1.0], 0.0).is_none());
    assert!(edits(&hs[0], [0.0, 0.0], [1.0, 1.0], f32::NAN).is_none());
    // O controle: um `warp` normal responde.
    assert!(edits(&hs[0], [0.0, 0.0], [1.0, 1.0], 1.0).is_some());
}

/// **O AGARRE ESCOLHE A MAIS PRÓXIMA, NÃO A PRIMEIRA.**
///
/// ⚠️ Num quadrilátero pouco deformado a tangente nasce perto do canto. Um
/// "primeiro que couber" faria o canto (índice menor) roubar sempre o gesto da
/// tangente, e o artista leria *"esta alça não pega"*.
#[test]
fn the_grab_picks_the_nearest_handle_not_the_first() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let (hs, n) = handles(spec, unit_box(), 1.0, &params(&[]));
    let live = &hs[..n];
    // A tangente do topo (índice 4) vive em (4/3, 2); o canto TL (índice 0) em (0, 2).
    // Um ponto quase em cima da tangente, com um raio grande o bastante para os dois
    // caberem.
    let world_per_px = 0.15; // raio ≈ 1,65 unidades — alcança os dois
    let near_tangent = [4.0 / 3.0 + 0.05, 2.0];
    assert_eq!(
        hit(live, near_tangent, world_per_px),
        Some(4),
        "o ponto está sobre a tangente, e é ela que tem de pegar"
    );
    // O controle: perto do canto, é o canto.
    assert_eq!(hit(live, [0.05, 2.0], world_per_px), Some(0));
    // E fora do raio, ninguém.
    assert_eq!(hit(live, [2.0, -5.0], 0.01), None);
}

/// **CADA TANGENTE SABE DE QUE CANTO SAI** — o braço que a torna legível.
#[test]
fn every_tangent_names_the_corner_its_arm_leaves_from() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let (hs, n) = handles(spec, unit_box(), 1.0, &params(&[]));
    for h in &hs[..n] {
        match h.kind {
            WarpHandleKind::Corner(_) => assert!(tangent_arm(h.kind).is_none()),
            WarpHandleKind::Tangent(..) => {
                let c = tangent_arm(h.kind).expect("uma tangente tem braço");
                assert!(c < 4, "o braço nomeia um canto");
            }
        }
    }
    // E o braço é o canto CERTO: a 1ª tangente do topo sai de TL, a 2ª chega a TR.
    assert_eq!(tangent_arm(WarpHandleKind::Tangent(0, 0)), Some(0));
    assert_eq!(tangent_arm(WarpHandleKind::Tangent(0, 1)), Some(1));
}

/// **UMA CAIXA DEGENERADA NÃO TEM GIZMO.**
///
/// Uma linha ou um ponto não tem quadrilátero, e o nó passa o layout verbatim ali —
/// alças sobre uma caixa que não existe seriam alças que não fazem nada.
#[test]
fn a_degenerate_layout_has_no_box() {
    assert!(WarpBox::of(&[]).is_none());
    assert!(WarpBox::of(&[[1.0, 1.0]]).is_none(), "um ponto");
    let line: Vec<[f32; 2]> = (0..5).map(|i| [i as f32, 3.0]).collect();
    assert!(WarpBox::of(&line).is_none(), "uma linha horizontal");
    // O controle: um bloco de verdade tem caixa.
    let block = [[0.0, 0.0], [2.0, 0.0], [0.0, 1.0], [2.0, 1.0]];
    let b = WarpBox::of(&block).expect("um bloco tem caixa");
    assert_eq!(b.lo, [0.0, 0.0]);
    assert_eq!(b.hi, [2.0, 1.0]);
}

/// **O CONTORNO DO BEZIER É A CURVA QUE O NÓ COMPUTA** — e não uma segunda cópia.
///
/// ⚠️ O gate mede o ponto médio de uma aresta com barriga contra a cúbica avaliada
/// directamente pela função do CRATE DO NÓ. Se alguém reimplementar a Bézier aqui, esta
/// linha é a que sangra.
#[test]
fn the_outline_is_the_nodes_own_curve() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let b = boundary(
        spec,
        unit_box(),
        1.0,
        &params(&[("top_a_dy", 1.5), ("top_b_dy", 1.5)]),
    );
    let ring = outline(&b);
    let k = OUTLINE_SEGMENTS / 2;
    let t = k as f32 / OUTLINE_SEGMENTS as f32;
    let want = ph2d_node_motion_bezier_warp::coons::bezier(
        b.corner[TL],
        b.tangent[TOP][0],
        b.tangent[TOP][1],
        b.corner[TR],
        t,
    );
    assert!(
        (ring[k][0] - want[0]).abs() < 1e-6 && (ring[k][1] - want[1]).abs() < 1e-6,
        "o contorno é a cúbica do nó: {:?} vs {want:?}",
        ring[k]
    );
    // E o CONTROLE: a barriga existe (senão a comparação seria sobre uma recta).
    assert!(ring[k][1] > 2.5, "a aresta de cima arqueia: {:?}", ring[k]);
}
