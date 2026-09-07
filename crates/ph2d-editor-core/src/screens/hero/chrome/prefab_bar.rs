// ph2d-chrome-sync:z=225 (dispatch priority, ADR-0107; lower = earlier)
//! **A SAÍDA do modo de edição de receita** — o botão `Done` da barra do *Edit Prefab*.
//!
//! ⚠️ **O handler mora aqui e o pintor mora em [`super::super::prefab_bar`]**, e o corte é o da
//! casa: `chrome/` é a lista de quem CONSOME um clique (ela é gerada e ordenada por z), e um
//! pintor de canvas com medidas e tokens não pertence a essa lista. *A alternativa — o handler ao
//! lado do pintor — deixaria o `dispatch_all` sem uma linha para ele, e o botão nasceria morto sob
//! o ponteiro.*
//!
//! ⚠️ **Antes dos punhos de canvas** (`curve_point_handle` z=230, `falloff_handle` z=240): a barra
//! desenha-se por cima da área de desenho, e um punho que estivesse debaixo dela não pode ganhar o
//! clique.

use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    crate::screens::hero::prefab_bar::apply_event(hero, event)
}
