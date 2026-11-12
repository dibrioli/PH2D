//! ⭐ **CartoesUI T57–T59 (smoke do dono 02/11, o Platform Player solto)** — no hero montado, com o
//! cartão do Player ABERTO: arrasta-se da lista para o canvas e move-se solto (T58); as linhas dele
//! nunca se sobrepõem (T59); e o solto mais alto que o canvas rola por dentro (T57).
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{arrasta, centro, pinta};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::loose_rect_of;
use ph2d_editor_core::screens::hero::HeroScreen;

const PLAYER: NodeId = ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION;

/// O hero só com o Inspector e o cartão do Player ABERTO (o armado tem de vir antes).
fn hero_com_o_player_aberto() -> HeroScreen {
    hero_com_aberto(PLAYER)
}

/// O Player em cartões (versão B) nasce com as dobras FECHADAS: o ensaio que mede uma linha de dentro abre a dobra dela.
pub(super) fn abre_dobras(h: &mut HeroScreen, dobras: &[NodeId]) {
    for &d in dobras {
        h.store.set_collapsed(d, false);
        h.store.set_section_open_live(d, 1.0);
    }
    pinta(h, 30);
}

/// O hero só com o Inspector e o cartão `sec` ABERTO.
pub(super) fn hero_com_aberto(sec: NodeId) -> HeroScreen {
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
    pinta(&mut h, 30);
    if h.store.is_collapsed(sec) {
        h.store.toggle_collapsed(sec);
        h.store.set_section_open_live(sec, 1.0);
        pinta(&mut h, 30);
    }
    assert!(!h.store.is_collapsed(sec), "o cartão {sec:?} não abriu");
    // Os cartões de cima têm a altura deles: traz o título para dentro da janela.
    let (painel, grip) = (
        ph2d_editor_core::ids::INSP_PANEL,
        ph2d_editor_core::ids::grip_de(sec),
    );
    let mut rolar = 0.0;
    while h.hit_index.rect_for(grip).is_none() && rolar < 30_000.0 {
        rolar += 100.0;
        h.store.set_panel_scroll(painel, rolar);
        h.store.set_panel_scroll_live(painel, rolar);
        pinta(&mut h, 3);
    }
    h
}

/// A pega do CARTÃO: o título inteiro menos os ícones (a mais larga das registadas com o id).
fn pega(h: &HeroScreen) -> ph2d_editor_core::zones::Rect {
    pega_de(h, PLAYER)
}

fn pega_de(h: &HeroScreen, sec: NodeId) -> ph2d_editor_core::zones::Rect {
    let g = ph2d_editor_core::ids::grip_de(sec);
    h.hit_index
        .iter_registrations()
        .filter(|(id, _)| *id == g)
        .map(|(_, r)| r)
        .max_by(|a, b| a.w.total_cmp(&b.w))
        .expect("o título do Player é a pega do cartão")
}

/// O meio do título do cartão — onde o artista pega.
fn no_titulo(h: &HeroScreen) -> (f32, f32) {
    let p = pega(h);
    (p.x + p.w * 0.4, p.y + p.h * 0.5)
}

/// O Player solto no alto do canvas, com a largura `w`.
fn solto(h: &mut HeroScreen, w: f32) {
    solto_de(h, PLAYER, w);
}

pub(super) fn solto_de(h: &mut HeroScreen, sec: NodeId, w: f32) {
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let p = pega_de(h, sec);
    let de = (p.x + p.w * 0.4, p.y + p.h * 0.5);
    arrasta(
        h,
        de,
        (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.05),
    );
    h.store.loose_card_mut(sec).expect("o cartão ficou solto").w = w;
    pinta(h, 30);
}

/// T58: o cartão do Player ABERTO sai da lista para o canvas e, solto, anda com a mão.
#[test]
fn an_open_component_card_drags_onto_the_canvas_and_moves_there() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_o_player_aberto();
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let (p, g) = (pega(&h), ph2d_editor_core::ids::grip_de(PLAYER));
    for i in 0..8 {
        let (x, y) = (p.x + p.w * (i as f32 + 0.5) / 8.0, p.y + p.h * 0.5);
        assert_eq!(
            h.hit_index.hit(x, y),
            Some(g),
            "o clique em ({x}, {y}) no título do Player aberto não vai à pega do cartão {p:?}"
        );
    }
    let de = no_titulo(&h);
    let longe = (canvas.x + canvas.w * 0.3, canvas.y + canvas.h * 0.2);
    arrasta(&mut h, de, longe);
    let solto = h
        .store
        .loose_card(PLAYER)
        .expect("o Player aberto ficou solto");
    let antes = solto.at;
    let de = no_titulo(&h);
    let ate = (de.0 + 120.0, de.1 + 60.0);
    arrasta(&mut h, de, ate);
    let depois = h
        .store
        .loose_card(PLAYER)
        .expect("o Player continua solto")
        .at;
    assert!(
        (depois.0 - antes.0 - 120.0).abs() < 2.0 && (depois.1 - antes.1 - 60.0).abs() < 2.0,
        "o Player solto e aberto não andou com a mão: {antes:?} → {depois:?}"
    );
    super::o_inspector_armado::desarma_tudo();
}

/// T59: no cartão, cada linha do Player tem o rótulo EM CIMA da caixa — entre duas linhas seguidas
/// (também de uma subsecção para a seguinte) cabe sempre a caixa e o rótulo.
#[test]
fn the_player_card_rows_never_overlap() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_o_player_aberto();
    solto(&mut h, 230.0);
    let mut anterior: Option<(NodeId, ph2d_editor_core::zones::Rect)> = None;
    for (titulo, linhas) in ph2d_panel_inspector::player_card_spans() {
        for id in linhas {
            let Some(r) = h.hit_index.rect_for(id) else {
                continue; // a linha escondida pelo modo (a mola, o empurrão)
            };
            if let Some((a, ra)) = anterior
                && (r.y - ra.y).abs() > 0.5
            {
                assert!(
                    r.y - ra.y >= m::FIELD_H + m::LABEL_PX,
                    "em {titulo}, {id:?} {r:?} cai por cima de {a:?} {ra:?}"
                );
            }
            anterior = Some((id, r));
        }
    }
    super::o_inspector_armado::desarma_tudo();
}

/// T59 (a lei da T56): no cartão, cada par do Player divide uma linha — mesma altura, lado a lado,
/// e a letra de cada caixa é a da última palavra do rótulo dela.
#[test]
fn the_player_card_pairs_share_a_row() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_o_player_aberto();
    abre_dobras(&mut h, &ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS);
    solto(&mut h, 270.0);
    // Rola o solto até ao fim e confere cada par que passa à vista.
    let mut vistos = std::collections::BTreeSet::new();
    let r = h.store.panel_rect(loose_rect_of(PLAYER)).expect("o solto");
    for _ in 0..40 {
        for &(a, b) in ph2d_panel_inspector::player_card_pairs() {
            let (Some(ra), Some(rb)) = (h.hit_index.rect_for(a), h.hit_index.rect_for(b)) else {
                continue; // escondido pelo modo, ou rolado para fora
            };
            assert!(
                (ra.y - rb.y).abs() < 0.5 && ra.x + ra.w <= rb.x,
                "o par n.º {} {a:?} {ra:?} | {b:?} {rb:?} não divide uma linha",
                ph2d_panel_inspector::player_card_pairs()
                    .iter()
                    .position(|p| p.0 == a)
                    .unwrap_or(99)
            );
            vistos.insert(a);
        }
        let arena = bumpalo::Bump::new();
        let (x, y) = centro(r);
        let roda = ph2d_host::WheelEvent {
            x,
            y,
            delta_x: 0.0,
            delta_y: -120.0,
            modifiers: Default::default(),
            timestamp_ns: 0,
        };
        let _ = ph2d_editor_core::interaction::dispatch_wheel(&mut h.store, roda, &arena);
        pinta(&mut h, 4);
    }
    // Os 16 menos os escondidos pelo modo armado (a mola da perna, o empurrão).
    assert!(vistos.len() >= 14, "só {} pares conferidos", vistos.len());
    super::o_inspector_armado::desarma_tudo();
}

/// T59: a roda — Radius | Out Radius (os dois diâmetros do eixo) dividem uma linha no cartão.
#[test]
fn the_wheel_card_puts_its_two_radii_on_one_row() {
    super::o_inspector_armado::arma_tudo();
    let sec = ph2d_editor_core::ids::INSP_LIVE_WHEEL_SECTION;
    let mut h = hero_com_aberto(sec);
    solto_de(&mut h, sec, 270.0);
    let (a, b) = (
        ph2d_panel_inspector::ids::INSP_WHEEL_RADIUS,
        ph2d_panel_inspector::ids::INSP_WHEEL_RADIUS_OUT,
    );
    let ra = h.hit_index.rect_for(a).expect("Radius no cartão");
    let rb = h.hit_index.rect_for(b).expect("Out Radius no cartão");
    assert!(
        (ra.y - rb.y).abs() < 0.5 && ra.x + ra.w <= rb.x,
        "Radius {ra:?} e Out Radius {rb:?} não dividem uma linha"
    );
    super::o_inspector_armado::desarma_tudo();
}

/// T57: o Player solto é mais alto que o canvas ⇒ o cartão pára no fundo do canvas, a roda sobre
/// ele rola-lhe o corpo e o cabeçalho (a pega) fica onde estava.
#[test]
fn a_loose_card_taller_than_the_canvas_scrolls_inside() {
    super::o_inspector_armado::arma_tudo();
    let mut h = hero_com_o_player_aberto();
    abre_dobras(&mut h, &ph2d_panel_inspector::ids::INSP_PLAYER_FOLDS);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    solto(&mut h, 270.0);
    let r = h
        .store
        .panel_rect(loose_rect_of(PLAYER))
        .expect("o Player solto publicou o rect");
    // O fundo é o do canvas — ou o topo do HUD de baixo, que se pinta por cima dele.
    let hud = h.last_layout.expect("houve um quadro").bottom_hud;
    let fundo = (canvas.y + canvas.h).min(hud.y);
    assert!(
        r.y + r.h <= fundo + 0.5,
        "o Player solto {r:?} passa do fundo {fundo} (canvas {canvas:?}, HUD {hud:?})"
    );
    let titulo = pega(&h);
    let campo = ph2d_panel_inspector::ids::INSP_PLAYER_SPEED; // a 1.ª linha do cartão Walk, o de cima
    let antes = h.hit_index.rect_for(campo).expect("a 1.ª linha do Player");
    let arena = bumpalo::Bump::new();
    let (cx, cy) = centro(r);
    let roda = ph2d_host::WheelEvent {
        x: cx,
        y: cy,
        delta_x: 0.0,
        delta_y: -120.0,
        modifiers: Default::default(),
        timestamp_ns: 0,
    };
    let _ = ph2d_editor_core::interaction::dispatch_wheel(&mut h.store, roda, &arena);
    pinta(&mut h, 30);
    assert!(
        h.store.panel_scroll(loose_rect_of(PLAYER)) > 0.0,
        "a roda sobre o Player solto não o rolou"
    );
    let depois = h.hit_index.rect_for(campo).map(|c| c.y);
    assert!(
        depois.is_none_or(|y| y < antes.y),
        "a 1.ª linha não subiu: {antes:?} → {depois:?}"
    );
    assert_eq!(pega(&h).y, titulo.y, "o cabeçalho rolou com o corpo");
    super::o_inspector_armado::desarma_tudo();
}
