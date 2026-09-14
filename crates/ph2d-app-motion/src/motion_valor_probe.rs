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
