//! **SONDA — o `value.switch` de N entradas já é exprimível por composição?**
//!
//! A folha 15 fecha com **um** `P1`: *"`switch`: N entradas (⚠️ contrato)"*, e a justificativa é
//! que `NodeManifest.inputs` é `&'static [PortSpec]` — logo *"fora de qualquer wave de params"*.
//! A afirmação sobre o contrato é **verdade** e não é a pergunta: um contrato congelado diz o que
//! não se pode mudar **naquele nó**, não o que o catálogo não consegue exprimir (§5.0 do
//! `CLAUDE.md`: *antes de construir um item de lista aberta, MEÇA se a composição já o exprime*).
//!
//! O que a composição tem a favor está escrito no próprio nó: o índice é
//! **`clamp(round(select), 0, N−1)`**, e o `select` é uma **porta**, não um param. Um clamp é uma
//! saturação — então um switch encadeado, alimentado por um `select` DESLOCADO, escolhe a
//! sub-árvore de baixo em todo o intervalo que não é dele. A conta é:
//!
//! ```text
//! interior:  switch(select,      in0, in1, in2, in3)
//! exterior:  switch(select − 3,  <interior>, in4, in5, in6)
//! ```
//!
//! `select ∈ 0..3` ⇒ `select − 3 ∈ −3..0` ⇒ **clampa a 0** ⇒ o exterior devolve o interior, que
//! escolhe certo. `select = 4|5|6` ⇒ `1|2|3` ⇒ as entradas novas. **Sete entradas.**
//!
//! E o deslocamento não custa dois nós: uma `value.map_range` afim (`0..1 → −3..−2`) é
//! exactamente `v − 3` num só nó, sem precisar de uma constante ao lado.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_switch_arity -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantas entradas um `value.switch` tem hoje.
const FANIN: usize = 4;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .expect("wire");
}

/// A geometria que dá a CONTAGEM ao domínio de valor — um ponto.
///
/// ⚠️ **Sem ela toda constante sai VAZIA e a sonda mede nada:** o `value.pattern` lê a porta `in`
/// (`INST_VEC2`) *só pela contagem*, e desligada ela é zero. A 1ª versão desta sonda errou os
/// quatro índices de um switch SOZINHO — o defeito estava na fixture, não na composição.
fn one_point(g: &mut Graph) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", 1.0);
    g.set_param(n, "cols", 1.0);
    n
}

/// Uma CONSTANTE de valor num nó — a cadeia que a folha 17 já mediu
/// (`value.pattern` com `steps = 1`), pendurada na geometria que dá a contagem.
fn constant(g: &mut Graph, seed: NodeId, k: f32) -> NodeId {
    let n = g.add_node("value.pattern");
    g.set_param(n, "steps", 1.0);
    g.set_param(n, "v0", k);
    wire(g, seed, 0, n, 0);
    n
}

/// `v − k` num nó só: a `value.map_range` afim `0..1 → −k..(1−k)` é exactamente isso.
fn minus(g: &mut Graph, src: NodeId, k: f32) -> NodeId {
    let n = g.add_node("value.map_range");
    g.set_param(n, "in_lo", 0.0);
    g.set_param(n, "in_hi", 1.0);
    g.set_param(n, "out_lo", -k);
    g.set_param(n, "out_hi", 1.0 - k);
    g.set_param(n, "clamp", 0.0);
    wire(g, src, 0, n, 0);
    n
}

/// Um mux de `sources.len()` entradas, montado por encadeamento. Devolve `(saída, nós gastos)`.
fn wide_mux(g: &mut Graph, select: NodeId, sources: &[NodeId]) -> (NodeId, usize) {
    let mut cost = 0usize;
    let mut acc: Option<NodeId> = None;
    let mut taken = 0usize;
    while taken < sources.len() {
        let sw = g.add_node("value.switch");
        cost += 1;
        // O primeiro bloco usa as 4 portas; os seguintes gastam a `in0` com o acumulado.
        let (sel, first_port, room) = match acc {
            None => (select, 1u16, FANIN),
            Some(prev) => {
                let shifted = minus(g, select, (taken - 1) as f32);
                cost += 1;
                wire(g, prev, 0, sw, 1);
                (shifted, 2u16, FANIN - 1)
            }
        };
        wire(g, sel, 0, sw, 0);
        for k in 0..room.min(sources.len() - taken) {
            wire(g, sources[taken + k], 0, sw, first_port + k as u16);
        }
        taken += room.min(sources.len() - taken);
        acc = Some(sw);
    }
    (acc.expect("pelo menos um bloco"), cost)
}

fn read(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> f32 {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    match s.get("v") {
        Some(Column::Scalar(v)) if !v.is_empty() => v[0],
        _ => f32::NAN,
    }
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn how_wide_can_a_switch_get_by_composition() {
    let reg = registry();
    eprintln!("\n[switch-arity] um mux de N entradas montado por ENCADEAMENTO");
    eprintln!("  (a fonte `k` vale `100 + k`, entao o valor lido DIZ qual entrada respondeu)\n");
    for n in [4usize, 7, 10, 13] {
        let mut hits = String::new();
        let mut cost = 0usize;
        let mut wrong = 0usize;
        for want in 0..n {
            let mut g = Graph::new();
            let seed = one_point(&mut g);
            let select = constant(&mut g, seed, want as f32);
            let sources: Vec<NodeId> = (0..n)
                .map(|k| constant(&mut g, seed, 100.0 + k as f32))
                .collect();
            let (sink, c) = wide_mux(&mut g, select, &sources);
            cost = c;
            g.validate(&reg).expect("bem-tipado");
            let got = read(&g, &reg, sink);
            let ok = (got - (100.0 + want as f32)).abs() < 1e-6;
            if !ok {
                wrong += 1;
            }
            hits.push(if ok { '.' } else { 'X' });
        }
        eprintln!(
            "  N = {n:>3}  nos gastos {cost:>2} (contra 1 se o no' tivesse N portas)  \
             select 0..{}: {hits}  errados: {wrong}",
            n - 1
        );
    }

    eprintln!(
        "\n  LEITURA: um `.` por indice quer dizer que a entrada CERTA respondeu. Se todos os
  indices acertarem, N entradas e' exprimivel hoje e a celula e' de CUSTO, nao de contrato."
    );
}

/// **O que o switch de hoje faz quando o `select` aponta para uma porta VAZIA** — e se a
/// composição preserva o per-elemento, que é a propriedade que o doc-comment do nó destaca.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_an_unwired_port_reads_and_whether_the_chain_stays_per_element() {
    let reg = registry();
    eprintln!("\n[switch-arity] o `select` a apontar para uma porta DESLIGADA");
    eprintln!("  (so' `in0 = 100` e `in1 = 101` ligadas; `in2`/`in3` vazias)\n");
    for want in 0..4 {
        let mut g = Graph::new();
        let seed = one_point(&mut g);
        let select = constant(&mut g, seed, want as f32);
        let a = constant(&mut g, seed, 100.0);
        let b = constant(&mut g, seed, 101.0);
        let sw = g.add_node("value.switch");
        wire(&mut g, select, 0, sw, 0);
        wire(&mut g, a, 0, sw, 1);
        wire(&mut g, b, 0, sw, 2);
        g.validate(&reg).expect("bem-tipado");
        eprintln!("  select = {want}  =>  {:.3}", read(&g, &reg, sw));
    }

    eprintln!("\n[switch-arity] o PER-ELEMENTO sobrevive ao encadeamento?");
    eprintln!("  (uma fileira de 7 pecas; o `select` de cada uma e' o proprio indice)\n");
    let mut g = Graph::new();
    let row = g.add_node("motion.grid");
    g.set_param(row, "rows", 1.0);
    g.set_param(row, "cols", 7.0);
    // Um `select` que vale o INDICE de cada peca: `value.instance_field` em modo indice.
    let idx = g.add_node("value.instance_field");
    g.set_param(idx, "mode", 0.0);
    wire(&mut g, row, 0, idx, 0);
    let sources: Vec<NodeId> = (0..7)
        .map(|k| constant(&mut g, row, 100.0 + k as f32))
        .collect();
    let (sink, cost) = wide_mux(&mut g, idx, &sources);
    g.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &reg, sink, 0.0).expect("coza");
    let CookValue::Instances(st) = &out[0] else {
        panic!("stream")
    };
    match st.get("v") {
        Some(Column::Scalar(v)) => eprintln!(
            "  {cost} nos, saida: {}",
            v.iter()
                .map(|x| format!("{x:.0}"))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        _ => eprintln!("  sem coluna `v`"),
    }
    eprintln!(
        "\n  LEITURA: `100 101 102 103 104 105 106` = cada peca escolheu a SUA entrada, e o
  encadeamento nao colapsou o mux num escalar. O modo do `instance_field` pode nao ser
  o indice cru — se a saida for outra coisa, e' a fixture, nao a composicao."
    );
}
