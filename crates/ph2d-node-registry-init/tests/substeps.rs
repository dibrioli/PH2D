//! **O SUBSTEP POR-ZONA** (doc 89, folha 13 — o último P1), medido na cadeia REAL.
//!
//! A folha provou que encadear `sim.step` duas vezes no interior é no-op EXATO (o 1º já escreveu
//! `sim_t = playhead`, então o 2º lê `dt = 0`) e tirou daí o corolário de que não havia onde pôr
//! um 2º passe. O corolário caiu: o `dt` é `playhead − sim_t(i)`, **por elemento**, então um
//! substep é propriedade do **RELÓGIO** — subdividir o playhead subdivide a integração sem tocar
//! num kernel.
//!
//! ⚠️ **O que estes gates protegem não é a convergência, é o ESCOPO.** Subdividir pelo
//! `advance_tick` também converge — e cobra de TODO `pre` do grafo (medido em
//! `measure_substeps.rs`: 16,36× para 16 passadas, num vizinho que nada tem com a zona).

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Uma zona que CAI sob aceleração constante: grid → zone(init) ; zone =pre=> wind → step →
/// zone(state). A rajada é zero de propósito — assim a resposta exata é `a·t²/2` e o erro do
/// integrador é o único desvio.
fn falling_zone(g: &mut Graph, strength: f32) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    g.set_param(seed, "gap_x", 1.0);
    g.set_param(seed, "gap_y", 1.0);

    let zone = g.add_node("sim.zone");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", strength);
    g.set_param(wind, "gust", 0.0);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);

    wire(g, seed, 0, zone, 0, false);
    wire(g, zone, 0, wind, 0, true);
    wire(g, wind, 0, step, 0, false);
    wire(g, step, 0, zone, 1, false);
    zone
}

fn px(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0].x(),
        _ => f32::NAN,
    }
}

trait Vec2X {
    fn x(&self) -> f32;
}
impl Vec2X for [f32; 2] {
    fn x(&self) -> f32 {
        self[0]
    }
}

/// Corre `frames` quadros a 60 fps. `sub` > 1 pede o substep POR-ZONA ao alvo dado.
fn run(g: &Graph, reg: &NodeRegistry, target: NodeId, read: NodeId, frames: u64, sub: u32) -> f32 {
    let mut cook = Cook::new();
    let mut last = f32::NAN;
    for k in 0..frames {
        let t = (k + 1) as f64 / 60.0;
        cook.substep(g, reg, target, k as f64 / 60.0, t, sub)
            .expect("substeps");
        last = px(cook.cook(g, reg, read, t).expect("cooks")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

/// **A entrega:** o erro cai pela METADE a cada dobra do número de passadas — a assinatura de
/// Euler de 1ª ordem, e a prova de que o substep integra em vez de só re-cozinhar.
///
/// FALSIFICADO se o `substep` fosse um no-op: as cinco colunas seriam o mesmo número.
#[test]
fn the_substep_converges_first_order_on_the_zone() {
    let reg = registry();
    let mut g = Graph::new();
    let zone = falling_zone(&mut g, 40.0);
    g.validate(&reg).expect("bem-tipado");

    let exact = 40.0f32 / 2.0; // a·t²/2 com t = 1 s
    let mut prev_err = f32::INFINITY;
    for sub in [1u32, 2, 4, 8, 16] {
        let err = (run(&g, &reg, zone, zone, 60, sub) - exact).abs();
        if prev_err.is_finite() {
            let ratio = prev_err / err;
            assert!(
                (1.7..2.3).contains(&ratio),
                "dobrar as passadas tem de cortar o erro pela metade; sub={sub} deu {ratio:.3}× \
                 ({prev_err:.4} -> {err:.4})"
            );
        }
        prev_err = err;
    }
    assert!(
        prev_err < 0.1,
        "16 passadas têm de chegar perto da analítica; erro {prev_err:.4}"
    );
}

/// **O ESCOPO — o gate que separa esta wave de subdividir pelo `advance_tick`.** Duas zonas
/// independentes no MESMO grafo; substepar uma não pode mover a outra.
///
/// ⚠️ As duas metades são precisas: a vizinha tem de sair **byte-idêntica** ao controle (senão o
/// substep vazou) **e** a alvo tem de MUDAR (senão um `substep` que não faz nada passaria aqui).
#[test]
fn substepping_one_zone_leaves_its_neighbour_byte_identical() {
    let reg = registry();
    let mut g = Graph::new();
    let a = falling_zone(&mut g, 40.0);
    let b = falling_zone(&mut g, 25.0);
    g.validate(&reg).expect("bem-tipado");

    // Controle: ninguém substepa.
    let ctl_a = run(&g, &reg, a, a, 30, 1);
    let ctl_b = run(&g, &reg, b, b, 30, 1);

    // O mesmo grafo, com a zona A substepada 8×; lemos as DUAS.
    let mut cook = Cook::new();
    let (mut got_a, mut got_b) = (f32::NAN, f32::NAN);
    for k in 0..30u64 {
        let t = (k + 1) as f64 / 60.0;
        cook.substep(&g, &reg, a, k as f64 / 60.0, t, 8)
            .expect("substeps");
        got_a = px(cook.cook(&g, &reg, a, t).expect("cooks")[0].as_stream());
        got_b = px(cook.cook(&g, &reg, b, t).expect("cooks")[0].as_stream());
        cook.advance_tick(&g, &reg, t).expect("tick");
    }

    assert_eq!(
        got_b.to_bits(),
        ctl_b.to_bits(),
        "a vizinha não paga o substep de A: {got_b} contra o controle {ctl_b}"
    );
    assert!(
        (got_a - ctl_a).abs() > 1e-3,
        "e a alvo TEM de mudar, senão o substep é um no-op: {got_a} vs {ctl_a}"
    );
}

/// **O relógio é RESTAURADO.** A lei de contagem do `sim.spawn` lê o `dt` do `prev_playhead`;
/// se o substep o deixasse na última fatia, um spawner FORA da zona receberia `1/n` de quadro
/// como se fosse o quadro e pariria de menos.
#[test]
fn the_substep_gives_the_clock_back_to_the_rest_of_the_graph() {
    let reg = registry();
    let mut g = Graph::new();
    let zone = falling_zone(&mut g, 40.0);

    // Um spawner independente, cuja contagem é função do dt do quadro.
    let tmpl = g.add_node("motion.grid");
    g.set_param(tmpl, "rows", 1.0);
    g.set_param(tmpl, "cols", 1.0);
    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", 30.0);
    g.set_param(spawn, "scatter", 0.0);
    wire(&mut g, tmpl, 0, spawn, 0, false);
    g.validate(&reg).expect("bem-tipado");

    let born = |sub: u32| -> usize {
        let mut cook = Cook::new();
        let mut total = 0usize;
        for k in 0..30u64 {
            let t = (k + 1) as f64 / 60.0;
            cook.substep(&g, &reg, zone, k as f64 / 60.0, t, sub)
                .expect("substeps");
            total += cook.cook(&g, &reg, spawn, t).expect("cooks")[0]
                .as_stream()
                .count();
            cook.advance_tick(&g, &reg, t).expect("tick");
        }
        total
    };

    let ctl = born(1);
    assert!(ctl > 0, "a fixture precisa PARIR: {ctl}");
    assert_eq!(
        born(8),
        ctl,
        "substepar a zona não pode mudar quantos o spawner de FORA pariu"
    );
}

/// **`n <= 1` não é caso especial a lembrar** — o laço não executa e o tique sai byte-idêntico
/// a um que nunca chamou `substep`.
#[test]
fn a_substep_of_one_is_the_tick_that_never_called_it() {
    let reg = registry();
    let mut g = Graph::new();
    let zone = falling_zone(&mut g, 40.0);
    g.validate(&reg).expect("bem-tipado");

    let mut plain = Cook::new();
    let mut asked = Cook::new();
    for k in 0..30u64 {
        let t = (k + 1) as f64 / 60.0;
        let a = px(plain.cook(&g, &reg, zone, t).expect("cooks")[0].as_stream());
        asked
            .substep(&g, &reg, zone, k as f64 / 60.0, t, 1)
            .expect("substeps");
        let b = px(asked.cook(&g, &reg, zone, t).expect("cooks")[0].as_stream());
        assert_eq!(a.to_bits(), b.to_bits(), "quadro {k}: {a} contra {b}");
        plain.advance_tick(&g, &reg, t).expect("tick");
        asked.advance_tick(&g, &reg, t).expect("tick");
    }
}

/// **A última sub-passada cai em `playhead`**, então o `cook` do quadro bate no MEMO em vez de
/// dar um passo a mais. Oráculo: cozinhar o alvo mais vezes depois do substep não pode mover a
/// zona nem um bit.
#[test]
fn the_frames_own_cook_hits_the_memo_instead_of_stepping_again() {
    let reg = registry();
    let mut g = Graph::new();
    let zone = falling_zone(&mut g, 40.0);
    g.validate(&reg).expect("bem-tipado");

    let mut cook = Cook::new();
    for k in 0..20u64 {
        let t = (k + 1) as f64 / 60.0;
        cook.substep(&g, &reg, zone, k as f64 / 60.0, t, 4)
            .expect("substeps");
        let once = px(cook.cook(&g, &reg, zone, t).expect("cooks")[0].as_stream());
        let twice = px(cook.cook(&g, &reg, zone, t).expect("cooks")[0].as_stream());
        assert_eq!(
            once.to_bits(),
            twice.to_bits(),
            "quadro {k}: um cook a mais moveu a zona ({once} -> {twice})"
        );
        cook.advance_tick(&g, &reg, t).expect("tick");
    }
}

/// **Um substep é um sub-TIQUE, e a fixture tem de conter um pre-consumidor PURE.**
///
/// O fingerprint de um nó que consome `pre` inclui o tique — o memo existe para o circuito
/// sequencial avançar *uma vez por tique*. ⚠️ Na zona que CAI acima quem consome o `pre` é o
/// `force.wind`, que é **`Effect::Temporal`**: o playhead entra no fingerprint dele, então ele
/// re-cozinha entre sub-passadas por conta própria e **mascara** um bump de tique ausente. Quem
/// consome o `pre` na fiação canônica de zona é o `motion.combine`, que é **`Effect::Pure`** —
/// esta zona acumula uma grade por tique e é onde a ausência aparece.
///
/// FALSIFICADO se o substep não bumpasse o tique: as passadas 2..n bateriam no memo e a zona
/// cresceria uma vez por QUADRO em vez de uma por sub-passada.
#[test]
fn a_substep_is_a_sub_tick_for_a_pure_pre_consumer_too() {
    let reg = registry();
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let feed = g.add_node("motion.grid");
    g.set_param(feed, "rows", 1.0);
    g.set_param(feed, "cols", 1.0);

    let zone = g.add_node("sim.zone");
    let merge = g.add_node("motion.combine");
    wire(&mut g, seed, 0, zone, 0, false);
    wire(&mut g, zone, 0, merge, 0, true); // o `pre` — e o consumidor dele e PURE
    wire(&mut g, feed, 0, merge, 1, false);
    wire(&mut g, merge, 0, zone, 1, false);
    g.validate(&reg).expect("bem-tipado");

    let grown = |sub: u32| -> usize {
        let mut cook = Cook::new();
        let mut last = 0usize;
        for k in 0..10u64 {
            let t = (k + 1) as f64 / 60.0;
            cook.substep(&g, &reg, zone, k as f64 / 60.0, t, sub)
                .expect("substeps");
            last = cook.cook(&g, &reg, zone, t).expect("cooks")[0]
                .as_stream()
                .count();
            cook.advance_tick(&g, &reg, t).expect("tick");
        }
        last
    };

    let one = grown(1);
    let four = grown(4);
    assert!(one > 1, "o controle precisa CRESCER: {one}");
    assert_eq!(
        four,
        one * 4,
        "quatro sub-tiques acumulam quatro vezes: {four} contra {one} em passada unica"
    );
}

/// **UMA ILHA, UM RELÓGIO — o defeito que a pergunta acoplada expôs.**
///
/// Duas zonas onde B lê a saída de A: A vive no CONE de B. Substepar as duas por conta
/// OVER-STEPA A, porque o laço de B re-cozinha o cone de B, que contém A — medido, A ia de
/// `4,876` para `15,094`, quase o triplo. [`substep_islands`] devolve UMA raiz (a de baixo,
/// cujo cone cobre a de cima), e substepar essa raiz dá a MESMA trajetória de A que substepá-la
/// sozinha.
///
/// FALSIFICADO se o achador devolvesse os dois declarantes: A andaria o triplo.
#[test]
fn a_coupled_pair_is_one_island_with_one_clock() {
    use ph2d_nodegraph::cook::substep_islands;
    let reg = registry();
    let mut g = Graph::new();
    let a = falling_zone(&mut g, 40.0);
    let seedb = g.add_node("motion.grid");
    g.set_param(seedb, "rows", 1.0);
    g.set_param(seedb, "cols", 1.0);
    let b = g.add_node("sim.zone");
    let merge = g.add_node("motion.combine");
    wire(&mut g, seedb, 0, b, 0, false);
    wire(&mut g, b, 0, merge, 0, true);
    wire(&mut g, a, 0, merge, 1, false); // o ACOPLAMENTO: A entra no interior de B
    wire(&mut g, merge, 0, b, 1, false);
    g.set_param(a, "substeps", 4.0);
    g.set_param(b, "substeps", 4.0);
    g.validate(&reg).expect("bem-tipado");

    let islands = substep_islands(&g, &reg);
    assert_eq!(
        islands.len(),
        1,
        "o par acoplado e UMA ilha, e a raiz e a de baixo: {islands:?}"
    );
    assert_eq!(
        islands[0].root, b,
        "a raiz cobre o cone, entao e a de BAIXO"
    );

    // E a trajetoria de A tem de ser a mesma que substepa-la sozinha.
    let mut solo = Cook::new();
    let mut pair = Cook::new();
    for k in 0..30u64 {
        let t = (k + 1) as f64 / 60.0;
        let fs = k as f64 / 60.0;
        solo.substep(&g, &reg, a, fs, t, 4).expect("s");
        for isl in substep_islands(&g, &reg) {
            pair.substep(&g, &reg, isl.root, fs, t, isl.substeps)
                .expect("s");
        }
        let pa = px(solo.cook(&g, &reg, a, t).expect("c")[0].as_stream());
        let pb = px(pair.cook(&g, &reg, a, t).expect("c")[0].as_stream());
        assert!(
            (pa - pb).abs() < 1e-4,
            "quadro {k}: substepar a ilha nao pode adiantar A ({pa} contra {pb})"
        );
        solo.advance_tick(&g, &reg, t).expect("t");
        pair.advance_tick(&g, &reg, t).expect("t");
    }
}

/// **Ilhas independentes são DUAS ilhas — e correm no relógio do GRAFO.**
///
/// ⚠️ As duas metades são separadas de propósito. **Duas ilhas** é correção: substepá-las por uma
/// raiz só deixaria a outra parada. **Um relógio** é a leitura das referências no nível do
/// contêiner — e o contêiner passou a ser o grafo, porque cada objeto Motion tem o seu (Houdini
/// põe `Substeps` na DOP Network, Niagara o Fixed Tick no System). É isso que faz o device nunca
/// precisar recusar: marchar o plano inteiro `n` vezes dá a cada ilha os mesmos `n` sub-tiques.
#[test]
fn independent_islands_are_two_islands_on_the_graphs_one_clock() {
    use ph2d_nodegraph::cook::substep_islands;
    let reg = registry();
    let mut g = Graph::new();
    let a = falling_zone(&mut g, 40.0);
    let b = falling_zone(&mut g, 25.0);
    g.set_param(a, "substeps", 4.0);
    g.set_param(b, "substeps", 8.0);
    g.validate(&reg).expect("bem-tipado");

    let mut islands = substep_islands(&g, &reg);
    islands.sort_by_key(|i| i.root.0);
    assert_eq!(islands.len(), 2, "duas zonas desacopladas sao DUAS ilhas");
    assert_eq!(
        (islands[0].substeps, islands[1].substeps),
        (8, 8),
        "um grafo, um relogio -- o mais fino que qualquer uma pediu: {islands:?}"
    );
    assert_eq!(
        ph2d_nodegraph::cook::graph_substeps(&g, &reg),
        8,
        "e a porta que o device pergunta devolve o MESMO numero"
    );
}

/// **Uma ilha corre no MAIOR ritmo que os seus membros pedem** — um mundo, um relógio (Box2D
/// conta substeps por `b2World_Step`, Rapier por pipeline, Niagara por System). Quem pediu 4 não
/// o perde por estar acoplado a quem pediu 1.
#[test]
fn an_island_runs_at_the_finest_rate_any_member_asked_for() {
    use ph2d_nodegraph::cook::substep_islands;
    let reg = registry();
    let mut g = Graph::new();
    let a = falling_zone(&mut g, 40.0);
    let seedb = g.add_node("motion.grid");
    let b = g.add_node("sim.zone");
    let merge = g.add_node("motion.combine");
    wire(&mut g, seedb, 0, b, 0, false);
    wire(&mut g, b, 0, merge, 0, true);
    wire(&mut g, a, 0, merge, 1, false);
    wire(&mut g, merge, 0, b, 1, false);
    g.set_param(a, "substeps", 4.0);
    g.set_param(b, "substeps", 1.0); // a raiz pede 1, o membro pede 4
    g.validate(&reg).expect("bem-tipado");

    let islands = substep_islands(&g, &reg);
    assert_eq!(islands.len(), 1);
    assert_eq!(
        islands[0].substeps, 4,
        "o membro que pediu 4 nao o perde por a raiz pedir 1"
    );
    assert_eq!(ph2d_nodegraph::cook::graph_substeps(&g, &reg), 4);
}

/// **Ler a outra zona por um `pre` NÃO acopla as duas** — a aresta `pre` diz *"o valor do tique
/// ANTERIOR"*, e um valor do tique anterior não é deste alvo para avançar. É a fronteira que o
/// cone existe para desenhar, e ela é o que separa *acoplado* de *apenas observado*.
///
/// FALSIFICADO se o cone atravessasse arestas `delayed`: B engoliria A numa ilha só, e A perderia
/// o próprio ritmo por ser OLHADO.
#[test]
fn reading_another_zone_through_a_pre_does_not_couple_them() {
    use ph2d_nodegraph::cook::substep_islands;
    let reg = registry();
    let mut g = Graph::new();
    let a = falling_zone(&mut g, 40.0);
    let seedb = g.add_node("motion.grid");
    g.set_param(seedb, "rows", 1.0);
    g.set_param(seedb, "cols", 1.0);
    let b = g.add_node("sim.zone");
    let merge = g.add_node("motion.combine");
    wire(&mut g, seedb, 0, b, 0, false);
    wire(&mut g, b, 0, merge, 0, true);
    // ⚠️ A diferenca com `a_coupled_pair_is_one_island_with_one_clock`: aqui a leitura de A e
    // DELAYED. Uma linha, e as duas zonas passam de uma ilha para duas.
    wire(&mut g, a, 0, merge, 1, true);
    wire(&mut g, merge, 0, b, 1, false);
    g.set_param(a, "substeps", 4.0);
    g.set_param(b, "substeps", 8.0);
    g.validate(&reg).expect("bem-tipado");

    let mut islands = substep_islands(&g, &reg);
    islands.sort_by_key(|i| i.root.0);
    assert_eq!(
        islands.len(),
        2,
        "quem so OLHA o tique anterior da outra nao esta acoplado a ela: {islands:?}"
    );
    assert_eq!(
        (islands[0].substeps, islands[1].substeps),
        (8, 8),
        "duas ilhas, um relogio de grafo: {islands:?}"
    );
}
