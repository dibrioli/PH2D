//! ⭐ **R4: uma lista aberta fica por cima de tudo**, por pintura real do hero. A foto
//! `docs/CartoesUI/maquete/j_timeline_track_aberto.png` mostrava o HUD a tapar «Opacity» na lista de
//! «+ Track». A ordem lê-se na gravação da cena: nada do HUD pode vir depois da lista.
#![cfg(feature = "panel-timeline")]

use super::o_cartao_solto_no_canvas::{clica, pinta};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::ids as tl;
use ph2d_text::TextSystem;
use ph2d_vector::{DrawOp, Shape as _, VectorScene};

fn hero_com(painel: &'static str) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == painel);
    }
    h.view.stats_visible = true;
    pinta(&mut h, 30);
    h
}

/// As caixas que o quadro pintou, pela ordem em que ficam na cena.
fn caixas_do_quadro(h: &mut HeroScreen) -> Vec<Rect> {
    let mut scene = VectorScene::new();
    scene.record();
    let mut text = TextSystem::without_system_fonts();
    let viewport = Rect::new(0.0, 0.0, 1600.0, 900.0);
    ph2d_editor_core::screens::hero::paint_hero_screen(h, viewport, &mut scene, &mut text);
    scene
        .take_record()
        .into_iter()
        .filter_map(|op| match op {
            DrawOp::Fill { path, .. } => {
                let b = path.bounding_box();
                #[expect(clippy::cast_possible_truncation, reason = "píxeis do ecrã")]
                Some(Rect::new(
                    b.x0 as f32,
                    b.y0 as f32,
                    b.width() as f32,
                    b.height() as f32,
                ))
            }
            _ => None,
        })
        .collect()
}

/// A última caixa pintada com o tamanho de `r` (a moldura dele, tolerando o traço).
fn ordem(caixas: &[Rect], r: Rect, nome: &str) -> usize {
    let igual = |c: &Rect| {
        (c.x - r.x).abs() < 2.0
            && (c.y - r.y).abs() < 2.0
            && (c.w - r.w).abs() < 3.0
            && (c.h - r.h).abs() < 3.0
    };
    caixas
        .iter()
        .rposition(igual)
        .unwrap_or_else(|| panic!("{nome} {r:?} não foi pintado"))
}

fn o_hud(h: &HeroScreen) -> Rect {
    let mut text = TextSystem::without_system_fonts();
    let layout = h.last_layout.expect("houve um quadro");
    ph2d_editor_core::screens::hero::bottom_hud_rect(&layout, &h.store, &mut text, h.stats)
}

/// A lista de «+ Track» (pintada à mão) vem depois do HUD; e a linha dela ganha o clique.
/// *Mutação: `fecha` antes do HUD, ou o pintor a escrever direto em `ctx.scene` ⇒ vermelho.*
#[test]
fn the_add_track_list_paints_after_the_hud() {
    let mut h = hero_com("timeline");
    clica(&mut h, tl::TIMELINE_ADD_TRACK);
    pinta(&mut h, 2);
    let caixas = caixas_do_quadro(&mut h);
    let (primeira, _) = tl::ADDPROP_BUTTONS[0];
    let r = h.hit_index.rect_for(primeira).expect("a 1.ª linha");
    let (ultima, _) = tl::ADDPROP_BUTTONS[tl::ADDPROP_BUTTONS.len() - 1];
    let u = h.hit_index.rect_for(ultima).expect("a última linha");
    let lista = Rect::new(r.x, r.y, r.w, u.y + u.h - r.y);
    let (l, hud) = (
        ordem(&caixas, lista, "a lista"),
        ordem(&caixas, o_hud(&h), "o HUD"),
    );
    assert!(l > hud, "a lista ({l}) ficou sob o HUD ({hud})");
    assert_eq!(h.hit_index.hit(r.x + 2.0, r.y + 2.0), Some(primeira));
}

/// A lista da porta comum (o filtro da Hierarquia) também. *Mutação: a porta sem `por_cima` ⇒ vermelho.*
#[cfg(feature = "panel-hierarchy")]
#[test]
fn a_door_list_paints_after_the_hud() {
    let mut h = hero_com("hierarchy");
    clica(&mut h, ph2d_panel_hierarchy::ids::HIER_FILTER_DD);
    pinta(&mut h, 2);
    let aberta = h.store.dropdown_popover().expect("a lista não abriu");
    let caixas = caixas_do_quadro(&mut h);
    let (l, hud) = (
        ordem(&caixas, aberta.1, "a lista"),
        ordem(&caixas, o_hud(&h), "o HUD"),
    );
    assert!(l > hud, "a lista ({l}) ficou sob o HUD ({hud})");
}

/// ⭐ O clique fora da lista de «+ Track» fecha-a e continua a valer no que acertou (visto em janela real,
/// F8: ela só fechava no próprio chip). *Mutação: tirar `dismiss_own_list` do Down ⇒ vermelho.*
#[test]
fn a_click_outside_closes_the_add_track_list() {
    let mut h = hero_com("timeline");
    clica(&mut h, tl::TIMELINE_ADD_TRACK);
    pinta(&mut h, 2);
    let (primeira, _) = tl::ADDPROP_BUTTONS[0];
    assert!(h.hit_index.rect_for(primeira).is_some(), "abriu");
    let down = |h: &mut HeroScreen, kind, xy| super::o_cartao_solto_no_canvas::mao(h, kind, xy);
    // Fora = o meio do canvas. Era `(900, 20)`, que na barra de menus versão C é o campo «Find
    // anything» (centro da faixa): o clique abria a busca e ela comia o clique seguinte no chip.
    let c = h.last_canvas;
    let fora = (c.x + c.w * 0.5, c.y + c.h * 0.5);
    down(&mut h, ph2d_host::PointerKind::Down, fora);
    down(&mut h, ph2d_host::PointerKind::Up, fora);
    pinta(&mut h, 2);
    assert!(
        h.hit_index.rect_for(primeira).is_none(),
        "o clique de fora tem de a fechar"
    );
    clica(&mut h, tl::TIMELINE_ADD_TRACK);
    pinta(&mut h, 2);
    assert!(
        h.hit_index.rect_for(primeira).is_some(),
        "volta a abrir à primeira"
    );
}
