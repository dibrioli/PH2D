//! ⭐⭐⭐ **A DOBRA COM UM VERBO POR FORMA** (W97) — os gates da lei de [`ph2d_field::fold_verb`].
//!
//! ⚠️ **Eles medem o CAMPO, não a estrutura.** Um gate que afirmasse *«o nó guardou `Some(op)`»*
//! passaria com o avaliador a ignorar o campo inteiro — é o defeito clássico de medir o trabalho
//! FEITO em vez do ENTREGUE. Aqui pergunta-se sempre a um **ponto do espaço** quanto ele vale.
//!
//! ⚠️ E cada um corre pelas **duas rotas**, porque a lei está escrita duas vezes de propósito (a
//! árvore de `fidget` e os números do [`crate::hybrid`], com o
//! `the_numeric_law_is_the_same_law_as_the_tree` a arbitrá-las). Um verbo honrado só numa delas dá
//! uma peça que **muda de forma ao entrar uma escultura**.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

/// Uma esfera de raio `r` centrada em `x`.
fn ball(x: f32, r: f32) -> Node {
    Node::new(
        Xform::at(x, 0.0, 0.0),
        NodeKind::Leaf(Primitive::Sphere { radius: r }),
    )
}

/// Uma peça de três esferas em fila, com o verbo de cada filho autorado.
///
/// ⚠️ `verbs[0]` entra no documento na mesma: é o que prova que ele **não é usado**.
fn three(parent: Op, verbs: [Option<Op>; 3]) -> FieldDoc {
    let mut nodes = vec![ball(-0.3, 0.5), ball(0.0, 0.5), ball(0.3, 0.5)];
    for (n, v) in nodes.iter_mut().zip(verbs) {
        n.verb = v;
    }
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: parent,
            children: vec![NodeId(0), NodeId(1), NodeId(2)],
        },
    ));
    FieldDoc::new(nodes, NodeId(3)).expect("documento válido")
}

/// O valor do campo num ponto — **pelas duas rotas**, e elas têm de concordar.
///
/// ⚠️ A lei da booleana está escrita duas vezes de propósito (a árvore de `fidget` e os números do
/// [`crate::hybrid`]): um verbo honrado só numa delas dá uma peça que **muda de forma no dia em que
/// entrar uma escultura**. Perguntar às duas aqui torna cada gate abaixo um gate das duas.
fn at(doc: &FieldDoc, p: [f32; 3]) -> f32 {
    let arvore =
        crate::Field::new(doc).at(f64::from(p[0]), f64::from(p[1]), f64::from(p[2])) as f32;
    let mut h = crate::hybrid::Hybrid::new(doc, &crate::hybrid::Registry::default());
    let numeros = h.eval(&[p[0]], &[p[1]], &[p[2]]).expect("avaliou")[0];
    assert!(
        (arvore - numeros).abs() < 1e-5,
        "as DUAS leis discordam em {p:?}: árvore {arvore}, números {numeros}"
    );
    numeros
}

/// ⭐⭐⭐ **Ninguém se pronunciou ⇒ o campo é o da booleana de sempre.**
///
/// É a promessa que torna a mudança segura: *ausência é herança*. Se ela falhasse, toda peça já
/// autorada mudaria de forma ao abrir.
///
/// # ⚠️ O controlo é uma CONSTRUÇÃO independente, e não o mesmo documento duas vezes
///
/// Comparar `three(op, [None; 3])` consigo próprio seria uma tautologia — ela passa com a dobra
/// inteira apagada. O controlo aqui é a árvore **aninhada de dois filhos** `((a − b) − c)`, que é
/// como a mesma peça se exprimia antes desta wave e **não usa o campo novo em lado nenhum**. Se a
/// dobra N-ária com silêncio deixar de valer a cadeia binária, é aqui que se vê.
#[test]
fn silence_is_the_boolean_of_always() {
    let op = Op::Difference(Blend::Exact { radius: 0.1 });
    let n_aria = three(op, [None; 3]);
    let aninhada = {
        let nodes = vec![
            ball(-0.3, 0.5),
            ball(0.0, 0.5),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op,
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
            ball(0.3, 0.5),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op,
                    children: vec![NodeId(2), NodeId(3)],
                },
            ),
        ];
        FieldDoc::new(nodes, NodeId(4)).expect("documento válido")
    };
    for p in pontos() {
        let (a, b) = (at(&n_aria, p), at(&aninhada, p));
        assert!(
            (a - b).abs() < 1e-5,
            "o silêncio deixou de ser a booleana de sempre em {p:?}: {a} contra {b}"
        );
    }
}

/// ⭐⭐ **Um verbo PRÓPRIO muda o campo** — e é a metade que prova que ele é lido.
///
/// ⚠️ O ponto é escolhido dentro da terceira esfera e **fora** das outras duas: com ela unida o
/// campo é negativo (dentro da peça); com ela subtraída é positivo (fora).
#[test]
fn a_shape_that_speaks_folds_with_its_own_verb() {
    let unida = three(Op::Union(Blend::Sharp), [None; 3]);
    let subtraida = three(
        Op::Union(Blend::Sharp),
        [None, None, Some(Op::Difference(Blend::Sharp))],
    );
    let dentro_da_terceira = [0.7, 0.0, 0.0];
    assert!(
        at(&unida, dentro_da_terceira) < 0.0,
        "controlo: com tudo unido aquele ponto está DENTRO da peça"
    );
    assert!(
        at(&subtraida, dentro_da_terceira) > 0.0,
        "a terceira forma pediu para subtrair e continuou a somar"
    );
}

/// ⭐⭐⭐ **A dobra é sobre o ACUMULADO, não sobre a base.**
///
/// ⚠️ É a diferença que nenhum caso de duas formas consegue ver, e é o coração da receita: com três
/// formas, `((a ∪ b) − c)` e `(a ∪ b) ∪ (a − c)` respondem diferente **exactamente** onde `c` cobre
/// `b` e não cobre `a`. Um avaliador que dobrasse cada filho contra `children[0]` passaria em todo
/// gate de duas formas.
#[test]
fn each_shape_folds_over_the_accumulated_not_over_the_base() {
    // `c` colocada por cima de `b` e longe de `a`.
    let mut nodes = vec![ball(-1.2, 0.4), ball(0.0, 0.4), ball(0.05, 0.4)];
    nodes[2].verb = Some(Op::Difference(Blend::Sharp));
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(0), NodeId(1), NodeId(2)],
        },
    ));
    let doc = FieldDoc::new(nodes, NodeId(3)).expect("documento válido");
    // Um ponto no coração de `b`, que `c` cobre: se `c` tivesse sido subtraída **da base** (`a`,
    // que está longe), este ponto continuaria dentro.
    assert!(
        at(&doc, [0.0, 0.0, 0.0]) > 0.0,
        "a subtracção não mordeu o acumulado — ela foi aplicada à base"
    );
    // E o coração de `a` continua dentro: `c` não chega lá.
    assert!(
        at(&doc, [-1.2, 0.0, 0.0]) < 0.0,
        "a subtracção comeu uma forma que ela não toca"
    );
}

/// ⭐⭐ **A ORDEM decide** — as mesmas três formas, os mesmos verbos, trocadas de lugar, dão outra
/// peça.
///
/// Sem isto a «receita» seria um conjunto, e a Hierarquia deixaria de a exprimir.
#[test]
fn the_order_is_the_recipe() {
    let corta_no_fim = three(
        Op::Union(Blend::Sharp),
        [None, None, Some(Op::Difference(Blend::Sharp))],
    );
    // A MESMA forma que corta, agora no meio: ela come só o que veio antes dela.
    let corta_no_meio = {
        let mut nodes = vec![ball(-0.3, 0.5), ball(0.3, 0.5), ball(0.0, 0.5)];
        nodes[1].verb = Some(Op::Difference(Blend::Sharp));
        nodes.push(Node::new(
            Xform::IDENTITY,
            NodeKind::Combine {
                op: Op::Union(Blend::Sharp),
                children: vec![NodeId(0), NodeId(1), NodeId(2)],
            },
        ));
        FieldDoc::new(nodes, NodeId(3)).expect("documento válido")
    };
    let diferentes = pontos()
        .into_iter()
        .any(|p| (at(&corta_no_fim, p) - at(&corta_no_meio, p)).abs() > 1e-4);
    assert!(
        diferentes,
        "trocar a ordem não mudou nada — a dobra não está a respeitar a sequência"
    );
}

/// ⭐⭐⭐ **O verbo do PRIMEIRO filho não é usado** — ele semeia o acumulado.
///
/// ⚠️ É o gate que impede a leitura *«o acumulado começa vazio»*: com ela, uma subtração no topo
/// apagaria a peça inteira (`∅ − a = ∅`), e reordenar destruiria o modelo em silêncio.
#[test]
fn the_first_shapes_verb_is_never_asked() {
    let calada = three(Op::Union(Blend::Sharp), [None; 3]);
    for verbo in [
        Op::Difference(Blend::Sharp),
        Op::Intersection(Blend::Sharp),
        Op::Union(Blend::Exact { radius: 0.3 }),
    ] {
        let falante = three(Op::Union(Blend::Sharp), [Some(verbo), None, None]);
        for p in pontos() {
            assert_eq!(
                at(&calada, p).to_bits(),
                at(&falante, p).to_bits(),
                "o verbo da BASE mudou o campo em {p:?} — ele não devia ser perguntado"
            );
        }
    }
}

/// ⭐⭐⭐ **A MISTURA viaja com o verbo** — é isto que faz «um raio por objeto» existir.
///
/// ⚠️ Duas formas com o mesmo verbo e raios diferentes **têm de** produzir junções diferentes. Antes
/// desta wave o raio era do grupo, e a única forma de o exprimir era aninhar.
/// ⚠️ **A fixtura tem de ser SEPARÁVEL**, e a primeira versão desta não era: com as esferas a
/// `0,3` de distância e raio `0,5` a peça é um blob só, e um filete de `0,25` na junção da esquerda
/// **alcança** o vale da direita — o «controlo» media o fenómeno que devia excluir. Espaçadas a
/// `0,9` cada par tem o seu vale, e o campo da esfera distante vale `0,88` no vale oposto, muito
/// fora do alcance do filete.
fn three_spread(pai: Op, verbo_do_meio: Option<Op>) -> FieldDoc {
    let mut nodes = vec![ball(-0.9, 0.5), ball(0.0, 0.5), ball(0.9, 0.5)];
    nodes[1].verb = verbo_do_meio;
    nodes.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: pai,
            children: vec![NodeId(0), NodeId(1), NodeId(2)],
        },
    ));
    FieldDoc::new(nodes, NodeId(3)).expect("documento válido")
}

/// ⭐⭐⭐ **HERDAR é herdar o verbo E a mistura DO PAI** — e não um verbo qualquer.
///
/// # ⚠️ Este gate nasceu de uma MUTAÇÃO QUE SOBREVIVEU
///
/// Trocar `child.unwrap_or(parent)` por `child.unwrap_or(Op::Union(Blend::Sharp))` — isto é,
/// apagar a herança inteira — passou em **todos** os outros gates deste arquivo. A causa não foi
/// falta de cobertura: eles **comparam duas construções**, e a mutação afectava as duas da mesma
/// maneira. *Um controlo que partilha o defeito do sujeito não é um controlo.*
///
/// ⇒ A cura é medir contra um **oráculo**, não contra um irmão: com o pai a **subtrair**, o coração
/// da segunda forma está FORA da peça — e isso é verdade ou falso sozinho.
#[test]
fn inheriting_means_the_parents_verb_and_the_parents_blend() {
    // ── O VERBO do pai, contra um oráculo ──
    let corta = three_spread(Op::Difference(Blend::Sharp), None);
    assert!(
        at(&corta, [0.0, 0.0, 0.0]) > 0.0,
        "o pai subtrai e ninguém se pronunciou: o coração da 2.ª forma tinha de estar FORA da peça"
    );
    assert!(
        at(&corta, [-0.9, 0.0, 0.0]) < 0.0,
        "controlo: a BASE não é subtraída de nada — o coração dela está dentro"
    );

    // ── A MISTURA do pai: herdar `Union(Exact)` não pode dar `Union(Sharp)` ──
    let vivo = three_spread(Op::Union(Blend::Sharp), None);
    let gordo = three_spread(Op::Union(Blend::Exact { radius: 0.25 }), None);
    let vale = [-0.45, 0.3, 0.0];
    let (a, b) = (at(&vivo, vale), at(&gordo, vale));
    assert!(
        b < a - 1e-4,
        "o filete do PAI não foi herdado: viva {a:.5}, com filete {b:.5}"
    );
}

#[test]
fn each_shape_carries_the_radius_of_its_own_joint() {
    let vivo = three_spread(Op::Union(Blend::Sharp), Some(Op::Union(Blend::Sharp)));
    let gordo = three_spread(
        Op::Union(Blend::Sharp),
        Some(Op::Union(Blend::Exact { radius: 0.25 })),
    );
    // No vale entre a 1.ª e a 2.ª esfera — a junção que a forma do meio faz: com filete o campo é
    // MENOS positivo ali (a matéria do filete aproxima-se do ponto).
    let vale = [-0.45, 0.3, 0.0];
    let a = at(&vivo, vale);
    let b = at(&gordo, vale);
    assert!(
        b < a - 1e-4,
        "o raio de junção da forma não foi honrado: viva {a:.5}, com filete {b:.5}"
    );
    // ⚠️ E o CONTROLO: a 3.ª forma não pediu filete nenhum e dobra com o verbo do PAI (vivo), então
    // o vale do outro lado fica como estava. Sem ele, um filete aplicado ao grupo inteiro passaria
    // neste gate inteiro.
    let outro_vale = [0.45, 0.3, 0.0];
    let (c, d) = (at(&vivo, outro_vale), at(&gordo, outro_vale));
    assert!(
        (c - d).abs() < 1e-6,
        "o filete de UMA forma vazou para a junção de outra: {c:.6} contra {d:.6}"
    );
}

/// Uma grelha de pontos que atravessa a peça inteira e o espaço à volta.
fn pontos() -> Vec<[f32; 3]> {
    let mut v = Vec::new();
    for i in -6i8..=6 {
        for j in -3i8..=3 {
            v.push([f32::from(i) * 0.2, f32::from(j) * 0.2, 0.0]);
        }
    }
    v
}
