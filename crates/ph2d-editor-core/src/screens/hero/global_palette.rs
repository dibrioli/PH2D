//! **A PALETA DE COMANDOS GLOBAL — e ela é uma PROJEÇÃO, não uma tabela.**
//!
//! ## A decisão que decide tudo o resto
//!
//! Uma paleta de comandos precisa de *uma lista de todos os verbos do app, com um nome humano*. A
//! forma óbvia de a ter é escrevê-la; a forma óbvia é também a que **apodrece no dia seguinte** — o
//! verbo novo nasce na barra e não na paleta, e ninguém repara porque nada falha.
//!
//! Este módulo não escreve lista nenhuma. Ele **projeta três que o app já mantém**:
//!
//! * [`super::left_rail::rail_entries`] — os chips do rail, cada um com `node_id()` e `label()`;
//! * o **registry de painéis** — os 23 painéis registados, pelo `panel_node_id` do manifesto;
//! * [`super::menu_rows::menu_rows`] — as **rows** dos menus folha da barra de topo.
//!
//! ⚠️ **E a primeira já era a fonte de verdade de outra coisa**: o gate
//! `every_painted_rail_button_is_dispatched_by_somebody` existe precisamente porque *uma lista
//! escrita à mão num teste driftaria da tela*. A paleta herda essa propriedade de graça — um verbo
//! que aparece aqui é, **por construção**, um verbo que o app consome.
//!
//! ## O que a paleta NÃO sabe
//!
//! Ela não sabe o que um item *significa*. O item carrega o **`NodeId` do próprio botão**, e quem o
//! escolhe **re-toca o clique** (`HeroScreen::apply_event(WidgetEvent::Click(id))`) — a mesma porta
//! por onde o rato entra. Não há segunda tabela `id → acção` para divergir da primeira, e é o
//! mesmo desenho com que a paleta do Motion roteia um pick (*«exactamente como um pick de rato»*).
//!
//! ## ⚠️ O PILL da barra fica de fora; a ROW de dentro dele NÃO
//!
//! A primeira versão deste módulo projetava os pills da barra, e o gate reprovou nove deles na
//! primeira corrida: `Forge`, `Level_01`, `Save`, `Open`, `MIX`, `WAVE`, `WIDGET`, `GRID`,
//! `SETTINGS`. Eles **não estão mortos** — o que eles fazem é outra coisa: são **abridores de menu
//! ancorados a um RECTÂNGULO** (`pointer_down_menus.rs` chama `open_context_menu` com o `hit_rect`
//! do chip). Um `WidgetEvent::Click` neles não faz nada, medido; o que os aciona é um Down numa
//! posição.
//!
//! ⇒ **Os verbos do app são de dois tipos**, e só um é servível por uma paleta:
//!
//! 1. o clique **É** o verbo (os chips do rail; a visibilidade de um painel; **uma row de menu**) —
//!    endereçável por id, porque `chrome::dispatch_all` o resolve **sem geometria nenhuma**;
//! 2. o clique **ABRE um menu onde o verbo está** (o pill Save, Open, Theme, Settings…) —
//!    posicional, e uma paleta não tem rectângulo: o chip pode nem estar visível.
//!
//! ⚠️ E a conclusão que a primeira versão tirou daí era LARGA DEMAIS: ela escreveu *«a barra de topo
//! fica de fora»*, quando o que ela mediu foi só que **o PILL** fica. Uma row de menu folha é
//! **tipo 1** — ver [`super::menu_rows::TOPBAR_LEAF_MENUS`], que nomeia as três classes de menu que
//! **não** são servíveis e o mecanismo de cada uma.
//!
//! ## O nome de um painel é DERIVADO do id, e não uma tabela
//!
//! O trait `Panel` carrega `ID`, `NODE_ID` e `DEFAULT_VISIBLE` — **não** um título. Acrescentar um
//! `TITLE` tocaria as ~23 crates de painel (contrato foundational, churn próprio), e uma tabela
//! `id → nome` aqui seria a segunda lista que apodrece. Então o título é `snake_case` → Title Case,
//! uma **derivação do facto que já existe**: medido sobre os 23 ids registados, **21 saem limpos**
//! (`audio_editor` → *Audio Editor*) e **dois saem feios** (`bgremoval` → *Bgremoval*, `sculpt3d` →
//! *Sculpt3d*). Isso é cosmético e tem cura própria; uma tabela paralela não teria.

use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
use ph2d_a11y::NodeId;
use ph2d_tokens::ColorToken;

/// O título da paleta global — o que o cabeçalho mostra.
pub const GLOBAL_PALETTE_TITLE: &str = "Commands";

/// `snake_case` → `Title Case`. Ver o doc-header: é uma DERIVAÇÃO do id, nunca uma tabela.
#[must_use]
pub fn humanise_panel_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for (i, word) in id.split('_').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
}

/// O rótulo de uma row de menu, lido como NOME em vez de como linha de menu.
///
/// ⚠️ Uma coisa só muda: o **travessão de recuo**. Metade das rows do menu Look abre com `"— "`
/// porque o pintor de rows não tem afordância de indentação, então o menu **codifica o recuo como
/// caractere** — e numa lista plana de pills não há nada contra o que recuar. O resto passa
/// verbatim, incluindo o atalho (`Save · Cmd+S`): num *command palette* mostrar a tecla ao lado do
/// verbo é o que todo produto do género faz, e é informação que o artista quer ali.
///
/// Isto **não é uma segunda tabela** — é uma derivação da mesma string, no ponto onde ela deixa de
/// ser desenhada como menu.
fn palette_label(label: &str) -> String {
    label.strip_prefix("\u{2014} ").unwrap_or(label).to_string()
}

/// **Constrói o modelo da paleta global a partir do que o app REGISTA agora.**
///
/// O modo sai da porta única [`super::HeroScreen::rail_shows_painter_tools`], a MESMA que o `paint`
/// pergunta: com o Painter em mãos a coluna mostra as ferramentas de pintura, e a paleta oferece as
/// mesmas. Ela reflecte o momento em vez de mostrar um catálogo fixo — *o que ela oferece é o que
/// está alcançável agora*, que é a promessa que uma paleta de comandos faz.
///
/// ⚠️ O grupo dos painéis sai do **registry global**, então numa suíte que não o povoou ele nasce
/// vazio — e isso é correcto, não um bug: o gate dele mora na `ph2d-panel-registry-init`, a crate
/// mais barata que enxerga todos os painéis (a mesma disciplina do censo do scrub).
#[must_use]
pub fn build_global_model(hero: &super::HeroScreen) -> PaletteModel {
    let mut groups = Vec::new();

    // ── Os chips do RAIL (ferramentas + os toggles de painel do topo da coluna) ──
    let tools: Vec<PaletteItem> =
        super::left_rail::rail_entries(&hero.store, hero.rail_shows_painter_tools())
            .iter()
            .filter_map(|e| {
                Some(PaletteItem {
                    label: e.label()?.to_string(),
                    id: e.node_id()?,
                })
            })
            .collect();
    if !tools.is_empty() {
        groups.push(PaletteGroup {
            title: "Tools".into(),
            color: ColorToken::NodeCatTransform,
            subs: vec![PaletteSub {
                title: None,
                items: tools,
            }],
        });
    }

    // ── Os PAINÉIS registados: "mostrar/esconder X", o verbo que mais falta ──
    // É aqui que uma paleta se paga: um painel sem chip no rail e sem pill na barra (o editor de
    // áudio, o Tuning da aquarela, os params do Motion) só é alcançável por quem sabe a tecla.
    let mut panels: Vec<PaletteItem> = Vec::new();
    // ⚠️ `with_registry_opt` e não `with_registry_ref`: a variante tolerante é a que o orquestrador
    // já usa, e por este exacto motivo — o registry vive numa crate que DEPENDE desta, então a
    // suíte da `editor-core` corre sem ele e a variante estrita PANICARIA.
    crate::panel::with_registry_opt(|reg| {
        for p in reg.panels() {
            panels.push(PaletteItem {
                label: humanise_panel_id(p.manifest.id),
                id: p.manifest.panel_node_id,
            });
        }
    });
    panels.sort_by(|a, b| a.label.cmp(&b.label));
    if !panels.is_empty() {
        groups.push(PaletteGroup {
            title: "Panels".into(),
            color: ColorToken::NodeCatUtility,
            subs: vec![PaletteSub {
                title: None,
                items: panels,
            }],
        });
    }

    // ── As ROWS dos menus da barra de topo — o «tipo 2» que o doc-header nomeia ──
    // O pill abre um menu ancorado a um rectângulo, e por isso não é servível daqui; a **row** de
    // dentro dele é, porque um pick nela é um `Click(id)` que o `chrome::dispatch_all` resolve sem
    // geometria nenhuma. É a diferença entre *abrir* um menu e *escolher* nele.
    //
    // ⚠️ Um GRUPO, um SUB por menu — e não N grupos. Cada grupo é um CARD no masonry, então nove
    // cards de duas ou três linhas seriam ruído; um sub é exactamente «um cluster nomeado dentro de
    // uma categoria», que é o que estas rows são.
    let menus: Vec<PaletteSub> = super::menu_rows::TOPBAR_LEAF_MENUS
        .iter()
        .filter_map(|kind| {
            let items: Vec<PaletteItem> = super::menu_rows::menu_rows(*kind)
                .iter()
                .map(|(id, label, _)| PaletteItem {
                    label: palette_label(label),
                    id: *id,
                })
                .collect();
            (!items.is_empty()).then(|| PaletteSub {
                title: super::menu_rows::menu_title(*kind).map(str::to_string),
                items,
            })
        })
        .collect();
    if !menus.is_empty() {
        groups.push(PaletteGroup {
            title: "Menus".into(),
            color: ColorToken::NodeCatOutput,
            subs: menus,
        });
    }

    PaletteModel {
        title: GLOBAL_PALETTE_TITLE.into(),
        groups,
    }
}

/// **EXECUTA o comando escolhido — e mora aqui de propósito.**
///
/// Quem EMITE cada item é a [`build_global_model`], e quem o executa é esta função, no mesmo módulo:
/// é o que torna um item morto **estruturalmente impossível** em vez de apenas gateado. Os dois
/// tipos de item que o modelo emite têm as duas execuções:
///
/// * um id de **painel** (o `panel_node_id` do manifesto) ⇒ alterna a visibilidade dele;
/// * qualquer outro id vem do **rail** ⇒ **re-toca o clique** pela mesma porta do rato
///   (`HeroScreen::apply_event`). A paleta nunca aprende o que um verbo *significa* — não há segunda
///   tabela `id → acção` para divergir da primeira.
///
/// Devolve `false` quando o id não é dela (o canal do pick é partilhado com a paleta de nós do
/// Motion, e um consumidor que toma incondicionalmente é o que torna a ORDEM load-bearing).
pub fn route_global_pick(hero: &mut super::HeroScreen, id: NodeId) -> bool {
    let panel_id = {
        let mut found: Option<&'static str> = None;
        crate::panel::with_registry_opt(|reg| {
            found = reg
                .panels()
                .iter()
                .find(|p| p.manifest.panel_node_id == id)
                .map(|p| p.manifest.id);
        });
        found
    };
    if let Some(pid) = panel_id {
        let now = hero.is_panel_visible(pid);
        <super::HeroScreen as crate::panel::PanelHostInternal>::set_panel_visible(hero, pid, !now);
        return true;
    }
    hero.apply_event(crate::interaction::WidgetEvent::Click(id))
}

#[cfg(test)]
#[path = "global_palette_tests.rs"]
mod tests;
