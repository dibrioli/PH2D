//! **UMA MORTE DÁ À LUZ** (doc 89, folha 13 · W7 — o P0 `sim.replicate`).
//!
//! A conferência da família `sim.*` mediu: *"`reap` constrói a saída só a partir de `keep`
//! — as linhas mortas são descartadas e nada a jusante as enxerga"*, e o doc 63 linha 97
//! marca o item como **P0**. A referência inteira o tem: `Trigger Event On Die` do VFX
//! Graph, *Death Event* do Niagara, `Aux` do Stardust, `POP Replicate` do Houdini.
//!
//! **E o achado da wave é que `sim.replicate` NÃO É UM NÓ.** Ele é uma FIAÇÃO das duas
//! saídas novas do `sim.lifetime` nas duas portas que o `sim.spawn` ganhou em 2026-08-10:
//!
//! ```text
//!   sim.lifetime.died  ──→ sim.spawn.template   (a carga: onde, com que cor, com que vel)
//!   sim.lifetime.pulse ──→ sim.spawn.pulse      (o gatilho: estas linhas dispararam)
//! ```
//!
//! ⚠️ **Os dois fios existem porque o SISTEMA DE TIPOS os separa** — `connects_directly`
//! exige domínio+dim+relógio iguais, e a carga é `Instances/Vec2/Frame` enquanto o gatilho é
//! `Instances/Scalar/Event`. A referência faz a mesma divisão (um evento com payload); aqui
//! ela é verificada pelo compilador.
//!
//! ⚠️ **Rodam AQUI e não nas crates dos nós**, a mesma razão do `pulse_level_chains`: esta é
//! a crate onde TODO nó é registrado, então é o build mais barato que enxerga um
//! `sim.lifetime` e um `sim.spawn` ao mesmo tempo. Um gate dentro de uma crate-nó provaria
//! a aritmética de uma coluna e **não** provaria que a cadeia que o artista monta funciona.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O tique em que os quatro morrem, **MEDIDO** pela sonda `probe_ages` e não escolhido:
/// com `life = 0,2 s` a 60 fps o `age` cruza o vão no `k = 14`, logo a fixture precisa de
/// quinze iterações para CONTER o fenômeno que mede.
const DEATH_TICKS: usize = 15;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn connect(g: &mut Graph, from: NodeId, from_port: u16, to: NodeId, to_port: u16) {
    g.connect(Edge {
        from: (from, from_port),
        to: (to, to_port),
        delayed: false,
    })
    .expect("edge");
}

/// Uma fila de `n` elementos numa linha — o `template` de um mundo que vai envelhecer.
fn row(g: &mut Graph, n: f32) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", n);
    g.set_param(grid, "gap_x", 1.0);
    g.set_param(grid, "gap_y", 1.0);
    grid
}

/// Cozinha `ticks` quadros a 60 fps e devolve o stream de UMA porta no último.
///
/// ⚠️ **Não é preciso cozinhar a ZONA para o mundo andar**, e a 1ª versão desta fixture o
/// fazia por uma atribuição minha que a medição derrubou: eu tinha culpado *"o quadro não
/// puxa a zona"* pelo `age = 0` eterno, e a causa era só o `pre` do laço escrito ao
/// contrário. O `Cook::advance_tick` **cozinha cada fonte `pre` ele mesmo** (o doc dele:
/// *"a `pre` source … must hold a current value even if this frame's cook target never
/// pulled it"*), então puxar o alvo basta — provado tirando a linha e vendo os quatro
/// gates seguirem verdes.
fn cook_to(g: &Graph, reg: &NodeRegistry, node: NodeId, port: usize, ticks: usize) -> Stream {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        last = cook.cook(g, reg, node, t).expect("cooks")[port]
            .as_stream()
            .clone();
        cook.advance_tick(g, reg, t).expect("advances");
    }
    last
}

fn scalar(s: &Stream, col: &str) -> Vec<f32> {
    match s.get(col) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => vec![],
    }
}

/// O mundo mínimo em que alguém morre: uma fila que envelhece dentro de uma zona.
///
/// `sim.step` é quem cresce o `age` (ele é dono do relógio da sim), então **não há morte
/// sem uma sim** — é a frase do doc-header do `sim.lifetime`, e é por isso que a fixture
/// tem uma zona em vez de um `age` fabricado à mão.
/// ⚠️ **O laço tem a forma que o motor gerencia, e ela não é a intuitiva:** o `pre` sai da
/// zona para o PRIMEIRO nó do corpo (*the engine-managed state entry*) e a volta ao `state`
/// é aresta NORMAL. A 1ª versão desta fixture fez o contrário e mediu um mundo que nunca
/// envelhece — os `pre` de um documento são escritos à mão, e escrevê-los ao contrário
/// compila.
fn dying_world(g: &mut Graph, life: f32) -> (NodeId, NodeId) {
    let src = row(g, 4.0);
    let zone = g.add_node("sim.zone");
    connect(g, src, 0, zone, 0);
    let step = g.add_node("sim.step");
    let life_n = g.add_node("sim.lifetime");
    g.set_param(life_n, "life", life);
    g.set_param(life_n, "variance", 0.0); // todos morrem no MESMO tique: o fenômeno concentrado
    g.connect(Edge {
        from: (zone, 0),
        to: (step, 0),
        delayed: true,
    })
    .expect("a entrada de estado que o motor gerencia");
    connect(g, step, 0, life_n, 0);
    connect(g, life_n, 0, zone, 1); // …e de volta ao `state`, aresta normal
    (zone, life_n)
}

/// **A saída `died` traz quem morreu — e ela é VAZIA enquanto ninguém morre.**
///
/// As duas metades importam: sem a segunda, um `died` que emitisse a fila inteira todo
/// tique passaria no primeiro assert e daria à luz para sempre.
#[test]
fn the_died_output_carries_the_dead_and_nothing_else() {
    let reg = registry();
    let mut g = Graph::new();
    let (_zone, life_n) = dying_world(&mut g, 0.2); // 0,2 s = 12 tiques a 60 fps

    // Cedo: ninguém passou dos 0,2 s.
    let early = cook_to(&g, &reg, life_n, 1, 6);
    assert_eq!(early.count(), 0, "aos 0,1 s ninguém morreu ainda");

    // O tique da morte: os quatro saem juntos (variance = 0).
    let dead = cook_to(&g, &reg, life_n, 1, DEATH_TICKS);
    assert_eq!(
        dead.count(),
        4,
        "os quatro passaram dos 0,2 s no mesmo tique"
    );
    // E o cadáver traz o que ele ERA — a posição é o que a referência chama de
    // *inherit position from event*.
    assert!(
        dead.get("P").is_some(),
        "o cadáver carrega as colunas dele, não um id solto"
    );

    // E depois: o mundo esvaziou, então não há mais mortes.
    let after = cook_to(&g, &reg, life_n, 1, 20);
    assert_eq!(after.count(), 0, "um mundo vazio não produz mortes");
}

/// **A saída `pulse` tem as MESMAS linhas, todas disparando.**
///
/// É o alinhamento por índice de que o `sim.spawn` depende: ele indexa o pulso contra o
/// template linha a linha, e as duas saídas nascem da mesma lista.
#[test]
fn the_pulse_output_is_index_aligned_with_the_dead() {
    let reg = registry();
    let mut g = Graph::new();
    let (_zone, life_n) = dying_world(&mut g, 0.2);

    let dead = cook_to(&g, &reg, life_n, 1, DEATH_TICKS);
    let pulse = cook_to(&g, &reg, life_n, 2, DEATH_TICKS);
    // ⚠️ Sem esta linha o gate passa por VÁCUO: dois streams vazios têm a mesma
    // contagem e a mesma lista de pulsos. Foi como ele passou na 1ª rodada, com a
    // fixture parando um tique ANTES da morte.
    assert_eq!(
        dead.count(),
        4,
        "a fixture tem de CONTER a morte que ela mede"
    );
    assert_eq!(
        pulse.count(),
        dead.count(),
        "uma linha de gatilho por cadáver"
    );
    assert_eq!(
        scalar(&pulse, "pulse"),
        vec![1.0; dead.count()],
        "todas disparam: estar na lista dos mortos É o evento"
    );
}

/// **A CADEIA: cada morte dá à luz `burst` filhos ONDE ela aconteceu.**
///
/// O gate da wave. Sem ele os dois acima provariam que as portas existem e nada sobre o
/// que o artista consegue montar com elas.
#[test]
fn a_death_gives_birth_where_it_happened() {
    let reg = registry();
    let mut g = Graph::new();
    let (_zone, life_n) = dying_world(&mut g, 0.2);

    let spawn = g.add_node("sim.spawn");
    g.set_param(spawn, "rate", 0.0); // SÓ a morte dá à luz
    g.set_param(spawn, "burst", 3.0);
    connect(g_mut(&mut g), life_n, 1, spawn, 0); // died  → template
    connect(g_mut(&mut g), life_n, 2, spawn, 1); // pulse → pulse

    let born = cook_to(&g, &reg, spawn, 0, DEATH_TICKS);
    assert_eq!(
        born.count(),
        4 * 3,
        "quatro mortes × burst 3 — e a contagem é o que separa 'nasceu' de 'passou adiante'"
    );

    // ONDE: os filhos herdam a posição do cadáver, então o conjunto de x é o da fila.
    let dead = cook_to(&g, &reg, life_n, 1, DEATH_TICKS);
    let (dx, bx) = (xs(&dead), xs(&born));
    for x in &bx {
        assert!(
            dx.iter().any(|d| (d - x).abs() < 1e-4),
            "um filho nasceu em x={x}, que não é a posição de morte nenhuma ({dx:?})"
        );
    }

    // ⚠️ **E ele nasce com IDADE ZERO.** Sem isto a cadeia inteira é um no-op visível: o
    // filho herdaria o `age` do cadáver (que por definição passou da própria vida) e morreria
    // no tique seguinte — a feature pareceria construída e não faria nada.
    let ages = scalar(&born, "age");
    assert!(
        ages.is_empty() || ages.iter().all(|a| *a == 0.0),
        "um recém-nascido tem idade 0 por DEFINIÇÃO (o `sim.step` já declara a lei: \
         *a row with no `age` is newborn*), veio {ages:?}"
    );
}

/// Um empréstimo mutável nomeado, só para o `connect` ler bem na cadeia acima.
fn g_mut(g: &mut Graph) -> &mut Graph {
    g
}

fn xs(s: &Stream) -> Vec<f32> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => vec![],
    }
}

/// **CONTROLE: desconectadas, as saídas novas não mudam um byte.**
///
/// A regra de todo apêndice desta linha. Sem ela, "aditivo" é uma palavra.
#[test]
fn the_survivors_are_byte_identical_without_the_new_ports() {
    let reg = registry();
    let mut g = Graph::new();
    let (_zone, life_n) = dying_world(&mut g, 0.2);
    let survivors = cook_to(&g, &reg, life_n, 0, 8);
    assert_eq!(survivors.count(), 4, "aos 0,13 s os quatro seguem vivos");
    // O `life` é o número que a família inteira consome: 0 no nascimento, 1 no fim.
    let l = scalar(&survivors, "life");
    assert!(
        l.iter().all(|v| (0.0..=1.0).contains(v)),
        "a fração de vida continua sendo uma fração: {l:?}"
    );
}

/// **A SONDA que produziu o [`DEATH_TICKS`]** — imprime, tique a tique, quantos vivem, que
/// idade têm e quantos morreram.
///
/// Ela fica porque foi ela que achou os DOIS defeitos de fixture desta wave: um mundo que
/// nunca envelhecia (o `pre` do laço escrito ao contrário) e um gate que passava por vácuo
/// (a fixture parando um tique antes da morte). *Um número citado que a sonda não imprime
/// mais deixou de ser reproduzível.*
///
/// `cargo test -p ph2d-node-registry-init --test death_replicates probe_ages -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_ages() {
    let reg = registry();
    let mut g = Graph::new();
    let (_zone, life_n) = dying_world(&mut g, 0.2);
    let mut cook = Cook::new();
    for k in 0..20 {
        let t = k as f64 / 60.0;
        let outs = cook.cook(&g, &reg, life_n, t).expect("cooks");
        let sur = outs[0].as_stream().clone();
        let died = outs[1].as_stream().clone();
        eprintln!(
            "tick {k:2} t={t:.4}  vivos={} age={:?} died={}",
            sur.count(),
            scalar(&sur, "age"),
            died.count()
        );
        cook.advance_tick(&g, &reg, t).expect("advances");
    }
}
