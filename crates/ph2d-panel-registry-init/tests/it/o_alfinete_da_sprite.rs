//! ⭐ **F4 — o alfinete nos cartões de sprite** (`docs/CartoesUI/09`): Texture & Atlas, Color & Tint,
//! 9-Slice e Sampling, por gesto real no hero: prende-se a A, escolhe-se outra coisa, edita-se no
//! solto, e o pedido é de A e não se espalha, com a seleção em B, em B+C e em A+B.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{arrasta, centro, clica, mao, pinta, solta};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{
    INSP_LIVE_COLOR_SECTION, INSP_LIVE_RENDER_SECTION, INSP_LIVE_SAMPLING_SECTION,
    INSP_LIVE_SLICE_SECTION,
};
use ph2d_editor_core::interaction::{loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{
    HeroScreen, InspectorSamplingInfo, InspectorSliceInfo, InspectorSpriteInfo,
    InspectorSpriteSource, SpriteFieldEdit,
};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_panel_inspector::ids as pids;
use std::collections::BTreeSet;

const A: u64 = 31;
const B: u64 = 32;
const C: u64 = 33;
const SELECOES: [&[u64]; 3] = [&[B], &[B, C], &[A, B]];

/// A sprite de `b`, como a shell a publica; A difere de B e C.
fn arma(b: u64) {
    let a = b == A;
    insp::set_current_inspector_name(Some(ph2d_editor_core::screens::hero::InspectorNameInfo {
        entity_bits: b,
        name: if a { "Hero" } else { "Tree" }.to_owned(),
    }));
    insp::set_current_inspector_sprite(Some(InspectorSpriteInfo {
        entity_bits: b,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::HandPacked {
            sheet: 1,
            region: 2,
        },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        emissive: 0.0,
        sheet_label: Some("sheet".to_owned()),
        source_pixels: Some((64, 64)),
        can_reimport: false,
        flip_x: a,
        flip_y: false,
        opacity: if a { 0.25 } else { 0.75 },
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        image_use: ph2d_editor_core::InspectorImageUse::Still,
        image_cells: 1,
        draw: ph2d_editor_core::InspectorDraw::Fixed,
        frame: 0,
        tint: [1.0, 1.0, 1.0, 1.0],
        self_tint: [1.0, 1.0, 1.0, 1.0],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: true,
        region_rect: [0.0, 0.0, 32.0, 32.0],
        region_filter_clip: false,
        centered: a,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: Default::default(),
    }));
    insp::set_current_inspector_sampling(Some(InspectorSamplingInfo {
        entity_bits: b,
        filter_tag: 1,
        repeat_tag: 1,
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
        selected_count: 1,
        mixed: Default::default(),
    }));
    insp::set_current_inspector_slice(Some(InspectorSliceInfo {
        entity_bits: b,
        present: true,
        draw_mode_tag: 1,
        borders: [4.0; 4],
        size: [2.0, 2.0],
        tile_modes: [1; 8],
        centre_tile_mode: 1,
        tile_mode_tag: 1,
        fill_center: a,
        selected_count: 1,
        mixed: Default::default(),
    }));
}

fn quadros(h: &mut HeroScreen, sel: &[u64], n: usize) {
    for _ in 0..n {
        insp::com_cada_preso(
            &h.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL),
            arma,
        );
        arma(sel[0]);
        insp::set_current_inspector_selecionados(sel.len());
        pinta(h, 1);
    }
}

/// O cartão `sec` solto e preso a A, com a seleção em `sel`.
fn preso(sec: NodeId, sel: &[u64]) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    insp::set_cards_on(true);
    arma(A);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    h.gizmo.replace_selection(Some(A));
    h.store.set_card_subject(
        ph2d_editor_core::ids::INSP_PANEL,
        Some((A, "Hero".to_owned())),
    );
    // Os outros dobrados: o cartão fica acima da dobra para a mão o apanhar.
    for c in [
        INSP_LIVE_RENDER_SECTION,
        INSP_LIVE_COLOR_SECTION,
        INSP_LIVE_SLICE_SECTION,
        INSP_LIVE_SAMPLING_SECTION,
        insp::CARD_TRANSFORM,
        insp::CARD_APPEARANCE,
    ] {
        h.store.set_collapsed(c, c != sec);
    }
    pinta(&mut h, 30);
    solta(&mut h, sec);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(A));
    h.gizmo.replace_selection(Some(sel[0]));
    for &b in &sel[1..] {
        h.gizmo.add_to_selection(b);
    }
    quadros(&mut h, sel, 6);
    h
}

/// Um clique em `id`, que tem de cair no solto; os pedidos que ele deixou.
fn no_solto(h: &mut HeroScreen, sec: NodeId, id: NodeId, sel: &[u64]) -> Vec<EditorAction> {
    let r = h
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{id:?} não pintou"));
    let s = h.store.panel_rect(loose_rect_of(sec)).expect("o solto");
    let c = centro(r);
    assert!(s.contains(c.0, c.1), "{id:?} fora do solto");
    let _: Vec<_> = h.bus.drain().collect();
    mao(h, PointerKind::Down, c);
    mao(h, PointerKind::Up, c);
    assert_eq!(
        h.store.card_edit_subject(),
        Some(A),
        "o gesto não é do preso"
    );
    let fan = h.store.card_fan_out(sel.len(), sel.iter().copied());
    assert!(fan.is_empty(), "espalha por {fan:?}");
    h.bus.drain().collect()
}

/// Texture & Atlas: a caixa do Centered é de A. *Mutação: `card_event_subject` sempre `None`.*
#[test]
fn the_pinned_render_card_edits_only_its_object() {
    for sel in SELECOES {
        let sec = INSP_LIVE_RENDER_SECTION;
        let mut h = preso(sec, sel);
        let feitos = no_solto(&mut h, sec, pids::INSP_SPRITE_TINT_FILL, sel);
        assert!(
            feitos.iter().any(|a| matches!(
                a,
                EditorAction::InspectorSpriteEdit {
                    entity_bits: A,
                    edit: SpriteFieldEdit::TintFill(_)
                }
            )),
            "{sel:?}: {feitos:?}"
        );
    }
}

/// 9-Slice: Fill Center é de A, e parte do estado de A (ligado), não do da seleção (desligado).
#[test]
fn the_pinned_slice_card_edits_only_its_object() {
    for sel in SELECOES {
        let sec = INSP_LIVE_SLICE_SECTION;
        let mut h = preso(sec, sel);
        let feitos = no_solto(&mut h, sec, pids::INSP_SLICE_FILL_CENTER, sel);
        let dele: Vec<_> = feitos
            .iter()
            .filter_map(|a| match a {
                EditorAction::InspectorSliceEdit { entity_bits, edit } => {
                    Some((*entity_bits, *edit))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            dele,
            [(
                A,
                ph2d_editor_core::screens::hero::SliceFieldEdit::FillCenter(false)
            )],
            "{sel:?}"
        );
    }
}

/// Sampling: a escolha de filtro é de A.
#[test]
fn the_pinned_sampling_card_edits_only_its_object() {
    for sel in SELECOES {
        let sec = INSP_LIVE_SAMPLING_SECTION;
        let mut h = preso(sec, sel);
        let feitos = no_solto(&mut h, sec, pids::INSP_SAMPLE_FILTER[0], sel);
        assert!(
            feitos.iter().all(|a| !matches!(
                a,
                EditorAction::InspectorSamplingEdit { entity_bits, .. } if *entity_bits != A
            )) && feitos.iter().any(|a| matches!(
                a,
                EditorAction::InspectorSamplingEdit { entity_bits: A, .. }
            )),
            "{sel:?}: {feitos:?}"
        );
    }
}

/// Color & Tint: a amostra abre o seletor do preso; a cor escolhida vai a A em todo quadro, e a
/// seleção não recebe nada, mesmo trocando de objecto com ele aberto.
/// *Mutações: abrir pelo `set_picker_target` (o seletor é da seleção) · tirar `seletor_do_preso`.*
#[test]
fn the_colour_picked_on_the_pinned_tint_swatch_goes_to_its_object_only() {
    for sel in SELECOES {
        let sec = INSP_LIVE_COLOR_SECTION;
        let mut h = preso(sec, sel);
        let _ = no_solto(&mut h, sec, pids::INSP_SPRITE_TINT_SWATCH, sel);
        assert_eq!(h.store.picker_target(), Some(pids::INSP_SPRITE_TINT_SWATCH));
        assert_eq!(h.store.picker_subject(), Some(A), "{sel:?}");
        h.store.set_blender_value(
            ph2d_editor_core::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(10, 20, 30, 255),
        );
        quadros(&mut h, sel, 1);
        let tinta: Vec<_> = h
            .bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::InspectorSpriteEdit {
                    entity_bits,
                    edit: SpriteFieldEdit::Tint(_),
                } => Some(entity_bits),
                _ => None,
            })
            .collect();
        assert_eq!(tinta, [A], "{sel:?}");
        assert_eq!(h.store.picker_subject(), Some(A), "fechou");
    }
}

/// Os controlos do solto que a mão apanha: o que responde ao clique no próprio centro.
fn controlos_do_solto(h: &HeroScreen, sec: NodeId) -> Vec<NodeId> {
    let s = h.store.panel_rect(loose_rect_of(sec)).expect("o solto");
    let mut ids: Vec<NodeId> = h
        .hit_index
        .iter_registrations()
        .filter(|(id, r)| {
            let c = centro(*r);
            s.contains(c.0, c.1) && h.hit_index.hit(c.0, c.1) == Some(*id)
        })
        .map(|(id, _)| id)
        .collect();
    ids.sort_by_key(|i| i.0);
    ids.dedup();
    ids
}

/// O gesto de `id`: scrub na pega, se é campo; clique, senão. Devolve os pedidos que ele deixou.
fn gesto(h: &mut HeroScreen, sel: &[u64], id: NodeId) -> Vec<String> {
    let r = h.hit_index.rect_for(id).expect("pintado");
    let _: Vec<_> = h.bus.drain().collect();
    if h.store.card_field(id).is_some() {
        let y = r.y + r.h * 0.5;
        arrasta(
            h,
            (r.x + m::ICON_CELL * 0.5, y),
            (r.x + m::ICON_CELL * 0.5 + 40.0, y),
        );
    } else {
        mao(h, PointerKind::Down, centro(r));
        mao(h, PointerKind::Up, centro(r));
    }
    let antes: BTreeSet<NodeId> = h.hit_index.iter_registrations().map(|(i, _)| i).collect();
    quadros(h, sel, 2);
    let mut feitos: Vec<String> = h.bus.drain().map(|a| format!("{a:?}")).collect();
    // Abriu uma lista: a última opção é a que difere da acesa em toda lista de duas ou mais.
    let opcao = h
        .hit_index
        .iter_registrations()
        .filter(|(i, r)| {
            !antes.contains(i) && {
                let c = centro(*r);
                h.hit_index.hit(c.0, c.1) == Some(*i)
            }
        })
        .last();
    if let Some((_, r)) = opcao {
        mao(h, PointerKind::Down, centro(r));
        mao(h, PointerKind::Up, centro(r));
        quadros(h, sel, 2);
        feitos.extend(h.bus.drain().map(|a| format!("{a:?}")));
    }
    feitos
}

/// ⭐ A lei inteira (F11): cada controlo de um cartão de sprite preso. Nenhum pedido sai para a seleção,
/// o seletor de cor aberto é o do preso, e `piso` é o censo medido dos que escrevem no gesto.
/// *Mutações: `card_event_subject` sempre `None` · `open_picker_on` sem o sujeito.*
fn todo_controlo_escreve_no_preso(sec: NodeId, piso: usize, amostras: usize) {
    let sel: &[u64] = &[B, C];
    let dele = format!("entity_bits: {A}");
    let alheios = [format!("entity_bits: {B}"), format!("entity_bits: {C}")];
    let ids = controlos_do_solto(&preso(sec, sel), sec);
    let (mut escreveram, mut seletores) = (0, 0);
    for id in &ids {
        let mut h = preso(sec, sel);
        let feitos = gesto(&mut h, sel, *id);
        for a in &feitos {
            assert!(
                !alheios.iter().any(|x| a.contains(x.as_str())),
                "{sec:?} {id:?} escreveu na seleção: {a}"
            );
        }
        escreveram += usize::from(feitos.iter().any(|a| a.contains(dele.as_str())));
        if h.store.picker_target().is_some() {
            assert_eq!(h.store.picker_subject(), Some(A), "{sec:?} {id:?}");
            seletores += 1;
        }
    }
    assert!(escreveram >= piso, "{sec:?}: {escreveram} de {}", ids.len());
    assert_eq!(seletores, amostras, "{sec:?}: seletores abertos");
}

#[test]
fn todo_controlo_do_render_preso_escreve_no_preso() {
    // ⬇️ `9 → 6` (05/11): Centered, Offset e Hand-packed saíram do cartão (ficha da Sprite).
    todo_controlo_escreve_no_preso(INSP_LIVE_RENDER_SECTION, 6, 0);
}

#[test]
fn todo_controlo_do_color_preso_escreve_no_preso() {
    // ⬇️ seletores `6 → 1` (05/11): os cantos foram para o Advanced e o Self Tint só existe com filhos.
    todo_controlo_escreve_no_preso(INSP_LIVE_COLOR_SECTION, 3, 1);
}

#[test]
fn todo_controlo_do_slice_preso_escreve_no_preso() {
    todo_controlo_escreve_no_preso(INSP_LIVE_SLICE_SECTION, 21, 0);
}

#[test]
fn todo_controlo_do_sampling_preso_escreve_no_preso() {
    todo_controlo_escreve_no_preso(INSP_LIVE_SAMPLING_SECTION, 14, 0);
}

/// Os quatro cartões têm alfinete: a lei é ícone só com escrita certa, e a escrita é a provada acima.
#[test]
fn every_sprite_card_is_pinnable() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    for sec in [
        INSP_LIVE_RENDER_SECTION,
        INSP_LIVE_COLOR_SECTION,
        INSP_LIVE_SLICE_SECTION,
        INSP_LIVE_SAMPLING_SECTION,
    ] {
        let h = preso(sec, &[B]);
        assert!(h.store.is_card_pinnable(sec), "{sec:?}");
    }
}
