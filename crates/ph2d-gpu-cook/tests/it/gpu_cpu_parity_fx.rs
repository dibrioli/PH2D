//! **Paridade dispositivo × CPU do grupo APARÊNCIA** (ciclo 7, doc 112) — os nós `Fx` que
//! MULTIPLICAM as linhas (`StreamOp::SourceRows`).
//!
//! O `fx.rgb_split` devolve `n × 3` elementos (o fantasma R, o fantasma G+B e o próprio elemento
//! por cima) e o `fx.drop_shadow` `n × 2` (`n × 17` com a maciez) — ou `n` quando estão apagados
//! ou acima do tecto. O cozimento canónico
//! (`evaluate_motion_into`) e o sequenciador do dispositivo têm de concordar na CONTAGEM, na
//! ORDEM (blocos, não intercalado: é a ordem que põe as franjas atrás), na posição de cada cópia
//! e na COR de cada cópia — e é a cor que o separa dos deformadores, cujo gate só olha a posição.
//!
//! `#[ignore]`: precisa de um adaptador real.
//!   cargo test -p ph2d-gpu-cook --test it gpu_cpu_parity_fx -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::RenderInstance;
use ph2d_render::SinkStyle;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_move::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_tint::register(&mut reg).unwrap();
    ph2d_node_fx_rgb_split::register(&mut reg).unwrap();
    ph2d_node_fx_drop_shadow::register(&mut reg).unwrap();
    reg
}

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;

/// Posição. Medida (impressa abaixo), não emprestada — o pior caso é a ABERRAÇÃO, onde o eixo sai
/// de uma soma de `n` posições (a redução `Sum` do pivô) e a diferença entre a árvore do
/// dispositivo e o `fold` da CPU entra multiplicada pela força. A família dos deformadores mede o
/// mesmo `Sum` em `2e-4` (`gpu_cpu_parity_deform::EPS_POS`), e esta corrente é mais curta.
const EPS_POS: f32 = 2e-4;

/// Cor. A mesma barra do resto da casa para a tinta (`gpu_cpu_parity::assert_instance_parity`):
/// o alfa das franjas é `a · opacity · falloff`, três produtos sem soma — nada que um dispositivo
/// contraia num FMA —, e o `falloff` e a `tint` de montante já chegam com o ε deles.
const EPS_TINT: f32 = 1e-5;

/// Lado da grelha: `128² × 3 = 49 152` linhas, acima do limiar paralelo da CPU e da primeira costura
/// de bloco da redução.
const SIDE: f32 = 128.0;

fn cook_cpu(reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let mut cook = Cook::new();
    let mut cpu = Vec::new();
    ph2d_eval_motion::evaluate_motion_into(
        &mut cook,
        g,
        reg,
        out,
        PLAYHEAD,
        DEFAULT_UV,
        DEFAULT_SIZE,
        &mut cpu,
    )
    .expect("cpu cook");
    cpu
}

/// Coze no dispositivo. ⛔ Reprova se o plano não reclamou a cadeia INTEIRA: um nó que caísse na
/// CPU em silêncio faria este gate comparar a CPU consigo mesma.
fn cook_gpu(gpu: &GpuContext, reg: &NodeRegistry, g: &Graph, out: NodeId) -> Vec<RenderInstance> {
    let plan = ph2d_gpu_cook::plan(g, reg, reg, out);
    assert!(
        plan.is_fully_gpu(),
        "a cadeia tem de ficar INTEIRA no dispositivo — uma costura faria este gate comparar a \
         CPU consigo mesma"
    );
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.cook(
        gpu,
        g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        SinkStyle::PLAIN,
    )
    .expect("gpu cook");
    ph2d_gpu_cook::read_instances(gpu, gc.instances().expect("cooked"))
}

/// Compara posição, tamanho e cor, instância a instância; devolve o pior `(Δpos, Δtint)`.
fn compare(label: &str, cpu: &[RenderInstance], dev: &[RenderInstance]) -> (f32, f32) {
    assert_eq!(cpu.len(), dev.len(), "{label}: contagem de instâncias");
    let (mut worst_p, mut worst_t) = (0.0f32, 0.0f32);
    for (i, (c, d)) in cpu.iter().zip(dev).enumerate() {
        for k in 0..2 {
            let dp = (c.world_pos[k] - d.world_pos[k]).abs();
            worst_p = worst_p.max(dp);
            assert!(
                dp <= EPS_POS,
                "{label}: instância {i} world_pos[{k}]: cpu {} vs disp {} (|Δ| {dp:e})",
                c.world_pos[k],
                d.world_pos[k]
            );
            // O tamanho é uma coluna REUNIDA (a cópia herda o do elemento-fonte) — um erro de
            // reunião troca-o de elemento; a grelha dá-o igual a todos, então ele só mede que a
            // coluna chegou.
            assert!(
                (c.size[k] - d.size[k]).abs() <= 1e-6,
                "{label}: instância {i} size[{k}]: cpu {} vs disp {}",
                c.size[k],
                d.size[k]
            );
        }
        // O modo POR LINHA (`blend`) desce para o `flip_uv` — a sombra com modo escreve-o.
        assert_eq!(c.flip_uv, d.flip_uv, "{label}: instância {i} flip_uv");
        for k in 0..4 {
            let dt = (c.tint[k] - d.tint[k]).abs();
            worst_t = worst_t.max(dt);
            assert!(
                dt <= EPS_TINT,
                "{label}: instância {i} tint[{k}]: cpu {} vs disp {} (|Δ| {dt:e})",
                c.tint[k],
                d.tint[k]
            );
        }
    }
    eprintln!(
        "{label}: {} instâncias, pior |Δpos| = {worst_p:e}, pior |Δtint| = {worst_t:e}",
        cpu.len()
    );
    (worst_p, worst_t)
}

/// Os params do `fx.rgb_split` num caso.
#[derive(Clone, Copy, Debug)]
struct Split {
    mode: f32,
    x: f32,
    y: f32,
    strength: f32,
    opacity: f32,
    center: [f32; 2],
    start: f32,
}

impl Split {
    const OFF: Self = Self {
        mode: 0.0,
        x: 0.13,
        y: -0.07,
        strength: 0.04,
        opacity: 0.8,
        center: [0.0, 0.0],
        start: 0.0,
    };
}

/// `grid → move → [falloff → tint] → fx → output`, com o nó de efeito posto por `fx`.
///
/// ⚠️ **A grelha é DESLOCADA** para o centroide não ser a origem (a aberração mede-o), e a `tint`
/// de montante tem os QUATRO canais diferentes e nenhum redondo — com a branca de omissão um
/// kernel que trocasse `g` por `b` no fantasma G+B passaria.
fn chain(
    reg: &NodeRegistry,
    side: f32,
    coloured: bool,
    fx: impl FnOnce(&mut Graph) -> NodeId,
) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", side);
    g.set_param(grid, "cols", side);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let mv = g.add_node("motion.move");
    g.set_param(mv, "dx", 4.0);
    g.set_param(mv, "dy", -2.5);
    let fx = fx(&mut g);
    let out = g.add_node("motion.output");
    let mut path = vec![grid, mv];
    if coloured {
        // Um campo por cima, para o `falloff` variar AO LONGO da grelha: as franjas lêem-no no
        // alfa, e um campo constante deixaria um kernel que o ignorasse verde.
        let foc = g.add_node("motion.falloff");
        g.set_param(foc, "radius", 27.3);
        g.set_param(foc, "center_x", 2.9);
        g.set_param(foc, "center_y", -1.7);
        let tint = g.add_node("motion.tint");
        g.set_param(tint, "mode", 0.0);
        g.set_param(tint, "r", 0.31);
        g.set_param(tint, "g", 0.72);
        g.set_param(tint, "b", 0.16);
        g.set_param(tint, "a", 0.85);
        path.extend([foc, tint]);
    }
    path.extend([fx, out]);
    for w in path.windows(2) {
        g.connect(Edge {
            from: (w[0], 0),
            to: (w[1], 0),
            delayed: false,
        })
        .unwrap();
    }
    g.validate(reg).expect("well-typed");
    (g, out)
}

impl Split {
    fn node(self) -> impl FnOnce(&mut Graph) -> NodeId {
        move |g| {
            let fx = g.add_node("fx.rgb_split");
            g.set_param(fx, "mode", self.mode);
            g.set_param(fx, "x", self.x);
            g.set_param(fx, "y", self.y);
            g.set_param(fx, "strength", self.strength);
            g.set_param(fx, "opacity", self.opacity);
            g.set_param(fx, "center_x", self.center[0]);
            g.set_param(fx, "center_y", self.center[1]);
            g.set_param(fx, "start", self.start);
            fx
        }
    }
}

/// `fx.rgb_split`, no dispositivo, concorda com a CPU nos dois modos, no eixo deslocado, no raio
/// limpo e com o campo a modular o alfa.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_rgb_split_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let aberration = Split {
        mode: 1.0,
        ..Split::OFF
    };
    let cases = [
        ("split branco", Split::OFF, false),
        ("split colorido", Split::OFF, true),
        ("aberração", aberration, true),
        (
            "aberração, eixo deslocado",
            Split {
                center: [1.3, -0.6],
                strength: -0.03,
                ..aberration
            },
            true,
        ),
        (
            "aberração, raio limpo",
            Split {
                start: 6.5,
                strength: 0.05,
                ..aberration
            },
            true,
        ),
    ];
    let n = (SIDE * SIDE) as usize;
    for (label, split, coloured) in cases {
        let (g, out) = chain(&reg, SIDE, coloured, split.node());
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        assert_eq!(cpu.len(), n * 3, "{label}: a CPU triplica");
        compare(label, &cpu, &dev);
    }
}

/// ⚠️ **Os dois casos em que a CPU devolve a entrada tal e qual** — a opacidade apagada e o tecto.
///
/// Um nó `SourceRows` parte de uma base VAZIA, logo o dispositivo não pode usar um passa-tudo
/// aqui: a lei de contagem devolve `n` e o corpo copia. Uma lei que discordasse não rebentava —
/// desenhava outro número de coisas, que é o que a contagem apanha.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_rgb_split_forwards_the_input_where_the_cpu_does() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let dead = Split {
        opacity: 0.0,
        ..Split::OFF
    };
    let (g, out) = chain(&reg, SIDE, true, dead.node());
    let cpu = cook_cpu(&reg, &g, out);
    let dev = cook_gpu(&gpu, &reg, &g, out);
    assert_eq!(
        cpu.len(),
        (SIDE * SIDE) as usize,
        "apagado: a CPU devolve n"
    );
    compare("opacidade apagada", &cpu, &dev);

    // ⛔⛔⛔ **A METADE «ACIMA DO TECTO» MORREU COM O TECTO DE INSTÂNCIAS DO DONO, e a morte é o
    // achado.** Ela pedia `⌈√(MAX/3)⌉ + 1 = 1025` de lado ao `motion.grid`; desde 2026-09-21 a
    // grelha clampa cada LADO em `LADO_MAX_DE_GRELHA`, logo o máximo que um GERADOR deste
    // catálogo entrega é [`CELULAS_DE_UMA_GRELHA_CHEIA`] e este nó vê `× 3` disso.
    //
    // ⚠️⚠️ **Ela não reprovava por defeito de produto**: ela pedia uma população que o catálogo já
    // não sabe construir, e lia `98 283` onde esperava `1 050 625`. *Um gate cujo caso deixou de
    // ser construível não afirma nada — ele acusa o produto de não fazer o impossível.*
    //
    // ⚠️ **E ela esteve vermelha e INVISÍVEL desde 21/09**, porque este ficheiro é todo `#[ignore]`
    // e nem o CI nem a varredura impactada correm gates de dispositivo.
    //
    // ⇒ o que fica afirmado é a verdade NOVA, que é a que decide o ciclo 12: **nenhum gerador
    // alcança o tecto deste nó**. No dia em que o tecto de instâncias subir o bastante — ou em que
    // alguém encadeie um dos cinco multiplicadores não-clampados — este gate reprova, e a metade
    // de cima volta com ele.
    let maior_de_um_gerador = ph2d_nodegraph::node::CELULAS_DE_UMA_GRELHA_CHEIA;
    assert!(
        maior_de_um_gerador * 3 < ph2d_node_fx_rgb_split::MAX_INSTANCES,
        "o tecto deste fx voltou a ser alcançável por um gerador só ({maior_de_um_gerador} × 3 \
         >= {}) -- reponha a metade «acima do tecto», que mede o que o no' faz quando a entrada \
         o passa",
        ph2d_node_fx_rgb_split::MAX_INSTANCES
    );
    // E a paridade continua a ser medida no MAIOR caso CONSTRUÍVEL — que é o que o produto pode
    // de facto pôr neste nó, e que antes desta cura já era o que corria (a grelha clampava o
    // `1025` em silêncio).
    let lado = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    let (g, out) = chain(&reg, lado, false, Split::OFF.node());
    let cpu = cook_cpu(&reg, &g, out);
    let dev = cook_gpu(&gpu, &reg, &g, out);
    // ⭐ E o que ela afirma é o RAMO QUE SOBROU: abaixo do tecto o nó **multiplica** (`× 3`). O
    // ramo que ENCAMINHA a entrada sem multiplicar — o que dá o nome a este gate — só arma acima
    // do `MAX_INSTANCES`, e é ele que a asserção acima declara inalcançável.
    assert_eq!(
        cpu.len(),
        maior_de_um_gerador * 3,
        "abaixo do tecto o no' MULTIPLICA; ler a contagem da entrada aqui seria o ramo de \
         encaminhamento a armar onde ele nao devia"
    );
    compare("o maior caso construivel", &cpu, &dev);
}

/// Os params do `fx.drop_shadow` num caso.
#[derive(Clone, Copy, Debug)]
struct Shadow {
    direction: f32,
    distance: f32,
    rgba: [f32; 4],
    softness: f32,
    blend: f32,
}

impl Shadow {
    /// Os defaults do nó (o Photoshop: 35 % de preto, para baixo e à direita).
    const DEFAULT: Self = Self {
        direction: 315.0,
        distance: 0.2,
        rgba: [0.0, 0.0, 0.0, 0.35],
        softness: 0.0,
        blend: 0.0,
    };

    fn node(self) -> impl FnOnce(&mut Graph) -> NodeId {
        move |g| {
            let fx = g.add_node("fx.drop_shadow");
            g.set_param(fx, "direction", self.direction);
            g.set_param(fx, "distance", self.distance);
            for (k, v) in ["r", "g", "b", "a"].into_iter().zip(self.rgba) {
                g.set_param(fx, k, v);
            }
            g.set_param(fx, "softness", self.softness);
            g.set_param(fx, "shadow_blend", self.blend);
            fx
        }
    }
}

/// `fx.drop_shadow`, no dispositivo, concorda com a CPU: a sombra dura, a colorida num ângulo
/// que não é um dos quatro do seno parabólico, a MACIA (o disco de Vogel e o alfa por tap), e o
/// MODO por linha — que desce para o `flip_uv` e é por isso que o `compare` o olha.
///
/// ⚠️ **A coluna `blend` de MONTANTE não está aqui**, e não por esquecimento: os nós que a
/// escrevem (`motion.trail`, `motion.strobe`) ainda não vivem no dispositivo (doc 112 §5, W1c), logo
/// uma cadeia com ela nunca é reclamada inteira. Os elementos leem a identidade (`0`), que é o
/// `_ => 0.0` da CPU.
#[test]
#[ignore = "requires a GPU adapter"]
fn the_drop_shadow_matches_the_cpu_within_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let coloured = Shadow {
        direction: 37.0,
        distance: 1.3,
        rgba: [0.62, 0.18, 0.44, 0.7],
        ..Shadow::DEFAULT
    };
    let soft = Shadow {
        softness: 0.3,
        ..coloured
    };
    let n = (SIDE * SIDE) as usize;
    for (label, shadow, upstream, copies) in [
        ("sombra de omissão", Shadow::DEFAULT, false, 2),
        ("sombra colorida", coloured, true, 2),
        ("sombra macia", soft, true, 17),
        (
            "sombra com modo (Multiply)",
            Shadow {
                blend: 4.0,
                ..coloured
            },
            true,
            2,
        ),
        (
            "sombra macia com modo (Screen, a meio passo)",
            Shadow { blend: 4.5, ..soft },
            false,
            17,
        ),
    ] {
        // A maciez corta no tecto a `n ≤ 15 420`: a grelha de 128² (16 384) passava-o. Um lado
        // de 96 (9 216) fica dentro dele.
        let side = if copies > 2 { 96.0 } else { SIDE };
        let n = if copies > 2 {
            (side * side) as usize
        } else {
            n
        };
        let (g, out) = chain(&reg, side, upstream, shadow.node());
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        assert_eq!(
            cpu.len(),
            n * copies,
            "{label}: a CPU multiplica por {copies}"
        );
        compare(label, &cpu, &dev);
    }
}

/// ⚠️ **Os três casos em que a sombra devolve a entrada tal e qual** — o alfa apagado, o tecto
/// da sombra dura, e o tecto da MACIA, que corta muito mais cedo (`n × 17`).
#[test]
#[ignore = "requires a GPU adapter"]
fn the_drop_shadow_forwards_the_input_where_the_cpu_does() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let max = ph2d_node_fx_drop_shadow::MAX_INSTANCES;
    let dead = Shadow {
        rgba: [0.2, 0.2, 0.2, 0.0],
        ..Shadow::DEFAULT
    };
    let soft = Shadow {
        softness: 0.3,
        ..Shadow::DEFAULT
    };
    // ⛔⛔⛔ **AS DUAS METADES «ACIMA DO TECTO» MORRERAM COM O TECTO DE INSTÂNCIAS DO DONO** — ver
    // a irmã em [`the_rgb_split_forwards_the_input_where_the_cpu_does`], que conta o mecanismo.
    // Aqui o multiplicador é `17` (a sombra macia), logo o maior caso construível vê
    // `CELULAS_DE_UMA_GRELHA_CHEIA × 17` — e mesmo esse fica abaixo do `MAX_INSTANCES` deste nó.
    let maior_de_um_gerador = ph2d_nodegraph::node::CELULAS_DE_UMA_GRELHA_CHEIA;
    assert!(
        maior_de_um_gerador * 17 < max,
        "o tecto deste fx voltou a ser alcançável por um gerador só ({maior_de_um_gerador} × 17 \
         >= {max}) -- reponha as duas metades «acima do tecto»"
    );
    // O que fica é a paridade nos TRÊS casos que o produto sabe construir: o alfa apagado (a
    // metade que nunca dependeu do tecto) e os dois multiplicadores, no MAIOR lado que uma grelha
    // dá. ⚠️ Antes desta cura os dois últimos já corriam a este lado — a grelha clampava-os em
    // silêncio, e só a asserção da contagem via a diferença.
    let lado = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    // ⭐ `copias` é o que o nó MULTIPLICA abaixo do tecto: `1` com o alfa apagado (não há sombra
    // a somar), `2` na dura (a peça e a sombra) e `17` na macia (o leque do desfoque). O ramo que
    // ENCAMINHA a entrada sem multiplicar — o que dá o nome a este gate — só arma acima do
    // `MAX_INSTANCES`, e é ele que a asserção acima declara inalcançável.
    for (label, side, shadow, copias) in [
        ("alfa apagado", SIDE, dead, 1),
        ("o maior construivel (dura)", lado, Shadow::DEFAULT, 2),
        ("o maior construivel (macia)", lado, soft, 17),
    ] {
        let (g, out) = chain(&reg, side, true, shadow.node());
        let cpu = cook_cpu(&reg, &g, out);
        let dev = cook_gpu(&gpu, &reg, &g, out);
        let n = (side * side) as usize;
        assert_eq!(
            cpu.len(),
            n * copias,
            "{label}: abaixo do tecto o no' multiplica por {copias}"
        );
        compare(label, &cpu, &dev);
    }
}

/// **SONDA, não gate — o `MAX_INSTANCES` dos dois nós, medido no DISPOSITIVO** (doc 112 §3,
/// `CLAUDE.md` §0.0: *nunca deixe o fallback definir o produto*).
///
/// O tecto `262 144` foi medido no caminho de CPU (`~10–15 ns` por linha). Com os dois nós no
/// dispositivo o recurso muda, e esta sonda imprime o que ele custa: `grid side² → oscillator →
/// fx → output` (a fonte MEXE-SE, senão o memo da CPU devolve o quadro anterior sem cozer), a
/// mediana por quadro nos dois caminhos, e o que a descida pede em memória (`184 B × linhas`)
/// contra o limite do adaptador.
///
/// ⚠️ A sonda não passa do tecto que mede: para amostrar acima dele, suba o `MAX_INSTANCES` dos
/// dois nós localmente (as linhas marcadas `CORTADO` dizem quando isso não foi feito).
///   cargo test -p ph2d-gpu-cook --release --test it fx_row_ceiling_probe -- --ignored --nocapture
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn fx_row_ceiling_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let mut reg = registry();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    let limits = gpu.device.limits();
    let inst = std::mem::size_of::<RenderInstance>() as u64;
    eprintln!(
        "adaptador: binding {} MiB · buffer {} MiB · RenderInstance {inst} B",
        limits.max_storage_buffer_binding_size >> 20,
        limits.max_buffer_size >> 20
    );
    let median = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    eprintln!(
        "  nó             │   n (fonte) │     linhas │ disp ms │  CPU ms │ memória da descida"
    );
    // ⛔⛔⛔ **OS LADOS ERAM `[256, 512, 1024, 1448, 2048]` E OS CINCO CLAMPAM NO MESMO NÚMERO**
    // desde 2026-09-21: a grelha limita cada LADO em `LADO_MAX_DE_GRELHA`. A sonda imprimia
    // CINCO linhas com o `n (fonte)` pedido e a coluna `linhas` igual em todas — *uma varredura
    // cujos pontos colapsaram, com a forma de uma varredura.*
    //
    // ⇒ os pontos passam a sair do tecto e a dizer o que o produto pode de facto construir, e a
    // guarda logo abaixo recusa imprimir uma tabela colapsada.
    let lado_max = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    for node in ["fx.rgb_split", "fx.drop_shadow"] {
        let mut realizadas = Vec::new();
        for side in [
            (lado_max / 8.0).ceil(),
            (lado_max / 4.0).ceil(),
            (lado_max / 2.0).ceil(),
            lado_max,
        ] {
            let mut g = Graph::new();
            let seed = g.add_node("motion.grid");
            g.set_param(seed, "rows", side);
            g.set_param(seed, "cols", side);
            let osc = g.add_node("motion.oscillator");
            g.set_param(osc, "amplitude", 3.0);
            g.set_param(osc, "frequency", 0.7);
            let fx = g.add_node(node);
            let out = g.add_node("motion.output");
            for (a, b) in [(seed, osc), (osc, fx), (fx, out)] {
                g.connect(Edge {
                    from: (a, 0),
                    to: (b, 0),
                    delayed: false,
                })
                .unwrap();
            }
            let n = (side * side) as u64;
            let plan = ph2d_gpu_cook::plan(&g, &reg, &reg, out);
            if !plan.is_fully_gpu() {
                eprintln!(
                    "  {node:<14} │ {n:>11} │ NÃO FICA NO DISPOSITIVO — {:?}",
                    plan.boundaries
                );
                continue;
            }
            let mut gc = ph2d_gpu_cook::GpuCook::new();
            let mut dev_ms = Vec::new();
            let mut rows = 0u64;
            let mut refused = None;
            for f in 0..24 {
                let t0 = std::time::Instant::now();
                let r = gc.cook(
                    &gpu,
                    &g,
                    &reg,
                    &reg,
                    &plan,
                    &[],
                    CookClock::at(f64::from(f) / 60.0),
                    DEFAULT_UV,
                    DEFAULT_SIZE,
                    SinkStyle::PLAIN,
                );
                if let Err(e) = r {
                    refused = Some(format!("{e:?}"));
                    break;
                }
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
                if f >= 4 {
                    dev_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
                rows = u64::from(gc.instances().map_or(0, |b| b.len()));
            }
            if let Some(e) = refused {
                eprintln!("  {node:<14} │ {n:>11} │ RECUSADO — {e}");
                continue;
            }
            let mut cook = Cook::new();
            let mut cpu_ms = Vec::new();
            for f in 0..5 {
                let mut buf = Vec::new();
                let t0 = std::time::Instant::now();
                ph2d_eval_motion::evaluate_motion_into(
                    &mut cook,
                    &g,
                    &reg,
                    out,
                    f64::from(f) / 60.0 + 1.0,
                    DEFAULT_UV,
                    DEFAULT_SIZE,
                    &mut buf,
                )
                .expect("cpu cook");
                if f >= 1 {
                    cpu_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
                }
            }
            let note = if rows == n {
                "  ← CORTADO pelo tecto"
            } else {
                ""
            };
            eprintln!(
                "  {node:<14} │ {n:>11} │ {rows:>10} │ {:>7.2} │ {:>7.2} │ {:>6} MiB{note}",
                median(dev_ms),
                median(cpu_ms),
                (rows * inst) >> 20
            );
            realizadas.push(rows);
        }
        // ⚠️ **A guarda que faltava:** dois lados diferentes que cozam a MESMA população deixam
        // esta tabela a descrever um ponto com N rótulos — foi exactamente o que aconteceu entre
        // 21/09 e hoje, em silêncio, porque uma sonda imprime e ninguém a lê.
        for par in realizadas.windows(2) {
            assert!(
                par[1] > par[0],
                "{node}: a varredura colapsou ({} depois de {}) -- um clamp a montante esta a \
                 cortar os pontos",
                par[1],
                par[0]
            );
        }
    }
    eprintln!(
        "  (um quadro de 60 fps são 16,7 ms; o disp. inclui a grelha, o oscilador e a descida)"
    );
}
