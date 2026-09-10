//! ⭐⭐⭐ **O INSTRUMENTO DE UM CICLO** — o retrato, os params, o cartão, o despertar e o preço,
//! para **qualquer** grupo de nós (protocolo do [doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)).
//!
//! ⛔ **Nasceu ao abrir o ciclo 4, e a razão é a lei da casa:** este código foi escrito uma vez
//! para o ciclo 3 e ia ser copiado para o 4 — *uma lei escrita em dois sítios ainda não é uma
//! lei, só uma PORTA é*. Copiá-lo deixaria duas réguas a envelhecer em sentidos diferentes, e
//! sete ciclos ainda por abrir.
//!
//! ⚠️ **A coluna do dispositivo lê-se do `register_gpu_kernel`, NUNCA do `lowerings`** — o ciclo
//! 2 pagou essa: o `lowerings` é o que o nó declara saber baixar **sozinho**, e imprimir essa
//! coluna fabrica uma tabela de dívida com oito kernels que já existem.
//!
//! ⚠️ **Cada ciclo mantém os NOMES dos seus testes** (os docs citam-nos) e chama estas portas —
//! o que muda entre ciclos é a lista de nós, nunca a régua.

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Edge, NodeId};

// ---------------------------------------------------------------------------------------------
// 0. O GRUPO — derivado do registry quando ele é uma FAMÍLIA.
// ---------------------------------------------------------------------------------------------

/// ⭐⭐ **Os nós registados cujo nome começa por `prefixo`, em ordem.**
///
/// ⚠️ **Um ciclo cujo grupo é uma FAMÍLIA (`force.*`, `value.*`, `rig.*`) não a escreve à mão:**
/// uma lista escrita aqui envelhece em silêncio no dia em que um nó da família nasce, e o ciclo
/// fecha com ele por auditar. *A fonte é o registry.*
pub(crate) fn familia(prefixo: &str) -> Vec<&'static str> {
    let m = MotionState::new();
    let mut v: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| man.name.starts_with(prefixo))
        .map(|man| man.name)
        .collect();
    v.sort_unstable();
    v
}

// ---------------------------------------------------------------------------------------------
// 1. O RETRATO — o que cada nó do grupo declara.
// ---------------------------------------------------------------------------------------------

/// Uma linha por nó: params · quantos chegam ao cartão · dispositivo · portas · efeito.
pub(crate) fn retrato(grupo: &[&str]) {
    let base = MotionState::new();
    eprintln!("\n  nó                        | params | no cartão | device | portas | efeito");
    eprintln!("  --------------------------|--------|-----------|--------|--------|--------");
    for nome in grupo {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node((*nome).to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            base.registry.resolve(tid).map(|op| op.manifest())
        };
        let Some(man) = man else {
            eprintln!("  {nome:<26} | (nao registado)");
            continue;
        };
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let no_cartao = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or(0, |v| v.params.len());
        let device = {
            use ph2d_nodegraph::gpu::KernelResolver;
            m.registry.gpu_kernel(tid).is_some()
        };
        eprintln!(
            "  {nome:<26} | {:>6} | {no_cartao:>9} | {:^6} | {}->{:<4} | {:?}",
            man.params.len(),
            if device { "sim" } else { "NAO" },
            man.inputs.len(),
            man.outputs.len(),
            man.effect,
        );
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 2. OS PARAMS, um a um — o que a auditoria compara contra as referências.
// ---------------------------------------------------------------------------------------------

/// ⚠️ Sem esta lista, *«falta X»* é um palpite.
pub(crate) fn params_de(grupo: &[&str]) {
    let m = MotionState::new();
    for nome in grupo {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        eprintln!("\n  === {nome} ===");
        eprintln!(
            "  portas: [{}] -> [{}]",
            man.inputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", "),
            man.outputs
                .iter()
                .map(|p| p.name)
                .collect::<Vec<_>>()
                .join(", ")
        );
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let h = hints.iter().find(|h| h.param == spec.name);
            let w = h.map_or("(sem hint)".to_string(), |h| match h.widget {
                ph2d_node_registry::ParamWidget::Enum { labels } => {
                    format!("Enum[{}]", labels.join("|"))
                }
                outro => format!("{outro:?}"),
            });
            eprintln!(
                "    {:<20} default {:>9.3}  {:<16}  {}",
                spec.name,
                spec.default,
                h.map_or("—".to_string(), |h| format!("{}..{}", h.min, h.max)),
                w
            );
        }
        // ⚠️ **Os params de TEXTO não estão no manifesto** — eles vivem no canal aditivo
        // (`Graph::set_text_param`) e aparecem só como um `ParamUiHint` sem `ParamSpec` ao
        // lado. Derivá-los da diferença é a única forma de não escrever uma segunda lista.
        for h in hints {
            if man.params.iter().any(|p| p.name == h.param) {
                continue;
            }
            eprintln!(
                "    {:<20} (TEXTO)  {:<16}  {:?}",
                h.param, h.label, h.widget
            );
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 3. O CARTÃO — as rows que ele DE FACTO pinta, com o rótulo da tela.
// ---------------------------------------------------------------------------------------------

/// ⚠️ **Um passo de smoke que manda clicar numa linha AFIRMA que ela está na lista**, e a casa
/// já pagou por escrever um passo impossível. Esta porta imprime o que o `stamp_card_params`
/// produz, que é literalmente o que o pintor desenha.
///
/// ⚠️ **As SECÇÕES fazem parte do que o cartão MOSTRA, e a 1.ª redacção não as lia:** o plano do
/// ciclo 3 acusou o `motion.bezier_warp` de pintar `In X · In Y · …` quatro vezes sem dizer de
/// que aresta — uma acusação construída sobre a lista de rótulos, que é metade da resposta.
/// *Uma sonda que lê metade da superfície fabrica dívida.*
/// O NOME que o cartão de cada nó pinta — o que o tutorial tem de escrever.
///
/// ⚠️ **Ele não é o `type_name`**: o `field.remap` pinta-se **`Remap`**, e um passo de smoke que
/// diga *«o cartão `Field Remap`»* manda o dono procurar uma coisa que não existe.
pub(crate) fn nomes(grupo: &[&str]) {
    let m = MotionState::new();
    for nome in grupo {
        let mut d = MotionState::new();
        let id = d.doc.graph.add_node((*nome).to_string());
        let snap = ph2d_panel_motion_graph::snapshot_from(&d.doc.graph, &d.registry);
        let t = snap
            .nodes
            .iter()
            .find(|v| v.id == id.0)
            .map_or("(sem cartao)", |v| v.display_name.as_str());
        eprintln!("  {nome:<26} -> cartão «{t}»");
    }
    let _ = m;
    eprintln!();
}

pub(crate) fn cartao(grupo: &[&str]) {
    for nome in grupo {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node((*nome).to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        let view = snap.nodes.iter().find(|v| v.id == id.0);
        let rows: Vec<String> = view
            .map(|v| v.params.iter().map(|c| c.hint.label.to_string()).collect())
            .unwrap_or_default();
        let secs: Vec<String> = view
            .map(|v| {
                v.sections
                    .iter()
                    .map(|s| {
                        format!(
                            "{}@{}{}",
                            s.title,
                            s.at,
                            if s.open { "" } else { " (fechada)" }
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        eprintln!("  {nome:<26} | {}", rows.join(" · "));
        if !secs.is_empty() {
            eprintln!("  {:<26} > secções: {}", "", secs.join(" · "));
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 3-bis. O VOCABULÁRIO — dois nós que guardam a MESMA pergunta chamam-lhe o mesmo nome?
// ---------------------------------------------------------------------------------------------

/// ⭐⭐⭐ **O achado §2.3 do ciclo 3, virado régua** — *«seis vocabulários para onde é o centro»*.
///
/// Para cada nome de param que **dois ou mais** nós do grupo declaram, imprime os rótulos
/// distintos que eles pintam. Um nome partilhado com rótulos diferentes é o defeito: com o
/// painel lateral fora, o cartão é a **única** superfície onde estes nomes aparecem, e o artista
/// que aprendeu um tem de reconhecer o outro.
///
/// ⚠️ **A população é DERIVADA do manifesto**, nunca de uma lista de nós escrita à mão — foi
/// assim que o censo do canto do ciclo 3 (`every_node_that_offsets_a_corner_calls_it_the_same_thing`)
/// se manteve honesto quando um terceiro nó apareceu.
///
/// ⚠️ **Divergir pode ser CERTO** (um rótulo mais específico desambigua), e é por isso que esta
/// porta **imprime** em vez de reprovar: quem lê decide, e escreve a decisão ao lado.
pub(crate) fn vocabulario(grupo: &[&str]) {
    use std::collections::BTreeMap;
    let m = MotionState::new();
    // nome do param → (rótulo → quem o pinta)
    let mut tabela: BTreeMap<&str, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for nome in grupo {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let man = {
            use ph2d_nodegraph::cook::OpResolver;
            let Some(op) = m.registry.resolve(tid) else {
                continue;
            };
            op.manifest()
        };
        let hints = m.registry.param_ui(tid).unwrap_or(&[]);
        for spec in man.params {
            let rotulo = hints
                .iter()
                .find(|h| h.param == spec.name)
                .map_or("(sem hint)", |h| h.label);
            tabela
                .entry(spec.name)
                .or_default()
                .entry(rotulo.to_string())
                .or_default()
                .push((*nome).to_string());
        }
    }
    eprintln!("\n  param partilhado          | rótulo(s) | quem");
    eprintln!("  --------------------------|-----------|------");
    for (param, rotulos) in &tabela {
        let quantos: usize = rotulos.values().map(Vec::len).sum();
        if quantos < 2 {
            continue; // um nó só não tem com quem divergir
        }
        let marca = if rotulos.len() > 1 { "⚠️ " } else { "   " };
        for (rotulo, quem) in rotulos {
            eprintln!("{marca} {param:<24} | {rotulo:<9} | {}", quem.join(", "));
        }
    }
    eprintln!();
}

// ---------------------------------------------------------------------------------------------
// 4. O PREÇO — o relógio, a contagem e a coluna do dispositivo.
// ---------------------------------------------------------------------------------------------

pub(crate) fn wire(m: &mut MotionState, from: NodeId, fp: u16, to: NodeId, tp: u16) {
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
pub(crate) fn build_com(m: &mut MotionState, lado: f32, no: Option<&str>, aceso: bool) -> NodeId {
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
pub(crate) fn acordar(m: &mut MotionState, no: NodeId, nome: &str) {
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
pub(crate) const NO_REPS: usize = 3;
pub(crate) const BASE_REPS: usize = NO_REPS;

/// A mediana de 3 cozimentos **FRIOS** — um `MotionState` novo por corrida, senão o memo do cook
/// responde à segunda e a sonda mede a tabela de hash. Devolve `(ms, n, no_device)`.
/// O que uma linha da tabela diz.
pub(crate) struct Medida {
    /// A mediana de 3 cozimentos FRIOS, em ms — com o nó **acordado** (ver [`acordar`]).
    pub(crate) ms: f64,

    /// Quantos objectos saíram.
    pub(crate) n: usize,

    /// ⛔⛔ **O planeador reivindica a cadeia com o nó nos DEFAULTS dele?** É esta a pergunta
    /// do produto: é o que o artista recebe ao largar o nó.
    pub(crate) gpu_neutro: bool,

    /// E com o nó acordado. ⚠️ **Quando as duas diferem, a residência do nó DEPENDE de um
    /// knob** — não é ruído da sonda, é uma propriedade do nó que vale a pena ler.
    pub(crate) gpu_aceso: bool,
}

/// ⛔⛔ **DUAS PERGUNTAS, DOIS GRAFOS — e a primeira redacção respondia-as com um só.**
///
/// O relógio quer o nó a **trabalhar** ([`acordar`]); a coluna do dispositivo quer o nó como o
/// artista o **recebe**. Misturá-las custou uma leitura errada na primeira corrida: acordar o
/// `motion.look_at` pôs `target_x`/`target_y` fora de zero, que é exactamente o que a cláusula
/// `applicable` dele lê, e a coluna virou 🔴 — *a MINHA perturbação, lida como uma regressão do
/// produto*. ⚠️ E o comentário que eu tinha escrito ao lado do `acordar` dizia que só `Enum` e
/// `Toggle` alimentam um `applicable`: **falso**, e a tabela desmentiu-o na corrida seguinte.
pub(crate) fn cook_com(lado: f32, no: Option<&str>, repeticoes: usize) -> Medida {
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

/// ⭐⭐⭐ **O DESPERTAR TEM DE ACORDAR** — devolve os nós do grupo cuja saída ACESA é igual à
/// saída deles nos defaults. Um grupo sadio devolve a lista vazia.
///
/// ⚠️ **A comparação é do STREAM INTEIRO**, e não da coluna `P`: o `motion.rotate` e o
/// `motion.scale` escrevem `rot` e `size`, nunca `P`, e liam-se «não mudou» com o nó a girar.
/// A grelha é pequena de propósito (8×8): a pergunta é *«mudou?»*, não *«quanto custa?»*.
pub(crate) fn quem_o_despertar_nao_acorda(grupo: &[&'static str]) -> Vec<&'static str> {
    let mut mudos: Vec<&'static str> = Vec::new();
    let reg = MotionState::new().registry;
    for nome in grupo {
        // ⛔⛔ **UM NÓ QUE DECLARA PRECISAR DE OUTRA PORTA NÃO TEM FENÓMENO NESTA CADEIA.**
        //
        // A fixtura é `grid → <nó> → output` — uma porta só. O `field.combine` sem o segundo
        // campo e o `field.shape` sem a geometria são a **identidade** por construção, então
        // acusá-los de «o despertar não os acordou» seria a terceira leitura de uma mutação
        // sobrevivente: *a fixtura não produz o fenómeno*. ⚠️ E a lista é **declarada pelo nó**
        // (`register_required_inputs`, o mesmo canal que acende o ⚠️ no cartão), nunca escrita
        // aqui — um nó que passe a exigir uma porta amanhã sai desta conta sozinho.
        if reg
            .required_inputs(ph2d_nodegraph::node::NodeTypeId::of(nome))
            .is_some_and(|r| !r.is_empty())
        {
            continue;
        }
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
    mudos
}

/// ⭐⭐⭐ **A TABELA DE PREÇOS DO GRUPO** (doc 103 §1, passo 5).
///
/// ⚠️ **A coluna que se cita é a ABSOLUTA.** A razão vai ao lado e carrega o ruído do
/// denominador: a linha de base é o menor número da tabela e é o divisor de todas as outras.
/// ⛔ Dar-lhe mais amostras foi construído, medido e revertido — *o número de repetições faz
/// parte da medição*, então os dois lados de uma razão têm de o partilhar.
pub(crate) fn tabela(grupo: &[&str], lado: f32) {
    eprintln!(
        "\n  load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let so_grade = cook_com(lado, None, BASE_REPS);
    let nu = so_grade.ms;
    eprintln!(
        "\n  grade {lado}×{lado} = {} objectos · so\' a grade: {nu:.2} ms {}",
        so_grade.n,
        if so_grade.gpu_neutro { "🟢" } else { "🔴" }
    );
    eprintln!(
        "\n  ⚠️ o nó é medido ACORDADO (cada controlo contínuo fora do neutro) — nos defaults,
     metade de um grupo é a identidade e o relógio media um `clone`.\n"
    );
    eprintln!(
        "  nó                        | objectos  | cozimento  | vs. so' a grade | no device?"
    );
    eprintln!(
        "  --------------------------|-----------|------------|-----------------|------------------"
    );
    let reg = MotionState::new().registry;
    for nome in grupo {
        // ⛔⛔ **UM NÓ QUE PRECISA DE OUTRA PORTA NÃO É MEDIDO, E DIZ-SE.**
        //
        // A cadeia é `grid → <nó> → output`, uma porta só. O `field.combine` sem o segundo campo
        // e o `field.shape` sem a geometria são a **identidade** — cronometrá-los aqui daria o
        // preço de um `clone`, que é **exactamente** o defeito que o despertar do ciclo 3 curou
        // um nível abaixo (a tabela dizia que o patch de Coons era barato e media um `clone`).
        // ⚠️ E alimentar a porta genericamente é pior: um campo real na porta do `field.shape`
        // dá-lhe um polígono de milhares de vértices, e o `O(N·M)` dele passa a medir a fixtura.
        // ⇒ **um traço é honesto; um número seria mentira.**
        if reg
            .required_inputs(ph2d_nodegraph::node::NodeTypeId::of(nome))
            .is_some_and(|r| !r.is_empty())
        {
            eprintln!(
                "  {nome:<26} |         — |          — |               — | ⚪ precisa de outra porta"
            );
            continue;
        }
        let d = cook_com(lado, Some(nome), NO_REPS);
        let device = match (d.gpu_neutro, d.gpu_aceso) {
            (true, true) => "🟢 sim".to_string(),
            (false, false) => "🔴 NAO".to_string(),
            (true, false) => "🟡 so\' nos defaults".to_string(),
            (false, true) => "🟡 so\' fora dos defaults".to_string(),
        };
        eprintln!(
            "  {nome:<26} | {:>9} | {:>7.2} ms | {:>14.2}× | {device}",
            d.n,
            d.ms,
            d.ms / nu.max(1e-9),
        );
    }
    eprintln!(
        "\n  🟢 = o planeador reivindica a cadeia inteira · 🔴 = ela cai na CPU
  ⚪ = NAO MEDIDO: o no' declara precisar de outra porta, e nesta cadeia ele e' a identidade
  🟡 = a residência DEPENDE de um knob (o `applicable` do kernel lê-o)
  (um quadro de 60 fps tem 16,67 ms)\n"
    );
}
