//! A costura da secção Biblioteca (W18): cada verbo existe, pinta-se com hit-rect, está registado
//! (o clique no MEIO dá `Click`) e chega ao barramento como o pedido certo; a sequência escolher ▸
//! Criar novo leva ao pedido da macro escolhida.

use super::seam_macros::{BITS, Palco, info, so};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::macro_edits::{
    FamiliaDaMacro, MacroDaBibliotecaView, MacroPanelInfo, MacroRequest, OndeAplicar,
};
use ph2d_panel_macros::{MacrosPanel, ids, set_current_macros};

const ALVO: u64 = 7;

fn entrada(nome: &str, f: FamiliaDaMacro, tela: Option<(u32, u32)>) -> MacroDaBibliotecaView {
    MacroDaBibliotecaView {
        nome: nome.into(),
        familias: vec![f],
        duracao_s: 2.5,
        tela,
    }
}

fn com_biblioteca() -> MacroPanelInfo {
    MacroPanelInfo {
        biblioteca: vec![
            entrada("Arvore", FamiliaDaMacro::Painter, Some((96, 64))),
            entrada("Vento", FamiliaDaMacro::Motion, None),
        ],
        pode_guardar: true,
        por_cima_de: Some("Ceu".into()),
        por_cima_alvo: Some(ALVO),
        ..info(None, None)
    }
}

fn aplica(nome: &str, onde: OndeAplicar) -> MacroRequest {
    MacroRequest::Aplica {
        nome: nome.into(),
        onde,
    }
}

#[test]
fn guardar_guardar_como_asset_e_importar_chegam_ao_barramento() {
    let mut p = Palco::novo(com_biblioteca());
    so(
        &p.carrega(ids::MACROS_LIB_SAVE),
        MacroRequest::Guarda { entity_bits: BITS },
    );
    so(
        &p.carrega(ids::MACROS_LIB_SAVE_ASSET),
        MacroRequest::GuardaComoAsset { entity_bits: BITS },
    );
    so(&p.carrega(ids::MACROS_LIB_IMPORT), MacroRequest::Importa);
}

/// Sem gravação que se guarde, os dois «Guardar» pintam-se apagados: sem hit-rect, sem clique.
#[test]
fn sem_gravacao_os_guardar_nao_respondem() {
    let p = Palco::novo(MacroPanelInfo {
        pode_guardar: false,
        ..com_biblioteca()
    });
    assert!(p.rect(ids::MACROS_LIB_SAVE).is_none());
    assert!(p.rect(ids::MACROS_LIB_SAVE_ASSET).is_none());
    // Controlo: o Importar não depende do objecto.
    assert!(p.rect(ids::MACROS_LIB_IMPORT).is_some());
}

/// ⭐ A sequência: escolher uma macro na lista ▸ Criar novo = o pedido DESSA macro, no centro.
#[test]
fn escolher_e_criar_novo_leva_ao_pedido_da_escolhida() {
    let mut p = Palco::novo(com_biblioteca());
    assert!(
        p.rect(ids::MACROS_LIB_APPLY_NEW).is_none(),
        "sem escolha não há verbos da escolhida"
    );
    assert!(
        p.carrega(ids::lib_row_id("Vento")).is_empty(),
        "escolher não pede nada"
    );
    assert_eq!(p.st.lib_escolhida.as_deref(), Some("Vento"));
    p.pinta();
    so(
        &p.carrega(ids::MACROS_LIB_APPLY_NEW),
        aplica("Vento", OndeAplicar::Novo { em: None }),
    );
}

#[test]
fn os_verbos_da_escolhida_chegam_ao_barramento() {
    let mut p = Palco::novo(com_biblioteca());
    let _ = p.carrega(ids::lib_row_id("Arvore"));
    p.pinta();
    so(
        &p.carrega(ids::MACROS_LIB_APPLY_OVER),
        aplica("Arvore", OndeAplicar::PorCima { alvo: ALVO }),
    );
    so(
        &p.carrega(ids::MACROS_LIB_EXPORT),
        MacroRequest::Exporta {
            nome: "Arvore".into(),
        },
    );
    so(
        &p.carrega(ids::MACROS_LIB_DELETE),
        MacroRequest::ApagaDaBiblioteca {
            nome: "Arvore".into(),
        },
    );
    assert!(
        p.st.lib_escolhida.is_none(),
        "a apagada deixa de estar escolhida"
    );
}

/// Sem alvo compatível o «Aplicar por cima» não se pinta (a frase diz porquê) — nem com o nome
/// sem os bits, nem com os bits sem o nome.
#[test]
fn sem_alvo_o_aplicar_por_cima_nao_se_pinta() {
    for (de, alvo) in [(None, Some(ALVO)), (Some("Ceu".to_owned()), None)] {
        let mut p = Palco::novo(MacroPanelInfo {
            por_cima_de: de,
            por_cima_alvo: alvo,
            por_cima_recusa: Some("panel.macros.lib.apply_over_needs_target"),
            ..com_biblioteca()
        });
        let _ = p.carrega(ids::lib_row_id("Arvore"));
        p.pinta();
        assert!(p.rect(ids::MACROS_LIB_APPLY_OVER).is_none());
        assert!(p.rect(ids::MACROS_LIB_APPLY_NEW).is_some(), "controlo");
    }
}

#[test]
fn duplo_clique_renomeia_a_macro_e_a_escolha_segue_o_nome() {
    let mut p = Palco::novo(com_biblioteca());
    let linha = ids::lib_row_id("Arvore");
    assert!(p.rect(linha).is_some(), "a linha não se pintou");
    let _ = p
        .host
        .apply_panel_event::<MacrosPanel>(&mut p.st, WidgetEvent::DoubleClick(linha));
    assert!(p.st.lib_renomeando);
    p.pinta();
    assert!(
        p.rect(ids::MACROS_LIB_RENAME_INPUT).is_some(),
        "o campo não se pintou sobre a linha"
    );
    p.host.set_text(ids::MACROS_LIB_RENAME_INPUT, "Carvalho");
    let _ = p.host.apply_panel_event::<MacrosPanel>(
        &mut p.st,
        WidgetEvent::Submit(ids::MACROS_LIB_RENAME_INPUT),
    );
    so(
        &p.host.drained_actions(),
        MacroRequest::Renomeia {
            de: "Arvore".into(),
            para: "Carvalho".into(),
        },
    );
    assert_eq!(p.st.lib_escolhida.as_deref(), Some("Carvalho"));
    assert!(!p.st.lib_renomeando);
}

/// A escolhida que sai da lista (apagada noutra janela) cai: os verbos nunca apontam o que não
/// se vê.
#[test]
fn a_escolhida_que_sai_da_lista_cai() {
    let mut p = Palco::novo(com_biblioteca());
    let _ = p.carrega(ids::lib_row_id("Arvore"));
    let mut i = com_biblioteca();
    i.biblioteca.retain(|m| m.nome != "Arvore");
    set_current_macros(i);
    p.pinta();
    assert!(p.st.lib_escolhida.is_none());
    assert!(p.rect(ids::MACROS_LIB_EXPORT).is_none());
}
