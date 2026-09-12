//! **Toda OPÇÃO de todo seletor CHEGA ao painel** — o censo do teto de opções.
//!
//! Irmão exato do `rowcap_tests`, e a mesma falha um nível abaixo: aquele mede se a *row*
//! aparece, este mede se as *opções dentro dela* aparecem. Acima do `MAX_ENUM_OPTIONS` o
//! `.min(…)` do `rows_paint_kinds` **não desenha nem registra** o segmento excedente — a
//! opção existe no modelo, o cook a lê, e o artista não tem gesto que a alcance. É a mesma
//! falha silenciosa que as quatro condições de UI proíbem, e a única testemunha possível é
//! um censo sobre o registry inteiro: nenhum gate por-nó a veria, porque cada um usa a
//! fixture do seu próprio nó.
//!
//! ⚠️ **Por que ele não existia, e o que ele achou na primeira corrida:** o teto de 8 nasceu
//! com a justificativa *"cobre os conjuntos de canal / onda / easing com folga"* — uma frase
//! sobre o que ele cobria naquele dia, não sobre de que RECURSO ele é (§0). Medido, o censo
//! nasceu **VERMELHO**: o `source.shape` declara **43** formas (o catálogo fillável inteiro,
//! append-only porque o índice é formato de arquivo) e o painel pintava **8** — trinta e
//! cinco formas cozinháveis e inalcançáveis, em silêncio. E **QUATRO** rows sentam exatamente
//! no teto (`value.unary` · `motion.stagger` · `field.combine` · `value.attribute`), então a
//! próxima opção que qualquer uma delas ganhar somia pelo mesmo caminho.
//!
//! ⚠️ **Numa row `Channels` o que disputa o teto é `channels.len() + 1`:** o "Custom…" é
//! acrescentado pelo PAINT, não guardado na lista. Contar só os canais responderia à
//! pergunta errada e o preço não seria um canal perdido — seria o ESCAPE de coluna
//! arbitrária, que é a razão de o `value.attribute` existir.

use super::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_editor::ProjectSettings;
use ph2d_panel_motion_params::{MAX_ENUM_OPTIONS, ParamRow};

/// Quantos SEGMENTOS a row pinta — o número que disputa o teto.
///
/// Só as duas famílias de seletor curado contam. Os chips de coluna VIVA de um picker em
/// Custom (`ChannelsRow::extra`) ficam de fora **de propósito**: eles são conveniência sobre
/// um campo de texto que o artista continua podendo digitar, então truncá-los degrada a
/// ajuda e não perde uma capacidade — e o comprimento deles é do GRAFO cozido, não do nó,
/// logo não é um número que um censo de registry possa afirmar.
fn segments(row: &ParamRow) -> usize {
    match row {
        ParamRow::Enum(r) => r.labels.len(),
        // +1 = o "Custom…" que o paint acrescenta.
        ParamRow::Channels(r) => r.channels.len() + 1,
        _ => 0,
    }
}

/// O seletor mais largo de cada tipo de nó do registry, do maior para o menor.
fn option_census() -> Vec<(&'static str, usize)> {
    let mut motion = MotionState::new();
    let types: Vec<&'static str> = motion.registry.manifests().map(|m| m.name).collect();
    let mut census: Vec<(&'static str, usize)> = types
        .into_iter()
        .map(|ty| {
            let node = motion.doc.graph.add_node(ty);
            ph2d_panel_motion_graph::set_graph_selection(vec![node.0]);
            let widest = build_params_snapshot(&motion, ProjectSettings::default())
                .map_or(0, |s| s.rows.iter().map(segments).max().unwrap_or(0));
            (ty, widest)
        })
        .collect();
    ph2d_panel_motion_graph::set_graph_selection(Vec::new());
    census.sort_by_key(|&(_, n)| std::cmp::Reverse(n));
    census
}

/// **Nenhuma opção fica fora da tela.**
///
/// A mutação que o prova é baixar `MAX_ENUM_OPTIONS` para 7: o gate nomeia o nó e a
/// contagem (`motion.stagger`, 8 opções), em vez de dizer só "falhou".
#[test]
fn the_panel_paints_every_option_of_every_selector() {
    let census = option_census();
    let over: Vec<String> = census
        .iter()
        .filter(|(_, n)| *n > MAX_ENUM_OPTIONS)
        .map(|(ty, n)| format!("{ty} ({n} opções)"))
        .collect();
    assert!(
        over.is_empty(),
        "estes nós têm um seletor com mais opções que MAX_ENUM_OPTIONS ({MAX_ENUM_OPTIONS}), \
         e o excedente é descartado pelo `.min()` do rows_paint_kinds — a opção existe e o \
         artista não a alcança: {over:?}"
    );
}

/// **E o teto não é folgado a ponto de não medir nada.**
///
/// A metade oposta, pelo mesmo motivo do irmão de linhas: sem ela, "conserte o gate acima"
/// tem uma resposta trivial e errada — pôr o teto em 64 e pagar `2 × 64 × MAX_PARAM_ROWS`
/// registros de botão no `populate` por um número que ninguém mediu. Os dois tetos se
/// MULTIPLICAM, que é o que torna a folga cara aqui.
#[test]
fn the_option_cap_is_measured_not_guessed() {
    let census = option_census();
    let worst = census.first().copied().expect("o registry não é vazio");
    assert!(
        worst.1 <= MAX_ENUM_OPTIONS,
        "o pior seletor ({} com {} opções) não cabe no teto {MAX_ENUM_OPTIONS}",
        worst.0,
        worst.1
    );
    assert!(
        MAX_ENUM_OPTIONS <= worst.1 * 2,
        "o teto {MAX_ENUM_OPTIONS} é mais que o dobro do pior seletor medido ({} com {} \
         opções) — cada opção custa 2 registros de botão por SLOT no populate, então isto é \
         orçamento gasto num número que ninguém mediu",
        worst.0,
        worst.1
    );
}

/// A SONDA: imprime o censo inteiro, para o número do teto sair de uma medição.
/// `cargo test -p ph2d-host-desktop measure_the_selector_option_census -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_the_selector_option_census() {
    let census = option_census();
    println!("\n=== OPÇÕES DO SELETOR MAIS LARGO POR NÓ (teto atual: {MAX_ENUM_OPTIONS}) ===");
    for (ty, n) in census.iter().take(20) {
        let flag = if *n > MAX_ENUM_OPTIONS {
            "  <-- CORTADO"
        } else if *n == MAX_ENUM_OPTIONS {
            "  <-- NO TETO"
        } else {
            ""
        };
        println!("{n:3}  {ty}{flag}");
    }
    let at = census
        .iter()
        .filter(|(_, n)| *n == MAX_ENUM_OPTIONS)
        .count();
    let over = census.iter().filter(|(_, n)| *n > MAX_ENUM_OPTIONS).count();
    println!(
        "--- {} tipos no total, {at} no teto, {over} acima\n",
        census.len()
    );
}
