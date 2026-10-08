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
use crate::widget::{RADIAL_MAX_SECTORS as MAX_SECTORS, RadialItem};
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;

/// O id do sector *"More…"* — a porta para a paleta quando a secção não cabe.
///
/// ⚠️ **Ele NÃO é um comando**, e é por isso que tem id próprio em vez de um item da paleta: quem o
/// escolhe não está a executar coisa nenhuma, está a pedir a **outra vista** da mesma lista. O
/// router reconhece-o antes de tudo, num sítio só.
pub const RADIAL_MORE: NodeId = NodeId(0x0052_4144_494F_4C21);

/// O rótulo do sector de transbordo — a CHAVE (o pintor traduz). ⚠️ A nota antiga dizia que ele não
/// passava por i18n «porque o app é inglês-only»; a ordem do dono de 2026-09-16 (*«converter o APP
/// todo ao manual»*) pôs a moldura inteira na tabela, como os selos da hierarquia antes dela.
pub const MORE_LABEL: TextKey = TextKey::new("chrome.radial.more");

/// **AS FERRAMENTAS SOB A CANETA** — a secção do meio do rail, pronta para o radial.
///
/// Vazia quando o rail não tem secção de ferramentas (não há radial a abrir).
#[must_use]
pub fn build_radial_model(hero: &HeroScreen) -> Vec<RadialItem> {
    // ⭐ **A secção pede-se pelo NOME**, não pelo índice do divisor. Ela era
    // `split(Divider).nth(1)`, e no dia em que os dois toggles de painel saíram da lista para o
    // menu *Ver* o `1` passou a apontar para *espaço/vista*: o radial das FERRAMENTAS passou a
    // oferecer *Frame view*. O gate apanhou-o; a cura foi deixar de haver número.
    if let Some(bars) = hero.store.tool_bars().filter(|b| !b.column.is_empty()) {
        return from_column(&bars.column);
    }
    let entries = super::left_rail::tool_section(&hero.store, hero.rail_shows_painter_tools());
    let tools: Vec<RadialItem> = entries
        .iter()
        .filter_map(|e| Some(RadialItem::leaf(e.label()?, e.node_id()?)))
        .collect();
    fit(tools)
}

/// **A COLUNA ① DE UM MÓDULO, sob a caneta** (F4) — um sector por GRUPO da coluna (os grupos são as
/// famílias que o módulo declarou, não uma escolha daqui), e o grupo abre em leque.
///
/// ⚠️ **As amostras ficam de fora**: o clique sem Down não as opera (abrem pelo seletor), e o radial
/// só oferece o que um `Click(id)` executa.
///
/// ⚠️ **O último sector é SEMPRE a porta para a paleta**: a roda mostra a ①, e as opções da ② e as
/// ações da ④ moram na paleta — a porta diz onde está o resto.
#[must_use]
pub fn from_column(column: &[Vec<crate::interaction::BarButton>]) -> Vec<RadialItem> {
    use crate::interaction::BarFace;
    let mut groups: Vec<RadialItem> = column
        .iter()
        .filter_map(|g| {
            let leaves: Vec<RadialItem> = g
                .iter()
                .filter(|b| !matches!(b.face, BarFace::Swatch { .. }))
                .map(|b| RadialItem::leaf(b.label.clone(), b.id))
                .collect();
            let family = g.first()?.family;
            match leaves.len() {
                0 => None,
                1 => leaves.into_iter().next(),
                _ => Some(RadialItem::family(
                    ph2d_i18n::tr(family.name_key()),
                    fit_fan(leaves),
                )),
            }
        })
        .collect();
    groups.truncate(MAX_SECTORS - 1);
    groups.push(more());
    groups
}

/// O leque também não corta em silêncio: acima de [`crate::widget::RADIAL_FAN_MAX`] o último filho
/// é a porta.
fn fit_fan(mut items: Vec<RadialItem>) -> Vec<RadialItem> {
    let max = crate::widget::RADIAL_FAN_MAX;
    if items.len() > max {
        items.truncate(max - 1);
        items.push(more());
    }
    items
}

fn more() -> RadialItem {
    RadialItem::leaf(MORE_LABEL.tr(), RADIAL_MORE)
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
    items.push(more());
    items
}

#[cfg(test)]
#[path = "radial_tests.rs"]
mod tests;
