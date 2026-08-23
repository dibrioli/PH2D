//! **O MODELO DO MENU RADIAL** (estudo de UI viva, E4) — *que oito coisas ficam sob a caneta*.
//!
//! # ⭐ Ele é uma PROJECÇÃO, nunca uma tabela
//!
//! É a mesma disciplina que a paleta global pagou: o radial não conhece comando nenhum: ele mostra
//! uma **vista** da lista que o app já oferece, e quem executa é
//! [`super::global_palette::route_global_pick`] — o mesmo router. É isso que torna um item morto
//! **estruturalmente impossível**: um comando que deixe de existir sai das duas vistas ao mesmo
//! tempo, porque as duas leem a mesma lista.
//!
//! # ⭐⭐ E "as ferramentas" é DERIVADO, não escolhido
//!
//! O rail já se declara em secções, separadas por [`ToolRailEntry::Divider`]: os interruptores de
//! painel no topo · **as FERRAMENTAS** · o espaço/vista/undo/redo no fim. O radial toma a secção do
//! MEIO — a que o rail chama de ferramentas —, e não uma lista que eu tenha escolhido.
//!
//! ⚠️ **Medido em 2026-08-23:** essa secção tem **4** entradas no modo normal (Translate · Rotate ·
//! Scale · Pivot) e **13** no Painter (as doze da pintura mais o chip de cor). Ou seja: num modo
//! ela cabe nos oito sectores com folga, e no outro **não cabe** — e o número não é meu para
//! ajustar.
//!
//! # ⛔ O que não cabe NÃO é truncado em silêncio
//!
//! Quando a secção passa dos [`MAX_SECTORS`], o radial mostra os primeiros **sete** e o oitavo é a
//! porta para a **paleta** (`Ctrl+K`), que segura qualquer número. *Um teto que esconde o que não
//! coube é um teto que mente*; este diz onde o resto está.

use super::HeroScreen;
use crate::widget::{RADIAL_MAX_SECTORS as MAX_SECTORS, RadialItem, ToolRailEntry};
use ph2d_a11y::NodeId;

/// O id do sector *"More…"* — a porta para a paleta quando a secção não cabe.
///
/// ⚠️ **Ele NÃO é um comando**, e é por isso que tem id próprio em vez de um item da paleta: quem o
/// escolhe não está a executar coisa nenhuma, está a pedir a **outra vista** da mesma lista. O
/// router reconhece-o antes de tudo, num sítio só.
pub const RADIAL_MORE: NodeId = NodeId(0x0052_4144_494F_4C21);

/// O rótulo do sector de transbordo. ⚠️ Não passa por i18n pela mesma razão que os selos da
/// hierarquia: o app é inglês-only por decisão do Enio.
pub const MORE_LABEL: &str = "More...";

/// **AS FERRAMENTAS SOB A CANETA** — a secção do meio do rail, pronta para o radial.
///
/// Vazia quando o rail não tem secção de ferramentas (não há radial a abrir).
#[must_use]
pub fn build_radial_model(hero: &HeroScreen) -> Vec<RadialItem> {
    let entries = super::left_rail::rail_entries(&hero.store, hero.rail_shows_painter_tools());
    // A secção do MEIO: entre o primeiro e o segundo divisor.
    let tools: Vec<RadialItem> = entries
        .split(|e| matches!(e, ToolRailEntry::Divider))
        .nth(1)
        .unwrap_or(&[])
        .iter()
        .filter_map(|e| {
            Some(RadialItem {
                label: e.label()?.to_string(),
                id: e.node_id()?,
            })
        })
        .collect();
    fit(tools)
}

/// **O QUE CABE, mais a porta para o resto.**
///
/// ⚠️ Ela é uma função à parte para poder ser medida sem um `HeroScreen`: o transbordo é a regra
/// que mais fácil se escreve errada (um `truncate` mudo é uma linha), e é a que mais custa quando
/// erra — o artista procura uma ferramenta que o menu decidiu não mostrar.
#[must_use]
pub fn fit(mut items: Vec<RadialItem>) -> Vec<RadialItem> {
    if items.len() <= MAX_SECTORS {
        return items;
    }
    items.truncate(MAX_SECTORS - 1);
    items.push(RadialItem {
        label: MORE_LABEL.to_string(),
        id: RADIAL_MORE,
    });
    items
}

#[cfg(test)]
#[path = "radial_tests.rs"]
mod tests;
