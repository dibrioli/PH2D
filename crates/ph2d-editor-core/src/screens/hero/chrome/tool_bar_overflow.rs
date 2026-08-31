// ph2d-chrome-sync:z=45 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐⭐⭐ **O `⋯` DA FILA DE FERRAMENTAS** — abre o que não coube, e fecha-se ao servir.
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! # ⚠️ O z é `45`, entre os toggles de vista (`40`) e os do trilho (`50`)
//!
//! E a razão é a segunda metade deste ficheiro: quando o menu de transbordo está aberto, **um
//! clique em qualquer outra coisa fecha-o e DEIXA PASSAR** (`return false`). Ele tem de correr
//! antes dos handlers do trilho — que são quem de facto executa o verbo do chip escolhido — para
//! que o menu já esteja fechado quando o verbo acontece.
//!
//! ⛔ **Um chip do transbordo não tem handler próprio, e é essa a decisão.** Os ids dentro do menu
//! são **os mesmos** da fila, logo quem despacha continua a ser o `chrome::rail_*`. Um verbo
//! copiado para aqui seria a segunda porta que o `CLAUDE.md` §5.0 cataloga como a espécie mais
//! cara de controlo morto: duas respostas para o mesmo clique, e a que envelhece é a que ninguém
//! relê.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == ids::TOOL_BAR_OVERFLOW {
        // ⚠️ Um segundo clique no `⋯` FECHA — é um interruptor, não uma porta de sentido único.
        if matches!(
            hero.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::ToolBarOverflow)
        ) {
            hero.store.close_context_menu();
            return true;
        }
        // Ancorado por BAIXO do próprio chip, como os menus da barra: o rect vem do índice de
        // acerto do quadro anterior, que é onde o chip de facto ficou.
        let (x, y) = hero
            .hit_index
            .rect_for(ids::TOOL_BAR_OVERFLOW)
            .map_or((0.0, hero.last_viewport.y), |r| (r.x, r.y + r.h));
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::ToolBarOverflow,
        });
        return true;
    }
    // ⭐ **Servir é fechar.** Qualquer outro clique com o transbordo aberto fecha-o — e devolve
    // `false`, para o verbo escolhido continuar até quem o executa.
    if matches!(
        hero.store.context_menu().map(|r| r.kind),
        Some(ContextMenuKind::ToolBarOverflow)
    ) {
        hero.store.close_context_menu();
    }
    false
}
