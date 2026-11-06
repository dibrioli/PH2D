//! O painel ocupa a vaga do Inspector só com a aba do som ativa, e o clique no cartão é dele.

use crate::Mesa;
use ph2d_app_sfxlab::Alvo;
use ph2d_editor_core::ids::SFX_PANEL;
use ph2d_editor_core::panel::Panel;
use ph2d_host::PointerKind;
use ph2d_panel_sfxlab::SfxLabPanel;
use ph2d_panel_sfxlab::tabela::CAMPOS;

const INSPECTOR: &str = "inspector";

fn vai_para_a_cena(m: &mut Mesa) {
    m.hero.documents.activate(None);
    m.quadro();
    m.quadro();
    assert!(m.hero.documents.scene_active(), "não saiu da aba");
}

fn volta(m: &mut Mesa, tab: ph2d_editor_core::doc_surface::SurfaceTab) {
    m.hero.documents.activate_surface(tab);
    m.quadro();
    m.quadro();
}

#[test]
fn com_a_aba_ativa_o_inspector_sai_e_ao_sair_volta_aberto() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    m.som_novo();
    assert!(
        m.hero.is_panel_visible(SfxLabPanel::ID),
        "o painel não entrou"
    );
    assert!(!m.hero.is_panel_visible(INSPECTOR), "o Inspector ficou");
    let tab = m.hero.documents.surface_active().expect("aba");
    vai_para_a_cena(&mut m);
    assert!(
        !m.hero.is_panel_visible(SfxLabPanel::ID),
        "o painel ficou na cena"
    );
    assert!(m.hero.is_panel_visible(INSPECTOR), "o Inspector não voltou");
    // E de novo: entrar e sair não gasta a memória.
    volta(&mut m, tab);
    assert!(!m.hero.is_panel_visible(INSPECTOR));
    vai_para_a_cena(&mut m);
    assert!(m.hero.is_panel_visible(INSPECTOR));
}

#[test]
fn o_inspector_que_o_artista_fechou_continua_fechado() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, false);
    m.quadro();
    m.som_novo();
    assert!(m.hero.is_panel_visible(SfxLabPanel::ID));
    vai_para_a_cena(&mut m);
    assert!(
        !m.hero.is_panel_visible(INSPECTOR),
        "abriu o que estava fechado"
    );
}

#[test]
fn fechar_a_aba_tambem_devolve_o_inspector() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    let id = m.som_novo();
    assert!(!m.hero.is_panel_visible(INSPECTOR));
    use ph2d_editor_core::doc_surface::DocumentSurface;
    m.docs().close(id);
    m.quadro();
    m.quadro();
    m.quadro();
    assert!(m.hero.is_panel_visible(INSPECTOR));
    assert!(!m.hero.is_panel_visible(SfxLabPanel::ID));
}

#[test]
fn o_clique_no_cartao_vai_ao_painel_e_nao_a_aba() {
    let mut m = Mesa::nova();
    let id = m.som_novo();
    m.espera(id);
    m.assenta();
    let painel = m
        .hero
        .store
        .panel_rect(SFX_PANEL)
        .expect("o painel tem rect");
    let area = m.docs().area().expect("a aba pintou");
    assert!(
        area.x + area.w <= painel.x + 0.5,
        "a aba {area:?} passa por baixo do painel {painel:?}"
    );
    let gain = CAMPOS
        .iter()
        .find(|c| c.chave == "sfxlab.card.gain_db")
        .expect("gain");
    let r = m.rect(gain.id).expect("o Gain pintado");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        m.hero.chrome_panel_at(x, y),
        Some(SFX_PANEL),
        "o painel não é o dono do ponto"
    );
    assert!(
        !m.ponteiro_da_shell(PointerKind::Down, x, y),
        "a aba tomou o clique do cartão"
    );
    // Dentro da aba, a linha continua sendo da aba.
    let c = m.receita(id).layers[0].id;
    let linha = m.docs().rect_de(id, Alvo::Linha(c)).expect("linha");
    assert!(
        m.hero
            .chrome_panel_at(linha.x + 4.0, linha.y + 4.0)
            .is_none()
    );
    // Fora da aba o painel não deixa rect: o clique ali volta a ser da cena.
    m.hero.documents.activate(None);
    m.quadro();
    m.quadro();
    assert_ne!(
        m.hero.chrome_panel_at(x, y),
        Some(SFX_PANEL),
        "rect velho na cena"
    );
}

#[test]
fn os_medidores_ficam_dentro_da_aba_a_direita_da_pilha() {
    let mut m = Mesa::nova();
    let id = m.som_novo();
    m.espera(id);
    m.assenta();
    let area = m.docs().area().expect("a aba pintou");
    let regua = m.docs().rect_de(id, Alvo::Regua).expect("régua");
    let sobra = area.x + area.w - (regua.x + regua.w);
    assert!(
        sobra >= ph2d_panel_sfxlab::medidor::LARGURA,
        "a pista come a coluna dos medidores: sobra {sobra}"
    );
}

/// Fechar o app com a aba ativa não pode gravar o Inspector como fechado.
#[test]
fn o_layout_grava_como_o_artista_deixou_nao_o_emprestimo() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    m.som_novo();
    assert!(!m.hero.is_panel_visible(INSPECTOR));
    assert!(
        m.hero.panel_no_arquivo(INSPECTOR),
        "gravaria o Inspector fechado"
    );
    assert!(
        !m.hero.panel_no_arquivo(SfxLabPanel::ID),
        "gravaria o painel aberto"
    );
    vai_para_a_cena(&mut m);
    assert!(
        m.hero.panel_emprestado.is_empty(),
        "o empréstimo não terminou"
    );
    assert!(m.hero.panel_no_arquivo(INSPECTOR));
}
