//! O painel entra como aba ao lado do Inspector com a aba do som ativa (report do dono de 08/09), e o clique no cartão é dele.

use crate::Mesa;
use ph2d_app_sfxlab::Alvo;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::SFX_PANEL;
use ph2d_editor_core::panel::Panel;
use ph2d_editor_core::screens::hero::slot_tabs;
use ph2d_editor_core::screens::slot::Slot;
use ph2d_host::PointerKind;
use ph2d_panel_sfxlab::SfxLabPanel;
use ph2d_panel_sfxlab::tabela::CAMPOS;

const INSPECTOR: &str = ph2d_panel_inspector::InspectorPanel::ID;
const INSPECTOR_PANEL: NodeId = ph2d_panel_inspector::InspectorPanel::NODE_ID;

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

/// Quem está na fileira da vaga e quem está à frente: a resposta do mecanismo das abas, não um booleano nosso.
fn fila(m: &Mesa) -> (Vec<&'static str>, Option<NodeId>) {
    (
        slot_tabs::occupants(&m.hero, Slot::RightTop)
            .iter()
            .map(|o| o.id)
            .collect(),
        slot_tabs::chosen(&m.hero, Slot::RightTop),
    )
}

fn clica_na_aba_do_painel(m: &mut Mesa, painel: NodeId) {
    let r = m
        .rect(slot_tabs::tab_node_id(painel))
        .expect("a aba do painel não foi pintada");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    m.mao(PointerKind::Down, x, y);
    m.mao(PointerKind::Up, x, y);
    m.quadro();
    m.quadro();
}

#[test]
fn o_som_entra_como_aba_ao_lado_do_inspector() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    m.som_novo();
    m.quadro();
    m.quadro();
    assert!(
        m.hero.is_panel_visible(INSPECTOR),
        "o Inspector saiu da vaga"
    );
    assert!(
        m.hero.is_panel_visible(SfxLabPanel::ID),
        "o painel não entrou"
    );
    let (ocupantes, frente) = fila(&m);
    assert!(
        ocupantes.contains(&INSPECTOR) && ocupantes.contains(&SfxLabPanel::ID),
        "os dois não dividem a vaga: {ocupantes:?}"
    );
    assert_eq!(frente, Some(SFX_PANEL), "o som abriu escondido");
    // As duas abas pintadas, e só um publica rect.
    assert!(m.rect(slot_tabs::tab_node_id(INSPECTOR_PANEL)).is_some());
    assert!(m.rect(slot_tabs::tab_node_id(SFX_PANEL)).is_some());
    assert!(m.hero.store.panel_rect(SFX_PANEL).is_some());
    assert!(m.hero.store.panel_rect(INSPECTOR_PANEL).is_none());

    // O clique na aba do Inspector troca quem desenha, intacto, e a volta também.
    clica_na_aba_do_painel(&mut m, INSPECTOR_PANEL);
    assert_eq!(fila(&m).1, Some(INSPECTOR_PANEL), "o clique não trocou");
    assert!(m.hero.store.panel_rect(INSPECTOR_PANEL).is_some());
    assert!(
        m.hero.store.panel_rect(SFX_PANEL).is_none(),
        "o painel de trás deixou rect"
    );
    assert!(
        m.hero.documents.surface_active().is_some(),
        "a aba do som fechou"
    );
    clica_na_aba_do_painel(&mut m, SFX_PANEL);
    assert_eq!(fila(&m).1, Some(SFX_PANEL));
}

#[test]
fn sair_da_aba_de_som_devolve_a_frente_ao_inspector() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    m.som_novo();
    m.quadro();
    let tab = m.hero.documents.surface_active().expect("aba");
    vai_para_a_cena(&mut m);
    assert!(
        !m.hero.is_panel_visible(SfxLabPanel::ID),
        "o painel ficou na cena"
    );
    assert_eq!(fila(&m).0, vec![INSPECTOR], "alguém ficou para trás");
    assert_eq!(fila(&m).1, Some(INSPECTOR_PANEL));
    // Voltar: de novo à frente; sair de novo: o mesmo. Entrar e sair não gasta nada.
    volta(&mut m, tab);
    assert_eq!(fila(&m).1, Some(SFX_PANEL), "o som voltou escondido");
    vai_para_a_cena(&mut m);
    assert_eq!(fila(&m).1, Some(INSPECTOR_PANEL));
}

#[test]
fn o_inspector_que_o_artista_fechou_continua_fechado() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, false);
    m.quadro();
    m.som_novo();
    assert!(m.hero.is_panel_visible(SfxLabPanel::ID));
    assert!(
        !m.hero.is_panel_visible(INSPECTOR),
        "abriu o que estava fechado"
    );
    vai_para_a_cena(&mut m);
    assert!(
        !m.hero.is_panel_visible(INSPECTOR),
        "abriu o que estava fechado"
    );
}

#[test]
fn fechar_a_aba_tira_o_painel_e_o_inspector_fica() {
    let mut m = Mesa::nova();
    m.hero.panel_visibility.insert(INSPECTOR, true);
    m.quadro();
    let id = m.som_novo();
    use ph2d_editor_core::doc_surface::DocumentSurface;
    m.docs().close(id);
    m.quadro();
    m.quadro();
    m.quadro();
    assert!(m.hero.is_panel_visible(INSPECTOR));
    assert!(!m.hero.is_panel_visible(SfxLabPanel::ID));
    assert_eq!(fila(&m).1, Some(INSPECTOR_PANEL));
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
