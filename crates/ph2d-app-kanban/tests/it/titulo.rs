//! O título das listas (dono 28/10: «maior e mais destacado», MAIÚSCULAS): pintado em maiúsculas,
//! MEDIDO com o mesmo texto, tamanho, peso e espaçamento entre letras com que se pinta, e o texto
//! guardado não muda.

use crate::{Mesa, centro};
use ph2d_text::FontWeight;

/// A recolhida, fora o título deitado: `10` + botão `28` + `6` + `6` + contagem `18` + `10`.
const RECOLHIDA_SEM_TITULO: f32 = 78.0;

#[test]
fn o_titulo_da_lista_e_pintado_em_maiusculas_medido_como_se_pinta_e_o_texto_guardado_fica() {
    let (mut m, id) = Mesa::cena1();
    let antes = m.bytes();
    let pintados = m.docs().probe_painted_titles(id.0);
    let k = m.kanban(id);
    assert_eq!(
        pintados.len(),
        k.lists.len(),
        "cada lista pintou o seu título"
    );
    for (l, texto, _, _) in &pintados {
        let guardado = &k.lists[l].title;
        assert_eq!(texto, &guardado.to_uppercase(), "pintado em MAIÚSCULAS");
        assert_ne!(texto, guardado, "e diferente do guardado ({guardado})");
    }

    // A recolhida mede o título deitado: a altura dela tem a largura do texto PINTADO.
    let review = m.lista(id, "Review");
    let (_, texto, px, peso) = pintados
        .iter()
        .find(|t| t.0 == review)
        .cloned()
        .expect("o título da Review");
    assert_eq!(texto, "REVIEW");
    let em = ph2d_tokens::kanban::LIST_TITLE_TRACKING_EM;
    let pintada =
        m.ts.prefix_width_tracked(&texto, px, FontWeight::new(peso), em)
            .ceil();
    let medida = m.rect_lista(id, review).h - RECOLHIDA_SEM_TITULO;
    assert!(
        (medida - pintada).abs() < 0.01,
        "medida {medida} == pintada {pintada}"
    );
    let minusculas =
        m.ts.prefix_width_tracked("Review", px, FontWeight::new(peso), em)
            .ceil();
    assert!(pintada > minusculas, "a medida é a das maiúsculas");
    let sem_espaco =
        m.ts.prefix_width_weighted(&texto, px, FontWeight::new(peso))
            .ceil();
    assert!(pintada > sem_espaco, "e com o espaçamento entre letras");

    // Editar no lugar mostra o texto como foi escrito, e o documento não mudou.
    let backlog = m.lista(id, "Backlog");
    let r = m.rect_lista(id, backlog);
    m.clica(centro(ph2d_editor_core::zones::Rect::new(
        r.x + 14.0,
        r.y + 10.0,
        40.0,
        20.0,
    )));
    assert_eq!(m.docs().probe_field(id.0).as_deref(), Some("Backlog"));
    assert_eq!(
        m.bytes(),
        antes,
        "o texto guardado ficou igual, byte a byte"
    );
}
