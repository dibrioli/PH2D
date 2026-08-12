//! **O TETO DO `motion.collide` É ONDE O KERNEL DEIXA DE HONRÁ-LO** (doc 88 B2 · doc 89 folha 03).
//!
//! Os números e as tabelas vivem no doc-comment do `PARAM_HARD_MAX` da crate; a sonda que os
//! mediu é a `measure_collide_ceiling`. Estes gates afirmam a **PROPRIEDADE**: dirigem o nó
//! pela porta do produto no teto e acima dele, e exigem que o comportamento mude ali.
//!
//! ⚠️ **E um deles afirma uma AUSÊNCIA:** o `radius` não tem teto de propósito, porque a
//! medição mostrou o nó escala-invariante. Sem gate, a próxima varredura "completa" a tabela.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

const COLLIDE: NodeTypeId = NodeTypeId::of("motion.collide");

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma nuvem de 36 discos que se SOBREPÕEM — a única condição em que este nó faz algo.
///
/// ⚠️ Os params da grade são `gap_x`/`gap_y`, **não** `spacing`: um `set_param` com nome que o
/// manifesto não declara é ignorado **em silêncio**, e a 1ª fixture da sonda mediu um no-op
/// inteiro por isso (discos nascidos separados, folga constante, tudo byte-idêntico).
fn scene(radius: f32, iterations: f32, strength: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 6.0);
    g.set_param(seed, "cols", 6.0);
    g.set_param(seed, "gap_x", 0.25);
    g.set_param(seed, "gap_y", 0.25);
    let collide = g.add_node("motion.collide");
    g.set_param(collide, "radius", radius);
    g.set_param(collide, "iterations", iterations);
    g.set_param(collide, "strength", strength);
    g.connect(Edge {
        from: (seed, 0),
        to: (collide, 0),
        delayed: false,
    })
    .expect("edge");
    (g, collide)
}

fn cook(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut c = Cook::new();
    let out = c.cook(g, reg, node, 0.0).expect("cook");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida do collide e um stream")
    };
    match Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// (a menor folga entre dois discos, a extensão da nuvem).
fn measure(p: &[[f32; 2]]) -> (f32, f32) {
    let mut gap = f32::INFINITY;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            gap = gap.min(((p[i][0] - p[j][0]).powi(2) + (p[i][1] - p[j][1]).powi(2)).sqrt());
        }
    }
    let span = p
        .iter()
        .fold(0.0f32, |m, q| m.max(q[0].abs().max(q[1].abs())));
    (gap, span)
}

fn ceiling(param: &str) -> f32 {
    registry()
        .param_hard_max(COLLIDE, param)
        .unwrap_or_else(|| panic!("o collide declara um teto digitavel para `{param}`"))
}

/// **A LEI: um teto digitável não pode passar do que o kernel HONRA.**
///
/// O `eval` clampa as varreduras, então acima do clamp o número do artista é jogado fora. Este
/// gate mede a consequência em vez de a afirmar: acima do teto o resultado é **byte a byte** o
/// do teto — e é isso que torna um valor maior um controle que mente.
#[test]
fn above_the_iterations_ceiling_the_kernel_returns_the_ceilings_own_answer() {
    let reg = registry();
    let cap = ceiling("iterations");
    let at_cap = {
        let (g, n) = scene(0.3, cap, 1.0);
        cook(&g, &reg, n)
    };
    for over in [cap + 1.0, cap * 3.0, 100_000.0] {
        let (g, n) = scene(0.3, over, 1.0);
        let p = cook(&g, &reg, n);
        assert_eq!(p.len(), at_cap.len());
        for (a, b) in p.iter().zip(&at_cap) {
            assert_eq!(
                (a[0].to_bits(), a[1].to_bits()),
                (b[0].to_bits(), b[1].to_bits()),
                "com iterations = {over} o kernel devolve a resposta do teto ({cap}) byte a \
                 byte -- e por isso o teto digitavel nao pode passar dali"
            );
        }
    }
    // E ABAIXO do teto o número ainda MORDE: sem isto o gate acima passaria sobre um nó cujas
    // varreduras não fazem nada, e a fixture não conteria o fenômeno.
    let (g, n) = scene(0.3, cap / 2.0, 1.0);
    let (half, _) = measure(&cook(&g, &reg, n));
    let (full, _) = measure(&at_cap);
    assert!(
        full > half + 1e-3,
        "abaixo do teto varrer mais tem de empacotar mais apertado ({half} -> {full}); se nao \
         empacota, a cena nao tem discos sobrepostos e o gate acima e vacuo"
    );
}

/// **O teto do `strength` é onde a sobre-relaxação para de comprar e começa a ATIRAR.**
///
/// ⚠️ As duas metades importam, e a de baixo é a que justifica o teto ser MAIOR que o slider:
/// a sobre-relaxação compra empacotamento de verdade (+16% pelo mesmo custo), então cortá-la em
/// `1,0` roubaria faixa útil. A de cima é o preço: acima dela a nuvem sai do lugar.
#[test]
fn the_strength_ceiling_still_packs_and_does_not_throw_the_cloud() {
    let reg = registry();
    let cap = ceiling("strength");
    let it = ceiling("iterations");

    let (g1, n1) = scene(0.3, it, 1.0);
    let (gap_one, span_one) = measure(&cook(&g1, &reg, n1));
    let (gc, nc) = scene(0.3, it, cap);
    let (gap_cap, span_cap) = measure(&cook(&gc, &reg, nc));

    assert!(
        gap_cap > gap_one * 1.10,
        "no teto ({cap}) a sobre-relaxacao tem de empacotar bem mais apertado que a correcao \
         inteira ({gap_one} -> {gap_cap}); se nao compra nada, o teto acima do slider nao se \
         justifica"
    );
    assert!(
        span_cap < span_one * 1.15,
        "e tem de o fazer SEM atirar a nuvem ({span_one} -> {span_cap})"
    );
}

/// **O `radius` NÃO tem teto, e a razão é medida: este nó é ESCALA-INVARIANTE.**
///
/// ⚠️ Sem este gate a próxima varredura "completa" a tabela e inventa um número — o palpite que
/// o §0 proíbe. O oráculo é a fração adimensional `folga / 2·raio`: se ela não se move sob
/// quinze ordens de grandeza, não existe ponto onde o kernel deixe de honrar o raio.
#[test]
fn the_radius_has_no_ceiling_because_the_node_is_scale_free() {
    let reg = registry();
    assert!(
        registry().param_hard_max(COLLIDE, "radius").is_none(),
        "o radius nao tem teto de proposito -- a medicao nao achou nenhum ponto de quebra"
    );
    let frac = |r: f32| {
        let (g, n) = scene(r, 64.0, 1.0);
        let (gap, _) = measure(&cook(&g, &reg, n));
        gap / (2.0 * r)
    };
    let base = frac(1.0);
    for r in [1e3f32, 1e6, 1e9, 1e12] {
        let f = frac(r);
        assert!(
            (f - base).abs() < 0.01,
            "a fracao adimensional folga/2r tem de ser a MESMA em qualquer escala \
             (r=1 -> {base}, r={r} -> {f})"
        );
    }
}
