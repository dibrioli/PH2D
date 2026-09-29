//! ⭐⭐ **AS ABAS DA PEÇA SEGUEM A FERRAMENTA** — a metade da shell do report do dono (29/09): *«não
//! consigo voltar para o modo sculpt (faça voltar ao abrir/selecionar o painel sculpt e
//! vice-versa)»*.
//!
//! A metade da UI (o CLIQUE numa aba pede a ferramenta) vive em
//! `ph2d_editor_core::screens::hero::slot_tabs_ferramenta`. Esta é a que a torna ALCANÇÁVEL e
//! legível:
//!
//! 1. **Com a peça no ecrã e o modo IMG ligado, a aba do Painter fica**, mesmo com ele fora da
//!    mão — sem ela o «vice-versa» não tinha onde clicar (a ponte do Painter esconde a aba em todo
//!    quadro em que ele não está activo). Ela nasce ATRÁS da da escultura.
//! 2. **O Painter sai da mão ⇒ a aba da escultura vem à frente** — o gémeo da ponte do Painter, que
//!    já traz a aba dele à frente quando ele entra. Sem isto o painel da frente dizia «Painter» com
//!    a mão a esculpir.
//!
//! ⚠️ Corre DEPOIS da ponte do Painter no quadro: ela reescreve a visibilidade da aba dele em todo
//! quadro, e quem escreve por último é quem fica.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::ids::{PAINTER_LAYERS_PANEL, SCULPT3D_PANEL};
use std::cell::Cell;

thread_local! {
    static PAINTER_ESTAVA: Cell<bool> = const { Cell::new(false) };
}

/// Ver o cabeçalho. `barro` = a peça está no ecrã; `painter_na_mao` = a ferramenta activa é o
/// Painter.
pub fn abas_seguem_a_ferramenta(hero: &mut HeroScreen, barro: bool, painter_na_mao: bool) {
    let estava = PAINTER_ESTAVA.with(|c| c.replace(painter_na_mao));
    decide(hero, barro, painter_na_mao, estava);
}

/// A lei, sem a memória do quadro anterior — é ela que os gates medem.
fn decide(hero: &mut HeroScreen, barro: bool, painter_na_mao: bool, estava: bool) {
    if !barro {
        return;
    }
    if !painter_na_mao && hero.image_edit.mode_on {
        // A ordem z só guarda os painéis visíveis no quadro anterior: estar lá é «já estava à vista».
        let ja_a_vista = hero.store.panel_z_order().contains(&PAINTER_LAYERS_PANEL);
        hero.panel_visibility.insert("painter_layers", true);
        if !ja_a_vista {
            // Nasce ATRÁS: sobe ela e logo a da escultura por cima.
            hero.store.bump_panel_z(PAINTER_LAYERS_PANEL);
            hero.store.bump_panel_z(SCULPT3D_PANEL);
        }
    }
    if estava && !painter_na_mao {
        hero.store.bump_panel_z(SCULPT3D_PANEL);
    }
}

#[cfg(test)]
#[path = "abas_tests.rs"]
mod tests;
