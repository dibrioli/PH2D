//! **O PREÇO DO GRUPO DOS DEFORMADORES** — o relógio, a contagem de objectos e a coluna do
//! dispositivo. Irmão do [`motion_deformadores_probe`](super::motion_deformadores_probe) pelo
//! tecto de 600 LOC (HR-18), e o corte é por RESPONSABILIDADE: *aquele* responde **o que o nó
//! declara e o que o cartão pinta** (retratos), este responde **quanto custa e onde corre**.
//!
//! ⚠️ **O que vem junto vem por necessidade, não por arrumação:** o [`acordar`] só faz sentido
//! ao lado do relógio que ele existe para tornar honesto, e o censo
//! [`waking_a_node_takes_it_off_the_identity`] é a régua DELE — separá-los poria a lei num
//! ficheiro e a prova noutro.

use super::motion_deformadores_probe::GRUPO;
use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

fn wire(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    m.doc
        .graph
        .connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed: false,
        })
        .expect("liga");
}

/// Monta `motion.grid(lado × lado) [→ <nó>] → motion.output` e devolve o sink.
fn build_com(m: &mut MotionState, lado: f32, no: Option<&str>, aceso: bool) -> NodeId {
    let grid = m.doc.graph.add_node("motion.grid");
    m.doc.graph.set_param(grid, "rows", lado);
    m.doc.graph.set_param(grid, "cols", lado);
    let out = m.doc.graph.add_node("motion.output");
    match no {
        Some(nome) => {
            let d = m.doc.graph.add_node(nome.to_string());
            if aceso {
                acordar(m, d, nome);
            }
            wire(m, grid, 0, d, 0);
            wire(m, d, 0, out, 0);
        }
        None => wire(m, grid, 0, out, 0),
    }
    out
}

/// ⛔⛔ **TIRA O NÓ DO PONTO NEUTRO ANTES DE O CRONOMETRAR.**
///
/// ⚠️ **A primeira redacção desta sonda media cada nó nos DEFAULTS dele**, e o
/// `motion.bezier_warp` denunciou-a assim que chegou ao dispositivo: ele leu **`0,34×` o custo
/// da grelha sozinha** — *mais rápido que não estar lá* —, porque com os 24 offsets a zero o
/// `eval` toma o atalho da identidade e devolve o stream clonado. A tabela dizia que o patch de
/// Coons é barato e o que ela cronometrava era um `clone`.
///
/// ⚠️ **Não é um caso especial dele:** metade deste grupo nasce na identidade (o `motion.move`
/// em `(0,0)`, o `rotate` a `0°`, os dois warps com os cantos parados). *Um corpus no ponto
/// neutro de um knob não testa esse knob*, e uma tabela de PREÇO medida no neutro mede o preço
/// de não fazer nada.
///
/// ⚠️ **Os knobs movidos são DERIVADOS dos hints, nunca uma tabela à mão** — uma lista escrita
/// aqui envelheceria em silêncio a cada param novo, e um param renomeado deixaria de ser
/// movido sem nada acusar.
///
/// ⛔ **Só os controlos CONTÍNUOS se movem** — mexer num `Enum`, num `Toggle` ou num `Seed`
/// mudaria o MODO do nó, que é outra medição.
///
/// ⛔⛔ **E a 1.ª redacção lia «contínuo» como `Slider`, o que deixou DOIS nós por acordar.** O
/// `motion.rotate` **não tem um único `Slider`**: o controlo dele é um `ParamWidget::Angle`, e a
/// linha dele na tabela cronometrava uma rotação de **0°** — *a identidade que este despertar
/// existe para evitar*. O `motion.bend` tem a mesma forma no `direction`, que é justamente o
/// param que a cláusula `applicable` dele lê, então a coluna do dispositivo dizia 🟢 sobre um nó
/// com aquele knob adormecido. ⚠️ **Um `Angle` é um NÚMERO com uma unidade em graus, não um
/// modo** — a razão escrita para excluir widgets nunca o cobriu.
///
/// ⚠️⚠️ **Isto NÃO é suficiente para deixar a coluna do dispositivo em paz, e eu escrevi aqui
/// que era.** A cláusula `applicable` do `motion.look_at` lê `target_x`/`target_y`, que são
/// **sliders**: a primeira corrida com o despertar acendeu-os e o nó apareceu 🔴 — a minha
/// perturbação com cara de regressão. ⇒ a coluna do dispositivo passou a ser lida num grafo
/// SEPARADO, nos defaults (ver [`cook_com`]), e o desacordo entre os dois virou uma leitura
/// própria (🟡). *Não há subconjunto de knobs seguro: a única forma de não perturbar uma
/// medição é não a fazer no mesmo grafo.*
fn acordar(m: &mut MotionState, no: NodeId, nome: &str) {
    let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
    let mut i = 0u32;
    for h in m.registry.param_ui(tid).unwrap_or(&[]) {
        if !matches!(
            h.widget,
            ph2d_node_registry::ParamWidget::Slider
                | ph2d_node_registry::ParamWidget::IntSlider
                | ph2d_node_registry::ParamWidget::Angle
        ) {
            continue;
        }
        // ⛔⛔ **A fracção VARIA com a ordem, e a fracção única deixava dois nós na
        // identidade.** Com `0,25` para todos, os oito `P0X..P3Y` do `motion.spline_wrap`
        // recebiam **o mesmo número** ⇒ os quatro pontos de controlo caíam **em cima uns dos
        // outros**, a cúbica media comprimento zero e o nó tomava o atalho inerte: a linha da
        // tabela cronometrava um `clone`, que é exactamente o defeito que este despertar
        // existe para curar, um nível abaixo. O mesmo valia para os quatro cantos do
        // `motion.four_point_warp`.
        //
        // ⚠️ **A razão áurea, e não um sorteio:** ela é determinística (a mesma tabela em toda
        // corrida), nunca repete um valor em `n` pequeno, e não precisa de saber quais hints
        // formam uma tupla — *uma lista escrita à mão de «estes dois são um ponto»
        // envelheceria a cada param novo*.
        //
        // ⚠️ **Preço declarado:** acrescentar ou reordenar um hint muda os números desta
        // tabela. Ela mede *o nó a trabalhar*, não uma pose autorada — a comparação que vale é
        // entre as linhas da MESMA corrida.
        // A faixa é `0,25..0,75`: em `i = 0` ela dá o `0,25` de sempre (⚠️ e isso é
        // load-bearing para o `motion.scale`, cuja faixa é `0..5` — uma fracção de `0,2`
        // pediria `Scale = 1`, que é **a identidade**), e sobe daí sem tocar nos extremos,
        // que é onde uma cerca degenerada moraria.
        #[expect(
            clippy::cast_precision_loss,
            reason = "um índice de hint; a fracção é o que interessa"
        )]
        let frac = 0.25 + 0.5 * (0.618_034_f32 * i as f32).fract();
        i += 1;
        let alvo = h.min + (h.max - h.min) * frac;
        let v = if (alvo - h.min).abs() < f32::EPSILON {
            h.min + (h.max - h.min) * 0.5
        } else {
            alvo
        };
        m.doc.graph.set_param(no, h.param, v);
    }
}

/// Quantos cozimentos frios por nó, e quantos para a linha de base — ver o comentário
/// no [`measure_the_deformer_group`].
const NO_REPS: usize = 3;
const BASE_REPS: usize = NO_REPS;

/// A mediana de 3 cozimentos **FRIOS** — um `MotionState` novo por corrida, senão o memo do cook
/// responde à segunda e a sonda mede a tabela de hash. Devolve `(ms, n, no_device)`.
/// O que uma linha da tabela diz.
struct Medida {
    /// A mediana de 3 cozimentos FRIOS, em ms — com o nó **acordado** (ver [`acordar`]).
    ms: f64,
    /// Quantos objectos saíram.
    n: usize,
    /// ⛔⛔ **O planeador reivindica a cadeia com o nó nos DEFAULTS dele?** É esta a pergunta
    /// do produto: é o que o artista recebe ao largar o nó.
    gpu_neutro: bool,
    /// E com o nó acordado. ⚠️ **Quando as duas diferem, a residência do nó DEPENDE de um
    /// knob** — não é ruído da sonda, é uma propriedade do nó que vale a pena ler.
    gpu_aceso: bool,
}

/// ⛔⛔ **DUAS PERGUNTAS, DOIS GRAFOS — e a primeira redacção respondia-as com um só.**
///
/// O relógio quer o nó a **trabalhar** ([`acordar`]); a coluna do dispositivo quer o nó como o
/// artista o **recebe**. Misturá-las custou uma leitura errada na primeira corrida: acordar o
/// `motion.look_at` pôs `target_x`/`target_y` fora de zero, que é exactamente o que a cláusula
/// `applicable` dele lê, e a coluna virou 🔴 — *a MINHA perturbação, lida como uma regressão do
/// produto*. ⚠️ E o comentário que eu tinha escrito ao lado do `acordar` dizia que só `Enum` e
/// `Toggle` alimentam um `applicable`: **falso**, e a tabela desmentiu-o na corrida seguinte.
fn cook_com(lado: f32, no: Option<&str>, repeticoes: usize) -> Medida {
    let mut ms: Vec<f64> = Vec::new();
    let (mut n, mut gpu_aceso) = (0usize, false);
    for _ in 0..repeticoes {
        let mut m = MotionState::new();
        let sink = build_com(&mut m, lado, no, true);
        gpu_aceso =
            ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu();
        let t = std::time::Instant::now();
        let out = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .expect("coze");
        ms.push(t.elapsed().as_secs_f64() * 1000.0);
        n = out[0].as_stream().count();
    }
    ms.sort_by(f64::total_cmp);
    let gpu_neutro = {
        let mut m = MotionState::new();
        let sink = build_com(&mut m, lado, no, false);
        ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, sink).is_fully_gpu()
    };
    Medida {
        ms: ms[1],
        n,
        gpu_neutro,
        gpu_aceso,
    }
}

/// ⭐⭐⭐ **O DESPERTAR TEM DE ACORDAR** — para cada um dos treze, a saída do nó ACESO difere da
/// saída dele nos defaults.
///
/// ⛔⛔ **É o censo que faltava, e ele nasceu de DOIS defeitos que a tabela escondia** (08/09):
/// - o `motion.rotate` **não tem um único `Slider`** (o controlo dele é um `ParamWidget::Angle`),
///   então o despertar não lhe tocava e a linha dele cronometrava uma rotação de **0°**;
/// - a fracção era a MESMA para todos os hints, então os oito `P0X..P3Y` do
///   `motion.spline_wrap` caíam no mesmo número, os quatro pontos de controlo colapsavam num
///   ponto, a cúbica media comprimento zero e o nó tomava o **atalho inerte** — um `clone`
///   cronometrado como se fosse o embrulho.
///
/// ⚠️ **Nenhuma das duas era visível na tabela:** um `clone` e um deformador barato leem-se
/// iguais numa coluna de razão, e é por isso que a régua tem de ser a **saída**, nunca o relógio.
/// *Um corpus no ponto neutro de um knob não testa esse knob* — a mesma frase que o despertar
/// foi escrito para honrar, e que ele próprio violava em dois nós.
///
/// ⚠️⚠️ **E a 1.ª redacção DESTE gate comparava só a coluna `P`** — que é a terceira vez que
/// esta linha paga a mesma cegueira: o `motion.rotate` e o `motion.scale` escrevem `rot` e
/// `size`, **nunca `P`**, então os dois liam-se «não mudou» com o nó a girar. A comparação é do
/// **stream inteiro**, que é a única que não tem de saber o que cada nó escreve.
///
/// ⚠️ **A grelha é pequena de propósito** (8×8): a pergunta é *«mudou?»*, não *«quanto custa?»*.
#[test]
fn waking_a_node_takes_it_off_the_identity() {
    let mut mudos: Vec<&str> = Vec::new();
    for nome in GRUPO {
        let saida = |aceso: bool| {
            let mut m = MotionState::new();
            let sink = build_com(&mut m, 8.0, Some(nome), aceso);
            let out = m
                .pump
                .cook
                .cook(&m.doc.graph, &m.registry, sink, 0.0)
                .expect("coze");
            out[0].as_stream().clone()
        };
        if saida(false) == saida(true) {
            mudos.push(nome);
        }
    }
    assert!(
        mudos.is_empty(),
        "o despertar nao mexeu nestes: {mudos:?} -- ou o widget deles nao esta' na lista de \
         controlos continuos, ou a fraccao poe o no' de volta na identidade"
    );
}

/// ⭐⭐⭐ **O PREÇO DO GRUPO, e o 🔴 é o achado — nunca a razão** (doc 103 §5.1).
///
/// Um nó que cai na CPU **no meio de uma cadeia** não custa o que ele custa: custa o
/// dispositivo inteiro ([doc 98](../../docs/Motion%20Nodes/98_auditoria_de_performance_2026-09-01.md)
/// mediu `50,9×`). Por isso a coluna que decide é *«a cadeia inteira é reivindicada?»*, e o
/// relógio está ao lado só para dizer quanto.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture measure_the_deformer_group
/// ```
#[test]
#[ignore = "sonda de medição — corra à mão, em RELEASE e com a máquina calma"]
fn measure_the_deformer_group() {
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    // ⛔⛔ **A LINHA DE BASE leva EXACTAMENTE as amostras dos outros, e dar-lhe mais foi
    // construído, MEDIDO e revertido no mesmo dia** (2026-09-08).
    //
    // O raciocínio parecia sólido: ela é o DENOMINADOR das treze razões e é o menor número da
    // tabela, logo a mais sensível — entre duas corridas na mesma máquina calma os absolutos
    // repetiram dentro de `10 %` e a base moveu-se `21 %` (`0,53` → `0,64 ms`), o que sozinho
    // fez a razão do `motion.kaleidoscope` ler `17,30×` e depois `14,58×`.
    //
    // ⚠️ **Com `9` amostras a base leu `0,26 ms`** — metade do que lê com `3`, e fora da
    // dispersão das duas corridas anteriores. *O «cozimento frio» não é frio: o processo
    // AQUECE ao longo das repetições* (alocador, caches, preditor), então a mediana de nove
    // assenta abaixo da mediana de três. ⇒ com contagens diferentes dos dois lados, a razão
    // passa a comparar uma base bem aquecida contra um nó frio, e o `kaleidoscope` saltou para
    // `39,02×`.
    //
    // ⭐ **A lei que fica:** *o número de repetições faz parte da medição*, então os dois lados
    // de uma razão têm de o partilhar. A dispersão do denominador é real e vive na tabela — a
    // coluna que se cita é a **absoluta**, e a razão diz ordem de grandeza.
    let so_grade = cook_com(lado, None, BASE_REPS);
    let nu = so_grade.ms;
    eprintln!(
        "\n  grade {lado}×{lado} = {} objectos · so' a grade: {nu:.2} ms {}",
        so_grade.n,
        if so_grade.gpu_neutro { "🟢" } else { "🔴" }
    );
    eprintln!(
        "\n  ⚠️ o nó é medido ACORDADO (cada slider a 1/4 da faixa) — nos defaults, metade
     deste grupo é a identidade e o relógio media um `clone`.\n"
    );
    eprintln!("  nó                     | objectos  | cozimento  | vs. so' a grade | no device?");
    eprintln!(
        "  -----------------------|-----------|------------|-----------------|------------------"
    );
    for nome in GRUPO {
        let d = cook_com(lado, Some(nome), NO_REPS);
        let device = match (d.gpu_neutro, d.gpu_aceso) {
            (true, true) => "🟢 sim".to_string(),
            (false, false) => "🔴 NAO".to_string(),
            // A residência depende de um knob — a coluna diz qual dos dois lados é qual.
            (true, false) => "🟡 so' nos defaults".to_string(),
            (false, true) => "🟡 so' fora dos defaults".to_string(),
        };
        eprintln!(
            "  {nome:<23} | {:>9} | {:>7.2} ms | {:>14.2}× | {device}",
            d.n,
            d.ms,
            d.ms / nu.max(1e-9),
        );
    }
    eprintln!(
        "\n  🟢 = o planeador reivindica a cadeia inteira · 🔴 = ela cai na CPU
  🟡 = a residência DEPENDE de um knob (o `applicable` do kernel lê-o)
  (um quadro de 60 fps tem 16,67 ms)\n"
    );
}
