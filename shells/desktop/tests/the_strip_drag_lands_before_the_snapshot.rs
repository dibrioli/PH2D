//! **Arch-gate da ORDEM DO FRAME** — o arrasto da tira vira documento ANTES de a tira ser
//! publicada.
//!
//! ## O que este gate protege
//!
//! O painel da tira é stateless: ele pinta o `FlipStripSnapshot` que o shell publicou
//! (`flip_bridge::publish`). O arrasto, por outro lado, só vira pedido no pen-up, e o
//! pedido é consumido no frame SEGUINTE (o paint que o enfileirou já tinha passado).
//!
//! Se o drain rodasse **depois** do publish, o snapshot deste frame descreveria a tira de
//! ANTES do gesto: a célula que o artista acabou de largar em outro quadro voltaria ao
//! lugar antigo por um frame e só então saltaria para o novo. Um pisca de um frame não
//! quebra nada que um teste de unidade veja — e é exatamente o tipo de coisa que se lê como
//! *"a tira treme quando eu arrasto"*.
//!
//! ## Por que um gate de TEXTO
//!
//! Os gates de comportamento (`flip_strip_drag::tests`, e o seam do painel que dirige o
//! ponteiro real) provam as duas METADES: o painel pede, o shell aplica. Nenhum dos dois vê
//! a ORDEM em que o frame do produto os chama — essa vive numa função que exige janela e
//! GPU, onde nenhum deles alcança. Então este gate lê o arquivo do produto e afirma a
//! relação POSICIONAL (não uma distância em bytes: entre os dois passos pode entrar o que
//! for, e um gate ancorado em distância expira sozinho na próxima linha que alguém
//! acrescentar).

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e confira \
             que o drain do arrasto continua ANTES do publish da tira)"
        )
    })
}

#[test]
fn the_strip_drag_is_applied_before_the_strip_snapshot_is_published() {
    let drain = at("flip_strip_drag::apply_strip_intents(");
    let publish = at("flip_bridge::publish(");
    assert!(
        drain < publish,
        "o drain do arrasto da tira (byte {drain}) tem de rodar ANTES de `flip_bridge::publish` \
         (byte {publish}) — senão o snapshot deste frame descreve a tira de antes do gesto e a \
         célula pisca de volta por um frame"
    );
}

/// **Controle positivo:** o gate acima só vale se as duas âncoras existirem de fato. Um
/// `find` que não acha nada entraria em pânico com a mensagem certa, mas um gate cuja
/// âncora é uma string que ninguém escreve mais passaria a medir o nada — e este par
/// (uma chamada que existe, uma que não) é o que prova que a busca discrimina.
#[test]
fn the_anchors_are_real_and_a_missing_one_is_noticed() {
    assert!(SRC.contains("flip_strip_drag::apply_strip_intents("));
    assert!(
        !SRC.contains("flip_strip_drag::apply_strip_intents_that_do_not_exist("),
        "o scanner tem de saber dizer não"
    );
}
