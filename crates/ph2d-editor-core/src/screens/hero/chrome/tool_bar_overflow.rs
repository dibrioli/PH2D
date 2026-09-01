// ph2d-chrome-sync:z=45 (dispatch priority, ADR-0107; lower = earlier)
//! ⭐⭐⭐ **OS DOIS MENUS DA FILA DE FERRAMENTAS** — abrem, e fecham-se ao servir.
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! São dois chips no fim da faixa, com a MESMA lei:
//!
//! | chip | abre | quem publica o corpo |
//! |---|---|---|
//! | `⋯` ([`ids::TOOL_BAR_OVERFLOW`]) | o que não coube na linha | `tool_bar::bar_split` |
//! | os pulldowns da área ([`ids::area_menu_button`]) | os comandos do editor com o canvas | o módulo |
//!
//! ⚠️ **UMA tabela, e não dois ficheiros.** A metade que se esquece ao copiar é a segunda — *servir
//! é fechar* —, e um menu que não fecha ao servir fica por cima do que o clique acabou de fazer.
//!
//! # ⭐⭐⭐ **SERVIR É FECHAR — e sob o DEDO quem o faz não é este ficheiro**
//!
//! ⛔⛔ **Medido por mutação em 2026-08-31, e duas redacções desta wave estavam erradas.** Apagar o
//! fecho deste ficheiro deixa **todos** os gates de gesto verdes, incluindo o do segundo toque.
//! Quem fecha, sob o ponteiro, é a **regra genérica do store** um nível abaixo: o
//! `dispatch::pointer_down` fecha todo menu aberto num **Down** primário que não *pertença* a ele
//! (`click_belongs_to_the_open_menu`) — e mede-se: depois do Down no chip o menu já está fechado, e
//! o Up seguinte não levanta `Click` nenhum.
//!
//! ⇒ isso vale de graça para os chips do `⋯` (ids do trilho) **e** para os comandos da área (ids de
//! um PAINEL, que o registry consome antes do chrome). ⚠️ Uma cura escrita no `pre_dispatch` para
//! «o painel consome e o menu fica aberto» foi construída nesta wave e **retirada**: era a terceira
//! cópia de uma lei que já existe.
//!
//! # ⚠️ Então por que o interruptor abaixo FICA
//!
//! Porque há uma fonte de `WidgetEvent::Click` **sem Down nenhum**: a **paleta de comandos global**
//! (`global_palette`) chama `HeroScreen::apply_event(Click(id))` directamente, e ela projecta a
//! lista que a fila pinta — estes dois chips incluídos. Por esse caminho o fecho genérico nunca
//! corre, e sem a metade de baixo escolher o chip na paleta **re-abriria** um menu já aberto.
//! O gate que o prende é `the_palette_path_toggles_the_menu_because_it_has_no_pointer_down`.
//!
//! # ⚠️ O z é `45`, entre os toggles de vista (`40`) e os do trilho (`50`)
//!
//! ⛔ **Um chip do transbordo não tem handler próprio, e é essa a decisão.** Os ids dentro do menu
//! são **os mesmos** da fila, logo quem despacha continua a ser o `chrome::rail_*`. Um verbo
//! copiado para aqui seria a segunda porta que o `CLAUDE.md` §5.0 cataloga como a espécie mais
//! cara de controlo morto: duas respostas para o mesmo clique, e a que envelhece é a que ninguém
//! relê.

use crate::ids;
use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use crate::screens::hero::HeroScreen;

/// **Os chips desta faixa e o menu que cada um abre** — o `⋯` mais um por pulldown de área.
///
/// ⚠️ **Derivada, e não escrita à mão:** a lista de pulldowns é do módulo, e uma tabela fixa aqui
/// ficaria com um chip a mais no dia em que um módulo publicasse menos. Ver
/// [`crate::ids::area_menu_button`].
fn bar_menus() -> impl Iterator<Item = (ph2d_a11y::NodeId, ContextMenuKind)> {
    std::iter::once((ids::TOOL_BAR_OVERFLOW, ContextMenuKind::ToolBarOverflow)).chain(
        (0..ids::MAX_AREA_MENUS).map(|slot| {
            (
                ids::area_menu_button(slot),
                ContextMenuKind::AreaCommands {
                    slot: u8::try_from(slot).unwrap_or(u8::MAX),
                },
            )
        }),
    )
}

/// Está algum destes menus aberto?
fn open_bar_menu(hero: &HeroScreen) -> Option<ContextMenuKind> {
    let open = hero.store.context_menu()?.kind;
    bar_menus().any(|(_, kind)| kind == open).then_some(open)
}

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    for (chip, kind) in bar_menus() {
        if id != chip {
            continue;
        }
        // ⚠️ Um segundo clique no MESMO chip FECHA — é um interruptor, não uma porta de sentido
        // único. ⛔ E com o OUTRO menu aberto, este clique tem de o trocar, não de o empilhar: o
        // `open_context_menu` substitui, que é o que dá a troca.
        if open_bar_menu(hero) == Some(kind) {
            hero.store.close_context_menu();
            return true;
        }
        // Ancorado por BAIXO do próprio chip, como os menus da barra: o rect vem do índice de
        // acerto do quadro anterior, que é onde o chip de facto ficou.
        let (x, y) = hero
            .hit_index
            .rect_for(chip)
            .map_or((0.0, hero.last_viewport.y), |r| (r.x, r.y + r.h));
        hero.store
            .open_context_menu(ContextMenuRequest { x, y, kind });
        return true;
    }
    false
}
