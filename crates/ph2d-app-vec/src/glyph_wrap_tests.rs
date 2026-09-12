//! Gates do **REFLUXO** (W2a) — a porta [`super::wrapped_lines`] e o que ela promete.
//!
//! Irmão do [`super::tests`] pelo teto de 600 LOC (HR-18), e **FILHO** de `vec_glyph` como
//! ele (via `#[path]`): o `use super::*` alcança a `line_advance`, que é privada e é a régua
//! que esta wave existe para fazer coincidir.
//!
//! # O oráculo central mede a TINTA, não a régua
//!
//! A wave inteira é *"o quebrador mede com a mesma régua que o cozedor desenha"*. Um gate que
//! chamasse `line_advance` dos dois lados seria **duas cópias da mesma régua a concordar** —
//! verde por construção, e cego a uma régua que esteja errada nas duas pontas. Por isso o gate
//! que importa cozinha o texto e mede a **largura dos glifos produzidos**: se o quebrador e o
//! cozedor discordassem, a tinta passaria da caixa e nenhuma aritmética compartilhada o
//! esconderia.

use super::*;

/// Uma caixa em unidades de mundo, larga o bastante para caber várias palavras do fixture no
/// tamanho 1.0 — e estreita o bastante para o texto de teste ter de quebrar mais de uma vez.
const BOX: f64 = 6.0;

fn boxed(size: f64, w: Option<f64>) -> TextLayout {
    tracked(size, w, 0.0)
}

/// O mesmo layout com **tracking**, e é a fixture que dá dentes ao gate central.
///
/// ⚠️ Com `tracking: 0.0` um quebrador que IGNORASSE o tracking mede exactamente o mesmo que
/// o cozedor — as duas réguas concordam por acidente da fixture, e a mutação que as separa
/// sobrevive. Medido: ela passou a bater só depois de a fixture carregar tracking.
fn tracked(size: f64, w: Option<f64>, tracking: f64) -> TextLayout {
    TextLayout {
        size,
        line_height: 1.2,
        tracking,
        align: TextAlign::Left,
        wrap_width: w,
    }
}

fn font() -> VariableFont {
    VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida")
}

/// A largura da TINTA de um conjunto de glyph-paths (a extensão em x das âncoras e alças).
/// `0.0` quando nada foi desenhado.
fn ink_width(paths: &[VecPath]) -> f64 {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for p in paths {
        for v in p.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = lo.min(q[0]);
                hi = hi.max(q[0]);
            }
        }
    }
    if lo.is_finite() { hi - lo } else { 0.0 }
}

/// ⭐ **O gate central: a tinta cabe na caixa.**
///
/// Ele não sabe o que é `line_advance` — mede o que foi DESENHADO. É a diferença entre provar
/// que duas cópias de uma régua concordam e provar que o texto cabe.
#[test]
fn the_ink_of_a_wrapped_block_fits_the_box_it_was_given() {
    let f = font();
    let text = "the quick brown fox jumps over the lazy dog again and again";
    let paths = text_to_vec_paths(
        &f,
        text,
        // ⚠️ **Tracking != 0 de propósito** — ver [`tracked`]: é ele que torna as duas réguas
        // distinguíveis, e sem ele este gate fica verde sobre um quebrador que o ignora.
        &tracked(0.3, Some(BOX), 0.25),
        &[],
        &TextPlacement::At([0.0, 0.0]),
        &Some(Paint::solid(ph2d_vec_scene::Rgba8::new(0, 0, 0, 255))),
        &None,
    );
    assert!(!paths.is_empty(), "a fixture tem de conter o fenomeno");
    let w = ink_width(&paths);
    assert!(
        w <= BOX,
        "a tinta ({w:.4}) passou da caixa ({BOX:.4}) - o quebrador e o cozedor mediram \
         com reguas diferentes"
    );
    // ⚠️ E a metade que impede o gate de passar por vácuo: uma caixa só é uma caixa se o texto
    // de facto a encheu. Sem isto, um quebrador que pusesse UMA palavra por linha passaria.
    assert!(
        w > BOX * 0.5,
        "a fixture nao enche a caixa (tinta {w:.4} contra {BOX:.4}) - o gate ficaria verde \
         sobre um quebrador que poe uma palavra por linha"
    );
}

/// **Sem caixa, a porta é exactamente `split('\n')`** — o controle da wave inteira.
///
/// É isto que torna `wrap_width: None` byte-idêntico ao mundo que já shipava: não há caminho
/// novo a percorrer, há o mesmo iterador de sempre.
#[test]
fn no_box_is_exactly_the_lines_the_artist_typed() {
    let f = font();
    for text in [
        "",
        "one",
        "a\nb",
        "a\n\nb",
        "trailing \n",
        "  spaced  out  ",
    ] {
        let got = wrapped_lines(
            &f,
            text,
            &boxed(1.0, None),
            &[],
            &TextPlacement::At([0.0; 2]),
        );
        let want: Vec<&str> = text.split('\n').collect();
        assert_eq!(
            got, want,
            "sem caixa a porta tem de ser o split cru ({text:?})"
        );
    }
}

/// **Uma caixa mais larga que o texto não quebra nada** — a outra metade do controle: o
/// refluxo LIGADO num texto que cabe produz as mesmas linhas do refluxo desligado.
#[test]
fn a_box_wider_than_the_text_changes_nothing() {
    let f = font();
    let text = "short line\nand another";
    let off = wrapped_lines(
        &f,
        text,
        &boxed(0.3, None),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    let on = wrapped_lines(
        &f,
        text,
        &boxed(0.3, Some(1000.0)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    assert_eq!(off, on, "uma caixa folgada nao pode inventar quebras");
}

/// **Uma palavra maior que a caixa TRANSBORDA, e não é partida ao meio.**
///
/// A decisão está escrita na porta; este gate é o que impede alguém de a "melhorar" para um
/// hífen automático sem passar por uma decisão de produto.
#[test]
fn a_word_wider_than_the_box_overflows_whole() {
    let f = font();
    let long = "supercalifragilisticexpialidocious";
    let lines = wrapped_lines(
        &f,
        long,
        &boxed(1.0, Some(0.5)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    assert_eq!(
        lines,
        vec![long],
        "a palavra tem de sair INTEIRA, transbordando"
    );
}

/// **Um parágrafo vazio continua a ser uma linha.** Sem isto `"a\n\nb"` perderia a linha em
/// branco e o bloco subiria uma entrelinha — uma quebra que o artista escreveu, apagada.
#[test]
fn an_empty_paragraph_survives_the_reflow() {
    let f = font();
    let lines = wrapped_lines(
        &f,
        "a\n\nb",
        &boxed(0.3, Some(BOX)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    assert_eq!(
        lines.len(),
        3,
        "as tres linhas: 'a', vazia, 'b' -> {lines:?}"
    );
    assert_eq!(lines[1], "", "a do meio e' a linha em branco");
}

/// ⭐ **Um texto EM CAMINHO não reflui** — a decisão de desenho, prendida na PORTA.
///
/// A recusa mora em `wrapped_lines` e não nos sítios que montam o `TextLayout`: enumerar
/// construtores é como o próximo nasce sem a regra. A mutação que a remove (tirar o
/// `matches!(placement, At(_))`) deixa este gate VERMELHO e todos os outros verdes.
#[test]
fn a_text_riding_a_path_never_reflows() {
    let f = font();
    let text = "the quick brown fox jumps over the lazy dog again and again";
    let path = ArcPath::from_contour(
        &[
            ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([100.0, 0.0]),
        ],
        false,
    )
    .expect("reta");
    let on_path = TextPlacement::OnPath {
        path: &path,
        start_offset: 0.0,
        flip: false,
    };
    let ride = wrapped_lines(&f, text, &boxed(0.3, Some(BOX)), &[], &on_path);
    assert_eq!(
        ride,
        vec![text],
        "sobre um caminho a caixa tem de ser INERTE - quem diz por onde os glifos correm \
         e' a curva"
    );
    // O controle, no MESMO layout: reto, a mesma caixa quebra de facto. Sem esta metade o
    // gate passaria com um quebrador que nunca quebrasse.
    let straight = wrapped_lines(
        &f,
        text,
        &boxed(0.3, Some(BOX)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    assert!(
        straight.len() > 1,
        "o controle tem de quebrar ({straight:?}) - senao este gate nao prova nada"
    );
}

/// **A caixa é medida na mesma unidade que o TAMANHO** — dobrar o tamanho do glifo com a
/// mesma caixa produz mais linhas. Um quebrador que medisse em unidades de design (em vez de
/// mundo) daria o mesmo número de linhas nos dois, e nada mais o denunciaria.
#[test]
fn a_bigger_glyph_in_the_same_box_needs_more_lines() {
    let f = font();
    let text = "the quick brown fox jumps over the lazy dog";
    let small = wrapped_lines(
        &f,
        text,
        &boxed(0.25, Some(BOX)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    let big = wrapped_lines(
        &f,
        text,
        &boxed(0.5, Some(BOX)),
        &[],
        &TextPlacement::At([0.0; 2]),
    );
    assert!(
        big.len() > small.len(),
        "tamanho 0.5 tem de quebrar mais que 0.25 na mesma caixa ({} vs {})",
        big.len(),
        small.len()
    );
}

/// **Uma caixa não-positiva é *sem refluxo*, não uma caixa de zero.** É o estado que um slider
/// a meio de um arrasto produz, e uma caixa de zero poria cada palavra na sua linha.
#[test]
fn a_non_positive_box_is_no_box_at_all() {
    let f = font();
    let text = "one two three";
    for w in [0.0, -1.0] {
        let lines = wrapped_lines(
            &f,
            text,
            &boxed(0.3, Some(w)),
            &[],
            &TextPlacement::At([0.0; 2]),
        );
        assert_eq!(lines, vec![text], "caixa {w} tem de ser inerte");
    }
}

/// ⭐ **O QUE O AUTO LAYOUT GANHA: a caixa do texto passa a ser a REFLUÍDA.**
///
/// O `layout_live` mede um filho pela bbox dos `VecPath` dele, e o texto vivo entra na cena
/// como UM compound produzido por [`super::text_to_compound_path`] — que passa pela mesma
/// porta de refluxo. Logo *não há código de layout nesta wave*: há uma consequência, e ela
/// tem de ser afirmada em vez de assumida. Sem ela, um texto num fluxo mediria a largura da
/// linha mais longe **como se ela nunca quebrasse**, e o contêiner cresceria para fora da
/// caixa que o artista autorou.
#[test]
fn a_boxed_text_measures_narrower_and_taller_than_a_loose_one() {
    let f = font();
    let text = "the quick brown fox jumps over the lazy dog again and again";
    let paint = Some(Paint::solid(ph2d_vec_scene::Rgba8::new(0, 0, 0, 255)));
    let measure = |w: Option<f64>| {
        let p = text_to_compound_path(
            &f,
            text,
            &boxed(0.3, w),
            &[],
            &TextPlacement::At([0.0, 0.0]),
            &paint,
            &None,
        )
        .expect("a fixture tem de conter o fenomeno");
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for v in p.verts_all() {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        [hi[0] - lo[0], hi[1] - lo[1]]
    };
    let loose = measure(None);
    let boxed_ = measure(Some(BOX));
    assert!(
        boxed_[0] <= BOX && boxed_[0] < loose[0] * 0.75,
        "a caixa tem de ESTREITAR a medida ({:.3} contra {:.3}, teto {BOX:.3})",
        boxed_[0],
        loose[0]
    );
    // A outra metade, e ela é o que impede o gate de passar sobre um texto TRUNCADO: nada é
    // perdido, o que não coube desceu.
    assert!(
        boxed_[1] > loose[1] * 1.5,
        "o que saiu da largura tem de aparecer na ALTURA ({:.3} contra {:.3})",
        boxed_[1],
        loose[1]
    );
}
