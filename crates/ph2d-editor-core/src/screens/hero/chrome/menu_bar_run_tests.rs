//! O menu *Run* escolhe onde jogar E joga — pela cadeia inteira (`dispatch_all`), não pelo handler.

use crate::action_bus::{EditorAction, TransportCmd};
use crate::game_stage::GameStage;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;
use ph2d_a11y::NodeId;

#[test]
fn cada_linha_do_run_escolhe_o_palco_e_joga() {
    for (id, stage) in [
        (ids::MENUBAR_RUN_FULLSCREEN, GameStage::Fullscreen),
        (ids::MENUBAR_RUN_EDITOR, GameStage::Editor),
        (ids::MENUBAR_RUN_WINDOW, GameStage::Window),
    ] {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.view.game_stage = if stage == GameStage::Window {
            GameStage::Editor
        } else {
            GameStage::Window
        };
        assert!(super::super::dispatch_all(&mut hero, WidgetEvent::Click(id)));
        assert_eq!(hero.view.game_stage, stage, "{id:?} não escolheu o palco");
        let got: Vec<EditorAction> = hero.bus.drain().collect();
        assert!(
            matches!(got.as_slice(), [EditorAction::Transport(TransportCmd::Play)]),
            "{id:?} não levantou o Play: {got:?}"
        );
        // A marca do menu diz a escolha.
        assert_eq!(super::super::super::menu_bar::module_is_on(&hero, id), Some(true));
    }
}

#[test]
fn o_jogo_abre_numa_janela_por_omissao() {
    assert_eq!(HeroScreen::new(NodeId(1)).view.game_stage, GameStage::Window);
}
