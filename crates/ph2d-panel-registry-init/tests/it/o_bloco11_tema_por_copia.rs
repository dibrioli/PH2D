//! ⭐ **T63 — tema, contorno e notas por CÓPIA solta** (`docs/CartoesUI/01_decisoes_do_dono.md`;
//! smoke do bloco 10, dono 02/11: «se num player coloque o theme da seção Physic Player no canvas
//! em vermelho, ao selecionar outro e arrastar a mesma seção para o canvas, o card muda de cor
//! sozinho»). A cópia nasce com o tema e as notas do cartão da lista; depois cada uma muda
//! sozinha; ao voltar à lista, devolve-os a ela.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{arrasta, clica, pinta, solta};
use super::o_bloco5_abertos::escolhe;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::{
    self, CTX_MENU_CREATE_NOTE, CTX_MENU_SECTION_THEME_CANDY, CTX_MENU_SECTION_THEME_OLED,
    CTX_MENU_SECTION_THEME_PLUMBER_RED,
};
use ph2d_editor_core::interaction::{
    WidgetEvent, loose_face_of, loose_instance_of, loose_pin_of, loose_rect_of, loose_return_of,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::theme_menu::section_theme_of_menu_id;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::CARD_TRANSFORM;
use ph2d_tokens::Theme;

const CIRCULO: u64 = 11;
const QUADRADO: u64 = 12;

/// Botão direito em `(x, y)` e o clique na linha `linha` do menu que ele abriu.
fn menu(h: &mut HeroScreen, (x, y): (f32, f32), linha: NodeId) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Secondary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
    // O Down no item fotografa o pedido e fecha o menu; o clique lê o pedido.
    h.store.close_context_menu();
    h.apply_event(WidgetEvent::Click(linha));
    pinta(h, 2);
}

fn tema(linha: NodeId) -> Option<Theme> {
    section_theme_of_menu_id(linha).flatten()
}

/// O cabeçalho e o corpo do solto `k` (fora dos ícones da direita).
fn solto(h: &HeroScreen, k: NodeId) -> ((f32, f32), (f32, f32)) {
    let r = h
        .store
        .panel_rect(loose_rect_of(k))
        .expect("o solto pinta-se");
    (
        (r.x + r.w * 0.3, r.y + 14.0),
        (r.x + r.w * 0.5, r.y + r.h - 4.0),
    )
}

/// As secções das notas do Inspector.
fn notas(h: &HeroScreen) -> Vec<Option<NodeId>> {
    h.store
        .notes_for_panel(ids::INSP_PANEL)
        .iter()
        .map(|n| n.section)
        .collect()
}

fn conta(v: &[Option<NodeId>], s: NodeId) -> usize {
    v.iter().filter(|n| **n == Some(s)).count()
}

/// *Mutação: a face = a própria instância (`loose_face_of(k) = k`) ⇒ a cópia 1 pinta a lista.*
#[test]
fn each_loose_copy_keeps_its_own_theme_and_notes() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    escolhe(CIRCULO, "Circle", 1.0);
    let mut h = HeroScreen::new(NodeId(1));
    let ids_: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids_ {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 30);
    let pega = h
        .hit_index
        .iter_registrations()
        .filter(|(id, _)| *id == ids::grip_de(CARD_TRANSFORM))
        .map(|(_, r)| r)
        .max_by(|a, b| a.w.total_cmp(&b.w))
        .expect("o título do Transform na lista");
    let cabeca = (pega.x + 12.0, pega.y + pega.h * 0.5);
    // A lista: tema Black e uma nota no Transform.
    menu(&mut h, cabeca, CTX_MENU_SECTION_THEME_OLED);
    menu(
        &mut h,
        (cabeca.0, pega.y + pega.h + 30.0),
        CTX_MENU_CREATE_NOTE,
    );
    let oled = tema(CTX_MENU_SECTION_THEME_OLED);
    assert_eq!(h.store.section_theme(CARD_TRANSFORM), oled);
    assert_eq!(
        conta(&notas(&h), CARD_TRANSFORM),
        1,
        "a nota da lista não nasceu"
    );

    // Cópia 1: nasce com o tema e a nota da lista; o vermelho é só dela.
    solta(&mut h, CARD_TRANSFORM);
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    let um = loose_face_of(CARD_TRANSFORM);
    assert_eq!(
        h.store.section_theme(um),
        oled,
        "a cópia não nasceu com o tema da lista"
    );
    assert_eq!(
        conta(&notas(&h), um),
        1,
        "a cópia não nasceu com a nota da lista"
    );
    // A nota da cópia pinta-se DENTRO dela, nunca no fim da lista (as que sobram).
    let i = notas(&h)
        .iter()
        .position(|n| *n == Some(um))
        .expect("a nota da cópia");
    let r = h
        .hit_index
        .rect_for(ids::note_ids(ids::INSP_PANEL).slot[i])
        .expect("a nota da cópia pinta-se");
    let caixa = h
        .store
        .panel_rect(loose_rect_of(CARD_TRANSFORM))
        .expect("o solto");
    assert!(
        caixa.contains(r.x + r.w * 0.5, r.y + r.h * 0.5),
        "a nota da cópia pintou-se fora dela: {r:?} × {caixa:?}"
    );
    let (cab1, _) = solto(&h, CARD_TRANSFORM);
    menu(&mut h, cab1, CTX_MENU_SECTION_THEME_PLUMBER_RED);
    let vermelho = tema(CTX_MENU_SECTION_THEME_PLUMBER_RED);
    assert_eq!(
        h.store.section_theme(um),
        vermelho,
        "o vermelho não chegou à cópia 1"
    );
    assert_eq!(
        h.store.section_theme(CARD_TRANSFORM),
        oled,
        "pintar a cópia solta pintou o cartão da lista"
    );

    // Cópia 2 de outro objecto: nasce com o da lista (Black), nunca com o vermelho da 1.
    escolhe(QUADRADO, "Square", 9.0);
    pinta(&mut h, 5);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    arrasta(
        &mut h,
        cabeca,
        (canvas.x + canvas.w * 0.6, canvas.y + canvas.h * 0.6),
    );
    pinta(&mut h, 3);
    let k2 = loose_instance_of(CARD_TRANSFORM, 1);
    let dois = loose_face_of(k2);
    assert!(h.store.loose_card(k2).is_some(), "não há cópia 2");
    assert_eq!(
        h.store.section_theme(dois),
        oled,
        "a cópia 2 não nasceu Black"
    );
    assert_eq!(
        h.store.section_theme(um),
        vermelho,
        "a cópia 1 mudou de cor sozinha"
    );
    let tinta = |s: NodeId| CardInk::of_card(h.store.section_theme(s), Theme::Dark).theme;
    assert_eq!((tinta(um), tinta(dois)), (Theme::PlumberRed, Theme::Oled));

    // A cópia 2 ganha uma nota e o Candy; nada disso toca na 1 nem na lista.
    let (cab2, corpo2) = solto(&h, k2);
    menu(&mut h, corpo2, CTX_MENU_CREATE_NOTE);
    menu(&mut h, cab2, CTX_MENU_SECTION_THEME_CANDY);
    let candy = tema(CTX_MENU_SECTION_THEME_CANDY);
    let v = notas(&h);
    assert_eq!(
        (conta(&v, CARD_TRANSFORM), conta(&v, um), conta(&v, dois)),
        (1, 1, 2),
        "as notas não são de cada cartão: {v:?}"
    );
    assert_eq!(h.store.section_theme(dois), candy);
    assert_eq!(h.store.section_theme(CARD_TRANSFORM), oled);

    // Volta à lista: o Candy e as duas notas da cópia 2 passam para o cartão da lista.
    clica(&mut h, loose_return_of(k2));
    assert!(h.store.loose_card(k2).is_none(), "a cópia 2 não voltou");
    let v = notas(&h);
    assert_eq!(
        h.store.section_theme(CARD_TRANSFORM),
        candy,
        "a cor não passou à lista"
    );
    assert_eq!(h.store.section_theme(dois), None);
    assert_eq!(
        (conta(&v, CARD_TRANSFORM), conta(&v, dois)),
        (3, 0),
        "{v:?}"
    );
    assert_eq!(h.store.section_theme(um), vermelho, "a cópia 1 mudou");
}
