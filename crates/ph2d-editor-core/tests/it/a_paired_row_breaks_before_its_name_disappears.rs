//! ⭐⭐⭐ **QUEM EMPARELHA DUAS PROPRIEDADES NUMA FILEIRA PERGUNTA ANTES SE O NOME AINDA CABE.**
//!
//! ⛔⛔ **Report do dono, 2026-09-15, com duas fotos (o painel largo e o mesmo painel estreito):**
//! *«Em Grain: Voronoi : Metric e Edges os nomes somem ao estreitar o painel. Melhor seria quebrar
//! a linha»*.
//!
//! `Metric` e `Edges` vivem **emparelhados**, cada um numa METADE da largura do painel. Ao
//! estreitar, a coluna do nome de uma metade fica menor do que a própria reticência e o
//! [`paint_property_label`] devolve **string vazia** — que é a resposta CERTA dele para uma coluna
//! degenerada (*«a caixa fica só com o número, que é o degrau seguinte da escada do estreito»*) e a
//! **errada** para quem escolheu emparelhar.
//!
//! ⇒ *o degrau a seguir a «não cabe o nome» não é apagar o nome: é deixar de emparelhar.*
//!
//! # ⚠️ O oráculo está FORA do predicado
//!
//! Comparar a [`property_row_fits`] com a `property_row_columns_for` seria comparar a função
//! consigo própria — o defeito que esta linha já pagou duas vezes. ⇒ a régua aqui é o **TEXTO
//! PINTADO**: quando o predicado diz *sim*, o rótulo que o pintor de facto põe na cena tem de ser o
//! nome **inteiro**, sem reticências e sem ficar vazio.

use ph2d_editor_core::property_row::property_row_fits;
use ph2d_editor_core::widget::property_label_origin;
use ph2d_text::TextSystem;

/// Nomes reais da secção que o dono fotografou, mais dois extremos.
const NOMES: &[&str] = &["Metric", "Edges", "Depth", "Contrast", "Randomness"];

/// A largura de uma METADE, ao longo do curso do dock (o painel menos o recuo do corpo, a meio).
fn metades() -> Vec<f32> {
    [
        220.0_f32, 240.0, 260.0, 280.0, 300.0, 320.0, 343.0, 369.7, 480.0, 720.0,
    ]
    .iter()
    .map(|painel| ((painel - 20.0 - 4.0) * 0.5).max(0.0))
    .collect()
}

#[test]
fn a_paired_row_breaks_before_its_name_disappears() {
    let mut ts = TextSystem::without_system_fonts();
    let font = ph2d_tokens::TypeToken::Sm.px();
    let mut mau = Vec::new();
    for nome in NOMES {
        let largura = ts.prefix_width(nome, font);
        for metade in metades() {
            if !property_row_fits(metade, largura) {
                continue; // disse que NÃO cabe — quem pergunta parte a fileira, e nada há a provar
            }
            // Disse que cabe. Então o pintor tem de pôr o nome INTEIRO.
            let row = ph2d_editor_core::widget::property_row_columns_for(
                0.0,
                metade,
                0.0,
                ph2d_tokens::ROW_H_PX,
                Some(largura),
                Some(ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX),
            );
            let (cabe, _, _) = property_label_origin(&mut ts, nome, row.label.x, font, row.label.w);
            if cabe != **nome {
                mau.push(format!(
                    "metade {metade:.1}: `property_row_fits` disse SIM para {nome:?} \
                     ({largura:.1} px) e o pintor poe {cabe:?}"
                ));
            }
        }
    }
    assert!(
        mau.is_empty(),
        "{} celula(s) emparelhariam com o nome cortado ou APAGADO — o report do dono de \
         2026-09-15:\n  {}",
        mau.len(),
        mau.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO** — sem ele, um `property_row_fits` que respondesse sempre `false` deixaria a
/// metade de cima verde sobre um painel que nunca emparelha.
///
/// ⚠️ **E a metade que importa é a de BAIXO:** o predicado tem de dizer **não** onde o dono viu o
/// nome sumir. Medido: `Metric` deixa de caber abaixo de um painel de **`~300 px`**, que é
/// exactamente a faixa das fotos dele.
#[test]
fn the_fit_predicate_says_yes_where_there_is_room_and_no_where_there_is_not() {
    let mut ts = TextSystem::without_system_fonts();
    let font = ph2d_tokens::TypeToken::Sm.px();
    let largura = ts.prefix_width("Metric", font);
    let metade = |painel: f32| ((painel - 20.0 - 4.0) * 0.5).max(0.0);
    assert!(
        property_row_fits(metade(480.0), largura),
        "num painel de 480 o par tem de caber"
    );
    assert!(
        !property_row_fits(metade(240.0), largura),
        "num painel de 240 o par NAO pode caber — foi ali que o dono viu o nome sumir"
    );
}
