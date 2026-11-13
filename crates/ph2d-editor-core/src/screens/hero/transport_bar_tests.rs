//! Os botões do jogo: no centro, sem pisar nos vizinhos, e acesos pelo estado da corrida.

use super::*;
use crate::interaction::{HitIndex, WidgetStore};

const BARRA: Rect = Rect::new(0.0, 0.0, 1000.0, 24.0);

/// A largura do Play com nome nestes testes (a real mede-se contra o texto: `play_width`).
const PLAY_W: f32 = 60.0;

/// Versão C (dono 12/11): o centro é do campo de busca; o jogo encosta à direita.
#[test]
fn ficam_encostados_a_direita() {
    let r = rects(BARRA, 100.0, 800.0, PLAY_W);
    let ids: Vec<_> = r.iter().map(|(id, ..)| *id).collect();
    assert_eq!(
        ids,
        [
            ids::TOPBAR_PLAY_BUTTON,
            ids::MENUBAR_PLAY_MENU,
            ids::TOPBAR_PAUSE,
            ids::TOPBAR_RESET,
            CONTROLS.0
        ],
        "o Play com nome, a seta, Pause, Stop e o Controls"
    );
    let fim = r[4].2.x + r[4].2.w;
    assert!(fim <= 800.0 && 800.0 - fim <= Spacing::Sm.px() + 0.01);
    assert!(
        (r[0].2.w - PLAY_W).abs() < 0.01,
        "o Play tem a largura do nome"
    );
    for par in r.windows(2) {
        assert!(par[0].2.x + par[0].2.w <= par[1].2.x + 0.01, "pisam-se");
    }
}

/// Mutação: tirar o `if ate - de < total` ⇒ os botões caem sobre o que está à esquerda.
#[test]
fn nao_pisam_no_vizinho_da_esquerda() {
    let r = rects(BARRA, 100.0, 520.0, PLAY_W);
    assert_eq!(r.len(), 5);
    assert!(r[4].2.x + r[4].2.w <= 520.0);
    assert!(r[0].2.x >= 100.0);
}

#[test]
fn sem_espaco_nao_aparecem() {
    assert!(rects(BARRA, 100.0, 150.0, PLAY_W).is_empty());
}

/// O Play com nome acende em qualquer palco; a seta nunca.
#[test]
fn o_play_com_nome_acende_com_o_jogo() {
    let corre = GameRun {
        stage: Some(GameStage::Editor),
        playing: true,
        ..Default::default()
    };
    assert!(aceso(ids::TOPBAR_PLAY_BUTTON, corre));
    assert!(!aceso(ids::TOPBAR_PLAY_BUTTON, GameRun::default()));
    assert!(!aceso(ids::MENUBAR_PLAY_MENU, corre));
}

#[test]
fn a_corrida_acende_o_botao_certo() {
    let parado = GameRun::default();
    assert!(BOTOES.iter().all(|(id, _)| !aceso(*id, parado)));
    let janela = GameRun {
        stage: Some(GameStage::Window),
        playing: true,
        ..Default::default()
    };
    assert!(aceso(ids::MENUBAR_RUN_WINDOW, janela));
    assert!(!aceso(ids::MENUBAR_RUN_EDITOR, janela));
    assert!(!aceso(ids::TOPBAR_PAUSE, janela));
    let pausado = GameRun {
        stage: Some(GameStage::Window),
        playing: false,
        ..Default::default()
    };
    assert!(aceso(ids::TOPBAR_PAUSE, pausado));
    assert!(!aceso(ids::MENUBAR_RUN_WINDOW, pausado));
    assert!(
        !BOTOES
            .iter()
            .any(|(id, _)| *id == ids::TOPBAR_RESET && aceso(*id, janela))
    );
}

/// Sem estes botões, a tela cheia não tinha como parar. Mutação: tirar o Stop da lista ⇒ reprova.
#[test]
fn a_tela_cheia_tem_jogar_pausar_e_parar_no_topo_ao_centro() {
    let r = rects_palco(BARRA);
    let ids: Vec<_> = r.iter().map(|(id, ..)| *id).collect();
    assert_eq!(
        ids,
        [
            ids::MENUBAR_RUN_FULLSCREEN,
            ids::TOPBAR_PAUSE,
            ids::TOPBAR_RESET,
            CONTROLS.0
        ]
    );
    let (inicio, fim) = (r[0].2.x, r[2].2.x + r[2].2.w);
    assert!(((inicio + fim) * 0.5 - 500.0).abs() < 0.01);
}

/// ⭐ Dono 07/11: a área clicável é o desenho. Cada botão responde até a borda do seu quadrado, com o ícone no centro,
/// e um ponto fora não é de ninguém. Mutação: registar o `glifo` em vez de `r` ⇒ a borda deixa de responder.
#[test]
fn a_area_clicavel_e_o_quadrado_que_o_botao_desenha() {
    let mut ts = ph2d_text::TextSystem::new();
    for (botoes, run) in [
        (rects(BARRA, 100.0, 800.0, PLAY_W), GameRun::default()),
        (
            rects_palco(BARRA),
            GameRun {
                stage: Some(GameStage::Fullscreen),
                playing: false,
                ..Default::default()
            },
        ),
    ] {
        let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
        paint(
            &botoes,
            run,
            &mut scene,
            &mut ts,
            Theme::Dark,
            &mut hits,
            &WidgetStore::default(),
        );
        for (id, _, r) in &botoes {
            let meio = r.y + r.h * 0.5;
            assert_eq!(hits.hit(r.x + r.w * 0.5, meio), Some(*id), "centro");
            assert_eq!(hits.hit(r.x + 0.5, meio), Some(*id), "borda esquerda");
            assert_eq!(hits.hit(r.x + r.w - 0.5, meio), Some(*id), "borda direita");
            assert_eq!(hits.hit(r.x + 0.5, r.y + 0.5), Some(*id), "canto");
        }
        let ultimo = botoes[botoes.len() - 1].2;
        assert_eq!(hits.hit(ultimo.x + ultimo.w + 1.0, ultimo.y + 1.0), None);
    }
}

/// ⭐ Dono 10/11: o «Controls» vem à direita do Stop, com o separador entre eles, e o grupo todo encostado à direita.
/// Mutação: tirar o `chain` do `rects` ⇒ o botão some; tirar o vão do separador ⇒ o separador cai sobre um botão.
#[test]
fn o_controls_fica_a_direita_do_stop_com_um_separador_entre_eles() {
    let r = rects(BARRA, 100.0, 800.0, PLAY_W);
    assert_eq!(r.len(), 5);
    assert_eq!((r[3].0, r[4].0), (ids::TOPBAR_RESET, CONTROLS.0));
    assert_eq!(r[4].1, IconId::Gamepad);
    let s = separador(&r).expect("o separador");
    let fim_do_stop = r[3].2.x + r[3].2.w;
    assert!(
        s.x > fim_do_stop && s.x + s.w < r[4].2.x,
        "{s:?} fora do vão"
    );
    assert!(
        s.h > 0.0 && s.h < r[4].2.h,
        "o separador é vertical e mais baixo que o botão"
    );
}

/// A moldura do «Controls» não é a pílula do grupo: a pílula acaba no Stop, antes do separador. Mutação: unir os cinco na
/// `pilula` ⇒ reprova.
#[test]
fn o_controls_tem_moldura_propria_fora_da_pilula_do_jogo() {
    let r = rects(BARRA, 100.0, 800.0, PLAY_W);
    let p = pilula(&r).expect("a pílula");
    let s = separador(&r).expect("o separador");
    assert!((p.x - r[0].2.x).abs() < 0.01, "a pílula começa no Play");
    assert!((p.x + p.w - (r[3].2.x + r[3].2.w)).abs() < 0.01);
    assert!(p.x + p.w < s.x && s.x < r[4].2.x);
}

/// Aceso = a tela aberta; desligado fora do Play ou num jogo sem tela.
#[test]
fn o_controls_acende_com_a_tela_aberta_e_so_age_no_play() {
    let parado = GameRun {
        has_controls_screen: true,
        ..Default::default()
    };
    assert!(!parado.controls_enabled() && !aceso(CONTROLS.0, parado));
    let aberto = GameRun {
        stage: Some(GameStage::Editor),
        controls_open: true,
        ..parado
    };
    assert!(aberto.controls_enabled() && aceso(CONTROLS.0, aberto));
    let sem_tela = GameRun {
        has_controls_screen: false,
        ..aberto
    };
    assert!(!sem_tela.controls_enabled());
}

/// O clique no Play pede a tela pelo bus (a shell a alterna pelo caminho da tecla); fora do Play o clique é consumido e
/// não faz nada. Mutação: tirar o `controls_enabled()` do despacho ⇒ fora do Play o pedido sai.
#[test]
fn o_clique_no_controls_so_pede_a_tela_no_play() {
    use crate::action_bus::EditorAction;
    use crate::interaction::WidgetEvent;
    let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
    hero.input_map.controls_mut().players_can_change_keys = true;
    publish(&mut hero, GameRun::default());
    assert!(crate::screens::hero::chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(CONTROLS.0)
    ));
    assert_eq!(hero.bus.drain().count(), 0, "fora do Play não faz nada");
    let run = GameRun {
        stage: Some(GameStage::Editor),
        playing: true,
        ..Default::default()
    };
    publish(&mut hero, run);
    assert!(crate::screens::hero::chrome::dispatch_all(
        &mut hero,
        WidgetEvent::Click(CONTROLS.0)
    ));
    assert!(matches!(
        hero.bus.drain().collect::<Vec<_>>().as_slice(),
        [EditorAction::PlayerControlsToggle]
    ));
}

/// A frase cita a tecla ATUAL do mapa e diz o porquê de estar desligado. Mutação: fixar «F1» ⇒ reprova no F4.
#[test]
fn a_frase_do_controls_cita_a_tecla_do_mapa_e_o_porque() {
    let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
    hero.input_map.controls_mut().open_key = ph2d_input::Key::all_named()
        .find(|k| k.label() == "F4")
        .expect("F4");
    let frase = |hero: &crate::screens::hero::HeroScreen| {
        hero.store
            .tooltip_for(CONTROLS.0)
            .unwrap_or_default()
            .to_owned()
    };
    publish(&mut hero, GameRun::default());
    assert!(frase(&hero).contains("change keys"), "{}", frase(&hero));
    hero.input_map.controls_mut().players_can_change_keys = true;
    publish(&mut hero, GameRun::default());
    let fora = frase(&hero);
    assert!(
        fora.starts_with("Press Play first") && fora.ends_with("F4"),
        "{fora}"
    );
    let run = GameRun {
        stage: Some(GameStage::Window),
        ..Default::default()
    };
    publish(&mut hero, run);
    let dentro = frase(&hero);
    assert!(
        dentro.starts_with("Show or hide") && dentro.ends_with("F4"),
        "{dentro}"
    );
}

/// ⛔ Smoke do dono 10/11: a dica «sem tela» saía em «Try…». As três frases, medidas como se pintam (Default e
/// CrispHeavyPlus, a 1600 e 1280), saem INTEIRAS e dentro da janela; a longa quebra em linhas. Mutação: voltar a
/// medir com o `layout` numa linha só ⇒ o censo da reticência apanha o corte.
#[test]
fn a_dica_do_controls_nunca_corta_e_cabe_na_janela() {
    use crate::text_elide::elisao;
    use ph2d_tokens::TextRendering;
    let mut ts = ph2d_text::TextSystem::new();
    for modo in [TextRendering::Default, TextRendering::CrispHeavyPlus] {
        ph2d_text::set_active_text_rendering(modo);
        for largura in [1600.0, 1280.0] {
            let janela = Rect::new(0.0, 0.0, largura, 900.0);
            let bar = Rect::new(0.0, 0.0, largura, 24.0);
            for (pode, stage) in [(false, None), (true, None), (true, Some(GameStage::Editor))] {
                let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
                hero.input_map.controls_mut().players_can_change_keys = pode;
                let run = GameRun {
                    stage,
                    ..Default::default()
                };
                publish(&mut hero, run);
                let botoes = rects(bar, largura * 0.25, largura * 0.75, PLAY_W);
                let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
                let run = hero.view.game_run;
                paint(
                    &botoes,
                    run,
                    &mut scene,
                    &mut ts,
                    Theme::Dark,
                    &mut hits,
                    &hero.store,
                );
                hero.store.set_hot(Some(CONTROLS.0));
                let (pintou, medidos) = elisao::medindo(|| {
                    crate::screens::hero::topbar::paint_hover_tooltip(
                        &mut scene,
                        &mut ts,
                        Theme::Dark,
                        &hits,
                        &hero.store,
                        janela,
                    )
                });
                let frase = hero
                    .store
                    .tooltip_for(CONTROLS.0)
                    .unwrap_or_default()
                    .to_owned();
                assert!(pintou, "{frase}");
                assert!(!medidos.is_empty(), "o censo não viu a dica");
                for m in &medidos {
                    assert!(
                        m.coube(),
                        "{modo:?} a {largura}: «{}» saiu «{}»",
                        m.texto,
                        m.pintado
                    );
                }
                let dica = crate::widget::mede_a_dica(&mut ts, &frase, janela.w);
                assert_eq!(dica.linhas.join(" "), frase, "as linhas cobrem a frase");
                let r = crate::screens::hero::topbar::tooltip_rect(
                    botoes[botoes.len() - 1].2,
                    dica.w,
                    dica.h,
                    Spacing::Sm.px(),
                    janela,
                );
                assert!(
                    r.x >= janela.x && r.x + r.w <= janela.x + janela.w,
                    "{modo:?} {largura}: {r:?}"
                );
                assert!(r.y >= janela.y && r.y + r.h <= janela.y + janela.h);
                if !pode {
                    assert!(dica.linhas.len() > 1, "a longa quebra: {:?}", dica.linhas);
                }
            }
        }
    }
    ph2d_text::set_active_text_rendering(TextRendering::Default);
}

fn centro(r: Rect) -> (f32, f32) {
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐ Onda 4: a barra da TELA CHEIA tem o «Controls» como a do Play — à direita do Stop, o separador entre eles, o
/// clique pelo mesmo caminho (`PlayerControlsToggle` → `host::toggle`) e a dica da casa, inteira, nos dois modos de
/// texto. Controlos: sem a mão sobre ele não há dica; num jogo sem a tela o clique não pede nada. Mutação: tirar o
/// `com_controls` do `rects_palco` ⇒ o botão some; tirar a dica do `paint_palco` ⇒ o censo não a vê.
#[test]
fn a_tela_cheia_tem_o_controls_com_separador_clique_e_dica_inteira() {
    use crate::action_bus::EditorAction;
    use crate::interaction::WidgetEvent;
    use crate::text_elide::elisao;
    use ph2d_tokens::TextRendering;
    let mut ts = ph2d_text::TextSystem::new();
    for largura in [1600.0, 1280.0, 900.0] {
        let area = Rect::new(0.0, 0.0, largura, 720.0);
        let r = rects_palco(area);
        assert_eq!(r.len(), 4, "os três do jogo e o Controls");
        assert_eq!((r[2].0, r[3].0), (ids::TOPBAR_RESET, CONTROLS.0));
        assert_eq!(r[3].1, IconId::Gamepad);
        let s = separador(&r).expect("o separador");
        assert!(s.x > r[2].2.x + r[2].2.w && s.x + s.w < r[3].2.x, "{s:?}");
        for pode in [true, false] {
            let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
            hero.input_map.controls_mut().players_can_change_keys = pode;
            let run = GameRun {
                stage: Some(GameStage::Fullscreen),
                playing: true,
                ..Default::default()
            };
            publish(&mut hero, run);
            for modo in [TextRendering::Default, TextRendering::CrispHeavyPlus] {
                ph2d_text::set_active_text_rendering(modo);
                for mao in [None, Some(CONTROLS.0)] {
                    hero.store.set_hot(mao);
                    let mut scene = VectorScene::new();
                    let ((), medidos) = elisao::medindo(|| {
                        let (hits, store) = (&mut hero.hit_index, &hero.store);
                        let run = hero.view.game_run;
                        paint_palco(area, run, &mut scene, &mut ts, Theme::Dark, hits, store);
                    });
                    assert_eq!(medidos.is_empty(), mao.is_none(), "a dica só sob a mão");
                    for m in &medidos {
                        assert!(
                            m.coube(),
                            "{modo:?} {largura}: «{}» saiu «{}»",
                            m.texto,
                            m.pintado
                        );
                    }
                }
            }
            let (x, y) = centro(r[3].2);
            assert!(
                sobre_o_controls(&hero, x, y),
                "o alvo do clique é o desenho"
            );
            assert!(crate::screens::hero::chrome::dispatch_all(
                &mut hero,
                WidgetEvent::Click(CONTROLS.0)
            ));
            let pedidos: Vec<_> = hero.bus.drain().collect();
            assert_eq!(
                matches!(pedidos.as_slice(), [EditorAction::PlayerControlsToggle]),
                pode,
                "sem a tela no jogo o clique não pede nada: {pedidos:?}"
            );
        }
    }
    ph2d_text::set_active_text_rendering(TextRendering::Default);
}

/// ⭐ Onda 4: com a tela «Controls» aberta, o movimento SOBRE o botão é da barra e o acende (D32, no mesmo movimento);
/// fora dele a tela fica com o movimento e o botão apaga, sem o hero o ver. Nas duas barras. Mutação: devolver sempre
/// `false` ⇒ não acende; tirar o `set_hot(None)` ⇒ fica aceso com o ponteiro na tela.
#[test]
fn com_a_tela_aberta_o_controls_acende_so_sob_o_ponteiro() {
    use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
    let mover = |x: f32, y: f32| PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind: PointerKind::Move,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    };
    let mut ts = ph2d_text::TextSystem::new();
    for botoes in [rects(BARRA, 100.0, 800.0, PLAY_W), rects_palco(BARRA)] {
        let mut hero = crate::screens::hero::HeroScreen::new(NodeId(1));
        let mut scene = VectorScene::new();
        let (hits, store) = (&mut hero.hit_index, &hero.store);
        paint(
            &botoes,
            GameRun::default(),
            &mut scene,
            &mut ts,
            Theme::Dark,
            hits,
            store,
        );
        let (x, y) = centro(botoes[botoes.len() - 1].2);
        assert!(
            movimento_com_a_tela(&mut hero, x, y),
            "sobre o botão: da barra"
        );
        let arena = bumpalo::Bump::new();
        let _ = crate::interaction::dispatch_pointer(
            &mut hero.store,
            &hero.hit_index,
            mover(x, y),
            &arena,
        );
        assert_eq!(hero.store.hot_id(), Some(CONTROLS.0));
        assert_eq!(
            hero.store.button_state(CONTROLS.0),
            Some(ButtonState::Hovered)
        );
        let no_jogo = y + BARRA.h * 10.0;
        assert!(
            !movimento_com_a_tela(&mut hero, x, no_jogo),
            "fora: da tela"
        );
        assert_eq!(hero.store.hot_id(), None);
        assert_eq!(
            hero.store.button_state(CONTROLS.0),
            Some(ButtonState::Normal)
        );
    }
}

/// Grava o que `f` pinta.
fn gravado(f: impl FnOnce(&mut VectorScene)) -> Vec<ph2d_vector::DrawOp> {
    let mut scene = VectorScene::new();
    scene.record();
    f(&mut scene);
    scene.take_record()
}

/// A caixa (y de cima, y de baixo) de cada operação da cor `cor`.
fn faixas(ops: &[ph2d_vector::DrawOp], cor: Color) -> Vec<(f64, f64)> {
    use ph2d_vector::{Brush, DrawOp, Shape};
    ops.iter()
        .filter_map(|o| match o {
            DrawOp::Fill {
                brush: Brush::Solid(c),
                path,
                ..
            }
            | DrawOp::Stroke {
                brush: Brush::Solid(c),
                path,
                ..
            } if *c == cor => {
                let b = path.bounding_box();
                Some((b.y0, b.y1))
            }
            _ => None,
        })
        .collect()
}

fn loja_com(hover: bool) -> WidgetStore {
    let mut store = WidgetStore::default();
    let state = if hover {
        ButtonState::Hovered
    } else {
        ButtonState::Normal
    };
    store.register(
        CONTROLS.0,
        crate::interaction::InteractiveState::Button { state },
    );
    store
}

/// ⭐ Onda 4: o «Controls» tem TRÊS estados que se veem — normal, aceso (a tela aberta) e aceso pairado (o acento de
/// hover da casa): com a tela aberta, pairar muda o botão. Mutação: o acento ignorar o `hover` ⇒ aceso = aceso+hover.
#[test]
fn o_controls_normal_aceso_e_aceso_pairado_pintam_diferente() {
    let r = rects(BARRA, 100.0, 800.0, PLAY_W);
    let mut ts = ph2d_text::TextSystem::new();
    let ligado = GameRun {
        stage: Some(GameStage::Editor),
        playing: true,
        has_controls_screen: true,
        ..Default::default()
    };
    let aberto = GameRun {
        controls_open: true,
        ..ligado
    };
    let mut pinta = |run: GameRun, hover: bool| {
        let store = loja_com(hover);
        format!(
            "{:?}",
            gravado(|scene| paint(
                &r,
                run,
                scene,
                &mut ts,
                Theme::Dark,
                &mut HitIndex::default(),
                &store
            ))
        )
    };
    let (normal, aceso, aceso_hover) = (
        pinta(ligado, false),
        pinta(aberto, false),
        pinta(aberto, true),
    );
    assert_ne!(normal, aceso, "aceso ≠ normal");
    assert_ne!(aceso, aceso_hover, "pairar o aceso sobe um degrau");
    assert_ne!(normal, aceso_hover);
}

/// ⭐ Onda 4: na tela cheia a moldura do «Controls» tem o recuo do realce do Pause (não encosta nas bordas da barra),
/// o chão dos botões é o da barra de menus e o separador tem a cor do da barra do editor sobre ele (o mesmo contraste).
/// Controlo: o Pause aceso pinta o realce, e a moldura fica dentro do quadrado dele nos quatro lados. Mutação: voltar a
/// caixa ao recuo de 1 px ⇒ a faixa da moldura ≠ a do realce; voltar o chão a `Bg1` em cartões ⇒ reprova o chão.
#[test]
fn na_tela_cheia_a_moldura_e_o_separador_sao_os_da_barra_do_editor() {
    let area = Rect::new(0.0, 0.0, 1600.0, 1000.0);
    let theme = Theme::Dark;
    crate::widget::lista_flutuante::force_cromo_on_this_thread(Some(true));
    let run = GameRun {
        stage: Some(GameStage::Fullscreen),
        playing: false,
        has_controls_screen: true,
        controls_open: true,
    };
    let palco = rects_palco(area);
    let store = loja_com(false);
    let mut ts = ph2d_text::TextSystem::new();
    let ops = gravado(|scene| {
        paint_palco(
            area,
            run,
            scene,
            &mut ts,
            theme,
            &mut HitIndex::default(),
            &store,
        )
    });
    let chao = faixas(&ops, chao_da_barra(theme));
    let u = crate::widget::cromo_barra::uniao(palco.iter().map(|(_, _, r)| *r)).expect("a união");
    assert!(
        chao.iter()
            .any(|(y0, y1)| (*y0 - f64::from(u.y)).abs() < 0.5
                && (*y1 - f64::from(u.y + u.h)).abs() < 0.5),
        "o chão da tela cheia é o da barra: {chao:?}"
    );
    let ctl = palco[3].2;
    let (caixa, _) = caixa_do_controls(ctl, theme);
    assert!(
        caixa.x > ctl.x
            && caixa.y > ctl.y
            && caixa.x + caixa.w < ctl.x + ctl.w
            && caixa.y + caixa.h < ctl.y + ctl.h,
        "a moldura não encosta: {caixa:?} em {ctl:?}"
    );
    {
        let b = crate::widget::cromo_barra::Barra::de(theme).expect("em cartões");
        let realce = faixas(&ops, b.tinta().color(ColorToken::KanbanCardRaised));
        assert!(
            !realce.is_empty(),
            "o Pause aceso pinta o realce (controlo)"
        );
        // Tolerância 0: o que se PINTA — a borda de fora do traço (o caminho ± meia espessura) e o fundo aceso — cai
        // exactamente na faixa do realce do Pause (a 2.ª rodada de fotos: 8–25 contra 9–24).
        let traco: Vec<(f64, f64)> = ops
            .iter()
            .filter_map(|o| match o {
                ph2d_vector::DrawOp::Stroke {
                    brush: ph2d_vector::Brush::Solid(c),
                    path,
                    style,
                    ..
                } if *c == b.tinta().color(ColorToken::Accent) => {
                    let bb = ph2d_vector::Shape::bounding_box(path);
                    Some((bb.y0 - style.width * 0.5, bb.y1 + style.width * 0.5))
                }
                _ => None,
            })
            .collect();
        let fundo = faixas(&ops, b.tinta().color(ColorToken::AccentSoft));
        // ±1e-6 px: o contorno arredondado sai de arcos em Bézier (2e-15 de folga, onda 7); um erro de
        // geometria é de pixel.
        let (t, r) = (traco.first().copied(), realce.first().copied());
        assert!(
            matches!((t, r), (Some(a), Some(b)) if (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6),
            "a moldura pintada (o 1.º traço; o 2.º é o glifo) ≠ o realce do Pause: {t:?} × {r:?}"
        );
        assert_eq!(fundo, [realce[0]], "o fundo aceso ≠ o realce do Pause");
        let normal = gravado(|scene| {
            paint(
                &rects(BARRA, 100.0, 800.0, PLAY_W),
                run,
                scene,
                &mut ts,
                theme,
                &mut HitIndex::default(),
                &store,
            )
        });
        assert!(
            !faixas(&normal, b.divisoria()).is_empty(),
            "o separador da barra do editor"
        );
        assert!(
            !faixas(&ops, b.divisoria()).is_empty(),
            "o MESMO na tela cheia"
        );
    }
    crate::widget::lista_flutuante::force_cromo_on_this_thread(None);
}
