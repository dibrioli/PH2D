//! ⭐⭐ **C5 (CartoesUI): o cartão SOLTO no canvas** (`docs/CartoesUI/02_especificacao.md` §8) — no
//! hero montado: arrastar o cartão para fora da coluna da lista solta-o sobre o canvas (o rect dele
//! leva o clique, na lista fica a linha-fantasma); «Return to panel» devolve-o; e arrastá-lo de
//! volta à coluna reencaixa-o.
#![cfg(feature = "panel-skeleton")]

use ph2d_editor_core::interaction::{loose_ghost_of, loose_pin_of, loose_rect_of, loose_return_of};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_skeleton::cards::CARD_TOOL;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

pub(super) fn pinta(h: &mut HeroScreen, n: usize) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..n {
        scene.reset();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || {
                ph2d_editor_core::screens::hero::paint_hero_screen(
                    h, VIEWPORT, &mut scene, &mut text,
                )
            },
        );
    }
}

pub(super) fn mao(h: &mut HeroScreen, kind: PointerKind, (x, y): (f32, f32)) {
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

/// Agarra `de`, anda até `ate` em passos (pintando entre eles, como o app) e larga.
pub(super) fn arrasta(h: &mut HeroScreen, de: (f32, f32), ate: (f32, f32)) {
    mao(h, PointerKind::Down, de);
    pinta(h, 2);
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        mao(
            h,
            PointerKind::Move,
            (de.0 + (ate.0 - de.0) * t, de.1 + (ate.1 - de.1) * t),
        );
        pinta(h, 2);
    }
    mao(h, PointerKind::Up, ate);
    pinta(h, 30);
}

pub(super) fn bones() -> ph2d_editor_core::NodeId {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == "skeleton")
            .map(|p| p.manifest.panel_node_id)
            .expect("o Bones está registado")
    })
}

pub(super) fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

pub(super) fn hero_com_o_bones() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_skeleton::state::set_current_bone(None);
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    pinta(&mut h, 60);
    h
}

#[test]
fn a_card_dragged_out_of_the_list_floats_on_the_canvas() {
    let mut h = hero_com_o_bones();
    let grip = ph2d_editor_core::ids::grip_de(CARD_TOOL);
    let pega = h.hit_index.rect_for(grip).expect("o título do Tool");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let longe = (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.4);
    arrasta(&mut h, de, longe);

    let solto = h.store.loose_card(CARD_TOOL).expect("o Tool ficou solto");
    let r = h
        .store
        .panel_rect(loose_rect_of(CARD_TOOL))
        .expect("o solto publicou o rect");
    assert!(
        r.contains(longe.0, longe.1),
        "o cartão {r:?} não ficou onde a mão o largou {longe:?} ({solto:?})"
    );
    assert_eq!(
        h.chrome_panel_at(longe.0, longe.1),
        Some(loose_rect_of(CARD_TOOL)),
        "o clique sobre o cartão solto foi para o canvas"
    );
    let fantasma = h
        .hit_index
        .rect_for(loose_ghost_of(CARD_TOOL))
        .expect("a linha-fantasma na lista");
    // A linha-fantasma (34 px, `LOOSE_GHOST_H`) é mais baixa que o cartão que lá estava.
    assert!(
        fantasma.h > 0.0 && fantasma.h < r.h,
        "a linha-fantasma {fantasma:?} contra o cartão {r:?}"
    );
    let nova = h
        .hit_index
        .rect_for(grip)
        .expect("a pega anda com o cartão");
    assert!(
        r.contains(nova.x + 1.0, nova.y + 1.0),
        "pega {nova:?} fora de {r:?}"
    );

    // «Return to panel» devolve-o à lista, e o rect sai.
    let volta = h
        .hit_index
        .rect_for(loose_return_of(CARD_TOOL))
        .expect("o ícone Return to panel");
    mao(&mut h, PointerKind::Down, centro(volta));
    mao(&mut h, PointerKind::Up, centro(volta));
    pinta(&mut h, 30);
    assert_eq!(h.store.loose_card(CARD_TOOL), None, "não voltou");
    assert_eq!(h.store.panel_rect(loose_rect_of(CARD_TOOL)), None);
    assert_eq!(h.hit_index.rect_for(loose_ghost_of(CARD_TOOL)), None);
}

#[test]
fn a_loose_card_dragged_back_into_the_column_docks_again() {
    let mut h = hero_com_o_bones();
    let grip = ph2d_editor_core::ids::grip_de(CARD_TOOL);
    let pega = h.hit_index.rect_for(grip).expect("o título do Tool");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    arrasta(
        &mut h,
        de,
        (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.4),
    );
    assert!(h.store.loose_card(CARD_TOOL).is_some(), "não soltou");
    let pega = h.hit_index.rect_for(grip).expect("a pega do solto");
    arrasta(&mut h, (pega.x + 12.0, pega.y + pega.h * 0.5), de);
    assert_eq!(h.store.loose_card(CARD_TOOL), None, "não reencaixou");
    // O lugar é o do Kanban (decide o CENTRO do cartão); o que se exige é que volte à coluna.
    let pega = h.hit_index.rect_for(grip).expect("a pega na lista");
    let lista = h.store.panel_rect(bones()).expect("a lista");
    assert!(
        pega.x >= lista.x && pega.x + pega.w <= lista.x + lista.w,
        "o Tool voltou para {pega:?}, fora da lista {lista:?}"
    );
}

/// ⭐ Achado da C5 (02/11): o cabeçalho de um cartão registava-se DEPOIS dos ícones dele e cobria-os
/// (o `hit` anda do fim para o princípio) — o «Rest Pose» do cartão Bone, na LISTA, não recebia o
/// clique. O ícone é o alvo, e o clique sai como `Click` dele.
#[test]
fn a_card_header_icon_takes_its_own_click() {
    let mut h = hero_com_o_bones();
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    pinta(&mut h, 30);
    let id = ph2d_editor_core::ids::VECTOR_BONE_REST_APPLY;
    let r = h
        .hit_index
        .rect_for(id)
        .expect("o ícone Rest Pose do cartão Bone");
    let c = centro(r);
    assert_eq!(
        h.hit_index.hit(c.0, c.1),
        Some(id),
        "o cabeçalho cobre o ícone"
    );
    ph2d_panel_skeleton::state::set_current_bone(None);
}

fn painel(id: &str) -> ph2d_editor_core::NodeId {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == id)
            .map(|p| p.manifest.panel_node_id)
            .expect("o painel está registado")
    })
}

/// ⭐ Aberto da C5 (02/11): o solto SUMIA quando a aba do painel dele ficava atrás (quem o pinta é o
/// painel, e o painel escondido por uma aba não pintava). O solto continua no canvas e no clique; a
/// lista escondida não volta a reservar coluna nem a levar cliques.
#[test]
fn a_loose_card_stays_on_the_canvas_when_its_panel_tab_is_behind() {
    let mut h = hero_com_o_bones();
    let bones = bones();
    h.panel_visibility.insert("inspector", true);
    pinta(&mut h, 5);
    h.store.bump_panel_z(bones);
    pinta(&mut h, 20);
    let grip = ph2d_editor_core::ids::grip_de(CARD_TOOL);
    let pega = h.hit_index.rect_for(grip).expect("o título do Tool");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let longe = (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.4);
    arrasta(&mut h, de, longe);
    assert!(h.store.loose_card(CARD_TOOL).is_some(), "não soltou");
    pinta(&mut h, 5);
    let alto = h
        .store
        .panel_rect(loose_rect_of(CARD_TOOL))
        .expect("rect")
        .h;

    // A aba do Inspector passa à frente: o Bones fica atrás.
    h.store.bump_panel_z(painel("inspector"));
    pinta(&mut h, 10);
    assert_eq!(
        h.chrome_panel_at(longe.0, longe.1),
        Some(loose_rect_of(CARD_TOOL)),
        "o solto sumiu com a aba do Bones atrás"
    );
    let nova = h
        .hit_index
        .rect_for(grip)
        .expect("a pega do solto pinta-se");
    let r = h
        .store
        .panel_rect(loose_rect_of(CARD_TOOL))
        .expect("o rect do solto");
    assert!(
        r.contains(nova.x + 1.0, nova.y + 1.0),
        "pega {nova:?} fora de {r:?}"
    );
    // T51 (smoke do dono 02/11): com a aba atrás o corpo perdia o fundo — a altura com que o fundo
    // se pinta vinha do rect publicado, que a aba escondida apaga antes do passo dos soltos. A altura
    // é estado do CARTÃO. *Mutação: a pintura não a grava no cartão ⇒ o cabeçalho.*
    let guardada = h.store.loose_card(CARD_TOOL).expect("solto").altura();
    assert!(
        (r.h - alto).abs() < 0.5
            && (guardada - alto).abs() < 0.5
            && alto > ph2d_card_stack::metrics::CARD_HEAD_H,
        "a aba atrás: rect {} · cartão {guardada} ≠ {alto} com ela à frente",
        r.h
    );
    assert_eq!(
        h.store.panel_rect(bones),
        None,
        "a lista escondida publicou o rect dela"
    );
    assert_eq!(
        h.hit_index.rect_for(loose_ghost_of(CARD_TOOL)),
        None,
        "a linha-fantasma da lista escondida pintou-se"
    );
}

/// ⭐ D31 (dono, 02/11, foto): a lista estava encaixada SOBRE a régua de cima. Fica 5 px abaixo da
/// régua e 5 px da borda direita da janela.
#[test]
fn the_card_list_sits_below_the_ruler_and_off_the_window_edge() {
    let mut h = hero_com_o_bones();
    // As réguas só se desenham com a projeção da grelha (a do app).
    h.set_grid_view(Some(ph2d_editor_core::GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: VIEWPORT.w,
        window_h: VIEWPORT.h,
        canvas: VIEWPORT,
    }));
    pinta(&mut h, 5);
    assert!(h.rulers_live(), "a cena precisa das réguas ligadas");
    let lista = h.store.panel_rect(bones()).expect("a lista");
    let gap = 5.0; // a palavra do dono
    let regua = h.last_content.y;
    assert!(
        regua > h.last_layout.expect("layout").draw_area.y,
        "sem régua de cima"
    );
    assert!(
        (lista.y - (regua + gap)).abs() < 0.5,
        "a lista começa em {} e a régua acaba em {regua}",
        lista.y
    );
    let borda = VIEWPORT.x + VIEWPORT.w;
    assert!(
        (borda - (lista.x + lista.w) - gap).abs() < 0.5,
        "a lista {lista:?} contra a borda {borda}"
    );
    pinta(&mut h, 1);
}

/// O Kanban: arrastar um cartão para a borda de baixo de uma lista que transborda rola-a.
#[test]
fn dragging_a_card_to_the_bottom_edge_scrolls_the_list() {
    const BAIXA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 500.0,
    };
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut pinta_baixa = |h: &mut HeroScreen, n: usize| {
        for _ in 0..n {
            scene.reset();
            ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || {
                    ph2d_editor_core::screens::hero::paint_hero_screen(
                        h, BAIXA, &mut scene, &mut text,
                    )
                },
            );
        }
    };
    pinta_baixa(&mut h, 40);
    let (Some(conteudo), Some(visivel)) = (
        h.store.panel_content_h(bones()),
        h.store.panel_visible_h(bones()),
    ) else {
        panic!("a lista não publicou as alturas da rolagem");
    };
    assert!(conteudo > visivel, "a lista cabe: {conteudo} ≤ {visivel}");
    let lista = h
        .store
        .panel_rect(bones())
        .expect("o Bones publicou o rect");
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(CARD_TOOL))
        .expect("o título do Tool");
    let de = (pega.x + 12.0, pega.y + pega.h * 0.5);
    let fundo = (de.0, lista.y + lista.h - 6.0);
    assert_eq!(h.store.panel_scroll_target(bones()), 0.0, "rolou sem mão");
    mao(&mut h, PointerKind::Down, de);
    for i in 1..=8 {
        let t = i as f32 / 8.0;
        mao(
            &mut h,
            PointerKind::Move,
            (de.0, de.1 + (fundo.1 - de.1) * t),
        );
        pinta_baixa(&mut h, 2);
    }
    pinta_baixa(&mut h, 30);
    assert!(
        h.store.panel_scroll(bones()) > 0.0,
        "a mão parada na borda de baixo não rolou a lista"
    );
    mao(&mut h, PointerKind::Up, fundo);
}

pub(super) fn clica(h: &mut HeroScreen, id: ph2d_editor_core::NodeId) {
    let r = h.hit_index.rect_for(id).expect("o botão está pintado");
    mao(h, PointerKind::Down, centro(r));
    mao(h, PointerKind::Up, centro(r));
    pinta(h, 3);
}

pub(super) fn osso_em_foco(bits: u64, nome: &str, mix: f64) {
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    ph2d_panel_skeleton::set_current_bone_subject(Some(bits));
    ph2d_panel_skeleton::set_current_bone_name(nome.to_owned());
    ph2d_panel_skeleton::set_current_bone_ik(Some((
        mix,
        0.0,
        2.0,
        ph2d_skeleton::BendSide::ALL[0],
    )));
}

pub(super) fn solta(h: &mut HeroScreen, secao: ph2d_editor_core::NodeId) {
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(secao))
        .expect("o título do cartão");
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    arrasta(
        h,
        (pega.x + 12.0, pega.y + pega.h * 0.5),
        (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.3),
    );
    assert!(
        h.store.loose_card(secao).is_some(),
        "o cartão não ficou solto"
    );
}

/// T7: o alfinete prende o cartão ao osso de agora — com outro osso escolhido ele continua a
/// mostrar o dele, e a pressão sobre ele faz dos pedidos pedidos desse osso.
#[test]
fn a_pinned_card_shows_and_asks_its_own_bone() {
    let mut h = hero_com_o_bones();
    let ik = ph2d_panel_skeleton::cards::CARD_IK;
    osso_em_foco(2, "Bone 2", 0.9);
    pinta(&mut h, 20);
    solta(&mut h, ik);
    clica(&mut h, loose_pin_of(ik));
    assert_eq!(
        h.store.loose_card(ik).and_then(|c| c.pin),
        Some(2),
        "o alfinete não prendeu ao osso de agora"
    );
    // Outro osso escolhido; a shell publica o preso pelas mesmas portas.
    osso_em_foco(3, "Bone 3", 0.2);
    ph2d_panel_skeleton::publica_presos(&h.store.loose_pins_of(bones()), |b| {
        osso_em_foco(b, "Bone 2", 0.9);
    });
    pinta(&mut h, 5);
    assert_eq!(h.store.card_subject_name(2), Some("Bone 2"));
    let r = h
        .store
        .panel_rect(loose_rect_of(ik))
        .expect("o preso está no canvas");
    mao(
        &mut h,
        PointerKind::Down,
        (r.x + r.w - 12.0, r.y + r.h - 4.0),
    );
    assert_eq!(
        h.store.card_press_subject(),
        Some(2),
        "a pressão no cartão preso não é do osso dele"
    );
    mao(&mut h, PointerKind::Up, (r.x + r.w - 12.0, r.y + r.h - 4.0));
    clica(&mut h, loose_pin_of(ik));
    let corda = h.store.loose_card(ik).map(|c| (c.pin, c.rope));
    assert_eq!(
        corda,
        Some((Some(2), true)),
        "T44: o 2.º clique põe a corda"
    );
    clica(&mut h, loose_pin_of(ik));
    assert_eq!(h.store.loose_card(ik).and_then(|c| c.pin), None);
    pinta(&mut h, 5);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    mao(
        &mut h,
        PointerKind::Down,
        (canvas.x + 20.0, canvas.y + canvas.h - 20.0),
    );
    assert_eq!(
        h.store.card_press_subject(),
        None,
        "a pressão fora ficou presa"
    );
}
