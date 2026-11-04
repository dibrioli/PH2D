//! Em cartões a caixa regista os mesmos ids: o despacho é por id, logo o clique em cada um faz o que
//! fazia. Os gestos de cada id estão em `tests.rs` (New Image) e nos irmãos de cada caixa.

use super::super::context_menu_dialogs::{
    paint_color_mode_dialog, paint_new_image_dialog, paint_palette_rename_dialog,
    paint_sheet_size_dialog,
};
use crate::interaction::{HitIndex, WidgetStore};
use crate::widget::lista_flutuante::force_cromo_on_this_thread;
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
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
