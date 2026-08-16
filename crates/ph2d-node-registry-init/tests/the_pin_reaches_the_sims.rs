//! **O PINO ALCANÇA AS SIMULAÇÕES — a cerca tinha uma porta que ninguém tentou.**
//!
//! O doc do `motion.pin_constraint` afirmava, sobre os três geradores:
//!
//! > *"são geradores (mintam os próprios pontos a partir de params e carregam
//! > estado; **nenhum stream de instâncias entra**), então os pinos intrínsecos
//! > deles ficam onde estão: **um pino a montante não tem fio por onde os
//! > alcançar**."*
//!
//! E o doc 34 §7 escreveu a mesma frase como cerca: *"hoje não há como um stream
//! entrar neles"*. ⚠️ **As duas eram verdade sobre a porta `in` e falsas sobre a
//! CADEIA DE ESTADO** — que é um fio, e que já era o fio pelo qual o `accel` entra
//! (`rope.out --pre--> force.wind --> rope.state` está documentado no próprio
//! `accel_col` da corda desde a wave do W1-A).
//!
//! Medido ANTES de uma linha ser escrita, com o pino no laço e os três nós a
//! ignorá-lo: **pior deslocamento `0,000000`** nos três. O vão era real e a cura
//! é **uma leitura de coluna** por nó — nenhum kernel novo, nenhuma porta nova.
//!
//! ## O CONTROLE é um pino que não seleciona ninguém
//!
//! `count = 0` faz o `motion.pin_constraint` escrever `inv_mass = [1,0; n]` — a
//! coluna EXISTE e o caminho de leitura RODA, com o valor neutro. Exigir que isso
//! seja **bit a bit** igual ao laço sem nó nenhum é mais forte que exigir que a
//! ausência não mude nada: prova o RAMO, não só o `else`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const TICKS: usize = 180;
const DT: f64 = 1.0 / 60.0;
/// O ponto da corda que o pino prega — no MEIO, onde nem a cabeça nem a cauda o
/// alcançam: é o *índice arbitrário* que a folha 03 (linha 51) pedia.
const ROPE_PIN: usize = 12;
const ROPE_COUNT: usize = 24;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("liga");
}

/// O que o pino faz na cadeia de estado, ou a ausência dele.
#[derive(Clone, Copy)]
enum Pin {
    /// `out --pre--> state`, sem nó no meio.
    None,
    /// Um `motion.pin_constraint` que seleciona `[first, first+count)` com
    /// `strength`. `count = 0` é o CONTROLE (escreve a coluna neutra).
    At {
        first: f32,
        count: f32,
        strength: f32,
    },
}

/// Fecha o laço de estado do nó `n` (porta `state` = índice 2 nos três), com ou
/// sem o pino no meio.
///
/// ⚠️ **O `pre` mora na aresta que ENTRA no pino**, não na que sai dele: é ela que
/// quebra o ciclo, e o `motion.pin_constraint` é `Effect::Pure` ⇒ não carimba
/// `sim_t`, então o solver ainda vê o `dt` do próprio relógio no tique seguinte.
fn close_loop(g: &mut Graph, node: NodeId, pin: Pin) {
    match pin {
        Pin::None => wire(g, node, 0, node, 2, true),
        Pin::At {
            first,
            count,
            strength,
        } => {
            let p = g.add_node("motion.pin_constraint");
            g.set_param(p, "first", first);
            g.set_param(p, "count", count);
            g.set_param(p, "strength", strength);
            wire(g, node, 0, p, 0, true);
            wire(g, p, 0, node, 2, false);
        }
    }
}

fn rope(pin: Pin) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let r = g.add_node("motion.verlet_rope");
    g.set_param(r, "count", ROPE_COUNT as f32);
    g.set_param(r, "length", 6.0);
    g.set_param(r, "gravity", 9.8);
    close_loop(&mut g, r, pin);
    (g, r)
}

fn body(pin: Pin) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let b = g.add_node("motion.soft_body");
    g.set_param(b, "rows", 6.0);
    g.set_param(b, "cols", 6.0);
    // ⚠️ O pino INTRÍNSECO desligado, senão a linha de topo já segura e o gate
    // ficaria verde sobre um corpo que nunca precisou do pino genérico.
    g.set_param(b, "pin", 0.0);
    g.set_param(b, "gravity", 9.8);
    close_loop(&mut g, b, pin);
    (g, b)
}

fn flock(pin: Pin) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let f = g.add_node("motion.boids");
    g.set_param(f, "count", 32.0);
    g.set_param(f, "seed", 7.0);
    close_loop(&mut g, f, pin);
    (g, f)
}

/// A pose no fim de `TICKS`. O `advance_tick` é o que faz a aresta `pre` CARREGAR
/// estado — sem ele a cadeia nunca integra e todo gate de feedback fica verde por
/// vácuo (a lição que o `rope_thickness` pagou duas vezes).
fn settle(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Stream {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avança");
        let out = cook.cook(g, reg, node, playhead).expect("coze");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saída é um stream")
        };
        last = s.clone();
    }
    last
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn pose(g: &Graph, reg: &NodeRegistry, n: NodeId) -> Vec<[f32; 2]> {
    positions(&settle(g, reg, n))
}

fn worst(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    assert_eq!(a.len(), b.len(), "as duas poses têm o mesmo tamanho");
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// A pose no tique `k` (para perguntar se um ponto SEGURA ao longo do tempo, e não
/// só onde ele acabou).
fn trail(g: &Graph, reg: &NodeRegistry, node: NodeId, i: usize) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut out = Vec::with_capacity(TICKS);
    for k in 0..TICKS {
        let playhead = k as f64 * DT;
        cook.advance_tick(g, reg, playhead).expect("o tique avança");
        let v = cook.cook(g, reg, node, playhead).expect("coze");
        let CookValue::Instances(s) = &v[0] else {
            panic!("stream")
        };
        out.push(positions(s)[i]);
    }
    out
}

// ── O CONTROLE ───────────────────────────────────────────────────────────────

/// **Um pino que não seleciona ninguém é BIT A BIT o laço nu.**
///
/// Prova o RAMO e não só o `else`: `count = 0` escreve `inv_mass = [1,0; n]`, a
/// coluna existe, o leitor roda, e a fórmula PBD `w/(w+w)` devolve `0,5` EXACTO —
/// `0/1`, `1/1` e `1/2` são todos representáveis. É por isso que a byte-identidade
/// desta wave é **aritmética, não promessa**.
#[test]
fn a_pin_that_selects_nothing_is_byte_identical() {
    let reg = registry();
    let neutral = Pin::At {
        first: 0.0,
        count: 0.0,
        strength: 1.0,
    };
    for (name, bare, with) in [
        ("corda", rope(Pin::None), rope(neutral)),
        ("corpo mole", body(Pin::None), body(neutral)),
        ("bando", flock(Pin::None), flock(neutral)),
    ] {
        let a = pose(&bare.0, &reg, bare.1);
        let b = pose(&with.0, &reg, with.1);
        assert!(!a.is_empty(), "{name}: a fixture tem pontos");
        assert_eq!(
            a.iter()
                .map(|p| [p[0].to_bits(), p[1].to_bits()])
                .collect::<Vec<_>>(),
            b.iter()
                .map(|p| [p[0].to_bits(), p[1].to_bits()])
                .collect::<Vec<_>>(),
            "{name}: o peso neutro tem de ser bit a bit o mundo de hoje",
        );
    }
}

// ── A CORDA (folha 03, linha 51) ─────────────────────────────────────────────

/// **O pino prega um ÍNDICE ARBITRÁRIO da corda — nem cabeça, nem cauda.**
///
/// A capacidade que a folha 03 (linha 51) pedia e que a cerca do doc 34 §7
/// declarava inalcançável.
#[test]
fn the_pin_nails_an_arbitrary_rope_point() {
    let reg = registry();
    let free = pose(&rope(Pin::None).0, &reg, rope(Pin::None).1);
    let (g, n) = rope(Pin::At {
        first: ROPE_PIN as f32,
        count: 1.0,
        strength: 1.0,
    });
    let held = pose(&g, &reg, n);

    // O CONTROLE: sem o pino aquele ponto CAI (senão o gate ficaria verde sobre
    // uma corda que nunca se moveu — o vácuo que o `rope_thickness` documentou).
    assert!(
        free[ROPE_PIN][1] < -1.0,
        "controle: sem pino o ponto {ROPE_PIN} pendura, e ele mediu y = {}",
        free[ROPE_PIN][1]
    );
    assert!(
        worst(&free, &held) > 1.0,
        "o pino tem de mover a corda; mediu {:.6}",
        worst(&free, &held)
    );

    // E ele SEGURA: a mesma posição em todo tique depois de o pino chegar (o `pre`
    // só carrega estado a partir do tique 1, então a régua começa no 2).
    let t = trail(&g, &reg, n, ROPE_PIN);
    let anchor_pos = t[2];
    for (k, p) in t.iter().enumerate().skip(2) {
        assert!(
            (p[0] - anchor_pos[0]).abs() < 1e-6 && (p[1] - anchor_pos[1]).abs() < 1e-6,
            "o ponto pinado tem de segurar; no tique {k} ele estava em {p:?} contra {anchor_pos:?}",
        );
    }
}

/// **Um pino PARCIAL é mais pesado, não pregado** — a metade do primitivo que um
/// booleano não teria.
#[test]
fn a_partial_pin_is_heavier_not_nailed() {
    let reg = registry();
    let at = |s: f32| {
        let (g, n) = rope(Pin::At {
            first: ROPE_PIN as f32,
            count: 1.0,
            strength: s,
        });
        pose(&g, &reg, n)[ROPE_PIN]
    };
    let (free, half, nailed) = (at(0.0), at(0.5), at(1.0));
    let drop = |p: [f32; 2]| p[1];
    assert!(
        drop(free) < drop(half) && drop(half) < drop(nailed),
        "a queda tem de ser monotónica em `strength`: livre {:.4} · meio {:.4} · pregado {:.4}",
        drop(free),
        drop(half),
        drop(nailed)
    );
}

// ── O CORPO MOLE ─────────────────────────────────────────────────────────────

/// **O pino segura uma partícula do corpo mole, e o corpo pendura dela.**
#[test]
fn the_pin_hangs_the_soft_body_from_a_particle() {
    let reg = registry();
    let (gf, nf) = body(Pin::None);
    let free = pose(&gf, &reg, nf);
    // A primeira linha da malha (6 partículas) — o corpo passa a pender dela.
    let (g, n) = body(Pin::At {
        first: 0.0,
        count: 6.0,
        strength: 1.0,
    });
    let held = pose(&g, &reg, n);

    assert!(
        free[0][1] < -5.0,
        "controle: sem pino o corpo CAI, e o elemento 0 mediu y = {:.4}",
        free[0][1]
    );
    assert!(
        held[0][1] > free[0][1] + 5.0,
        "o pino tem de segurar o corpo: livre {:.4} · pinado {:.4}",
        free[0][1],
        held[0][1]
    );
    // E as partículas LIVRES continuam a cair — um corpo que congelasse inteiro
    // seria um pino que virou um freeze.
    let last = held.len() - 1;
    assert!(
        held[last][1] < held[0][1] - 0.5,
        "o resto do corpo pende do pino: topo {:.4} · fundo {:.4}",
        held[0][1],
        held[last][1]
    );
}

// ── O BANDO ──────────────────────────────────────────────────────────────────

/// **Um agente de massa infinita segura, com velocidade zero — e o bando não
/// congela.**
///
/// ⚠️ *Ele continua a ser VISTO* é ESTRUTURAL e não tem gate próprio: o laço de
/// vizinhos lê `pos[j]`/`vel[j]` das arrays de ENTRADA, então nada no early-out o
/// remove da varredura dos outros. Não há segunda rota a divergir, logo não há o
/// que mutar — o fato fica escrito no `step`, ao lado do código que o garante.
#[test]
fn a_pinned_agent_holds_while_the_flock_flies() {
    let reg = registry();
    let (g, n) = flock(Pin::At {
        first: 0.0,
        count: 1.0,
        strength: 1.0,
    });
    let t = trail(&g, &reg, n, 0);
    let held = t[2];
    for (k, p) in t.iter().enumerate().skip(2) {
        assert!(
            (p[0] - held[0]).abs() < 1e-6 && (p[1] - held[1]).abs() < 1e-6,
            "o agente pinado segura; no tique {k} ele estava em {p:?} contra {held:?}",
        );
    }
    // O CONTROLE: o resto do bando continua a voar.
    let last = pose(&g, &reg, n);
    let free = pose(&flock(Pin::None).0, &reg, flock(Pin::None).1);
    let moved = last
        .iter()
        .skip(1)
        .zip(free.iter().skip(1))
        .map(|(a, b)| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    assert!(
        moved > 0.1,
        "um pino não pode congelar o bando inteiro; o maior desvio foi {moved:.6}"
    );
}

// ── A CÉLULA QUE ENVELHECEU (folha 03, linha 39) ─────────────────────────────

/// **O BANDO LÊ O QUE UMA FORÇA ESCREVE — a linha 39 fechou por MEDIÇÃO.**
///
/// A célula dizia: *"a cadeia É autorável e sai INERTE (…) **mas boids não lê
/// `accel`**, então a força é escrita e descartada"*. Ela era verdade quando foi
/// escrita e deixou de ser quando o W1-A fez os geradores consumirem `accel` —
/// *quem move o número que tornava algo inalcançável tem de reconferir a nota*.
///
/// O `force.curl` é divergence-free (Bridson), e o `max_speed` do bando ainda o
/// limita: é isto que o faz ler como o *wander* de Reynolds em vez de um empurrão.
#[test]
fn the_flock_reads_what_a_force_writes() {
    let reg = registry();
    let build = |curl: bool| {
        let mut g = Graph::new();
        let b = g.add_node("motion.boids");
        g.set_param(b, "count", 48.0);
        g.set_param(b, "seed", 7.0);
        if curl {
            let f = g.add_node("force.curl");
            g.set_param(f, "strength", 6.0);
            wire(&mut g, b, 0, f, 0, true);
            wire(&mut g, f, 0, b, 2, false);
        } else {
            wire(&mut g, b, 0, b, 2, true);
        }
        (g, b)
    };
    let (g0, n0) = build(false);
    let (g1, n1) = build(true);
    let calm = pose(&g0, &reg, n0);
    let wandering = pose(&g1, &reg, n1);
    assert!(
        worst(&calm, &wandering) > 1.0,
        "a força tem de alcançar o bando; o maior desvio foi {:.6}",
        worst(&calm, &wandering)
    );
}
