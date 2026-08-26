//! Os gates da **renumeração** ([`super::REINDEX`]) — a wave de 2026-08-19.
//!
//! ⚠️ **O defeito que estes gates fecham foi visto num SMOKE, e a suíte que já existia era
//! verde.** Ela media a permutação (`P` sai em X crescente) e nunca a coluna que o efector a
//! jusante de facto lê — o `Index`. Um gate que mede o lado certo da costura pelo lado errado
//! da fronteira passa exactamente enquanto o produto mente.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de `n` peças **fora de ordem em X**, com o `Index` honesto de quem nasce
/// (`0..n−1`) e um `id` que é a identidade durável.
///
/// ⚠️ As três colunas são de propósito distinguíveis: `P.x` diz *onde*, `Index` diz *que
/// posição na lista*, `id` diz *quem*. Uma fixture em que duas delas coincidissem não
/// conseguiria separar "renumerou" de "permutou".
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sort.test.idsrc"),
    name: "motion.sort.test.idsrc",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[ParamSpec {
        name: "bare",
        default: 0.0,
    }],
    lowerings: &[LoweringKind::Cpu],
};

/// As posições da fixture, na ordem de nascimento: o X corre 4, 1, 3, 0, 2.
const XS: [f32; 5] = [4.0, 1.0, 3.0, 0.0, 2.0];

struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let n = XS.len();
        let p: Vec<[f32; 2]> = XS.iter().map(|&x| [x, 0.0]).collect();
        let mut s = Stream::new(n).with("P", Column::Vec2(p));
        // `bare = 1` emite a MESMA lista sem coluna de identidade — o controle da cunhagem.
        if ctx.param("bare") < 0.5 {
            let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
            let ids: Vec<f32> = (0..n).map(|i| 100.0 + i as f32).collect();
            s = s
                .with("Index", Column::Scalar(idx))
                .with("id", Column::Scalar(ids));
        }
        ctx.emit(s);
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionSort),
            _ => None,
        }
    }
}

/// Coze `src → sort(X)` com o `reindex` pedido e devolve `(P.x, Index, id)`.
fn run(reindex: f32, bare: bool) -> (Vec<f32>, Option<Vec<f32>>, Option<Vec<f32>>) {
    let mut g = Graph::new();
    let src = g.add_node("motion.sort.test.idsrc");
    g.set_param(src, "bare", f32::from(u8::from(bare)));
    let s = g.add_node("motion.sort");
    g.set_param(s, "key", KEY_X as f32);
    g.set_param(s, REINDEX, reindex);
    g.connect(Edge {
        from: (src, 0),
        to: (s, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, s, 0.0).expect("coze");
    let st = out[0].as_stream();
    let scalar = |name: &str| match st.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    };
    let xs = match st.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect(),
        _ => Vec::new(),
    };
    (xs, scalar("Index"), scalar("id"))
}

/// **LIGADO, O `Index` É O POSTO** — e o `id` continua a viajar com a peça.
///
/// ⚠️ As duas metades são o desenho inteiro: renumerar sem preservar o `id` seria trocar um
/// apagão de ordem por um apagão de identidade, e o `value.instance_field(KeyBy::Id)`
/// perderia o elemento a que estava agarrado.
#[test]
fn the_renumbering_publishes_the_rank_and_keeps_the_durable_id() {
    let (xs, index, ids) = run(1.0, false);
    assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0], "ordenou por X");
    assert_eq!(
        index.expect("a lista traz Index"),
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        "o Index tem de ser o POSTO na lista ordenada, não o de nascimento"
    );
    // O nascimento foi 4,1,3,0,2 ⇒ ordenado por X a ordem de nascimento é 3,1,4,2,0.
    assert_eq!(
        ids.expect("a lista traz id"),
        vec![103.0, 101.0, 104.0, 102.0, 100.0],
        "o id é a identidade DURÁVEL: viaja com a peça, e é ele que fica embaralhado"
    );
}

/// **DESLIGADO, É EXACTAMENTE O QUE SEMPRE FOI** — o `Index` viaja com a peça.
///
/// ⚠️ Este é o gate que torna o default recuperável: se um dia a arte de alguém dependia da
/// pintura por identidade de montante, `reindex = 0` devolve-a ao bit.
#[test]
fn switching_it_off_restores_the_travelling_identity() {
    let (xs, index, ids) = run(0.0, false);
    assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0], "ordena na mesma");
    let index = index.expect("a lista traz Index");
    assert_eq!(
        index,
        vec![3.0, 1.0, 4.0, 2.0, 0.0],
        "desligado o Index é permutado como qualquer outra coluna"
    );
    // E ele anda AGARRADO ao id — os dois nasceram do mesmo elemento.
    let ids = ids.expect("a lista traz id");
    for (k, (i, d)) in index.iter().zip(&ids).enumerate() {
        assert!(
            (d - (100.0 + i)).abs() < 1e-6,
            "posição {k}: Index {i} e id {d} têm de vir do mesmo elemento"
        );
    }
}

/// **UMA LISTA SEM IDENTIDADE NÃO GANHA UMA** — a coluna ausente não é cunhada.
///
/// ⚠️ O porquê é medido e não estético: sem `Index` o `motion.tint` cai no seu próprio atalho
/// posicional `i/(n−1)`, que **já é** a ordem ordenada. Cunhar daria o mesmo número por outro
/// caminho, e uma coluna que não muda resposta nenhuma é peso que viaja o grafo abaixo.
#[test]
fn an_absent_identity_is_not_minted() {
    let (xs, index, ids) = run(1.0, true);
    assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0], "ordena na mesma");
    assert!(index.is_none(), "não cunha um Index que não existia");
    assert!(ids.is_none(), "nem nenhuma outra identidade");
}

/// **O DEFAULT É LIGADO**, e a régua é o MANIFESTO — não o número escrito aqui ao lado.
///
/// ⚠️ Um gate que repetisse o literal `1.0` provaria que eu sei copiar. O que se afirma é a
/// consequência: um `motion.sort` acabado de largar na tela, **sem ninguém tocar num knob**,
/// entrega o posto — que é a promessa do doc-comment do módulo.
#[test]
fn a_freshly_dropped_sort_already_renumbers() {
    let spec = MANIFEST
        .params
        .iter()
        .find(|p| p.name == REINDEX)
        .expect("o param existe");
    assert!(
        spec.default >= 0.5,
        "o default do reindex é o que o artista recebe sem tocar em nada: {}",
        spec.default
    );
    let mut g = Graph::new();
    let src = g.add_node("motion.sort.test.idsrc");
    let s = g.add_node("motion.sort");
    g.set_param(s, "key", KEY_X as f32);
    g.connect(Edge {
        from: (src, 0),
        to: (s, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, s, 0.0).expect("coze");
    match out[0].as_stream().get("Index") {
        Some(Column::Scalar(v)) => assert_eq!(
            *v,
            vec![0.0, 1.0, 2.0, 3.0, 4.0],
            "sem tocar num knob, o Index sai renumerado"
        ),
        other => panic!("Index: {other:?}"),
    }
}

/// **O KNOB APARECE NO PAINEL** — e com o widget que a decisão binária pede.
///
/// ⚠️ Um param sem `ParamUiHint` existe no cozimento e **não existe para o artista**. É o
/// mesmo defeito que o Enio apanhou no `corner` do `motion.shape` há dois dias, e a wave que
/// o fechou deixou o gate; esta deixa o dela.
#[test]
fn the_toggle_is_reachable_in_the_panel() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == REINDEX)
        .expect("o Reindex tem de estar pintado");
    assert!(
        matches!(hint.widget, ParamWidget::Toggle),
        "renumerar acontece ou não acontece — meio caminho não quer dizer nada"
    );
    // E o controle: nenhum gate o esconde (ele não é dead knob em chave nenhuma).
    assert!(
        !PARAM_GATES.iter().any(|g| g.param == REINDEX),
        "o Reindex vale em todas as chaves"
    );
}

/// **O `shift` ROTACIONA A ORDEM — e `0` é a permutação de sempre.**
///
/// ⚠️ **A metade que importa é «continua a ser uma PERMUTAÇÃO».** Um deslocamento
/// que empurrasse as pontas para fora perderia peças, e a saída deixaria de ter a
/// contagem da entrada — a coisa que um nó de ORDEM nunca pode fazer, e que um
/// teste de *"mudou alguma coisa?"* não apanha.
#[test]
fn the_shift_rotates_the_order_and_never_loses_a_piece() {
    let p = vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0], [4.0, 0.0]];
    let keys = keys(&p, KEY_X, [0.0, 0.0], 0, 0.0, &[]);
    let base = permutation(&keys, &[], false, 0);
    assert_eq!(base, vec![0, 1, 2, 3, 4], "shift 0 e' a ordem de sempre");
    assert_eq!(
        permutation(&keys, &[], false, 1),
        vec![1, 2, 3, 4, 0],
        "roda uma"
    );
    assert_eq!(
        permutation(&keys, &[], false, 5),
        base,
        "uma volta inteira e' identidade"
    );
    // ⚠️ Para trás: o `%` de Rust devolveria negativo e isto entraria em pânico.
    assert_eq!(
        permutation(&keys, &[], false, -1),
        vec![4, 0, 1, 2, 3],
        "roda ao contrario"
    );
    assert_eq!(
        permutation(&keys, &[], false, -7),
        permutation(&keys, &[], false, 3)
    );
    // E em TODO shift ela continua a ser uma permutação de `0..n`.
    for shift in -12i64..12 {
        let mut seen = permutation(&keys, &[], false, shift);
        seen.sort_unstable();
        assert_eq!(seen, vec![0, 1, 2, 3, 4], "shift {shift} perdeu ou repetiu");
    }
}

/// **Uma lista VAZIA não entra em pânico** — `rotate_left` sobre `0` elementos com
/// um `n` de zero seria uma divisão por zero no `rem_euclid`.
#[test]
fn an_empty_list_does_not_panic_on_a_shift() {
    assert!(permutation(&[], &[], false, 7).is_empty());
    assert!(permutation(&[], &[], true, -7).is_empty());
}
