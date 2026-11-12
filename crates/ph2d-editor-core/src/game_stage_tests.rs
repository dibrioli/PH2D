use super::*;

type P = Option<PedidoDaJanela<&'static str>>;

#[test]
fn a_janela_do_jogo_so_abre_com_a_ficha_depois_de_um_quadro_com_o_aviso() {
    let mut p: P = None;
    assert_eq!(
        passo_da_janela(true, false, &mut p),
        PassoDaJanela::PedeAFicha
    );
    assert!(
        aviso_da_janela(p.is_some(), false),
        "o aviso liga no pedido"
    );
    // A ficha chega antes do quadro seguinte: espera esse quadro, depois abre com ela.
    if let Some(q) = p.as_mut() {
        q.ficha = Ficha::Chegou("tok");
    }
    assert_eq!(
        passo_da_janela(true, false, &mut p),
        PassoDaJanela::Abre(Some("tok"))
    );
    assert!(p.is_none());
    assert!(
        aviso_da_janela(false, true),
        "o aviso fica com a janela aberta"
    );
    assert_eq!(passo_da_janela(true, true, &mut p), PassoDaJanela::Nada);
    assert_eq!(passo_da_janela(false, true, &mut p), PassoDaJanela::Fecha);
    assert!(
        !aviso_da_janela(false, false),
        "o aviso some quando o jogo pára"
    );
}

#[test]
fn sem_ficha_espera_ate_ao_tecto_e_abre_sem_ela() {
    let mut p: P = None;
    passo_da_janela(true, false, &mut p);
    for _ in 1..QUADROS_SEM_FICHA {
        assert_eq!(passo_da_janela(true, false, &mut p), PassoDaJanela::Espera);
    }
    assert_eq!(
        passo_da_janela(true, false, &mut p),
        PassoDaJanela::Abre(None)
    );
    let mut p: P = None;
    passo_da_janela(true, false, &mut p);
    if let Some(q) = p.as_mut() {
        q.ficha = Ficha::SemSuporte;
    }
    assert_eq!(
        passo_da_janela(true, false, &mut p),
        PassoDaJanela::Abre(None)
    );
}

#[test]
fn parar_antes_de_abrir_desliga_o_aviso() {
    let mut p: P = None;
    passo_da_janela(true, false, &mut p);
    assert_eq!(
        passo_da_janela(false, false, &mut p),
        PassoDaJanela::Desiste
    );
    assert!(p.is_none());
    assert_eq!(passo_da_janela(false, false, &mut p), PassoDaJanela::Nada);
}

#[test]
fn clique_no_editor_parado_traz_a_janela_do_jogo_a_frente() {
    assert!(reactiva_a_janela(true, true));
    assert!(!reactiva_a_janela(false, true));
    assert!(!reactiva_a_janela(true, false));
}
