//! ⭐⭐⭐ **PARIDADE CPU↔GPU DA FAMÍLIA `pulse.*`** (ciclo 6, W2 — doc 110 §8).
//!
//! Até esta wave os **nove** nós de pulso estavam fora do dispositivo, e a medição que abriu a
//! wave mostrou que o preço não é o nó: numa cadeia `grid → beat → sim.spawn` a fronteira caía em
//! `sim.spawn:0` — **a simulação inteira** ia para a CPU por causa do metrónomo.
//!
//! ⛔⛔ **Um kernel de pulso falha de um modo que um gate de rota NÃO vê.** Um pulso é `0` ou `1`,
//! e a coisa toda é o INSTANTE em que ele vira `1`: um kernel que nunca dispare, ou que dispare em
//! todo tique, devolve uma coluna perfeitamente bem formada. Por isso este gate
//!
//! 1. corre **várias voltas** com o estado a atravessar o `pre` — que é onde a recorrência vive —,
//! 2. compara **tique a tique** com a CPU, e
//! 3. exige, no lado da CPU sozinho, que a coluna **tenha disparado e tenha ficado calada** —
//!    senão «as duas rotas concordam» é a afirmação vazia de duas colunas de zeros.
//!
//! `#[ignore]`: precisa de adapter.
//! ```text
//! cargo test -p ph2d-gpu-cook --test it --release -- --ignored --nocapture parity_pulse
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
/// O orçamento herdado dos gates irmãos (ADR-0126). ⚠️ Um pulso é `0`/`1`, então qualquer
/// divergência REAL de instante vale `1,0` — mil vezes a barra. *Esta barra não pode ser afinada
/// para passar: ou os dois lados disparam no mesmo tique, ou não.*
const EPS: f32 = 1e-4;
/// Quantas voltas. Três é o mínimo que exercita as três coisas: semear o estado, atravessá-lo, e
/// voltar a atravessá-lo já com história.
const VOLTAS: usize = 6;
const DT: f64 = 1.0 / 60.0;

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
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_pulse_compare::register(&mut reg).unwrap();
    ph2d_node_pulse_level::register(&mut reg).unwrap();
    ph2d_node_pulse_on_change::register(&mut reg).unwrap();
    ph2d_node_pulse_sample_hold::register(&mut reg).unwrap();
    ph2d_node_pulse_signal::register(&mut reg).unwrap();
    ph2d_node_pulse_threshold::register(&mut reg).unwrap();
    ph2d_node_pulse_counter::register(&mut reg).unwrap();
    ph2d_node_pulse_beat::register(&mut reg).unwrap();
    ph2d_node_pulse_adsr::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    reg
}

fn liga(g: &mut Graph, de: (NodeId, u16), para: (NodeId, u16), pre: bool) {
    g.connect(Edge {
        from: de,
        to: para,
        delayed: pre,
    })
    .unwrap();
}

/// A fonte de VALOR: uma grelha de 64 peças através de um `value.lfo` — um campo que sobe e desce,
/// que é o que um gatilho precisa de ver para ter alguma coisa que dizer.
///
/// ⚠️ **O `value.lfo` está no dispositivo**, então esta fonte não introduz fronteira nenhuma — se
/// introduzisse, o gate mediria a costura em vez do nó de pulso.
fn fonte(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 8.0);
    g.set_param(grid, "cols", 8.0);
    let lfo = g.add_node("value.lfo");
    g.set_param(lfo, "period", 0.25);
    g.set_param(lfo, "amplitude", 1.0);
    g.set_param(lfo, "offset", 0.0);
    // Uma fase por elemento: sem isto as 64 linhas disparam todas juntas e o gate mediria UMA
    // coluna repetida 64 vezes — o campo por-linha é o que separa este motor de um escalar.
    g.set_param(lfo, "phase_stagger", 0.05);
    liga(g, (grid, 0), (lfo, 0), false);
    lfo
}

/// Corre as DUAS rotas sobre a mesma cadeia, `VOLTAS` tiques, e compara a `coluna` do `sink` em
/// cada uma. Devolve as colunas da CPU (para o controlo de não-vacuidade).
fn paridade(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    rotulo: &str,
    monta: impl Fn(&mut Graph) -> NodeId,
    coluna: &str,
) -> Vec<Vec<f32>> {
    let mut g = Graph::new();
    let sink = monta(&mut g);
    g.validate(reg).expect("a cadeia é bem tipada");
    let plano = plan(&g, reg, reg, sink);
    assert!(
        plano.is_fully_gpu(),
        "{rotulo}: a cadeia tem de ser do dispositivo -- senao este gate mede a costura: {:?}",
        plano.boundaries
    );

    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    let mut cpu_todas = Vec::new();
    for volta in 0..VOLTAS {
        let t = volta as f64 * DT;
        let cpu = cook.cook(&g, reg, sink, t).expect("cpu cook");
        let cpu_col: Vec<f32> = match cpu[0].as_stream().get(coluna) {
            Some(Column::Scalar(v)) => v.clone(),
            outra => panic!("{rotulo}: a CPU nao emitiu `{coluna}` escalar: {outra:?}"),
        };
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
        let dev = gc
            .read_column(gpu, sink, coluna)
            .unwrap_or_else(|| panic!("{rotulo}: `{coluna}` nao voltou do device"));
        assert_eq!(
            cpu_col.len(),
            dev.len(),
            "{rotulo}: volta {volta} -- comprimentos diferentes"
        );
        // ⚠️ **A linha CULPADA é nomeada, não a pior diferença sozinha.** A 1.ª redacção imprimia
        // as oito primeiras linhas e a divergência estava na 43.ª: *uma mensagem que mostra o
        // princípio de um vector prova que o princípio está bem.*
        let (onde, pior) = cpu_col
            .iter()
            .zip(&dev)
            .enumerate()
            .map(|(k, (a, b))| (k, (a - b).abs()))
            .fold((0usize, 0.0_f32), |m, c| if c.1 > m.1 { c } else { m });
        assert!(
            pior < EPS,
            "{rotulo}: volta {volta} diverge em {pior} na linha {onde}\n  cpu: {:?}\n  dev: {:?}",
            &cpu_col[onde.saturating_sub(2)..(onde + 3).min(cpu_col.len())],
            &dev[onde.saturating_sub(2)..(onde + 3).min(dev.len())]
        );
        cpu_todas.push(cpu_col);
        // ⚠️⚠️ **`advance_tick` é o que PUBLICA o `pre` do lado da CPU** — sem ele o estado nunca
        // atravessa e o gate compara um device que avança com uma CPU congelada. A primeira
        // redacção deste ficheiro não o tinha, e os cinco gates reprovaram na volta **1** com
        // `1,0` de divergência: *a régua estava a medir a régua.* O device publica o dele sozinho
        // (o sequenciador segura os `Arc` do tique — `GpuSource::Prev`).
        cook.advance_tick(&g, reg, t).expect("cpu tick");
    }
    cpu_todas
}

/// ⚠️ **O CONTROLO DA NÃO-VACUIDADE** — a coluna da CPU tem de ter `1`s e `0`s ao longo das voltas.
/// Sem isto, um kernel que nunca dispare concorda perfeitamente com uma CPU que nunca dispare.
fn nao_e_vazio(rotulo: &str, voltas: &[Vec<f32>]) {
    let disparou = voltas.iter().flatten().any(|&x| x > 0.5);
    let calou = voltas.iter().flatten().any(|&x| x <= 0.5);
    assert!(
        disparou && calou,
        "{rotulo}: a fixture nao exercita o no' (disparou={disparou}, calou={calou}) -- \
         «as duas rotas concordam» seria a afirmacao vazia de duas colunas iguais"
    );
}

/// ⭐⭐⭐ **O GATILHO DE SCHMITT**, com a memória a atravessar o `pre` no device.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_compare() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.compare",
        |g| {
            let src = fonte(g);
            let cmp = g.add_node("pulse.compare");
            // ⚠️⚠️ **Os limiares NÃO são redondos, e isso não é afinar a barra até passar.** Um
            // limiar que o campo atinja EXACTAMENTE (`0,5`, que esta fixture pisa na linha 39)
            // transforma o ε legítimo do `value.lfo` num pulso inteiro — o fio da navalha, que
            // tem gate PRÓPRIO logo abaixo, com a medição. Aqui mede-se o NÓ; ali mede-se a
            // navalha. *Misturar os dois daria uma barra folgada a responder às duas perguntas.*
            g.set_param(cmp, "rise", 0.4321);
            g.set_param(cmp, "fall", 0.1777);
            liga(g, (src, 0), (cmp, 0), false);
            liga(g, (cmp, 0), (cmp, 1), true);
            cmp
        },
        "pulse",
    );
    nao_e_vazio("pulse.compare", &voltas);
}

/// ⭐⭐ **O NÍVEL** — a ponte pulso→valor, e o nó mais simples da família.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_level() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.level",
        |g| {
            let src = fonte(g);
            let cmp = g.add_node("pulse.compare");
            g.set_param(cmp, "rise", 0.4321);
            g.set_param(cmp, "fall", 0.1777);
            // ⚠️ `Both`: um `Rise` puro dispara UMA vez em cada volta da onda, e a coluna do nivel
            // ficaria quase toda a zero -- o controlo de nao-vacuidade passaria a medir o ruido.
            g.set_param(cmp, "edge", 2.0);
            liga(g, (src, 0), (cmp, 0), false);
            liga(g, (cmp, 0), (cmp, 1), true);
            let lvl = g.add_node("pulse.level");
            liga(g, (cmp, 0), (lvl, 0), false);
            lvl
        },
        "v",
    );
    nao_e_vazio("pulse.level", &voltas);
}

/// ⭐⭐ **O `on_change`** — a outra ponte valor→pulso, com DUAS colunas de estado.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_on_change() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.on_change",
        |g| {
            let src = fonte(g);
            let oc = g.add_node("pulse.on_change");
            // Sobe = dispara; desce = calado. Com a onda a passar pelos dois lados, a coluna tem
            // `1`s e `0`s de verdade.
            g.set_param(oc, "direction", 0.0);
            g.set_param(oc, "epsilon", 0.001);
            liga(g, (src, 0), (oc, 0), false);
            liga(g, (oc, 0), (oc, 1), true);
            oc
        },
        "pulse",
    );
    nao_e_vazio("pulse.on_change", &voltas);
}

/// ⭐⭐ **O AMOSTRADOR** — o valor segurado atravessa o `pre`, e o gate lê o `v`, não o pulso.
///
/// ⚠️ **O controlo aqui é OUTRO**: um valor segurado não é `0`/`1`. O que ele tem de provar é que o
/// valor **muda** ao longo das voltas (senão o nó seguraria o primeiro para sempre nos dois lados,
/// e o gate seria vazio outra vez) — e isso mede-se abaixo, não pelo [`nao_e_vazio`].
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_sample_hold() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.sample_hold",
        |g| {
            let src = fonte(g);
            let cmp = g.add_node("pulse.compare");
            g.set_param(cmp, "rise", 0.4321);
            g.set_param(cmp, "fall", 0.1777);
            g.set_param(cmp, "edge", 2.0);
            liga(g, (src, 0), (cmp, 0), false);
            liga(g, (cmp, 0), (cmp, 1), true);
            let sh = g.add_node("pulse.sample_hold");
            liga(g, (src, 0), (sh, 0), false);
            liga(g, (cmp, 0), (sh, 1), false);
            liga(g, (sh, 0), (sh, 2), true);
            sh
        },
        "v",
    );
    let primeira = &voltas[0];
    let mudou = voltas
        .iter()
        .any(|v| v.iter().zip(primeira).any(|(a, b)| (a - b).abs() > 1e-3));
    assert!(
        mudou,
        "o valor segurado nunca mudou em {VOLTAS} voltas -- a fixture nao dispara e o gate seria vazio"
    );
}

/// ⭐ **O SINAL é um PASSTHROUGH, e tem de o ser AO BIT.** O sequenciador não emite passe nenhum
/// para ele; este gate prova que a corrente que sai é a que entrou, no device como na CPU.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_signal_is_a_true_passthrough() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.signal",
        |g| {
            let src = fonte(g);
            let cmp = g.add_node("pulse.compare");
            g.set_param(cmp, "rise", 0.4321);
            g.set_param(cmp, "fall", 0.1777);
            g.set_param(cmp, "edge", 2.0);
            liga(g, (src, 0), (cmp, 0), false);
            liga(g, (cmp, 0), (cmp, 1), true);
            let sig = g.add_node("pulse.signal");
            liga(g, (cmp, 0), (sig, 0), false);
            sig
        },
        "pulse",
    );
    nao_e_vazio("pulse.signal", &voltas);
}

/// ⛔⛔⛔ **O FIO DA NAVALHA — o único sítio onde as duas rotas podem discordar, medido.**
///
/// Um nó de pulso é uma **comparação**, e uma comparação não tem ε: se o `v` que chega vale
/// `0,5000009` numa rota e `0,4999995` na outra — a diferença de `1,4e-6` que toda a família
/// `value.*` já declara e que nenhum gate de valor acusa —, o limiar em `0,5` fica **entre as
/// duas** e o pulso vale `1` de um lado e `0` do outro. Divergência medida: **`1,0`**.
///
/// ⚠️ **Isto NÃO é um defeito do kernel, e a afirmação não é de fé — é o que este gate prova:**
/// toda linha em que as duas rotas discordam é uma linha cujos dois valores **abraçam** o limiar.
/// Uma linha que discordasse com os dois valores do mesmo lado seria um defeito de lei, e reprova
/// aqui.
///
/// ⚠️ **A régua é ESTRUTURAL, não uma barra escolhida:** `min(v_cpu, v_dev) ≤ limiar ≤ max(…)`.
/// Não há constante nenhuma para afrouxar no dia em que isto reprovar.
///
/// ⚠️ E ela olha para a **PRIMEIRA** volta que discorda, de propósito: o `armed` é latched, logo
/// uma discordância propaga-se para as voltas seguintes com os valores já bem do mesmo lado. *Uma
/// régua aplicada a todas as voltas acusaria a consequência e não a causa.*
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn a_value_on_the_knife_edge_is_the_only_place_the_two_routes_can_disagree() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // O limiar que a fixture PISA: a linha 39 do campo vale `0,5000009` na CPU.
    const RISE: f32 = 0.5;
    const FALL: f32 = 0.2;

    let mut g = Graph::new();
    let src = fonte(&mut g);
    let cmp = g.add_node("pulse.compare");
    g.set_param(cmp, "rise", RISE);
    g.set_param(cmp, "fall", FALL);
    liga(&mut g, (src, 0), (cmp, 0), false);
    liga(&mut g, (cmp, 0), (cmp, 1), true);
    g.validate(&reg).expect("bem tipada");
    let plano = plan(&g, &reg, &reg, cmp);
    assert!(plano.is_fully_gpu(), "{:?}", plano.boundaries);

    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    let escalar = |s: &ph2d_nodegraph::attr::Stream, c: &str| -> Vec<f32> {
        match s.get(c) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        }
    };
    let mut discordantes = 0usize;
    let mut amostras = 0usize;
    let mut primeira: Option<(usize, usize, f32, f32)> = None;
    for volta in 0..VOLTAS {
        let t = volta as f64 * DT;
        let cpu = cook.cook(&g, &reg, cmp, t).expect("cpu cook");
        let cpu_pulse = escalar(cpu[0].as_stream(), "pulse");
        // ⚠️ O `v` do CONDUTOR, lido do próprio cozedor — não uma segunda derivação à mão, que é
        // como um gate passa a concordar consigo próprio em vez de com o produto.
        let cpu_v = escalar(
            cook.cook(&g, &reg, src, t).expect("cpu lfo")[0].as_stream(),
            "v",
        );
        gc.cook(
            &gpu,
            &g,
            &reg,
            &reg,
            &plano,
            &[],
            CookClock::at(t),
            DEFAULT_UV,
            DEFAULT_SIZE,
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        let dev_pulse = gc.read_column(&gpu, cmp, "pulse").expect("pulse");
        let dev_v = gc.read_column(&gpu, src, "v").expect("v");
        for k in 0..cpu_pulse.len() {
            amostras += 1;
            if (cpu_pulse[k] - dev_pulse[k]).abs() > EPS {
                discordantes += 1;
                if primeira.is_none() {
                    primeira = Some((volta, k, cpu_v[k], dev_v[k]));
                }
            }
        }
        cook.advance_tick(&g, &reg, t).expect("cpu tick");
    }

    let Some((volta, linha, v_cpu, v_dev)) = primeira else {
        panic!(
            "esta fixture existe para PISAR o fio da navalha e nao discordou em {amostras} \
             amostras -- o campo ou o limiar mudaram, e o gate deixou de afirmar o que diz"
        );
    };
    eprintln!(
        "\n  fio da navalha: {discordantes} de {amostras} amostras · 1.ª na volta {volta}, \
         linha {linha}\n  v_cpu = {v_cpu} · v_dev = {v_dev} · limiar = {RISE} (delta = {})\n",
        (v_cpu - v_dev).abs()
    );
    let abraca = |limiar: f32| v_cpu.min(v_dev) <= limiar && limiar <= v_cpu.max(v_dev);
    assert!(
        abraca(RISE) || abraca(FALL),
        "a 1.ª discordancia (volta {volta}, linha {linha}) tem v_cpu={v_cpu} e v_dev={v_dev}, e \
         NENHUM limiar ({RISE} / {FALL}) fica entre os dois -- isto nao e' o fio da navalha, \
         e' um defeito de lei no kernel"
    );
}

/// ⭐⭐ **O GATILHO SOBRE UM CANAL DE TRANSFORME** — o irmão do `compare` que lê a geometria.
///
/// ⚠️ O canal é **Y**, que é o default do nó, e a grelha dá-lhe um campo que sobe e desce.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_threshold() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.threshold",
        |g| {
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", 8.0);
            g.set_param(grid, "cols", 8.0);
            let th = g.add_node("pulse.threshold");
            // O campo da grelha e' estatico, entao o gatilho arma no 1.º tique e cala-se: com
            // `Both` a coluna tem `1`s e `0`s de verdade ao longo das voltas.
            g.set_param(th, "channel", 1.0);
            g.set_param(th, "rise", 0.0111);
            g.set_param(th, "fall", -0.3333);
            g.set_param(th, "edge", 2.0);
            liga(g, (grid, 0), (th, 0), false);
            liga(g, (th, 0), (th, 1), true);
            th
        },
        "pulse",
    );
    nao_e_vazio("pulse.threshold", &voltas);
}

/// ⭐⭐ **E COM `debounce > 0` ELE CONTINUA NO DISPOSITIVO** — o espaço de params INTEIRO.
///
/// ⛔ **Este gate afirmava o CONTRÁRIO durante meia wave**, e a nota fica: o kernel nasceu com um
/// `applicable: Some(|p| !(p("debounce") > 0.0))`, porque o abrandador conta um relógio para trás
/// e o módulo gerado não tinha `dt`. A cura não foi afinar a recusa — foi pôr o **`dt` no uniform**
/// (`codegen::kernel_module` + `PARAMS_AT`), e a recusa evaporou. *Um `applicable` é uma dívida
/// datada: ele diz o que o SUBSTRATO não sabia no dia em que foi escrito, e nada sobre o nó.*
///
/// ⚠️ A régua varre a faixa do param em vez de olhar para um valor: um `applicable` reintroduzido
/// com a comparação invertida passaria num ponto só.
#[test]
fn a_debounced_threshold_is_still_claimed_across_the_whole_param_range() {
    let reg = registry();
    for debounce in [0.0_f32, 0.001, 0.25, 2.0, 60.0] {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let th = g.add_node("pulse.threshold");
        liga(&mut g, (grid, 0), (th, 0), false);
        liga(&mut g, (th, 0), (th, 1), true);
        g.set_param(th, "debounce", debounce);
        assert!(
            plan(&g, &reg, &reg, th).is_fully_gpu(),
            "com `debounce = {debounce}` o no' tem de continuar no dispositivo -- o `dt` e' uniform \
             desde o ciclo 6 W2"
        );
    }
}

/// ⭐⭐ **O CONTADOR** — aritmética inteira Euclidiana, nos três modos de limite.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_counter() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // ⚠️ **Os TRÊS modos**, porque cada um é uma lei diferente (`rem_euclid` · `clamp` · o
    // triângulo) e um kernel que portasse só o primeiro passaria num gate de um modo só.
    for (modo, nome) in [(0.0, "Wrap"), (1.0, "Clamp"), (2.0, "Zigzag")] {
        let voltas = paridade(
            &gpu,
            &reg,
            &format!("pulse.counter({nome})"),
            |g| {
                let src = fonte(g);
                let cmp = g.add_node("pulse.compare");
                g.set_param(cmp, "rise", 0.4321);
                g.set_param(cmp, "fall", 0.1777);
                g.set_param(cmp, "edge", 2.0);
                liga(g, (src, 0), (cmp, 0), false);
                liga(g, (cmp, 0), (cmp, 1), true);
                let ct = g.add_node("pulse.counter");
                g.set_param(ct, "count_max", 3.0);
                g.set_param(ct, "mode", modo);
                liga(g, (cmp, 0), (ct, 0), false);
                liga(g, (ct, 0), (ct, 1), true);
                ct
            },
            "v",
        );
        let andou = voltas
            .iter()
            .any(|v| v.iter().zip(&voltas[0]).any(|(a, b)| (a - b).abs() > 0.5));
        assert!(
            andou,
            "{nome}: a contagem nunca andou em {VOLTAS} voltas -- a fixture nao dispara"
        );
    }
}

/// ⛔⛔ **E COM O CARRY LIGADO ELE RECUA** — a porta ≠ 0, que é a metade (b) da W1.
///
/// ⚠️ **Um estágio de GPU produz UM buffer** (`GpuStage` guarda um `node`, nunca um `(nó, porta)`),
/// e sem esta recusa um consumidor do `carry` receberia a porta **0**: a corrente errada, em
/// silêncio. A recusa já existia no planeador e **nomeava este nó**; o que faltava era o kernel
/// para ela proteger — logo é agora que ela passa a ter efeito, e é agora que ela precisa de gate.
#[test]
fn a_counter_with_its_carry_wired_is_refused_by_the_planner() {
    let reg = registry();
    let monta = |carry: bool| {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let bt = g.add_node("pulse.beat");
        liga(&mut g, (grid, 0), (bt, 0), false);
        liga(&mut g, (bt, 0), (bt, 1), true);
        let ct = g.add_node("pulse.counter");
        liga(&mut g, (bt, 0), (ct, 0), false);
        liga(&mut g, (ct, 0), (ct, 1), true);
        if carry {
            let ct2 = g.add_node("pulse.counter");
            // A porta 1 do PRIMEIRO contador -- o divisor de relógio clássico.
            liga(&mut g, (ct, 1), (ct2, 0), false);
            liga(&mut g, (ct2, 0), (ct2, 1), true);
            let s = ct2;
            return (g, s);
        }
        (g, ct)
    };
    let (g, sink) = monta(false);
    assert!(
        plan(&g, &reg, &reg, sink).is_fully_gpu(),
        "com o carry solto a cadeia e' do dispositivo"
    );
    let (g, sink) = monta(true);
    assert!(
        !plan(&g, &reg, &reg, sink).is_fully_gpu(),
        "com o carry LIGADO o planeador tem de recuar -- um estagio produz UM buffer"
    );
}

/// ⭐⭐⭐ **O METRÓNOMO — e com ele a cadeia que a W2 mediu passa a ser do dispositivo.**
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_beat() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let voltas = paridade(
        &gpu,
        &reg,
        "pulse.beat",
        |g| {
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", 8.0);
            g.set_param(grid, "cols", 8.0);
            let bt = g.add_node("pulse.beat");
            // ⚠️ Um período que NÃO é múltiplo do tique (`1/60`): com um período redondo toda
            // batida cai em cima de uma amostra, que é o fio da navalha do §8 — e ali a régua é
            // outra. Aqui mede-se a LEI.
            g.set_param(bt, "period", 0.0537);
            // Uma fase por linha: sem ela as 64 linhas são a mesma coluna repetida.
            g.set_param(bt, "phase_stagger", 0.0031);
            liga(g, (grid, 0), (bt, 0), false);
            liga(g, (bt, 0), (bt, 1), true);
            bt
        },
        "pulse",
    );
    nao_e_vazio("pulse.beat", &voltas);
}

/// ⛔⛔⛔ **A CADEIA QUE ABRIU A W2 CONTINUA BLOQUEADA — e por OUTRA coisa. Este gate é o registo.**
///
/// A medição que abriu a wave mediu `grid → beat → sim.spawn(pulse) → output` com a fronteira em
/// **`sim.spawn:0`**: a simulação inteira na CPU por causa do metrónomo. Com os nove `pulse.*` no
/// dispositivo, o metrónomo deixou de ser a causa — **e a cadeia continua a recuar**, porque o
/// `sim.spawn` declara um `ColumnAccess::RefuseIfPresent` sobre a coluna `pulse` da porta 1.
///
/// ⚠️⚠️ **A justificação ESCRITA daquela recusa era *«a família `pulse.*` não tem kernel nenhum,
/// logo a cadeia que alimenta esta porta já é uma fronteira»* — e esta wave dissolveu-a.** O que
/// segura a recusa hoje é a OUTRA perna, que estava no mesmo comentário: um nascimento por pulso
/// nasce **na linha que disparou**, e a contagem disso é dado — mas o `CountLawCtx` proíbe por
/// escrito olhar para o CONTEÚDO de uma entrada (*«a law may only ask how WIDE its inputs are»*),
/// porque isso seria um readback. ⇒ **é uma wave de substrato**, não um kernel que falte.
///
/// ⚠️ **Este gate reprova no dia em que alguém curar o `sim.spawn`** — e é isso que ele existe para
/// fazer: obrigar quem o curar a vir aqui apagar a nota que deixou de ser verdade.
#[test]
fn the_chain_that_opened_the_wave_is_blocked_by_the_spawn_and_no_longer_by_the_metronome() {
    let mut reg = registry();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_sim_spawn::register(&mut reg).unwrap();
    let monta = |com_pulso: bool| {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let sp = g.add_node("sim.spawn");
        let out = g.add_node("motion.output");
        liga(&mut g, (grid, 0), (sp, 0), false);
        if com_pulso {
            let bt = g.add_node("pulse.beat");
            liga(&mut g, (grid, 0), (bt, 0), false);
            liga(&mut g, (bt, 0), (bt, 1), true);
            liga(&mut g, (bt, 0), (sp, 1), false);
        }
        liga(&mut g, (sp, 0), (out, 0), false);
        (g, out)
    };
    // ⚠️ **O CONTROLO vem primeiro, e é ele que nomeia o culpado.** Sem a porta de pulso fiada o
    // `sim.spawn` É do dispositivo — logo o que recua não é o spawn nem o metrónomo: é a PORTA.
    let (g, out) = monta(false);
    let plano = plan(&g, &reg, &reg, out);
    assert!(
        plano.is_fully_gpu(),
        "sem a porta de pulso a cadeia e' do dispositivo: {:?}",
        plano.boundaries
    );
    assert!(plano.stages.len() >= 3, "{:?}", plano.stages.len());

    let (g, out) = monta(true);
    let plano = plan(&g, &reg, &reg, out);
    assert!(
        !plano.is_fully_gpu(),
        "o `sim.spawn` RECUSA a porta `pulse` (nascimento na linha que disparou) -- se isto passou \
         a ser verdade, a recusa foi curada e a nota deste gate tem de ser reescrita"
    );

    // ⭐ **E o metrónomo, esse, já é do dispositivo** — a metade que esta wave comprou, afirmada
    // sozinha para que a recusa do spawn não a esconda.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let bt = g.add_node("pulse.beat");
    liga(&mut g, (grid, 0), (bt, 0), false);
    liga(&mut g, (bt, 0), (bt, 1), true);
    assert!(
        plan(&g, &reg, &reg, bt).is_fully_gpu(),
        "o `pulse.beat` tem kernel desde a W2 -- esta e' a metade que a wave comprou"
    );
}

/// ⭐⭐ **O ABRANDADOR do `pulse.threshold`** — a outra metade que o `dt` uniform destravou.
///
/// ⛔⛔ **A 1.ª redacção deste gate era VAZIA, e foi uma prova de mutação que o disse:** ela punha o
/// gatilho sobre uma grelha ESTÁTICA, que arma no primeiro tique e nunca mais cruza o limiar —
/// logo o abrandador nunca chegava a abrandar, e a mutação que zera o `dt` no uniform **sobreviveu**
/// (o relógio nunca precisava de descer). *Um controlo de «disparou e calou-se» é satisfeito por um
/// disparo único, que é exactamente o caso que não testa nada.*
///
/// ⇒ o campo passa a ser um `motion.oscillator` RÁPIDO (também do dispositivo, logo sem costura), e
/// o controlo é **comparativo**: a mesma cadeia com `debounce = 0` tem de disparar **MAIS**. Sem
/// essa metade, «as duas rotas concordam» volta a ser a afirmação vazia de duas colunas que nunca
/// chegam à janela de silêncio.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn parity_pulse_threshold_with_a_debounce() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    // Um campo que atravessa o limiar VÁRIAS vezes dentro das `VOLTAS`: `8 Hz` a `1/60 s` dá uma
    // volta completa a cada `7,5` tiques, e com `edge = Both` isso são dois cruzamentos por volta.
    let monta = |debounce: f32| {
        move |g: &mut Graph| {
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", 8.0);
            g.set_param(grid, "cols", 8.0);
            let osc = g.add_node("motion.oscillator");
            g.set_param(osc, "channel", 1.0); // Y
            g.set_param(osc, "amplitude", 1.0);
            g.set_param(osc, "frequency", 8.0);
            g.set_param(osc, "phase_stagger", 0.017);
            liga(g, (grid, 0), (osc, 0), false);
            let th = g.add_node("pulse.threshold");
            g.set_param(th, "channel", 1.0);
            g.set_param(th, "rise", 0.1111);
            g.set_param(th, "fall", -0.1111);
            g.set_param(th, "edge", 2.0);
            g.set_param(th, "debounce", debounce);
            liga(g, (osc, 0), (th, 0), false);
            liga(g, (th, 0), (th, 1), true);
            th
        }
    };
    // Duas voltas de silêncio a `1/60 s` — curto o suficiente para as `VOLTAS` o atravessarem,
    // longo o suficiente para engolir cruzamentos a `8 Hz`.
    let com = paridade(
        &gpu,
        &reg,
        "pulse.threshold(debounce)",
        monta(0.033),
        "pulse",
    );
    nao_e_vazio("pulse.threshold(debounce)", &com);

    // ⚠️ **O CONTROLO que prova que o abrandador ABRANDA** (e, com ele, que o `dt` desce o relógio).
    let sem = paridade(
        &gpu,
        &reg,
        "pulse.threshold(sem debounce)",
        monta(0.0),
        "pulse",
    );
    let conta = |v: &[Vec<f32>]| v.iter().flatten().filter(|&&x| x > 0.5).count();
    let (a, b) = (conta(&com), conta(&sem));
    assert!(
        a < b,
        "com `debounce` dispararam {a} e sem ele {b} -- o abrandador nao engoliu nada, logo este \
         gate nao testa o relogio que o `dt` faz descer"
    );
    eprintln!(
        "\n  abrandador: {a} disparos com `debounce`, {b} sem -- engoliu {}\n",
        b - a
    );
}
