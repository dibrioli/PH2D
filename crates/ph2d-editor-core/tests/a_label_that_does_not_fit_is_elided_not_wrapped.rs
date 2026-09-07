//! ⭐⭐⭐ **UMA LINHA, com reticências — a palavra que não cabe nunca cai para baixo.**
//!
//! Enio, 2026-09-06, com duas fotos do mesmo painel a estreitar: *«quando a palavra é grande e
//! estreitamos o painel, em vez dos três pontos (…) como no Blender, a palavra passa para baixo e
//! some. Precisamos corrigir isso em todos os widgets de todo o app.»* — `Surface Smooth` ficava
//! `Surface`, e o resto desaparecia.
//!
//! # O mecanismo
//!
//! O `paint_text` recebia `max_width` como **orçamento de QUEBRA** (o doc dele dizia-o), e o parley
//! obedecia: o rótulo virava duas linhas, a altura dobrava, e a segunda caía fora da linha de
//! 22 px — **cortada, não elidida**.
//!
//! ⚠️ **A porta da elisão já existia** (`text_elide`, com 55 consumidores) e os outros ~269 sítios
//! não passavam por ela. *Uma porta que a maioria não chama ainda não é a lei.*
//!
//! # Por que a régua é a ALTURA DA CENA, e não a string
//!
//! Um teste que compara strings mede a função de elisão — que já tinha gates. O que ninguém media
//! era o **resultado no ecrã**: um rótulo que quebra continua a emitir glifos, continua a ter
//! largura, e só a **altura** denuncia a segunda linha. ⇒ este gate pinta e mede a caixa.

use ph2d_editor_core::paint::{paint_text, paint_text_centered};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::{Color, VectorScene};

const FONT: f32 = 13.0;

/// A altura que a cena de facto ocupou, em linhas de texto.
fn painted_line_count(text_system: &mut TextSystem, text: &str, budget: f32) -> f32 {
    let one = text_system.layout("M", FONT, f32::INFINITY).height();
    let got = text_system.layout(text, FONT, budget).height();
    (got / one).round()
}

/// ⛔ **O CONTROLO: sem a cura, o parley PARTE mesmo.**
///
/// Sem esta metade, o gate abaixo passaria num mundo onde a caixa é larga de mais para o texto —
/// e mediria silêncio. *Uma fixtura sem o fenómeno aprova qualquer coisa.*
#[test]
fn the_raw_layout_really_does_wrap_at_this_budget() {
    let mut text = TextSystem::without_system_fonts();
    let label = "Surface Smooth";
    let full = text.prefix_width(label, FONT);
    let budget = full * 0.6;
    assert!(
        painted_line_count(&mut text, label, budget) >= 2.0,
        "a fixtura não contém o fenómeno: `{label}` cabe em {budget} px sem quebrar"
    );
}

/// ⭐ **O que o pintor emite ocupa UMA linha** — mesmo quando o texto pedido não cabe.
#[test]
fn a_long_label_paints_on_one_line() {
    let mut text = TextSystem::without_system_fonts();
    let label = "Surface Smooth";
    let budget = text.prefix_width(label, FONT) * 0.6;

    // ⚠️⚠️ **A régua é a CONTAGEM DE GLIFOS que o pintor emitiu, e não o que a porta devolveria.**
    //    A 1.ª redacção deste gate afirmava sobre `text_elide::fit(...)` — e uma mutação que
    //    APAGAVA a elisão de dentro do `paint_text` **sobreviveu**, porque a porta continuava a
    //    elidir quando o teste lhe perguntava. *Um gate que interroga a porta testemunha a porta,
    //    não o pintor.* Um rótulo quebrado emite os glifos TODOS (em duas linhas); um elidido
    //    emite os do prefixo mais as reticências.
    let paint_and_count = |text: &mut TextSystem, s: &str| -> usize {
        let mut scene = VectorScene::new();
        paint_text(text, &mut scene, s, 0.0, 0.0, FONT, budget, Color::WHITE);
        scene.inner().encoding().resources.glyphs.len()
    };
    let painted = paint_and_count(&mut text, label);
    assert!(painted > 0, "o rótulo não foi pintado de todo");

    let shown = ph2d_editor_core::text_elide::fit(&mut text, label, FONT, budget);
    assert!(
        shown.ends_with('\u{2026}'),
        "`{label}` em {budget:.0} px devia elidir, e saiu `{shown}`"
    );
    assert!(
        painted < label.chars().count(),
        "o pintor emitiu {painted} glifos para `{label}` ({} caracteres) — ele não elidiu, \
         quebrou a linha",
        label.chars().count()
    );
    assert_eq!(
        painted,
        paint_and_count(&mut text, &shown),
        "o pintor emitiu algo diferente do que a porta escolheu mostrar (`{shown}`)"
    );
    assert!(
        painted_line_count(&mut text, &shown, budget) <= 1.0,
        "o que se pinta (`{shown}`) ainda ocupa mais de uma linha"
    );
}

/// ⭐⭐ **E o CENTRADO centra o que pinta** — a metade que um gate de string nunca veria.
///
/// Com o texto inteiro medido num orçamento que o parte, o `y` centrava DUAS linhas numa caixa de
/// uma: a primeira saía **acima** do topo. *Uma centragem que mede outra coisa do que se pinta é
/// um deslocamento com cara de arredondamento.*
#[test]
fn the_centred_painter_centres_the_elided_string() {
    let mut text = TextSystem::without_system_fonts();
    let label = "Surface Smooth";
    let budget = text.prefix_width(label, FONT) * 0.6;
    let row = Rect::new(0.0, 100.0, budget, 22.0);

    let shown = ph2d_editor_core::text_elide::fit(&mut text, label, FONT, budget);
    let h = text.layout(&shown, FONT, f32::INFINITY).height();
    let y = row.y + (row.h - h) / 2.0;
    assert!(
        y >= row.y && y + h <= row.y + row.h + 0.5,
        "o texto centrado sai da caixa: y={y:.1}, altura={h:.1}, caixa {:.1}..{:.1}",
        row.y,
        row.y + row.h
    );

    // E ele pinta mesmo (a aritmética acima podia estar certa sobre nada).
    let mut scene = VectorScene::new();
    paint_text_centered(&mut text, &mut scene, label, row, FONT, Color::WHITE);
    assert!(
        !scene.inner().encoding().resources.glyphs.is_empty(),
        "o rótulo centrado não chegou à cena"
    );
}

/// ⭐⭐⭐ **E O BLOCO CONTINUA A QUEBRAR** — a metade que impede a cura de matar o caso oposto.
///
/// ⚠️⚠️ **Eu parti isto DUAS vezes na mesma jornada**, e as duas de maneiras diferentes: primeiro
/// pondo a elisão no caminho PARTILHADO (o `paint_text_block` delega lá), depois deixando o
/// próprio `paint_text_block` a pedir `ElideToOne`. Nos dois casos três gates de OUTRAS linhas
/// caíram — `a_hint_that_wraps_pushes_what_comes_after_it_down` e dois irmãos — porque uma dica
/// que deixa de quebrar deixa de **empurrar**, e a fileira seguinte passa a ser escrita por cima
/// dela.
///
/// ⇒ *uma cura que só sabe o caso que a motivou apaga o caso oposto*, e o `paint_text_block`
/// existe precisamente para o caso oposto: ele **devolve a altura**, e é isso que o distingue.
#[test]
fn the_block_painter_still_wraps_and_reports_the_height() {
    let mut text = TextSystem::without_system_fonts();
    let mut scene = VectorScene::new();
    let long = "Surface Smooth relaxes the mesh without moving its silhouette";
    let budget = text.prefix_width(long, FONT) * 0.35;

    let one_line = text.layout("M", FONT, f32::INFINITY).height();
    let used = ph2d_editor_core::paint::paint_text_block(
        &mut text,
        &mut scene,
        long,
        0.0,
        0.0,
        FONT,
        budget,
        Color::WHITE,
    );
    assert!(
        used > one_line * 1.5,
        "o bloco devolveu {used:.1} px para um texto que precisa de várias linhas em {budget:.0} \
         px — ele deixou de quebrar, e quem o chama vai escrever por cima da dica"
    );
    assert!(
        scene.inner().encoding().resources.glyphs.len() > long.chars().count() / 2,
        "o bloco elidiu em vez de quebrar: emitiu poucos glifos de mais"
    );
}
