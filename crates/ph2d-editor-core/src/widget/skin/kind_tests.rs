//! Gates do **CATÁLOGO** — irmão de `kind.rs`, e o corte é o mesmo do produto.
//!
//! ⚠️ Aqui moram as leis sobre *que tipos existem e o que cada um É*: o número que viaja no
//! documento, a ida-e-volta dele, a chave que o artista lê, e quem consome uma cor. As leis sobre
//! *como um tipo aparece* ficam no `tests.rs` ao lado — elas falham por motivos diferentes e
//! precisam de fixtures diferentes (aqui não se pinta nada).

use super::super::*;

/// **Os códigos são literais PINADOS.** Reordenar o enum não pode mover um número que já viaja
/// em arquivos salvos.
#[test]
fn the_codes_are_pinned_and_unique() {
    assert_eq!(WidgetKind::Button.code(), 1);
    assert_eq!(WidgetKind::Toggle.code(), 2);
    assert_eq!(WidgetKind::Checkbox.code(), 3);
    assert_eq!(WidgetKind::Slider.code(), 4);
    assert_eq!(WidgetKind::ProgressBar.code(), 5);
    assert_eq!(WidgetKind::Tag.code(), 6);
    assert_eq!(WidgetKind::TextInput.code(), 7);
    assert_eq!(WidgetKind::Card.code(), 8);
    assert_eq!(WidgetKind::SectionHeader.code(), 9);
    assert_eq!(WidgetKind::ListItem.code(), 10);
    assert_eq!(WidgetKind::Spinner.code(), 11);
    assert_eq!(WidgetKind::Divider.code(), 12);
    assert_eq!(WidgetKind::NumberInput.code(), 13);
    assert_eq!(WidgetKind::LevelMeter.code(), 14);
    assert_eq!(WidgetKind::ColorSwatch.code(), 15);
    assert_eq!(WidgetKind::IconButton.code(), 16);

    let mut seen = std::collections::BTreeSet::new();
    for kind in WidgetKind::ALL {
        assert!(seen.insert(kind.code()), "codigo repetido em {kind:?}");
    }
    assert_eq!(seen.len(), WidgetKind::ALL.len(), "a lista perdeu um tipo");
}

/// **Ida e volta é total**, e um código desconhecido devolve `None`.
///
/// ⚠️ O `None` é a metade que o plano pede como gate (*"um `kind` desconhecido degrada para o
/// desenho, nunca para um painel vazio"*): ele é o que um documento autorado por um build mais
/// novo produz, e recusá-lo seria recusar o arquivo.
#[test]
fn the_round_trip_is_total_and_the_unknown_degrades() {
    for kind in WidgetKind::ALL {
        assert_eq!(WidgetKind::from_code(kind.code()), Some(kind));
    }
    assert_eq!(WidgetKind::from_code(0), None, "0 nao e' tipo nenhum");
    assert_eq!(WidgetKind::from_code(9999), None, "um tipo do futuro");
}

/// **Cada tipo tem chave i18n PRÓPRIA** — duas iguais fariam dois chips com o mesmo nome, e o
/// artista não teria como distinguir o que está a escolher.
#[test]
fn every_kind_has_its_own_i18n_key() {
    let mut seen = std::collections::BTreeSet::new();
    for kind in WidgetKind::ALL {
        let key = kind.i18n_key();
        assert!(
            key.starts_with("panel.vector.widget.kind."),
            "{kind:?} tem chave fora da familia: {key}"
        );
        assert!(seen.insert(key), "chave repetida: {key}");
        assert_ne!(
            ph2d_i18n::tr(key),
            key,
            "a chave {key} nao esta' na tabela de i18n — o chip mostraria a chave crua"
        );
    }
}

/// **Quem CONSOME uma cor é exactamente a swatch** — a lista que os dois construtores perguntam.
///
/// ⚠️ Ele existe porque a `takes_colour` não é consultada pelo pintor (o braço já sabe o que
/// desenha): ela decide quem **lê o preenchimento da forma**, no construtor de spec do painel e na
/// ponte do canvas. Uma mutação dela sobrevive a todo gate de pintura, e o defeito seria um
/// `Slider` mudando de tinta porque o artista pintou a forma.
#[test]
fn the_colour_taking_list_is_exactly_the_swatch() {
    let takers: Vec<WidgetKind> = WidgetKind::ALL
        .into_iter()
        .filter(|k| k.takes_colour())
        .collect();
    assert_eq!(takers, vec![WidgetKind::ColorSwatch]);
}

/// **Quem CONSOME um ícone é exactamente o botão de ícone** — o irmão da lista acima, e ele existe
/// pela MESMA razão: a `takes_icon` não é consultada pelo pintor, ela decide quem **normaliza a
/// geometria da forma**. Uma mutação dela sobrevive a todo gate de pintura, e o preço seria uma
/// normalização por-quadro por-widget para catorze tipos que não desenham glifo nenhum.
#[test]
fn the_icon_taking_list_is_exactly_the_icon_button() {
    let takers: Vec<WidgetKind> = WidgetKind::ALL
        .into_iter()
        .filter(|k| k.takes_icon())
        .collect();
    assert_eq!(takers, vec![WidgetKind::IconButton]);
}

/// **Os dois canais são DISJUNTOS** — nenhum tipo pede os dois parâmetros.
///
/// ⚠️ Não é uma lei do mundo (um dia um tipo pode pedir cor E glifo, e então isto é reescrito com
/// um motivo); é a afirmação de que **hoje** cada campo do canal tem um dono só, que é o que torna
/// os dois gates de inércia acima capazes de excluir UM tipo cada.
#[test]
fn no_kind_asks_for_both_channels() {
    for kind in WidgetKind::ALL {
        assert!(
            !(kind.takes_colour() && kind.takes_icon()),
            "{kind:?} pede os dois canais — os gates de inercia excluem um tipo cada"
        );
    }
}
