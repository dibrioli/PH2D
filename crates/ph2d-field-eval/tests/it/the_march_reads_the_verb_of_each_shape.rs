//! ⛔⛔⛔ **O PASSO DA MARCHA PERGUNTAVA O FILETE DO GRUPO, e desde a W97 ele vive em CADA FORMA.**
//!
//! # O report
//!
//! Enio, 2026-08-29, com duas fotos do mesmo modelo em ângulos diferentes: *«4 formas juntas,
//! coloquei algum nível de joint em cada uma e ao rotacionar as áreas do joint mudam de aspecto.»*
//!
//! ⭐ **Um campo não muda com a câmera.** Se o *aspecto* muda ao rodar, quem muda é a **marcha** —
//! o passo é grande demais, o raio **atravessa** a superfície, e onde ele acerta passa a depender da
//! direcção. É exactamente o sintoma que o [`ph2d_field_eval::safe_march_step`] existe para não ter.
//!
//! # A causa, e ela é um DEFEITO QUE A W97 INTRODUZIU
//!
//! O tecto de `‖∇f‖` classifica um `Combine` por `op.blend()` — a mistura **do grupo**. Até à W97
//! isso era a única mistura que existia. A W97 pôs o **verbo em cada forma** (`Node::verb`), e a W98
//! pôs o **raio de junção** com ele: hoje o filete de cada passo da dobra sai do verbo **efectivo**
//! do filho (`combine_trees`: `fold_verb(parent, *verb)` → `op.blend()`).
//!
//! ⇒ com o grupo em `Sharp` e cada filho a trazer o seu `Exact`, a profundidade lia **zero**, o
//! passo ficava em **`1,0`**, e a peça furava nas juntas.
//!
//! ⚠️ *Quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0) —
//! e eu movi o filete de sítio sem re-perguntar o que esta lei media.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::{Field, gradient_bound, safe_march_step};

/// ⭐ **A PEÇA DO REPORT**: `n` formas irmãs num grupo só, o grupo **de aresta viva**, e cada forma
/// (menos a primeira, que semeia) com o **seu** raio de junção.
///
/// ⚠️ A primeira não leva verbo de propósito — é a lei da dobra (`fold_verb`), e pô-lo ali seria
/// testar uma peça que o produto não consegue autorar.
fn quatro_formas_com_junta_propria(n: usize, radius: f32) -> FieldDoc {
    let mut nodes: Vec<Node> = (0..n)
        .map(|i| {
            let mut leaf = Node::new(
                Xform {
                    translation: [0.34 * i as f32 - 0.5, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
                NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
            );
            if i > 0 {
                leaf.verb = Some(Op::Union(Blend::Exact { radius }));
            }
            leaf
        })
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            // ⚠️ **O GRUPO É `Sharp`** — é o caso que a lei antiga lia como «não infla nada».
            op: Op::Union(Blend::Sharp),
            children: (0..n).map(|i| NodeId(i as u32)).collect(),
        },
    ));
    let root = NodeId(n as u32);
    FieldDoc::new(nodes, root).expect("a peça do report")
}

fn worst_gradient(doc: &FieldDoc, e: f64, steps: usize) -> f64 {
    let f = Field::new(doc);
    let mut worst = 0.0f64;
    for i in 0..steps {
        for j in 0..steps {
            for k in 0..steps {
                let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
                let g = f.gradient_norm(at(i), at(j), at(k), 1.0e-4);
                if g.is_finite() {
                    worst = worst.max(g);
                }
            }
        }
    }
    worst
}

/// ⭐⭐⭐ **A LEI: `passo × ‖∇f‖ ≤ 1`, e ela vale com o filete NA FORMA.**
///
/// ⚠️ **Esta é a afirmação inteira do módulo sobre furar**, medida na peça do report. Um gate que só
/// olhasse a profundidade seria uma comparação de duas contas nossas; aqui um dos lados é o **campo**.
#[test]
fn the_march_never_outruns_a_joint_authored_on_the_shape() {
    for n in [2_usize, 3, 4, 5] {
        for radius in [0.05_f32, 0.15, 0.25] {
            let doc = quatro_formas_com_junta_propria(n, radius);
            let passo = f64::from(safe_march_step(&doc));
            let grad = worst_gradient(&doc, 0.9, 20);
            assert!(
                passo * grad <= 1.0,
                "{n} formas com junta {radius}: passo {passo:.4} × ‖∇f‖ {grad:.4} = {:.4} — acima de \
                 1 o raio ATRAVESSA a superfície, e onde ele acerta passa a depender da direcção. É \
                 o «ao rotacionar as áreas do joint mudam de aspecto» do report.",
                passo * grad
            );
        }
    }
}

/// ⭐⭐ **O tecto conta os passos da DOBRA que inflam, e não o verbo do grupo.**
///
/// ⚠️ **É a metade estrutural, e ela existe porque a de cima sozinha não localiza o defeito**: um
/// `passo × ‖∇f‖` vermelho diz que fura, não diz *onde a lei olhou*. Aqui afirma-se o número.
///
/// ⛔ E o **CONTROLE** é o que faz o gate valer: com os filhos calados (a herança) e o grupo em
/// `Sharp`, a peça de facto não infla, e a profundidade tem de ser **zero**. Sem esta metade, um
/// tecto que somasse `children.len()` sempre passaria a de cima e castigaria toda peça vulgar com
/// um passo duas vezes mais curto.
#[test]
fn the_bound_counts_the_folding_steps_that_inflate() {
    let doc = quatro_formas_com_junta_propria(4, 0.15);
    assert!(
        (gradient_bound(&doc) - 4.0f32.sqrt()).abs() < 1e-6,
        "quatro formas com filete somam `4`, e o tecto lido foi {}",
        gradient_bound(&doc)
    );

    // ⛔ O controle: os mesmos quatro irmãos, todos calados, sobre um grupo de aresta viva.
    let mut nodes: Vec<Node> = (0..4)
        .map(|i| {
            Node::new(
                Xform {
                    translation: [0.34 * i as f32 - 0.5, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
                NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
            )
        })
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: (0..4).map(NodeId).collect(),
        },
    ));
    let calado = FieldDoc::new(nodes, NodeId(4)).expect("a peça calada");
    assert!(
        (gradient_bound(&calado) - 1.0).abs() < 1e-6,
        "sem filete nenhum a peça não infla — castigá-la seria o caminho lento a definir o teto do \
         rápido; tecto lido {}",
        gradient_bound(&calado)
    );
}

/// ⭐ **E o verbo HERDADO do grupo continua a contar** — a forma calada usa o filete do pai.
///
/// ⚠️ Sem esta afirmação, uma cura que lesse **só** `node.verb` trocaria um defeito pelo simétrico:
/// o grupo com filete e os filhos calados voltaria a ler zero. *A lei é o verbo EFECTIVO
/// (`fold_verb`), nunca um dos dois lados.*
#[test]
fn an_inherited_joint_still_counts() {
    let mut nodes: Vec<Node> = (0..3)
        .map(|i| {
            Node::new(
                Xform {
                    translation: [0.34 * i as f32 - 0.34, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
                NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
            )
        })
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            // O filete vive no GRUPO, e os três filhos estão calados.
            op: Op::Union(Blend::Exact { radius: 0.15 }),
            children: (0..3).map(NodeId).collect(),
        },
    ));
    let doc = FieldDoc::new(nodes, NodeId(3)).expect("a peça herdada");
    assert!(
        (gradient_bound(&doc) - 3.0f32.sqrt()).abs() < 1e-6,
        "três formas herdando o filete do grupo somam `3`"
    );
    let passo = f64::from(safe_march_step(&doc));
    let grad = worst_gradient(&doc, 0.9, 20);
    assert!(passo * grad <= 1.0, "{passo:.4} × {grad:.4}");
}

/// ⭐⭐⭐ **A CAIXA DO MUNDO contém o que um filho ACRESCENTA contra o verbo do grupo** — o irmão
/// silencioso do defeito acima.
///
/// # ⛔ Ele foi achado a perguntar «quem MAIS lê a mistura do grupo?»
///
/// O `bounds::of_node` decidia como cada filho contribui olhando o `op` **do grupo**. Com o grupo em
/// `Difference` — *«o que se corta não acrescenta matéria, fica só o primeiro filho»* — um filho que
/// pede `Union` **acrescenta** e caía fora do bordo. ⇒ a peça sai **cortada** na parte que ele
/// junta, e nada explica porquê.
///
/// ⚠️ **A régua é o CAMPO**: procura-se um ponto que está **dentro** da peça e pergunta-se se ele
/// cabe na bola. Comparar duas contas nossas seria cego à mutação que mexesse nas duas.
#[test]
fn the_bounding_ball_holds_what_a_child_adds_against_the_group() {
    // Grupo em `Difference`; o 2.º filho corta (herda), o 3.º **junta** e fica longe.
    let mut nodes = vec![
        Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Sphere { radius: 0.35 }),
        ),
        Node::new(
            Xform {
                translation: [0.2, 0.0, 0.0],
                ..Xform::IDENTITY
            },
            NodeKind::Leaf(Primitive::Sphere { radius: 0.15 }),
        ),
        Node::new(
            Xform {
                translation: [1.1, 0.0, 0.0],
                ..Xform::IDENTITY
            },
            NodeKind::Leaf(Primitive::Sphere { radius: 0.25 }),
        ),
    ];
    nodes[2].verb = Some(Op::Union(Blend::Sharp));
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Difference(Blend::Sharp),
            children: (0..3).map(NodeId).collect(),
        },
    ));
    let doc = FieldDoc::new(nodes, NodeId(3)).expect("a peça mista");

    // O centro da 3.ª esfera está DENTRO da peça — é o que ela junta.
    let f = Field::new(&doc);
    assert!(
        f.at(1.1, 0.0, 0.0) < 0.0,
        "a 3.ª forma junta-se: o centro dela tem de estar dentro da peça"
    );
    let bola =
        ph2d_field_eval::bounds::bounding_ball(&doc, &ph2d_field_eval::hybrid::Registry::default())
            .expect("a peça tem bordo");
    let d = ((1.1 - f64::from(bola.center[0])).powi(2)
        + f64::from(bola.center[1]).powi(2)
        + f64::from(bola.center[2]).powi(2))
    .sqrt();
    assert!(
        d <= f64::from(bola.radius),
        "um ponto DENTRO da peça está a {d:.4} do centro do bordo, cujo raio é {:.4} — a caixa do \
         mundo corta o que o filho acrescenta, e o artista vê a peça mutilada sem explicação",
        bola.radius
    );
}

/// ⭐⭐ **Uma forma que se pronuncia por ARESTA VIVA não infla** — mesmo com o grupo a arredondar.
///
/// ⚠️ Este é o caso que uma cura preguiçosa (contar `children.len() − 1` sempre que o grupo tem
/// filete) leria errado, e o preço é o do §0: castigar a peça inteira pelo passo mais curto quando
/// metade das juntas é viva.
#[test]
fn a_shape_that_asks_for_a_sharp_joint_does_not_inflate_it() {
    let mut nodes: Vec<Node> = (0..3)
        .map(|i| {
            let mut leaf = Node::new(
                Xform {
                    translation: [0.34 * i as f32 - 0.34, 0.0, 0.0],
                    ..Xform::IDENTITY
                },
                NodeKind::Leaf(Primitive::Sphere { radius: 0.3 }),
            );
            // ⚠️ **Só o ÚLTIMO se pronuncia**, e por aresta viva: o do meio herda o filete do grupo.
            if i == 2 {
                leaf.verb = Some(Op::Union(Blend::Sharp));
            }
            leaf
        })
        .collect();
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Exact { radius: 0.15 }),
            children: (0..3).map(NodeId).collect(),
        },
    ));
    let doc = FieldDoc::new(nodes, NodeId(3)).expect("a peça mista");
    assert!(
        (gradient_bound(&doc) - 2.0f32.sqrt()).abs() < 1e-6,
        "dos dois passos de dobra só UM arredonda — o outro foi pedido vivo, e toma o MÁXIMO"
    );
}
