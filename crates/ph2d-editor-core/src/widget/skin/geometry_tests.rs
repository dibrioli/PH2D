//! Gates da geometria da família de LISTA — ver o pai.

use super::super::PREVIEW_ID;
use super::*;

/// **A geometria da opção é a DO PINTOR** — pinada contra os widgets reais, não re-derivada.
///
/// ⚠️ **Este gate existe porque a porta REPETE a expressão**, e a repetição sem prova é a segunda
/// resposta que este arquivo inteiro existe para não ter. Ele constrói um `Tabs` e um `RadioGroup`
/// de verdade e exige que os três números coincidam índice a índice: se um pintor mudar de layout,
/// a porta fica **vermelha** em vez de continuar a registar retângulos onde nada é desenhado.
///
/// ⚠️ E o `RadioGroup` é construído **Horizontal**, que é a orientação com que a pele o desenha —
/// medi-lo em Vertical seria pinar um layout que este painel não produz.
#[test]
fn the_inline_option_rect_is_where_the_painter_puts_it() {
    use crate::widget::{RadioGroup, RadioOption, RadioOrientation, TabItem, Tabs};

    let host = Rect::new(10.0, 20.0, 300.0, 28.0);
    let labels = ["A", "B", "C"];
    let n = labels.len();

    let tabs = Tabs::new(
        PREVIEW_ID,
        "t",
        labels
            .iter()
            .map(|l| TabItem::new(PREVIEW_ID, *l))
            .collect(),
    );
    let radio = RadioGroup::new(
        PREVIEW_ID,
        "r",
        labels
            .iter()
            .enumerate()
            .map(|(i, l)| RadioOption::new(PREVIEW_ID, i, *l))
            .collect(),
    )
    .orientation(RadioOrientation::Horizontal);

    for i in 0..n {
        let got = inline_option_rect(WidgetKind::Tabs, host, i, n)
            .expect("a faixa de abas nao devolveu retangulo");
        assert_eq!(got, tabs.tab_rect(host, i), "a aba {i} saiu noutro lugar");
        assert_eq!(
            inline_option_rect(WidgetKind::SegmentedAdaptive, host, i, n),
            Some(tabs.tab_rect(host, i)),
            "a segmentada e' desenhada pelo pintor de abas — o layout tem de ser o mesmo"
        );
        assert_eq!(
            inline_option_rect(WidgetKind::RadioGroup, host, i, n),
            Some(radio.option_rect(host, i)),
            "a opcao de radio {i} saiu noutro lugar"
        );
    }
}

/// **E ela devolve `None` onde não há opção DENTRO do retângulo.**
///
/// ⚠️ As três recusas têm causas diferentes e cada uma é um bug próprio se cair: um `Slider` com
/// retângulos de opção por cima ficaria inarrastável; um índice além da lista registaria uma
/// faixa fantasma à direita da última; e o **dropdown** é o caso que engana, porque ele *tem*
/// opções — só que elas não estão no `host`, e sim no popover, que pode até ter virado para cima.
#[test]
fn there_is_no_inline_rect_where_there_is_no_inline_option() {
    let host = Rect::new(0.0, 0.0, 120.0, 28.0);
    assert_eq!(inline_option_rect(WidgetKind::Slider, host, 0, 3), None);
    assert_eq!(inline_option_rect(WidgetKind::Tabs, host, 3, 3), None);
    assert_eq!(
        inline_option_rect(WidgetKind::Dropdown, host, 0, 3),
        None,
        "as opcoes de um dropdown vivem no popover — um retangulo aqui seria plausivel e errado"
    );
}
