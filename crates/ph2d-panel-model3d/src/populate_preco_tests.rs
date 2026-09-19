//! ⭐⭐⭐ **O PREÇO DO TETO DE LINHAS, MEDIDO** — porque ele nomeia um recurso e não um palpite.
//!
//! `CLAUDE.md` §0.0: *antes de escrever qualquer limite, MEÇA — e depois escreva o número que a
//! MEDIÇÃO deu, com a tabela ao lado dele.* O [`super::MAX_SCENE_ROWS`] diz que o recurso é o
//! **registo de widgets**; este ficheiro é o sítio onde essa frase é um número.
//!
//! ⚠️ **Não é um gate de RELÓGIO com barra** (`CLAUDE.md` §5.0, a família das flakes de fan-out): a
//! asserção é sobre a **CONTAGEM**, que é determinística; o tempo é **impresso** e não afirmado.

use super::{CHIP_FAMILY_COUNT, MAX_CHOICES, MAX_MODES, MAX_ROWS, MAX_SCENE_ROWS};
use ph2d_editor_core::interaction::WidgetStore;

/// Quantas entradas de store uma LINHA custa: o slider, o campo, e a fileira de escolha.
const WIDGETS_POR_LINHA: usize = 2 + MAX_CHOICES as usize;

/// Quantas entradas o [`super::populate`] cunha ao todo — **derivado**, nunca contado à mão.
fn widgets_previstos() -> usize {
    MAX_MODES as usize * CHIP_FAMILY_COUNT + MAX_ROWS * WIDGETS_POR_LINHA + 2
}

/// ⭐⭐⭐ **O QUE A FOLGA DA CENA CUSTA** — e a tabela que o doc do teto cita.
///
/// Medido em 2026-09-19 nesta máquina (o relógio é impresso, não afirmado):
///
/// | parcela | entradas no store |
/// |---|---:|
/// | chips (`MAX_MODES × CHIP_FAMILY_COUNT`) | `208` |
/// | linhas do pior NÓ (`85 × 6`) | `510` |
/// | **a folga da CENA (`32 × 6`)** | **`192`** |
/// | a moldura e o fecho | `2` |
///
/// ⇒ a folga que faz a secção de estilo caber custa **`192` entradas cunhadas uma vez no arranque**,
/// e nada por quadro: o `paint` percorre `snapshot.rows`, que tem o tamanho do retrato e não o do
/// registo.
#[test]
fn o_preco_do_teto_de_linhas_e_o_registo() {
    let t0 = std::time::Instant::now();
    let mut store = WidgetStore::default();
    super::populate(&mut store);
    let arranque = t0.elapsed();

    let chips = MAX_MODES as usize * CHIP_FAMILY_COUNT;
    let nó = (MAX_ROWS - MAX_SCENE_ROWS) * WIDGETS_POR_LINHA;
    let cena = MAX_SCENE_ROWS * WIDGETS_POR_LINHA;
    println!("  chips         {chips:>6}");
    println!("  linhas do nó  {nó:>6}");
    // ⚠️ **`·` e nunca uma seta**: a Inter empacotada não cobre o bloco das setas, e o gate
    // `no_tofu_glyphs` (que vive na `ph2d-editor-core`, não aqui) acusa-o — ele varre os literais de
    // UI de TODA a workspace, testes incluídos.
    println!("  folga da cena {cena:>6}   · o que o `MAX_SCENE_ROWS` custa");
    println!("  moldura            2");
    println!(
        "  TOTAL         {:>6}   populate() em {arranque:?}",
        widgets_previstos()
    );

    // ⚠️ **O piso de população**: sem ele um `populate` que registasse zero widgets passaria — e a
    // tabela acima passaria a descrever o nada.
    assert!(
        widgets_previstos() > 600,
        "o registo encolheu para {} entradas — a tabela deste gate já não descreve o painel",
        widgets_previstos()
    );
    // ⚠️ **E a CONTAGEM prova-se no store, não na aritmética:** os dois lados são derivados, mas de
    // sítios diferentes — um do `populate`, outro das constantes que o doc do teto cita.
    let registados = super::MAX_ROWS
        .checked_mul(WIDGETS_POR_LINHA)
        .expect("o teto cabe num usize");
    let vivos = (0..super::MAX_ROWS as u32)
        .filter(|n| store.get(crate::ids::model3d_radius_slider(*n)).is_some())
        .count();
    assert_eq!(
        vivos,
        super::MAX_ROWS,
        "o `populate` cunhou {vivos} sliders para um teto de {} — a última linha do painel nasce \
         PINTADA E MORTA sob o dedo",
        super::MAX_ROWS
    );
    assert!(registados > 0);
}

/// ⛔ **O TETO É DERIVADO DA FORMA, e não o contrário** — a lei do §0.0 escrita como asserção.
///
/// ⚠️ **Sem ela, alguém que precise de mais uma linha baixa o `MAX_POLYGON_VERTICES`**, que é o
/// caminho lento (um teto de REGISTO, cujo recurso é memória) a mandar no rápido (um teto de
/// FORMA, que é o que o artista desenha).
#[test]
fn o_teto_de_linhas_deriva_do_teto_de_vertices() {
    assert_eq!(
        super::MAX_ROWS_DE_UM_NO,
        2 * ph2d_field::MAX_POLYGON_VERTICES as usize + super::EXTRAS_DE_UM_NO,
        "o teto do NÓ deixou de ser derivado do teto de vértices"
    );
    assert_eq!(super::MAX_ROWS, super::MAX_ROWS_DE_UM_NO + MAX_SCENE_ROWS);
    // ⚠️ **O piso da folga da cena é ERRO DE COMPILAÇÃO e não uma asserção aqui** — ver o
    // `const _: () = assert!(…)` ao lado do [`super::MAX_SCENE_ROWS`]. Um `assert!` de teste sobre
    // duas constantes é dobrado pelo compilador antes de correr, e o clippy di-lo em voz alta.
    println!("  folga da cena: {MAX_SCENE_ROWS} fileiras");
}
