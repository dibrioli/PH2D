// ph2d-chrome-sync:z=15 (dispatch priority, ADR-0107; lower = earlier)
//! **As QUATRO linhas que a barra de menus trouxe** — as únicas dela sem dono anterior.
//!
//! ⚠️ **As outras vinte e cinco não estão aqui, e é a decisão da barra**: o *Save* é despachado
//! pelo [`super::io_menu`], o *Undo* pelo [`super::rail_tools`], os treze do menu *Window* pelos
//! próprios painéis. A barra **realoja** verbos (D2); um handler que os re-despachasse seria o
//! segundo dono de cada um.
//!
//! ⇒ o que sobra são quatro verbos que nenhum botão do app alcançava: a imagem nova (só a tecla),
//! as preferências e o tema (só os pills, retirados em 2026-08-30), e a régua (só uma caixa dentro
//! do painel do vetor, que deixou de ser o dono no dia em que as réguas passaram a valer em todos
//! os modos).

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // *New Image…* — o modal já existia e só a tecla `Cmd/Ctrl+N` o abria.
    // ⚠️ `open_new_image_dialog` **substitui** o menu aberto (é o mesmo slot), então não há fecho
    // a fazer aqui: fechá-lo antes apagaria o modal que ele acabou de pôr.
    if id == ids::MENUBAR_FILE_NEW {
        hero.store.open_new_image_dialog();
        return true;
    }
    // As duas linhas que abrem outra COISA. ⚠️ Elas estão excluídas do fecho antecipado
    // (`menu_bar::close_on_row_click`) exactamente por isto — o `cascade_anchor` lê o menu que
    // ainda está aberto para saber de onde a cascata sai.
    let cascade = if id == ids::MENUBAR_EDIT_PREFERENCES {
        ContextMenuKind::SettingsMenu
    } else if id == ids::MENUBAR_VIEW_THEME {
        ContextMenuKind::ThemeSelector
    } else if id == ids::MENUBAR_FILE_SCENES {
        ContextMenuKind::SceneList
    } else if id == ids::MENUBAR_VIEW_RESET_LAYOUT {
        // ⭐⭐ **Repor a arrumação** (Enio, 2026-08-30). Ver [`super::super::slot_tabs::reset`].
        //
        // ⚠️ **Não apaga o ficheiro, e não precisa:** a arrumação gravada é uma **projecção** do
        // que o app tem agora, e o detector do quadro grava a projecção vazia por si. *Apagar o
        // ficheiro seria um segundo caminho para o mesmo facto, e o dia em que os dois
        // discordassem seria silencioso.*
        super::super::slot_tabs::reset(hero);
        hero.store.close_context_menu();
        return true;
    } else if id == ids::MENUBAR_VIEW_RULERS {
        // ⚠️ **A régua é estado do HERO, não da ferramenta** — o mesmo campo que a caixa do painel
        // do vetor mexe. Duas portas, um valor.
        hero.view.rulers_visible = !hero.view.rulers_visible;
        hero.store.close_context_menu();
        return true;
    } else {
        return false;
    };
    let (x, y) = super::cascade_anchor(hero, id);
    hero.store.open_context_menu(ContextMenuRequest {
        x,
        y,
        kind: cascade,
    });
    true
}
