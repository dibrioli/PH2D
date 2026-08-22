//! **A CAÇA AOS KNOBS MORTOS** — a varredura genérica (Grupo W do handoff).
//!
//! > *"Coloque no fim da fila de implementação auditoria multiagêntica a busca de parâmetros
//! > mortos como esse."* — Enio, 2026-08-21, logo depois de o smoke do `=73` ter achado um.
//!
//! **Um knob morto é um controle que o painel PINTA e que não muda a imagem.** A linha já
//! encontrou quatro espécies distintas e elas não se acham com a mesma sonda — é por isso que
//! isto é uma varredura e não um `grep`:
//!
//! | espécie | caso medido | quem a acha |
//! |---|---|---|
//! | morto no braço DEFAULT | `field.remap::curve_offset` (`curve.map_or(t, …)` saía antes) | **esta sonda** (o contexto vazio é o nó recém-nascido) |
//! | inerte no MODO que o painel mostra | `curvature`/`steps` do mesmo nó | **esta sonda** (varre os índices de todo `ParamWidget::Enum`) |
//! | descartado a JUSANTE | `motion.color_array::offset` lido por `.first()` | **esta sonda** só quando muda a saída inteira; o caso por-elemento pede fio, não param |
//! | declarado e nunca LIDO | — | `knobs_declarados_nunca_lidos.py` (texto, não execução) |
//!
//! ## O método, e o que ele NÃO prova
//!
//! Para cada nó × cada param × cada contexto de modo: cozinhar duas vezes com valores
//! diferentes do param e comparar **todas as colunas de saída ao bit**. Idêntico em todo o
//! espaço varrido ⇒ **acusação**.
//!
//! ⚠️ **Acusação, nunca veredito** — e a distinção é a coisa mais importante deste arquivo.
//! *Inerte não é morto.* Um param legitimamente inerte noutro modo é exatamente o que o
//! `ParamGate` existe para esconder, e um audit que colapse os dois devolve uma lista de falsos
//! positivos do tamanho do catálogo. Por isso a saída separa:
//!
//! - **`MORTO`** — não mudou nada em NENHUM contexto varrido. É a acusação forte.
//! - **`SÓ-EM-MODO`** — mudou em algum contexto e não noutros. Só vira defeito **se o painel
//!   pinta o knob no modo em que ele é inerte**, e é o `ParamGate` que responde isso: a coluna
//!   `gate` diz se existe um. Sem gate ⇒ suspeita de painel.
//! - **`VIVO`** — mudou no contexto vazio (o estado em que o nó nasce).
//!
//! ⚠️ **E o espaço varrido não é o espaço todo.** Os contextos são *um-de-cada-vez*: o vazio,
//! mais cada índice de cada param de enum com os outros no default. Um param que só acorda com
//! DOIS enums fora do default aparece aqui como `MORTO` e **não é** — é por isso que a saída é
//! uma lista para verificar à mão, com o contexto impresso ao lado.
//!
//! ⚠️ **Um nó que esta sonda não consegue montar sai como `SEM-BANCADA`, não como limpo.** Um
//! nó ausente da lista de acusações porque a sonda não soube alimentá-lo seria a mesma mentira
//! que um gate verde por não ter corrido — a contagem final imprime as duas populações.
//!
//! Correr:
//! ```text
//! cargo test -p ph2d-node-registry-init --test dead_knob_sweep --release -- --ignored --nocapture
//! ```

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget};
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;
use ph2d_nodegraph::port::PortType;
use ph2d_nodegraph::value::CookValue;

/// Quantos quadros correr antes de ler a saída.
///
/// ⚠️ **Não é 1 de propósito.** Um param de simulação (`damping`, `stiffness`) não muda nada no
/// primeiro tique — o estado ainda é a pose inicial. Uma sonda de um quadro acusaria de morta
/// metade da pilha de simulação.
const TICKS: usize = 12;
const DT: f64 = 1.0 / 60.0;

/// Os alimentadores preferidos, por ordem.
///
/// ⚠️ **A primeira versão desta sonda alimentava toda porta VALUE com o `debug.const` — o único
/// nó-fonte registado cujo tipo bate — e ele emite `Stream::new(1).with("v", [1.0])`.** Isso
/// acusou de MORTO o `strength` do `value.gain`, que é vivíssimo: **1.0 é ponto fixo de toda
/// curva de ganho**, e uma fixture parada no ponto fixo da transformação não distingue knob
/// nenhum. *Uma fixture que não contém o fenômeno não prova a ausência dele* — ela só prova
/// que a fixture é fraca. É por isso que o alimentador passou a ser uma CADEIA (abaixo).
const PREFERRED: &[&str] = &[
    "motion.grid",
    "motion.emitter",
    "value.instance_field",
    "source.shape",
    "debug.wave",
    "debug.const",
];

/// Até que profundidade encadear nós para produzir o tipo de uma porta.
///
/// Nenhuma fonte registada emite um VALOR que VARIE — o `debug.const` é constante e o
/// `value.instance_field` precisa de um stream para saber quantos elementos emitir. A cadeia
/// `motion.grid → value.instance_field` produz `n` valores distintos, que é a fixture que a
/// varredura precisa. Dois saltos chegam para todo tipo do catálogo; mais do que isso começaria
/// a montar cenas em vez de bancadas.
const MAX_CHAIN: usize = 2;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn catalogue(reg: &NodeRegistry) -> Vec<&'static NodeManifest> {
    let mut v: Vec<&'static NodeManifest> = reg.manifests().collect();
    // Os preferidos primeiro, o resto por nome — a ordem É a política de alimentação.
    v.sort_by_key(|m| {
        (
            PREFERRED
                .iter()
                .position(|p| *p == m.name)
                .unwrap_or(usize::MAX),
            m.name,
        )
    });
    v
}

/// Monta, dentro de `g`, uma sub-árvore que PRODUZ o tipo `want`, e devolve a porta de saída.
///
/// ⚠️ **É recursiva de propósito.** Nenhuma fonte registada emite um VALOR que varie, então
/// alimentar `value.*` com uma fonte direta é alimentar com uma constante — o erro que a
/// primeira versão cometeu. A cadeia `motion.grid → value.instance_field` produz `n` valores
/// distintos porque o segundo nó lê o comprimento do primeiro.
///
/// ⚠️ Uma tentativa falhada deixa nós órfãos no grafo. É inofensivo: o cook é *pull*, e um nó
/// sem consumidor nunca é cozido — mas é a razão de esta função não poder ser usada para medir
/// contagem de nós.
fn feed(
    g: &mut Graph,
    all: &[&'static NodeManifest],
    want: PortType,
    depth: usize,
    used: &mut Vec<&'static str>,
) -> Option<(NodeId, u16)> {
    for m in all {
        let Some(op) = m.outputs.iter().position(|o| o.ty.connects_directly(want)) else {
            continue;
        };
        if m.inputs.is_empty() {
            used.push(m.name);
            return Some((g.add_node(m.name), op as u16));
        }
        if depth == 0 {
            continue;
        }
        let node = g.add_node(m.name);
        let mut ok = true;
        for (i, p) in m.inputs.iter().enumerate() {
            match feed(g, all, p.ty, depth - 1, used) {
                Some((s, sp)) => {
                    if g.connect(Edge {
                        from: (s, sp),
                        to: (node, i as u16),
                        delayed: false,
                    })
                    .is_err()
                    {
                        ok = false;
                        break;
                    }
                }
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            used.push(m.name);
            return Some((node, op as u16));
        }
    }
    None
}

/// Uma bancada para o nó. `self_loop` liga a saída dele à porta cujo tipo é o da própria saída,
/// com atraso — é o que um nó de ESTADO precisa (`motion.integrate`, a pilha `sim.*`), e sem
/// isso o acumulador nunca acumula e todo param de simulação lê como morto.
fn bench(
    all: &[&'static NodeManifest],
    m: &'static NodeManifest,
    self_loop: bool,
) -> Option<(Graph, NodeId, String)> {
    let mut g = Graph::new();
    let n = g.add_node(m.name);
    let out_ty = m.outputs.first().map(|o| o.ty);
    let mut used: Vec<&'static str> = Vec::new();
    for (i, p) in m.inputs.iter().enumerate() {
        let looped = self_loop && out_ty.is_some_and(|t| t.connects_directly(p.ty));
        if looped {
            g.connect(Edge {
                from: (n, 0),
                to: (n, i as u16),
                delayed: true,
            })
            .ok()?;
            used.push("<self>");
            continue;
        }
        let (s, sp) = feed(&mut g, all, p.ty, MAX_CHAIN, &mut used)?;
        g.connect(Edge {
            from: (s, sp),
            to: (n, i as u16),
            delayed: false,
        })
        .ok()?;
    }
    used.dedup();
    Some((g, n, used.join("+")))
}

/// Os bits de uma coluna — comparação ao BIT, não por `==`.
///
/// ⚠️ **`f32::NaN != f32::NaN`**, então um `PartialEq` cru diria *"mudou"* de duas saídas
/// idênticas que contivessem um NaN, e a sonda acusaria de VIVO um knob morto num nó que
/// produz NaN. Os bits não têm essa opinião.
fn bits(c: &Column) -> Vec<u32> {
    match c {
        Column::Scalar(v) => v.iter().map(|x| x.to_bits()).collect(),
        Column::Vec2(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
        Column::Vec3(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
        Column::Vec4(v) => v.iter().flatten().map(|x| x.to_bits()).collect(),
    }
}

/// A saída do nó depois de `TICKS` quadros, como pares (coluna, bits).
fn snapshot(g: &Graph, reg: &NodeRegistry, n: NodeId) -> Option<Vec<(String, Vec<u32>)>> {
    let mut cook = Cook::new();
    let mut last = None;
    for k in 0..TICKS {
        let t = k as f64 * DT;
        let out = cook.cook(g, reg, n, t).ok()?;
        if let Some(CookValue::Instances(s)) = out.first() {
            let mut cols: Vec<(String, Vec<u32>)> =
                s.columns().map(|(k, v)| (k.clone(), bits(v))).collect();
            cols.sort_by(|a, b| a.0.cmp(&b.0));
            last = Some(cols);
        }
        cook.advance_tick(g, reg, t).ok()?;
    }
    last.filter(|c| !c.is_empty())
}

/// Os valores com que sondar um param: os EXTREMOS que a UI permite, mais o meio.
fn probe_values(hint: Option<&ParamUiHint>, default: f32) -> Vec<f32> {
    match hint {
        Some(h) => match h.widget {
            // Um enum tem um espaço FINITO e exato — varre-o inteiro.
            ParamWidget::Enum { labels } => (0..labels.len()).map(|i| i as f32).collect(),
            ParamWidget::Toggle => vec![0.0, 1.0],
            // ⚠️ O meio entra porque min/max podem ser SIMÉTRICOS num nó que só lê o
            // módulo: `-1` e `+1` dariam a mesma saída e o knob passaria por morto.
            _ => vec![h.min, (h.min + h.max) * 0.5, h.max],
        },
        // Sem hint o painel pinta um slider genérico; sem faixa declarada, chuta em volta
        // do default — e a chutada tem de mudar de ORDEM DE GRANDEZA, senão um param cujo
        // efeito é sub-pixel passa por morto.
        None => vec![default, default + 1.0, default * 8.0 + 3.0],
    }
}

/// Os contextos de modo: o vazio (o nó recém-nascido), cada índice de cada enum/toggle, e um
/// contexto com TODA magnitude fora do neutro.
///
/// ⚠️ **O último é o que faz o `mode` do `value.gain` aparecer vivo, e ele custou a segunda
/// iteração desta sonda.** Um param de MODO só se distingue quando a grandeza que ele modula
/// está fora do neutro: com `strength = 0` (o default) toda curva de ganho é a identidade, e os
/// dois modos dão a MESMA saída. Varrer só os enums, com as magnitudes no default, acusa de
/// morto todo seletor de um nó cuja força nasce em zero — que é a convenção deste catálogo.
///
/// *A regra geral: um knob que só age MULTIPLICANDO outro nunca se vê com o outro no neutro.*
fn contexts(
    reg: &NodeRegistry,
    m: &'static NodeManifest,
) -> Vec<(String, Vec<(&'static str, f32)>)> {
    let mut out = vec![("default".to_string(), Vec::new())];
    let Some(hints) = reg.param_ui(m.id) else {
        return out;
    };
    let mut hot: Vec<(&'static str, f32)> = Vec::new();
    for h in hints {
        let values: Vec<f32> = match h.widget {
            ParamWidget::Enum { labels } => (0..labels.len()).map(|i| i as f32).collect(),
            ParamWidget::Toggle => vec![0.0, 1.0],
            _ => {
                // Uma magnitude fora do neutro: o ponto a 3/4 da faixa, e nunca o default.
                let v = h.min + (h.max - h.min) * 0.75;
                let d = m.param_default(h.param).unwrap_or(0.0);
                hot.push((h.param, if (v - d).abs() > 1e-6 { v } else { h.min }));
                continue;
            }
        };
        for v in values {
            out.push((format!("{}={v}", h.param), vec![(h.param, v)]));
        }
    }
    if !hot.is_empty() {
        out.push(("magnitudes-quentes".to_string(), hot));
    }
    out
}

/// **A VARREDURA.** Imprime uma linha por (nó, param) — e não afirma nada: o produto é uma
/// tabela de acusações para verificar à mão, que é o que o Grupo W pede.
#[test]
#[ignore = "sonda: cargo test -p ph2d-node-registry-init --test dead_knob_sweep --release -- --ignored --nocapture"]
fn hunt_the_dead_knobs() {
    let reg = registry();
    let all = catalogue(&reg);

    println!(
        "# {} nos no catalogo · cadeia de alimentacao ate {MAX_CHAIN} saltos",
        all.len()
    );
    println!("no\tparam\twidget\tstatus\tvivos/ctx\tgate?\tbancada\tdetalhe");

    let (mut vivo, mut morto, mut so_modo, mut sem_bancada, mut suspeita) = (0, 0, 0, 0, 0);
    for m in &all {
        // As DUAS bancadas: alimentada, e com a porta de estado em laço. Um knob só é morto
        // se nenhuma das duas o faz mexer — a direcção conservadora, que erra para MENOS
        // acusação.
        let mut benches: Vec<(Graph, NodeId, String)> = Vec::new();
        for self_loop in [false, true] {
            if let Some((g, n, chain)) = bench(&all, m, self_loop)
                && snapshot(&g, &reg, n).is_some()
            {
                benches.push((g, n, chain));
            }
        }
        if benches.is_empty() {
            sem_bancada += 1;
            println!(
                "{}\t—\t—\tSEM-BANCADA\t—\t—\t—\tnao monta ou nao coze isolado",
                m.name
            );
            continue;
        }
        if m.params.is_empty() {
            continue;
        }
        let hints = reg.param_ui(m.id).unwrap_or(&[]);
        let gates: Vec<&str> = reg
            .param_gates(m.id)
            .map(|gs| gs.iter().map(|g| g.param).collect())
            .unwrap_or_default();
        let ctxs = contexts(&reg, m);
        let chains: Vec<&str> = benches.iter().map(|(_, _, c)| c.as_str()).collect();
        let chain = chains.join(" | ");

        let mut rows: Vec<(String, &str, String, usize, usize, &str)> = Vec::new();
        for p in m.params {
            let hint = hints.iter().find(|h| h.param == p.name);
            let values = probe_values(hint, p.default);
            let widget = hint.map_or("—", |h| match h.widget {
                ParamWidget::Enum { .. } => "Enum",
                ParamWidget::Toggle => "Toggle",
                ParamWidget::IntSlider => "IntSlider",
                ParamWidget::Seed => "Seed",
                ParamWidget::Angle => "Angle",
                ParamWidget::Color { .. } => "Color",
                _ => "Slider",
            });

            let mut alive_ctx: Vec<&str> = Vec::new();
            for (label, fixed) in &ctxs {
                // ⚠️ Um contexto que fixe o PRÓPRIO param sondado não diz nada sobre ele.
                if fixed.iter().any(|(k, _)| *k == p.name) {
                    continue;
                }
                let mut differs = false;
                for (g0, n, _) in &benches {
                    let mut base: Option<Vec<(String, Vec<u32>)>> = None;
                    for v in &values {
                        let mut g = g0.clone();
                        for (k, fv) in fixed {
                            g.set_param(*n, *k, *fv);
                        }
                        g.set_param(*n, p.name, *v);
                        let Some(snap) = snapshot(&g, &reg, *n) else {
                            continue;
                        };
                        match &base {
                            None => base = Some(snap),
                            Some(b) => {
                                if *b != snap {
                                    differs = true;
                                    break;
                                }
                            }
                        }
                    }
                    if differs {
                        break;
                    }
                }
                if differs {
                    alive_ctx.push(label);
                }
            }

            let total = ctxs
                .iter()
                .filter(|(_, f)| !f.iter().any(|(k, _)| *k == p.name))
                .count();
            let detail = if alive_ctx.is_empty() {
                format!("valores {values:?} nao mudaram coluna nenhuma")
            } else {
                format!("vivo em: {}", alive_ctx.join(" "))
            };
            let status = if alive_ctx.is_empty() {
                "MORTO"
            } else if alive_ctx.contains(&"default") && alive_ctx.len() == total {
                "VIVO"
            } else if alive_ctx.contains(&"default") {
                "VIVO-PARCIAL"
            } else {
                "SO-EM-MODO"
            };
            rows.push((
                p.name.to_string(),
                widget,
                detail,
                alive_ctx.len(),
                total,
                status,
            ));
        }

        // ⚠️ **A regra que separa a acusação do artefacto.** Um nó em que TODO param lê morto
        // não é um nó com todos os knobs mortos — é uma bancada que não exprime o nó (um efeito
        // de raster cujo produto não são colunas, um nó que precisa de uma cena). Acusar ali
        // seria a lista de falsos positivos que o handoff manda evitar.
        let all_dead = rows.iter().all(|r| r.5 == "MORTO");
        for (param, widget, detail, n_alive, total, status) in rows {
            let st = if all_dead { "BANCADA-SUSPEITA" } else { status };
            match st {
                "MORTO" => morto += 1,
                "SO-EM-MODO" => so_modo += 1,
                "BANCADA-SUSPEITA" => suspeita += 1,
                _ => vivo += 1,
            }
            let gated = if gates.contains(&param.as_str()) {
                "gate"
            } else {
                "—"
            };
            println!(
                "{}\t{param}\t{widget}\t{st}\t{n_alive}/{total}\t{gated}\t{chain}\t{detail}",
                m.name
            );
        }
    }
    println!(
        "\n# RESUMO\tVIVO={vivo}\tSO-EM-MODO={so_modo}\tMORTO={morto}\tBANCADA-SUSPEITA={suspeita}\tnos SEM-BANCADA={sem_bancada}"
    );
    println!(
        "# ⚠️ MORTO e SO-EM-MODO sao ACUSACOES, nao vereditos — cada uma pede verificacao a' mao."
    );
    println!(
        "# ⚠️ BANCADA-SUSPEITA nao acusa nada: e' a sonda a dizer que nao soube exprimir o no'."
    );
}
