//! ⭐ **O MENU ADD DE OBJECTOS** — o modal que o `+` da Hierarquia e o `Shift+A` abrem
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`, F1).
//!
//! ⛔ **Não é um modal novo:** é o [`crate::widget::command_palette`]. Este módulo só constrói o
//! MODELO e devolve o pick.
//!
//! ⭐ **Cada família declara as suas entradas na própria crate** (`&[AddEntry]`), e a COMPOSIÇÃO
//! junta as que estão compiladas: um tipo cuja crate saiu do build não deixa um item morto.
//! O pick volta como a [`AddEntry`], e quem a sabe criar é a família dona — com os recursos dela.

use crate::widget::command_palette::{
    PaletteGroup, PaletteItem, PaletteModel, PaletteSelect, PaletteSub, PaletteTeach, PaletteTip,
};
use crate::widget::illustration::Mark;
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_tokens::ColorToken;

/// **O grupo do menu** — o que o objecto É para quem faz o jogo (Crítica `#addobject`, versão B, dono
/// 07/11): imagens, desenhos, peças de jogo, vazios.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddGroup {
    /// Uma imagem: nova, Sprite, Animated, Pixel, fundo.
    Pictures,
    /// Um desenho com modo próprio: Vetor, Texto, Flip, Esqueleto, Motion.
    Drawing,
    /// Um vazio com o componente de jogo já posto.
    Game,
    /// O objecto vazio e a Macro.
    Empty,
}

impl AddGroup {
    /// Todos, na ordem em que o modal os mostra.
    pub const ALL: [AddGroup; 4] = [
        AddGroup::Pictures,
        AddGroup::Drawing,
        AddGroup::Game,
        AddGroup::Empty,
    ];

    /// O título do grupo.
    #[must_use]
    pub fn title(self) -> &'static str {
        ph2d_i18n::tr(match self {
            AddGroup::Pictures => "object_add.group.pictures",
            AddGroup::Drawing => "object_add.group.drawing",
            AddGroup::Game => "object_add.group.game",
            AddGroup::Empty => "object_add.group.empty",
        })
    }

    /// A cor do grupo — das sete `NodeCat*` que o modal já pinta.
    #[must_use]
    pub const fn color(self) -> ColorToken {
        match self {
            AddGroup::Pictures => ColorToken::NodeCatSource,
            AddGroup::Drawing => ColorToken::NodeCatFx,
            AddGroup::Game => ColorToken::NodeCatOutput,
            AddGroup::Empty => ColorToken::NodeCatUtility,
        }
    }
}

/// A cor do grupo *Recent*.
const RECENT_COLOR: ColorToken = ColorToken::NodeCatFocus;

/// ⭐ **Uma busca que não acha, a ensinar** (plano 24 §2.5) — as palavras e a frase por chave i18n, e a
/// entrada que o botão destaca. Mora na família que sabe montar a peça.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AddTeach {
    /// As palavras (inglês, vírgula) que a disparam inteiras.
    pub words: TextKey,
    /// A frase: de que peças isto se monta.
    pub text: TextKey,
    /// A entrada por onde se começa.
    pub start: AddEntry,
}

impl AddTeach {
    /// O ensino de frase `text`, palavras `words` (chaves i18n) e começo `start`.
    #[must_use]
    pub const fn new(text: &'static str, words: &'static str, start: AddEntry) -> Self {
        Self {
            words: TextKey::new(words),
            text: TextKey::new(text),
            start,
        }
    }
}

/// ⭐ **O gesto «destaca, depois escolhe»** do modal (dono 07/11): o destaque inicial, o ensino e os
/// recentes (ids de entrada, o mais novo primeiro).
pub struct AddSelect<'a> {
    pub initial: AddEntry,
    pub teach: &'a [AddTeach],
    pub recent: &'a [NodeId],
}

/// ⭐ **A dica ILUSTRADA de uma entrada** (GameEngineFuncional N1, decisão 1 do dono no plano 03): a
/// explicação completa e o desenho do que o objecto É — nunca o nome da Godot.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AddTip {
    /// A chave i18n da explicação (várias linhas).
    pub body: TextKey,
    /// O desenho ([`crate::widget::illustration`]).
    pub picture: &'static [Mark],
}

/// **Uma entrada do menu**: o rótulo (que é também a IDENTIDADE), o grupo e, se tiver, a dica.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AddEntry {
    /// A chave i18n do rótulo. O id do item é o hash dela — sobrevive a inserções no meio.
    pub key: TextKey,
    /// Onde o modal a mostra.
    pub group: AddGroup,
    /// A dica ilustrada (`None` = nenhuma).
    pub tip: Option<AddTip>,
    /// A posição dentro do grupo (menor primeiro) — a ordem da maquete, não a das famílias compiladas.
    pub order: u8,
}

impl AddEntry {
    /// Uma entrada.
    #[must_use]
    pub const fn new(key: &'static str, group: AddGroup) -> Self {
        Self {
            key: TextKey::new(key),
            group,
            tip: None,
            order: u8::MAX,
        }
    }

    /// A mesma entrada, na posição `order` do grupo.
    #[must_use]
    pub const fn at(self, order: u8) -> Self {
        Self { order, ..self }
    }

    /// ⭐ **As palavras de busca** (`<chave>.keywords`, inglês, por vírgula — a lei das listas do Add
    /// Component); `""` sem lista.
    #[must_use]
    pub fn keywords(self) -> &'static str {
        let key = format!("{}.keywords", self.key.key());
        let words = ph2d_i18n::tr(&key);
        if words == key { "" } else { words }
    }

    /// A mesma entrada, com a dica ilustrada: a explicação pela chave `body_key` e o desenho.
    #[must_use]
    pub const fn with_tip(self, body_key: &'static str, picture: &'static [Mark]) -> Self {
        Self {
            tip: Some(AddTip {
                body: TextKey::new(body_key),
                picture,
            }),
            ..self
        }
    }

    /// O id de item desta entrada no modal.
    #[must_use]
    pub const fn id(self) -> NodeId {
        ph2d_tool_registry::hash_node_id(self.key.key())
    }
}

/// **As entradas da fundação**: o vazio e a imagem (que abre o diálogo de tamanho do `Ctrl+N`).
pub const EMPTY: AddEntry = AddEntry::new("object_add.empty", AddGroup::Empty)
    .at(0)
    .with_tip("object_add.empty.tip", crate::object_add_dicas::EMPTY);
/// Ver [`EMPTY`].
pub const IMAGE: AddEntry = AddEntry::new("object_add.image", AddGroup::Pictures)
    .at(0)
    .with_tip("object_add.image.tip", crate::object_add_dicas::IMAGE);
/// Ver [`EMPTY`].
pub const CORE: &[AddEntry] = &[IMAGE, EMPTY];

/// ⭐ **O modelo do modal**, sobre as famílias compiladas.
///
/// Cada grupo tem as entradas que podem nascer, na ordem de [`AddEntry::order`], e depois as que não
/// podem; um grupo sem entrada nenhuma não aparece. Com `select`, o *Recent* vem no topo e o modal
/// destaca antes de escolher ([`PaletteSelect`]).
///
/// ⚠️ `why_not` é a razão pela qual uma entrada **não pode nascer agora** (`None` = pode). Ela
/// APARECE na mesma, a seguir às que podem, com a razão no rótulo — a lei da paleta de formas: o
/// artista sabe que o tipo existe, e o clique responde com a mesma frase.
#[must_use]
pub fn build(
    families: &[&[AddEntry]],
    why_not: &dyn Fn(AddEntry) -> Option<&'static str>,
    select: Option<AddSelect<'_>>,
) -> PaletteModel {
    let all = || families.iter().flat_map(|f| f.iter().copied());
    let item = |e: AddEntry| {
        let label = e.key.tr();
        let label = match why_not(e) {
            None => label.to_string(),
            Some(reason) => format!("{label}  \u{2014}  {reason}"),
        };
        PaletteItem {
            badge: None,
            label,
            id: e.id(),
            keywords: e.keywords(),
        }
    };
    let recent: Vec<PaletteItem> = select
        .as_ref()
        .map(|s| s.recent)
        .unwrap_or_default()
        .iter()
        .filter_map(|id| entry_of_pick(families, *id))
        .filter(|e| why_not(*e).is_none())
        .take(crate::interaction::WidgetStore::ADD_RECENT_MAX)
        .map(item)
        .collect();
    let recent = (!recent.is_empty()).then(|| PaletteGroup {
        title: ph2d_i18n::tr("object_add.group.recent").to_string(),
        color: RECENT_COLOR,
        subs: vec![PaletteSub {
            title: None,
            items: recent,
        }],
    });
    let groups = AddGroup::ALL.into_iter().filter_map(|group| {
        let mut entries: Vec<AddEntry> = all().filter(|e| e.group == group).collect();
        entries.sort_by_key(|e| e.order);
        let (ready, blocked): (Vec<_>, Vec<_>) =
            entries.into_iter().partition(|e| why_not(*e).is_none());
        let subs: Vec<PaletteSub> = [ready, blocked]
            .into_iter()
            .filter(|v| !v.is_empty())
            .map(|v| PaletteSub {
                title: None,
                items: v.into_iter().map(item).collect(),
            })
            .collect();
        (!subs.is_empty()).then(|| PaletteGroup {
            title: group.title().to_string(),
            color: group.color(),
            subs,
        })
    });
    // ⭐ A dica de toda entrada que a tem — também das que não podem nascer agora: o artista lê o que
    //    o tipo é antes de saber porque não pode.
    let tips = all()
        .filter_map(|e| {
            e.tip.map(|t| PaletteTip {
                id: e.id(),
                title: e.key.tr().to_string(),
                body: t.body.tr().to_string(),
                picture: t.picture,
            })
        })
        .collect();
    let select = select.map(|s| PaletteSelect {
        initial: s.initial.id(),
        teach: s
            .teach
            .iter()
            .map(|t| PaletteTeach {
                words: t.words.tr(),
                text: t.text.tr().to_string(),
                start: t.start.id(),
                start_label: ph2d_i18n::tr("object_add.teach.start")
                    .replace("{name}", t.start.key.tr()),
            })
            .collect(),
    });
    PaletteModel {
        title: ph2d_i18n::tr("object_add.title").to_string(),
        groups: recent.into_iter().chain(groups).collect(),
        toggle: None,
        tips,
        select,
    }
}

/// A entrada que este id de item nomeia, se alguma. É o inverso de [`AddEntry::id`], e varre as
/// MESMAS fatias que o [`build`] — uma segunda lista envelheceria na primeira entrada nova.
#[must_use]
pub fn entry_of_pick(families: &[&[AddEntry]], id: NodeId) -> Option<AddEntry> {
    families
        .iter()
        .flat_map(|f| f.iter())
        .find(|e| e.id() == id)
        .copied()
}

/// ⭐ **O smoke do menu**: `PH2D_OBJECT_ADD_SMOKE=1` abre-o UMA vez, no primeiro quadro — é como a
/// foto do passo do smoke o apanha aberto, já que o clique sintético não chega à tela virtual
/// (`docs/Components/ferramentas/fotografa_cena.sh`).
#[must_use]
pub fn take_smoke_open() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    static TAKEN: AtomicBool = AtomicBool::new(false);
    !TAKEN.swap(true, Ordering::Relaxed)
        && std::env::var("PH2D_OBJECT_ADD_SMOKE").is_ok_and(|v| v == "1")
}

/// ⭐ **O rato do smoke sobre um item** — `PH2D_OBJECT_ADD_HOVER=<chave do item>` (ex.:
/// `object_add.game.character`), com o menu aberto pelo [`take_smoke_open`]: a foto apanha a dica
/// ilustrada desse item, que o rato sintético não consegue pedir na tela virtual.
#[must_use]
pub fn smoke_hover() -> Option<NodeId> {
    use std::sync::OnceLock;
    static ID: OnceLock<Option<NodeId>> = OnceLock::new();
    *ID.get_or_init(|| {
        let key = std::env::var("PH2D_OBJECT_ADD_HOVER").ok()?;
        Some(ph2d_tool_registry::hash_node_id_runtime(&key))
    })
}

/// ⭐ **O smoke do menu sobre o `store`, num passo**: o rato sobre um item ([`smoke_hover`], com o menu
/// aberto) e, UMA vez, o PICK de `PH2D_OBJECT_ADD_PICK=<chave do item>` — o objecto nasce pelo caminho do
/// menu e fica selecionado, e a foto apanha o Inspector de cada tipo (plano GEF 19 E5).
pub fn smoke_drive(store: &mut crate::interaction::WidgetStore) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static TAKEN: AtomicBool = AtomicBool::new(false);
    static QUERY: AtomicBool = AtomicBool::new(false);
    if let Some(id) = smoke_hover().filter(|_| store.command_palette_open()) {
        // No modo de seleção a dica é a do DESTACADO (plano 24): o smoke destaca em vez de pairar.
        if store.command_palette_selects() {
            store.highlight_command_palette(id);
        } else {
            store.set_hot(Some(id));
        }
    }
    // ⭐ `PH2D_OBJECT_ADD_QUERY=<texto>`: a busca escrita UMA vez, com o menu aberto (as fotos de «wall» e
    //    de «coin» do plano 24 §4).
    if store.command_palette_open()
        && let Ok(q) = std::env::var("PH2D_OBJECT_ADD_QUERY")
        && !QUERY.swap(true, Ordering::Relaxed)
    {
        q.chars().for_each(|c| store.command_palette_push_char(c));
    }
    if let Ok(key) = std::env::var("PH2D_OBJECT_ADD_PICK")
        && !TAKEN.swap(true, Ordering::Relaxed)
    {
        store.set_command_pick(ph2d_tool_registry::hash_node_id_runtime(&key));
    }
}

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
