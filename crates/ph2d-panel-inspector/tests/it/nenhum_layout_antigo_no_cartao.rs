//! ⭐ **Portão: dentro de um cartão do Inspector não há layout antigo** (queixa do dono, 04/11, «várias
//! seções ainda guardam o layout antigo»). Dois lados da mesma lei:
//! - por gesto de pintura: as portas antigas anotam-se em `censo_cartao` quando o corpo de um cartão as
//!   chama; a cena inteira da sprite e o estado de cada ramo correm sob gravação;
//! - por censo de fonte: nenhuma função de `sections/` chama uma porta antiga sem perguntar se está
//!   num cartão. Cobre os componentes que a cena de teste não publica.

use crate::o_inspector_em_cartoes_por_tipo::{abre_tudo, chaves_da_lista, pinta};
use ph2d_editor_core::widget::censo_cartao::{Velho, grava};
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_cards_on};
use ph2d_ui_testkit::MockPanelHost;
use std::path::Path;

const PORTAS_ANTIGAS: [&str; 5] = [
    "paint_property_label(",
    "paint_slider_with_chip(",
    "paint_number_input_with_buffer(",
    "paint_swatch_or_mixed(",
    "paint_color_swatch(",
];

/// Quem ainda pode chamar uma porta antiga sem o ramo do cartão. Só desce; hoje, vazio.
const PENDENTES: [&str; 0] = [];

fn pinta_sob_gravacao(preparar: impl FnOnce()) -> Vec<Velho> {
    set_cards_on(true);
    crate::every_painted_id_is_reachable::publish_the_whole_sprite_scene();
    preparar();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut host);
    let _ = pinta(&mut host, &mut st);
    abre_tudo(&mut host);
    let (_, velhos) = grava(|| pinta(&mut host, &mut st));
    assert!(
        chaves_da_lista(&mut host).len() > 5,
        "a cena não pintou em cartões"
    );
    crate::every_painted_id_is_reachable::clear_the_scene();
    set_cards_on(false);
    velhos
}

/// A cena inteira da sprite, em cartões, nunca pinta um nome à esquerda, uma barra, uma caixa de
/// número com setas ou uma amostra sem código.
#[test]
fn a_cena_da_sprite_em_cartoes_nao_pinta_layout_antigo() {
    let velhos = pinta_sob_gravacao(|| {});
    assert!(velhos.is_empty(), "layout antigo num cartão: {velhos:#?}");
}

/// As linhas do Script (número, par, amostra): a cena da secção viva, em cartões.
#[test]
fn o_script_em_cartoes_nao_pinta_layout_antigo() {
    let velhos = pinta_sob_gravacao(|| {
        ph2d_panel_inspector::set_current_inspector_script(Some(
            crate::a_seccao_script_esta_viva::info(),
        ));
    });
    ph2d_panel_inspector::set_current_inspector_script(None);
    assert!(velhos.is_empty(), "layout antigo no Script: {velhos:#?}");
}

/// O censo acorda: fora de um cartão a porta antiga não conta, dentro conta. *Contraprova do censo.*
#[test]
fn o_censo_so_conta_dentro_de_um_cartao() {
    use ph2d_editor_core::widget::card_ink;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let cor = ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Text1, Theme::Dark);
    let (_, fora) = grava(|| {
        ph2d_editor_core::widget::paint_property_label(
            &mut text, &mut scene, "Nome", 0.0, 0.0, 12.0, 80.0, cor,
        );
    });
    assert!(fora.is_empty(), "fora de um cartão não conta: {fora:?}");
    let (_, dentro) = grava(|| {
        card_ink::com(card_ink::CardInk::of_list(Theme::Dark), || {
            ph2d_editor_core::widget::paint_property_label(
                &mut text, &mut scene, "Nome", 0.0, 0.0, 12.0, 80.0, cor,
            );
        });
    });
    assert_eq!(dentro, vec![Velho::NomeAEsquerda("Nome".to_owned())]);
}

fn funcoes_sem_ramo(fonte: &str) -> Vec<String> {
    let mut fora = Vec::new();
    let partes = ["\nfn ", "\npub fn ", "\npub(crate) fn ", "\npub(super) fn "]
        .iter()
        .flat_map(|sep| fonte.split(sep).skip(1));
    for parte in partes {
        let corpo = parte.split("\n}\n").next().unwrap_or(parte);
        let nome = corpo.split(['(', '<']).next().unwrap_or("").trim();
        let chama = PORTAS_ANTIGAS.iter().any(|p| corpo.contains(p));
        let pergunta = ["card_body(", "card_rows(", "cartao"]
            .iter()
            .any(|q| corpo.contains(q));
        if chama && !pergunta {
            fora.push(nome.to_owned());
        }
    }
    fora.sort();
    fora.dedup();
    fora
}

/// Toda secção do Inspector: quem chama uma porta antiga pergunta antes se está num cartão.
#[test]
fn nenhuma_seccao_chama_a_porta_antiga_sem_o_ramo_do_cartao() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sections");
    let mut achados = Vec::new();
    for e in std::fs::read_dir(&raiz).expect("sections/").flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let fonte = std::fs::read_to_string(&p).expect("ler");
        let arquivo = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned();
        for f in funcoes_sem_ramo(&fonte) {
            let nome = format!("{arquivo}::{f}");
            if !PENDENTES.contains(&nome.as_str()) {
                achados.push(nome);
            }
        }
    }
    achados.sort();
    assert!(
        achados.is_empty(),
        "porta antiga sem o ramo do cartão: {achados:#?}"
    );
}

/// O botão no cartão é o do cartão: a largura do corpo, na altura de um campo.
#[test]
fn o_botao_no_cartao_tem_a_largura_do_corpo_e_a_altura_de_um_campo() {
    set_cards_on(true);
    crate::every_painted_id_is_reachable::publish_the_whole_sprite_scene();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    abre_tudo(&mut host);
    let _ = pinta(&mut host, &mut st);
    abre_tudo(&mut host);
    let r = pinta(&mut host, &mut st);
    crate::every_painted_id_is_reachable::clear_the_scene();
    set_cards_on(false);
    let de = |id| {
        r.iter()
            .rev()
            .find(|(i, q)| *i == id && q.w > 0.0)
            .map(|(_, q)| *q)
            .expect("pintado")
    };
    let botao = de(ph2d_panel_inspector::ids::INSP_SPRITE_CORNER_EQUALIZE);
    let campo = de(ph2d_panel_inspector::ids::INSP_SPRITE_OPACITY_CHIP);
    assert!(
        (botao.x - campo.x).abs() < 0.5
            && (botao.w - campo.w).abs() < 0.5
            && (botao.h - campo.h).abs() < 0.5,
        "botão {botao:?} × campo {campo:?}"
    );
}
