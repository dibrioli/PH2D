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
//! (`audio_editor` → *Audio Editor*) e **dois saíam feios** (`bgremoval` → *Bgremoval*, e um painel
//! que saiu com o 3D). Isso é cosmético e tem cura própria; uma tabela paralela não teria.

#[path = "global_palette_bars.rs"]
pub mod global_palette_bars;
/// Os comandos da barra de menus, com o caminho de onde moram.
#[path = "global_palette_menus.rs"]
pub mod global_palette_menus;
#[path = "global_palette_places.rs"]
pub mod global_palette_places;
/// A memória de sessão: os recentes e a seleção que rola para ficar à vista.
#[path = "global_palette_session.rs"]
mod global_palette_session;
pub use global_palette_session::PaletteSession;

use crate::widget::command_palette::{
    PaletteGroup, PaletteItem, PaletteLayout, PaletteModel, PaletteSelect, PaletteSub, item_ids,
    scoped, shortcut_column,
};
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

/// O título da paleta global — o que o cabeçalho mostra.
pub const GLOBAL_PALETTE_TITLE: TextKey = TextKey::new("chrome.palette.commands");

/// `snake_case` → `Title Case`. Ver o doc-header: é uma DERIVAÇÃO do id, nunca uma tabela.
#[must_use]
pub fn humanise_panel_id(id: &str) -> String {
    // Os que a derivação estraga: o id colou as palavras («Bgremoval», «Sfxlab») ou é só metade
    // do nome («Authored», «Tokens»). O nome é o título que a aba do painel já mostra.
    match id {
        "bgremoval" => return tr("panel.bg_removal.title").to_owned(),
        "sfxlab" => return tr("panel.sfxlab.title").to_owned(),
        "authored" => return tr("panel.authored.title").to_owned(),
        "tokens" => return tr("panel.tokens.title").to_owned(),
        _ => {}
    }
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
/// ser desenhada como menu. O atalho vai para a coluna dele pela porta dos menus
/// ([`shortcut_column`]).
fn palette_label(label: &str) -> String {
    shortcut_column(label.strip_prefix("\u{2014} ").unwrap_or(label))
}

/// O nome do chip com o atalho que a dica dele diz («Translate · G» ⇒ `G` na coluna), quando a
/// dica é o MESMO nome seguido da tecla.
fn with_tip_key(label: &str, tip: Option<&str>) -> String {
    match tip.and_then(|t| t.rsplit_once(crate::widget::command_palette::PATH_SEP)) {
        Some((name, key)) if name.eq_ignore_ascii_case(label) && !key.is_empty() => {
            shortcut_column(&format!(
                "{label}{}{key}",
                crate::widget::command_palette::PATH_SEP
            ))
        }
        _ => label.to_owned(),
    }
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
    // ── A BARRA DE MENUS (File · Edit · View · Window · Help), através das cascatas. Um id que
    //    mora nela leva o caminho DELA e aparece uma vez só: os outros grupos perdem-no. ──
    let mut bar = global_palette_menus::groups();

    // ── Os comandos das abas de FAMÍLIA: os que têm linha na barra vão para o lugar dela
    //    (`File · New`, `Window · Plan`); só os do menu PRÓPRIO da aba ficam em «This tab». ──
    let mut tab_only: Vec<PaletteItem> = Vec::new();
    for c in super::surface_input::commands(hero) {
        let item = PaletteItem {
            badge: None,
            label: shortcut_column(&c.label),
            id: c.id,
            keywords: "",
        };
        match super::document_tabs::doc_menu_kind(c.menu)
            .and_then(global_palette_menus::place_of_menu)
        {
            Some(at) => global_palette_menus::place(&mut bar, at, item),
            None => tab_only.push(item),
        }
    }

    // ── As listas que uma linha da barra ABRE (Preferences ▸ …, Theme): cada folha com o caminho
    //    real (`Edit · Preferences · Motion`). Um pick nelas é um `Click(id)` sem geometria. ──
    for kind in super::menu_rows::TOPBAR_LEAF_MENUS {
        let at = global_palette_menus::place_of_leaf_menu(*kind);
        for (id, label, _) in super::menu_rows::menu_rows(*kind) {
            let item = PaletteItem {
                badge: None,
                label: palette_label(label.tr()),
                id: *id,
                keywords: "",
            };
            global_palette_menus::place(&mut bar, at.clone(), item);
        }
    }
    let in_bar = |id: NodeId| {
        bar.iter()
            .flat_map(|g| &g.subs)
            .flat_map(|s| &s.items)
            .any(|it| it.id == id)
    };

    let mut groups = Vec::new();
    // ── Os chips do RAIL (ferramentas + os toggles de painel do topo da coluna); o atalho vem da
    //    dica do chip («Translate · G»). ──
    let tools: Vec<PaletteItem> =
        super::left_rail::rail_entries(&hero.store, hero.rail_shows_painter_tools())
            .iter()
            .filter_map(|e| {
                let (label, id) = (e.label()?, e.node_id()?);
                Some(PaletteItem {
                    badge: None,
                    label: with_tip_key(label, hero.store.tooltip_for(id)),
                    id,
                    keywords: "",
                })
            })
            .filter(|it| !in_bar(it.id))
            .collect();
    if !tools.is_empty() {
        groups.push(PaletteGroup {
            title: tr("chrome.palette.tools").into(),
            color: ColorToken::NodeCatTransform,
            subs: vec![PaletteSub {
                title: None,
                items: tools,
            }],
        });
    }

    // ── As BARRAS do módulo na mão (F4): ①, ② com os pulldowns, ④ — pelo nome do LUGAR ──
    groups.extend(global_palette_bars::groups(&hero.store));

    tab_only.retain(|it| !in_bar(it.id));
    if !tab_only.is_empty() {
        groups.push(PaletteGroup {
            title: tr("chrome.palette.document").into(),
            color: ColorToken::NodeCatSource,
            subs: vec![PaletteSub {
                title: None,
                items: tab_only,
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
            if in_bar(p.manifest.panel_node_id) {
                continue;
            }
            panels.push(PaletteItem {
                badge: None,
                label: humanise_panel_id(p.manifest.id),
                id: p.manifest.panel_node_id,
                keywords: "",
            });
        }
    });
    panels.sort_by(|a, b| a.label.cmp(&b.label));
    if !panels.is_empty() {
        groups.push(PaletteGroup {
            title: tr("chrome.palette.panels").into(),
            color: ColorToken::NodeCatUtility,
            subs: vec![PaletteSub {
                title: None,
                items: panels,
            }],
        });
    }
    groups.extend(bar);

    as_list(
        PaletteModel {
            title: GLOBAL_PALETTE_TITLE.tr().into(),
            groups,
            // O `Ctrl+K` não tem caixa nenhuma — ver `PaletteModel::toggle`.
            toggle: None,
            tips: Vec::new(),
            select: None,
            layout: PaletteLayout::Cloud,
        },
        hero.palette_session.recent(),
    )
}

/// ⭐ **A paleta global é uma LISTA** (dono 12/11): os recentes da sessão em cima e o destaque na
/// primeira linha que se pinta (o `select` é o que dá as setas e o Enter do destacado no store).
fn as_list(mut model: PaletteModel, recent: &[NodeId]) -> PaletteModel {
    model.layout = PaletteLayout::List {
        recent: recent.to_vec(),
    };
    let first = item_ids(&scoped(&model, "", None).model).first().copied();
    model.select = first.map(|initial| PaletteSelect {
        initial,
        teach: Vec::new(),
    });
    model
}

/// `Ctrl+K` — abre a paleta sobre o que o app oferece AGORA, com os prefixos `@` e `:` da aba
/// activa (I14). O modelo é construído na abertura, nunca por quadro.
pub fn open(hero: &mut super::HeroScreen) {
    open_with(hero, None);
}

/// [`open`] com o grupo de uma família que vive FORA do hero (o Run Command do Pixel, P12 R): a
/// família emite as linhas e executa o pick dela pelo próprio id ([`drain`] deixa-o no canal).
pub fn open_with(hero: &mut super::HeroScreen, family: Option<PaletteGroup>) {
    let mut model = build_global_model(hero);
    model.groups.extend(family);
    let model = as_list(model, hero.palette_session.recent());
    hero.store.open_command_palette(model);
    hero.palette_session.opened();
    let scopes = global_palette_places::scopes(hero);
    hero.store.set_command_palette_scopes(scopes);
}

/// Drena o pick DESTA paleta e executa-o (uma vez por quadro). ⚠️ O pick chega com a paleta já
/// fechada: o reconhecimento é «este id está no que EU ofereço agora», e o modelo reconstrói-se para
/// a pergunta; um item dos prefixos reconhece-se pelo id (sem perguntar à aba).
pub fn drain(hero: &mut super::HeroScreen) {
    global_palette_session::follow(hero);
    let model = build_global_model(hero);
    let mine = |id: NodeId| model.is_item(id) || global_palette_places::owns(id);
    let Some(id) = hero.store.take_command_pick_if(mine) else {
        return;
    };
    route_global_pick(hero, id);
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
    let done = run_pick(hero, id);
    if done {
        hero.palette_session.remember(id);
    }
    done
}

fn run_pick(hero: &mut super::HeroScreen, id: NodeId) -> bool {
    let query = hero.store.command_pick_query().to_owned();
    if global_palette_places::route(hero, id, &query) {
        return true;
    }
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
    // ⭐ F4: o comando das barras pisca ONDE MORA — a paleta ensina a UI nova. A amostra abre o
    // seletor pela porta dela (o clique sem Down não a opera).
    if global_palette_bars::is_swatch(&hero.store, id) {
        hero.store.open_picker_on(id);
        hero.store.flash(id);
        return true;
    }
    let teach = global_palette_bars::owns(&hero.store, id);
    let done = hero.apply_event(crate::interaction::WidgetEvent::Click(id));
    if teach {
        hero.store.flash(id);
    }
    done
}

#[cfg(test)]
#[path = "global_palette_tests.rs"]
mod tests;
