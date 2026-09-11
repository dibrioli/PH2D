//! **ONDE O `motion.collide` DEIXA DE HONRAR OS NÚMEROS** (doc 89, folha 03 — a linha 63).
//!
//! O nó não tem `ParamHardMax` nenhum, e os três params quebram por mecanismos DIFERENTES:
//!
//! * **`iterations` já é CLAMPADO** (`MAX_ITERATIONS = 64`, `lib.rs:254`). Hoje a caixa de
//!   texto aceita 200 e o kernel entrega 64 — **aceita e mente**, a cicatriz do `lattice` 400 e
//!   do `kaleidoscope` 256. Aqui não há o que medir: o teto É o clamp, e o gate é de LEI.
//! * **`strength` é um fator de RELAXAÇÃO.** Em `1` a varredura aplica a correção inteira;
//!   acima disso ela **sobre-corrige**, e a fronteira clássica da sobre-relaxação é onde o
//!   laço deixa de convergir e passa a oscilar. Isso é MEDÍVEL.
//! * **`radius` é a célula da GRADE** (`GridSpec { cell_param: "radius" }`), então um raio
//!   grande demais para a nuvem colapsa toda gente numa célula só e a varredura 3×3 vira
//!   `O(N²)` — o mesmo mecanismo que o `motion.boids` documenta no `spread`.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_collide_ceiling -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Uma nuvem quadrada de `side²` pontos, apertada de propósito: com o raio default os discos
/// se sobrepõem, que é a única condição em que este nó faz alguma coisa.
fn scene(side: f32, radius: f32, iterations: f32, strength: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", side);
    g.set_param(seed, "cols", side);
    // ⚠️ `gap_x`/`gap_y`, NÃO `spacing`: um `set_param` com nome que o manifesto não declara é
    // **ignorado em silêncio**, e a 1ª versão desta sonda mediu folga `1,0000` em toda a
    // varredura por isso — a grade usou o default `1,0`, os discos nasceram SEPARADOS e o nó
    // era um no-op. A fixture não continha o fenômeno, e nada acusou.
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

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

fn cook_once(g: &Graph, reg: &NodeRegistry, node: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, node, 0.0).expect("cook");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida do collide e um stream")
    };
    positions(s)
}

/// A menor distância entre dois discos, e o maior afastamento em relação à semeadura.
///
/// ⚠️ **O par é o oráculo, e sozinho nenhum dos dois serve:** a folga mínima diz se a relaxação
/// CONVERGIU (o nó promete `2·radius`), e o afastamento diz se ela EXPLODIU — uma sobre-relaxação
/// que oscila pode, por acaso, terminar um sweep com folga boa e a nuvem arremessada.
fn measure(p: &[[f32; 2]], seed_span: f32) -> (f32, f32) {
    let mut min_gap = f32::INFINITY;
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let d = ((p[i][0] - p[j][0]).powi(2) + (p[i][1] - p[j][1]).powi(2)).sqrt();
            min_gap = min_gap.min(d);
        }
    }
    let mut max_r = 0.0f32;
    for q in p {
        max_r = max_r.max(q[0].abs().max(q[1].abs()));
    }
    (min_gap, max_r / seed_span.max(1e-6))
}

#[test]
#[ignore = "probe: prints the measured ceilings, asserts nothing"]
fn measure_where_the_collide_stops_honouring_its_numbers() {
    let reg = registry();
    let side = 6.0f32; // 36 discos
    // O span da SEMEADURA, medido em vez de suposto: `iterations = 0` é o nó em repouso.
    let (g0, n0) = scene(side, 0.3, 0.0, 1.0);
    let seed_span = measure(&cook_once(&g0, &reg, n0), 1.0).1;
    println!(
        "\nsemeadura: {} discos, span {seed_span:.4}, folga min {:.4}",
        cook_once(&g0, &reg, n0).len(),
        measure(&cook_once(&g0, &reg, n0), 1.0).0
    );

    println!("\n== ITERATIONS: o kernel CLAMPA em 64 -- acima disso a caixa aceita e mente ==");
    println!(
        "{:>12}  {:>12}  {:>28}",
        "iterations", "folga min", "identico ao de 64?"
    );
    let at_cap = cook_once(
        &scene(side, 0.3, 64.0, 1.0).0,
        &reg,
        scene(side, 0.3, 64.0, 1.0).1,
    );
    for &it in &[8.0, 32.0, 64.0, 65.0, 200.0, 100_000.0] {
        let (g, n) = scene(side, 0.3, it, 1.0);
        let p = cook_once(&g, &reg, n);
        let (gap, _) = measure(&p, seed_span);
        let same = p.len() == at_cap.len()
            && p.iter()
                .zip(&at_cap)
                .all(|(a, b)| a[0].to_bits() == b[0].to_bits() && a[1].to_bits() == b[1].to_bits());
        println!(
            "{it:>12.0}  {gap:>12.4}  {:>28}",
            if same { "SIM (byte a byte)" } else { "-" }
        );
    }

    // ⚠️ **No PIOR caso, que é o teto das ITERAÇÕES.** Uma relaxação instável não explode em 8
    // varreduras — ela precisa de tempo. Medir o `strength` no default de 8 responderia sobre
    // um documento que o artista não é obrigado a autorar (a lição do canto da mola).
    println!("\n== STRENGTH a 64 iteracoes: a sobre-relaxacao, onde ajudar vira atrapalhar ==");
    println!(
        "{:>12}  {:>12}  {:>14}  {:>10}",
        "strength", "folga min", "raio/semeadura", "melhora?"
    );
    let mut best = 0.0f32;
    for &st in &[
        1.0, 2.0, 2.5, 3.0, 3.2, 3.4, 3.6, 3.8, 4.0, 4.5, 5.0, 6.0, 8.0, 16.0,
    ] {
        let (g, n) = scene(side, 0.3, 64.0, st);
        let p = cook_once(&g, &reg, n);
        let (gap, spread) = measure(&p, seed_span);
        // O oráculo é a MONOTONIA: mais correção tem de empacotar MELHOR. Onde parar de
        // melhorar, a sobre-relaxação começou a lutar contra si mesma.
        let better = gap >= best - 1e-4;
        if gap > best {
            best = gap;
        }
        println!(
            "{st:>12.2}  {gap:>12.4}  {spread:>14.3}  {:>10}",
            if better { "sim" } else { "NAO" }
        );
    }

    // ⚠️ **O PIOR CASO do `strength` é o PAR ISOLADO, e ele é mais apertado que a nuvem.**
    // Este é um Jacobi MEDIADO: com muitos vizinhos sobrepostos a média amortece a
    // sobre-correção, então a nuvem densa sobrevive a `strength` ~3,5. Dois discos sozinhos não
    // têm por quem ser amortecidos — cada um anda `penetração·strength/2`, logo a correção
    // total é `penetração·strength`, e acima de `2` eles **atravessam um pelo outro** e passam a
    // oscilar. Um teto tem de valer no documento mais simples que o artista pode desenhar.
    println!("\n== STRENGTH no PAR ISOLADO (o pior caso: ninguem para amortecer) ==");
    println!(
        "{:>12}  {:>12}  {:>26}",
        "strength", "folga", "converge (>= 2r)?"
    );
    for &st in &[0.5, 1.0, 1.5, 1.9, 2.0, 2.1, 2.5, 3.0, 4.0, 8.0] {
        let mut g = Graph::new();
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", 1.0);
        g.set_param(seed, "cols", 2.0);
        g.set_param(seed, "gap_x", 0.25); // dois discos sobrepostos, e mais ninguem
        let c = g.add_node("motion.collide");
        g.set_param(c, "radius", 0.3);
        g.set_param(c, "iterations", 64.0);
        g.set_param(c, "strength", st);
        g.connect(Edge {
            from: (seed, 0),
            to: (c, 0),
            delayed: false,
        })
        .expect("edge");
        let p = cook_once(&g, &reg, c);
        let gap = ((p[0][0] - p[1][0]).powi(2) + (p[0][1] - p[1][1]).powi(2)).sqrt();
        println!(
            "{st:>12.2}  {gap:>12.4}  {:>26}",
            if gap >= 0.6 - 1e-3 {
                "sim"
            } else {
                "NAO -- oscila"
            }
        );
    }

    println!("\n== RADIUS: o raio E a celula da grade; grande demais, todos caem numa so ==");
    println!(
        "{:>12}  {:>12}  {:>14}  {:>12}",
        "radius", "folga min", "folga/2r", "raio/semead."
    );
    for &r in &[0.1, 0.3, 1.0, 100.0, 1e4, 1e6, 1e8, 1e10, 1e12, 1e15] {
        let (g, n) = scene(side, r, 8.0, 1.0);
        let p = cook_once(&g, &reg, n);
        let (gap, spread) = measure(&p, seed_span);
        // O oráculo é a INVARIÂNCIA DE ESCALA: este nó só sabe afastar discos, então dobrar o
        // raio tem de dobrar tudo. A fração `folga / 2·raio` é adimensional — se ela se mantém,
        // não há teto de correção a escrever; se desaba, achámos o fim da precisão de `f32`.
        let frac = gap / (2.0 * r);
        println!("{r:>12.0}  {gap:>12.4}  {frac:>14.4}  {spread:>12.3}",);
    }
    println!();
}
