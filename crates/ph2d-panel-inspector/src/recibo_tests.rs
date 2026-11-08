//! O recibo vive enquanto o passo dele é o topo do desfazer (plano GEF 25 §4).

use super::*;

#[test]
fn o_recibo_vive_so_enquanto_o_passo_dele_e_o_topo() {
    mostra(7, "Physics Body added.".into());
    vigia_com(3, Some(7));
    assert_eq!(
        texto(7).as_deref(),
        Some("Physics Body added."),
        "armado no passo 3"
    );
    vigia_com(3, Some(7));
    assert!(texto(7).is_some(), "nada mudou: fica");
    assert!(texto(8).is_none(), "é só do objecto dele");
    vigia_com(4, Some(7));
    assert!(texto(7).is_none(), "outro passo: sai");
    mostra(7, "x".into());
    vigia_com(3, Some(7));
    vigia_com(2, Some(7));
    assert!(texto(7).is_none(), "o undo: sai");
    mostra(7, "x".into());
    vigia_com(3, Some(9));
    assert!(texto(7).is_none(), "outro objecto: sai");
}
