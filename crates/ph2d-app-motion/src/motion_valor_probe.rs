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
            m.doc
                .graph
                .drive_param(alvo, "amount", (v, 0))
                .expect("dirige");
        }
        let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, out);
        eprintln!(
            "  {rotulo:<38} | {}",
            if plano.is_fully_gpu() {
                "dispositivo"
            } else {
                "⛔ CPU"
            }
        );
    };
    eprintln!("\n  cadeia `grid -> scale -> output`        | onde corre");
    eprintln!("  ---------------------------------------|------------");
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

/// ⭐⭐⭐ **SONDA — O PREÇO de dirigir um param** (ciclo 6, passo 5 · §0.0).
///
/// ⚠️ **Em RELEASE e com a máquina calma** — e o `/proc/loadavg` sai na primeira linha, porque
/// *uma tabela de relógio sem a carga ao lado não é uma medição, é um número* (`CLAUDE.md` §5.0).
/// A régua é o **mínimo** de N corridas: a mediana desta workstation carrega o ruído de fundo.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib --release probe_the_price_of_driving -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao — corra em RELEASE, a` mao"]
fn probe_the_price_of_driving_one_param() {
    use ph2d_nodegraph::graph::{Edge, NodeId};
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("\n  loadavg: {}", carga.trim());
    let lado: f32 = std::env::var("PH2D_LADO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(320.0);
    // ⛔⛔ **A ÁRVORE É RECONSTRUÍDA A CADA CORRIDA, e a 1.ª redacção desta sonda não o fazia.**
    // A cadeia é `Pure`: ela memoiza na primeira corrida, e as quatro seguintes custam ~zero. Tirar
    // o MÍNIMO de cinco media o memo e imprimia `0,00 ms` para 102 400 objectos nos dois lados.
    // *Uma régua que lê zero não está a medir a lei — está a medir a cache.*
    let mede = |dirigir: bool| -> (f64, bool) {
        let mut m = crate::motion_state::MotionState::new();
        let grid = m.doc.graph.add_node("motion.grid".to_string());
        m.doc.graph.set_param(grid, "rows", lado);
        m.doc.graph.set_param(grid, "cols", lado);
        let alvo = m.doc.graph.add_node("motion.scale".to_string());
        let out = m.doc.graph.add_node("motion.output".to_string());
        for (de, para) in [(grid, alvo), (alvo, out)] {
            m.doc
                .graph
                .connect(Edge {
                    from: (NodeId(de.0), 0),
                    to: (NodeId(para.0), 0),
                    delayed: false,
                })
                .expect("fio");
        }
        if dirigir {
            let v = m.doc.graph.add_node("value.number".to_string());
            m.doc
                .graph
                .drive_param(alvo, "amount", (v, 0))
                .expect("dirige");
        }
        let no_device =
            ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, out).is_fully_gpu();
        // Uma corrida FRIA por amostra: o `Cook` vive dentro do `m`, logo refazer o estado é o
        // que esvazia o memo. E a saída é LIDA, senão nada garante que o trabalho aconteceu.
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let mut fresco = crate::motion_state::MotionState::new();
            fresco.doc = m.doc.clone();
            let t = std::time::Instant::now();
            let saida = fresco
                .pump
                .cook
                .cook(&fresco.doc.graph, &fresco.registry, out, 0.0);
            let n = saida.map_or(0, |v| v[0].as_stream().count());
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            assert!(
                n > 0,
                "a cadeia nao produziu peca nenhuma -- a sonda mede nada"
            );
            melhor = melhor.min(ms);
        }
        (melhor, no_device)
    };
    let (livre, d1) = mede(false);
    let (dirigido, d2) = mede(true);
    #[expect(clippy::cast_possible_truncation, reason = "uma contagem de objectos")]
    let n = (lado * lado) as u32;
    eprintln!("  {n} objectos, `grid -> scale -> output`");
    eprintln!(
        "  sem dirigir nada       | {livre:>8.2} ms | {}",
        if d1 { "dispositivo" } else { "CPU" }
    );
    eprintln!(
        "  com UM param dirigido  | {dirigido:>8.2} ms | {}",
        if d2 { "dispositivo" } else { "CPU" }
    );
    eprintln!("  custo                  | {:>8.2}x\n", dirigido / livre);
}
