//! **Arch-gate: o Delete apaga a FIGURA em mãos — e o alvo mais específico vence.**
//!
//! ## O pedido (Enio, 2026-08-07)
//!
//! *"Permita usar del para deletar a forma selecionada por último."*
//!
//! ## Por que a ORDEM é o gate, e não a presença
//!
//! Há uma tecla só e três donos possíveis, então a precedência **é** a feature:
//!
//! 1. **âncora de curva** — com um nó selecionado o Delete tira o nó
//!    (`curve_delete_selected` já se gateia nisso e recusa quando sobrariam menos de 2 pontos);
//! 2. **a FIGURA** — sem nó selecionado, sai a figura em mãos;
//! 3. **falloff / hero** — o resto do app.
//!
//! É a divisão que o Illustrator faz com *duas ferramentas* (seta branca × seta preta); aqui é uma
//! tecla, então a ordem é o que a expressa. Inverter 1 e 2 faz o Delete comer a curva inteira quando o
//! artista quis tirar um ponto — e o gate de unidade **não vê isso**, porque cada verbo, chamado
//! sozinho, está certo.
//!
//! ⚠️ **E o item 2 fecha um buraco que já existia:** sem ele um Delete com figura viva caía no caminho
//! genérico do hero e apagava a **ENTIDADE** — o sprite inteiro, com a arte dentro.

const CHAIN: &str = include_str!("../src/input_dispatch/keyboard_painter.rs");
/// A cadeia mora em `keyboard_painter.rs` (cortada por assunto); o `keyboard.rs` é quem a CHAMA, e as
/// duas metades precisam de gate — ver o controle positivo abaixo.
const KEYBOARD: &str = include_str!("../src/input_dispatch/keyboard.rs");

/// A cadeia do Delete no Painter está na ordem: âncora → figura → falloff.
///
/// **Mutação que deve sangrar:** trocar os blocos 1 e 2 de lugar, ou apagar o bloco da figura.
#[test]
fn the_painter_delete_chain_tries_the_anchor_then_the_shape_then_the_falloff() {
    let anchor = CHAIN
        .find("self.painter_curve_delete_selected_point()")
        .expect("o delete de ANCORA sumiu da cadeia");
    let shape = CHAIN.find("self.painter_delete_active_shape()").expect(
        "o Delete nao apaga mais a FIGURA em maos — com uma figura viva a tecla cai no caminho \
             generico do hero, que apaga a ENTIDADE (o sprite inteiro)",
    );
    let falloff = CHAIN
        .find("self.painter_delete_selected_falloff_point()")
        .expect("o delete do falloff sumiu da cadeia");
    assert!(
        anchor < shape,
        "a FIGURA vem antes da ANCORA ({shape} < {anchor}) — o Delete comeria a curva inteira quando o \
         artista quis tirar um ponto"
    );
    assert!(
        shape < falloff,
        "a figura vem depois do falloff ({falloff} < {shape})"
    );
}

/// Controle positivo: o arquivo lido é mesmo o roteador de teclado, e a cadeia é a do Painter.
#[test]
fn the_scanned_file_is_the_delete_chain_and_it_is_reached() {
    assert!(CHAIN.contains("KeyCode::Delete | KeyCode::Backspace"));
    assert!(CHAIN.contains("fn painter_delete_chain("));
    // ⚠️ E o roteador de teclado CHAMA a cadeia: sem esta metade, mover os três degraus para um
    // arquivo próprio deixaria a ordem perfeita e a tecla sem dono — o gate ficaria verde sobre uma
    // cadeia que ninguém executa.
    assert!(
        KEYBOARD.contains("self.painter_delete_chain(state, physical_key)"),
        "o roteador de teclado nao chama mais a cadeia do Painter"
    );
}
