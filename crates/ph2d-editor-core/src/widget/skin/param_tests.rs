//! Gates do **CANAL de parâmetro por-tipo** — o [`SkinParam`] (plano UI/UX W8b).
//!
//! ⚠️ **Filho de `tests.rs`, e não irmão:** as fixtures (`print`, `rect`, `text`) são as mesmas, e
//! duplicá-las daria duas impressões digitais para a mesma cena — o precedente é o
//! `undo_delta_tests` do Painter.
//!
//! Cada campo do canal tem aqui **um par**: *o parâmetro chega ao braço que o declara* e *ele é
//! inerte em todos os outros*. O segundo é o que mantém a fronteira escrita, e sem ele o dia em
//! que um braço passasse a ler o campo alheio não faria teste nenhum falhar.

use super::*;

/// **A COR chega ao pintor** — o gate do canal [`SkinParam`].
///
/// ⚠️ Sem ele, o braço da swatch podia ignorar o parâmetro e pintar sempre o xadrez: a lista de
/// tipos continuaria completa, todo gate de presença ficaria verde, e o defeito seria *a swatch
/// não mostra a cor que o artista pintou* — visível só numa screenshot.
///
/// ⚠️ **Duas asserções, e as duas são precisas.** *Duas cores diferentes pintam diferente* sozinha
/// ficaria verde num braço que lesse só o CANAL ALFA; *`Some` difere de `None`* sozinha ficaria
/// verde num braço que só distinguisse "tem cor" de "não tem". Juntas, o parâmetro tem de chegar
/// inteiro.
#[test]
fn the_colour_reaches_the_paint() {
    let mut ts = text();
    let paint_with = |rgba, ts: &mut TextSystem| {
        let mut sc = ph2d_vector::VectorScene::new();
        paint_widget_skin(
            WidgetKind::ColorSwatch,
            "Tint",
            SkinParam {
                rgba,
                icon: None,
                ..Default::default()
            },
            rect(),
            &mut sc,
            ts,
            Theme::Forge,
        );
        print(&sc)
    };
    let red = paint_with(Some([200, 40, 40, 255]), &mut ts);
    let blue = paint_with(Some([40, 40, 200, 255]), &mut ts);
    let empty = paint_with(None, &mut ts);
    assert_ne!(
        red, blue,
        "duas cores pintaram a MESMA swatch — o parametro nao chega ao pintor"
    );
    assert_ne!(
        red, empty,
        "uma swatch com cor pintou como uma sem — o xadrez de transparencia e' o `None`, e ele \
         tem de ser distinguivel de uma cor escolhida"
    );
}

/// **E o parâmetro é INERTE em quem não o consome.**
///
/// ⚠️ O irmão do gate acima, e o que mantém a fronteira escrita: um braço que passasse a ler a cor
/// faria o `Slider` mudar de tinta por o artista ter pintado a forma — sem ninguém ter decidido
/// isso, e sem um teste falhar.
///
/// ⚠️ **A exclusão é LITERAL, e não `takes_colour()`** — escrita assim porque a primeira versão
/// usava a própria função sob teste para escolher quem varrer: com ela devolvendo `true` para
/// todos, o laço ficava VAZIO e o gate passava por vácuo (medido). *Um oráculo que se filtra a si
/// próprio não pode falhar pelo motivo que alega.* Quem pina a lista é o gate irmão.
#[test]
fn the_colour_is_inert_in_every_kind_that_does_not_take_it() {
    let mut ts = text();
    let mut checked = 0;
    for kind in WidgetKind::ALL {
        if matches!(kind, WidgetKind::ColorSwatch) {
            continue;
        }
        checked += 1;
        let mut a = ph2d_vector::VectorScene::new();
        let mut b = ph2d_vector::VectorScene::new();
        paint_widget_skin(
            kind,
            "Save",
            SkinParam::default(),
            rect(),
            &mut a,
            &mut ts,
            Theme::Forge,
        );
        paint_widget_skin(
            kind,
            "Save",
            SkinParam {
                options: &[],
                selected: 0,
                rgba: Some([200, 40, 40, 255]),
                icon: None,
            },
            rect(),
            &mut b,
            &mut ts,
            Theme::Forge,
        );
        assert_eq!(
            print(&a),
            print(&b),
            "{kind:?} respondeu a uma cor que ele nao declara consumir"
        );
    }
    assert_eq!(
        checked,
        WidgetKind::ALL.len() - 1,
        "a varredura ficou vazia — o gate passaria por vacuo"
    );
}

/// Um glifo de teste: um triângulo que ninguém confunde com um retângulo.
fn glyph(w: f64) -> ph2d_vector::BezPath {
    let mut p = ph2d_vector::BezPath::new();
    p.move_to((0.0, 0.0));
    p.line_to((w, 12.0));
    p.line_to((0.0, 24.0));
    p.close_path();
    p
}

/// **O GLIFO chega ao pintor** — o irmão exato do gate da cor, no segundo campo do canal.
///
/// ⚠️ **Duas asserções, pelo mesmo motivo de lá.** *Dois desenhos diferentes pintam diferente*
/// sozinha ficaria verde num braço que desenhasse só a MOLDURA e ignorasse a curva — e é
/// exactamente esse o defeito plausível aqui, porque a moldura já é a maior parte da tinta.
/// *`Some` difere de `None`* fecha o outro lado.
#[test]
fn the_glyph_reaches_the_paint() {
    let mut ts = text();
    let (narrow, wide) = (glyph(6.0), glyph(20.0));
    let paint_with = |icon, ts: &mut TextSystem| {
        let mut sc = ph2d_vector::VectorScene::new();
        paint_widget_skin(
            WidgetKind::IconButton,
            "Play",
            SkinParam {
                rgba: None,
                icon,
                ..Default::default()
            },
            rect(),
            &mut sc,
            ts,
            Theme::Forge,
        );
        print(&sc)
    };
    let a = paint_with(Some(IconGlyph::Path(&narrow)), &mut ts);
    let b = paint_with(Some(IconGlyph::Path(&wide)), &mut ts);
    let none = paint_with(None, &mut ts);
    assert_ne!(
        a, b,
        "dois desenhos pintaram o MESMO botao — o glifo nao chega ao pintor"
    );
    assert_ne!(
        a, none,
        "um botao com glifo pintou como um sem — a moldura sozinha e' o `None`, e ela tem de ser          distinguivel de um icone escolhido"
    );
}

/// **E o glifo é INERTE em quem não o consome.**
///
/// ⚠️ A exclusão é **LITERAL** pela lição que o irmão da cor pagou: usar `takes_icon()` para
/// escolher quem varrer deixaria o laço VAZIO no dia em que ela devolvesse `true` para todos, e o
/// gate passaria por vácuo. Quem pina a lista é o gate do catálogo.
#[test]
fn the_glyph_is_inert_in_every_kind_that_does_not_take_it() {
    let mut ts = text();
    let g = glyph(20.0);
    let mut checked = 0;
    for kind in WidgetKind::ALL {
        if matches!(kind, WidgetKind::IconButton) {
            continue;
        }
        checked += 1;
        let mut a = ph2d_vector::VectorScene::new();
        let mut b = ph2d_vector::VectorScene::new();
        paint_widget_skin(
            kind,
            "Save",
            SkinParam::default(),
            rect(),
            &mut a,
            &mut ts,
            Theme::Forge,
        );
        paint_widget_skin(
            kind,
            "Save",
            SkinParam {
                options: &[],
                selected: 0,
                rgba: None,
                icon: Some(IconGlyph::Path(&g)),
            },
            rect(),
            &mut b,
            &mut ts,
            Theme::Forge,
        );
        assert_eq!(
            print(&a),
            print(&b),
            "{kind:?} respondeu a um glifo que ele nao declara consumir"
        );
    }
    assert_eq!(
        checked,
        WidgetKind::ALL.len() - 1,
        "a varredura ficou vazia — o gate passaria por vacuo"
    );
}
