//! B20.9 — o erro da prévia que se repete a cada quadro dá UM aviso, não um por quadro.

use super::avisa_uma_vez;
use ph2d_editor_core::toast::{Toast, ToastQueue};

const CHAVE: &str = "app.painter.painter_bridge_upload.preview_upload_failed";

#[test]
fn o_erro_de_envio_persistente_da_um_aviso_e_nao_um_por_quadro() {
    let mut fila = ToastQueue::new();
    for _ in 0..120 {
        avisa_uma_vez(&mut fila, CHAVE, &"device lost");
    }
    assert_eq!(
        fila.len(),
        1,
        "120 quadros com o mesmo erro empilharam avisos"
    );
    avisa_uma_vez(&mut fila, CHAVE, &"out of memory");
    assert_eq!(fila.len(), 2, "um erro DIFERENTE tem de avisar");
    // Controlo: o `push` de antes empilha um por quadro.
    let mut antes = ToastQueue::new();
    for _ in 0..5 {
        antes.push(Toast::error(ph2d_i18n::tr_with(
            CHAVE,
            &[("e", &"device lost")],
        )));
    }
    assert_eq!(antes.len(), 5, "o controlo deixou de ver o defeito");
}
