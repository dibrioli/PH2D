//! ⭐ **O condutor comum de uma cena de smoke**, pelos pedidos do produto (nenhum caminho paralelo):
//! ao quadro [`QUADRO_DO_ENQUADRAR`] o *Frame All* do botão VIEW; com a variável `play` a `1`, ao
//! [`QUADRO_DO_PLAY`] o clique no interruptor Physics da barra de transporte (`fisica`, ADR-0131: a
//! simulação não se arma sozinha) e *Play in the Editor* (o palco e o Play do menu).

use ph2d_editor_core::action_bus::{EditorAction as A, TransportCmd};
use ph2d_editor_core::screens::hero::HeroScreen;

/// O quadro do roteador em que a cena pede o *Frame All* (o que nasceu já foi desenhado).
pub const QUADRO_DO_ENQUADRAR: u32 = 3;
/// O quadro em que a cena entra em Play (a cena assentou e o enquadramento voou).
pub const QUADRO_DO_PLAY: u32 = 30;

/// Uma chamada por quadro da cena armada; `quadro` conta desde a montagem.
pub fn enquadra_e_joga(
    hero: &mut HeroScreen,
    quadro: u32,
    play: &str,
    fisica: ph2d_editor_core::NodeId,
) {
    if quadro == QUADRO_DO_ENQUADRAR {
        let kind = ph2d_editor_core::ViewFocusKind::All;
        hero.bus.push(A::SetViewFocus { kind });
    } else if quadro == QUADRO_DO_PLAY && std::env::var(play).is_ok_and(|v| v.trim() == "1") {
        let toggle = ph2d_editor_core::tool::PanelEvent::Toggle(fisica, true);
        hero.bus.push(A::TimelinePanelEvent(toggle));
        hero.view.game_stage = ph2d_editor_core::game_stage::GameStage::Editor;
        hero.bus.push(A::Transport(TransportCmd::Play));
    }
}
