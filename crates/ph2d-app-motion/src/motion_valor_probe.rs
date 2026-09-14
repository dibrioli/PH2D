//! ⭐⭐ **A AUDITORIA DO GRUPO DO VALOR E DO PULSO** (ciclo 6, passo 2 — doc 103 §5).
//!
//! *«Um número que manda em tudo»*: os cinco ciclos anteriores puseram coisas na tela, fizeram-nas
//! andar, dobraram-nas, escolheram quem é afectado e entregaram a decisão a uma lei. Este grupo é
//! o **cérebro** — onde um número é feito, medido, transformado, e onde ele DISPARA.
//!
//! ⚠️ **O grupo é DUAS famílias e nenhuma lista escrita à mão**: `value.*` (o número) e `pulse.*`
//! (o instante). Uma lista aqui envelhecia em silêncio no dia em que um nó da família nascesse.
//!
//! ⚠️⚠️ **E é o maior grupo até hoje** — os cinco anteriores tiveram 10 · 8 · 13 · 7 · 12 nós; este
//! tem mais do que qualquer um deles, e o gate abaixo conta-o em vez de o afirmar.
//!
//! ```text
//! cargo test -p ph2d-app-motion --lib -- --ignored --nocapture audit_the_value_group
//! ```

/// O grupo inteiro — as DUAS famílias, pedidas ao registry.
///
/// ⚠️ **A ordem é `value.*` e depois `pulse.*`**, e não alfabética global: são duas perguntas
/// diferentes (*que número é este* · *quando é que isto acontece*), e o retrato lê-se por bloco.
pub fn grupo() -> Vec<&'static str> {
    let mut v = crate::motion_ciclo_probe::familia("value.");
    v.extend(crate::motion_ciclo_probe::familia("pulse."));
    v
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_value_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ DO GRUPO** — o que a auditoria compara contra as referências.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_value_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_value_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_value_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// **O VOCABULÁRIO do grupo** — dois nós que guardam a mesma pergunta chamam-lhe o mesmo nome?
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_value_vocabulary() {
    crate::motion_ciclo_probe::vocabulario(&grupo());
}

/// ⭐⭐⭐ **O VOCABULÁRIO PELO EIXO DO ARTISTA** — rótulo, e as palavras do enum (ciclo 6, W3).
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_vocabulary_the_artist_reads() {
    crate::motion_ciclo_probe::vocabulario_do_artista(&grupo());
}

/// ⚠️ **O MESMO, sobre o CATÁLOGO INTEIRO** — uma lei de vocabulário não é do grupo, é do app.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_vocabulary_of_the_whole_catalogue() {
    let m = crate::motion_state::MotionState::new();
    let todos: Vec<&'static str> = m.registry.manifests().map(|man| man.name).collect();
    crate::motion_ciclo_probe::vocabulario_do_artista(&todos);
}

/// ⚠️ **AS DUAS FAMÍLIAS ESTÃO VIVAS, e o grupo cresce com elas.**
///
/// ⛔ Sem este piso, um prefixo mal escrito ou um registo que mudasse de nome deixava as cinco
/// sondas acima a auditar **zero** nós — e todas passariam, caladas. É a mesma metade que o ciclo
/// 5 escreveu para a família `force.*`.
#[test]
fn both_value_families_are_derived_and_not_empty() {
    let valores = crate::motion_ciclo_probe::familia("value.");
    let pulsos = crate::motion_ciclo_probe::familia("pulse.");
    assert!(
        valores.len() >= 20,
        "so' {} no(s) `value.*` -- o prefixo ou o registo mudaram: {valores:?}",
        valores.len()
    );
    assert!(
        pulsos.len() >= 6,
        "so' {} no(s) `pulse.*` -- idem: {pulsos:?}",
        pulsos.len()
    );
    assert_eq!(
        grupo().len(),
        valores.len() + pulsos.len(),
        "o grupo tem de ser as DUAS familias inteiras"
    );
}

/// ⭐⭐⭐ **SONDA — UMA CADEIA COM VALOR FICA NO DISPOSITIVO?** (ciclo 6, passo 2 · lei 1 do §2).
///
/// A razão de existir de um `value.*` é **dirigir um param** de outro nó. O
/// [doc 102 §2](../../../docs/Motion%20Nodes/102_o_outro_patamar_plano_dos_nos_2026-09-04.md)
/// afirma que *«um param dirigido OU um pino ≠ 0 ligado derrubam o nó para a CPU»* — e uma
/// afirmação dessas decide o ciclo inteiro, logo mede-se antes de se escrever uma linha.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_does_a_value_chain_stay -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_does_a_value_chain_stay_on_the_device() {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let liga = |m: &mut crate::motion_state::MotionState, de: NodeId, para: NodeId| {
        m.doc
            .graph
            .connect(Edge {
                from: (de, 0),
                to: (para, 0),
                delayed: false,
            })
            .expect("fio");
    };
    let mede = |rotulo: &str, dirigir: Option<&'static str>| {
        let mut m = crate::motion_state::MotionState::new();
        let grid = m.doc.graph.add_node("motion.grid".to_string());
        let alvo = m.doc.graph.add_node("motion.scale".to_string());
        let out = m.doc.graph.add_node("motion.output".to_string());
        liga(&mut m, grid, alvo);
        liga(&mut m, alvo, out);
        if let Some(no) = dirigir {
            let v = m.doc.graph.add_node(no.to_string());
            // ⚠️ **O condutor é ALIMENTADO**, e sem isto a tabela mentiria sobre dois deles: o
            // `value.math` opera sobre uma corrente e o `pulse.beat` conta uma, logo desligados
            // não produzem número nenhum — e o nó fica na CPU pela lei do condutor VAZIO, que é
            // outra coisa do que uma recusa do planeador. *Duas causas que se leem iguais numa
            // coluna só.*
            let semente = m.doc.graph.add_node("value.number".to_string());
            let _ = m.doc.graph.connect(Edge {
                from: (semente, 0),
                to: (v, 0),
                delayed: false,
            });
            m.doc
                .graph
                .drive_param(alvo, "amount", (v, 0))
                .expect("dirige");
        }
        // ⭐ **Com os valores na mão** — a mesma porta que o `cook_gpu` usa. ⚠️ A `plan` sem mapa
        // continua a recusar, de propósito (ver `plan::DrivenParams`), então uma sonda que a
        // chamasse mediria a lei ANTIGA e diria que nada mudou.
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, out, &dirigidos);
        // ⚠️ **A terceira coluna é o que separa uma recusa de uma AUSÊNCIA.** Um condutor que não
        // produz número nenhum deixa o param a cair no default — e o nó fica na CPU *por essa*
        // razão, não porque o planeador o recusou. Sem esta coluna as duas leem-se iguais, e a
        // tabela ensinaria que a wave não funcionou naquele nó.
        let valor = dirigidos
            .values()
            .flat_map(std::collections::BTreeMap::values)
            .next()
            .map_or_else(
                || "—  (fio nenhum)".to_string(),
                |v| v.map_or_else(|| "vazio ⇒ cai no default".to_string(), |n| format!("{n}")),
            );
        eprintln!(
            "  {rotulo:<38} | {:<11} | {valor}",
            if plano.is_fully_gpu() {
                "dispositivo"
            } else {
                "⛔ CPU"
            }
        );
    };
    eprintln!("\n  cadeia `grid -> scale -> output`        | onde corre  | o valor que chegou");
    eprintln!("  ---------------------------------------|-------------|-------------------");
    mede("sem valor nenhum (o controlo)", None);
    for no in [
        "value.number",
        "value.lfo",
        "value.math",
        "value.time",
        "value.noise",
        "pulse.beat",
    ] {
        mede(&format!("com `{no}` a dirigir o `amount`"), Some(no));
    }
    eprintln!();
}

/// ⭐⭐⭐ **SONDA — os TRÊS que ficam fora do dispositivo, e o que cada um custa numa cadeia REAL**
/// (ciclo 6, W5 — doc 110 §11).
///
/// Depois da W2 o grupo está `32 de 35` no dispositivo. ⚠️ **Um `NAO` na coluna do retrato não é um
/// preço** — ele só custa alguma coisa se o nó estiver no caminho do OBJECTO, e a rota do param
/// dirigido é CPU **por desenho** (a W1a coze o condutor e entrega o NÚMERO). ⇒ esta sonda põe cada
/// um numa cadeia que chega mesmo ao stream de objectos e pergunta ao planeador onde é a costura.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_what_the_three_off_device -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_what_the_three_off_device_nodes_cost() {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let liga =
        |m: &mut crate::motion_state::MotionState, de: (NodeId, u16), para: (NodeId, u16)| {
            m.doc
                .graph
                .connect(Edge {
                    from: de,
                    to: para,
                    delayed: false,
                })
                .expect("fio");
        };
    let mede = |rotulo: &str, monta: &dyn Fn(&mut crate::motion_state::MotionState) -> NodeId| {
        let mut m = crate::motion_state::MotionState::new();
        let sink = monta(&mut m);
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, sink, &dirigidos);
        let costuras: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(no, porta)| {
                format!(
                    "{}:{porta}",
                    m.doc.graph.node(*no).map_or("?", |i| i.type_name.as_str())
                )
            })
            .collect();
        // ⚠️ **O número que decide é QUANTOS ELEMENTOS a costura carrega**, e não que ela exista:
        // uma fronteira de UM elemento é um `f32` por quadro; uma de `N` é o stream inteiro a
        // subir. *«Está fora do dispositivo» não é um preço enquanto ninguém contar o que sobe.*
        let elementos: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(no, porta)| {
                m.pump
                    .cook
                    .cook(&m.doc.graph, &m.registry, *no, 0.0)
                    .ok()
                    .and_then(|o| o.get(*porta).map(|v| v.as_stream().count()))
                    .map_or_else(|| "?".to_string(), |n| n.to_string())
            })
            .collect();
        eprintln!(
            "  {rotulo:<48} | {:>6} | {:<22} | {}",
            plano.stages.len(),
            if costuras.is_empty() {
                "—".to_string()
            } else {
                costuras.join(" · ")
            },
            if elementos.is_empty() {
                "— (nada sobe)".to_string()
            } else {
                elementos.join(" · ")
            }
        );
    };
    eprintln!(
        "\n  cadeia que chega ao STREAM DE OBJECTOS            | stages | costura                | elementos que SOBEM"
    );
    eprintln!(
        "  -------------------------------------------------|--------|------------------------|--------------------"
    );
    // O CONTROLO: a mesma forma, com um condutor que TEM kernel.
    mede("grid -> drive(v = value.lfo) -> output", &|m| {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        // ⚠️ **320 × 320**, a contagem das tabelas do doc 98 — a `3 × 3` do default esconde
        // exactamente a grandeza que esta sonda existe para medir.
        m.doc.graph.set_param(g, "rows", 320.0);
        m.doc.graph.set_param(g, "cols", 320.0);
        let d = m.doc.graph.add_node("motion.drive".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        let v = m.doc.graph.add_node("value.lfo".to_string());
        liga(m, (g, 0), (d, 0));
        liga(m, (g, 0), (v, 0));
        liga(m, (v, 0), (d, 1));
        liga(m, (d, 0), (o, 0));
        o
    });
    for no in ["value.number", "value.table", "value.cursor"] {
        mede(&format!("grid -> drive(v = {no}) -> output"), &|m| {
            let g = m.doc.graph.add_node("motion.grid".to_string());
            m.doc.graph.set_param(g, "rows", 320.0);
            m.doc.graph.set_param(g, "cols", 320.0);
            let d = m.doc.graph.add_node("motion.drive".to_string());
            let o = m.doc.graph.add_node("motion.output".to_string());
            let v = m.doc.graph.add_node(no.to_string());
            liga(m, (g, 0), (d, 0));
            // O `value.number` é gerador (zero entradas); os outros dois lêem a contagem.
            if m.doc
                .graph
                .node(v)
                .is_some_and(|i| i.type_name != "value.number")
            {
                liga(m, (g, 0), (v, 0));
            }
            liga(m, (v, 0), (d, 1));
            liga(m, (d, 0), (o, 0));
            o
        });
    }
    eprintln!();
}

/// ⛔⛔ **A SONDA DO PREÇO FOI APAGADA, e a razão fica escrita** (doc 110 §6).
///
/// Ela media `grid → scale → output` com e sem um fio, e imprimia uma razão. **Os dois lados dela
/// corriam o cozedor da CPU** (`pump.cook`) e o plano dela era o antigo — ela nunca tocou no
/// dispositivo. O que ela chamava *«o preço de dirigir um param»* era o custo de cozer o CONDUTOR,
/// que é a parte que esta wave não muda.
///
/// ⚠️ **E ela deu um número plausível** (`1,21×`), que é o que a tornava perigosa: uma régua que
/// devolve `0,00 ms` desconfia-se; uma que devolve `1,21×` cita-se. *Uma régua que mede outro
/// programa é pior que régua nenhuma.*
///
/// ⇒ O instrumento do preço passa a ser a **cena `=116`** (o app a correr, com
/// `PH2D_MOTION_ROUTE_LOG=1` a dizer a rota) e a paridade
/// `the_device_reads_the_driven_param_and_agrees_with_the_cpu`, que corre as DUAS rotas a sério.
/// Um A/B de relógio device-contra-CPU pede um `GpuContext` no arnês, e é wave própria.
const _PRECO: () = ();

/// ⭐⭐⭐ **SONDA — A FAIXA DO PULSO: quem consome um pulso, e chega algum deles ao dispositivo?**
/// (ciclo 6, W2 — doc 110 §5, item 2).
///
/// ⚠️ **Esta sonda corre ANTES de se escrever um kernel, e a razão é o §0.0:** a W2 diz *«a família
/// `pulse.*` no dispositivo»*, e um kernel só paga se o **consumidor** do pulso também lá estiver.
/// Um pulso não se desenha: ele ou dirige um param (que a W1a resolveu na CPU, de propósito) ou
/// entra numa **porta** de outro nó. *A pergunta não é «este nó tem kernel», é «esta FAIXA existe».*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_the_pulse_lane -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_the_pulse_lane() {
    let mut m = crate::motion_state::MotionState::new();
    // ⚠️ O tipo lê-se do GRAFO depois de o nó nascer (a mesma porta do `retrato`): um
    // `NodeTypeId::of` escrito à mão sobre um nome errado devolveria «sem kernel» em silêncio.
    let mut tem_kernel = |ty: &str| -> bool {
        use ph2d_nodegraph::gpu::KernelResolver;
        let id = m.doc.graph.add_node(ty.to_string());
        let tid = m.doc.graph.node(id).expect("no'").type_id();
        m.registry.gpu_kernel(tid).is_some()
    };
    eprintln!("\n  PRODUTORES (a familia `pulse.*`)");
    eprintln!("  no                        | kernel");
    eprintln!("  --------------------------|-------");
    for no in crate::motion_ciclo_probe::familia("pulse.") {
        eprintln!(
            "  {no:<25} | {}",
            if tem_kernel(no) { "sim" } else { "⛔ NAO" }
        );
    }
    eprintln!("\n  CONSUMIDORES (quem tem uma PORTA de pulso) -- e' aqui que a faixa acaba");
    eprintln!("  no                        | kernel");
    eprintln!("  --------------------------|-------");
    for no in ["motion.strobe", "motion.step", "sim.spawn", "util.reroute"] {
        eprintln!(
            "  {no:<25} | {}",
            if tem_kernel(no) { "sim" } else { "⛔ NAO" }
        );
    }
    eprintln!();
}

/// ⭐⭐⭐ **SONDA — ONDE A FRONTEIRA CAI numa cadeia de pulso de verdade** (ciclo 6, W2).
///
/// A sonda irmã diz quem TEM kernel. Esta diz o que isso custa: monta as três cadeias em que um
/// pulso chega de facto ao stream de objectos e pergunta ao planeador **onde é a costura**.
///
/// ⚠️ **Uma fronteira não é o mesmo que «tudo na CPU»** — o nó da fronteira é cozido na CPU e a
/// corrente dele é carregada na costura; o resto pode ficar no dispositivo. *Contar nós sem olhar
/// para o sítio da costura é a régua que mente aqui.*
///
/// ```text
/// cargo test -p ph2d-app-motion --lib probe_where_the_pulse_seam_falls -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_where_the_pulse_seam_falls() {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let mede = |rotulo: &str, monta: &dyn Fn(&mut crate::motion_state::MotionState) -> NodeId| {
        let mut m = crate::motion_state::MotionState::new();
        let sink = monta(&mut m);
        let dirigidos = crate::motion_bridge::gpu::valores_dirigidos(&mut m, 0.0);
        let plano =
            ph2d_gpu_cook::plan_driven(&m.doc.graph, &m.registry, &m.registry, sink, &dirigidos);
        let costuras: Vec<String> = plano
            .boundaries
            .iter()
            .map(|(no, porta)| {
                let ty = m
                    .doc
                    .graph
                    .node(*no)
                    .map_or("?", |i| i.type_name.as_str())
                    .to_string();
                format!("{ty}:{porta}")
            })
            .collect();
        eprintln!(
            "  {rotulo:<44} | {:>6} | {}",
            plano.stages.len(),
            if costuras.is_empty() {
                "— (tudo no dispositivo)".to_string()
            } else {
                costuras.join(" · ")
            }
        );
    };
    let liga =
        |m: &mut crate::motion_state::MotionState, de: (NodeId, u16), para: (NodeId, u16)| {
            m.doc
                .graph
                .connect(Edge {
                    from: de,
                    to: para,
                    delayed: false,
                })
                .expect("fio");
        };
    eprintln!("\n  cadeia                                       | stages | onde a CPU ainda coze");
    eprintln!("  ---------------------------------------------|--------|----------------------");
    // (1) O CONTROLO: a mesma cena sem pulso nenhum.
    mede("grid -> scale -> output (o controlo)", &|m| {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        let s = m.doc.graph.add_node("motion.scale".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        liga(m, (g, 0), (s, 0));
        liga(m, (s, 0), (o, 0));
        o
    });
    // (2) O ÚNICO consumidor de pulso COM kernel.
    mede("grid -> beat -> sim.spawn(pulse) -> output", &|m| {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        let b = m.doc.graph.add_node("pulse.beat".to_string());
        let sp = m.doc.graph.add_node("sim.spawn".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        liga(m, (g, 0), (b, 0));
        liga(m, (g, 0), (sp, 0));
        liga(m, (b, 0), (sp, 1));
        liga(m, (sp, 0), (o, 0));
        o
    });
    // (2-bis) **O CONTROLO da linha acima**: o mesmo `sim.spawn` com a porta de pulso SOLTA.
    // Sem ele, «o spawn caiu» lê-se como culpa do metrónomo — e pode ser do próprio spawn.
    mede("grid -> sim.spawn (porta de pulso SOLTA) -> output", &|m| {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        let sp = m.doc.graph.add_node("sim.spawn".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        liga(m, (g, 0), (sp, 0));
        liga(m, (sp, 0), (o, 0));
        o
    });
    // (3) O consumidor que o artista alcança primeiro — e que NÃO tem kernel.
    mede("grid -> threshold -> strobe(pulse) -> output", &|m| {
        let g = m.doc.graph.add_node("motion.grid".to_string());
        let t = m.doc.graph.add_node("pulse.threshold".to_string());
        let st = m.doc.graph.add_node("motion.strobe".to_string());
        let o = m.doc.graph.add_node("motion.output".to_string());
        liga(m, (g, 0), (t, 0));
        liga(m, (g, 0), (st, 0));
        liga(m, (t, 0), (st, 1));
        liga(m, (st, 0), (o, 0));
        o
    });
    eprintln!();
}
