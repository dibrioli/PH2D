//! ⭐ **O Inspector com a aba Input Map à frente** (onda 4, 7b): a acção aberta em cartões só de leitura, ou a frase
//! do que fazer; outra aba à frente ⇒ o Inspector de sempre.

use crate::o_inspector_em_cartoes_por_tipo::{chaves_da_lista, pinta};
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    CARD_EMPTY, CARD_IM_ACTION, CARD_IM_KEYS, CARD_IM_PICK, CARD_IM_READERS, CARD_IM_STICK,
    InputMapAction, InputMapPane, InspectorPanel, InspectorState, set_cards_on,
    set_current_input_map,
};
use ph2d_ui_testkit::MockPanelHost;

fn jump(stick: bool) -> InputMapPane {
    InputMapPane::Action(InputMapAction {
        name: "move_left".to_owned(),
        group: "Platform Player".to_owned(),
        readers: vec!["Platform Player".to_owned(), "Door Trigger Left".to_owned()],
        keys: vec![
            "A".to_owned(),
            "Left Arrow".to_owned(),
            "Left Stick".to_owned(),
        ],
        stick: stick.then(|| ("0.15".to_owned(), "0.50".to_owned())),
    })
}

fn chaves_com(pane: Option<InputMapPane>) -> Vec<ph2d_a11y::NodeId> {
    set_cards_on(true);
    set_current_input_map(pane);
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    let _ = pinta(&mut host, &mut st);
    let k = chaves_da_lista(&mut host);
    set_current_input_map(None);
    k
}

#[test]
fn a_aba_sem_accao_diz_o_que_fazer() {
    assert_eq!(chaves_com(Some(InputMapPane::Pick)), vec![CARD_IM_PICK]);
}

#[test]
fn a_aba_com_accao_mostra_os_cartoes_dela() {
    assert_eq!(
        chaves_com(Some(jump(true))),
        vec![CARD_IM_ACTION, CARD_IM_READERS, CARD_IM_KEYS, CARD_IM_STICK]
    );
    // Uma tecla sem manche não tem Dead/Press.
    assert!(!chaves_com(Some(jump(false))).contains(&CARD_IM_STICK));
}

/// Controlo: sem a aba à frente o Inspector de sempre (o cartão vazio da selecção), nenhum cartão da aba.
#[test]
fn outra_aba_a_frente_deixa_o_inspector_de_sempre() {
    let k = chaves_com(None);
    assert_eq!(k, vec![CARD_EMPTY]);
}

/// Nenhum texto dos cartões cortado, do piso real do painel a bem largo, nos dois modos de texto (o de fábrica é o
/// `CrispHeavyPlus`).
#[test]
fn nada_dos_cartoes_da_aba_e_cortado_nas_larguras_do_inspector() {
    use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
    use ph2d_editor_core::text_elide::elisao;
    use ph2d_editor_core::widget::card_censo;
    use ph2d_tokens::TextRendering;
    for largura in [
        WidgetStore::DOCK_W_MIN,
        ph2d_tokens::chrome::PANEL_MIN_W_PX,
        300.0,
        420.0,
    ] {
        for (pane, modo) in [InputMapPane::Pick, jump(true)].into_iter().flat_map(|p| {
            [TextRendering::Default, TextRendering::CrispHeavyPlus].map(|m| (p.clone(), m))
        }) {
            ph2d_text::set_active_text_rendering(modo);
            set_cards_on(true);
            set_current_input_map(Some(pane));
            let mut host = MockPanelHost::with_panel::<InspectorPanel>();
            let mut st = InspectorState::default();
            let janela = Rect::new(0.0, 0.0, 1600.0, 1000.0);
            let bandas = ChromeBands {
                right_dock_w: largura,
                ..ChromeBands::DEFAULT
            };
            let mut pinta_a = || {
                host.settle_section_folds();
                for _ in 0..30 {
                    let layout = HeroLayout::for_viewport_bands(
                        janela,
                        false,
                        bandas,
                        CenterSplit::None,
                        DockSides::BOTH,
                    );
                    let _ = ph2d_editor_core::aquecimento::com_passo_fixo(
                        ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                        || host.paint_with_layout::<InspectorPanel>(&mut st, layout, janela),
                    );
                }
            };
            pinta_a();
            let ((_, medidos), censo) = card_censo::grava(|| elisao::medindo(&mut pinta_a));
            let cortados: Vec<_> = medidos
                .iter()
                .filter(|m| !m.coube())
                .map(|m| m.texto.clone())
                .collect();
            assert!(cortados.is_empty(), "a {largura} {modo:?}: {cortados:?}");
            assert_eq!(censo.textos_cortados(), Vec::<String>::new(), "a {largura}");
            set_current_input_map(None);
        }
    }
}
