//! **Arch-gate: a ESCRITA dos campos de forma recebe o MESMO latch de «armado para desenhar».**
//!
//! # O defeito (report do Enio, 2026-08-31)
//!
//! *"Troco de Shape na tool Shape e as propriedades da shape não trocam imediatamente."* O latch
//! guarda a ORDEM dos gestos (armei o Polígono × desenhei uma estrela): acendê-lo, apagá-lo e
//! publicá-lo ao painel moram em `ph2d_app_vec::shape_params::frame_fields` e têm gates de
//! COMPORTAMENTO lá (`shape_fields_frame_tests`, F9 — eram metade deste ficheiro, a ler texto). Aqui
//! fica a ponta que continua na shell: o dreno que ESCREVE.

/// O QUADRO pela ordem em que corre (`frame_text::render_frame`).
static SRC: std::sync::LazyLock<String> = std::sync::LazyLock::new(crate::frame_text::render_frame);

/// A ESCRITA é a ponta perigosa e tem de receber o MESMO latch que a pintura: os slots do painel
/// são por ÍNDICE, então pintar a Estrela armada e escrever no Polígono selecionado põe *lados*
/// onde o artista digitou *pontas*, sem erro nenhum.
#[test]
fn the_write_route_receives_the_latch() {
    let escrita = SRC
        .find("crate::vec_shape_params::edit_selected_shape(")
        .expect("a rota de escrita dos campos de forma");
    let corpo = &SRC[escrita..escrita + 1200];
    assert!(
        corpo.contains("self.vec.shape_armed,"),
        "a ESCRITA nao recebe o latch — ela alcancaria a forma que o painel ja' nao mostra"
    );
    // E a pintura sai da porta única, na fase dos campos — o mesmo latch, na mesma ordem.
    assert!(
        SRC.contains("ph2d_app_vec::shape_params::frame_fields("),
        "a fase dos campos de forma deixou de chamar a porta única"
    );
}
