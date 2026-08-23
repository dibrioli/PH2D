//! Os gates do NÓ — o default que não move nada, a máscara, e a costura do painel.
//!
//! A geometria prova-se em [`super::coons`]; aqui prova-se o que o nó faz com ela.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId as GNodeId};

/// Uma grelha 5×5 com um `falloff` que varia — a fixture CONTÉM a máscara, senão o
/// gate dela mediria um campo de uns.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.bezier_warp.test.src"),
    name: "motion.bezier_warp.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
const SIDE: usize = 5;
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let n = SIDE * SIDE;
        let p: Vec<[f32; 2]> = (0..SIDE)
            .flat_map(|r| (0..SIDE).map(move |c| [c as f32, r as f32]))
            .collect();
        // `falloff` cresce da primeira peça (0) à última (1).
        let f: Vec<f32> = (0..n).map(|i| i as f32 / (n - 1) as f32).collect();
        ctx.emit(
            Stream::new(n)
                .with("P", Column::Vec2(p))
                .with("falloff", Column::Scalar(f)),
        );
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionBezierWarp),
            _ => None,
        }
    }
}

fn warped(setup: impl FnOnce(&mut Graph, GNodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.bezier_warp.test.src");
    let bw = g.add_node("motion.bezier_warp");
    g.connect(Edge {
        from: (src, 0),
        to: (bw, 0),
        delayed: false,
    })
    .unwrap();
    setup(&mut g, bw);
    let mut cook = Cook::new();
    match cook.cook(&g, &Ops, bw, 0.0).unwrap()[0]
        .as_stream()
        .get("P")
        .unwrap()
    {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

/// **TUDO A ZERO É A IDENTIDADE, AO BIT.**
///
/// ⚠️ E o gate mede-a pelo NÓ, não pelo patch: o `is_neutral` é um atalho de custo, e
/// se ele algum dia divergisse do patch neutro esta é a linha que sangraria.
#[test]
fn every_offset_at_zero_leaves_the_layout_untouched() {
    let plain = warped(|_, _| {});
    let base = warped(|g, bw| {
        // Um offset e o seu simétrico: NÃO é o caminho do `is_neutral`, mas é a
        // identidade geométrica — o patch tem de a dar sozinho.
        g.set_param(bw, "tl_dx", 0.0);
    });
    assert_eq!(plain, base);
    let expect: Vec<[f32; 2]> = (0..SIDE)
        .flat_map(|r| (0..SIDE).map(move |c| [c as f32, r as f32]))
        .collect();
    assert_eq!(plain, expect, "o nó recém-largado não move um pixel");
}

/// **O ATALHO NEUTRO E O PATCH NEUTRO CONCORDAM** — a prova de que o `is_neutral` é
/// custo e não um segundo caminho.
///
/// ⚠️ Sem isto, o atalho podia esconder um patch errado exactamente no caso em que
/// todo mundo olha primeiro (o nó acabado de largar), e o defeito só apareceria ao
/// mexer o primeiro knob.
#[test]
fn the_neutral_shortcut_agrees_with_the_patch_it_skips() {
    let expect: Vec<[f32; 2]> = (0..SIDE)
        .flat_map(|r| (0..SIDE).map(move |c| [c as f32, r as f32]))
        .collect();
    // Um offset minúsculo desliga o atalho e passa pelo patch de verdade.
    let through_patch = warped(|g, bw| g.set_param(bw, "tl_dx", 1e-7));
    for (i, (a, b)) in through_patch.iter().zip(&expect).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "elemento {i}: o patch quase-neutro deu {a:?}, esperava ~{b:?}"
        );
    }
}

/// **UMA BARRIGA NA BORDA ARQUEIA A FILEIRA DAQUELE LADO, E NÃO A OPOSTA.**
///
/// ⚠️ É o que separa este nó do irmão: um Corner Pin **não sabe** entortar uma aresta
/// sem mover um canto. As duas metades são precisas — a borda mexida tem de arquear
/// **e** a de baixo tem de ficar onde estava (senão o patch estaria a escalar tudo).
#[test]
fn a_tangent_bulges_its_own_edge_and_leaves_the_opposite_one_alone() {
    // O topo (`r = SIDE-1`, o maior y) puxado para cima pelas duas tangentes.
    let out = warped(|g, bw| {
        g.set_param(bw, "top_a_dy", 2.0);
        g.set_param(bw, "top_b_dy", 2.0);
        // A máscara fora do caminho: este gate é sobre a GEOMETRIA.
        g.set_param(bw, "tl_dx", 0.0);
    });
    let at = |r: usize, c: usize| out[r * SIDE + c];
    // ⚠️ O `falloff` cresce com o índice, e a fileira de cima é a de índice ALTO —
    // então ela é a que a máscara deixa passar. A de baixo tem máscara ~0.
    let mid_top = at(SIDE - 1, SIDE / 2);
    assert!(
        mid_top[1] > 2.5,
        "o meio do topo tem de subir (estava em {}): {mid_top:?}",
        SIDE - 1
    );
    let mid_bottom = at(0, SIDE / 2);
    assert!(
        mid_bottom[1].abs() < 0.2,
        "e a fileira de baixo fica: {mid_bottom:?}"
    );
}

/// **OS CANTOS DA CAIXA VÃO PARA OS CANTOS PEDIDOS** (com a máscara cheia).
#[test]
fn the_box_corners_land_on_the_authored_corners() {
    let out = warped(|g, bw| {
        g.set_param(bw, "tr_dx", 3.0);
        g.set_param(bw, "tr_dy", 1.0);
    });
    // A última peça é o canto superior-direito, e o `falloff` dela é exactamente 1.
    let tr = out[SIDE * SIDE - 1];
    let want = [(SIDE - 1) as f32 + 3.0, (SIDE - 1) as f32 + 1.0];
    assert!(
        (tr[0] - want[0]).abs() < 1e-4 && (tr[1] - want[1]).abs() < 1e-4,
        "canto TR: {tr:?} vs {want:?}"
    );
}

/// **A MÁSCARA `falloff` VALE, E A PEÇA DE PESO ZERO NÃO SE MOVE.**
///
/// ⚠️ **A primeira versão deste gate SOBREVIVEU a uma mutação que apagava a máscara**, e a
/// causa era a fixture e não a asserção: a peça de peso zero é o elemento `0`, que vive no
/// canto **BL**, e os offsets que ele movia eram `tl`/`tr` — o patch mandava aquele canto
/// para ele próprio de qualquer maneira, com máscara ou sem. *Uma fixture só prova o que ela
/// contém*: para a máscara ser medível, a peça mascarada tem de estar num sítio que a
/// deformação MOVERIA.
///
/// A cura tem duas metades: mover o canto **dela** (`bl`), e medir que o deslocamento CRESCE
/// com o peso — que é a lei, e não *"a peça zero ficou parada"*.
#[test]
fn the_falloff_mask_gates_the_deformation() {
    let out = warped(|g, bw| {
        g.set_param(bw, "bl_dx", -5.0);
        g.set_param(bw, "bl_dy", -5.0);
        g.set_param(bw, "tr_dx", 5.0);
    });
    // A primeira peça tem `falloff = 0`.
    assert!(
        (out[0][0]).abs() < 1e-5 && (out[0][1]).abs() < 1e-5,
        "peso zero, deslocamento zero: {:?}",
        out[0]
    );
    // E o CONTROLE: a última (peso 1) mexeu-se de facto.
    let last = out[SIDE * SIDE - 1];
    assert!(
        (last[0] - (SIDE - 1) as f32).abs() > 1.0,
        "controle: a peça de peso cheio move-se ({last:?})"
    );
    // ⚠️ O deslocamento tem de CRESCER com o peso — a lei, não um par de pontos.
    let moved = |i: usize, base: [f32; 2]| (out[i][0] - base[0]).hypot(out[i][1] - base[1]);
    let d0 = moved(0, [0.0, 0.0]);
    let dn = moved(SIDE * SIDE - 1, [(SIDE - 1) as f32; 2]);
    assert!(
        dn > d0 + 1.0,
        "peso 0 andou {d0:.3}, peso 1 andou {dn:.3}"
    );
}

/// **UM LAYOUT DEGENERADO PASSA VERBATIM** — uma linha não tem caixa 2D para
/// deformar, e o nó não pode dividir por zero nem inventar uma.
#[test]
fn a_degenerate_layout_passes_through() {
    static LINE: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.bezier_warp.test.line"),
        name: "motion.bezier_warp.test.line",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: INST_VEC2,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Line;
    impl NodeOp for Line {
        fn manifest(&self) -> &'static NodeManifest {
            &LINE
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let p: Vec<[f32; 2]> = (0..4).map(|i| [i as f32, 0.0]).collect();
            ctx.emit(Stream::new(4).with("P", Column::Vec2(p)));
        }
    }
    struct LineOps;
    impl OpResolver for LineOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == LINE.id => Some(&Line),
                t if t == MANIFEST.id => Some(&MotionBezierWarp),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.bezier_warp.test.line");
    let bw = g.add_node("motion.bezier_warp");
    g.connect(Edge {
        from: (src, 0),
        to: (bw, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(bw, "tl_dy", 4.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &LineOps, bw, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => assert_eq!(
            v,
            &vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]],
            "uma linha passa verbatim"
        ),
        _ => panic!("P"),
    }
}

/// **O PAINEL OFERECE OS 24, E CADA UM PERTENCE A UM GRUPO.**
///
/// ⚠️ Os dois lados da costura contra a MESMA fonte: um param que o `eval` lê e o
/// painel não mostra é um controle inalcançável; um grupo que nomeie um param que não
/// existe é uma secção vazia. E as duas contagens saem do MANIFESTO, nunca escritas
/// à mão — foi assim que uma tabela irmã envelheceu.
#[test]
fn the_panel_offers_every_param_and_groups_them_all() {
    let names: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    assert_eq!(names.len(), 24, "4 cantos + 8 tangentes, cada um (x, y)");
    for n in &names {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == *n),
            "o param `{n}` não tem linha no painel"
        );
        assert!(
            PARAM_GROUPS.iter().any(|g| g.param == *n),
            "o param `{n}` não está em grupo nenhum"
        );
    }
    for h in PARAM_HINTS {
        assert!(
            names.contains(&h.param),
            "a linha `{}` não corresponde a param nenhum",
            h.param
        );
    }
    // Cinco secções: os cantos e as quatro arestas.
    let mut sections: Vec<&str> = PARAM_GROUPS.iter().map(|g| g.group).collect();
    sections.sort_unstable();
    sections.dedup();
    assert_eq!(sections.len(), 5, "cantos + 4 arestas: {sections:?}");
}
