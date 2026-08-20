// ph2d-chrome-sync:z=273 (dispatch priority, ADR-0107; lower = earlier)
//! **O pill MODEL** — abre e fecha o módulo de modelagem 3D por campo implícito (ADR-0161).
//!
//! ⚠️ **Ele é irmão do PHYS e do TOK, não do SCULPT ao lado.** Aquele muda quem é dono do ponteiro
//! e por isso precisa de uma saída visível; este abre um MÓDULO com o seu painel, e o ponteiro só é
//! dele dentro da área que ele desenha.
//!
//! ⭐ **Ele é a razão de o módulo existir para quem abre o app.** Até 2026-08-19 a única porta era a
//! variável de ambiente `PH2D_FIELD_SMOKE` — uma feature que só existe para quem já sabe que ela
//! existe (Enio: *"não temos um Pill no topo do app para abrir o painel/módulo 3d Modeling"*). É a
//! mesma lição que a nota do `sculpt3d_toggle` já tinha escrito ao lado, e que este módulo repetiu.
//!
//! ⚠️ **Ele não guarda bool nenhum:** a visibilidade do painel É o estado, e o *pressed* do pill é
//! derivado dela a cada quadro. Um bool próprio aqui seria a segunda porta que diverge — o pill a
//! dizer *fechado* sobre um painel que está na tela.
//!
//! Fiação central: `ids::TOPBAR_MODEL3D` + o pill em `screens/hero/fixture.rs` (`IconId::Cube`) +
//! registro em `topbar/mod.rs::populate` + o shell a ler `panel_visibility` para armar o módulo.

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

/// A chave de visibilidade do painel de modelagem 3D.
///
/// ⚠️ Ela é a MESMA que a crate do painel declara (`ph2d_panel_model3d::PANEL_ID`) — mas esta crate
/// não a pode importar (o painel depende dela, não o contrário). O gate
/// `the_model_pill_toggles_the_panel_the_shell_knows` prende as duas juntas, que é o que impede a
/// divergência silenciosa: uma chave errada aqui alterna um painel que ninguém pinta.
pub const PANEL_KEY: &str = "model3d";

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_MODEL3D {
        return false;
    }
    let visible = !hero.is_panel_visible(PANEL_KEY);
    hero.panel_visibility.insert(PANEL_KEY, visible);
    true
}
