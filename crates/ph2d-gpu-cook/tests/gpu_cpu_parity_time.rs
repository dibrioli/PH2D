//! GPU-vs-CPU parity para **o RUÍDO e o RELÓGIO** — os params que o grupo B da
//! conferência (doc 89 folha 15) acrescentou aos dois geradores TEMPORAIS.
//!
//! ## A forma é a do irmão `gpu_cpu_parity_arith`, e o motivo é o mesmo
//!
//! Cada param novo entra com o seu **CONTROLE** — o mesmo nó, a mesma entrada,
//! só o param desligado — e o gate afirma duas coisas por par: *o device
//! concorda com a CPU* **e** *os dois produzem campos DIFERENTES*. Sem a segunda
//! metade, um WGSL que ignorasse o param passaria em todas as linhas: o default
//! de todos eles É o mundo anterior, então um kernel cego a eles concorda com a
//! CPU **por vácuo** exactamente onde o gate parece mais verde.
//!
//! ## ⚠️ A fixture do LAÇO não pode pousar NA descontinuidade
//!
//! É a lição que o grupo A pagou, e aqui ela tem um nome novo: o laço faz
//! `u = t / L` e depois `floor(u)`, e **o WGSL não exige que a divisão seja
//! correctamente arredondada** (o IEEE-754 exige; a spec do WGSL admite folga).
//! Se `t / L` cair a um ulp de um inteiro, os dois lados podem escolher `floor`
//! diferente, `tau` salta um `L` inteiro e o campo muda por completo — um erro
//! macroscópico atravessando um ε que existe para absorver um ulp.
//!
//! A fixture escolhe **`L = 1.0`**, onde `u = t` **exactamente** (dividir por
//! 1,0 não arredonda em lado nenhum) e `t = 0,37` está longe de qualquer
//! fronteira. ⛔ **Não "melhore" isto pondo `L` num múltiplo do playhead** — é
//! precisamente o caso que o gate não pode medir.
//!
//! ## O que o par do laço mede, e por que a primeira amostra é a MESMA
//!
//! Com `u = 0,37` o laço lê `t_a = 0,37` — o mesmo instante de sempre — e mistura
//! `t_b = −0,63` com peso `w = u²(3−2u) = 0,309`. Então a diferença contra o
//! controle é `0,309 × (campo(t_b) − campo(t_a))`, e é por isso que a fixture usa
//! um `speed` alto: os dois instantes têm de cair em CÉLULAS diferentes do
//! reticulado, senão o campo mal difere e o gate mediria ruído.
//!
//! `#[ignore]`: precisa de adapter real. Numa máquina de dev / na lane de GPU:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_time --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;
/// O comprimento do laço das fixtures. `1.0` para `u = t / L` ser exacto — ver o
/// cabeçalho.
const LOOP: f32 = 1.0;
/// **O orçamento é RELATIVO, e a mudança de forma é MEDIDA — não preferência.**
///
/// ⚠️ O ε dos gates irmãos é `1e-4` **absoluto**, e a primeira corrida deste
/// nasceu VERMELHA no próprio CONTROLE (`lacunarity 2`, o mundo que já shipava,
/// `max |d| = 1,3e-4`). O diagnóstico veio da coluna que o gate passou a
/// imprimir: sobre um campo de amplitude 9,58 aquilo é **1,36e-5 relativo** —
/// não é um ramo errado, é ACUMULAÇÃO. Uma pilha de cinco oitavas soma cinco
/// interpolações de reticulado, e o WGSL pode contrair `a*b + c` em FMA onde a
/// CPU não contrai; o erro é **proporcional à magnitude**, e um orçamento
/// absoluto é a forma errada para ele.
///
/// | caso | `max \|d\|` | amplitude | relativo |
/// |---|---|---|---|
/// | lacunarity 2 (controle) | 1,30e-4 | 9,58 | 1,36e-5 |
/// | laço, em `t = 0` | 2,19e-4 | 10,22 | 2,15e-5 |
/// | **lacunarity 3** | 5,54e-4 | 9,43 | **5,87e-5** |
///
/// ⚠️ **E o pior caso ser a `lacunarity 3` NÃO é acaso — é a mesma medição vista
/// do outro lado.** A oitava `k` amostra em `x · lacᵏ`, então uma lacunarity
/// maior põe o topo da pilha num ponto MAIOR do reticulado, onde um `f32` tem
/// menos bits fraccionários para a interpolação; a sonda
/// `measure_where_lacunarity_stops_resolving` mede isso como *"menos fracções
/// distintas"* e aqui ele reaparece como erro de paridade. **A degradação é do
/// param, não do kernel** — e é por isso que o teto do slider é 4.
///
/// `2e-4` = o pior medido com ~3,4× de folga para variação de vendedor.
///
/// ⛔ **E isto NÃO revoga a regra do irmão** (*"não conserte alargando o ε"*): lá
/// o erro era **um degrau inteiro** de uma função descontínua, que nenhuma folga
/// pode admitir sem o gate deixar de distinguir os modos. Aqui a separação que o
/// gate procura é de `1e-1` a `1e0` **relativos** — quatro ordens acima deste
/// orçamento. Mecanismos diferentes, formas de barra diferentes.
const EPS_REL: f32 = 2e-4;
/// Quanto dois casos vizinhos têm de diferir para a comparação significar alguma
/// coisa. Muito acima de [`EPS`] de propósito.
const MODE_GAP: f32 = 1e-2;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// O registry MÍNIMO — só os nós que estas fixtures montam.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_noise::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    reg
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

fn connect_to(g: &mut Graph, a: NodeId, b: NodeId, port: u16) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, port),
        delayed: false,
    })
    .unwrap();
}

fn grid(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    grid
}

/// Um `value.noise` com a pilha de oitavas ARMADA — sem ela a `lacunarity` é
/// provadamente inerte (o `px *= lac` acontece depois da única amostra) e o par
/// dela seria verde por construção.
fn noise(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = g.add_node("value.noise");
    g.set_param(vn, "frequency", 0.23);
    // Alto de propósito: o par do LAÇO precisa que `t_a` e `t_b` caiam em células
    // diferentes do reticulado (ver o cabeçalho).
    g.set_param(vn, "speed", 2.0);
    g.set_param(vn, "octaves", 5.0);
    g.set_param(vn, "roughness", 0.55);
    g.set_param(vn, "amplitude", 1.9);
    g.set_param(vn, "seed", 4.0);
    connect(g, src, vn);
    vn
}

type Build = fn(&mut Graph, NodeId) -> NodeId;

struct Case {
    label: &'static str,
    build: Build,
    /// `true` ⇒ tem de produzir um campo DIFERENTE do anterior (o seu controle).
    differs_from_previous: bool,
}

fn noise_lac_two(g: &mut Graph, src: NodeId) -> NodeId {
    noise(g, src) // lacunarity 2.0, o default
}

fn noise_lac_three(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = noise(g, src);
    g.set_param(vn, "lacunarity", 3.0);
    vn
}

fn noise_no_loop(g: &mut Graph, src: NodeId) -> NodeId {
    noise(g, src) // loop_period 0.0, o default
}

fn noise_looped(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = noise(g, src);
    g.set_param(vn, "loop_period", LOOP);
    vn
}

fn noise_no_pan(g: &mut Graph, src: NodeId) -> NodeId {
    noise(g, src) // pan 0, o default
}

fn noise_panned(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = noise(g, src);
    g.set_param(vn, "pan_x", 0.6);
    g.set_param(vn, "pan_y", 0.5);
    vn
}

/// O `value.noise` no eixo ESPACIAL, com laço e pan LIGADOS — o ramo `HAS_P` do
/// kernel, que os seis casos acima não percorrem.
fn noise_world_looped_panned(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = noise(g, src);
    g.set_param(vn, "space", 1.0);
    g.set_param(vn, "loop_period", LOOP);
    g.set_param(vn, "pan_x", 0.6);
    g.set_param(vn, "pan_y", 0.5);
    vn
}

fn noise_world_plain(g: &mut Graph, src: NodeId) -> NodeId {
    let vn = noise(g, src);
    g.set_param(vn, "space", 1.0);
    vn
}

/// O LFO na régua de SEGUNDOS — o controle do par do BPM. `phase_stagger` dá-lhe
/// variação entre instâncias: um campo chato concordaria com qualquer kernel, e
/// o gate rejeita-o pela medida de `spread`.
fn lfo_seconds(g: &mut Graph, src: NodeId) -> NodeId {
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.8);
    g.set_param(lfo, "amplitude", 1.5);
    g.set_param(lfo, "phase_stagger", 0.017);
    connect(g, src, lfo);
    lfo
}

fn lfo_bpm(g: &mut Graph, src: NodeId) -> NodeId {
    let lfo = lfo_seconds(g, src);
    g.set_param(lfo, "time_mode", 1.0);
    // 200 BPM = 0,3 s por ciclo — bem longe dos 0,8 s do controle.
    g.set_param(lfo, "bpm", 200.0);
    lfo
}

static CASES: &[Case] = &[
    Case {
        label: "noise lacunarity 2 (controle)",
        build: noise_lac_two,
        differs_from_previous: false,
    },
    Case {
        label: "noise lacunarity 3",
        build: noise_lac_three,
        differs_from_previous: true,
    },
    Case {
        label: "noise sem laco (controle)",
        build: noise_no_loop,
        differs_from_previous: false,
    },
    Case {
        label: "noise com laco",
        build: noise_looped,
        differs_from_previous: true,
    },
    Case {
        label: "noise sem pan (controle)",
        build: noise_no_pan,
        differs_from_previous: false,
    },
    Case {
        label: "noise com pan",
        build: noise_panned,
        differs_from_previous: true,
    },
    Case {
        label: "noise World simples (controle)",
        build: noise_world_plain,
        differs_from_previous: false,
    },
    Case {
        label: "noise World + laco + pan",
        build: noise_world_looped_panned,
        differs_from_previous: true,
    },
    Case {
        label: "lfo Seconds (controle)",
        build: lfo_seconds,
        differs_from_previous: false,
    },
    Case {
        label: "lfo BPM",
        build: lfo_bpm,
        differs_from_previous: true,
    },
];

/// A amplitude que a fixture de facto dirige — o denominador do orçamento, e a
/// prova de que ela CONTÉM o fenômeno (um campo chato concorda com tudo).
fn field_swing(v: &[f32]) -> f32 {
    v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x))
        - v.iter().fold(f32::INFINITY, |m, x| m.min(*x))
}

fn worst_delta(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "contagem de elementos");
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// Coza o grafo nos dois lados e devolve o `Y` que o valor dirigiu. `None` se não
/// houver adapter.
fn drive_and_compare(gpu: &GpuContext, reg: &NodeRegistry, case: &Case, at: f64) -> Vec<f32> {
    let mut g = Graph::new();
    let src = grid(&mut g);
    let value = (case.build)(&mut g, src);
    // O valor dirige Y para a comparação atravessar o MESMO lowering que o
    // produto usa, e não só a coluna solta.
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, src, drive);
    connect_to(&mut g, value, drive, 1);

    g.validate(reg)
        .unwrap_or_else(|e| panic!("{}: {e:?}", case.label));
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, drive);
    assert!(
        plan.is_fully_gpu(),
        "{}: a cadeia tem de ser reivindicada de ponta a ponta",
        case.label
    );

    let mut cook = Cook::new();
    let cpu = cook
        .cook(&g, reg, drive, at)
        .unwrap_or_else(|e| panic!("{}: cpu cook {e:?}", case.label));
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(at),
        DEFAULT_UV,
        DEFAULT_SIZE,
        0,
    )
    .unwrap_or_else(|e| panic!("{}: gpu cook {e:?}", case.label));

    let cpu_stream = cpu[0].as_stream();
    let cpu_y: Vec<f32> = match cpu_stream.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("{}: sem coluna P", case.label),
    };
    let gpu_p = gc
        .read_column_vec2(gpu, drive, "P")
        .unwrap_or_else(|| panic!("{}: P não volta do device", case.label));
    let gpu_y: Vec<f32> = gpu_p.iter().map(|p| p[1]).collect();

    let worst = worst_delta(&cpu_y, &gpu_y);
    let swing = field_swing(&cpu_y);
    let budget = EPS_REL * swing;
    eprintln!(
        "{:<34} t={at:.2}  max |d| = {worst:e}  amplitude = {swing:.3}  rel = {:e}",
        case.label,
        worst / swing
    );
    if worst >= budget {
        for (i, (a, b)) in cpu_y.iter().zip(&gpu_y).enumerate() {
            if (a - b).abs() >= budget {
                eprintln!("    i={i} cpu={a:.9} gpu={b:.9} (de {})", cpu_y.len());
                break;
            }
        }
    }
    assert!(
        worst < budget,
        "{}: max |d| = {worst:e} sobre amplitude {swing:.3} ({:e} relativo, orçamento {EPS_REL:e})",
        case.label,
        worst / swing
    );
    cpu_y
}

/// **Os dez casos do ruído e do relógio, no device, cada um com o seu controle.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_noise_and_clock_params_match_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut previous: Option<Vec<f32>> = None;

    for case in CASES {
        let field = drive_and_compare(&gpu, &reg, case, PLAYHEAD);

        // A fixture CONTÉM o fenômeno: um campo constante concordaria com
        // qualquer kernel.
        let spread = field_swing(&field);
        assert!(
            spread > MODE_GAP,
            "{}: o campo é chato ({spread:e}) — a fixture não exercita o param",
            case.label
        );

        if case.differs_from_previous {
            let prev = previous.as_ref().expect("um controle precede cada par");
            let gap = worst_delta(&field, prev);
            eprintln!("{:<34}   contra o controle: {gap:e}", "");
            assert!(
                gap > MODE_GAP,
                "{}: NÃO se distingue do controle ({gap:e}) — ou o param é \
                 ignorado no device, ou a fixture não contém a diferença",
                case.label
            );
        }
        previous = Some(field);
    }
}

/// **A COSTURA FECHA NO DEVICE** — a propriedade, não a paridade.
///
/// ⚠️ Este gate é o irmão que a paridade **não** substitui: o sweep prova que o
/// WGSL concorda com a CPU num instante, e um laço que não fechasse nos dois
/// lados concordaria perfeitamente e estaria errado nos dois. O que se afirma
/// aqui é a lei — `campo(0)` e `campo(L)` são o MESMO campo, lido do device.
///
/// ⚠️ E o CONTROLE está dentro: sem laço as duas leituras têm de DIFERIR (senão
/// o gate ficaria verde sobre um campo congelado, que fecha qualquer costura).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_loop_seam_closes_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let looped = Case {
        label: "laco: fecho no device",
        build: noise_looped,
        differs_from_previous: false,
    };
    let open = Case {
        label: "sem laco: o controle",
        build: noise_no_loop,
        differs_from_previous: false,
    };

    let a = drive_and_compare(&gpu, &reg, &looped, 0.0);
    let b = drive_and_compare(&gpu, &reg, &looped, f64::from(LOOP));
    let seam = worst_delta(&a, &b);
    let budget = EPS_REL * field_swing(&a);
    eprintln!("costura t=0 contra t=L: {seam:e} (orçamento {budget:e})");
    assert!(seam < budget, "o laço NÃO fechou no device: {seam:e}");

    // O controle: o mesmo par de instantes SEM laço tem de dar campos diferentes.
    let c = drive_and_compare(&gpu, &reg, &open, 0.0);
    let d = drive_and_compare(&gpu, &reg, &open, f64::from(LOOP));
    let drift = worst_delta(&c, &d);
    eprintln!("sem laco, os mesmos dois instantes: {drift:e}");
    assert!(
        drift > MODE_GAP,
        "o campo mal se move em {LOOP}s ({drift:e}) — o fecho acima é vácuo"
    );
}
