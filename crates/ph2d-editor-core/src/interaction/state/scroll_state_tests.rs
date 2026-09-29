//! As leis do quadro de rolagem (D9 da spec `04_a_rolagem_unica`): o que um quadro não publicou é
//! esquecido no fim dele, e a roda sem altura publicada não mexe.

use super::*;

const LISTA: NodeId = NodeId(1);
const VAZIO: NodeId = NodeId(2);
const BARRA: NodeId = NodeId(3);
const BARRA_VELHA: NodeId = NodeId(4);

fn publica(store: &mut WidgetStore, id: NodeId) {
    store.set_panel_content_h(id, 1000.0);
    store.set_panel_visible_h(id, 200.0);
}

/// ⭐ Quem publicou neste quadro fica; quem não publicou sai — as alturas, o voo e o dono da barra.
#[test]
fn o_que_o_quadro_nao_publicou_e_esquecido() {
    let mut store = WidgetStore::default();
    // O quadro ANTERIOR: os dois rolavam, as duas barras tinham dono, e o VAZIO estava em voo.
    store.begin_scroll_frame();
    publica(&mut store, LISTA);
    publica(&mut store, VAZIO);
    store.publish_scroll_bar(BARRA, LISTA, Rect::new(0.0, 0.0, 8.0, 200.0));
    store.publish_scroll_bar(BARRA_VELHA, VAZIO, Rect::new(0.0, 0.0, 8.0, 200.0));
    store.set_panel_scroll(VAZIO, 300.0);
    store.set_fling(VAZIO, Some(900.0));
    store.end_scroll_frame();
    // ⚠️ CONTROLO: um quadro inteiro não apaga o que ele próprio publicou.
    assert_eq!(store.panel_content_h(VAZIO), Some(1000.0));
    assert!(
        store.scroll_is_direct(VAZIO),
        "o voo do quadro que o publicou continua"
    );

    // Este quadro: o VAZIO deixou de passar pela porta (o estado vazio da escultura).
    store.begin_scroll_frame();
    publica(&mut store, LISTA);
    store.publish_scroll_bar(BARRA, LISTA, Rect::new(0.0, 0.0, 8.0, 200.0));
    store.end_scroll_frame();

    assert_eq!(store.panel_content_h(LISTA), Some(1000.0));
    assert_eq!(store.panel_visible_h(LISTA), Some(200.0));
    assert_eq!(store.scroll_bar_panel(BARRA), Some(LISTA));
    assert_eq!(
        store.panel_content_h(VAZIO),
        None,
        "altura velha sobreviveu ao quadro"
    );
    assert_eq!(store.panel_visible_h(VAZIO), None);
    assert_eq!(
        store.scroll_bar_panel(BARRA_VELHA),
        None,
        "a barra velha ficou com dono"
    );
    assert!(
        !store.scroll_is_direct(VAZIO),
        "a inércia continua a voar sobre um painel que já não desenha lista"
    );
    assert_eq!(
        store.panel_scroll_target(VAZIO),
        300.0,
        "o sítio onde o artista deixou a lista NÃO se esquece — reabrir volta lá"
    );
}

/// ⭐ Sem altura publicada, a roda não mexe no alvo — antes ela somava sem tecto.
#[test]
fn a_roda_sem_altura_publicada_nao_mexe() {
    let mut store = WidgetStore::default();
    store.set_panel_scroll(VAZIO, 40.0);
    store.wheel_panel(VAZIO, -500.0);
    assert_eq!(store.panel_scroll_target(VAZIO), 40.0);
    // CONTROLO: com as alturas, a mesma roda anda e pára no fim publicado.
    publica(&mut store, LISTA);
    store.wheel_panel(LISTA, -500.0);
    assert_eq!(store.panel_scroll_target(LISTA), 500.0);
    store.wheel_panel(LISTA, -5000.0);
    assert_eq!(
        store.panel_scroll_target(LISTA),
        800.0,
        "o tecto é content − visible"
    );
}

/// Escrever alturas FORA de um quadro de pintura (um teste, uma cena) fica até ao quadro seguinte.
#[test]
fn alturas_escritas_fora_de_um_quadro_ficam_ate_ao_proximo() {
    let mut store = WidgetStore::default();
    publica(&mut store, LISTA);
    store.begin_scroll_frame();
    store.end_scroll_frame();
    assert_eq!(
        store.panel_content_h(LISTA),
        None,
        "CONTROLO: um quadro que não o publicou esquece-o"
    );
    publica(&mut store, LISTA);
    assert_eq!(store.panel_content_h(LISTA), Some(1000.0));
}
