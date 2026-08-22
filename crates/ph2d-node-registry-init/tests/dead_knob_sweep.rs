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

mod common;

use common::*;
use ph2d_node_registry::ParamWidget;
use ph2d_nodegraph::graph::{Graph, NodeId};

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
        let benches: Vec<(Graph, NodeId, String)> = all_benches(&reg, &all, m)
            .into_iter()
            .filter(|(g, n, _)| snapshot(g, &reg, *n).is_some())
            .collect();
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
