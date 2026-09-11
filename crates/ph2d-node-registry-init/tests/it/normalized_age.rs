//! **A IDADE NORMALIZADA — a célula da folha 15 envelheceu, e a medição a derruba.**
//!
//! A linha 97 da [folha 15](../../../docs/Motion%20Nodes/89_conferencia/15_value.md)
//! marca `value.time` / *idade normalizada* como **P1**, justificando assim:
//!
//! > **PARCIAL** — `value.attribute(Age) → value.map_range(0..vida → 0..1)`, **mas
//! > `life` não é coluna legível** ⇒ a vida é digitada à mão e desincroniza do
//! > `motion.emitter` em silêncio.
//!
//! ⚠️ **As duas metades da justificativa são FALSAS contra o código de hoje**, e é o
//! §0 do CLAUDE.md a morder em casa: *"fora de escopo porque é inalcançável" é uma
//! afirmação sobre um número que outra pessoa pode mudar*. O `motion.emitter`
//! escreve `life` como coluna — na CPU (`.with("life", …)`) **e** no device (a
//! `ColumnBinding` de saída) — e o `value.attribute` a oferece **no picker**, ao
//! lado de `Age`, desde que o canal de leitura existe. A cadeia é de **TRÊS** nós,
//! exata, e a vida vem do STREAM.
//!
//! ⚠️ **É a TERCEIRA célula desta folha a envelhecer antes de alguém voltar a ela**
//! (as outras duas estão registadas na linha 121), e o que se perde ao não
//! reconferir não é tempo: é **construir o que já existe**.
//!
//! ⚠️ **A metade que este gate mede não é *"a cadeia dá um número"*** — um número
//! sai de qualquer coisa. É que **mudar o `life` do emissor muda a resposta**, que é
//! precisamente o que uma vida digitada à mão *não* faz. Sem essa metade o gate
//! ficaria verde sobre uma constante.
//!
//! O que resta em aberto é **ergonomia** (três nós contra um canal do picker), e
//! isso é P2, não P1.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// `value.math` op 3 = Divide.
const OP_DIVIDE: f32 = 3.0;
const DT: f64 = 1.0 / 60.0;
/// Poucos tiques, de propósito: com `life` de 4 s e 8 s **ninguém morre** em
/// nenhuma das duas corridas, então a única diferença entre elas é o DIVISOR.
const TICKS: usize = 30;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, port: u16) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed: false,
    })
    .expect("edge");
}

/// A cadeia inteira que a folha diz não existir: emissor → (Age, Life) → Divide.
fn chain(life: f32) -> (Graph, NodeId, NodeId, NodeId) {
    let mut g = Graph::new();
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 20.0);
    g.set_param(em, "life", life);
    g.set_param(em, "speed", 0.0);

    let age = g.add_node("value.attribute");
    g.set_text_param(age, "attr", "age");
    wire(&mut g, em, age, 0);

    let span = g.add_node("value.attribute");
    g.set_text_param(span, "attr", "life");
    wire(&mut g, em, span, 0);

    let div = g.add_node("value.math");
    g.set_param(div, "op", OP_DIVIDE);
    wire(&mut g, age, div, 0);
    wire(&mut g, span, div, 1);
    (g, em, age, div)
}

/// A coluna `v` do nó terminal, depois de `TICKS` tiques.
fn cook_v(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut cook = Cook::new();
    for i in 0..TICKS {
        cook.advance_tick(g, reg, DT).expect("advance_tick");
        let t = (i + 1) as f64 * DT;
        cook.cook(g, reg, sink, t).expect("cook");
    }
    let out = cook
        .cook(g, reg, sink, TICKS as f64 * DT)
        .expect("cook final");
    match out[0].as_stream().get("v") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("o nó terminal não emitiu `v`"),
    }
}

fn scalar(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("sem coluna `{name}`"),
    }
}

/// **A idade normalizada é EXPRIMÍVEL, em três nós, e a vida vem do STREAM.**
#[test]
fn normalised_age_is_three_nodes_and_the_lifespan_comes_from_the_emitter() {
    let reg = registry();

    // --- a cadeia dá exactamente `age / life` -------------------------------
    let (g, em, _, div) = chain(4.0);
    let mut cook = Cook::new();
    for i in 0..TICKS {
        cook.advance_tick(&g, &reg, DT).expect("advance_tick");
        cook.cook(&g, &reg, div, (i + 1) as f64 * DT).expect("cook");
    }
    let emitted = cook
        .cook(&g, &reg, em, TICKS as f64 * DT)
        .expect("cook do emissor");
    let src = emitted[0].as_stream();
    let ages = scalar(src, "age");
    let lives = scalar(src, "life");
    let v = cook_v(&g, &reg, div);

    assert!(
        !v.is_empty() && v.len() == ages.len(),
        "fixture: o emissor tem de ter emitido algo ({} instâncias)",
        v.len()
    );
    // ⚠️ A fixture TEM de conter o fenômeno: com todas as idades iguais o quociente
    // seria constante e o gate não distinguiria a divisão de uma cópia.
    let (lo, hi) = v
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
    assert!(
        hi - lo > 0.05,
        "fixture: as idades têm de estar ESPALHADAS, e medem [{lo:.4}, {hi:.4}]"
    );
    for (i, (&a, &l)) in ages.iter().zip(&lives).enumerate() {
        let want = a / l;
        assert!(
            (v[i] - want).abs() < 1e-6,
            "instância {i}: idade {a} / vida {l} = {want}, a cadeia deu {}",
            v[i]
        );
    }
    assert!(
        (0.0..=1.0).contains(&lo) && (0.0..=1.0).contains(&hi),
        "uma idade normalizada vive em [0,1]: [{lo:.4}, {hi:.4}]"
    );

    // --- e DOBRAR a vida do emissor METADE o resultado ----------------------
    // Esta é a metade que a folha diz não existir. Um número digitado à mão daria
    // exactamente o mesmo campo aqui.
    let (g8, _, _, div8) = chain(8.0);
    let v8 = cook_v(&g8, &reg, div8);
    assert_eq!(
        v8.len(),
        v.len(),
        "ninguém morre em nenhuma das duas corridas"
    );
    for (i, (&a, &b)) in v.iter().zip(&v8).enumerate() {
        assert!(
            (b - a * 0.5).abs() < 1e-6,
            "instância {i}: dobrar a vida tem de METADE a fração ({a} -> {b})"
        );
    }
    eprintln!(
        "idade normalizada: vida 4 s -> [{lo:.4}, {hi:.4}] · vida 8 s -> metade exacta \
         em {} instâncias",
        v.len()
    );
}

/// **O `life` é uma coluna de PRIMEIRA CLASSE do emissor** — a premissa que a
/// célula nega, medida directamente em vez de inferida da cadeia acima.
#[test]
fn the_emitter_publishes_the_lifespan_as_a_readable_column() {
    let reg = registry();
    let (g, em, _, _) = chain(4.0);
    let mut cook = Cook::new();
    cook.advance_tick(&g, &reg, DT).expect("advance_tick");
    let out = cook.cook(&g, &reg, em, DT).expect("cook");
    let s = out[0].as_stream();
    let lives = scalar(s, "life");
    assert!(!lives.is_empty(), "o emissor emitiu instâncias");
    assert!(
        lives.iter().all(|&l| (l - 4.0).abs() < 1e-6),
        "a coluna `life` carrega o param do emissor: {lives:?}"
    );
    // E o `age` ao lado dela — as duas metades da fração.
    assert_eq!(scalar(s, "age").len(), lives.len());
}
