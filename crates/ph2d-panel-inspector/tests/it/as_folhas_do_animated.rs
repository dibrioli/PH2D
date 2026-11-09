//! ⭐ **As folhas de um Animated no cartão Image e os vazios no cartão Animation** (plano GEF 26 §3.3, maquete 26,
//! painéis 1, 2, 3 e 6): cada gesto entra como ponteiro no que está pintado e sai como `ImageUseEdit`.

use crate::o_inspector_em_cartoes_por_tipo::{BITS, limpa, objeto, pinta, sprite};
use crate::seam_anim::anim;
use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{AnimFrameRef, InspectorAnimInfo};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{ImageUseEdit, InspectorImageUse};
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::slot_image::SlotImage;
use ph2d_panel_inspector::slot_sheets::{SheetCard, SheetFace, set_current_sheets};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, dropdown_aberto, linhas_em_alerta, set_cards_on,
    set_current_inspector_anim, set_current_inspector_sprite,
};
use ph2d_ui_testkit::MockPanelHost;
use std::sync::Arc;

fn folha(nome: &str, grid: [u32; 2]) -> SheetFace {
    SheetFace {
        image: SlotImage {
            entity_bits: BITS,
            name: nome.into(),
            rgba: Arc::new(vec![255; 4 * 4 * 4]),
            w: 4,
            h: 4,
        },
        px: (grid[0] * 32, grid[1] * 32),
        grid,
    }
}

pub(crate) fn folhas(n: usize, shown: u8) -> SheetCard {
    SheetCard {
        entity_bits: BITS,
        shown,
        sheets: (0..n)
            .map(|k| folha(&format!("sheet{k}"), [4 + k as u32, 1]))
            .collect(),
    }
}

fn cena_com(a: InspectorAnimInfo, card: Option<SheetCard>) -> (MockPanelHost, InspectorState) {
    limpa();
    set_cards_on(true);
    objeto();
    set_current_inspector_anim(Some(InspectorAnimInfo {
        entity_bits: BITS,
        ..a
    }));
    set_current_inspector_sprite(Some(sprite(InspectorImageUse::Animated)));
    set_current_sheets(card);
    (
        MockPanelHost::with_panel::<InspectorPanel>(),
        InspectorState::default(),
    )
}

fn cena(card: Option<SheetCard>) -> (MockPanelHost, InspectorState) {
    cena_com(anim(true, true), card)
}

fn pintou(r: &[(NodeId, Rect)], id: NodeId) -> bool {
    r.iter().any(|(i, q)| *i == id && q.w > 0.0)
}

fn rect_de(r: &[(NodeId, Rect)], id: NodeId) -> Rect {
    r.iter()
        .rev()
        .find(|(i, q)| *i == id && q.w > 0.0)
        .map(|(_, q)| *q)
        .unwrap_or_else(|| panic!("{id:?} não pintou"))
}

/// O clique real em `id`: os eventos do ponteiro vão ao painel; devolve os pedidos.
fn clica(host: &mut MockPanelHost, st: &mut InspectorState, id: NodeId) -> Vec<EditorAction> {
    let r = rect_de(&pinta(host, st), id);
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = host.drained_actions();
    let mut eventos = Vec::new();
    for kind in [PointerKind::Move, PointerKind::Down, PointerKind::Up] {
        eventos.extend(host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        }));
    }
    for ev in &eventos {
        let _ = host.apply_panel_event::<InspectorPanel>(st, *ev);
    }
    host.drained_actions()
}

fn edicoes(a: &[EditorAction]) -> Vec<ImageUseEdit> {
    a.iter()
        .filter_map(|x| match x {
            EditorAction::ImageUseEdit {
                entity_bits: BITS,
                edit,
            } => Some(*edit),
            _ => None,
        })
        .collect()
}

fn fim() {
    set_current_sheets(None);
    limpa();
}

/// Uma folha: a miniatura troca a imagem como hoje, sem ▾ nem Delete; **+ Load sheet** pede `LoadSheet`.
/// *Mutação: abrir o dropdown com uma folha só (`varias` aceitar `len >= 1`) ⇒ a miniatura deixa de ser o slot.*
#[test]
fn com_uma_folha_o_cartao_e_o_de_hoje_mais_load_sheet() {
    let (mut host, mut st) = cena(Some(folhas(1, 0)));
    let r = pinta(&mut host, &mut st);
    assert!(
        pintou(&r, INSP_RENDER_TEXTURE_SLOT),
        "a miniatura troca a imagem"
    );
    assert!(
        !pintou(&r, INSP_IMAGE_SHEET_PICK),
        "sem dropdown com uma folha"
    );
    assert!(!pintou(&r, INSP_IMAGE_SHEET_DELETE));
    let a = clica(&mut host, &mut st, INSP_IMAGE_SHEET_LOAD);
    assert_eq!(edicoes(&a), [ImageUseEdit::LoadSheet]);
    let a = clica(&mut host, &mut st, INSP_RENDER_TEXTURE_SLOT);
    assert!(
        a.iter()
            .any(|x| matches!(x, EditorAction::OpenImagePicker { entity_bits: BITS })),
        "{a:?}"
    );
    fim();
}

/// Duas folhas: a miniatura abre o dropdown, um item por folha; escolher a de índice 0 (a visível é a 1) pede
/// `ShowSheet(0)` e fecha. *Mutação: o item pedir o índice errado (`k + 1`) ⇒ `ShowSheet(1)`.*
#[test]
fn com_duas_folhas_a_miniatura_abre_o_dropdown_e_escolher_mostra_a_folha() {
    let (mut host, mut st) = cena(Some(folhas(2, 1)));
    let r = pinta(&mut host, &mut st);
    assert!(
        !pintou(&r, INSP_RENDER_TEXTURE_SLOT),
        "a miniatura já não troca a imagem"
    );
    assert!(
        !pintou(&r, INSP_IMAGE_SHEET_ITEM[0]),
        "fechado até ao clique"
    );
    let a = clica(&mut host, &mut st, INSP_IMAGE_SHEET_PICK);
    assert!(edicoes(&a).is_empty());
    assert!(dropdown_aberto(BITS));
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_IMAGE_SHEET_ITEM[0]) && pintou(&r, INSP_IMAGE_SHEET_ITEM[1]));
    assert!(!pintou(&r, INSP_IMAGE_SHEET_ITEM[2]), "um item por folha");
    let a = clica(&mut host, &mut st, INSP_IMAGE_SHEET_ITEM[0]);
    assert_eq!(edicoes(&a), [ImageUseEdit::ShowSheet(0)]);
    assert!(!dropdown_aberto(BITS), "escolher fecha");
    fim();
}

/// **Delete sheet** só com duas ou mais, e pede `DeleteSheet`. *Mutação: mostrá-lo com uma folha ⇒ pinta.*
#[test]
fn delete_sheet_so_aparece_com_duas() {
    let (mut host, mut st) = cena(Some(folhas(1, 0)));
    assert!(!pintou(&pinta(&mut host, &mut st), INSP_IMAGE_SHEET_DELETE));
    fim();
    let (mut host, mut st) = cena(Some(folhas(2, 0)));
    let a = clica(&mut host, &mut st, INSP_IMAGE_SHEET_DELETE);
    assert_eq!(edicoes(&a), [ImageUseEdit::DeleteSheet]);
    fim();
}

fn com_vazios(toda_vazia: bool) -> InspectorAnimInfo {
    let mut a = anim(true, true);
    a.rows[0].frames = vec![
        AnimFrameRef::cell(0),
        AnimFrameRef::EMPTY,
        AnimFrameRef::EMPTY,
    ];
    if toda_vazia {
        a.rows[1].frames = vec![AnimFrameRef::EMPTY; 2];
    }
    a
}

/// A linha com vazios fica em alerta (e só ela); a fila de limpeza surge e **Remove empty frames** pede o seu.
/// *Mutação: alertar sem vazios (`vazios(row) >= 0`) ⇒ a linha 1 entra no alerta.*
#[test]
fn a_animacao_com_vazios_fica_em_alerta_no_cartao() {
    let (mut host, mut st) = cena_com(anim(true, true), Some(folhas(1, 0)));
    let r = pinta(&mut host, &mut st);
    assert!(linhas_em_alerta().is_empty());
    assert!(
        !pintou(&r, INSP_ANIM_REMOVE_EMPTY_FRAMES),
        "sem vazios, sem fila"
    );
    fim();
    let (mut host, mut st) = cena_com(com_vazios(false), Some(folhas(1, 0)));
    let _ = pinta(&mut host, &mut st);
    let mut alerta = linhas_em_alerta();
    alerta.dedup();
    assert_eq!(alerta, [INSP_ANIM_ROW[0]], "só a linha com vazios");
    let a = clica(&mut host, &mut st, INSP_ANIM_REMOVE_EMPTY_FRAMES);
    assert_eq!(edicoes(&a), [ImageUseEdit::RemoveEmptyFrames]);
    fim();
}

/// **Delete empty animations** só quando alguma animação é toda vazia. *Mutação: o segundo botão sempre ⇒ pinta
/// com a linha 0 só meio vazia.*
#[test]
fn delete_empty_animations_so_aparece_com_uma_toda_vazia() {
    let (mut host, mut st) = cena_com(com_vazios(false), Some(folhas(1, 0)));
    let r = pinta(&mut host, &mut st);
    assert!(pintou(&r, INSP_ANIM_REMOVE_EMPTY_FRAMES));
    assert!(!pintou(&r, INSP_ANIM_DELETE_EMPTY_ANIMS));
    fim();
    let (mut host, mut st) = cena_com(com_vazios(true), Some(folhas(1, 0)));
    let a = clica(&mut host, &mut st, INSP_ANIM_DELETE_EMPTY_ANIMS);
    assert_eq!(edicoes(&a), [ImageUseEdit::DeleteEmptyAnimations]);
    fim();
}

/// ⭐ **O selo da tinta do Painter a caminho da prévia** (dono 09/11): com `sync`, o canto de cima à direita da prévia
/// pinta o selo e ele tem a dica em inglês; sem `sync`, nada. *Mutações: não registar o selo ⇒ não pintou; sem a dica
/// em `prepara` ⇒ `None`; o selo fora do canto ⇒ a borda direita dele foge da do cartão.*
#[test]
fn a_previa_mostra_o_selo_da_tinta_a_caminho_com_a_dica() {
    let mut a = anim(true, true);
    a.card.sheet = ph2d_editor_core::SheetImage::new(Arc::new(vec![255; 4 * 4 * 4]), 4, 4);
    a.card.sync = Some(ph2d_editor_core::widget::ProgressPhase::Filling(0.5));
    let (mut host, mut st) = cena_com(a.clone(), Some(folhas(1, 0)));
    let r = pinta(&mut host, &mut st);
    let selo = rect_de(&r, INSP_ANIM_PREVIEW_SYNC);
    let lista = rect_de(&r, INSP_ANIM_ROW[0]);
    // A linha da lista pára antes do ▶; o selo vai até à borda da prévia, à direita dela.
    assert!(
        selo.y + selo.h <= lista.y
            && selo.x + selo.w > lista.x + lista.w
            && selo.h <= ph2d_tokens::ROW_H_PX,
        "o selo pequeno no canto de cima à direita da prévia, acima da lista: {selo:?} × {lista:?}"
    );
    assert_eq!(
        host.store().tooltip_for(INSP_ANIM_PREVIEW_SYNC),
        Some("The animation updates when the paint settles")
    );
    fim();
    a.card.sync = None;
    let (mut host, mut st) = cena_com(a, Some(folhas(1, 0)));
    assert!(!pintou(&pinta(&mut host, &mut st), INSP_ANIM_PREVIEW_SYNC));
    fim();
}
