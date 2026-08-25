//! Os gates das setas — **geometria pura**, sem janela e sem GPU.

use super::*;
use ph2d_morph_machine::MorphEdge;

const A: ShapeId = 10;
const B: ShapeId = 20;

fn win() -> WindowSize {
    WindowSize {
        width: 800,
        height: 600,
    }
}

fn cam(height_world: f32) -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world,
        cull_mask: u32::MAX,
    }
}

fn boxes() -> Vec<ShapeBox> {
    vec![
        ShapeBox {
            id: A,
            center: [-2.0, 0.0],
            half: [0.5, 0.5],
        },
        ShapeBox {
            id: B,
            center: [2.0, 0.0],
            half: [0.5, 0.5],
        },
    ]
}

fn edge(from: ShapeId, to: ShapeId) -> MorphEdge {
    MorphEdge::new(from, to)
}

fn graph(edges: Vec<MorphEdge>) -> MorphGraph {
    MorphGraph { start: A, edges }
}

/// O ponto de controlo da quadrática — o que carrega a curvatura.
fn control(a: &Arrow) -> Point {
    for el in a.path.elements() {
        if let ph2d_vector::PathEl::QuadTo(c, _) = el {
            return *c;
        }
    }
    panic!("a seta tem de ser uma quadratica");
}

fn first(a: &Arrow) -> Point {
    match a.path.elements()[0] {
        ph2d_vector::PathEl::MoveTo(p) => p,
        _ => panic!("comeca com um MoveTo"),
    }
}

/// ⭐ **`A→B` e `B→A` NÃO desenham a mesma linha.**
///
/// Toda máquina útil tem pares de ida e volta, e duas rectas entre os mesmos dois centros são
/// **uma** recta na tela — o artista veria uma seta onde há duas.
///
/// **Mutação que deve sangrar:** `sign` virar `1.0` fixo.
#[test]
fn the_two_directions_never_draw_the_same_line() {
    let bs = boxes();
    let ida = arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(10.0), win());
    let volta = arrows(&graph(vec![edge(B, A)]), &bs, None, &cam(10.0), win());
    let (ci, cv) = (control(&ida[0]), control(&volta[0]));
    assert!(
        (ci.y - cv.y).abs() > BEND_PX,
        "as duas curvas dobram para o mesmo lado ({ci:?} vs {cv:?}): na tela sao uma so'"
    );
}

/// ⚠️ **O lado da curva NÃO depende da ordem em que o artista desenhou.**
///
/// Ele sai da direcção de viagem (`perp(u)`), e a garantia estrutural é que a função que o calcula
/// **não recebe o índice da aresta**. Este gate é a rede contra o dia em que alguém o passar para
/// lá — por uma cor por-índice, um desvio anti-sobreposição, o que vier.
///
/// **Mutação que deve sangrar:** somar o índice ao desvio.
#[test]
fn the_bend_side_does_not_depend_on_the_authoring_order() {
    let bs = boxes();
    let um = arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(10.0), win());
    // A MESMA seta, agora em segundo lugar na lista.
    let dois = arrows(
        &graph(vec![edge(B, A), edge(A, B)]),
        &bs,
        None,
        &cam(10.0),
        win(),
    );
    assert_eq!(
        control(&um[0]),
        control(&dois[1]),
        "a seta A->B mudou de lado so' por ter mudado de posicao na lista"
    );
}

/// ⭐ **A seta começa na BORDA da forma, nunca no centro dela.**
///
/// Uma seta que nasce no meio de um rectângulo grande fica escondida por baixo dele.
///
/// **Mutação que deve sangrar:** o `exit` devolver o próprio centro.
#[test]
fn an_arrow_starts_at_the_shapes_edge_not_at_its_centre() {
    let bs = boxes();
    let a = &arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(10.0), win())[0];
    let (cx, _) = cam(10.0).world_to_screen(bs[0].center, win());
    let (ex, _) = cam(10.0).world_to_screen([bs[0].center[0] + bs[0].half[0], 0.0], win());
    let half_px = f64::from(ex - cx).abs();
    let start = first(a);
    assert!(
        start.x - f64::from(cx) >= half_px,
        "a seta comeca DENTRO da forma: {} px do centro, e a meia-largura e' {half_px}",
        start.x - f64::from(cx)
    );
}

/// ⭐⭐ **A CURVATURA É DE ECRÃ** — ela não encolhe quando o artista afasta o zoom.
///
/// ⚠️ É exactamente com a máquina inteira à vista que a ida e a volta precisam de se distinguir.
/// Uma curvatura em mundo desapareceria aí, que é o único sítio onde ela importa.
///
/// **Mutação que deve sangrar:** o `BEND_PX` ser aplicado em unidades de mundo.
#[test]
fn the_bend_is_screen_space_so_zooming_out_never_merges_the_pair() {
    let bs = boxes();
    let sep = |h: f32| {
        let i = arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(h), win());
        let v = arrows(&graph(vec![edge(B, A)]), &bs, None, &cam(h), win());
        (control(&i[0]).y - control(&v[0]).y).abs()
    };
    let (perto, longe) = (sep(10.0), sep(1000.0));
    assert!(
        (perto - longe).abs() < 0.5,
        "a separacao das duas curvas mudou com o zoom ({perto} px perto, {longe} px longe): \
         afastar a camera funde a ida com a volta"
    );
}

/// **Uma forma apagada leva a seta dela** — em vez de a fazer apontar para a origem do mundo.
///
/// ⚠️ Mesma escolha do `morph_live`, que **congela** a forma quando uma fonte some.
#[test]
fn a_missing_shape_drops_its_arrow_instead_of_pointing_at_the_origin() {
    let so_a = vec![boxes()[0]];
    let out = arrows(&graph(vec![edge(A, B)]), &so_a, None, &cam(10.0), win());
    assert!(out.is_empty(), "a seta sobreviveu a' forma que ela aponta");
    // O CONTROLE: com as duas formas, a MESMA aresta desenha.
    assert_eq!(
        arrows(&graph(vec![edge(A, B)]), &boxes(), None, &cam(10.0), win()).len(),
        1
    );
}

/// **A ponta aponta na direcção com que a curva CHEGA.**
///
/// ⚠️ Numa seta curva a tangente do fim (`end − mid`) e a recta centro-a-centro apontam para sítios
/// diferentes — usar a segunda faria a ponta olhar de lado.
///
/// **Mutação que deve sangrar:** derivar a ponta de `(ux, uy)` em vez da tangente.
#[test]
fn the_head_points_the_way_the_curve_arrives() {
    let bs = boxes();
    let a = &arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(10.0), win())[0];
    let mid = control(a);
    let end = match a.path.elements()[1] {
        ph2d_vector::PathEl::QuadTo(_, p) => p,
        _ => panic!("quadratica"),
    };
    let (tx, ty) = (end.x - mid.x, end.y - mid.y);
    let tl = tx.hypot(ty);
    let mut arms = Vec::new();
    for el in a.head.elements() {
        if let ph2d_vector::PathEl::LineTo(p) = el {
            arms.push(*p);
        }
    }
    assert_eq!(arms.len(), 2, "a ponta tem duas abas");
    for p in &arms {
        assert!(
            ((p.x - end.x).hypot(p.y - end.y) - HEAD_PX).abs() < 0.01,
            "a aba nao mede o que o token diz"
        );
    }
    // ⚠️ **O sinal do produto interno NAO chega, e a mutacao provou-o**: com uma curvatura de
    // 22 px sobre um segmento de centenas, a recta centro-a-centro e a tangente de chegada
    // apontam quase para o mesmo lado, e as duas passam num teste de sinal. A regua tem de ser o
    // ANGULO: o bissector das duas abas e' EXACTAMENTE o oposto da tangente de chegada.
    let bis = (
        (arms[0].x + arms[1].x) * 0.5 - end.x,
        (arms[0].y + arms[1].y) * 0.5 - end.y,
    );
    let bl = bis.0.hypot(bis.1);
    let cos = -(bis.0 * tx + bis.1 * ty) / (bl * tl);
    assert!(
        cos > 1.0 - 1e-9,
        "o bissector da ponta desvia da tangente de chegada em {:.3}° -- a seta olha de lado",
        cos.clamp(-1.0, 1.0).acos().to_degrees()
    );
}

/// **Duas formas no mesmo sítio não dividem por zero** — devolvem um caminho vazio.
#[test]
fn two_shapes_in_the_same_place_draw_nothing_instead_of_dividing_by_zero() {
    let bs = vec![
        ShapeBox {
            id: A,
            center: [1.0, 1.0],
            half: [0.5, 0.5],
        },
        ShapeBox {
            id: B,
            center: [1.0, 1.0],
            half: [0.5, 0.5],
        },
    ];
    let out = arrows(&graph(vec![edge(A, B)]), &bs, None, &cam(10.0), win());
    assert_eq!(out.len(), 1, "a seta existe -- as duas formas existem");
    assert!(out[0].path.elements().is_empty(), "e nao desenha nada");
}

/// **A transição em VOO vem marcada** — é o realce que o artista lê como *"é esta que está a
/// correr agora"*.
#[test]
fn the_flying_edge_is_the_one_marked_live() {
    let bs = boxes();
    let g = graph(vec![edge(A, B), edge(B, A)]);
    let out = arrows(&g, &bs, Some((B, A)), &cam(10.0), win());
    assert_eq!(
        out.iter().map(|a| a.live).collect::<Vec<_>>(),
        vec![false, true]
    );
}
