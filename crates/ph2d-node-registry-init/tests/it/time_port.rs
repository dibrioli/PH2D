//! **A PORTA DE TEMPO** — os gates do `SUPERAR 1` da folha 06, os três animadores de
//! uma vez (`motion.oscillator` · `motion.noise` · `motion.wiggle`).
//!
//! Uma porta `time` opcional, de tipo VALUE, em cada um deles: **desligada ⇒
//! `ctx.playhead()`**, ligada ⇒ um relógio **por elemento**. Os gates aqui provam as
//! duas metades, e a segunda é a razão de a primeira ser interessante:
//!
//! 1. **A AUSÊNCIA** — ligar um `value.time` neutro tem de dar **bit-a-bit** o mesmo
//!    que não ligar nada. Um teste que só verificasse *"anda"* passaria sobre uma
//!    porta que mudou o relógio de todos os grafos que já existem.
//! 2. **A PRESENÇA** — com `phase_stagger = 0` (sem porta, toda peça se move JUNTA)
//!    um campo de tempo defasado tem de partir a fileira numa onda que viaja. Este é
//!    o par presença+ausência: sem o controle, *"as peças diferem"* seria satisfeito
//!    por qualquer coisa.
//! 3. **O LOOP POR CONSTRUÇÃO** — `value.time → value.wrap → time` fecha o ciclo
//!    **exactamente** (`t` e `t+L` são o MESMO número que entra no nó), que é o que
//!    nenhum `loop_len` por-nó consegue: aquele é um cross-fade, e aproxima.
//! 4. **O BROADCAST 1→N** — a lei do `motion.drive` (doc 12), herdada e não
//!    reinventada.
//!
//! ⚠️ **A porta é APENDADA** (índice 1): as arestas de um documento salvo guardam o
//! índice, então a porta 0 continua a 0 e um doc de ontem abre igual.
//!
//! ⚠️ **Não é o escopo de tempo do `motion.time_remap`** (cerca 6 da folha): aquele
//! recozinha uma sub-árvore, e por isso **recusa** um nó sequencial a montante
//! (`CookError::SequentialInTimeScope`). Isto é uma COLUNA — não há segundo cozimento
//! e nada é recusado. O gate `the_time_port_is_a_column_not_a_cook_scope` prende isso.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantas peças a fileira tem. Ímpar não importa aqui; o que importa é serem
/// bastantes para uma onda que viaja ter mais de um valor distinto.
const N: usize = 8;

/// Os três nós que ganharam a porta, com o param que os faz mover o Y.
const ANIMATORS: &[(&str, &[(&str, f32)])] = &[
    (
        "motion.oscillator",
        &[
            ("channel", 1.0),
            ("amplitude", 1.0),
            ("frequency", 1.0),
            // ⚠️ ZERO de propósito: é o que faz a fileira mover-se JUNTA sem a porta,
            // e portanto o que torna a onda que viaja atribuível à porta e a mais nada.
            ("phase_stagger", 0.0),
        ],
    ),
    (
        "motion.noise",
        &[("channel", 1.0), ("amplitude", 1.0), ("speed", 1.0)],
    ),
    (
        "motion.wiggle",
        &[("channel", 1.0), ("amplitude", 1.0), ("frequency", 1.0)],
    ),
];

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
    .expect("wire");
}

/// Uma fileira de `N` peças.
fn row(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", N as f32);
    g.set_param(grid, "gap_x", 1.0);
    grid
}

/// O animador `ty` sobre a fileira, com os params da tabela.
fn animator(g: &mut Graph, src: NodeId, ty: &str, params: &[(&str, f32)]) -> NodeId {
    let n = g.add_node(ty);
    for (k, v) in params {
        g.set_param(n, *k, *v);
    }
    wire(g, src, n, 0);
    n
}

/// O `y` de cada peça, cozido em `playhead`.
fn ys(g: &Graph, reg: &NodeRegistry, sink: NodeId, playhead: f64) -> Vec<f32> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, playhead).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    }
}

/// Quantos valores DISTINTOS (a 5 casas) a fileira tem — 1 significa *toda peça
/// no mesmo sítio*.
fn distinct(v: &[f32]) -> usize {
    let mut k: Vec<i64> = v.iter().map(|y| (y * 1e5).round() as i64).collect();
    k.sort_unstable();
    k.dedup();
    k.len()
}

/// **A AUSÊNCIA — um relógio neutro na porta é BIT-A-BIT o playhead.**
///
/// ⚠️ O oráculo é o próprio catálogo: um `value.time` com `rate = 1`, `offset = 0`,
/// `stagger = 0` e a entrada DESLIGADA emite **um** valor, que é o playhead. Se ligar
/// isso à porta muda um único bit, então a porta reescreveu o relógio de todo grafo
/// que já existe — que é exactamente o risco desta wave.
///
/// A comparação é `==` sobre `f32`, não uma tolerância: *byte-idêntico* é a promessa
/// que a folha escreveu na coluna de risco, e uma barra com ε aceitaria uma promessa
/// mais fraca sem ninguém reparar.
#[test]
fn an_unconnected_time_port_is_bit_identical_to_a_neutral_clock() {
    let reg = registry();
    for (ty, params) in ANIMATORS {
        for playhead in [0.0, 0.37, 2.5] {
            let mut g = Graph::new();
            let src = row(&mut g);
            let bare = animator(&mut g, src, ty, params);

            let mut h = Graph::new();
            let src2 = row(&mut h);
            let fed = animator(&mut h, src2, ty, params);
            let clock = h.add_node("value.time"); // entrada desligada ⇒ UM valor = t
            wire(&mut h, clock, fed, 1);

            let a = ys(&g, &reg, bare, playhead);
            let b = ys(&h, &reg, fed, playhead);
            assert_eq!(
                a, b,
                "{ty} em t={playhead}: a porta neutra mudou o resultado\n  sem porta {a:?}\n  com porta {b:?}"
            );
        }
    }
}

/// **A PRESENÇA — um campo de tempo defasado parte a fileira numa onda que viaja.**
///
/// ⚠️ **O controle é a metade que dá sentido à outra.** Com `phase_stagger = 0` o
/// oscilador move toda peça JUNTA (um valor distinto na fileira); é só isso que faz
/// *"agora há N valores distintos"* significar *o relógio virou um campo*, em vez de
/// significar *alguma coisa mexeu*.
///
/// O `motion.noise` e o `motion.wiggle` já variam por elemento sem porta (um por
/// POSIÇÃO, o outro por ÍNDICE — cerca 5 da folha), então para eles o controle é
/// outro: a fileira com a porta tem de **diferir** da fileira sem ela.
#[test]
fn a_staggered_time_field_gives_each_element_its_own_clock() {
    let reg = registry();
    for (ty, params) in ANIMATORS {
        let mut g = Graph::new();
        let src = row(&mut g);
        let bare = animator(&mut g, src, ty, params);
        let plain = ys(&g, &reg, bare, 1.0);

        let mut h = Graph::new();
        let src2 = row(&mut h);
        let fed = animator(&mut h, src2, ty, params);
        // `value.time` COM entrada ⇒ N valores, `t + i·stagger`.
        let clock = h.add_node("value.time");
        h.set_param(clock, "stagger", 0.25);
        wire(&mut h, src2, clock, 0);
        wire(&mut h, clock, fed, 1);
        let field = ys(&h, &reg, fed, 1.0);

        assert_eq!(field.len(), N, "{ty}: a largura e' a da porta 0");
        assert!(
            distinct(&field) > 1,
            "{ty}: com um campo de tempo as pecas tem de diferir, e ha' {} valor(es)",
            distinct(&field)
        );
        assert_ne!(
            plain, field,
            "{ty}: o campo de tempo nao mudou nada em relacao ao relogio global"
        );
    }
    // E o CONTROLE do oscilador, que é o único cuja fileira era UNÍSSONA sem a porta.
    let reg = registry();
    let mut g = Graph::new();
    let src = row(&mut g);
    let osc = animator(&mut g, src, ANIMATORS[0].0, ANIMATORS[0].1);
    assert_eq!(
        distinct(&ys(&g, &reg, osc, 1.0)),
        1,
        "sem porta e com phase_stagger = 0 a fileira tem de mover-se JUNTA -- \
         sem isto, o gate acima nao prova que foi o relogio que virou campo"
    );
}

/// **O LOOP POR CONSTRUÇÃO — `t` e `t+L` são o MESMO número, não dois parecidos.**
///
/// A cadeia `value.time → value.wrap(0..L, Repeat) → time` fecha o ciclo por
/// construção. ⚠️ É isto que um `loop_len` por-nó **não** consegue: aquele mistura a
/// amostra em `t` com a de `t−L` por um peso smoothstep (a lei do `motion.noise`,
/// cerca 4) e por isso *aproxima* — aqui o relógio que entra no nó é literalmente
/// igual, então a saída é `==`.
///
/// ⚠️ A barra é a igualdade EXACTA e o gate corre nos três nós, inclusive nos dois
/// que já têm `loop_len`: a comparação é entre o mecanismo novo e ele mesmo um
/// período depois, não entre os dois mecanismos.
#[test]
fn a_wrapped_ramp_closes_the_cycle_exactly_where_a_cross_fade_only_approximates() {
    let reg = registry();
    const L: f32 = 2.0;
    // ⚠️ `L · frequency` NÃO pode ser inteiro, senão o nó já repetiria sozinho em `L`
    // e o gate ficaria verde sobre um wrap que não fez nada (a primeira versão pediu
    // `frequency = 1` e o controle mediu `[0,…] == [0,…]`: a onda tinha dado voltas
    // exactas). Com 0,3 ciclos por segundo, `L` são 0,6 de ciclo.
    const FREQ: f32 = 0.3;
    for (ty, params) in ANIMATORS {
        // A mesma cena duas vezes: uma com o wrap, outra sem — a ÚNICA diferença.
        let build = |wrapped: bool| {
            let mut g = Graph::new();
            let src = row(&mut g);
            let node = animator(&mut g, src, ty, params);
            g.set_param(node, "frequency", FREQ);
            let clock = g.add_node("value.time");
            wire(&mut g, src, clock, 0);
            let head = if wrapped {
                let wrap = g.add_node("value.wrap");
                g.set_param(wrap, "lo", 0.0);
                g.set_param(wrap, "hi", L);
                // ⚠️ `Repeat` é **1** — o `0` é `Clamp`, e com ele o gate reprovava
                // sobre produto correto (o relógio passava inteiro, sem dobrar).
                g.set_param(wrap, "mode", 1.0);
                wire(&mut g, clock, wrap, 0);
                wrap
            } else {
                clock
            };
            wire(&mut g, head, node, 1);
            (g, node)
        };

        let (g, node) = build(true);
        let a = ys(&g, &reg, node, 0.5);
        let b = ys(&g, &reg, node, 0.5 + f64::from(L));
        assert_eq!(
            a, b,
            "{ty}: um relogio wrapado tem de fechar o ciclo EXACTAMENTE\n  t     {a:?}\n  t+L   {b:?}"
        );
        // CONTROLE: a MESMA cena sem o wrap tem de mudar em `t+L` — senão a igualdade
        // acima seria a de um nó que ignora o relógio, e não a de um ciclo fechado.
        let (h, bare) = build(false);
        assert_ne!(
            ys(&h, &reg, bare, 0.5),
            ys(&h, &reg, bare, 0.5 + f64::from(L)),
            "{ty}: o controle SEM wrap tinha de mudar em t+L"
        );
    }
}

/// **O BROADCAST 1→N — um relógio vale para toda instância** (a lei do `motion.drive`,
/// doc 12), e a fileira volta a mover-se em uníssono.
#[test]
fn a_single_clock_value_is_held_across_every_element() {
    let reg = registry();
    let (ty, params) = ANIMATORS[0];
    let mut g = Graph::new();
    let src = row(&mut g);
    let osc = animator(&mut g, src, ty, params);
    // Sem entrada, o `value.time` emite UM valor — e `rate = 2` prova que é o valor
    // dele que chega, não o playhead por outro caminho.
    let clock = g.add_node("value.time");
    g.set_param(clock, "rate", 2.0);
    wire(&mut g, clock, osc, 1);

    let held = ys(&g, &reg, osc, 0.3);
    assert_eq!(held.len(), N);
    assert_eq!(distinct(&held), 1, "um valor tem de valer para toda peca");

    // E ele é o relógio DELE: `rate = 2` em t = 0,3 é o mesmo que o playhead a 0,6.
    let mut h = Graph::new();
    let src2 = row(&mut h);
    let bare = animator(&mut h, src2, ty, params);
    assert_eq!(
        held,
        ys(&h, &reg, bare, 0.6),
        "o relogio da porta e' o do `value.time`, nao o playhead"
    );
}

/// **A porta é uma COLUNA, não um escopo de cook** — a resposta à cerca 6 da folha.
///
/// Um `motion.time_remap` a montante de um nó SEQUENCIAL é recusado pelo cook
/// (`CookError::SequentialInTimeScope`), porque um escopo **recozinha** a sub-árvore.
/// A porta de tempo não recozinha nada: ela entrega um número por elemento. O gate
/// prova que a mesma vizinhança que o escopo recusaria coze sem erro pela porta.
#[test]
fn the_time_port_is_a_column_not_a_cook_scope() {
    let reg = registry();
    let mut g = Graph::new();
    let src = row(&mut g);
    // Um SEQUENCIAL (`motion.spring`) entre a fonte e o animador — a exacta vizinhança
    // que faz o escopo de tempo recusar.
    let spring = g.add_node("motion.spring");
    wire(&mut g, src, spring, 0);
    let osc = animator(&mut g, spring, ANIMATORS[0].0, ANIMATORS[0].1);
    let clock = g.add_node("value.time");
    g.set_param(clock, "stagger", 0.1);
    wire(&mut g, src, clock, 0);
    wire(&mut g, clock, osc, 1);

    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, osc, 0.5);
    assert!(
        out.is_ok(),
        "a porta de tempo nao pode herdar a recusa do ESCOPO de tempo: {:?}",
        out.err()
    );
}
