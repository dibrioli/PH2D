//! ⭐ **O condutor comum de uma cena de smoke**, pelos pedidos do produto (nenhum caminho paralelo):
//! ao quadro [`QUADRO_DO_ENQUADRAR`] o *Frame All* do botão VIEW; com a variável `play` a `1`, ao
//! [`QUADRO_DO_PLAY`] o clique no interruptor Physics da barra de transporte (`fisica`, ADR-0131: a
//! simulação não se arma sozinha) e *Play in the Editor* (o palco e o Play do menu). Com a variável a
//! `2`, também → segurada desde [`QUADRO_DA_SETA`]: a shell aplica-a ao `InputState` como a tecla real
//! ([`seta_pedida`], a porta do condutor do jogo).

use ph2d_editor_core::action_bus::{EditorAction as A, TransportCmd};
use ph2d_editor_core::screens::hero::HeroScreen;

/// O quadro do roteador em que a cena pede o *Frame All* (o que nasceu já foi desenhado).
pub const QUADRO_DO_ENQUADRAR: u32 = 3;
/// O quadro em que a cena entra em Play (a cena assentou e o enquadramento voou).
pub const QUADRO_DO_PLAY: u32 = 30;

/// O quadro em que `play = 2` prime → (o herói já pousou: 60 quadros depois do Play).
pub const QUADRO_DA_SETA: u32 = QUADRO_DO_PLAY + 60;

/// As setas pedidas pela cena, `(seta, desce)` pela ordem → ↑ ← ↓; a shell drena-as.
static SETAS: std::sync::Mutex<Vec<(u8, bool)>> = std::sync::Mutex::new(Vec::new());

/// A próxima seta pedida (a shell aplica-a como `KeyDown`/`KeyUp`).
pub fn seta_pedida() -> Option<(u8, bool)> {
    let mut v = SETAS.lock().ok()?;
    (!v.is_empty()).then(|| v.remove(0))
}

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
    }
    let modo = std::env::var(play).unwrap_or_default();
    let modo = modo.trim();
    if quadro == QUADRO_DA_SETA && modo == "2" {
        if let Ok(mut v) = SETAS.lock() {
            v.push((0, true));
        }
    } else if quadro == QUADRO_DO_PLAY && matches!(modo, "1" | "2") {
        let toggle = ph2d_editor_core::tool::PanelEvent::Toggle(fisica, true);
        hero.bus.push(A::TimelinePanelEvent(toggle));
        hero.view.game_stage = ph2d_editor_core::game_stage::GameStage::Editor;
        hero.bus.push(A::Transport(TransportCmd::Play));
    }
}
