//! **Os dois pintores próprios deste painel PERGUNTAM ao store.**
//!
//! Irmão exacto do gate do Audio Editor, e pela mesma razão: este painel também não usa
//! `widget::Button` — o `paint_toggle` dele pinta `Bg3`/`active_bg` à mão —, então o gate global
//! `every_button_wears_the_live_hover` **é cego a ele**. O `paint_labeled_slider` usa o
//! `paint_slider` canónico e mesmo assim estava inerte: construir o `Slider` **não** é vestir o
//! par vivo.
//!
//! ⚠️ **A mutação que este ficheiro existe para apanhar SOBREVIVEU à suíte inteira** antes de ele
//! existir: trocar `store.button_visual(id)` por um par duro deixa os 24 testes do painel verdes
//! e todo mute/solo/enable morto sob o rato. É o mesmo buraco que o painel irmão teve, medido duas
//! vezes.
//!
//! ⚠️ **O controlo positivo é metade do gate:** um scanner que deixe de encontrar as `fn` reporta
//! zero ofensores — o mesmo que um produto correcto reporta.

use std::fs;

/// Os dois pintores, com a `fn` e a porta que cada um tem de atravessar.
const PAINTERS: [(&str, &str); 2] = [
    ("pub(crate) fn paint_toggle(", "store.button_visual(id)"),
    (
        "pub(crate) fn paint_labeled_slider(",
        "store.slider_visual(id)",
    ),
];

fn body_after(src: &str, start: usize) -> &str {
    let rest = &src[start..];
    match rest.find("\n}\n") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

fn widgets_src() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/paint_widgets.rs"))
        .expect("paint_widgets.rs")
}

/// **Cada pintor lê o estado vivo do id que está a pintar.**
///
/// *Mutação que deve sangrar:* trocar a chamada ao store por um par duro em qualquer um dos dois.
#[test]
fn the_two_painters_read_the_live_visual() {
    let src = widgets_src();
    for (sig, door) in PAINTERS {
        // ⚠️ `expect` e não `if let`: um pintor que se mude tem de falhar ALTO, nunca varrer um
        // sítio onde a coisa julgada deixou de existir e passar por vácuo.
        let at = src
            .find(sig)
            .unwrap_or_else(|| panic!("`{sig}` desapareceu — o gate ficou a olhar para nada"));
        let body = body_after(&src, at);
        assert!(
            body.contains(door),
            "`{sig}` pinta sem perguntar ao store (esperava `{door}`) — o controlo volta a ser \
             inerte sob o rato"
        );
    }
}

/// **O tom quente NÃO é escolhido aqui: ele é pedido à família.**
///
/// O `active_bg` deste painel é um PARÂMETRO de quem chama (`Danger` no Mute, `Warn` no Solo,
/// `Accent` nas master-fx), então uma tabela de pares repouso→quente escrita neste ficheiro
/// cresceria com cada chamador novo — e o quarto nasceria sem ela. Quem responde é
/// `motion::hover_of`, com o gate `the_family_hover_map_agrees_with_the_button_kinds` a provar que
/// ela concorda com o catálogo onde ambos respondem.
///
/// *Mutação que deve sangrar:* escolher o tom quente com um `match` local.
#[test]
fn the_hot_tone_comes_from_the_family_not_from_a_local_table() {
    let src = widgets_src();
    let at = src
        .find("pub(crate) fn paint_toggle(")
        .expect("`paint_toggle` desapareceu — o gate ficou a olhar para nada");
    let body = body_after(&src, at);
    assert!(
        body.contains("hover_of(") && body.contains("pressed_of("),
        "o tom quente foi escolhido aqui em vez de pedido a' familia"
    );
    assert!(
        body.contains("motion::hover_axis("),
        "a transicao foi re-escrita aqui em vez de pedida ao substrato"
    );
}
