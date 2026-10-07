use super::*;
use parley::PositionedLayoutItem;
use ph2d_board_model::{Mark, Marks};

fn plain(ts: &mut TextSystem, text: &str, size: f32, width: f32) -> Layout<Ink> {
    let mut lcx = LayoutContext::new();
    shape(ts, &mut lcx, text, &[], size, width, false)
}

#[test]
fn text_wraps_to_the_width_and_grows_in_height() {
    let mut ts = TextSystem::without_system_fonts();
    let one = plain(&mut ts, "uma ideia", 20.0, 1000.0);
    let narrow = plain(&mut ts, "uma ideia muito comprida que não cabe", 20.0, 80.0);
    assert_eq!(one.lines().count(), 1);
    assert!(narrow.lines().count() > 2, "não quebrou");
    assert!(narrow.height() > one.height() * 2.0);
    assert!(
        narrow.width() <= 80.0 + 0.5,
        "passou da largura: {}",
        narrow.width()
    );
}

#[test]
fn the_cache_reshapes_only_when_something_changed() {
    let mut ts = TextSystem::without_system_fonts();
    let mut c = TextCache::default();
    for _ in 0..10 {
        c.get(&mut ts, (1, 1), "olá", &[], 20.0, 100.0, false);
        c.end_frame();
    }
    assert_eq!(c.shaped(), 1, "um ecrã parado remoldou");
    c.get(&mut ts, (1, 1), "olá!", &[], 20.0, 100.0, false);
    c.get(&mut ts, (1, 1), "olá!", &[], 20.0, 120.0, false);
    assert_eq!(c.shaped(), 3);
    let mut bold = RichText::plain("olá!");
    bold.toggle(0..1, Mark::Bold);
    c.get_rich(&mut ts, (1, 1), &bold, 20.0, 120.0, false);
    assert_eq!(c.shaped(), 4, "um trecho novo é texto novo");
    for _ in 0..(KEEP_FRAMES * 2) {
        c.end_frame();
    }
    assert!(c.is_empty(), "o que ninguém usa fica para sempre");
}

/// ⭐ A MEDIDA de 06/10 que decidiu o desenho: a Inter da casa é variável — o negrito sai do eixo
/// `wght` (coordenadas normalizadas não nulas, que o desenho passa); itálico ela não tem, e o parley
/// pede uma inclinação sintética (que o desenho aplica por glifo).
#[test]
fn bold_is_a_real_weight_and_italic_a_skew() {
    let mut ts = TextSystem::without_system_fonts();
    let mut rich = RichText::plain("normal negrito italico");
    rich.toggle(7..14, Mark::Bold);
    rich.toggle(15..22, Mark::Italic);
    let mut lcx = LayoutContext::new();
    let l = shape(
        &mut ts,
        &mut lcx,
        rich.as_str(),
        rich.spans(),
        20.0,
        1000.0,
        false,
    );
    let mut seen = Vec::new();
    for line in l.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(g) = item {
                let r = g.run();
                seen.push((
                    r.text_range(),
                    r.normalized_coords().iter().any(|c| *c != 0),
                    r.synthesis().skew(),
                ));
            }
        }
    }
    let bold = seen
        .iter()
        .find(|s| s.0 == (7..14))
        .expect("um trecho a negrito");
    assert!(bold.1, "o negrito não mexeu no eixo do peso");
    let it = seen
        .iter()
        .find(|s| s.0 == (15..22))
        .expect("um trecho itálico");
    assert!(
        it.2.is_some_and(|deg| deg > 5.0),
        "sem inclinação: {:?}",
        it.2
    );
    let normal = seen.iter().find(|s| s.0 == (0..7)).expect("o simples");
    assert!(!normal.1 && normal.2.is_none());
}

#[test]
fn a_bold_word_is_wider_and_the_caret_follows_it() {
    let mut ts = TextSystem::without_system_fonts();
    let w_plain = plain(&mut ts, "mmmm", 20.0, 1000.0).width();
    let mut rich = RichText::plain("mmmm");
    rich.toggle(0..4, Mark::Bold);
    let mut e = TextEdit::new(&rich, 20.0, 1000.0);
    e.motion(&mut ts, Move::TextEnd, false);
    let (_, caret) = e.decorations(1.0);
    let x = caret.expect("cursor")[0] as f32;
    let w_bold = e.layout(&mut ts).width();
    assert!(
        w_bold > w_plain * 1.03,
        "o negrito não alargou: {w_plain} → {w_bold}"
    );
    // Centrado: o fim do texto está a meio + metade da largura do NEGRITO, não do simples.
    let expected = (1000.0 + w_bold) / 2.0;
    assert!(
        (x - expected).abs() < 1.5,
        "cursor em {x}, o fim do negrito em {expected}"
    );
}

#[test]
fn editing_starts_with_everything_selected_and_typing_replaces_it() {
    let mut ts = TextSystem::without_system_fonts();
    let mut e = TextEdit::new(&RichText::plain("antigo"), 20.0, 200.0);
    assert_eq!(e.selected().as_deref(), Some("antigo"));
    e.insert(&mut ts, "novo");
    assert_eq!(e.text(), "novo");
    e.insert(&mut ts, "\nlinha");
    e.motion(&mut ts, Move::TextStart, false);
    e.motion(&mut ts, Move::WordRight, true);
    assert_eq!(e.selected().as_deref(), Some("novo"));
    e.backspace(&mut ts, false);
    assert_eq!(e.text(), "\nlinha");
    let (_, caret) = e.decorations(1.0);
    assert!(caret.is_some(), "sem cursor");
}

#[test]
fn backspace_takes_one_character_and_ctrl_backspace_a_word() {
    let mut ts = TextSystem::without_system_fonts();
    let mut e = TextEdit::new(&RichText::plain("uma ideia só"), 20.0, 400.0);
    e.motion(&mut ts, Move::TextEnd, false);
    e.backspace(&mut ts, false);
    assert_eq!(e.text(), "uma ideia s", "o «ó» tem 2 bytes e sai inteiro");
    // A fronteira de palavra do parley (a mesma do `PlainEditor` de antes) leva o espaço também.
    e.backspace(&mut ts, true);
    assert_eq!(e.text(), "uma ideia");
    e.motion(&mut ts, Move::TextStart, false);
    e.delete(&mut ts, false);
    assert_eq!(e.text(), "ma ideia");
}

#[test]
fn ctrl_b_on_a_selection_bolds_it_and_without_one_bolds_what_comes_next() {
    let mut ts = TextSystem::without_system_fonts();
    let mut e = TextEdit::new(&RichText::plain("uma ideia"), 20.0, 400.0);
    e.select_range(&mut ts, 4..9);
    e.toggle(Mark::Bold);
    assert!(e.has(Mark::Bold));
    assert_eq!(e.rich().spans()[0].range(), 4..9);
    // Sem selecção, no fim: o negrito do «ideia» continua no que se escreve…
    e.motion(&mut ts, Move::TextEnd, false);
    assert!(e.has(Mark::Bold));
    // … até um Ctrl+B desligar para o que vem a seguir.
    e.toggle(Mark::Bold);
    assert!(!e.has(Mark::Bold));
    e.insert(&mut ts, " nova");
    assert_eq!(e.text(), "uma ideia nova");
    assert_eq!(
        e.rich().spans()[0].range(),
        4..9,
        "o « nova» entrou sem negrito"
    );
    assert_eq!(e.rich().marks_at(10), Marks::default());
}

#[test]
fn a_colour_paints_only_the_selection() {
    let mut ts = TextSystem::without_system_fonts();
    let red = Some(Rgba([200, 0, 0, 255]));
    let mut e = TextEdit::new(&RichText::plain("abc"), 20.0, 400.0);
    e.select_range(&mut ts, 1..2);
    e.set_color(red);
    assert_eq!(e.color(), Some(red));
    assert_eq!(e.rich().color(0..1), Some(None));
    e.select_all(&mut ts);
    assert_eq!(e.color(), None, "duas cores na selecção");
}

/// ⭐ O RASCUNHO escreve na letra à mão (a Virgil embutida — outra face, não a de reserva), a cache
/// remolda quando a forma troca de modo, e o editor de texto mede na MESMA letra que o desenho (o
/// cursor cai nas letras que se vêem).
#[test]
fn the_sketch_letters_are_the_hand_face_in_the_drawing_and_in_the_editor() {
    let mut ts = TextSystem::without_system_fonts();
    let text = RichText::plain("Worth it? ação");
    let mut lcx = LayoutContext::new();
    let plain_w = shape(&mut ts, &mut lcx, text.as_str(), &[], 20.0, 1000.0, false).width();
    let hand_w = shape(&mut ts, &mut lcx, text.as_str(), &[], 20.0, 1000.0, true).width();
    assert!(
        (plain_w - hand_w).abs() > 1.0,
        "a letra à mão mede o mesmo que a da interface: {plain_w} = {hand_w}"
    );
    let mut c = TextCache::default();
    c.get_rich(&mut ts, (1, 1), &text, 20.0, 1000.0, false);
    let n = c.shaped();
    c.get_rich(&mut ts, (1, 1), &text, 20.0, 1000.0, false);
    assert_eq!(c.shaped(), n, "sem mudança, sem remoldar");
    let w = c
        .get_rich(&mut ts, (1, 1), &text, 20.0, 1000.0, true)
        .width();
    assert_eq!(c.shaped(), n + 1, "trocar para rascunho remolda");
    assert_eq!(w, hand_w);
    let mut e = TextEdit::new(&text, 20.0, 1000.0).hand(true);
    assert_eq!(
        e.layout(&mut ts).width(),
        hand_w,
        "o editor mede na letra do desenho"
    );
}
