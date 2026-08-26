//! **SONDA — o `motion.clone` de VÁRIAS fontes já é exprimível?**
//!
//! A folha 08 marca `P2`: *"multi-fonte (iterate/random/blend entre várias formas) — Cavalry
//! Duplicator `Auto Id`/`Shape Id`"*, com o veredito *"**NÃO no `clone`** — o `motion.combine`
//! funde as fontes num stream e o clone multiplica o conjunto INTEIRO, não escolhe uma por
//! cópia"*, e uma emenda de 22/08: *"metade da cura já shipou — o `motion.duplicator` tem
//! `pick`"*.
//!
//! ⚠️ **A emenda parou a meio da pergunta.** Se o `pick` escolhe uma forma por PONTO, e o
//! `motion.clone` é precisamente o nó que fabrica pontos em arranjo (radial, linear, arco),
//! então **`clone` → `duplicator(pick)`** é multi-fonte por composição, em dois nós — e a
//! célula estaria a pedir um param para o que o catálogo já dá.
//!
//! As duas rotas que esta sonda corre, lado a lado:
//!
//! ```text
//! A (o que a célula descreve):  combine(f1,f2,f3) → clone(count = N)
//! B (a rota por composição):    grid(1) → clone(count = N) → duplicator(points = ·,
//!                                                        shape = combine(f1,f2,f3), pick)
//! ```
//!
//! ⚠️ **A diferença que decide não é «funciona», é a CONTAGEM**: em `A` o clone multiplica o
//! conjunto inteiro (3 formas × N cópias = 3N linhas — o produto cartesiano que a célula
//! acusa); em `B` cada uma das N posições recebe UMA forma. Se `B` der `N` linhas com as
//! marcas a alternar, a rota existe e o que sobra é uma pergunta de ergonomia, não de alcance.
//!
//! ⚠️ **E há uma segunda pergunta que só a medição responde: o ARRANJO sobrevive?** O `clone`
//! aplica `scale_taper`/`rot_taper` ao longo das cópias. Na rota `B` o taper é aplicado aos
//! PONTOS, e o duplicator soma a pose do ponto à da forma — se o `scale` do ponto não viajar,
//! a rota exprime a posição e perde o afunilamento, que é meia resposta.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_clone_multisource -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

/// Quantas cópias o arranjo faz.
const COPIES: f32 = 6.0;
/// Quantas fontes distintas entram na junção.
const SOURCES: usize = 3;

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

/// Uma "forma" de UMA linha, marcada no `y`.
fn marked(g: &mut Graph, mark: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dy", mark);
    wire(g, seed, 0, mv, 0);
    mv
}

/// As `SOURCES` fontes marcadas, fundidas num stream só.
fn merged_sources(g: &mut Graph) -> NodeId {
    let merge = g.add_node("motion.combine");
    for k in 0..SOURCES {
        let s = marked(g, 100.0 * (k + 1) as f32);
        wire(g, s, 0, merge, k as u16);
    }
    merge
}

struct Read {
    count: usize,
    marks: Vec<String>,
    scales: Vec<String>,
}

fn read(cook: &mut Cook, g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Read {
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    // A marca vive no `y` do `P`; o `x` é o arranjo. Divido por 100 para ler a fonte.
    let marks = match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| format!("{:.0}", q[1] / 100.0)).collect(),
        _ => Vec::new(),
    };
    let scales = match s.get("size") {
        Some(Column::Vec2(v)) => v.iter().map(|q| format!("{:.2}", q[0])).collect(),
        Some(Column::Scalar(v)) => v.iter().map(|x| format!("{x:.2}")).collect(),
        _ => vec!["—".to_string()],
    };
    Read {
        count: s.count(),
        marks,
        scales,
    }
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_clone_into_duplicator_already_deal_one_source_per_copy() {
    let reg = registry();
    eprintln!(
        "\n[clone] {SOURCES} fontes, {COPIES:.0} copias — quantas linhas saem, e qual fonte em cada\n"
    );
    eprintln!("  {:<34}  {:>6}  fonte de cada copia", "rota", "linhas");

    // ── A: o que a célula descreve — o clone multiplica o CONJUNTO ──────────────
    {
        let mut g = Graph::new();
        let merge = merged_sources(&mut g);
        let clone = g.add_node("motion.clone");
        g.set_param(clone, "count", COPIES);
        g.set_param(clone, "distance", 1.0);
        wire(&mut g, merge, 0, clone, 0);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let r = read(&mut cook, &g, &reg, clone);
        eprintln!(
            "  {:<34}  {:>6}  {}",
            "A  combine -> clone",
            r.count,
            r.marks.join(" ")
        );
    }

    // ── B: a rota por composição — o clone fabrica os PONTOS ────────────────────
    for (rotulo, pick) in [
        ("B  clone -> duplicator(Off)", 0.0f32),
        ("B  clone -> duplicator(Cycle)", 1.0),
        ("B  clone -> duplicator(Random)", 2.0),
    ] {
        let mut g = Graph::new();
        let merge = merged_sources(&mut g);
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", 1.0);
        g.set_param(seed, "cols", 1.0);
        let clone = g.add_node("motion.clone");
        g.set_param(clone, "count", COPIES);
        g.set_param(clone, "distance", 1.0);
        wire(&mut g, seed, 0, clone, 0);
        let dup = g.add_node("motion.duplicator");
        g.set_param(dup, "pick", pick);
        wire(&mut g, merge, 0, dup, 0);
        wire(&mut g, clone, 0, dup, 1);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let r = read(&mut cook, &g, &reg, dup);
        eprintln!("  {rotulo:<34}  {:>6}  {}", r.count, r.marks.join(" "));
    }
    eprintln!(
        "\n  LEITURA: `A` deve dar {} linhas (o produto cartesiano que a celula acusa).
  Se `B/Cycle` der {COPIES:.0} linhas com as fontes a alternar `1 2 3 1 2 3`, a rota existe
  em DOIS nos e a celula e' de ergonomia, nao de alcance.",
        SOURCES * COPIES as usize
    );
}

/// ⚠️ **A 1.ª versão desta sonda mediu a MINHA suposição.** Ela corria a rota `B` com o
/// `point_scale` no default e concluiu que o afunilamento se perdia — mas o default é **`0`**,
/// e o doc do param diz-se assim de propósito (*"a escala do ponto fica de fora, que é o que
/// sempre aconteceu"*). ⇒ eu tinha medido *«o knob desligado não faz nada»*, que é uma
/// tautologia, e ia escrever uma célula em cima dela. A sonda passa a varrer o param.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_the_arrangement_taper_survive_the_detour_through_the_duplicator() {
    let reg = registry();
    eprintln!("\n[clone] o `scale_taper` do arranjo sobrevive a rota por composicao?\n");
    eprintln!("  {:<34}  {:>6}  size.x de cada copia", "rota", "linhas");
    // A: o taper aplicado pelo próprio clone.
    {
        let mut g = Graph::new();
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", 1.0);
        g.set_param(seed, "cols", 1.0);
        let clone = g.add_node("motion.clone");
        g.set_param(clone, "count", COPIES);
        g.set_param(clone, "distance", 1.0);
        g.set_param(clone, "scale_taper", 0.5);
        wire(&mut g, seed, 0, clone, 0);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let r = read(&mut cook, &g, &reg, clone);
        eprintln!(
            "  {:<34}  {:>6}  {}",
            "A  clone(scale_taper) direto",
            r.count,
            r.scales.join(" ")
        );
    }
    // B: o mesmo taper, mas o que sai é o duplicator — varrendo o `point_scale`, que é o
    // interruptor de quem manda na escala e nasce DESLIGADO.
    for ps in [0.0f32, 0.5, 1.0] {
        let mut g = Graph::new();
        let merge = merged_sources(&mut g);
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", 1.0);
        g.set_param(seed, "cols", 1.0);
        let clone = g.add_node("motion.clone");
        g.set_param(clone, "count", COPIES);
        g.set_param(clone, "distance", 1.0);
        g.set_param(clone, "scale_taper", 0.5);
        wire(&mut g, seed, 0, clone, 0);
        let dup = g.add_node("motion.duplicator");
        g.set_param(dup, "pick", 1.0);
        g.set_param(dup, "point_scale", ps);
        wire(&mut g, merge, 0, dup, 0);
        wire(&mut g, clone, 0, dup, 1);
        g.validate(&reg).expect("bem-tipado");
        let mut cook = Cook::new();
        let r = read(&mut cook, &g, &reg, dup);
        eprintln!(
            "  {:<34}  {:>6}  {}",
            format!("B  -> duplicator(point_scale {ps:.1})"),
            r.count,
            r.scales.join(" ")
        );
    }
    eprintln!(
        "\n  LEITURA: se `B` com `point_scale = 1` repetir a escada de `A`, o arranjo inteiro
  viaja e a rota por composicao e' completa — e a celula fecha por refutacao."
    );
}

/// **A TERCEIRA pergunta, e a que separa as duas células:** o duplicator replica as colunas da
/// FORMA (`for (name, col) in shape.columns()`), soma `P`/`rot` dos dois lados e compõe `size`
/// sob o `point_scale`. **Tudo o resto que exista só nos PONTOS não tem rota nenhuma** — nem
/// com knob. É esse o *"modos de transferência de atributo"* da célula do `motion.duplicator`,
/// e a sonda mede-o com a coluna mais comum de todas: a cor.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn which_point_columns_reach_the_output_at_all() {
    let reg = registry();
    eprintln!("\n[dup] uma coluna autorada SO' nos pontos — ela chega a' saida?\n");
    eprintln!("  {:<30}  na saida", "coluna");
    let mut g = Graph::new();
    let merge = merged_sources(&mut g);
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let clone = g.add_node("motion.clone");
    g.set_param(clone, "count", COPIES);
    g.set_param(clone, "distance", 1.0);
    wire(&mut g, seed, 0, clone, 0);
    // Uma cor por cópia, escrita SOBRE OS PONTOS (a rampa é o caso canónico do artista).
    let tint = g.add_node("motion.color_ramp");
    wire(&mut g, clone, 0, tint, 0);
    let dup = g.add_node("motion.duplicator");
    g.set_param(dup, "pick", 1.0);
    wire(&mut g, merge, 0, dup, 0);
    wire(&mut g, tint, 0, dup, 1);
    g.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    // Antes: o que os PONTOS de facto carregam.
    let pts = cook.cook(&g, &reg, tint, 0.0).expect("coza");
    let CookValue::Instances(p) = &pts[0] else {
        panic!("stream")
    };
    let nos_pontos: Vec<String> = p.columns().map(|(n, _)| n.clone()).collect();
    let out = cook.cook(&g, &reg, dup, 0.0).expect("coza");
    let CookValue::Instances(o) = &out[0] else {
        panic!("stream")
    };
    let na_saida: Vec<String> = o.columns().map(|(n, _)| n.clone()).collect();
    for c in &nos_pontos {
        let chegou = if na_saida.contains(c) {
            "sim"
        } else {
            "NAO ⚠️"
        };
        eprintln!("  {c:<30}  {chegou}");
    }
    eprintln!(
        "\n  LEITURA: as colunas que somem sao exactamente as que a politica hardcoded nao
  nomeia (`P`/`rot` somam, `size` compoe sob o `point_scale`, `Index`/`Count` renumeram,
  e o resto e' 'a forma vence'). Cada uma delas e' um atributo que o artista autorou no
  ponto e que desaparece SEM AVISO."
    );
}
