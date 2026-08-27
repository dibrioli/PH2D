//! As provas do plano de preguiça — **cada condição com o seu recuo**.
//!
//! ⚠️ A régua destes gates é sempre a mesma: *quando a condição falha, o ramo NÃO é declarado
//! saltável*. Um plano que declarasse a mais não dá erro nenhum — dá uma simulação parada no
//! passado ou um resultado com a contagem errada, as duas coisas que só se vêem no smoke.

use super::*;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::Clock;

/// Três nós de teste, um por EFEITO — é o que torna a condição do estado mensurável sem
/// arrastar meio catálogo para as dependências deste crate (e o `registry-init` nem podia
/// entrar: ele depende de todos os nós, incluindo este).
macro_rules! stub {
    ($ty:ident, $name:literal, $effect:expr) => {
        struct $ty;
        impl NodeOp for $ty {
            fn manifest(&self) -> &'static NodeManifest {
                static M: NodeManifest = NodeManifest {
                    id: NodeTypeId::of($name),
                    name: $name,
                    inputs: &[
                        PortSpec {
                            name: "a",
                            ty: crate::VALUE,
                        },
                        PortSpec {
                            name: "b",
                            ty: crate::VALUE,
                        },
                    ],
                    outputs: &[PortSpec {
                        name: "out",
                        ty: crate::VALUE,
                    }],
                    effect: $effect,
                    clock: Clock::Frame,
                    params: &[ph2d_nodegraph::node::ParamSpec {
                        name: "a",
                        default: 0.0,
                    }],
                    lowerings: &[ph2d_nodegraph::node::LoweringKind::Cpu],
                };
                &M
            }
            fn eval(&self, ctx: &mut ph2d_nodegraph::cook::EvalCtx<'_>) {
                ctx.emit(ph2d_nodegraph::attr::Stream::new(0));
            }
        }
    };
}

stub!(PureOp, "value.switch.test.pure", Effect::Pure);
stub!(TemporalOp, "value.switch.test.temporal", Effect::Temporal);
stub!(StatefulOp, "value.switch.test.stateful", Effect::Stateful);

const PURE: &str = "value.switch.test.pure";
const TEMPORAL: &str = "value.switch.test.temporal";
const STATEFUL: &str = "value.switch.test.stateful";

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    crate::register(&mut reg).expect("o switch regista");
    reg.register(Box::new(PureOp)).expect("pure");
    reg.register(Box::new(TemporalOp)).expect("temporal");
    reg.register(Box::new(StatefulOp)).expect("stateful");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed,
    })
    .expect("liga");
}

/// Um switch com o modo LIGADO e quatro ramos puros vindos de `value.const`-like.
fn scene(lazy: f32) -> (Graph, NodeId, Vec<NodeId>) {
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, lazy);
    let mut branches = Vec::new();
    for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
        let n = g.add_node(PURE);
        g.set_param(n, "a", k as f32);
        wire(&mut g, n, sw, *port, false);
        branches.push(n);
    }
    (g, sw, branches)
}

#[test]
fn the_mode_is_off_by_default_and_an_off_switch_is_not_in_the_plan() {
    let reg = registry();
    let (g, _, _) = scene(0.0);
    assert!(
        super::plan(&g, &reg).is_empty(),
        "com o modo desligado o plano tem de ficar VAZIO — vazio e' o caminho de sempre"
    );
    // E o default do manifesto é o desligado.
    let d = crate::MANIFEST
        .params
        .iter()
        .find(|p| p.name == crate::LAZY)
        .expect("o param existe");
    assert_eq!(d.default, 0.0);
}

#[test]
fn an_on_switch_with_pure_branches_declares_all_four_skippable() {
    let reg = registry();
    let (g, sw, _) = scene(1.0);
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("o switch entrou no plano");
    assert_eq!(e.select_port, crate::SELECT_PORT);
    assert_eq!(e.choices, crate::CHOICE_PORTS);
    assert!(
        e.skippable[..crate::CHOICE_PORTS.len()].iter().all(|b| *b),
        "quatro ramos puros e nenhum declarado saltavel: {:?}",
        e.skippable
    );
}

/// **UMA ARESTA `pre` NO CONE TIRA O RAMO DO PLANO** — a realimentação é o que congela.
#[test]
fn a_feedback_edge_anywhere_in_the_cone_makes_the_branch_unskippable() {
    let reg = registry();
    // (a) o `pre` na PRÓPRIA aresta do ramo.
    let (mut g, sw, branches) = scene(1.0);
    let extra = g.add_node(PURE);
    g.disconnect(sw, crate::CHOICE_PORTS[2]);
    wire(&mut g, extra, sw, crate::CHOICE_PORTS[2], true);
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("no plano");
    assert!(!e.skippable[2], "um `pre` na aresta do ramo passou");
    assert!(
        e.skippable[0] && e.skippable[1],
        "os outros ramos regrediram"
    );

    // (b) o `pre` mais ACIMA, dentro do cone — é o caso que uma verificação de um nível
    //     de profundidade deixaria passar.
    let (mut g, sw, branches2) = scene(1.0);
    let deep = g.add_node(PURE);
    let feeder = g.add_node(PURE);
    g.connect(Edge {
        from: (feeder, 0),
        to: (deep, 0),
        delayed: true,
    })
    .expect("liga");
    wire(&mut g, deep, branches2[1], 0, false);
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("no plano");
    assert!(!e.skippable[1], "um `pre` a dois niveis do ramo passou");
    assert!(e.skippable[0], "o ramo vizinho regrediu");
    let _ = branches;
}

/// **UM PARAM CONDUZIDO É PARTE DO CONE** (doc 58) — um fio que conduz um número é uma
/// dependência tão real quanto uma porta.
#[test]
fn a_driven_param_is_part_of_the_cone() {
    let reg = registry();
    let (mut g, sw, branches) = scene(1.0);
    let driver = g.add_node(PURE);
    let feedback = g.add_node(PURE);
    g.connect(Edge {
        from: (feedback, 0),
        to: (driver, 0),
        delayed: true,
    })
    .expect("liga");
    g.drive_param(branches[3], "a", (driver, 0))
        .expect("o param conduzido liga — senao este gate era VAZIO");
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("no plano");
    assert!(
        !e.skippable[3],
        "o cone do param conduzido nao foi visitado — um driver com realimentacao passou"
    );
}

/// Uma porta SEM aresta é saltável por vacuidade — não há nada para cozinhar.
#[test]
fn an_unwired_branch_is_skippable_by_vacuity() {
    let reg = registry();
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, 1.0);
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("no plano");
    assert!(e.skippable[..crate::CHOICE_PORTS.len()].iter().all(|b| *b));
}

/// **A LEI DE QUAIS RAMOS SÃO PRECISOS** — a do roteamento e a da mistura, incluindo o colapso
/// do par em `t == 0`.
#[test]
fn the_needed_law_matches_what_the_node_actually_reads() {
    let mut m = [false; 4];
    crate::needed_round(0.0, &mut m);
    assert_eq!(m, [true, false, false, false]);
    crate::needed_round(2.4, &mut m);
    assert_eq!(m, [false, false, true, false], "arredonda ao mais proximo");
    crate::needed_round(-5.0, &mut m);
    assert_eq!(m, [true, false, false, false], "grampeia em baixo");
    crate::needed_round(99.0, &mut m);
    assert_eq!(m, [false, false, false, true], "grampeia em cima");

    crate::needed_blend(1.5, &mut m);
    assert_eq!(m, [false, true, true, false], "a mistura precisa do PAR");
    crate::needed_blend(1.0, &mut m);
    assert_eq!(
        m,
        [false, true, false, false],
        "em t == 0 o par colapsa: o no' devolve `a` verbatim"
    );
    crate::needed_blend(3.7, &mut m);
    assert_eq!(m, [false, false, false, true], "o topo grampeia nos dois");
}

/// **O plano nunca declara mais candidatas do que a máscara do cook segura.**
#[test]
fn a_lazy_router_declares_no_more_choices_than_the_mask_holds() {
    assert!(
        crate::CHOICE_PORTS.len() <= MAX_LAZY_CHOICES,
        "{} candidatas contra um tecto de {MAX_LAZY_CHOICES}",
        crate::CHOICE_PORTS.len()
    );
    // E as portas declaradas são as do MANIFESTO, na ordem dele — senão a lei do nó indexaria
    // uma coisa e o cook saltaria outra.
    let names: Vec<&str> = crate::CHOICE_PORTS
        .iter()
        .map(|p| crate::MANIFEST.inputs[*p as usize].name)
        .collect();
    assert_eq!(names, vec!["in0", "in1", "in2", "in3"]);
    assert_eq!(
        crate::MANIFEST.inputs[crate::SELECT_PORT as usize].name,
        "select"
    );
}

/// **UM NÓ COM ESTADO NO CONE TIRA O RAMO DO PLANO — e um `Temporal` NÃO.**
///
/// ⚠️ **Esta é a assimetria que a cerca tem de nomear certo.** O que congela não é «ler o
/// relógio»: um oscilador é função pura do playhead e recalcula-se no tique em que voltarem a
/// pedi-lo. O que congela é a REALIMENTAÇÃO — uma aresta `pre` (gate irmão) ou um nó que declara
/// mutar estado. Recusar o `Temporal` seria conservador e, pior, ensinaria o mecanismo errado a
/// quem ler a cerca a seguir.
#[test]
fn stateful_in_the_cone_blocks_the_branch_and_temporal_does_not() {
    let reg = registry();
    let (mut g, sw, branches) = scene(1.0);
    let st = g.add_node(STATEFUL);
    wire(&mut g, st, branches[2], 0, false);
    let tm = g.add_node(TEMPORAL);
    wire(&mut g, tm, branches[0], 0, false);
    let plan = super::plan(&g, &reg);
    let e = plan.get(&sw).expect("no plano");
    assert!(!e.skippable[2], "um no' com ESTADO no cone passou");
    assert!(
        e.skippable[0],
        "um `Temporal` foi recusado — a cerca esta' a nomear o mecanismo errado"
    );
    assert!(
        e.skippable[1] && e.skippable[3],
        "os ramos limpos regrediram"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// O QUE O COOK DE FACTO FAZ — a metade que conta AVALIAÇÕES em vez de inspeccionar o plano.
//
// ⚠️ **Um plano correcto não é uma preguiça correcta.** Os gates acima medem o que o construtor
// DECLARA; estes medem o que o escalonador EXECUTA, contando quantas vezes cada ramo foi
// avaliado. É a diferença entre «o trabalho foi planeado» e «o trabalho foi feito» — e a régua
// tem de estar no consumidor.
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::atomic::{AtomicUsize, Ordering};

static EVALS: [AtomicUsize; 4] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];

/// Uma fonte que EMITE o próprio índice e conta quantas vezes foi avaliada.
struct Counted(usize);
impl NodeOp for Counted {
    fn manifest(&self) -> &'static NodeManifest {
        // Quatro manifestos distintos (um por índice) — o registry chaveia por tipo.
        static M: [NodeManifest; 4] = [
            counted_manifest("value.switch.test.c0"),
            counted_manifest("value.switch.test.c1"),
            counted_manifest("value.switch.test.c2"),
            counted_manifest("value.switch.test.c3"),
        ];
        &M[self.0]
    }
    fn eval(&self, ctx: &mut ph2d_nodegraph::cook::EvalCtx<'_>) {
        EVALS[self.0].fetch_add(1, Ordering::Relaxed);
        ctx.emit(ph2d_nodegraph::attr::Stream::new(1).with(
            crate::SELECT_COLUMN,
            #[expect(clippy::cast_precision_loss, reason = "0..4")]
            ph2d_nodegraph::attr::Column::Scalar(vec![self.0 as f32 * 10.0]),
        ));
    }
}

const fn counted_manifest(name: &'static str) -> NodeManifest {
    NodeManifest {
        id: NodeTypeId::of(name),
        name,
        // ⚠️ **Uma porta de entrada, mesmo sem a usar** — sem ela um nó não tem CONE, e o gate
        // do ramo com estado ligava o nó de estado a uma porta que o manifesto não declara: a
        // travessia iterava `0..0`, não via a aresta, e o gate reprovava sobre produto correcto.
        // *Uma fixtura sem a forma que a lei percorre mede a fixtura.*
        inputs: &[PortSpec {
            name: "up",
            ty: crate::VALUE,
        }],
        outputs: &[PortSpec {
            name: "out",
            ty: crate::VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[ph2d_nodegraph::node::LoweringKind::Cpu],
    }
}

const COUNTED: [&str; 4] = [
    "value.switch.test.c0",
    "value.switch.test.c1",
    "value.switch.test.c2",
    "value.switch.test.c3",
];

/// Um switch com quatro fontes CONTADAS e um `select` autorado por um nó próprio.
/// `select_len` > 1 monta um `select` POR ELEMENTO (a condição que recusa a preguiça).
fn counted_scene(lazy: f32, select: Vec<f32>) -> (Graph, NodeRegistry, NodeId) {
    let mut reg = NodeRegistry::new();
    crate::register(&mut reg).expect("switch");
    for k in 0..4 {
        reg.register(Box::new(Counted(k))).expect("counted");
    }
    reg.register(Box::new(SelectSrc(std::sync::Mutex::new(select))))
        .expect("select");
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, lazy);
    for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
        let n = g.add_node(COUNTED[k]);
        wire(&mut g, n, sw, *port, false);
    }
    let sel = g.add_node("value.switch.test.select");
    wire(&mut g, sel, sw, crate::SELECT_PORT, false);
    (g, reg, sw)
}

/// A fonte do `select` — devolve o campo que lhe deram (um valor = uniforme, N = por elemento).
struct SelectSrc(std::sync::Mutex<Vec<f32>>);
impl NodeOp for SelectSrc {
    fn manifest(&self) -> &'static NodeManifest {
        static M: NodeManifest = counted_manifest("value.switch.test.select");
        &M
    }
    fn eval(&self, ctx: &mut ph2d_nodegraph::cook::EvalCtx<'_>) {
        let v = self.0.lock().expect("lock").clone();
        ctx.emit(ph2d_nodegraph::attr::Stream::new(v.len()).with(
            crate::SELECT_COLUMN,
            ph2d_nodegraph::attr::Column::Scalar(v),
        ));
    }
}

/// ⚠️ **Os contadores são GLOBAIS, então as corridas serializam-se.** Sem isto duas provas em
/// paralelo somam no mesmo balde e a leitura mistura-as — foi exactamente o que a 1.ª versão
/// destes gates mediu (`[1,1,2,2]` onde só podia haver um por ramo). *Um contador partilhado é
/// um recurso, e um recurso partilhado num arnês paralelo mede a partilha.*
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn run(lazy: f32, select: Vec<f32>) -> [usize; 4] {
    let _guard = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for e in &EVALS {
        e.store(0, Ordering::Relaxed);
    }
    let (g, reg, sw) = counted_scene(lazy, select);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    if lazy >= 0.5 {
        cook.set_lazy_branches(super::plan(&g, &reg));
    }
    cook.cook(&g, &reg, sw, 0.0).expect("coze");
    [
        EVALS[0].load(Ordering::Relaxed),
        EVALS[1].load(Ordering::Relaxed),
        EVALS[2].load(Ordering::Relaxed),
        EVALS[3].load(Ordering::Relaxed),
    ]
}

/// **O MODO DESLIGADO PUXA AS QUATRO** — o comportamento de sempre, e o controle de que a
/// contagem mede alguma coisa.
#[test]
fn with_the_mode_off_every_branch_is_still_cooked() {
    assert_eq!(run(0.0, vec![2.0]), [1, 1, 1, 1]);
}

/// **O MODO LIGADO PUXA UM SÓ** — o que a célula pedia, e o que o Blender documenta.
#[test]
fn with_the_mode_on_only_the_chosen_branch_is_cooked() {
    assert_eq!(run(1.0, vec![2.0]), [0, 0, 1, 0], "select 2");
    assert_eq!(run(1.0, vec![0.0]), [1, 0, 0, 0], "select 0");
    // E o grampeamento é o do nó: um select fora da faixa escolhe a ponta, não nada.
    assert_eq!(run(1.0, vec![99.0]), [0, 0, 0, 1], "grampeia em cima");
}

/// **UM `select` POR ELEMENTO RECUSA A PREGUIÇA** — a primeira condição, medida no cook.
///
/// ⚠️ É a condição mais fácil de esquecer porque ela é uma feature *documentada* deste nó: com
/// um campo de selecção, cada elemento escolhe o seu ramo, logo **nenhum** ramo é dispensável.
/// Uma preguiça que olhasse só para o primeiro valor entregaria o ramo errado a todos os outros
/// elementos — em silêncio, com o número certo de elementos e os valores errados.
#[test]
fn a_per_element_select_falls_back_to_cooking_everything() {
    assert_eq!(
        run(1.0, vec![0.0, 1.0, 2.0]),
        [1, 1, 1, 1],
        "um select por elemento tem de recuar para o caminho de sempre"
    );
    // Mas um campo de N valores TODOS IGUAIS é uniforme, e a preguiça vale.
    assert_eq!(run(1.0, vec![3.0, 3.0, 3.0]), [0, 0, 0, 1]);
}

/// **A CONDIÇÃO DO ESTADO, MEDIDA NO COOK** — não basta o plano declará-la.
///
/// ⚠️ **Este gate nasceu de uma mutação que SOBREVIVEU.** O irmão
/// `stateful_in_the_cone_blocks_the_branch_and_temporal_does_not` inspecciona o PLANO, e por
/// isso ficou verde quando o escalonador passou a saltar só pela lei do nó, ignorando a metade
/// do estado (`skip[k] = !needed[k]` em vez de `!needed[k] && lazy.skippable[k]`). *Um gate
/// sobre a declaração é verde quando o executor a ignora* — a régua tem de contar avaliações.
#[test]
fn a_stateful_branch_is_cooked_even_when_it_is_not_chosen() {
    let _guard = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for e in &EVALS {
        e.store(0, Ordering::Relaxed);
    }
    let mut reg = NodeRegistry::new();
    crate::register(&mut reg).expect("switch");
    for k in 0..4 {
        reg.register(Box::new(Counted(k))).expect("counted");
    }
    reg.register(Box::new(StatefulOp)).expect("stateful");
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, 1.0);
    for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
        let n = g.add_node(COUNTED[k]);
        wire(&mut g, n, sw, *port, false);
    }
    // O ramo 3 ganha um nó com ESTADO acima dele. O `select` fica desligado ⇒ escolhe o 0.
    let st = g.add_node(STATEFUL);
    let branch3 = g
        .input_edge(sw, crate::CHOICE_PORTS[3] as usize)
        .expect("o ramo 3 esta' ligado")
        .0;
    wire(&mut g, st, branch3, 0, false);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.set_lazy_branches(super::plan(&g, &reg));
    cook.cook(&g, &reg, sw, 0.0).expect("coze");
    let n = [
        EVALS[0].load(Ordering::Relaxed),
        EVALS[1].load(Ordering::Relaxed),
        EVALS[2].load(Ordering::Relaxed),
        EVALS[3].load(Ordering::Relaxed),
    ];
    assert_eq!(n[0], 1, "o ramo escolhido tem de cozinhar");
    assert_eq!(
        n[3], 1,
        "o ramo com ESTADO foi saltado — no tique seguinte ele estaria parado no passado"
    );
    assert_eq!(
        (n[1], n[2]),
        (0, 0),
        "os ramos puros nao escolhidos tinham de ser saltados"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A RÉGUA QUE FALTAVA: a RESPOSTA, e não o plano.
//
// ⚠️ **Os gates acima medem todos o PLANO** (`skippable`, `plan().len()`), e a auditoria de
// 2026-08-27 mostrou o preço disso: um plano perfeitamente formado ainda escolhe a LEI ERRADA
// se o param que a escolhe chegar por um fio. Nenhuma asserção sobre `skippable` podia ver isso,
// porque o defeito não estava em que ramos são saltáveis — estava em **quantos** a lei pede.
// *Um gate sobre a declaração é verde quando o executor lê outra coisa.*
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Uma fonte de um `1.0` — o DRIVER de um param. (Reusa o manifesto dos contadores: uma porta de
/// entrada, uma de saída, `Pure`.)
struct OneSrc;
impl NodeOp for OneSrc {
    fn manifest(&self) -> &'static NodeManifest {
        static M: NodeManifest = counted_manifest(ONE);
        &M
    }
    fn eval(&self, ctx: &mut ph2d_nodegraph::cook::EvalCtx<'_>) {
        ctx.emit(ph2d_nodegraph::attr::Stream::new(1).with(
            crate::SELECT_COLUMN,
            ph2d_nodegraph::attr::Column::Scalar(vec![1.0]),
        ));
    }
}
const ONE: &str = "value.switch.test.one";

/// Por onde o `blend` chega ao nó nesta corrida — e as três rotas **não** são equivalentes:
/// `Driven` é a única que o construtor do plano não consegue ler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Blend {
    Off,
    Authored,
    Driven,
}

/// Coze o roteador e devolve a **coluna de saída**. Os ramos emitem `0 · 10 · 20 · 30`, então o
/// valor identifica sozinho *qual* ramo o nó leu — e um ramo saltado que ele precisava aparece
/// como `0`, que é o que o `CookValue::Empty` se lê.
fn cooked(lazy: bool, blend: Blend, select: Vec<f32>) -> Vec<f32> {
    let _guard = SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut reg = NodeRegistry::new();
    crate::register(&mut reg).expect("switch");
    for k in 0..4 {
        reg.register(Box::new(Counted(k))).expect("counted");
    }
    reg.register(Box::new(SelectSrc(std::sync::Mutex::new(select))))
        .expect("select");
    reg.register(Box::new(OneSrc)).expect("one");
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, if lazy { 1.0 } else { 0.0 });
    match blend {
        Blend::Off => {}
        Blend::Authored => g.set_param(sw, crate::BLEND, 1.0),
        Blend::Driven => {
            let d = g.add_node(ONE);
            g.drive_param(sw, crate::BLEND, (d, 0))
                .expect("o fio conduz — senao este gate era VAZIO");
        }
    }
    for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
        let n = g.add_node(COUNTED[k]);
        wire(&mut g, n, sw, *port, false);
    }
    let sel = g.add_node("value.switch.test.select");
    wire(&mut g, sel, sw, crate::SELECT_PORT, false);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    if lazy {
        cook.set_lazy_branches(super::plan(&g, &reg));
    }
    let out = cook.cook(&g, &reg, sw, 0.0).expect("coze");
    match out.first() {
        Some(ph2d_nodegraph::value::CookValue::Instances(s)) => {
            crate::scalar_col(s, crate::SELECT_COLUMN)
        }
        _ => Vec::new(),
    }
}

/// ⛔⛔ **O `blend` CONDUZIDO POR FIO — o defeito que shipava.**
///
/// O plano nasce **antes** do cozimento e lê `node_param_overrides`; o `EvalCtx::param` lê o
/// **conduzido** primeiro. Com o `blend` num fio e sem override, o plano instalava `needed_round`
/// (um ramo) enquanto o `eval` misturava **dois** — e o não-marcado chegava `Empty`, isto é `0.0`.
///
/// A régua é o **valor**: `select = 1.5` mistura `in1 = 10` com `in2 = 20` a meio caminho ⇒ `15`.
/// Sem a cerca, o preguiçoso devolvia `10` (`lerp(0, 20, ½)`).
#[test]
fn a_driven_blend_cannot_make_the_lazy_mode_answer_differently_from_the_eager_one() {
    let truth = cooked(false, Blend::Driven, vec![1.5]);
    assert_eq!(
        truth,
        vec![15.0],
        "controle: o modo ansioso com `blend` conduzido tem de MISTURAR in1=10 com in2=20"
    );
    assert_eq!(
        cooked(true, Blend::Driven, vec![1.5]),
        truth,
        "o modo preguicoso divergiu do ansioso: o plano escolheu a lei do ROTEAMENTO \
         enquanto o `eval` misturava, e o ramo saltado entrou como zero"
    );
}

/// O **mecanismo** do gate acima: o nó fica **fora do plano**, e por isso o cook puxa as quatro.
///
/// ⚠️ As duas metades são precisas. Sem a de cima, uma cura que fizesse os dois modos concordarem
/// pelo motivo errado (p.ex. desligar a preguiça inteira) passaria; sem esta, a de cima não diz
/// **onde** a cerca vive.
#[test]
fn a_driven_law_param_keeps_the_router_out_of_the_plan_entirely() {
    let reg = registry();
    for (name, blend) in [(crate::BLEND, Blend::Driven), (crate::LAZY, Blend::Off)] {
        let mut g = Graph::new();
        let sw = g.add_node(crate::MANIFEST.name);
        g.set_param(sw, crate::LAZY, 1.0);
        let d = g.add_node(PURE);
        g.drive_param(sw, name, (d, 0))
            .expect("o fio conduz — senao este gate era VAZIO");
        assert!(
            super::plan(&g, &reg).is_empty(),
            "com `{name}` conduzido o plano nao pode afirmar nada (blend={blend:?})"
        );
    }
    // E o controle: sem fio nenhum, o MESMO grafo entra no plano.
    let mut g = Graph::new();
    let sw = g.add_node(crate::MANIFEST.name);
    g.set_param(sw, crate::LAZY, 1.0);
    assert!(
        super::plan(&g, &reg).contains_key(&sw),
        "controle: sem param conduzido o roteador TEM de entrar, senao este gate mede o nada"
    );
}

/// **AS PORTAS CANDIDATAS SÃO EXACTAMENTE AS ENTRADAS QUE O NÓ LÊ.**
///
/// ⚠️ `CHOICE_PORTS.len()` e `N_INPUTS` eram **dois números independentes** que valiam `4` por
/// coincidência de dois literais (auditoria de 2026-08-27). O gate que existia
/// (`a_lazy_router_declares_no_more_choices_than_the_mask_holds`) compara com o `MAX_LAZY_CHOICES`
/// — o **tecto da máscara**, que é outra pergunta — e um `N_INPUTS = 6` com `in4`/`in5` no
/// manifesto deixava-o verde.
///
/// A direcção perigosa é a lista ficar MAIOR: `needed_*` deriva o `last` de `out.len()` e o
/// `switch` deriva o dele de `ins.len()`, então o plano marcaria um índice que o `eval` grampeia
/// para baixo — e o cook saltaria o ramo que o nó de facto lê. *É a mesma classe do `blend`
/// conduzido: a lei e o executor a contar de fontes diferentes.*
#[test]
fn the_choice_ports_are_exactly_the_inputs_the_node_reads() {
    assert_eq!(
        crate::CHOICE_PORTS.len(),
        crate::N_INPUTS,
        "as portas candidatas e as entradas que o `eval` percorre tem de ser a MESMA contagem"
    );
    // E as portas nomeiam-se no manifesto na ordem que o plano indexa.
    for (k, port) in crate::CHOICE_PORTS.iter().enumerate() {
        assert_eq!(
            crate::MANIFEST.inputs[*port as usize].name,
            format!("in{k}"),
            "a porta {port} nao e' o ramo {k}"
        );
    }
}

/// **AS DUAS LEIS TÊM DE CONCORDAR COM O QUE O NÓ LÊ, E A PROVA É O VALOR.**
///
/// ⚠️ O gate irmão `the_needed_law_matches_what_the_node_actually_reads` compara `needed_*` com
/// uma expectativa **escrita à mão** e nunca chama [`crate::switch`] — então uma divergência de
/// arredondamento entre a lei e o nó (`round` contra `floor`, ou o desempate de `0,5`) ficava
/// verde dos dois lados. Aqui o oráculo é o **modo ansioso**, que é o nó a responder por si.
///
/// Os selects são os frágeis: `0,5` (o empate), `1,6` (onde `round` e `floor` discordam), `2,4`
/// (onde concordam — o controle), e as bordas.
#[test]
fn the_lazy_mode_never_changes_the_answer_on_the_selects_where_the_two_laws_are_fragile() {
    for s in [0.5_f32, 1.5, 1.6, 2.4, 2.5, -0.5, 0.0, 3.0, 99.0] {
        for blend in [Blend::Off, Blend::Authored] {
            let eager = cooked(false, blend, vec![s]);
            let lazy = cooked(true, blend, vec![s]);
            assert_eq!(
                lazy, eager,
                "select {s} com blend {blend:?}: o modo preguicoso mudou a RESPOSTA"
            );
        }
    }
}
