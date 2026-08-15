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
pub(super) fn glyph(w: f64) -> ph2d_vector::BezPath {
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

/// Os quatro tipos da família de LISTA, **enumerados à mão**.
///
/// ⚠️ Escrito assim pela MESMA lição que os gates da cor e do glifo pagaram: derivar esta lista de
/// `takes_options()` — a função que os braços do `match` consultam — faria o laço encolher junto
/// com o defeito, e um gate que se filtra a si próprio não pode falhar pelo motivo que alega.
const LIST_KINDS: [WidgetKind; 4] = [
    WidgetKind::Tabs,
    WidgetKind::RadioGroup,
    WidgetKind::SegmentedAdaptive,
    WidgetKind::Dropdown,
];

fn three() -> Vec<String> {
    vec!["A".into(), "B".into(), "C".into()]
}

/// Pinta um tipo da família de lista com as opções e a marca pedidas.
fn paint_list(kind: WidgetKind, opts: &[String], sel: usize, ts: &mut TextSystem) -> Print {
    let mut sc = ph2d_vector::VectorScene::new();
    paint_widget_skin(
        kind,
        "Mode",
        SkinParam {
            options: opts,
            selected: sel,
            ..Default::default()
        },
        rect(),
        &mut sc,
        ts,
        Theme::Forge,
    );
    print(&sc)
}

/// **Os RÓTULOS chegam ao pintor** — o gate do terceiro campo do canal.
///
/// ⚠️ **Duas asserções, e são precisas pela mesma razão do gate da cor:** *listas diferentes
/// pintam diferente* sozinha ficaria verde num braço que só contasse os itens (as duas listas têm
/// três); *vazia difere de cheia* sozinha ficaria verde num braço que só distinguisse "tem opções"
/// de "não tem". Juntas, os rótulos têm de atravessar inteiros.
#[test]
fn the_options_reach_the_paint() {
    let mut ts = text();
    let other = vec!["Xyz".to_string(), "Wq".into(), "Kk".into()];
    for kind in LIST_KINDS {
        let a = paint_list(kind, &three(), 0, &mut ts);
        let b = paint_list(kind, &other, 0, &mut ts);
        let empty = paint_list(kind, &[], 0, &mut ts);
        assert_ne!(a, b, "{kind:?} pintou dois conjuntos de rotulos IGUAIS");
        assert_ne!(
            a, empty,
            "{kind:?} pintou com opcoes como pinta sem nenhuma"
        );
    }
}

/// **E os rótulos são INERTES em quem não os consome.**
///
/// ⚠️ Exclusão LITERAL, e a varredura é conferida contra o vácuo — o irmão exacto dos gates da cor
/// e do glifo. Sem ele, o dia em que um braço de fora da família passasse a ler `options` faria um
/// slider mudar de desenho porque o artista pendurou filhos na forma, sem teste nenhum falhar.
#[test]
fn the_options_are_inert_in_every_kind_that_does_not_take_them() {
    let mut ts = text();
    let opts = three();
    let mut checked = 0;
    for kind in WidgetKind::ALL {
        if LIST_KINDS.contains(&kind) {
            continue;
        }
        checked += 1;
        let a = paint_list(kind, &[], 0, &mut ts);
        let b = paint_list(kind, &opts, 1, &mut ts);
        assert_eq!(
            a, b,
            "{kind:?} respondeu a opcoes que ele nao declara consumir"
        );
    }
    assert_eq!(
        checked,
        WidgetKind::ALL.len() - LIST_KINDS.len(),
        "a varredura ficou vazia — o gate passaria por vacuo"
    );
}

/// **A MARCA chega ao pintor** — o quarto campo do canal, e o que faz a lista ser um *controle*.
///
/// ⚠️ Os três estados têm de ser distintos DOIS A DOIS. *A marca 0 difere da 2* sozinha ficaria
/// verde num braço que só distinguisse "a primeira" de "qualquer outra" — o que um `> 0` produz.
#[test]
fn the_marked_option_reaches_the_paint() {
    let mut ts = text();
    let opts = three();
    for kind in LIST_KINDS {
        let p: Vec<Print> = (0..3)
            .map(|i| paint_list(kind, &opts, i, &mut ts))
            .collect();
        assert_ne!(p[0], p[1], "{kind:?}: marcar a 1a e a 2a pinta igual");
        assert_ne!(p[1], p[2], "{kind:?}: marcar a 2a e a 3a pinta igual");
        assert_ne!(p[0], p[2], "{kind:?}: marcar a 1a e a 3a pinta igual");
    }
}

/// **UM ÍNDICE FORA DO ALCANCE NÃO MARCA NADA — ele não inventa uma escolha.**
///
/// A lei que o doc do [`SkinParam::selected`] sempre afirmou e que metade da família violava:
/// medido antes desta wave, com três opções e `selected = 7`, `Tabs` e `SegmentedAdaptive`
/// pintavam **exactamente** como `selected = 2` (o construtor `Tabs::selected` clampa) enquanto
/// `RadioGroup` e `Dropdown` não marcavam nenhuma. Três comportamentos, um campo.
///
/// ⚠️ **As duas asserções são a lei inteira, e nenhuma basta sozinha.** *Dois índices fora do
/// alcance pintam igual* é verdade sob o CLAMP também (7 e 99 clampam ambos para 2), então ela
/// não distingue nada por si; *fora do alcance difere da última* é o que o clamp viola. A primeira
/// existe porque sem ela um braço podia marcar `selected % len`, que difere da última e ainda
/// assim inventa uma escolha.
///
/// ⚠️ **O oráculo não menciona `marked_option`** — ele compara desenhos, então continua honesto se
/// a porta for reescrita.
#[test]
fn an_index_past_the_end_marks_nothing_rather_than_inventing_a_choice() {
    let mut ts = text();
    let opts = three();
    for kind in LIST_KINDS {
        let last = paint_list(kind, &opts, opts.len() - 1, &mut ts);
        let past = paint_list(kind, &opts, opts.len() + 4, &mut ts);
        let far = paint_list(kind, &opts, 99, &mut ts);
        assert_eq!(
            past, far,
            "{kind:?}: dois indices que nao nomeiam opcao nenhuma pintaram DIFERENTE — algum \
             deles esta' a escolher alguma coisa"
        );
        assert_ne!(
            past, last,
            "{kind:?}: um indice fora do alcance pintou como a ULTIMA opcao — a pele inventou uma \
             escolha que o documento nao fez"
        );
    }
}
