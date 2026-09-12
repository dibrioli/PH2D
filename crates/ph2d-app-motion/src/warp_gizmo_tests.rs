//! Os gates do gizmo de canvas dos deformadores de quadrilátero.
//!
//! ⚠️ A lei central é **SEED = SAMPLE**: onde a alça é desenhada e o que o arrasto
//! escreve têm de ser inversos exactos. Um round-trip que não fecha é o gizmo a
//! escorregar do cursor, e é o defeito que esta casa já pagou em três gizmos.

use super::doc::fit_downstream;
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
        let e = edits(&h, [0.0, 0.0], delta, warp, Downstream::IDENTITY)
            .expect("warp não-nulo tem inverso");
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
    assert!(edits(&hs[0], [0.0, 0.0], [1.0, 1.0], 0.0, Downstream::IDENTITY).is_none());
    assert!(
        edits(
            &hs[0],
            [0.0, 0.0],
            [1.0, 1.0],
            f32::NAN,
            Downstream::IDENTITY
        )
        .is_none()
    );
    // O controle: um `warp` normal responde.
    assert!(edits(&hs[0], [0.0, 0.0], [1.0, 1.0], 1.0, Downstream::IDENTITY).is_some());
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

// ─── A projecção: o que se PINTA e o que se AGARRA têm de ser a mesma coisa ───

/// **A ALÇA DESENHADA É A ALÇA QUE PEGA** — o gate que teria apanhado o defeito de
/// 2026-08-23.
///
/// ⚠️ **Relato do Enio:** *"grade fora do lugar. drift. Não consegui manipular pontos e
/// alças no canvas"* — dois sintomas, **uma** causa: a tinta projectava com a janela
/// CHEIA e o hit-test com a da CENA. A alça que se via não era a alça que existia, então
/// o desenho saía deslocado *e* o clique errava. A lei estava escrita no `field_gizmo`
/// (*"a vector shape projected with the FULL window drifts off them"*), eu li-a, e mesmo
/// assim passei a janela errada ao overlay. **Uma lei sem gate é uma nota.**
///
/// O gate fecha o ciclo: projecta cada alça para a tela com a MESMA porta que a tinta
/// usa, volta ao mundo pela porta do ponteiro, e exige que o agarre encontre aquela alça.
/// Ele reprova para qualquer par de janelas que discorde — que é exactamente o defeito.
#[test]
fn the_handle_you_see_is_the_handle_you_grab() {
    use ph2d_host::WindowSize;
    use ph2d_render::Camera2d;

    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let b = unit_box();
    let port = params(&[("tr_dx", 0.8), ("top_a_dy", 0.6)]);
    let (hs, n) = handles(spec, b, 1.0, &port);
    let camera = Camera2d::new([2.0, 1.0], 6.0);

    // ⚠️ A janela da CENA — a mesma dos dois lados. Uma janela "cheia" mais alta é
    // exactamente o que o defeito passava à tinta.
    let scene = WindowSize::new(800, 450);
    let to_screen = camera.world_to_screen_affine(scene);

    for (i, h) in hs[..n].iter().enumerate() {
        // Onde a TINTA põe esta alça.
        let p = to_screen * ph2d_vector::Point::new(f64::from(h.world[0]), f64::from(h.world[1]));
        // O PONTEIRO clica exactamente ali, e volta ao mundo pela porta dele.
        #[expect(clippy::cast_possible_truncation, reason = "px de tela cabem num f32")]
        let world = camera.screen_to_world((p.x as f32, p.y as f32), scene);
        // Um pixel de tela em unidades de mundo, como a costura calcula.
        let a = camera.screen_to_world((0.0, 0.0), scene);
        let c = camera.screen_to_world((1.0, 0.0), scene);
        let wpp = (c[0] - a[0]).hypot(c[1] - a[1]);
        assert_eq!(
            hit(&hs[..n], world, wpp),
            Some(i),
            "a alça {i} desenhada em {p:?} tem de ser a que o clique ali agarra"
        );
    }
}

/// **E O CONTROLE: com as janelas DIFERENTES, o ciclo QUEBRA.**
///
/// ⚠️ Sem esta metade o gate acima passaria por vacuidade num dia em que a projecção
/// deixasse de importar. Ela reproduz o defeito de propósito — pinta com uma janela e
/// agarra com outra — e exige que ele seja detectável.
#[test]
fn painting_with_one_window_and_grabbing_with_another_breaks_the_cycle() {
    use ph2d_host::WindowSize;
    use ph2d_render::Camera2d;

    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let b = unit_box();
    let (hs, n) = handles(spec, b, 1.0, &params(&[]));
    let camera = Camera2d::new([2.0, 1.0], 6.0);
    let scene = WindowSize::new(800, 450);
    // A janela CHEIA, mais alta que a da cena — o que o defeito passava à tinta.
    let full = WindowSize::new(800, 900);

    let painted = camera.world_to_screen_affine(full);
    let mut missed = 0;
    for h in &hs[..n] {
        let p = painted * ph2d_vector::Point::new(f64::from(h.world[0]), f64::from(h.world[1]));
        #[expect(clippy::cast_possible_truncation, reason = "px de tela cabem num f32")]
        let world = camera.screen_to_world((p.x as f32, p.y as f32), scene);
        let a = camera.screen_to_world((0.0, 0.0), scene);
        let c = camera.screen_to_world((1.0, 0.0), scene);
        let wpp = (c[0] - a[0]).hypot(c[1] - a[1]);
        if hit(&hs[..n], world, wpp).is_none() {
            missed += 1;
        }
    }
    assert!(
        missed >= n / 2,
        "com janelas diferentes o clique tem de ERRAR a maioria das alças ({missed} de {n}) \
         — se não errasse, o gate irmão não estaria a medir a projecção"
    );
}

// ─── A cadeia de JUSANTE: o gizmo no espaço do que se VÊ ───

/// **UM AFIM A JUSANTE É RECUPERADO EXACTAMENTE, E VERIFICADO.**
///
/// ⚠️ **Relato do Enio (2026-08-23):** *"se o nó Bezier Warp é colocado antes de
/// Transform, a grade é desenhada na posição (0,0)"*. E o gizmo estava certo e inútil ao
/// mesmo tempo: os params dele são offsets sobre a caixa do que ENTRA, e com o transform a
/// jusante o que entra é a grelha crua na origem. *Um gizmo correcto no frame errado é um
/// gizmo errado.*
///
/// A cura mede a cadeia por **correspondência de elemento** em vez de a presumir. Este
/// gate prova a recuperação num afim genérico (roda, escala e translada de uma vez).
#[test]
fn a_downstream_affine_is_recovered_exactly() {
    // Um afim que mistura tudo — nada de só-translação, que passaria com metade da conta.
    let truth = Downstream {
        lin: [[0.8, -0.6], [0.5, 1.3]],
        tr: [4.0, -2.5],
    };
    let from: Vec<[f32; 2]> = (0..30)
        .map(|i| {
            let t = i as f32;
            [t * 0.31 - 2.0, (t * 0.17).sin() * 3.0]
        })
        .collect();
    let to: Vec<[f32; 2]> = from.iter().map(|p| truth.apply(*p)).collect();
    let got = fit_downstream(&from, &to).expect("um afim exacto tem de ser recuperado");
    for p in &from {
        let (a, b) = (got.apply(*p), truth.apply(*p));
        assert!(
            (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3,
            "{a:?} vs {b:?}"
        );
    }
}

/// **E UMA CADEIA NÃO-AFIM É RECUSADA** — a metade que impede o ajuste de virar palpite.
///
/// ⚠️ Sem a verificação, uma cadeia não-afim receberia o afim "menos errado" e o gizmo
/// cairia num sítio plausível e falso — pior que cair na origem, porque ninguém repara.
#[test]
fn a_non_affine_downstream_is_refused_rather_than_approximated() {
    let from: Vec<[f32; 2]> = (0..40)
        .map(|i| [i as f32 * 0.25 - 5.0, (i % 7) as f32 * 0.4])
        .collect();
    // Um mapa QUADRÁTICO: nenhum afim o reproduz.
    let to: Vec<[f32; 2]> = from.iter().map(|p| [p[0] * p[0], p[1]]).collect();
    assert!(
        fit_downstream(&from, &to).is_none(),
        "um mapa quadrático não é afim, e o ajuste tem de RECUSAR"
    );
    // E o CONTROLE: os mesmos pontos sob um afim de verdade passam.
    let ok: Vec<[f32; 2]> = from
        .iter()
        .map(|p| [p[0] * 2.0 + 1.0, p[1] - 3.0])
        .collect();
    assert!(
        fit_downstream(&from, &ok).is_some(),
        "controle: um afim passa"
    );
    // E pontos COLINEARES não determinam um afim — recusa, e não uma divisão por zero.
    let line: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
    let line_to: Vec<[f32; 2]> = line.iter().map(|p| [p[0] + 1.0, 0.0]).collect();
    assert!(fit_downstream(&line, &line_to).is_none(), "colineares");
}

/// **AS ALÇAS SEGUEM O TRANSFORM DE JUSANTE, E O ARRASTO CONTINUA A FECHAR.**
///
/// ⚠️ **O gate do defeito, e ele mede as DUAS pontas.** Com um transform a jusante, a alça
/// tem de ser desenhada onde o artista vê a figura (deslocada) **e** um arrasto ali tem de
/// escrever o param que a repõe sob o dedo — o que exige desfazer a jusante ANTES de
/// dividir pelo `warp`. Uma cura que só movesse o desenho poria a alça no sítio certo e
/// faria o arrasto saltar.
#[test]
fn the_handles_follow_the_downstream_transform_and_the_drag_still_closes() {
    let spec = spec_for(NodeTypeId::of("motion.bezier_warp")).expect("spec");
    let down = Downstream {
        lin: [[1.5, 0.0], [0.0, 1.5]],
        tr: [10.0, -4.0],
    };
    let v = WarpGizmoView {
        node: ph2d_nodegraph::graph::NodeId(0),
        spec,
        bbox: unit_box(),
        warp: 1.0,
        down,
    };
    let port = params(&[]);
    let (plain, _) = handles(spec, v.bbox, v.warp, &port);
    let (moved, n) = view_handles(&v, &port);
    for i in 0..n {
        let want = down.apply(plain[i].world);
        assert!(
            (moved[i].world[0] - want[0]).abs() < 1e-4
                && (moved[i].world[1] - want[1]).abs() < 1e-4,
            "alça {i}: {:?} vs {want:?}",
            moved[i].world
        );
    }
    // E o arrasto fecha: mover a alça por um delta NO ESPAÇO DO QUE SE VÊ tem de a pôr ali.
    let h = moved[1]; // TR
    let delta = [0.9f32, 0.6];
    let target = [h.world[0] + delta[0], h.world[1] + delta[1]];
    let e = edits(&h, [0.0, 0.0], delta, v.warp, down).expect("um afim invertível");
    let written: Vec<(String, f32)> = e.iter().map(|(k, val)| ((*k).to_string(), *val)).collect();
    let after_port = move |name: &str| {
        written
            .iter()
            .find(|(k, _)| k == name)
            .map_or(0.0, |(_, val)| *val)
    };
    let (after, _) = view_handles(&v, &after_port);
    assert!(
        (after[1].world[0] - target[0]).abs() < 1e-3
            && (after[1].world[1] - target[1]).abs() < 1e-3,
        "com transform a jusante o arrasto tem de fechar: {:?} vs {target:?}",
        after[1].world
    );
}

/// ⭐⭐ **O QUE SE VÊ É O QUE SE APONTA** — a alça pintada cobre o raio em que ela é agarrada.
///
/// ⛔⛔ **Report do Enio, 2026-09-08:** *«o gizmo de Bezier Warp tem ponto e handles/alças muito
/// pequenos»*. Medido: o canto pintava meio-lado `4,5` e a tangente raio `3,5`, contra um
/// [`GRAB_PX`] de **`11`** — o alvo era `2,4×` a `3,1×` maior do que a tinta. *O artista via um
/// ponto e apontava para outra coisa.*
///
/// ⚠️ **A casa já tinha a lei, e este gizmo era o único fora dela:** o `connector` pinta `7` e
/// agarra `7`, o `envelope` `6` e `6`, o `vec_text_ride` `15` e `15`. **Uma constante, dois
/// consumidores** — e aqui eram dois números, dos quais o que o artista vê é o que envelhece.
///
/// ⚠️ **A barra é `0,75 × GRAB_PX` sobre o raio INSCRITO da marca** (o quadrado pelo meio-lado, o
/// losango pela apótema `r/√2`), e não sobre o circunscrito: o inscrito é o pior caso do que a
/// marca cobre, e é ele que decide se o dedo cai dentro do que o olho vê.
///
/// FALSIFICADO pelos números que shipavam: `4,5` dá `0,41×` e `3,5` dá `0,22×`.
#[test]
fn a_painted_handle_covers_the_radius_it_is_grabbed_at() {
    let grab = f64::from(GRAB_PX);
    let bar = grab * 0.75;
    // O quadrado do canto: o raio inscrito é o meio-lado.
    let corner_inscribed = crate::warp_overlay::CORNER_PX;
    // O losango: quatro vértices a `r`, logo a apótema é `r / √2`.
    let tangent_inscribed = crate::warp_overlay::TANGENT_PX / std::f64::consts::SQRT_2;
    assert!(
        corner_inscribed >= bar,
        "o quadrado de um canto cobre {corner_inscribed:.2} px e é agarrado a {grab:.1} \
         ({:.2}× o agarre) — a barra é {bar:.2}. Os valores que shipavam davam 0,41×",
        corner_inscribed / grab
    );
    assert!(
        tangent_inscribed >= bar,
        "o losango de uma tangente cobre {tangent_inscribed:.2} px e é agarrado a {grab:.1} \
         ({:.2}× o agarre) — a barra é {bar:.2}. Os valores que shipavam davam 0,22×",
        tangent_inscribed / grab
    );
    // ⚠️ **E o CONTROLE: a barra tem de reprovar o que shipava.** Sem isto uma barra baixa demais
    // passaria sobre os mesmos números que produziram o report, e o gate seria decoração.
    //
    // ⚠️⚠️ **São DUAS asserções e não um `&&`, e a diferença não é de estilo.** Cada marca
    // mede-se à maneira dela — o quadrado pelo meio-lado, o losango pela apótema —, então são
    // duas grandezas distintas com duas causas distintas de falhar. Num `&&` a segunda é
    // **logicamente implicada** pela primeira (`3,5/√2 ≈ 2,47 < 4,5`), o clippy diz que ela não
    // tem efeito, **e tem razão**: escrita assim, o dia em que o losango deixasse de reprovar
    // não moveria o gate. *Uma conjunção esconde qual metade caiu; dois asserts nomeiam-na.*
    assert!(
        4.5 < bar,
        "a barra ({bar:.2}) tem de reprovar o meio-lado do quadrado que shipava (4,5)"
    );
    assert!(
        3.5 / std::f64::consts::SQRT_2 < bar,
        "a barra ({bar:.2}) tem de reprovar a apotema do losango que shipava (3,5/√2 = {:.2})",
        3.5 / std::f64::consts::SQRT_2
    );
}

/// ⭐⭐⭐ **O GIZMO CHEGA A PIXEL** — a metade que faltava, e que uma mudança de sítio cobrou.
///
/// ⛔⛔ **Report do Enio, 2026-09-08:** *«nessa última rodada vc sumiu com o gizmo do Bezier
/// Warp»*. Mover o desenho ~2 000 linhas para baixo no quadro fê-lo desaparecer, e **nada neste
/// repo o teria dito**: os gates deste ficheiro medem a GEOMETRIA (onde a alça está, se o arrasto
/// fecha) e a geometria continuou perfeita — *a faixa reservada não é a faixa pintada*, e aqui
/// nem sequer a faixa era o problema: era a tinta não sair.
///
/// Este gate não prova o SÍTIO (isso exige o quadro inteiro, e é por isso que a mudança tem de
/// ser instrumentada e não inferida). Ele prova a outra metade: **dado um retrato, o pintor emite
/// tinta.** Com ele, quem voltar a mexer no sítio sabe que a variável é o sítio.
///
/// FALSIFICADO por um `return` cedo no `draw_warp_gizmo`, por uma moldura degenerada, ou por o
/// `active` deixar de ser honrado.
#[test]
fn the_gizmo_paints_geometry_and_paints_none_when_inactive() {
    use ph2d_vector::VectorScene;

    let v = WarpGizmoView {
        node: ph2d_nodegraph::graph::NodeId(1),
        spec: WarpGizmoSpec { has_tangents: true },
        bbox: unit_box(),
        warp: 1.0,
        down: Downstream::IDENTITY,
    };
    let port = |_: &str| 0.0;
    let camera = ph2d_render::Camera2d::default();
    let janela = ph2d_host::WindowSize {
        width: 1200,
        height: 800,
    };
    let split = ph2d_editor::screens::layout::CenterSplit::None;

    let segmentos = |ativo: bool| -> u32 {
        let mut cena = VectorScene::new();
        crate::warp_overlay::draw_warp_gizmo(ativo, &v, &port, &camera, split, janela, &mut cena);
        cena.inner().encoding().n_path_segments
    };

    // ⚠️ **O controlo vem PRIMEIRO**: sem ele, um pintor que emitisse tinta sempre — mesmo com a
    // tool trocada — passaria a asserção de baixo e poria alças por cima de outra ferramenta.
    assert_eq!(
        segmentos(false),
        0,
        "fora da tool Motion o gizmo nao pinta NADA — as alcas dali seriam alvos a roubar o \
         clique de outra ferramenta"
    );
    let pintou = segmentos(true);
    assert!(
        pintou > 0,
        "com um retrato publicado o gizmo tem de emitir tinta — o contorno, os bracos e as doze \
         alcas ({pintou} segmentos)"
    );
}
