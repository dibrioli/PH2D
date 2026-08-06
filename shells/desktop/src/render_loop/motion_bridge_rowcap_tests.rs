//! **Todo param de todo nó CHEGA ao painel** — o censo do teto de linhas (doc 88, B3).
//!
//! Irmão do `range_tests` (a ESCALA de um valor) e do `unit_tests` (a UNIDADE dele): este mede
//! se o valor **aparece**. Um param acima do `MAX_PARAM_ROWS` não é desenhado nem registrado —
//! o `.take()` do `paint_rows` o descarta —, então ele existe no modelo, o cook o lê, e o
//! artista não tem gesto nenhum que o alcance. É a falha silenciosa que as quatro condições de
//! UI proíbem, e a única testemunha possível é um censo sobre o registry inteiro: nenhum gate
//! por-nó a veria, porque cada um usa a fixture do seu próprio nó.
//!
//! ⚠️ O teto é um recurso de verdade — o `populate` do painel registra **21 widgets por slot**
//! —, então ele não pode simplesmente sumir; o que ele pode é ser **medido** (§0). A sonda
//! abaixo imprime o censo; o gate o mantém honesto.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_panel_motion_params::MAX_PARAM_ROWS;

/// Quantas linhas de painel cada tipo de nó do registry produz, do maior para o menor.
///
/// ⚠️ Conta as linhas do SNAPSHOT, não os `ParamSpec` do manifesto: um nó emite também as
/// linhas de text param (Curve / Gradient / Palette / Text / Source / Channels) **antes** do
/// laço do manifesto, e é a soma que disputa os slots. Contar o manifesto responderia a outra
/// pergunta e reportaria um teto folgado demais.
fn row_census() -> Vec<(&'static str, usize)> {
    let mut motion = MotionState::new();
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    let mut census: Vec<(&'static str, usize)> = types
        .into_iter()
        .map(|ty| {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let n = build_params_snapshot(&motion, ProjectSettings::default())
                .map_or(0, |s| s.rows.len());
            (ty, n)
        })
        .collect();
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    census.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    census
}

/// **Nenhum param fica fora da tela.**
///
/// Nasceu VERMELHO contra o teto de 8 que shipava: o `field.remap` produz linhas acima dele, e
/// as excedentes eram descartadas em silêncio. A mutação que o prova é baixar
/// `MAX_PARAM_ROWS` de volta — o gate nomeia o nó e a contagem, em vez de dizer só "falhou".
#[test]
fn the_panel_shows_every_param_of_every_node() {
    let census = row_census();
    let over: Vec<String> = census
        .iter()
        .filter(|(_, n)| *n > MAX_PARAM_ROWS)
        .map(|(ty, n)| format!("{ty} ({n} linhas)"))
        .collect();
    assert!(
        over.is_empty(),
        "estes nós têm mais linhas que MAX_PARAM_ROWS ({MAX_PARAM_ROWS}), e o excedente é \
         descartado pelo `.take()` do paint_rows — o param existe e o artista não o alcança: \
         {over:?}"
    );
}

/// **E o teto não é folgado a ponto de não medir nada.**
///
/// A metade oposta, e ela não é cerimônia: sem isto, "conserte o gate acima" tem uma resposta
/// trivial e errada — pôr o teto em 256 e pagar 5376 registros de widget no `populate` por um
/// número que ninguém mediu. O teto é o pior caso medido mais folga de uma família; se o censo
/// cair muito abaixo dele, é sinal de que ele foi escolhido em vez de medido.
#[test]
fn the_row_cap_is_measured_not_guessed() {
    let census = row_census();
    let worst = census.first().copied().expect("o registry não é vazio");
    assert!(
        worst.1 <= MAX_PARAM_ROWS,
        "o pior nó ({} com {} linhas) não cabe no teto {MAX_PARAM_ROWS}",
        worst.0,
        worst.1
    );
    assert!(
        MAX_PARAM_ROWS <= worst.1 * 2,
        "o teto {MAX_PARAM_ROWS} é mais que o dobro do pior nó medido ({} com {} linhas) — \
         cada slot custa 21 registros de widget no populate, então isto é orçamento gasto \
         num número que ninguém mediu",
        worst.0,
        worst.1
    );
}

/// A SONDA: imprime o censo inteiro, para o número do teto sair de uma medição.
/// `cargo test -p ph2d-host-desktop measure_the_param_row_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_the_param_row_census() {
    let census = row_census();
    println!("\n=== LINHAS DE PAINEL POR NÓ (teto atual: {MAX_PARAM_ROWS}) ===");
    for (ty, n) in census.iter().take(20) {
        let flag = if *n > MAX_PARAM_ROWS { "  <-- CORTADO" } else { "" };
        println!("{n:3}  {ty}{flag}");
    }
    let over = census.iter().filter(|(_, n)| *n > MAX_PARAM_ROWS).count();
    println!(
        "--- {} tipos no total, {} acima do teto\n",
        census.len(),
        over
    );
}

