//! Em cartões a caixa regista os mesmos ids: o despacho é por id, logo o clique em cada um faz o que
//! fazia. Os gestos de cada id estão em `tests.rs` (New Image) e nos irmãos de cada caixa.

use super::super::context_menu_dialogs::{
    paint_color_mode_dialog, paint_new_image_dialog, paint_palette_rename_dialog,
    paint_sheet_size_dialog,
};
use crate::interaction::{HitIndex, WidgetStore};
use crate::screens::hero::chrome::paint_onion_modal;
use crate::widget::card_ink::CardInk;
use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
use crate::widget::lista_flutuante::force_cromo_on_this_thread;
use crate::widget::{
    Dropdown, DropdownOption, Popover, TextInput, paint_dropdown_popover, paint_popover,
    paint_text_input_with_buffer,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;
use std::collections::BTreeSet;

type Pintor = fn(&mut VectorScene, &mut TextSystem, Theme, &mut HitIndex, &WidgetStore, Rect);

fn ids_de(pinta: Pintor, store: &WidgetStore, cartao: bool) -> (BTreeSet<u64>, Vec<Rect>) {
    force_cromo_on_this_thread(Some(cartao));
    let (mut scene, mut ts, mut hit) = (
        VectorScene::new(),
        TextSystem::without_system_fonts(),
        HitIndex::default(),
    );
    let v = Rect::new(0.0, 0.0, 1400.0, 900.0);
    pinta(&mut scene, &mut ts, Theme::Dark, &mut hit, store, v);
    force_cromo_on_this_thread(None);
    let rects: Vec<_> = hit.iter_registrations().map(|(_, r)| r).collect();
    let ids = hit.iter_registrations().map(|(i, _)| i.0).collect();
    (ids, rects)
}

#[test]
fn em_cartoes_cada_caixa_regista_os_mesmos_ids() {
    let store = WidgetStore::default();
    let pintores: [(&str, Pintor); 4] = [
        ("new image", paint_new_image_dialog),
        ("sheet size", paint_sheet_size_dialog),
        ("rename", paint_palette_rename_dialog),
        ("color mode", paint_color_mode_dialog),
    ];
    for (nome, pinta) in pintores {
        let (antes, _) = ids_de(pinta, &store, false);
        let (agora, rects) = ids_de(pinta, &store, true);
        assert!(!antes.is_empty(), "{nome}: a fixtura não registou nada");
        assert_eq!(agora, antes, "{nome}: ids diferentes");
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                let longe =
                    a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(longe, "{nome}: alvos sobrepostos {a:?} {b:?}");
            }
        }
    }
}

/// A cor pintada em `scene`, pelos dados de desenho.
fn tem_cor(scene: &VectorScene, cor: ph2d_vector::Color) -> bool {
    let palavra = cor.premultiply().to_rgba8().to_u32();
    scene.inner().encoding().draw_data.contains(&palavra)
}

fn com_cromo<R>(on: bool, f: impl FnOnce() -> R) -> R {
    force_cromo_on_this_thread(Some(on));
    let r = f();
    force_cromo_on_this_thread(None);
    r
}

fn tinta() -> CardInk {
    CardInk::of_list(Theme::Dark)
}

/// O campo vestido escreve o placeholder na tinta do cartão, sem o token cru.
#[test]
fn o_placeholder_de_um_campo_vestido_le_a_tinta_do_cartao() {
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let ti = TextInput::new(NodeId(1), "").placeholder("Search");
    crate::widget::card_ink::com(tinta(), || {
        let r = Rect::new(0.0, 0.0, 200.0, 28.0);
        let t = Theme::Dark;
        paint_text_input_with_buffer(&ti, Some(""), Some(0), None, r, &mut scene, &mut ts, t);
    });
    let esperado = crate::widget::card_ink::com(tinta(), || {
        crate::widget::card_ink::tinta(ColorToken::Text3, Theme::Dark)
    });
    assert!(tem_cor(&scene, esperado), "o placeholder não leu a tinta");
}

/// A moldura genérica e a lista aberta de fora de um cartão trocam com o modo, e o de antes é o de antes.
#[test]
fn as_listas_de_fora_de_um_cartao_vestem_a_superficie_do_cromo() {
    let r = Rect::new(10.0, 10.0, 200.0, 100.0);
    let dados = |on: bool| {
        com_cromo(on, || {
            let mut scene = VectorScene::new();
            paint_popover(&Popover::new(NodeId(1)), r, &mut scene, Theme::Dark);
            scene.inner().encoding().draw_data.clone()
        })
    };
    assert_ne!(dados(true), dados(false), "a moldura não trocou");

    let opt = DropdownOption::new(NodeId(2), 0_u8, "One");
    let dd = Dropdown::new(NodeId(3), "", vec![opt])
        .selected(0)
        .open(true);
    let lista = |on: bool| {
        com_cromo(on, || {
            let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
            let chip = Rect::new(10.0, 10.0, 200.0, 28.0);
            paint_dropdown_popover(&dd, chip, &mut scene, &mut ts, Theme::Dark);
            tem_cor(
                &scene,
                tinta().color(ColorToken::KanbanCardRaised).with_alpha(1.0),
            )
        })
    };
    assert!(lista(true), "a lista não vestiu a superfície");
    assert!(!lista(false), "PH2D_CARDS=0 vestiu a lista");
}

fn paleta(on: bool) -> (bool, usize) {
    com_cromo(on, || {
        let model = PaletteModel {
            title: "Add".into(),
            toggle: None,
            tips: Vec::new(),
            groups: vec![PaletteGroup {
                title: "Source".into(),
                color: ColorToken::NodeCatSource,
                subs: vec![PaletteSub {
                    title: None,
                    items: vec![PaletteItem {
                        label: "Grid".into(),
                        id: NodeId(11),
                        keywords: "",
                    }],
                }],
            }],
        };
        let (mut scene, mut ts, mut hit) = (
            VectorScene::new(),
            TextSystem::without_system_fonts(),
            HitIndex::default(),
        );
        let store = WidgetStore::default();
        let motion = crate::motion::UiMotion::default();
        let v = Rect::new(0.0, 0.0, 1400.0, 900.0);
        let t = Theme::Forge;
        let _ = crate::widget::command_palette::paint(
            &mut scene, &mut ts, t, &mut hit, &model, "", v, &motion, &store,
        );
        let fundo = CardInk::of_list(Theme::Forge).color(ColorToken::KanbanCardRaised);
        (
            tem_cor(&scene, fundo.with_alpha(1.0)),
            scene.inner().encoding().resources.glyph_runs.len(),
        )
    })
}

/// Em cartões a paleta veste a superfície da caixa e perde o «X» escrito à mão.
#[test]
fn a_paleta_em_cartoes_veste_a_caixa_e_nao_escreve_o_x() {
    let (vestida, glifos_on) = paleta(true);
    let (antiga, glifos_off) = paleta(false);
    assert_eq!(glifos_on + 1, glifos_off, "o X virou ícone");
    assert!(vestida, "a moldura é a da caixa");
    assert!(!antiga, "PH2D_CARDS=0 dá a de antes");
}

/// O fechar do Onion deixa de ser a letra escrita à mão.
#[test]
fn o_fechar_do_onion_em_cartoes_nao_e_uma_letra() {
    let correr = |on: bool| {
        com_cromo(on, || {
            let (mut scene, mut ts, mut hit) = (
                VectorScene::new(),
                TextSystem::without_system_fonts(),
                HitIndex::default(),
            );
            let mut store = WidgetStore::default();
            let (a, b) = ([230, 80, 80, 255], [80, 200, 120, 255]);
            store.open_onion_modal(100.0, 100.0, 0.5, 0.3, 0.3, a, b);
            let v = Rect::new(0.0, 0.0, 1400.0, 900.0);
            paint_onion_modal(&mut scene, &mut ts, Theme::Dark, &mut hit, &store, v);
            scene.inner().encoding().resources.glyph_runs.len()
        })
    };
    assert_eq!(correr(true) + 1, correr(false), "o X virou ícone");
}
