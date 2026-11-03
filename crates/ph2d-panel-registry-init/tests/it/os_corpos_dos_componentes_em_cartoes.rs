//! ⭐ **CartoesUI T29: o corpo de um cartão de componente tem a cara do cartão** (D5/D13, maquete
//! `docs/CartoesUI/maquete/bones.png`): dentro do cartão, um número de componente pinta o rótulo EM
//! CIMA com a unidade nele e a caixa Figma (glifo + valor, sem sufixo), e arrasta-se pela pega com o
//! passo do ajudante; fora dos cartões o mesmo ajudante pinta o rótulo à esquerda, como hoje.
#![cfg(feature = "panel-inspector")]

use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1800.0,
};

fn pinta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..30 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn ponteiro(h: &mut HeroScreen, kind: PointerKind, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    let ev = PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let evs: Vec<_> =
        ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
            .to_vec();
    for e in evs {
        h.apply_event(e);
    }
}

fn valor(h: &HeroScreen, id: NodeId) -> f64 {
    match h.store.get(id) {
        Some(InteractiveState::NumberInput { value, .. }) => *value,
        o => panic!("{id:?} não é número: {o:?}"),
    }
}

/// O hero só com o Inspector, o cartão da física aberto (o armado tem de vir antes).
fn hero_com_a_fisica_aberta() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h);
    let seccao = ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION;
    if h.store.is_collapsed(seccao) {
        let t = h
            .hit_index
            .rect_for(ph2d_editor_core::ids::grip_de(seccao))
            .expect("a física não virou cartão");
        ponteiro(&mut h, PointerKind::Down, t.x + 10.0, t.y + t.h * 0.5);
        ponteiro(&mut h, PointerKind::Up, t.x + 10.0, t.y + t.h * 0.5);
        h.store.set_section_open_live(seccao, 1.0);
        pinta(&mut h);
    }
    h
}

#[test]
fn a_component_number_in_a_card_has_its_label_above_with_the_unit() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_a_fisica_aberta();
    let seccao = ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION;
    let titulo = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(seccao))
        .expect("o título da física");
    let (ox, oy) = (
        ph2d_panel_inspector::ids::INSP_PHYS_OFFSET_X,
        ph2d_panel_inspector::ids::INSP_PHYS_OFFSET_Y,
    );
    let a = h.hit_index.rect_for(ox).expect("Offset X no cartão");
    let b = h.hit_index.rect_for(oy).expect("Offset Y no cartão");
    // A caixa do cartão: a altura dele e a largura toda (o rótulo não come a coluna da esquerda).
    assert!(
        (b.h - m::FIELD_H).abs() < 0.5,
        "a caixa não é a do cartão: {b:?}"
    );
    assert!(
        b.x + b.w - a.x > titulo.w * 0.75,
        "o rótulo ainda come a coluna da esquerda: caixas {a:?} {b:?} no cartão {titulo:?}"
    );
    // O rótulo EM CIMA: entre duas linhas seguidas (Half Size → Offset) cabe a linha do rótulo.
    let meia = h
        .hit_index
        .rect_for(ph2d_panel_inspector::ids::INSP_PHYS_HALF_Y)
        .expect("Half Height no cartão");
    assert!(
        a.y - meia.y >= m::FIELD_H + m::LABEL_PX,
        "sem linha de rótulo entre as linhas: {meia:?} → {a:?}"
    );
    // A unidade no RÓTULO (a dica do arrasto leva o rótulo inteiro).
    let dica = ph2d_editor_core::panel::cards::dicas::field_hint(&h.store, oy)
        .expect("o número do componente não é campo de cartão");
    assert!(
        dica.contains("(px)") || dica.contains("(m)"),
        "a unidade não está no rótulo: {dica:?}"
    );
    // O passo do ajudante (`num_row_unit`: o `NumberInput` sem `.step` anda 1.0).
    let spec = h.store.card_field(oy).expect("sem passo registado");
    assert_eq!((spec.step, spec.scrub), (1.0, 1.0));
    // O arrasto pela PEGA (o glifo) muda o número.
    let antes = valor(&h, oy);
    let (px, py) = (b.x + m::ICON_CELL * 0.5, b.y + b.h * 0.5);
    ponteiro(&mut h, PointerKind::Down, px, py);
    for dx in [4.0, 8.0, 12.0, 16.0, 20.0] {
        ponteiro(&mut h, PointerKind::Move, px + dx, py);
    }
    ponteiro(&mut h, PointerKind::Up, px + 20.0, py);
    assert!(
        valor(&h, oy) > antes,
        "o arrasto pela pega não mudou o número ({antes} → {})",
        valor(&h, oy)
    );
    super::o_inspector_armado::desarma_tudo();
}

/// ⭐ T29/T64: a linha de marcar no cartão é à Figma (dono 02/11) — a caixa do cartão (`FIELD_H`)
/// em linhas de cartão (`FIELD_H + FIELD_ROW_GAP`), e o clique no nome marca. *Mutação: o ramo
/// do cartão em `paint_check_row` desligado ⇒ a linha volta a ser a do formulário.*
#[test]
fn a_check_in_a_card_is_a_figma_checkbox_row() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_a_fisica_aberta();
    let seccao = ph2d_editor_core::ids::INSP_LIVE_CAMERA_SECTION;
    // A câmera fica abaixo da parte visível: rola a lista até o título dela aparecer.
    let grip = ph2d_editor_core::ids::grip_de(seccao);
    for passo in 1..60 {
        if h.hit_index.rect_for(grip).is_some() {
            break;
        }
        let painel = ph2d_editor_core::ids::INSP_PANEL;
        h.store.set_panel_scroll(painel, passo as f32 * 300.0);
        h.store.set_panel_scroll_live(painel, passo as f32 * 300.0);
        pinta(&mut h);
    }
    if h.store.is_collapsed(seccao) {
        let t = h
            .hit_index
            .rect_for(ph2d_editor_core::ids::grip_de(seccao))
            .expect("a câmera não virou cartão");
        ponteiro(&mut h, PointerKind::Down, t.x + 10.0, t.y + t.h * 0.5);
        ponteiro(&mut h, PointerKind::Up, t.x + 10.0, t.y + t.h * 0.5);
        h.store.set_section_open_live(seccao, 1.0);
        pinta(&mut h);
    }
    let (ia, ib) = (
        ph2d_panel_inspector::ids::INSP_CAMERA_ACTIVE,
        ph2d_panel_inspector::ids::INSP_CAMERA_PREVIEW,
    );
    let mut rolar = h.store.panel_scroll(ph2d_editor_core::ids::INSP_PANEL);
    while h.hit_index.rect_for(ia).is_none() && rolar < 30_000.0 {
        rolar += 200.0;
        h.store
            .set_panel_scroll(ph2d_editor_core::ids::INSP_PANEL, rolar);
        h.store
            .set_panel_scroll_live(ph2d_editor_core::ids::INSP_PANEL, rolar);
        pinta(&mut h);
    }
    let a = h.hit_index.rect_for(ia).expect("Active no cartão");
    let b = h.hit_index.rect_for(ib).expect("Look Through no cartão");
    for r in [a, b] {
        assert!(
            (r.h - m::FIELD_H).abs() < 0.5,
            "a linha não é a do cartão: {r:?}"
        );
    }
    // À Figma os dois nomes curtos cabem lado a lado numa linha do cartão.
    assert!(
        (a.y - b.y).abs() < 0.5 && b.x > a.x,
        "as duas caixas não partilham a linha: {a:?} × {b:?}"
    );
    // O NOME (à direita do quadradinho) é alvo da caixa.
    let nome = (a.x + m::CHECK_BOX + m::CHECK_GAP + 4.0, a.y + a.h * 0.5);
    assert_eq!(
        h.hit_index.hit(nome.0, nome.1),
        Some(ia),
        "o nome não marca"
    );
    super::o_inspector_armado::desarma_tudo();
}

#[test]
fn outside_a_card_the_same_row_keeps_its_label_on_the_left() {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut hit = ph2d_editor_core::interaction::HitIndex::default();
    let store = ph2d_editor_core::interaction::WidgetStore::default();
    let id = NodeId(0x7_29_0001);
    let (x, w) = (0.0, 300.0);
    ph2d_editor_core::property_row::paint_fields_row(
        &mut scene,
        &mut text,
        ph2d_tokens::Theme::default(),
        &mut hit,
        &store,
        x,
        w,
        0.0,
        "Mass",
        &[id],
        0.1,
        Some(ph2d_editor_core::widget::Unit::Px),
        ph2d_editor_core::property_row::Seccao::apenas_campos(1),
    );
    let r = hit.rect_for(id).expect("a caixa");
    assert!(
        r.x > x + w * 0.25,
        "fora do cartão o rótulo deixou a esquerda: {r:?}"
    );
    assert!(
        store.card_field(id).is_none(),
        "fora do cartão virou campo de cartão"
    );
}

/// Duas caixas na MESMA linha, lado a lado, com estas letras na pega.
fn lado_a_lado(h: &HeroScreen, a: NodeId, b: NodeId, letras: [&str; 2], o_que: &str) {
    let ra = h
        .hit_index
        .rect_for(a)
        .unwrap_or_else(|| panic!("{o_que}: sem a 1.ª caixa"));
    let rb = h
        .hit_index
        .rect_for(b)
        .unwrap_or_else(|| panic!("{o_que}: sem a 2.ª caixa"));
    assert!(
        (ra.y - rb.y).abs() < 0.5 && ra.x + ra.w <= rb.x,
        "{o_que}: as duas caixas não partilham a linha: {ra:?} {rb:?}"
    );
    let pintadas = [a, b].map(ph2d_editor_core::widget::card_field::painted_letter);
    assert_eq!(
        pintadas,
        letras.map(|l| Some(l.to_owned())),
        "{o_que}: as letras das pegas"
    );
}

/// ⭐ **T56 (ordem do dono 02/11): no cartão do Physics Body, dois campos por linha sempre que
/// fizer sentido** — Half Size W · H (não o «H» de «Half»), Offset X · Y, Init Velocity X · Y,
/// Bounce | Friction, Linear | Angular Damping, Freeze X | Freeze Y.
#[test]
fn the_physics_body_card_puts_two_fields_per_row() {
    use ph2d_panel_inspector::ids as i;
    super::o_inspector_armado::arma_tudo();
    let mut info = super::o_inspector_armado::fisica_armada();
    info.kind_tag = 0; // Dynamic: as linhas da dinâmica pintam-se.
    ph2d_panel_inspector::set_current_inspector_physics(Some(info));
    let h = hero_com_a_fisica_aberta();
    lado_a_lado(
        &h,
        i::INSP_PHYS_HALF_X,
        i::INSP_PHYS_HALF_Y,
        ["W", "H"],
        "Half Size",
    );
    lado_a_lado(
        &h,
        i::INSP_PHYS_OFFSET_X,
        i::INSP_PHYS_OFFSET_Y,
        ["X", "Y"],
        "Offset",
    );
    lado_a_lado(
        &h,
        i::INSP_PHYS_LINVEL_X,
        i::INSP_PHYS_LINVEL_Y,
        ["X", "Y"],
        "Init Velocity",
    );
    lado_a_lado(
        &h,
        i::INSP_PHYS_RESTITUTION,
        i::INSP_PHYS_FRICTION,
        ["B", "F"],
        "Bounce | Friction",
    );
    lado_a_lado(
        &h,
        i::INSP_PHYS_LINEAR_DAMPING,
        i::INSP_PHYS_ANGULAR_DAMPING,
        ["D", "D"],
        "Damping",
    );
    let (fx, fy) = (
        h.hit_index
            .rect_for(i::INSP_PHYS_LOCKX[0])
            .expect("Freeze X"),
        h.hit_index
            .rect_for(i::INSP_PHYS_LOCKY[0])
            .expect("Freeze Y"),
    );
    assert!(
        (fx.y - fy.y).abs() < 0.5 && fx.x < fy.x,
        "Freeze X | Freeze Y não partilham a linha: {fx:?} {fy:?}"
    );
    // A dica do arrasto leva o rótulo da linha com a unidade (D13).
    let dica = ph2d_editor_core::panel::cards::dicas::field_hint(&h.store, i::INSP_PHYS_HALF_Y)
        .expect("Half Height não é campo de cartão");
    assert!(dica.contains("Half Size ("), "o rótulo da linha: {dica:?}");
    super::o_inspector_armado::desarma_tudo();
}
