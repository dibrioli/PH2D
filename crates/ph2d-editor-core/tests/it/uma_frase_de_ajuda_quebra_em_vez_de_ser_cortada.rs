//! ⛔⛔⛔ **A FRASE QUE O APP MOSTRA QUANDO NÃO HÁ NADA ESCOLHIDO SAÍA CORTADA A MEIO DA PALAVRA.**
//!
//! Achado pela varredura das elisões em 2026-09-18. Na coluna de `252 px` do Inspector:
//!
//! > `Select an entity in the Hierarchy to inspec…`
//!
//! e no painel de Tags, em `240 px`: `No tags yet. Press + New to make the fi…`.
//!
//! ⚠️ **É a primeira coisa que o artista lê ao abrir o app** — e *uma dica cortada a meio da palavra
//! é pior que nenhuma dica: ela ensina que o app está partido.*
//!
//! # A causa, e porque ela nasceu de uma cura
//!
//! O `paint_text` **elide para UMA linha desde 2026-09-06** (report do dono: *«a palavra passa para
//! baixo e some»*). Aquela cura estava certa para um RÓTULO de linha — e estas duas não são
//! rótulos, são **frases**. ⇒ elas passam pelo [`ph2d_editor_core::paint::paint_text_block`], que
//! é o pintor que existe PARA quebrar e que **devolve a altura** para quem empilha por baixo.
//!
//! # ⛔⛔ Por que esta régua não pode ser o censo de elisões
//!
//! Um texto que QUEBRA não passa pela lei da reticência, logo **não deixa registo nenhum** — e um
//! painel que simplesmente deixasse de pintar a frase ficaria igualmente mudo. *Zero lê-se como
//! aprovação*, e é por isso que a régua aqui é a **TINTA**: os glifos que a cena de facto recebeu.

use ph2d_editor_core::paint::{paint_text, paint_text_block, resolve};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// As duas frases do PRODUTO, com a largura que a varredura mediu em cada painel.
const FRASES: &[(&str, f32)] = &[
    (
        "Select an entity in the Hierarchy to inspect its properties.",
        252.0,
    ),
    ("No tags yet. Press + New to make the first one.", 240.0),
];

/// Quantos glifos a cena levou — a única régua que distingue a frase inteira da cortada.
///
/// ⚠️ **Não é `n_paths` nem `n_path_segments`:** o Vello encaminha texto por `draw_glyphs` e as duas
/// contagens dão **zero** para qualquer texto (lição já paga por três gates desta crate).
fn glifos(cena: &VectorScene) -> usize {
    cena.inner()
        .encoding()
        .resources
        .glyph_runs
        .iter()
        .map(|r| r.glyphs.end - r.glyphs.start)
        .sum()
}

fn cor() -> ph2d_vector::Color {
    resolve(ColorToken::Text3, Theme::default())
}

/// ⭐⭐⭐ **A frase INTEIRA chega à tinta, e ocupa mais de uma linha.**
#[test]
fn uma_frase_de_ajuda_quebra_e_nada_dela_se_perde() {
    let fonte = TypeToken::Sm.px();
    for &(frase, largura) in FRASES {
        let mut ts = TextSystem::without_system_fonts();

        // A referência: a mesma frase sem cerca nenhuma — todos os glifos dela.
        let mut solta = VectorScene::new();
        let uma_linha = paint_text_block(
            &mut ts,
            &mut solta,
            frase,
            0.0,
            0.0,
            fonte,
            f32::INFINITY,
            cor(),
        );

        let mut cena = VectorScene::new();
        let alta = paint_text_block(&mut ts, &mut cena, frase, 0.0, 0.0, fonte, largura, cor());

        assert_eq!(
            glifos(&cena),
            glifos(&solta),
            "a frase perdeu tinta em {largura} px: {frase:?}"
        );
        assert!(
            alta > uma_linha * 1.5,
            "ela devia QUEBRAR em {largura} px e saiu com {alta} px de altura (uma linha é \
             {uma_linha}): {frase:?}"
        );
    }
}

/// ⛔ **E o CONTROLO: o pintor de RÓTULO continua a cortá-la.**
///
/// Sem esta metade o gate acima passaria num mundo onde ninguém elide coisa nenhuma — e a cura de
/// 06/09 (*«a palavra que não cabe passa para baixo e some»*) seria apagada em silêncio. *Um gate
/// que passa com a lei vizinha desligada não afirma nada sobre a escolha entre as duas.*
#[test]
fn o_pintor_de_rotulo_continua_a_elidir_a_mesma_frase() {
    let fonte = TypeToken::Sm.px();
    for &(frase, largura) in FRASES {
        let mut ts = TextSystem::without_system_fonts();
        let mut inteira = VectorScene::new();
        paint_text_block(
            &mut ts,
            &mut inteira,
            frase,
            0.0,
            0.0,
            fonte,
            f32::INFINITY,
            cor(),
        );
        let mut cortada = VectorScene::new();
        paint_text(
            &mut ts,
            &mut cortada,
            frase,
            0.0,
            0.0,
            fonte,
            largura,
            cor(),
        );
        assert!(
            glifos(&cortada) < glifos(&inteira),
            "o `paint_text` devia elidir {frase:?} em {largura} px — se ele deixou de o fazer, a \
             escolha do pintor deixou de importar e este par de gates tem de ser refeito"
        );
    }
}
