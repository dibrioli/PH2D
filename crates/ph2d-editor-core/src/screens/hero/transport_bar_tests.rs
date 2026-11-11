//! Os botões do jogo: no centro, sem pisar nos vizinhos, e acesos pelo estado da corrida.

use super::*;
use crate::interaction::{HitIndex, WidgetStore};

const BARRA: Rect = Rect::new(0.0, 0.0, 1000.0, 24.0);

#[test]
fn ficam_no_centro_da_barra() {
    let r = rects(BARRA, 100.0, 800.0);
    assert_eq!(r.len(), 6, "os cinco do jogo e o Controls");
    let inicio = r[0].2.x;
    let fim = r[4].2.x + r[4].2.w;
    assert!(((inicio + fim) * 0.5 - 500.0).abs() < 0.01);
}

/// Mutação: tirar o `clamp` ⇒ os botões caem sobre as abas da direita.
#[test]
fn saem_do_centro_antes_de_pisar_nos_vizinhos() {
    let r = rects(BARRA, 100.0, 520.0);
    assert_eq!(r.len(), 6);
    assert!(r[5].2.x + r[5].2.w <= 520.0);
    assert!(r[0].2.x >= 100.0);
}

#[test]
fn sem_espaco_nao_aparecem() {
    assert!(rects(BARRA, 100.0, 150.0).is_empty());
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
            ids::TOPBAR_RESET
        ]
    );
    let (inicio, fim) = (r[0].2.x, r[2].2.x + r[2].2.w);
    assert!(((inicio + fim) * 0.5 - 500.0).abs() < 0.01);
}

/// ⭐ Dono 07/11: a área clicável é o desenho. Cada botão responde até a borda do seu quadrado, com o ícone no centro,
/// e um ponto fora não é de ninguém. Mutação: registar o `glifo` em vez de `r` ⇒ a borda deixa de responder.
#[test]
fn a_area_clicavel_e_o_quadrado_que_o_botao_desenha() {
    for (botoes, run) in [
        (rects(BARRA, 100.0, 800.0), GameRun::default()),
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

/// ⭐ Dono 10/11: o «Controls» vem à direita do Stop, com o separador entre eles, e o grupo do jogo fica ao centro.
/// Mutação: tirar o `chain` do `rects` ⇒ o botão some; tirar o vão do separador ⇒ o separador cai sobre um botão.
#[test]
fn o_controls_fica_a_direita_do_stop_com_um_separador_entre_eles() {
    let r = rects(BARRA, 100.0, 800.0);
    assert_eq!(r.len(), 6);
    assert_eq!((r[4].0, r[5].0), (ids::TOPBAR_RESET, CONTROLS.0));
    assert_eq!(r[5].1, IconId::Gamepad);
    let s = separador(&r).expect("o separador");
    let fim_do_stop = r[4].2.x + r[4].2.w;
    assert!(
        s.x > fim_do_stop && s.x + s.w < r[5].2.x,
        "{s:?} fora do vão"
    );
    assert!(
        s.h > 0.0 && s.h < r[5].2.h,
        "o separador é vertical e mais baixo que o botão"
    );
}

/// A moldura do «Controls» não é a pílula do grupo: a pílula acaba no Stop, antes do separador. Mutação: unir os seis na
/// `pilula` ⇒ reprova.
#[test]
fn o_controls_tem_moldura_propria_fora_da_pilula_do_jogo() {
    let r = rects(BARRA, 100.0, 800.0);
    let p = pilula(&r).expect("a pílula");
    let s = separador(&r).expect("o separador");
    assert!((p.x + p.w - (r[4].2.x + r[4].2.w)).abs() < 0.01);
    assert!(p.x + p.w < s.x && s.x < r[5].2.x);
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
                let botoes = rects(bar, largura * 0.25, largura * 0.75);
                let (mut scene, mut hits) = (VectorScene::new(), HitIndex::default());
                let run = hero.view.game_run;
                paint(
                    &botoes,
                    run,
                    &mut scene,
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
                    botoes[5].2,
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
