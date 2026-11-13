//! As cenas de smoke da aba (`PH2D_GAME_SMOKE` 6..13, plano §8): o código que o roteiro do jogo dá é o mesmo que a
//! família arma, cada cena põe a aba no estado da maquete, e a cena tem os leitores do fire e do laser (o reload não).

use super::Mesa;
use ph2d_app_game::smoke::input_map::{CODIGOS as CENAS, DEAD, PRIMEIRA, arma, codigo, leitores};
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::Key;
use ph2d_input::readers::Reader;

/// A composição da shell (`janela_do_jogo.rs::game_smoke`): arma a aba, instala o mapa e pede a aba.
fn armada(cena: &str) -> Mesa {
    let mut m = Mesa::new(View::Keyboard);
    let map = arma(&mut m.tab_mut().doc, cena).expect("código do roteiro");
    m.hero.input_map = map;
    m.hero.store.request_input_map_tab();
    m.frame();
    m
}

fn id(m: &Mesa, n: &str) -> ph2d_input::ActionId {
    m.hero.input_map.id(n).expect("a acção do mapa do dono")
}

#[test]
fn as_cenas_do_input_map_fecham_o_roteiro_pela_ordem() {
    use ph2d_app_game::smoke::{CENAS as TODAS, input_map::ULTIMA, projectile, topdown};
    assert_eq!(codigo(PRIMEIRA - 1), None);
    assert_eq!(codigo(PRIMEIRA), Some("A1"));
    assert_eq!(codigo(ULTIMA), CENAS.last().copied());
    assert_eq!(codigo(ULTIMA + 1), None);
    // A aba fecha o seu trecho; o Top-Down Player vem logo a seguir e o Projectile Motion fecha o roteiro.
    assert_eq!(
        (topdown::CENA, projectile::CENA, TODAS),
        (ULTIMA + 1, ULTIMA + 2, ULTIMA + 2)
    );
    assert!(arma(&mut Mesa::new(View::List).tab_mut().doc, "Z9").is_none());
}

#[test]
fn cada_cena_abre_a_aba_na_vista_dela_com_as_11_accoes() {
    for c in CENAS {
        let m = armada(c);
        assert!(m.active(), "{c}: a aba aberta e à frente");
        assert_eq!(m.hero.input_map.len(), 11, "{c}");
        let want = match c.as_bytes()[0] {
            b'A' => View::List,
            b'B' => View::Keyboard,
            _ => View::TryIt,
        };
        assert_eq!(m.tab().doc.view(), want, "{c}");
    }
}

#[test]
fn as_cenas_do_try_it_e_do_play_poem_a_tecla_a_manche_e_a_tela_do_jogador() {
    use ph2d_app_game::smoke::input_map::{
        GAME_ID, MANCHE_C3, NA_TELA, abre_a_aba, manche, tecla_do_p1, troca_do_jogador,
    };
    let up = Key::named_in_script("up").and_then(|v| v.first().copied());
    let m = armada("C1");
    assert_eq!(
        m.tab().doc.tryit.confirm,
        None,
        "C1: as receitas, sem pergunta"
    );
    let m = armada("C2");
    assert_eq!(
        m.tab().doc.tryit.held_keys(),
        up.into_iter().collect::<Vec<_>>()
    );
    let c3 = PRIMEIRA + 10;
    assert_eq!(codigo(c3), Some("C3"));
    let (eixo, v) = manche(c3).expect("C3 empurra a manche");
    assert!(eixo == ph2d_input::GamepadAxis::LeftStickX && (v - MANCHE_C3).abs() < 1e-6);
    assert!(manche(c3 - 1).is_none() && manche(c3 + 1).is_none());
    assert!(
        armada("C3")
            .hero
            .input_map
            .actions()
            .iter()
            .any(|a| a.is_analog())
    );
    for c in ["C4", "P1"] {
        let m = armada(c);
        let map = &m.hero.input_map;
        assert!(map.controls().players_can_change_keys, "{c}");
        let vistas: Vec<_> = (map.actions().iter())
            .filter(|a| map.controls().shown_in_game(a.id))
            .map(|a| a.name.as_str())
            .collect();
        assert_eq!(vistas, NA_TELA, "{c}");
    }
    let m = armada("C4");
    assert_eq!(
        troca_do_jogador(&m.hero.input_map).changed(&m.hero.input_map),
        1
    );
    let mut sim = ph2d_ecs::SimWorld::new();
    let _ = leitores(&mut sim, PRIMEIRA + 11, |s| s.world_mut().spawn(()).id());
    let id = sim
        .world()
        .get_resource::<ph2d_app_input_map::ph2d_player_controls::GameId>();
    assert_eq!(id.map(|g| g.0), Some(GAME_ID));
    let p1 = PRIMEIRA + 12;
    assert!(!abre_a_aba("P1") && abre_a_aba("C4"));
    let teclas: Vec<_> = (0..400).filter_map(|q| tecla_do_p1(p1, q)).collect();
    assert_eq!(teclas.len(), 5, "abre, desce, desce, direita, Enter");
    assert!((0..400).all(|q| tecla_do_p1(p1 - 1, q).is_none()));
    // As teclas do condutor, pela porta da sessão do Play: a tela abre, desce ao Fire, 2.ª tecla, à escuta.
    let mut m = armada("P1");
    let map = m.hero.input_map.clone();
    m.jogador.start_with(&map, troca_do_jogador(&map), None);
    for k in teclas {
        let _ = m.jogador.key(k, true, &map);
    }
    let tela = m.jogador.screen();
    assert!(tela.is_open() && tela.listening(), "P1: aberta e à escuta");
    assert_eq!(m.jogador.rows(&map)[tela.selected()].action, "fire");
    assert_eq!(tela.slot(), 1, "a 2.ª tecla do Fire");
}

/// A C4 põe «a player changed 1 key» em MEMÓRIA: nenhuma cena escreve sob a pasta das prefs (a do `$HOME`, a que a
/// aba usa por omissão), corra ela dentro ou fora do `fotografa_cena.sh`.
#[test]
fn a_cena_c4_mostra_a_troca_do_jogador_sem_escrever_no_home() {
    use ph2d_app_game::smoke::input_map::GAME_ID;
    use ph2d_app_input_map::ph2d_player_controls::{home_base, player_file};
    let carimbo = |p: &std::path::Path| {
        std::fs::metadata(p)
            .ok()
            .map(|m| (m.len(), m.modified().ok()))
    };
    let ficheiro = home_base().map(|b| player_file(&b, GAME_ID));
    let antes = ficheiro.as_deref().map(carimbo);
    for c in CENAS {
        let m = armada(c);
        assert_eq!(
            m.tab().doc.tryit.base,
            home_base(),
            "{c}: a pasta é a de omissão"
        );
        if c == "C4" {
            assert_eq!(
                m.tab().doc.tryit.player_changed,
                1,
                "C4: 1 troca, lida da memória"
            );
        }
    }
    assert_eq!(
        ficheiro.as_deref().map(carimbo),
        antes,
        "nenhuma cena tocou no ficheiro do jogador no HOME"
    );
}

#[test]
fn as_cenas_da_lista_escutam_filtram_e_mostram_a_manche() {
    let m = armada("A2");
    assert_eq!(m.tab().doc.snap.listening, Some(id(&m, "reload")));
    let m = armada("A3");
    let shown = ph2d_app_input_map::view_list::shown(&m.tab().doc);
    assert_eq!(shown, vec![id(&m, "jump"), id(&m, "move_up")]);
    let m = armada("A4");
    let ml = m
        .hero
        .input_map
        .get(id(&m, "move_left"))
        .expect("move_left");
    assert!(ml.is_analog() && (ml.dead_zone - DEAD).abs() < 1e-6);
    let a1 = armada("A1");
    assert!(a1.hero.input_map.actions().iter().all(|a| !a.is_analog()));
}

#[test]
fn as_cenas_do_teclado_abrem_a_accao_e_o_desenho_delas() {
    let up = Key::named_in_script("up").and_then(|v| v.first().copied());
    let m = armada("B1");
    assert_eq!(m.tab().doc.kb.open, Some(id(&m, "jump")));
    let m = armada("B2");
    assert_eq!(m.tab().doc.kb.hover, up.map(KbHit::Key));
    let m = armada("B3");
    assert_eq!(m.tab().doc.kb.sub, Sub::Gamepad);
    assert_eq!(m.tab().doc.kb.open, Some(id(&m, "move_left")));
    let m = armada("B4");
    let reload = id(&m, "reload");
    assert_eq!(m.tab().doc.kb.open, Some(reload));
    assert!(
        m.hero
            .input_map
            .get(reload)
            .is_some_and(|a| a.bindings.is_empty())
    );
}

#[test]
fn a_cena_tem_um_leitor_do_fire_e_do_laser_e_nenhum_do_reload() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let _ = leitores(&mut sim, PRIMEIRA, |s| s.world_mut().spawn(()).id());
    let readers = ph2d_app_input_map::readers::of_world(sim.world());
    let fire = readers.get("fire").map_or(&[][..], Vec::as_slice);
    assert!(fire.iter().any(|r| matches!(r, Reader::Trigger { .. })));
    let laser = readers.get("laser").map_or(&[][..], Vec::as_slice);
    assert!(laser.iter().any(|r| matches!(r, Reader::Script { .. })));
    assert!(readers.get("reload").is_none_or(Vec::is_empty));
}

/// As P2..P4 abrem a tela só com a tecla dela e põem o ponteiro pela porta do host, depois de a tela pintada: P2 =
/// a linha Jump acesa; P3 = o «Reset to defaults» premido; P4 = premido e arrastado para fora (o premido apaga).
#[test]
fn as_cenas_p2_a_p4_poem_o_ponteiro_pela_porta_do_host() {
    use ph2d_app_game::smoke::input_map::{abre_a_aba, rato_do_p, tecla_do_p1, troca_do_jogador};
    use ph2d_app_input_map::ph2d_player_controls::Hover;
    let p2 = PRIMEIRA + 14;
    for (n, c) in [(p2, "P2"), (p2 + 1, "P3"), (p2 + 2, "P4")] {
        assert_eq!(codigo(n), Some(c));
        assert!(!abre_a_aba(c), "{c}: o jogo na vista da cena");
        let mut m = armada(c);
        let map = m.hero.input_map.clone();
        m.jogador.start_with(&map, troca_do_jogador(&map), None);
        let teclas: Vec<_> = (0..400).filter_map(|q| tecla_do_p1(n, q)).collect();
        assert_eq!(teclas.len(), 1, "{c}: só a tecla que abre");
        assert!(
            rato_do_p(n, 160, &mut m.jogador, &map).is_none(),
            "{c}: fechada, nada"
        );
        for k in teclas {
            let _ = m.jogador.key(k, true, &map);
        }
        let pinta = |m: &mut Mesa| {
            let mut scene = ph2d_vector::VectorScene::new();
            let t = ph2d_tokens::Theme::default();
            m.jogador.paint(super::AREA, &map, &mut scene, &mut m.ts, t);
        };
        pinta(&mut m);
        let pontos: Vec<_> = (0..400)
            .filter_map(|q| rato_do_p(n, q, &mut m.jogador, &map))
            .collect();
        pinta(&mut m);
        let tela = m.jogador.screen();
        assert!(
            tela.is_open() && !tela.listening(),
            "{c}: aberta, sem escutar"
        );
        let jump = m.jogador.rows(&map).iter().position(|r| r.action == "jump");
        match c {
            "P2" => assert_eq!(
                tela.hovered(),
                jump.map(Hover::Row),
                "P2: a linha Jump acesa"
            ),
            "P3" => assert!(tela.reset_pressed() && tela.hovered() == Some(Hover::Reset)),
            _ => assert!(
                !tela.reset_pressed() && tela.hovered().is_none(),
                "P4: o premido apaga"
            ),
        }
        assert_eq!(pontos.len(), if c == "P4" { 2 } else { 1 }, "{c}");
    }
    assert!(
        rato_do_p(
            PRIMEIRA + 12,
            160,
            &mut Mesa::new(View::List).jogador,
            &Default::default()
        )
        .is_none()
    );
}

/// A P5 (dono 10/11, D32): a tela aberta e uma linha acesa, o condutor pousa o ponteiro no centro do «Controls» da
/// barra — o de `rects` (editor) e o de `rects_palco` (tela cheia) — e o movimento é da barra: o hero acende o botão e
/// a linha apaga. Controlo: sem a barra pintada (índice limpo), noutro quadro ou noutra cena, nenhum movimento.
#[test]
fn a_cena_p5_poe_o_ponteiro_no_controls_da_barra() {
    use ph2d_app_game::smoke::input_map::{rato_do_p5, tecla_do_p1, troca_do_jogador};
    use ph2d_app_input_map::ph2d_player_controls::{Hover, host};
    use ph2d_editor_core::game_stage::GameRun;
    use ph2d_editor_core::screens::hero::{menu_bar::MENU_BAR_H, transport_bar};
    use ph2d_editor_core::zones::Rect;
    let p5 = PRIMEIRA + 17;
    assert_eq!(codigo(p5), Some("P5"));
    let a = super::AREA;
    let bar = Rect::new(a.x, a.y, a.w, MENU_BAR_H);
    let play_w = transport_bar::play_width(&mut ph2d_text::TextSystem::new());
    let modos = [
        transport_bar::rects(bar, a.w * 0.25, a.w * 0.75, play_w),
        transport_bar::rects_palco(a),
    ];
    for botoes in modos {
        let mut m = armada("P5");
        let map = m.hero.input_map.clone();
        m.jogador.start_with(&map, troca_do_jogador(&map), None);
        for k in (0..400).filter_map(|q| tecla_do_p1(p5, q)) {
            let _ = m.jogador.key(k, true, &map);
        }
        let mut scene = ph2d_vector::VectorScene::new();
        let t = ph2d_tokens::Theme::default();
        m.jogador.paint(a, &map, &mut scene, &mut m.ts, t);
        m.hero.hit_index.clear_for_frame();
        assert!(
            rato_do_p5(p5, 160, &m.hero).is_none(),
            "sem a barra pintada"
        );
        let (hits, store) = (&mut m.hero.hit_index, &m.hero.store);
        transport_bar::paint(
            &botoes,
            GameRun::default(),
            &mut scene,
            &mut m.ts,
            t,
            hits,
            store,
        );
        assert!(
            rato_do_p5(p5, 159, &m.hero).is_none() && rato_do_p5(p5 - 1, 160, &m.hero).is_none()
        );
        let lay = m.jogador.layout().expect("a tela pintada").clone();
        let jump = m.jogador.rows(&map).iter().position(|r| r.action == "jump");
        let linha = lay.rows[jump.expect("jump na tela") - lay.first].band;
        let _ = host::pointer_move(&mut m.jogador, Some((linha.x + 1.0, linha.y + 1.0)));
        assert!(
            matches!(m.jogador.screen().hovered(), Some(Hover::Row(_))),
            "controlo: a linha acesa"
        );
        let e = rato_do_p5(p5, 160, &m.hero).expect("a barra pintada");
        let at = m.hero.escala().to_logical((e.x, e.y));
        let r = botoes.last().expect("o Controls").2;
        assert!((at.0 - (r.x + r.w * 0.5)).abs() < 0.5 && (at.1 - (r.y + r.h * 0.5)).abs() < 0.5);
        let dela = host::pointer_move_com_a_barra(&mut m.jogador, Some(&mut m.hero), Some(at)).0;
        assert!(!dela, "sobre o botão o movimento é da barra");
        let arena = bumpalo::Bump::new();
        let _ = m.hero.handle_pointer_fisico(e, &mut m.ts, &arena);
        assert_eq!(
            m.hero.store.hot_id(),
            Some(transport_bar::CONTROLS.0),
            "o botão acende"
        );
        assert_eq!(
            m.jogador.screen().hovered(),
            None,
            "nenhuma linha da tela acesa"
        );
    }
}
