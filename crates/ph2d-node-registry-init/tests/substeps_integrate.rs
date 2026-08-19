//! **O SUB-PASSO DO `motion.integrate`** (doc 89, folha 17 — a linha 76).
//!
//! A célula pedia *sub-steps / o timestep exposto* e listava quatro rotas tentadas e recusadas.
//! **Duas delas caíram quatro dias depois** e a nota nunca foi reconferida (§0 do `CLAUDE.md`):
//! o motor do sub-tique aterrou em 2026-08-12 (`ph2d_nodegraph::cook::Cook::substep`, folha 13) e
//! **nada nele sabe o que é uma zona** — a declaração é uma convenção de manifesto, e o `dt` deste
//! nó é `playhead − sim_t`, exactamente a grandeza que subdividir o playhead subdivide.
//!
//! Então o que faltava não era mecanismo, era o nó **declarar**. Estes gates provam as duas
//! metades do que a declaração compra e a que ela não pode mexer.
//!
//! ⚠️ **O irmão destes gates é o censo** em `substeps.rs`
//! (`only_the_declared_clock_owners_offer_the_substeps_param`): o ritmo é do GRAFO, então quem
//! declara escolhe o relógio de toda a gente, e a lista de quem pode fazê-lo é fechada.

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

/// A cadeia canônica: `grid → integrate.rest`, com o laço
/// `integrate.out =pre=> force.wind =fwd=> integrate.forces`. A rajada é ZERO de propósito — assim
/// a resposta exata é `a·T²/2` e o erro do integrador é o único desvio.
fn falling_chain(g: &mut Graph) -> NodeId {
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);

    let integ = g.add_node("motion.integrate");
    let wind = g.add_node("force.wind");
    g.set_param(wind, "angle", 0.0);
    g.set_param(wind, "strength", ACCEL);
    g.set_param(wind, "gust", 0.0);

    wire(g, seed, 0, integ, 0, false);
    wire(g, integ, 0, wind, 0, true);
    wire(g, wind, 0, integ, 1, false);
    integ
}

/// A aceleração da fixture (unidades de mundo/s²), e a resposta exata a 1 s é `ACCEL/2`.
const ACCEL: f32 = 40.0;

fn px(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
        _ => f32::NAN,
    }
}

/// Marcha 1 s a 60 fps pedindo `sub` sub-passadas ao alvo.
///
/// ⚠️ **O primeiro tique PULA, e isso é a marcha do pump, não um atalho.** Ali
/// `Cook::prev_playhead()` é `None` — não há começo de quadro honesto a subdividir, e o
/// integrador ainda está a SEMEAR (o `pre` chega vazio). Um helper que substepasse o 1º quadro
/// mediria outra marcha que a do app, e a comparação com `run_via_pump` deixaria de ser ao bit.
fn run(g: &Graph, reg: &NodeRegistry, target: NodeId, sub: u32) -> f32 {
    let mut cook = Cook::new();
    let mut last = f32::NAN;
    for k in 0..60u64 {
        let t = (k + 1) as f64 / 60.0;
        if let Some(frame_start) = cook.prev_playhead() {
            cook.substep(g, reg, target, frame_start, t, sub)
                .expect("substeps");
        }
        last = px(cook.cook(g, reg, target, t).expect("coza")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

/// **A ENTREGA: o erro cai pela METADE a cada dobra** — a assinatura de Euler de 1ª ordem, e a
/// prova de que o bracket INTEGRA em vez de só re-cozinhar.
///
/// FALSIFICADO se a declaração fosse decorativa: as seis linhas seriam o mesmo número.
///
/// ⚠️ **O alvo analítico NÃO é `a·T²/2`, e o gate nasceu vermelho por isso.** O primeiro tique é o
/// de SEMEADURA (o `pre` chega vazio, `sim_d = 0`) e o pump não o substepa, então o corpo começa
/// a cair em `t = 1/60`, não em `0` — a queda é de `59/60 s` e a resposta exata é
/// `a/2·(59/60)²`. Contra os `20,0` ingênuos as sub-passadas **afastavam-se** (0,3333 → 0,4972),
/// e a leitura errada teria sido *"o substep piora o integrador"*. Contra o alvo certo elas
/// convergem 2,000×. *Uma fixture só prova o que contém — e esta contém a semeadura.*
#[test]
fn the_substep_converges_first_order_on_the_integrator() {
    let reg = registry();
    let mut g = Graph::new();
    let integ = falling_chain(&mut g);
    g.validate(&reg).expect("bem-tipado");

    let fall = 1.0 - 1.0 / 60.0; // o tique de semeadura não integra
    let exact = ACCEL / 2.0 * fall * fall;
    let mut prev_err = f32::INFINITY;
    for sub in [1u32, 2, 4, 8, 16, 32] {
        let err = (run(&g, &reg, integ, sub) - exact).abs();
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
        prev_err < 0.02,
        "32 passadas têm de chegar perto da analítica; erro {prev_err:.4}"
    );
}

/// **O DEFAULT DE 1 É O MUNDO DE ANTES, AO BIT.** Um param novo que mexesse no quadro em repouso
/// seria uma regressão em toda cena já autorada — e a fila de undo do editor vê a diferença.
///
/// ⚠️ O caminho do **override escrito à mão** e o do **default do manifesto** têm de pousar no
/// mesmo bit: um `1` que resolvesse por outra porta seria um bug invisível até alguém digitar 1.
#[test]
fn the_default_of_one_leaves_the_integrator_byte_identical() {
    let reg = registry();

    let mut g = Graph::new();
    let implicit_node = falling_chain(&mut g);
    g.validate(&reg).expect("bem-tipado");
    let implicit = run(&g, &reg, implicit_node, 1);

    let mut g2 = Graph::new();
    let explicit_node = falling_chain(&mut g2);
    g2.set_param(explicit_node, "substeps", 1.0);
    g2.validate(&reg).expect("bem-tipado");
    let explicit = run(&g2, &reg, explicit_node, 1);

    assert_eq!(implicit.to_bits(), explicit.to_bits());

    // O CONTROLE de que a fixture contém o fenômeno: com 4 o mesmo grafo TEM de dar outros bits,
    // senão a igualdade acima é verde porque o param é inerte.
    let four = run(&g, &reg, implicit_node, 4);
    assert_ne!(
        implicit.to_bits(),
        four.to_bits(),
        "o CONTROLE: quatro sub-passadas têm de mudar a resposta"
    );
}

/// Marcha pelo caminho do PUMP — ilhas descobertas no grafo, nada de `n` escrito à mão.
/// É a rota literal do `MotionCookPump::substep_declared_zones`.
fn run_via_pump(g: &Graph, reg: &NodeRegistry, target: NodeId) -> f32 {
    let mut cook = Cook::new();
    let mut last = f32::NAN;
    for k in 0..60u64 {
        let t = (k + 1) as f64 / 60.0;
        if let Some(frame_start) = cook.prev_playhead() {
            for island in ph2d_nodegraph::cook::substep_islands(g, reg) {
                cook.substep(g, reg, island.root, frame_start, t, island.substeps)
                    .expect("substep");
            }
        }
        last = px(cook.cook(g, reg, target, t).expect("coza")[0].as_stream());
        cook.advance_tick(g, reg, t).expect("tick");
    }
    last
}

/// **É A DECLARAÇÃO QUE FAZ O PUMP SUB-PASSAR ESTE NÓ** — e sem este gate os outros só provariam
/// que o motor funciona quando alguém lhe passa um `n` à mão.
///
/// ⚠️ A distinção é exactamente onde a folha 17 se enganou: o mecanismo já existia, o que faltava
/// era o nó **entrar na lista de quem o sequenciador procura**. Um gate que chama `Cook::substep`
/// directamente passa verde num mundo em que o `params` do manifesto está vazio.
///
/// ⚠️ **O `frame_start` do pump é `None` no primeiro tique de todos e ele PULA** — a rota forçada
/// pula pelo mesmo teste, então as duas usam a mesma marcha e a única diferença é de onde vem o
/// `n`. Foi assim que este gate nasceu vermelho: a 1ª versão substepava o 1º quadro à mão e
/// discordava do pump por um quadro grosso, não por causa da declaração.
#[test]
fn the_declaration_is_what_makes_the_pump_substep_it() {
    let reg = registry();

    let mut g = Graph::new();
    let integ = falling_chain(&mut g);
    g.set_param(integ, "substeps", 8.0);
    g.validate(&reg).expect("bem-tipado");
    let declared = run_via_pump(&g, &reg, integ);

    // O mesmo grafo, com o `8` entregue à mão ao motor: o pump tem de ter descoberto o MESMO
    // número, pela leitura do manifesto + override.
    let mut g8 = Graph::new();
    let i8 = falling_chain(&mut g8);
    g8.validate(&reg).expect("bem-tipado");
    assert_eq!(
        declared.to_bits(),
        run(&g8, &reg, i8, 8).to_bits(),
        "o pump tem de sub-passar o integrador 8× só por o documento pedir 8"
    );

    // E o CONTROLE: sem o override, o pump acha `1` e a marcha é a de sempre.
    let mut g1 = Graph::new();
    let i1 = falling_chain(&mut g1);
    g1.validate(&reg).expect("bem-tipado");
    assert_eq!(
        run_via_pump(&g1, &reg, i1).to_bits(),
        run(&g1, &reg, i1, 1).to_bits(),
        "um documento que não pede sub-passos marcha como sempre marchou"
    );
    assert_ne!(
        declared.to_bits(),
        run(&g1, &reg, i1, 1).to_bits(),
        "o CONTROLE: os dois números TÊM de diferir, senão a igualdade acima é vazia"
    );
}

/// **DOIS DECLARANTES, UM RELÓGIO — então os TETOS têm de ser o mesmo número.**
///
/// O ritmo é do grafo: todas as ilhas correm no maior que qualquer declarante pede. Se o
/// `motion.integrate` parasse em 32 e a `sim.zone` fosse até 64, uma zona ao lado contornaria o
/// teto do integrador — e um teto contornável é uma mentira no painel, não um limite.
///
/// ⚠️ Isto é um gate de CONSISTÊNCIA, não de valor: ele não diz que 64 é certo (a tabela medida
/// está no `MAX_SUBSTEPS` de cada crate), diz que os dois não podem divergir em silêncio.
#[test]
fn the_two_clock_declarers_share_one_ceiling() {
    let reg = registry();
    let param = ph2d_nodegraph::cook::SUBSTEPS_PARAM;
    let integ = reg
        .param_hard_max(ph2d_node_motion_integrate::MANIFEST.id, param)
        .expect("o integrador declara um teto digitável");
    let zone = reg
        .param_hard_max(ph2d_node_sim_zone::MANIFEST.id, param)
        .expect("a zona declara um teto digitável");
    assert_eq!(
        integ, zone,
        "os dois declarantes partilham o relógio do grafo, então um teto menor num deles é \
         contornável pelo outro: integrate={integ}, zone={zone}"
    );

    // E a faixa CONFORTÁVEL do arrasto também, pelo mesmo motivo.
    let drag = |id| {
        reg.param_ui(id)
            .and_then(|h| h.iter().find(|h| h.param == param))
            .map(|h| h.max)
            .expect("o declarante tem hint de slider")
    };
    assert_eq!(
        drag(ph2d_node_motion_integrate::MANIFEST.id),
        drag(ph2d_node_sim_zone::MANIFEST.id),
        "a faixa de arrasto dos dois declarantes tem de coincidir"
    );
}
