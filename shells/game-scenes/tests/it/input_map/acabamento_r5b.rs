//! O acabamento da R5b (as primeiras fotos, `docs/InputMap/fotos/a1_1280.png`): a aba pára acima do HUD de baixo,
//! a escuta é TRACEJADA na A e na B (maquete A2), e com o Play a correr o Try it lê o mapa do JOGADOR.

use super::Mesa;
use super::lista::onze;
use ph2d_app_input_map::ids::ListHit;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_app_input_map::{TAB_ID, View};
use ph2d_editor_core::doc_surface::DocumentSurface;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::board_view;
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key, PlayerKeys};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn key(name: &str) -> Key {
    Key::named_in_script(name).expect("tecla com nome")[0]
}

/// A aba pintada como o hero a pinta numa janela `w×h`: a área do quadro, com a faixa do HUD dita antes.
fn paint_like_the_hero(m: &mut Mesa, (w, h): (f32, f32)) -> (Rect, Rect) {
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, w, h));
    let area = board_view::area(&layout);
    let hud = layout.bottom_hud;
    let mut scene = VectorScene::new();
    let ts = &mut m.ts;
    let (tab, id) = m.hero.documents.active_surface_mut().expect("a aba activa");
    assert_eq!(id, TAB_ID);
    tab.chrome_over(id, Some(hud));
    tab.paint(id, area, &mut scene, ts, &Theme::default());
    (area, hud)
}

#[test]
fn nenhum_rect_da_aba_cruza_o_hud_a_1280_e_a_1600() {
    for view in View::ALL {
        for size in [(1280.0, 720.0), (1600.0, 1000.0)] {
            let mut m = Mesa::aberta(view);
            m.hero.input_map = onze();
            m.frame();
            let (area, hud) = paint_like_the_hero(&mut m, size);
            assert!(
                area.y + area.h > hud.y,
                "{view:?} {size:?}: a área do quadro desce por baixo do HUD (a régua tem o fenómeno)"
            );
            let doc = &m.tab().doc;
            let body = doc.body();
            assert!(
                body.y + body.h <= hud.y,
                "{view:?} {size:?}: o cartão {body:?} entra no HUD {hud:?}"
            );
            // Só a vista que rola recorta: nela conta a parte do texto dentro do recorte.
            let cut = doc.census.clip.map_or(f32::INFINITY, |c| c.y + c.h);
            assert!(
                cut <= hud.y || cut.is_infinite(),
                "{view:?} {size:?}: o recorte em {cut}"
            );
            assert_eq!(
                doc.census.clip.is_some(),
                view == View::TryIt,
                "{view:?}: quem recorta"
            );
            for (r, s, _) in &doc.census.texts {
                let seen = (r.y + r.h).min(cut);
                assert!(
                    seen <= hud.y,
                    "{view:?} {size:?}: «{s}» em {r:?} sob o HUD {hud:?}"
                );
            }
        }
    }
}

#[test]
fn a_escuta_da_lista_e_tracejada() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    let reload = m.hero.input_map.id("reload").expect("fábrica");
    assert!(
        m.tab().doc.census.dashed.is_empty(),
        "sem escuta, nada tracejado"
    );
    m.click(m.at(ListHit::Listen(reload).id()));
    let slot = m.tab().doc.rect_of(ListHit::Listen(reload).id());
    let slot = slot.expect("a caixa da escuta pintada");
    assert_eq!(
        m.tab().doc.census.dashed,
        vec![slot],
        "a caixa à escuta é a tracejada"
    );
}

#[test]
fn o_ladrilho_a_escuta_do_teclado_e_tracejado() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    let reload = m.hero.input_map.id("reload").expect("fábrica");
    m.click(m.at(KbHit::Row(reload).id()));
    assert!(
        m.tab().doc.census.dashed.is_empty(),
        "parado, o «Press a key» não conta"
    );
    m.click(m.at(KbHit::Listen.id()));
    assert_eq!(m.tab().doc.snap.listening, Some(reload));
    let tile = m.tab().doc.rect_of(KbHit::Listen.id()).expect("o ladrilho");
    assert_eq!(
        m.tab().doc.census.dashed,
        vec![tile],
        "o ladrilho à escuta é tracejado"
    );
}

#[test]
fn com_o_play_a_correr_o_try_it_le_o_mapa_do_jogador() {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = ph2d_input::InputMap::with_player_defaults();
    let mut keys = PlayerKeys::new();
    keys.set("jump", vec![Binding::Key(key("k"))]);
    m.jogador.start_with(&m.hero.input_map, keys, None);
    m.sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    m.frame();
    let jump = m.hero.input_map.id("jump").expect("fábrica");
    m.live.keyboard.handle_key_down(key("k"));
    m.frame();
    let on = |m: &Mesa| m.tab().doc.tryit.actions.sample(jump).strength;
    assert!(
        (on(&m) - 1.0).abs() < f32::EPSILON,
        "o K do jogador salta no jogo"
    );
    m.live.keyboard.handle_key_up(key("k"));
    m.live.keyboard.handle_key_down(key("up"));
    m.frame();
    assert!(on(&m) < f32::EPSILON, "a seta do projecto já não é do jump");
    m.sim.world_mut().remove_resource::<ph2d_ecs::RunOnCopy>();
    m.frame();
    assert!(
        m.tab().doc.snap.play_map.is_none(),
        "parado, o mapa do projecto"
    );
}
