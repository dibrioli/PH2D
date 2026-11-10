//! **R9c · C · Try it**: o «Forget them» com o Play a correr chega à SESSÃO (defeito confirmado na auditoria: só
//! apagava o ficheiro; o jogo continuava com as teclas e a troca seguinte regravava-as) e as cenas C3/C4 rolam até
//! o cartão delas.

use super::Mesa;
use ph2d_app_input_map::ph2d_player_controls::{self as pc, GameId};
use ph2d_app_input_map::view_try::state::{TryCard, TryHit};
use ph2d_app_input_map::{InputMapTab, View};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{Binding, Key, PlayerKeys};

fn key(name: &str) -> Key {
    Key::named_in_script(name).expect("tecla com nome")[0]
}

/// As linhas da shell depois do `frame` da família (`fase_snapshots_publish::document_families`).
fn shell_forget(m: &mut Mesa) {
    let tab = m.hero.documents.surfaces_mut().find_mut::<InputMapTab>();
    if let Some(t) = tab {
        t.doc.forget_reaches(&mut m.jogador, &m.hero.input_map);
    }
}

#[test]
fn forget_no_play_chega_a_sessao_e_a_troca_seguinte_nao_regrava_as_esquecidas() {
    let dir = std::env::temp_dir().join(format!("ph2d-try-r9c-{}", std::process::id()));
    let mut m = Mesa::new(View::TryIt);
    m.tab_mut().doc.tryit.base = Some(dir.clone());
    m.sim.world_mut().insert_resource(GameId(9));
    m.hero.input_map.controls_mut().players_can_change_keys = true;
    let file = pc::player_file(&dir, 9);
    let mut keys = PlayerKeys::new();
    keys.set("jump", vec![Binding::Key(key("x"))]);
    pc::file::save(&file, &keys).expect("grava");
    m.menu();
    let project = m.hero.input_map.clone();
    m.sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    m.jogador.start(&project, Some(file.clone()));
    assert_ne!(*m.jogador.map(&project), project, "o Play joga com a troca");
    m.frame();
    m.click(m.at(TryHit::Forget.id()));
    shell_forget(&mut m);
    assert!(!file.exists(), "o ficheiro saiu");
    assert_eq!(
        *m.jogador.map(&m.hero.input_map),
        m.hero.input_map,
        "o jogo larga as teclas esquecidas JÁ"
    );
    // O jogador troca OUTRA acção na tela do Play: F1, desce até ela, Enter, a tecla nova.
    let other = (m.jogador.rows(&project).iter())
        .position(|r| r.action != "jump")
        .expect("outra acção na tela");
    m.jogador.key(key("f1"), true, &project);
    (0..other).for_each(|_| {
        m.jogador.key(key("down"), true, &project);
    });
    m.jogador.key(key("enter"), true, &project);
    m.jogador.key(key("k"), true, &project);
    let saved = pc::file::load(&file);
    assert_eq!(saved.get("jump"), None, "a troca esquecida não volta");
    assert_eq!(saved.changed(&project), 1, "só a troca nova: {saved:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn c4_rola_ate_o_cartao_controls_a_1280() {
    let small = Rect::new(297.0, 28.0, 690.0, 604.0); // LITERAL-PX-OK: a área central medida na foto c3_1280
    for probe in [TryHit::PlayersCanChange, TryHit::Forget] {
        let mut m = Mesa::aberta(View::TryIt);
        m.tab_mut().doc.tryit.player_changed = 1;
        m.tab_mut().doc.smoke_reveal(TryCard::Controls);
        m.paint_in(small);
        let r = m.tab().doc.rect_of(probe.id()).expect("o alvo pintado");
        assert!(m.tab().doc.tryit.scroll > 0.0, "rolou");
        assert!(
            r.y >= small.y && r.y + r.h <= small.y + small.h,
            "{r:?} à vista"
        );
    }
}
