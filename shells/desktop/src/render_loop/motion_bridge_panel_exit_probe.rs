//! ⭐⭐⭐ **O QUE O PAINEL LATERAL FAZ QUE O CARTÃO AINDA NÃO FAZ** — a medição que tem de vir
//! ANTES de o painel sair ([doc 103 §4](../../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md):
//! *«o painel lateral de params SAI»*, ordem do Enio de 2026-09-05).
//!
//! ⛔⛔ **Apagar uma superfície e levar junto um controlo sem substituto é a forma exacta do
//! knob INALCANÇÁVEL** (`CLAUDE.md §5.0`), e é o defeito que nenhuma sonda de registo apanha:
//! o controlo continua declarado, continua pintado, e simplesmente não abre. O gate do ciclo 1
//! (`no_param_the_panel_offers_falls_off_the_card`) responde *«o cartão MOSTRA todos?»*; esta
//! sonda responde a outra pergunta, que é a que decide a wave: ***o cartão ALCANÇA todos?***
//!
//! ⚠️ **A régua não é uma lista de espécies escrita aqui** — é a porta única do próprio gesto,
//! [`ph2d_panel_motion_graph::click_does`]. Uma espécie nova entra no produto e entra na conta
//! no mesmo dia; uma lista à mão deixaria a espécie nova invisível, que é o defeito que esta
//! sonda existe para não ter.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture what_the_card_still_cannot_reach
//! ```

use crate::motion_state::MotionState;
use ph2d_panel_motion_graph::{ClickDoes, click_does};
use std::collections::BTreeMap;

/// Como o PAINEL desenha este param — o que se perderia ao fechá-lo.
fn especie_no_painel(row: &ph2d_panel_motion_params::ParamRow) -> &'static str {
    use ph2d_panel_motion_params::ParamRow as R;
    match row {
        R::Scalar(_) => "número",
        R::Color(_) => "AMOSTRA + selector OKLCH",
        R::Toggle(_) => "interruptor",
        R::Enum(_) => "selector segmentado",
        R::Angle(_) => "ângulo",
        R::Seed(_) => "semente + re-sortear",
        R::Text(_) => "CAMPO DE TEXTO",
        R::Curve(_) => "EDITOR DE CURVA",
        R::Gradient(_) => "EDITOR DE GRADIENTE",
        R::Palette(_) => "EDITOR DE PALETA",
        R::Channels(_) => "selector de CANAL (+ texto)",
        R::Source(_) => "selector de FONTE publicada",
        R::File(_) => "caminho + diálogo de FICHEIRO",
    }
}

/// `(total de rows, por veredito, os inalcançáveis por espécie)`.
type Censo = (
    usize,
    BTreeMap<&'static str, usize>,
    BTreeMap<String, Vec<String>>,
);

fn censo() -> Censo {
    let todos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    let (mut total, mut veredito, mut presos): Censo = (0, BTreeMap::new(), BTreeMap::new());
    for nome in todos.iter().map(String::as_str) {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.to_string());
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        crate::render_loop::motion_bridge::params::card::stamp_card_params(
            &m,
            ph2d_editor::ProjectSettings::default(),
            &mut snap,
        );
        // O MESMO nó no painel — para a linha dizer o que se perde, e não só que se perde.
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let painel = crate::render_loop::motion_bridge::params::build_params_snapshot(
            &m,
            ph2d_editor::ProjectSettings::default(),
        );
        ph2d_panel_motion_graph::set_graph_selection(Vec::new());
        let Some(cartao) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        for p in &cartao.params {
            total += 1;
            let v = match click_does(p) {
                ClickDoes::Type => "escreve-se",
                ClickDoes::Toggle => "vira",
                ClickDoes::Cycle(_) => "avança",
                ClickDoes::Nothing => "NADA",
            };
            *veredito.entry(v).or_default() += 1;
            if v != "NADA" {
                continue;
            }
            // O que o painel faz com ele — a metade que diz o que há para construir.
            let no_painel = painel
                .as_ref()
                .and_then(|s| {
                    s.rows
                        .iter()
                        .find(|r| r.params().contains(&p.hint.param))
                        .map(especie_no_painel)
                })
                .unwrap_or("(o painel também não o mostra)");
            presos
                .entry(no_painel.to_string())
                .or_default()
                .push(format!("{nome}::{}", p.hint.param));
        }
    }
    (total, veredito, presos)
}

#[test]
#[ignore = "sonda de censo — corra à mão"]
fn what_the_card_still_cannot_reach() {
    let (total, veredito, presos) = censo();
    eprintln!("\n  {total} rows de cartão em todo o catálogo:");
    for (v, n) in &veredito {
        eprintln!("    {v:<12} {n:>4}");
    }
    let inalcancaveis: usize = presos.values().map(Vec::len).sum();
    eprintln!(
        "\n  ⛔ {inalcancaveis} controlos que o cartão PINTA e não abre — o que o painel faria:\n"
    );
    for (especie, quais) in &presos {
        eprintln!("  ── {especie} — {} controlo(s)", quais.len());
        for q in quais {
            eprintln!("       {q}");
        }
    }
    eprintln!();
}

/// ⛔⛔ **O PAINEL LATERAL NÃO PODE SAIR ENQUANTO ISTO NÃO FOR ZERO.**
///
/// A ordem do Enio (2026-09-05) é retirá-lo; esta é a conta que diz **quando**. Enquanto houver
/// um controlo que só o painel abre, fechá-lo torna-o inalcançável — e um controlo que se vê e
/// não se toca é pior que um ausente, porque o artista conclui que o app está avariado.
///
/// ⚠️ **Este gate falha de PROPÓSITO hoje?** Não: ele afirma o que é verdade agora — que a
/// contagem é conhecida e está ANOTADA. Ele quebra quando alguém acrescenta um editor rico novo
/// sem o alcançar no cartão (a conta sobe) **ou** quando a wave o cura (a conta desce e a
/// anotação fica a mentir). Nos dois casos o número aqui tem de ser reconciliado por MEDIÇÃO.
#[test]
fn the_side_panel_cannot_leave_while_the_card_cannot_open_these() {
    let (total, _, presos) = censo();
    let inalcancaveis: usize = presos.values().map(Vec::len).sum();
    assert!(total > 600, "controle: a varredura viu {total} rows");
    assert_eq!(
        inalcancaveis, TRANCADOS_NO_PAINEL,
        "a conta dos controlos que só o painel abre mudou ({inalcancaveis} contra \
         {TRANCADOS_NO_PAINEL}) — corra `what_the_card_still_cannot_reach` e reconcilie o \
         número com a MEDIÇÃO, nunca ao contrário"
    );
}

/// **Medido em 2026-09-06: `26` de `683` rows** — reconciliado pela sonda, nunca escrito de
/// memória. Em sete espécies, e a maior é o campo de texto (9):
///
/// | espécie | quantos | nós |
/// |---|---:|---|
/// | campo de TEXTO | 9 | `source.text` (×2) · `value.table` (×2) · `motion.expression` · `pulse.signal` · `rig.skeleton` · `value.pattern` · `motion.sub_uv` |
/// | amostra + selector de COR | 4 | `motion.tint` · `fx.glow` · `fx.drop_shadow` · `motion.strobe` |
/// | selector de FONTE publicada | 4 | `motion.path` · `motion.spline_wrap` · `fx.glow` · `source.object` |
/// | caminho + diálogo de FICHEIRO | 3 | `audio.bands` · `source.table` · `value.table` |
/// | editor de CURVA | 2 | `value.curve` · `motion.strobe` |
/// | editor de GRADIENTE | 2 | `motion.color_ramp` · `fx.glow` |
/// | editor de PALETA | 1 | `motion.color_array` |
/// | selector de CANAL | 1 | `value.attribute` |
///
/// ⚠️ **Os `4 + 3 + 4 = 11` de cor, ficheiro e fonte são os BARATOS:** quem abre o selector, o
/// diálogo e a lista de publicados é a **SHELL**, não o painel — o cartão só precisa de emitir a
/// mesma intenção. Os outros `15` pedem uma superfície de edição que hoje só o painel tem.
const TRANCADOS_NO_PAINEL: usize = 26;
