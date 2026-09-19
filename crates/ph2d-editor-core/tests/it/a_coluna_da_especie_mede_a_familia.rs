//! ⭐⭐⭐ **A COLUNA DE UMA ESCOLHA MEDE A FAMÍLIA, nunca a opção que está escolhida.**
//!
//! A coluna da espécie do [`VariantEditor`] era **`45 %` do que sobrava da linha**, e uma fracção
//! não sabe quantas letras a palavra tem. Medido em 2026-09-19, com o editor pintado à largura do
//! Inspector:
//!
//! | espécie | mede | orçamento antigo | saía |
//! |---|---:|---:|---|
//! | `Dictionary` | `63,62` | `114` na raiz · `27,82` num filho | — |
//! | `Integer` | `44,01` | `27,82` | `Inte…` |
//! | `Color` | `33,54` | `27,82` | **`C…`** |
//! | `None` | `33,14` | `27,82` | `No…` |
//! | `Float` | `30,72` | `27,82` | **`Fl…`** |
//! | `Text` | `26,72` | `27,82` | cabia |
//!
//! ⚠️⚠️ **E o censo de elisões do app via DOIS dos seis**, porque ele mede o que foi PINTADO — que
//! é a opção escolhida de cada linha. `Integer` e `None` vivem na mesma lista e nunca tinham sido
//! medidos por ninguém. *Quem dimensiona um chip de escolha mede a LISTA, não o item em mãos* — a
//! mesma lei que a barra da tira do Flip (18/09) e a coluna do transporte da Timeline (19/09).
//!
//! # ⛔ A CHAVE continua a ser uma fracção, e isso é a decisão
//!
//! A chave de um dicionário é dado do ARTISTA: ela não tem tamanho conhecido, logo o que se lhe
//! pode garantir é uma parte da linha, e ser elidida ali é a resposta certa. *O que não pode ser
//! uma fracção é a coluna que a CASA escreve.*

use ph2d_editor_core::text_elide::elisao::{self, Medido};
use ph2d_editor_core::widget::{
    VariantEditor, VariantKind, VariantValue, dropdown_chip_width_for, paint_variant_editor,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Theme, TypeToken};
use ph2d_vector::VectorScene;

/// A largura do corpo do Inspector de hoje, que é onde este editor vive.
const HOST_W: f32 = 268.0;
/// A altura de uma linha de formulário do Inspector.
const ROW_H: f32 = 24.0;

/// A árvore da vitrina: uma raiz `Dict` com um `Float`, um `Str` e uma `Color`.
///
/// ⚠️ **Os filhos têm CHAVE**, e é isso que aperta a linha deles — a raiz não tem, logo ela era a
/// única que cabia antes da cura. *Uma fixtura só com a raiz não continha o fenómeno.*
fn editor() -> VariantEditor {
    VariantEditor::new(
        ph2d_a11y::NodeId(500),
        "user_data",
        VariantValue::Dict(vec![
            ("hue_shift".to_string(), VariantValue::Float(0.25)),
            ("label".to_string(), VariantValue::Str("muzzle".to_string())),
            ("tint".to_string(), VariantValue::Color([200, 30, 30, 255])),
        ]),
    )
}

fn pintado() -> Vec<Medido> {
    let mut ts = TextSystem::without_system_fonts();
    let mut cena = VectorScene::new();
    let ed = editor();
    let linhas = ed.rows().len() as f32;
    elisao::medindo(|| {
        paint_variant_editor(
            &ed,
            Rect::new(0.0, 0.0, HOST_W, ROW_H * linhas),
            ROW_H,
            &mut cena,
            &mut ts,
            Theme::default(),
        );
    })
    .1
}

/// ⭐⭐⭐ **As espécies pintadas chegam INTEIRAS, e partilham UMA coluna.**
#[test]
fn nenhuma_especie_pintada_e_cortada_e_a_coluna_e_uma_so() {
    let medidos = pintado();
    let nomes: Vec<&'static str> = VariantKind::ALL.iter().map(|k| k.label()).collect();
    let especies: Vec<&Medido> = medidos
        .iter()
        .filter(|m| nomes.contains(&m.texto.as_str()))
        .collect();
    assert!(
        especies.len() >= 4,
        "o editor pintou {} espécie(s) — a fixtura tem quatro linhas, e com menos este gate mede \
         o vazio",
        especies.len()
    );
    let primeira = especies[0].largura;
    for m in &especies {
        assert!(
            m.coube(),
            "⛔ a espécie {:?} saiu {:?} num orçamento de {:.2} px",
            m.texto,
            m.pintado,
            m.largura
        );
        assert_eq!(
            m.largura, primeira,
            "⛔ a espécie {:?} recebeu um orçamento ({}) diferente do das irmãs ({primeira}) — as \
             linhas deste editor partilham UMA coluna, senão a fracção voltou",
            m.texto, m.largura
        );
    }
}

/// ⛔⛔ **E a coluna é a da FAMÍLIA INTEIRA — incluindo a espécie que esta árvore não mostra.**
///
/// Sem esta metade, medir só as quatro pintadas passaria: a mais larga delas é `Dictionary`, que
/// por acaso é também a mais larga das seis. ⇒ o gate compara contra a família **derivada do
/// `VariantKind::ALL`**, que é a lista que a caixa de escolha de cada linha oferece.
///
/// ⚠️ E ela é **TIGHT**: uma coluna com folga passaria o gate acima e roubaria lugar ao valor.
#[test]
fn a_coluna_e_a_da_familia_inteira_e_nao_tem_folga() {
    let medidos = pintado();
    let nomes: Vec<&'static str> = VariantKind::ALL.iter().map(|k| k.label()).collect();
    let col = medidos
        .iter()
        .find(|m| nomes.contains(&m.texto.as_str()))
        .expect("alguma espécie tem de ser pintada")
        .largura;

    let mut ts = TextSystem::without_system_fonts();
    let familia = ph2d_editor_core::paint::label_column_width(
        &mut ts,
        TypeToken::Base.px(),
        nomes.iter().copied(),
    );
    assert!(
        col >= familia,
        "a coluna dá {col:.2} px e a espécie mais larga da família mede {familia:.2} — a coluna \
         voltou a medir o item em mãos"
    );
    // ⛔ TIGHT: o que a coluna reserva é o invólucro do chip mais a família, e nada além disso.
    let alvo = dropdown_chip_width_for(familia, ROW_H);
    let col_chip = VariantEditor::kind_col_w(&mut ts, ROW_H);
    assert!(
        (col_chip - alvo).abs() < 0.01,
        "o chip reserva {col_chip:.2} px e a família + invólucro pedem {alvo:.2} — uma folga aqui \
         é paga pela coluna do VALOR, que é a que sobra"
    );
}
