//! ⭐⭐⭐ **A COSTURA do quadro assente: o refinamento recebe a CENA, e não uma cena vazia.**
//!
//! ⚠️⚠️ **Este gate é textual de propósito, e a razão é o sítio onde a lei corre.** O laço vive
//! dentro de um `std::thread::spawn` do [`crate::smoke_draw_thread`], que nenhum teste alcança — é
//! precisamente por isso que a lei da acumulação foi extraída para a porta
//! [`ph2d_field_render::refine_hemisphere`], onde ela **é** testada. O que sobra por provar é a
//! outra metade: *a thread chama a porta com os materiais e as lâmpadas da cena?*
//!
//! ⛔⛔ Sem isto, trocar `&surfaces` por `&Surfaces { all: &[], owners: None }` deixa o ricochete
//! **inerte no produto** com todos os gates da crate do traçado VERDES — a porta degenera para o
//! canal vazio de propósito (é o que faz uma cena sem luz continuar byte-idêntica), logo a
//! degeneração é **indistinguível** de nunca ter sido ligada.
//!
//! *Um gate que chama a função em vez de percorrer a rota afirma que a peça certa existe, nunca
//! que a thread a usa.*

/// O quadro assente entrega ao refinamento os materiais e as lâmpadas da cena.
#[test]
fn o_refinamento_do_quadro_assente_recebe_a_cena() {
    let fonte = include_str!("smoke_draw_thread.rs");
    let chamada = fonte
        .split_once("refine_hemisphere(")
        .map(|(_, resto)| resto)
        .expect("a thread do quadro assente tinha de chamar o `refine_hemisphere`");
    // Os argumentos até ao fecho da chamada — o `|sh, passagem|` é o início do fecho.
    let args = chamada
        .split_once("|sh, passagem|")
        .map(|(a, _)| a)
        .expect("a chamada tinha de terminar no fecho da entrega");

    for (o_que, agulha) in [
        ("os materiais da cena", "&surfaces,"),
        ("as lâmpadas da cena", "&p.lights,"),
    ] {
        assert!(
            args.contains(agulha),
            "o quadro assente não passa {o_que} ao refinamento (`{agulha}` não está na chamada) — \
             o ricochete fica inerte no produto e NENHUM gate do traçado o vê, porque a porta \
             degenera para o canal vazio de propósito"
        );
    }
    assert!(
        !args.contains("all: &[]"),
        "o quadro assente passa uma cena VAZIA ao refinamento — isso desliga o ricochete em \
         silêncio"
    );
}
