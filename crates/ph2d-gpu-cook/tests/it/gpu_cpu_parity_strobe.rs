//! ⭐⭐ **PARIDADE CPU↔GPU DO `motion.strobe`** (ciclo 7, W1c — doc 112) — o consumidor que o
//! ciclo 6 deixou a apontar para aqui (os nove `pulse.*` estavam no dispositivo sem ninguém que
//! os lesse).
//!
//! O flash é um ESTADO que atravessa o `pre` (`glow` · `glow_age` · `glow_seq`), e um kernel de
//! estado falha de um modo que a rota não vê: um que nunca acenda, ou que nunca apague, devolve
//! colunas bem formadas. Por isso este gate, como o dos pulsos,
//!
//! 1. corre **muitas voltas** com o estado a atravessar o `pre` (as três fases do envelope — subir,
//!    platô, cair — só aparecem com tempo),
//! 2. compara **tique a tique** as TRÊS colunas do estado e as DUAS do look (`size`, `tint`), e
//! 3. exige, do lado da CPU sozinho, que o envelope tenha passado pelas três fases — e, no caso da
//!    probabilidade, que houve pulsos RECUSADOS.
//!
//! `#[ignore]`: precisa de adaptador.
//! ```text
//! cargo test -p ph2d-gpu-cook --test it -- --ignored --nocapture gpu_cpu_parity_strobe
//! ```

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const DT: f64 = 1.0 / 60.0;
/// Voltas: um batimento a cada ~19 tiques, e a queda por omissão dura 34 — duas batidas inteiras
/// e o princípio da terceira.
const VOLTAS: usize = 48;
/// **O estado é EXACTO** (medido: pior `0` nos dois casos; o look `1,2e-7` e `2,4e-7`). A queda é o mesmo `f32` nas duas rotas (o uniform
/// derivado), a subida é `idade / attack` e o platô é `1` — nada que um dispositivo contraia. Uma
/// barra folgada aqui esconderia o `pow` que o canal derivado existe para evitar.
const EPS_ESTADO: f32 = 0.0;
/// O look: `1 + boost · g` e o `mix` da cor são FMA-contraíveis, e a curva é uma LUT (a tabela da
/// identidade devolve o brilho a poucos ULP; a da fixture tem os nós em cima de amostras, então o
/// erro é o mesmo). A barra de cor da casa (`gpu_cpu_parity::assert_instance_parity`).
const EPS_LOOK: f32 = 1e-5;
/// Uma curva por pedaços LINEARES com o nó em `0,2 = 51/255` — em cima de uma amostra da LUT de
/// 256, para a tabela ser exacta em cada pedaço e a paridade medir o KERNEL, não a resolução.
const CURVA: &str = "c1 0:0:L 0.2:0.6:L 1:1:L";
/// O metrónomo. ⚠️⚠️ **Os dois números foram ESCOLHIDOS longe do fio da navalha, e o gate prova-o**
/// ([`longe_do_fio`]): o índice do ciclo é um `floor` e o `playhead` do dispositivo é `f32`, logo
/// as rotas discordam por um tique inteiro quando `(t − i·defasagem)/período` cai a um ULP de um
/// inteiro (a divergência DECLARADA do `pulse.beat`). A 1.ª redacção usava `0,31`/`0,0023` e a
/// linha 200 no tique 9 dava `(0,15 − 0,46)/0,31 = −1` EXACTO: o gate acusou o strobe de uma
/// discordância que era do metrónomo.
const PERIODO: f32 = 0.3137;
const DEFASAGEM: f32 = 0.001731;
const LADO: f32 = 24.0;

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
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_pulse_beat::register(&mut reg).unwrap();
    ph2d_node_motion_strobe::register(&mut reg).unwrap();
    reg
}

fn liga(g: &mut Graph, de: NodeId, para: (NodeId, u16), pre: bool) {
    g.connect(Edge {
        from: (de, 0),
        to: para,
        delayed: pre,
    })
    .unwrap();
}

/// `grid → [falloff] → strobe ← pulse.beat`, com o `pre` do strobe fechado sobre si. Devolve o
/// strobe (as colunas lêem-se nele) e a saída.
fn cadeia(params: &[(&str, f32)], curva: bool, campo: bool) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", LADO);
    g.set_param(grid, "cols", LADO);
    let bt = g.add_node("pulse.beat");
    // ⚠️ Um período que NÃO é múltiplo do tique (o fio da navalha do doc 110 §8 é outra régua) e
    // uma fase por linha — sem ela as 576 linhas são a mesma coluna repetida.
    g.set_param(bt, "period", PERIODO);
    g.set_param(bt, "phase_stagger", DEFASAGEM);
    liga(&mut g, grid, (bt, 0), false);
    liga(&mut g, bt, (bt, 1), true);
    let mut fonte = grid;
    if campo {
        // O `falloff` varia AO LONGO da grelha: o look lê-o, e um campo constante deixaria verde um
        // kernel que o ignorasse.
        let foc = g.add_node("motion.falloff");
        g.set_param(foc, "radius", 6.3);
        g.set_param(foc, "center_x", 1.9);
        g.set_param(foc, "center_y", -0.7);
        liga(&mut g, grid, (foc, 0), false);
        fonte = foc;
    }
    let sb = g.add_node("motion.strobe");
    for (k, v) in params {
        g.set_param(sb, *k, *v);
    }
    if curva {
        g.set_text_param(sb, "curve", CURVA);
    }
    liga(&mut g, fonte, (sb, 0), false);
    liga(&mut g, bt, (sb, 1), false);
    liga(&mut g, sb, (sb, 2), true);
    let out = g.add_node("motion.output");
    liga(&mut g, sb, (out, 0), false);
    (g, sb, out)
}

fn escalar(s: &ph2d_nodegraph::attr::Stream, c: &str) -> Vec<f32> {
    match s.get(c) {
        Some(Column::Scalar(v)) => v.clone(),
        Some(Column::Vec2(v)) => v.iter().flatten().copied().collect(),
        Some(Column::Vec4(v)) => v.iter().flatten().copied().collect(),
        outra => panic!("a CPU não emitiu `{c}`: {outra:?}"),
    }
}

/// O pior `|Δ|` de duas colunas achatadas, com a linha culpada.
fn pior(a: &[f32], b: &[f32]) -> (usize, f32) {
    assert_eq!(a.len(), b.len(), "comprimentos");
    a.iter()
        .zip(b)
        .enumerate()
        .map(|(k, (x, y))| (k, if x == y { 0.0 } else { (x - y).abs() }))
        .fold((0, 0.0), |m, c| if c.1 > m.1 { c } else { m })
}

/// Corre as duas rotas `VOLTAS` tiques e compara. Devolve o `glow` e o `glow_seq` da CPU por volta.
fn paridade(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    rotulo: &str,
    (g, sb, out): (Graph, NodeId, NodeId),
    blend: bool,
) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    g.validate(reg).expect("bem tipada");
    let plano = plan(&g, reg, reg, out);
    assert!(
        plano.is_fully_gpu(),
        "{rotulo}: a cadeia tem de ser do dispositivo — senão este gate mede a costura: {:?}",
        plano.boundaries
    );
    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    let (mut glows, mut seqs) = (Vec::new(), Vec::new());
    let (mut pior_estado, mut pior_look) = (0.0f32, 0.0f32);
    for volta in 0..VOLTAS {
        let t = volta as f64 * DT;
        let cpu = cook.cook(&g, reg, sb, t).expect("cpu cook");
        let cpu = cpu[0].as_stream();
        gc.cook(
            gpu,
            &g,
            reg,
            reg,
            &plano,
            &[],
            CookClock::at(t),
            DEFAULT_UV,
            DEFAULT_SIZE,
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        let mut colunas = vec![
            ("glow", EPS_ESTADO),
            ("glow_age", EPS_ESTADO),
            ("glow_seq", EPS_ESTADO),
            ("size", EPS_LOOK),
            ("tint", EPS_LOOK),
        ];
        if blend {
            colunas.push(("blend", 0.0));
        }
        for (c, eps) in colunas {
            let a = escalar(cpu, c);
            let b = gc
                .read_column(gpu, sb, c)
                .unwrap_or_else(|| panic!("{rotulo}: `{c}` não voltou do dispositivo"));
            let (onde, d) = pior(&a, &b);
            assert!(
                d <= eps,
                "{rotulo}: volta {volta}, `{c}` diverge {d:e} na posição {onde}: cpu {} disp {}",
                a[onde],
                b[onde]
            );
            if c.starts_with("glow") {
                pior_estado = pior_estado.max(d);
            } else {
                pior_look = pior_look.max(d);
            }
        }
        glows.push(escalar(cpu, "glow"));
        seqs.push(escalar(cpu, "glow_seq"));
        // ⚠️ `advance_tick` é o que PUBLICA o `pre` do lado da CPU (a lição do gate dos pulsos).
        cook.advance_tick(&g, reg, t).expect("cpu tick");
    }
    eprintln!("{rotulo}: pior estado {pior_estado:e}, pior look {pior_look:e}");
    (glows, seqs)
}

/// **A fixture está LONGE do fio da navalha** — em `f64`, a menor distância de
/// `(t − i·defasagem)/período` a um inteiro, fora da origem (que é exacta nas duas rotas), tem de
/// ficar muito acima do erro de um `f32` naquela conta (`~3e-7`). Medido: `1,49e-5`.
fn longe_do_fio() {
    let (p, s) = (f64::from(PERIODO), f64::from(DEFASAGEM));
    let linhas = (LADO * LADO) as usize;
    let mut menor = f64::MAX;
    for i in 0..linhas {
        for k in 0..VOLTAS {
            if i == 0 && k == 0 {
                continue;
            }
            let x = (k as f64 * DT - i as f64 * s) / p;
            menor = menor.min((x - x.round()).abs());
        }
    }
    assert!(
        menor > 1e-5,
        "a fixture pisa o fio da navalha do `pulse.beat` ({menor:e}) — o gate mediria o \
         metrónomo e não o strobe"
    );
}

/// ⚠️ **O CONTROLO DA NÃO-VACUIDADE** — o envelope passou pelo PICO e CAIU (e, com `subida`,
/// também SUBIU sem chegar ao pico). ⚠️ Não se pede o ZERO: uma queda geométrica nunca lá chega
/// (a 1.ª redacção pedia-o e reprovou sobre a fixture certa).
fn fases(rotulo: &str, glows: &[Vec<f32>], subida: bool) {
    let pares = || {
        (1..glows.len()).flat_map(move |v| {
            glows[v]
                .iter()
                .zip(&glows[v - 1])
                .map(|(&agora, &antes)| (antes, agora))
        })
    };
    let pico = glows.iter().flatten().any(|&g| g == 1.0);
    let caiu = pares().any(|(antes, agora)| agora < antes && agora > 0.0);
    let subiu = pares().any(|(antes, agora)| agora > antes && agora < 1.0);
    assert!(
        pico && caiu && (subiu || !subida),
        "{rotulo}: a fixture não exercita o envelope (pico={pico}, caiu={caiu}, subiu={subiu})"
    );
}

/// O flash de omissão — subida instantânea, queda de 34 tiques, todo pulso acende.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_default_strobe_matches_the_cpu_tick_by_tick() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    longe_do_fio();
    let (glows, _) = paridade(
        &gpu,
        &reg,
        "strobe de omissão",
        cadeia(&[], false, false),
        false,
    );
    fases("strobe de omissão", &glows, false);
}

/// O envelope com FORMA (subida de 3, platô de 2, queda de 12), a CURVA, o CAMPO e o MODO — e
/// `probability = 0,6`, para o sorteio correr.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_shaped_strobe_matches_the_cpu_tick_by_tick() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    longe_do_fio();
    let params = [
        ("attack", 3.0),
        ("hold", 2.0),
        ("decay", 12.0),
        ("probability", 0.6),
        ("size_boost", 1.3),
        ("flash_r", 0.2),
        ("flash_g", 0.9),
        ("flash_b", 0.35),
        ("flash_amount", 0.7),
        ("flash_blend", 4.5),
    ];
    let (glows, seqs) = paridade(
        &gpu,
        &reg,
        "strobe com forma",
        cadeia(&params, true, true),
        true,
    );
    fases("strobe com forma", &glows, true);
    // ⚠️ **Houve pulsos RECUSADOS** — uma linha cuja pista andou sem o brilho voltar ao pico nem à
    // subida. Sem isto, um sorteio que aceitasse tudo concordaria com uma CPU que aceitasse tudo.
    let recusados = (1..VOLTAS)
        .flat_map(|v| (0..seqs[v].len()).map(move |i| (v, i)))
        .filter(|&(v, i)| seqs[v][i] > seqs[v - 1][i] && glows[v][i] < glows[v - 1][i])
        .count();
    assert!(
        recusados > 0,
        "nenhum pulso recusado — o sorteio não correu"
    );
    eprintln!("strobe com forma: {recusados} pulsos recusados");
}
