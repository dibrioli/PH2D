//! ⭐ **O MENU ADD DE OBJECTOS** — o modal que o `+` da Hierarquia e o `Shift+A` abrem
//! (`docs/UI_New_and_Simple/spec/06_tipos_e_modos_de_objeto.md`, F1).
//!
//! ⛔ **Não é um modal novo:** é o [`crate::widget::command_palette`]. Este módulo só constrói o
//! MODELO e devolve o pick.
//!
//! ⭐ **Cada família declara as suas entradas na própria crate** (`&[AddEntry]`), e a COMPOSIÇÃO
//! junta as que estão compiladas: um tipo cuja crate saiu do build não deixa um item morto.
//! O pick volta como a [`AddEntry`], e quem a sabe criar é a família dona — com os recursos dela.

use crate::widget::command_palette::{PaletteGroup, PaletteItem, PaletteModel, PaletteSub};
use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_tokens::ColorToken;

/// **O grupo do menu** — por natureza do objecto, como o *Add* do Blender agrupa por tipo de dado.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AddGroup {
    /// Imagem, Flip, Vetor.
    TwoD,
    /// Um vazio com o componente de jogo já posto (só modo Object).
    Game,
    /// O objecto vazio.
    Empty,
}

impl AddGroup {
    /// Todos, na ordem em que o modal os mostra.
    pub const ALL: [AddGroup; 3] = [AddGroup::TwoD, AddGroup::Game, AddGroup::Empty];

    /// O título do grupo.
    #[must_use]
    pub fn title(self) -> &'static str {
        ph2d_i18n::tr(match self {
            AddGroup::TwoD => "object_add.group.two_d",
            AddGroup::Game => "object_add.group.game",
            AddGroup::Empty => "object_add.group.empty",
        })
    }

    /// A cor do grupo — das sete `NodeCat*` que o modal já pinta.
    #[must_use]
    pub const fn color(self) -> ColorToken {
        match self {
            AddGroup::TwoD => ColorToken::NodeCatSource,
            AddGroup::Game => ColorToken::NodeCatOutput,
            AddGroup::Empty => ColorToken::NodeCatUtility,
        }
    }
}

/// **Uma entrada do menu**: o rótulo (que é também a IDENTIDADE) e o grupo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AddEntry {
    /// A chave i18n do rótulo. O id do item é o hash dela — sobrevive a inserções no meio.
    pub key: TextKey,
    /// Onde o modal a mostra.
    pub group: AddGroup,
}

impl AddEntry {
    /// Uma entrada.
    #[must_use]
    pub const fn new(key: &'static str, group: AddGroup) -> Self {
        Self {
            key: TextKey::new(key),
            group,
        }
    }

    /// O id de item desta entrada no modal.
    #[must_use]
    pub const fn id(self) -> NodeId {
        ph2d_tool_registry::hash_node_id(self.key.key())
    }
}

/// **As entradas da fundação**: o vazio e a imagem (que abre o diálogo de tamanho do `Ctrl+N`).
pub const EMPTY: AddEntry = AddEntry::new("object_add.empty", AddGroup::Empty);
/// Ver [`EMPTY`].
pub const IMAGE: AddEntry = AddEntry::new("object_add.image", AddGroup::TwoD);
/// Ver [`EMPTY`].
pub const CORE: &[AddEntry] = &[IMAGE, EMPTY];

/// ⭐ **O modelo do modal**, sobre as famílias compiladas.
///
/// Cada fatia é um AGRUPAMENTO dentro do seu grupo (uma família = uma coluna de itens); um grupo
/// sem entrada nenhuma não aparece.
///
/// ⚠️ `why_not` é a razão pela qual uma entrada **não pode nascer agora** (`None` = pode). Ela
/// APARECE na mesma, a seguir às que podem, com a razão no rótulo — a lei da paleta de formas: o
/// artista sabe que o tipo existe, e o clique responde com a mesma frase.
#[must_use]
pub fn build(
    families: &[&[AddEntry]],
    why_not: &dyn Fn(AddEntry) -> Option<&'static str>,
) -> PaletteModel {
    let groups = AddGroup::ALL
        .into_iter()
        .filter_map(|group| {
            let subs: Vec<PaletteSub> = families
                .iter()
                .flat_map(|family| {
                    let mut ready = Vec::new();
                    let mut blocked = Vec::new();
                    for e in family.iter().filter(|e| e.group == group) {
                        let label = e.key.tr();
                        match why_not(*e) {
                            None => ready.push(PaletteItem {
                                label: label.to_string(),
                                id: e.id(),
                            }),
                            Some(reason) => blocked.push(PaletteItem {
                                label: format!("{label}  \u{2014}  {reason}"),
                                id: e.id(),
                            }),
                        }
                    }
                    [ready, blocked]
                        .into_iter()
                        .filter(|items| !items.is_empty())
                        .map(|items| PaletteSub { title: None, items })
                })
                .collect();
            (!subs.is_empty()).then(|| PaletteGroup {
                title: group.title().to_string(),
                color: group.color(),
                subs,
            })
        })
        .collect();
    PaletteModel {
        title: ph2d_i18n::tr("object_add.title").to_string(),
        groups,
        toggle: None,
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

#[cfg(test)]
#[path = "object_add_tests.rs"]
mod tests;
