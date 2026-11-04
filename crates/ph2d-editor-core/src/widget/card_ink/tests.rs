use super::*;

fn rgb(c: Color) -> [u8; 3] {
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    [r, g, b]
}

fn gap(a: Color, b: Color) -> u8 {
    let (a, b) = (rgb(a), rgb(b));
    (0..3).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}

/// ⭐⭐ Report do dono (01/11): o tema do cartão só mudava as letras. Dois temas diferentes dão
/// fundos, caixas e segmentos diferentes; e o cartão SEM tema fica o da lista (o tema de um
/// vizinho não vaza para ele).
#[test]
fn two_card_themes_paint_two_different_cards_and_the_list_theme_stays_put() {
    let lista = Theme::Dark;
    let (a, b, sem) = (
        CardInk::of_card(Some(Theme::PlumberRed), lista),
        CardInk::of_card(Some(Theme::PlumberGreen), lista),
        CardInk::of_card(None, lista),
    );
    for t in [
        ColorToken::KanbanCard,
        ColorToken::CardField,
        ColorToken::CardFieldHover,
        ColorToken::KanbanCardRaised,
        ColorToken::CardLabel,
        ColorToken::KanbanBlueBold,
    ] {
        assert_ne!(
            rgb(a.color(t)),
            rgb(b.color(t)),
            "{t:?}: os dois temas pintam igual"
        );
        assert_eq!(
            rgb(sem.color(t)),
            rgb(resolve(t, lista)),
            "{t:?}: o cartão sem tema saiu do tom da lista"
        );
        assert_ne!(
            rgb(a.color(t)),
            rgb(sem.color(t)),
            "{t:?}: o tema próprio pinta como a lista"
        );
    }
    // Escolher o tema da própria lista é o mesmo que não ter tema.
    assert!(!CardInk::of_card(Some(lista), lista).is_own());
}

/// ⭐ Em TODO tema que o menu oferece a um cartão, a caixa, a caixa sob o rato e o segmento aceso
/// separam-se do fundo (e a caixa sob o rato da caixa) — senão o campo some no cartão.
#[test]
fn every_card_theme_keeps_its_boxes_off_its_surface() {
    for t in Theme::ALL.iter().copied() {
        let ink = CardInk::of_card(
            Some(t),
            if t == Theme::Dark {
                Theme::Gray
            } else {
                Theme::Dark
            },
        );
        let fundo = ink.color(ColorToken::KanbanCard);
        let campo = ink.color(ColorToken::CardField);
        let hover = ink.color(ColorToken::CardFieldHover);
        let aceso = ink.color(ColorToken::KanbanCardRaised);
        assert!(gap(campo, fundo) >= 8, "{t:?}: caixa colada ao fundo");
        assert!(gap(hover, campo) >= 4, "{t:?}: hover igual à caixa");
        assert!(gap(aceso, fundo) >= 4, "{t:?}: aceso igual ao fundo");
    }
}

/// ⭐ A foto da galeria (03/11): os campos de sempre pintavam-se a preto sobre o cartão. Dentro do
/// corpo publicado, a família de campos enche o `CardField` da tinta; fora dele, nada muda (a
/// janela flutuante, `PH2D_CARDS=0`). *Mutação: tirar o ramo do `field_fill` ⇒ a 2.ª asserção.*
#[test]
fn a_field_inside_a_card_body_sits_on_the_card_and_outside_nothing_changes() {
    use crate::widget::text_input::{TextInputState, field_fill};
    let tema = Theme::Dark;
    let fora = field_fill(TextInputState::Normal, tema);
    let ink = CardInk::of_list(tema);
    assert_ne!(
        rgb(fora),
        rgb(ink.color(ColorToken::CardField)),
        "fixtura: o campo de fora tem de diferir do do cartão"
    );
    let dentro = com(ink, || field_fill(TextInputState::Normal, tema));
    assert_eq!(rgb(dentro), rgb(ink.color(ColorToken::CardField)));
    assert_eq!(
        rgb(field_fill(TextInputState::Normal, tema)),
        rgb(fora),
        "a tinta vazou para fora do corpo"
    );
}

/// ⭐ T25, 05 §4: a lista aberta de um chip de cartão leva a tinta dele — tema próprio incluído —
/// e só a do chip dela; fora de um cartão `opaque` é o token de sempre, ao bit.
/// *Mutação: o `lembra_popover` fora ⇒ `None`; `opaque` sem a tinta ⇒ o tom do Inspector.*
#[test]
fn a_chip_list_in_a_card_takes_the_card_ink() {
    use crate::widget::resolve_opaque as opaque;
    use crate::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
    use crate::zones::Rect;
    use ph2d_a11y::NodeId;
    let lista = Theme::Dark;
    let ink = CardInk::of_card(Some(Theme::PlumberRed), lista);
    let (id, outro) = (NodeId(70), NodeId(71));
    let dd = Dropdown::new(id, "", vec![DropdownOption::new(NodeId(72), 0_u8, "A")]).open(true);
    let mut cena = ph2d_vector::VectorScene::new();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let r = Rect::new(0.0, 0.0, 100.0, 28.0);
    paint_dropdown_chip(&dd, r, &mut cena, &mut ts, lista);
    assert_eq!(tinta_do_popover(id), None, "chip fora do cartão não lembra");
    com(ink, || {
        paint_dropdown_chip(&dd, r, &mut cena, &mut ts, lista)
    });
    assert_eq!(
        tinta_do_popover(outro),
        None,
        "a lista de outro chip não a leva"
    );
    com(ink, || {
        paint_dropdown_chip(&dd, r, &mut cena, &mut ts, lista)
    });
    assert_eq!(tinta_do_popover(id), Some(ink));
    assert_eq!(tinta_do_popover(id), None, "lê-se uma vez");
    let fundo = ColorToken::BgElev;
    let de_sempre = opaque(fundo, lista);
    assert_eq!(rgb(de_sempre), rgb(resolve(fundo, lista)));
    let no_cartao = com(ink, || opaque(fundo, lista));
    assert_eq!(rgb(no_cartao), rgb(ink.color(ColorToken::KanbanCardRaised)));
    assert!(
        gap(de_sempre, no_cartao) > 0,
        "o popover ficou no tom do Inspector"
    );
}
