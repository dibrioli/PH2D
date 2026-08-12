// ph2d-chrome-sync:z=130 (dispatch priority, ADR-0107; lower = earlier)
//! Settings → Motion cascade: o CARÁCTER da UI viva + o *reduced motion*.
//!
//! Espelha o `settings_text.rs` 1:1 na forma, e difere numa coisa: as três linhas **não são um
//! rádio de três posições**. As duas primeiras escolhem o carácter (o GOSTO — Expressivo contra
//! Discreto) e a terceira alterna o *reduced motion* (a GARANTIA, que se sobrepõe a qualquer
//! carácter). Colapsá-las tornaria *Expressivo + reduced* — uma combinação legítima, e a que um
//! artista com sensibilidade vestibular escolheria — **inexprimível**.
//!
//! Escolher escreve em `HeroScreen.motion`, que é o dono do facto. A **persistência** é da shell,
//! que nota a diferença e grava (`shells/desktop/src/prefs.rs`): o ficheiro é uma *projecção* do
//! estado vivo, e uma projecção não precisa de um canal próprio para ser notificada.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::motion::UiCharacter;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::CTX_MENU_SETTINGS_MOTION {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::SettingsMotionSubmenu,
        });
        return true;
    }
    if id == ids::CTX_MENU_MOTION_REDUCED {
        // ⚠️ TOGGLE, não escolha: a linha lê o estado corrente e inverte-o. Um `set(true)` faria
        // da row uma porta de sentido único — ligável e nunca desligável — e o bullet passaria a
        // dizer a verdade sobre um interruptor que o artista não consegue mexer.
        let on = hero.motion.reduced_motion();
        hero.motion.set_reduced_motion(!on);
        hero.store.close_context_menu();
        return true;
    }
    let chosen = if id == ids::CTX_MENU_MOTION_EXPRESSIVE {
        UiCharacter::Expressive
    } else if id == ids::CTX_MENU_MOTION_DISCRETE {
        UiCharacter::Discrete
    } else {
        return false;
    };
    hero.motion.set_character(chosen);
    hero.store.close_context_menu();
    true
}
