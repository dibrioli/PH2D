//! A pele do menu troca com `PH2D_CARDS`; os alvos não (`docs/CartoesUI/15_CHROME_CENSO.md`, M1).

use super::super::context_menu_overlay::paint_context_menu_overlay;
use crate::ids;
use crate::interaction::{ContextMenuKind as K, ContextMenuRequest, HitIndex, WidgetStore};
use crate::motion::UiMotion;
use crate::paint::{fill_rounded_rect, resolve};
use crate::project::ProjectSettings;
use crate::widget::lista_flutuante::force_cromo_on_this_thread;
use crate::widget::{Tooltip, paint_tooltip};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

const TEMA: Theme = Theme::Dark;

fn janela() -> Rect {
    Rect::new(0.0, 0.0, 1930.0, 1040.0)
}

fn menu(kind: K, hot: Option<NodeId>, on: bool) -> (VectorScene, HitIndex) {
    force_cromo_on_this_thread(Some(on));
    let mut store = WidgetStore::default();
    store.open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 60.0,
        kind,
    });
    store.set_hot(hot);
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    paint_context_menu_overlay(
        &mut scene,
        &mut TextSystem::without_system_fonts(),
        TEMA,
        &mut hits,
        &store,
        &ProjectSettings::default(),
        &UiMotion::default(),
        &crate::luz_edits::PreferenciaDaLuz::default(),
        janela(),
    );
    force_cromo_on_this_thread(None);
    (scene, hits)
}

/// Quantos preenchimentos desta cor a cena traz.
fn vezes_da_cor(scene: &VectorScene, token: ColorToken) -> usize {
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

fn pinta_a_cor(scene: &VectorScene, token: ColorToken) -> bool {
    vezes_da_cor(scene, token) > 0
}

#[test]
fn cada_linha_de_cada_menu_regista_o_mesmo_alvo_nos_dois_modos() {
    let mut com_alvos = 0;
    for kind in [
        K::HierarchyRow { row: NodeId(1) },
        K::MenuBarFile,
        K::MenuBarView,
        K::SettingsMenu,
        K::SceneList,
        K::ThemeSelector,
    ] {
        let rects = |on| {
            let (_, hits) = menu(kind, None, on);
            hits.iter_registrations().collect::<Vec<_>>()
        };
        let (novos, antigos) = (rects(true), rects(false));
        assert_eq!(novos, antigos, "{kind:?}: o cromo moveu um alvo");
        com_alvos += usize::from(!novos.is_empty());
    }
    assert_eq!(com_alvos, 6, "um menu sem alvos não prova nada");
}

#[test]
fn o_menu_em_cartoes_pinta_a_superficie_da_lista_e_o_de_antes_a_antiga() {
    let kind = K::HierarchyRow { row: NodeId(1) };
    let (novo, _) = menu(kind, None, true);
    assert!(pinta_a_cor(&novo, ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&novo, ColorToken::BgElev));
    let (antigo, _) = menu(kind, None, false);
    assert!(pinta_a_cor(&antigo, ColorToken::BgElev));
    assert!(!pinta_a_cor(&antigo, ColorToken::KanbanCardRaised));
}

#[test]
fn a_linha_sob_o_rato_acende_no_tom_do_campo_do_cartao() {
    let kind = K::HierarchyRow { row: NodeId(1) };
    let hot = Some(ids::CTX_MENU_HIER_DUPLICATE);
    assert!(!pinta_a_cor(
        &menu(kind, None, true).0,
        ColorToken::CardFieldHover
    ));
    assert!(pinta_a_cor(
        &menu(kind, hot, true).0,
        ColorToken::CardFieldHover
    ));
    assert!(pinta_a_cor(&menu(kind, hot, false).0, ColorToken::Bg2));
}

#[test]
fn a_lista_de_cenas_e_o_transbordo_vestem_a_mesma_superficie() {
    let (cenas, _) = menu(K::SceneList, None, true);
    assert!(pinta_a_cor(&cenas, ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&cenas, ColorToken::BgElev));
}

#[test]
fn as_caixas_internas_vestem_a_superficie_sem_mover_alvo() {
    for kind in [
        K::NewImageDialog,
        K::NewSpriteDialog,
        K::SheetSizeDialog,
        K::PixelColorModeDialog,
        K::PixelEditDialog,
        K::RenamePaletteDialog,
    ] {
        let (novo, alvos_novos) = menu(kind, None, true);
        let (antigo, alvos_antigos) = menu(kind, None, false);
        assert!(pinta_a_cor(&novo, ColorToken::KanbanCardRaised), "{kind:?}");
        assert!(!pinta_a_cor(&novo, ColorToken::BgElev), "{kind:?}");
        assert!(pinta_a_cor(&antigo, ColorToken::BgElev), "{kind:?}");
        let (novos, antigos): (Vec<_>, Vec<_>) = (
            alvos_novos.iter_registrations().collect(),
            alvos_antigos.iter_registrations().collect(),
        );
        if kind == K::NewSpriteDialog {
            // O corpo passou a linhas de cartão: os controlos são os mesmos, noutro sítio.
            let so_ids = |v: &[(NodeId, Rect)]| {
                let mut ids: Vec<_> = v.iter().map(|(id, _)| id.0).collect();
                ids.sort_unstable();
                ids
            };
            assert_eq!(so_ids(&novos), so_ids(&antigos));
        } else {
            assert_eq!(novos, antigos, "{kind:?}");
        }
    }
}

#[test]
fn o_transbordo_da_fila_veste_a_superficie_sem_mover_alvo() {
    let pinta = |on| {
        force_cromo_on_this_thread(Some(on));
        let mut store = WidgetStore::default();
        store.set_tool_overflow(vec![crate::widget::ToolRailEntry::Glyph {
            id: NodeId(77),
            label: "Tool".to_string(),
            path: ph2d_vector::BezPath::new(),
            active: false,
            sub: String::new(),
        }]);
        store.open_context_menu(ContextMenuRequest {
            x: 40.0,
            y: 60.0,
            kind: K::ToolBarOverflow,
        });
        let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
        paint_context_menu_overlay(
            &mut scene,
            &mut TextSystem::without_system_fonts(),
            TEMA,
            &mut hits,
            &store,
            &ProjectSettings::default(),
            &UiMotion::default(),
            &crate::luz_edits::PreferenciaDaLuz::default(),
            janela(),
        );
        force_cromo_on_this_thread(None);
        (scene, hits.iter_registrations().collect::<Vec<_>>())
    };
    let ((novo, alvos_novos), (antigo, alvos_antigos)) = (pinta(true), pinta(false));
    assert!(pinta_a_cor(&novo, ColorToken::KanbanCardRaised));
    assert_eq!(vezes_da_cor(&novo, ColorToken::BgElev), 0);
    assert!(vezes_da_cor(&antigo, ColorToken::BgElev) > 0);
    assert!(!alvos_novos.is_empty(), "um transbordo sem alvos não prova nada");
    assert_eq!(alvos_novos, alvos_antigos);
}

/// O New Sprite em cartões: mesmos ids, unidade no rótulo, e o Width mede a caixa do cartão.
#[test]
fn o_new_sprite_em_cartoes_regista_cada_controlo_e_leva_a_unidade_no_rotulo() {
    let mut store = WidgetStore::default();
    store.open_new_sprite_dialog();
    force_cromo_on_this_thread(Some(true));
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    paint_context_menu_overlay(
        &mut scene,
        &mut TextSystem::without_system_fonts(),
        TEMA,
        &mut hits,
        &store,
        &ProjectSettings::default(),
        &UiMotion::default(),
        &crate::luz_edits::PreferenciaDaLuz::default(),
        janela(),
    );
    force_cromo_on_this_thread(None);
    let modos = ids::NEW_SPRITE_MODES.iter().map(|(_, id, _)| *id);
    let fundos = ids::NEW_SPRITE_BGS.iter().map(|(_, id, _)| *id);
    for id in modos
        .chain(fundos)
        .chain(ids::NEW_SPRITE_FIELDS)
        .chain([ids::NEW_SPRITE_OK, ids::NEW_SPRITE_CANCEL])
    {
        assert!(hits.rect_for(id).is_some(), "{id:?} sem alvo");
    }
    // A caixa e os dois segmentos escolhidos: sem a moldura, os segmentos sozinhos davam verde.
    assert_eq!(vezes_da_cor(&scene, ColorToken::KanbanCardRaised), 3);
    let largura = hits.rect_for(ids::NEW_SPRITE_WIDTH).expect("Width");
    assert_eq!(largura.h, ph2d_card_stack::metrics::FIELD_H);
    for (id, nome) in [
        (ids::NEW_SPRITE_WIDTH, "Width (px)"),
        (ids::NEW_SPRITE_HEIGHT, "Height (px)"),
    ] {
        let dica = crate::panel::cards::dicas::field_hint(&store, id).expect("campo de cartão");
        assert!(dica.contains(nome), "{dica}");
    }
}

/// O texto da dica não pode cortar-se na caixa que a mede: o defeito «Add ent…» (foto m1_dica).
#[test]
fn a_dica_em_cartoes_mostra_o_texto_inteiro_na_largura_do_texto() {
    let mut text = TextSystem::without_system_fonts();
    let msg = "Add entity";
    let px = ph2d_tokens::TypeToken::Sm.px();
    let w = text.prefix_width(msg, px);
    assert!(w > 0.0, "sem fonte a largura não prova nada");
    let glifos = |rect_w: f32, text: &mut TextSystem| {
        force_cromo_on_this_thread(Some(true));
        let mut scene = VectorScene::new();
        paint_tooltip(
            &Tooltip::new(NodeId(1), msg),
            Rect::new(0.0, 0.0, rect_w, 24.0),
            &mut scene,
            text,
            TEMA,
        );
        force_cromo_on_this_thread(None);
        scene.inner().encoding().resources.glyphs.len()
    };
    let inteiro = glifos(w * 4.0, &mut text);
    assert!(inteiro > 0);
    assert_eq!(glifos(w, &mut text), inteiro, "a dica cortou o próprio texto");
}

/// A trilha do Fill começa depois do rótulo em cartões, e o modo de antes fica como era.
#[test]
fn o_fill_em_cartoes_tem_rotulo_e_a_trilha_nos_tons_do_cartao() {
    let pinta = |on| {
        force_cromo_on_this_thread(Some(on));
        let mut store = WidgetStore::default();
        store.open_fill_modal(100.0, 100.0, 0.5);
        let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
        crate::screens::hero::chrome::paint_fill_adjust_modal(
            &mut scene,
            &mut TextSystem::without_system_fonts(),
            TEMA,
            &mut hits,
            &store,
            &crate::tether::Tether::default(),
            janela(),
        );
        force_cromo_on_this_thread(None);
        let r = hits.rect_for(ids::PAINTER_FILL_MODAL_SLIDER).expect("trilha");
        (scene, r)
    };
    let ((novo, r_novo), (antigo, r_antigo)) = (pinta(true), pinta(false));
    assert!(r_novo.x > r_antigo.x, "o rótulo mora à esquerda da trilha");
    assert!(
        (r_novo.x + r_novo.w - (r_antigo.x + r_antigo.w)).abs() < 0.01,
        "a trilha acaba no mesmo sítio"
    );
    assert!(pinta_a_cor(&novo, ColorToken::CardField));
    assert!(!pinta_a_cor(&novo, ColorToken::Bg2));
    assert!(pinta_a_cor(&antigo, ColorToken::Bg2));
}

/// A barra de menus inteira num quadro, com File aberto.
fn barra(on: bool) -> (VectorScene, HitIndex) {
    force_cromo_on_this_thread(Some(on));
    let mut text = TextSystem::without_system_fonts();
    let mut store = WidgetStore::default();
    store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 24.0,
        kind: K::MenuBarFile,
    });
    let layout = crate::screens::HeroLayout::for_viewport(janela());
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
    crate::screens::hero::menu_bar::paint_menu_bar(
        &layout,
        &mut scene,
        &mut text,
        TEMA,
        &mut hits,
        &store,
        &UiMotion::default(),
        &crate::documents::Documents::default(),
    );
    force_cromo_on_this_thread(None);
    (scene, hits)
}

#[test]
fn o_titulo_do_menu_aberto_acende_no_tom_do_cartao_sem_mover_alvo() {
    let (novo, alvos_novos) = barra(true);
    assert!(pinta_a_cor(&novo, ColorToken::CardFieldHover));
    let (antigo, alvos_antigos) = barra(false);
    assert_eq!(vezes_da_cor(&novo, ColorToken::AccentSoft), 0);
    assert!(vezes_da_cor(&antigo, ColorToken::AccentSoft) > 0);
    assert_eq!(
        alvos_novos.iter_registrations().collect::<Vec<_>>(),
        alvos_antigos.iter_registrations().collect::<Vec<_>>(),
    );
}

#[test]
fn a_dica_em_cartoes_nao_e_a_antiga() {
    let dica = |on| {
        force_cromo_on_this_thread(Some(on));
        let mut scene = VectorScene::new();
        paint_tooltip(
            &Tooltip::new(NodeId(1), "Save"),
            Rect::new(0.0, 0.0, 120.0, 24.0),
            &mut scene,
            &mut TextSystem::without_system_fonts(),
            TEMA,
        );
        force_cromo_on_this_thread(None);
        scene
    };
    assert!(pinta_a_cor(&dica(true), ColorToken::KanbanCardRaised));
    assert!(!pinta_a_cor(&dica(true), ColorToken::Bg3));
    assert!(pinta_a_cor(&dica(false), ColorToken::Bg3));
}
