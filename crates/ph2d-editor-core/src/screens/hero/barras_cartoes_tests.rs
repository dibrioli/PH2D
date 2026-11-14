//! As barras do topo em cartões (`docs/CartoesUI/15_CHROME_CENSO.md`, M3 a M7): os tons novos entram, os
//! de antes saem, e nenhum alvo muda. `PH2D_CARDS=0` continua a pintar os de antes.

use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, resolve};
use crate::screens::HeroLayout;
use crate::widget::cromo_barra::Barra;
use crate::widget::lista_flutuante::force_cromo_on_this_thread;
use crate::widget::{
    RailAxis, StatusBar, StatusSegment, ToolRail, ToolRailEntry, entry_rects, paint_status_bar,
    paint_tool_rail_axis,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

const TEMA: Theme = Theme::Dark;

fn janela() -> Rect {
    Rect::new(0.0, 0.0, 1400.0, 900.0)
}

fn vezes(scene: &VectorScene, token: ColorToken) -> usize {
    let mut uma = VectorScene::new();
    let r = Rect::new(0.0, 0.0, 1.0, 1.0);
    fill_rounded_rect(&mut uma, r, 0.0, resolve(token, TEMA));
    let cor = uma.inner().encoding().draw_data.clone();
    scene
        .inner()
        .encoding()
        .draw_data
        .windows(cor.len())
        .filter(|w| *w == cor.as_slice())
        .count()
}

fn nos_dois_modos<T>(mut f: impl FnMut() -> T) -> (T, T) {
    force_cromo_on_this_thread(Some(true));
    let novo = f();
    force_cromo_on_this_thread(Some(false));
    let antigo = f();
    force_cromo_on_this_thread(None);
    (novo, antigo)
}

fn menu_bar() -> (VectorScene, Vec<(NodeId, Rect)>) {
    let mut text = TextSystem::without_system_fonts();
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    crate::screens::hero::menu_bar::paint_menu_bar(
        &HeroLayout::for_viewport(janela()),
        &mut scene,
        &mut text,
        TEMA,
        &mut hits,
        &WidgetStore::default(),
        &crate::motion::UiMotion::default(),
        &crate::documents::Documents::default(),
        crate::game_stage::GameRun::default(),
    );
    let alvos = hits.iter_registrations().collect();
    (scene, alvos)
}

/// M3: as abas de modo e de documento são segmentos numa pílula, sem bloco de acento e sem mover alvo.
#[test]
fn as_abas_do_topo_sao_segmentos_sem_acento() {
    let ((novo, alvos_novos), (antigo, alvos_antigos)) = nos_dois_modos(menu_bar);
    assert_eq!(alvos_novos, alvos_antigos);
    assert!(alvos_novos.len() > 6, "sem abas não prova nada");
    // A aba de documento, o campo «Find anything» (versão C, 12/11) e os botões do jogo: três
    // fundos de campo. ⚠️ As abas de layout (Scene·Draw·…) saíram da barra por ordem do dono (12/11).
    assert_eq!(vezes(&novo, ColorToken::CardField), 3);
    // Um escolhido: o documento activo (o layout escolhido saiu com as abas de layout).
    assert_eq!(vezes(&novo, ColorToken::KanbanCardRaised), 1);
    assert_eq!(vezes(&novo, ColorToken::AccentSoft), 0);
    assert_eq!(vezes(&novo, ColorToken::KanbanList), 1, "o chão da barra");
    assert_eq!(vezes(&novo, ColorToken::Bg1), 0);
    // No antigo, o acento só no documento activo (o do layout saiu com as abas, 12/11).
    assert_eq!(vezes(&antigo, ColorToken::AccentSoft), 1);
    assert_eq!(vezes(&antigo, ColorToken::Bg1), 1);
}

fn faixa_de_abas() -> (VectorScene, Vec<(NodeId, Rect)>) {
    use crate::screens::hero::slot_tabs::{Occupant, paint_slot_tabs};
    let ocupante = |node, title| Occupant {
        id: "x",
        node: NodeId(node),
        title: crate::panel::TextKey::new(title),
        icon: crate::icons::IconId::Inspector,
    };
    let occ = [ocupante(1, "Inspector"), ocupante(2, "Vector")];
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    paint_slot_tabs(
        Rect::new(0.0, 0.0, 300.0, 22.0),
        crate::screens::slot::Slot::RightTop,
        &occ,
        Some(NodeId(1)),
        &mut scene,
        &mut TextSystem::without_system_fonts(),
        TEMA,
        &mut hits,
        &WidgetStore::default(),
    );
    let alvos = hits.iter_registrations().collect();
    (scene, alvos)
}

/// M4: a fila de encaixe deixa o chão preto e a divisória; a escolhida é um segmento.
#[test]
fn a_fila_de_abas_de_encaixe_nao_tem_chao_preto_nem_divisoria() {
    let ((novo, alvos_novos), (antigo, alvos_antigos)) = nos_dois_modos(faixa_de_abas);
    assert_eq!(alvos_novos, alvos_antigos);
    assert_eq!(vezes(&novo, ColorToken::WindowGround), 0);
    assert_eq!(vezes(&novo, ColorToken::KanbanList), 1);
    assert_eq!(vezes(&novo, ColorToken::KanbanCardRaised), 1);
    assert_eq!(vezes(&novo, ColorToken::PanelBg), 0);
    assert_eq!(vezes(&antigo, ColorToken::WindowGround), 1);
    assert_eq!(vezes(&antigo, ColorToken::PanelBg), 1);
}

/// M4: a aba diz o nome inteiro ou só o ícone, nunca o nome cortado, e o balão guarda o nome.
#[test]
fn a_aba_em_cartoes_mostra_o_nome_inteiro_ou_so_o_icone() {
    use crate::screens::hero::slot_tabs_face::{face, natural_w};
    force_cromo_on_this_thread(Some(true));
    let mut text = TextSystem::without_system_fonts();
    let titulo = "Color Equalization";
    let natural = natural_w(titulo, &mut text);
    let (mut com_nome, mut so_icone) = (0, 0);
    let mut w = ph2d_tokens::ROW_H_PX;
    while w <= natural + 0.5 {
        let f = face(Rect::new(0.0, 0.0, w, 22.0), titulo, &mut text);
        match f.label {
            Some((s, _, _)) => {
                assert_eq!(s, titulo, "a {w} px");
                com_nome += 1;
            }
            None => so_icone += 1,
        }
        w += 1.0;
    }
    force_cromo_on_this_thread(None);
    assert!(com_nome > 0 && so_icone > 0, "{com_nome} × {so_icone}");
}

/// M4: quem não cabe recolhe ao ícone, da última para a primeira, e a escolhida fica com o nome.
#[test]
fn a_escolhida_e_a_ultima_a_recolher_ao_icone() {
    use crate::screens::hero::slot_tabs::Occupant;
    use crate::screens::hero::slot_tabs::tab_plan;
    let ocupante = |node, title| Occupant {
        id: "x",
        node: NodeId(node),
        title: crate::panel::TextKey::new(title),
        icon: crate::icons::IconId::Inspector,
    };
    // A escolhida no FIM: a recolha começa pela última, e ela tem de passar à frente.
    let occ = [
        ocupante(1, "Inspector"),
        ocupante(2, "Vector"),
        ocupante(3, "Color Equalization"),
    ];
    let mut text = TextSystem::without_system_fonts();
    let naturais: Vec<f32> = occ
        .iter()
        .map(|o| crate::screens::hero::slot_tabs_face::natural_w(o.title.tr(), &mut text))
        .collect();
    // O piso de uma aba é a altura da fila (`TAB_BAR_H`, a da fila de ferramentas desde a onda 7).
    let piso = crate::screens::hero::slot_tabs::TAB_BAR_H;
    let aperto = naturais[2] + piso * 2.0 + ph2d_tokens::Spacing::Xs.px() * 2.0;
    force_cromo_on_this_thread(Some(true));
    let bar = Rect::new(0.0, 0.0, aperto, piso);
    let plano = tab_plan(&occ, Some(NodeId(3)), bar, &mut text).expect("cabe sem setas");
    force_cromo_on_this_thread(None);
    assert_eq!(plano.hidden_after, 0);
    assert!(
        (plano.widths[2] - naturais[2]).abs() < 0.01,
        "{:?}",
        plano.widths
    );
    assert_eq!(&plano.widths[..2], &[piso, piso]);
}

fn fila() -> (ToolRail, Rect) {
    let rail = ToolRail::new(
        NodeId(9),
        "fila",
        vec![
            ToolRailEntry::icon(NodeId(10), "Move", crate::icons::IconId::Move)
                .with_sub("MOVE")
                .active(),
            ToolRailEntry::icon(NodeId(11), "Rot", crate::icons::IconId::Reset).with_sub("ROT"),
            ToolRailEntry::Divider,
            ToolRailEntry::compound(NodeId(12), "Stroke", "Stroke ▾", "LINE"),
            ToolRailEntry::number(NodeId(13), "W", "W", None),
            ToolRailEntry::swatch(NodeId(14), "Fill", [1, 2, 3, 255]).with_sub("FILL"),
        ],
    );
    (rail, Rect::new(0.0, 4.0, 900.0, 50.0))
}

fn pinta_fila() -> (VectorScene, Vec<(Option<NodeId>, Rect)>) {
    let (rail, r) = fila();
    let mut text = TextSystem::without_system_fonts();
    let store = WidgetStore::default();
    let mut scene = VectorScene::new();
    paint_tool_rail_axis(
        &rail,
        r,
        &mut scene,
        &mut text,
        TEMA,
        &store,
        &|_| None,
        false,
        RailAxis::Horizontal,
    );
    let slots = entry_rects(&rail, r, store.rail_button_size(), RailAxis::Horizontal)
        .into_iter()
        .map(|s| (s.id, s.rect))
        .collect();
    (scene, slots)
}

/// M5/M6: o chip em repouso não tem caixa nem filete, o campo é a caixa do cartão, o escolhido sobe.
#[test]
fn a_fila_de_chips_usa_os_tons_do_cartao_e_nao_move_alvo() {
    let ((novo, slots_novos), (antigo, slots_antigos)) = nos_dois_modos(pinta_fila);
    assert_eq!(slots_novos, slots_antigos);
    for token in [
        ColorToken::BgElev,
        ColorToken::AccentSoft,
        ColorToken::Border,
        ColorToken::BorderEmph,
        ColorToken::Bg3,
    ] {
        assert_eq!(vezes(&novo, token), 0, "{token:?} ficou na fila em cartões");
    }
    // O campo de número e o de opção; a ferramenta ativa.
    assert_eq!(vezes(&novo, ColorToken::CardField), 2);
    assert_eq!(vezes(&novo, ColorToken::KanbanCardRaised), 1);
    assert_eq!(vezes(&novo, ColorToken::CardFieldHover), 1, "a divisória");
    assert!(vezes(&antigo, ColorToken::BgElev) > 0);
}

/// M5: as setas do campo só aparecem sob o rato; o alvo delas (o stepper) não depende da pintura.
#[test]
fn as_setas_do_campo_so_aparecem_quando_pedidas() {
    let caminhos = |setas| {
        let mut scene = VectorScene::new();
        crate::widget::paint_number_chip_do_cartao(
            Rect::new(0.0, 0.0, 60.0, 28.0),
            crate::widget::TextInputState::Normal,
            3.0,
            None,
            None,
            false,
            0,
            None,
            &mut scene,
            &mut TextSystem::without_system_fonts(),
            TEMA,
            setas,
        );
        scene.inner().encoding().n_paths
    };
    assert!(caminhos(true) > caminhos(false));
}

/// M6: a barra que flutua perde a moldura; o botão aceso lê-se no acento, o outro na família.
#[test]
fn a_barra_flutuante_veste_a_lista_e_o_botao_o_chip() {
    use crate::screens::hero::float_bar::{Btn, paint_button, paint_panel};
    let pinta = || {
        let r = Rect::new(10.0, 10.0, 32.0, 32.0);
        let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
        paint_panel(&mut scene, [r], TEMA);
        let store = WidgetStore::default();
        let id = NodeId(5);
        let btn = |on| Btn {
            id,
            rect: r,
            on,
            tint: ColorToken::Text2,
        };
        let aceso = paint_button(&mut scene, &mut hits, &store, TEMA, btn(true));
        let solto = paint_button(&mut scene, &mut hits, &store, TEMA, btn(false));
        (scene, hits.rect_for(id), aceso, solto)
    };
    let ((novo, alvo_novo, aceso, solto), (antigo, alvo_antigo, _, _)) = nos_dois_modos(pinta);
    assert_eq!(alvo_novo, alvo_antigo);
    assert_eq!(vezes(&novo, ColorToken::BgElev), 0);
    assert_eq!(
        vezes(&novo, ColorToken::KanbanCardRaised),
        1,
        "o botão aceso"
    );
    assert!(vezes(&antigo, ColorToken::BgElev) > 0);
    assert_eq!(aceso, resolve(ColorToken::KanbanBlueBold, TEMA));
    assert_eq!(solto, resolve(ColorToken::Text2, TEMA));
}

/// M7: o HUD é a superfície da lista, sem filete, com os textos nos tons do cartão.
#[test]
fn o_hud_em_cartoes_nao_tem_filete() {
    let pinta = || {
        let bar = StatusBar::new(
            NodeId(1),
            "hud",
            vec![
                StatusSegment::new("EDIT".to_owned()),
                StatusSegment::new("60 fps".to_owned()),
            ],
        );
        let mut scene = VectorScene::new();
        let r = Rect::new(0.0, 0.0, 300.0, 32.0);
        let mut text = TextSystem::without_system_fonts();
        paint_status_bar(&bar, r, &mut scene, &mut text, TEMA);
        scene
    };
    let (novo, antigo) = nos_dois_modos(pinta);
    assert_eq!(vezes(&novo, ColorToken::BgElev), 0);
    assert_eq!(vezes(&novo, ColorToken::Border), 0);
    assert_eq!(vezes(&novo, ColorToken::KanbanCardRaised), 1);
    assert_eq!(
        vezes(&novo, ColorToken::CardField),
        1,
        "a divisória entre as duas leituras"
    );
    assert_eq!(vezes(&antigo, ColorToken::BgElev), 1);
}

/// O chão da faixa do topo atravessa a janela, só com a fila em cima e só em cartões.
#[test]
fn a_faixa_do_topo_cobre_as_colunas_so_com_a_fila_em_cima() {
    let mut layout = HeroLayout::for_viewport(janela());
    layout.tool_bar = Rect::new(
        layout.top_bar.x,
        layout.top_bar.y + layout.top_bar.h,
        layout.top_bar.w,
        54.0,
    );
    let pinta = |layout: &HeroLayout| {
        let mut scene = VectorScene::new();
        crate::screens::hero::canvas::paint_top_band(layout, &mut scene, TEMA);
        scene
    };
    let (novo, antigo) = nos_dois_modos(|| pinta(&layout));
    assert_eq!(vezes(&novo, ColorToken::KanbanList), 1);
    assert_eq!(vezes(&antigo, ColorToken::KanbanList), 0);
    layout.tool_bar.y += 500.0; // a fila em baixo: a faixa cobriria as listas
    force_cromo_on_this_thread(Some(true));
    assert_eq!(vezes(&pinta(&layout), ColorToken::KanbanList), 0);
    force_cromo_on_this_thread(None);
}

#[test]
fn a_barra_existe_so_em_cartoes() {
    let (novo, antigo) = nos_dois_modos(|| Barra::de(TEMA).is_some());
    assert!(novo);
    assert!(!antigo);
}
