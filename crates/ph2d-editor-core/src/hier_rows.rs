//! ⭐⭐ **O que uma linha da Hierarquia diz além do nome** — o PAPEL dela, os ícones dos
//! componentes que o objecto TEM e o CONTEÚDO que a família dele declara.
//!
//! As leis vivem em [`docs/ObjectsAndComponents/01_objeto_e_componente.md`] §4–§7: OBJECTO é linha,
//! COMPONENTE é ícone na linha + cartão no Inspector, CONTEÚDO é sub-linha recolhida sob o objecto,
//! e a CENA é a raiz. ⛔ Componente nunca vira linha-filha (§10).
//!
//! [`docs/ObjectsAndComponents/01_objeto_e_componente.md`]: https://github.com/dibrioli/PH2D/blob/main/docs/ObjectsAndComponents/01_objeto_e_componente.md

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_component_desc::ComponentCategory;

use crate::icons::IconId;
use crate::object_mode::ObjectMode;

/// **O que a linha É.** Só [`RowRole::Object`] é um objecto da cena: os outros três não se
/// arrastam, não se renomeiam e não abrem o menu de objecto (o painel não os publica como
/// `hierarchy_row_ids`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RowRole {
    #[default]
    Object,
    /// A raiz (§4.6). Seleccionada quando nenhum objecto está.
    Scene,
    /// O cabeçalho de um grupo de conteúdo (*Layers (3)*), recolhido por omissão.
    ContentGroup { owner: NodeId, group: u8 },
    /// Uma peça do conteúdo; clicar abre `mode` (o do grupo) com ela em mãos.
    ContentItem {
        owner: NodeId,
        group: u8,
        key: u64,
        mode: ObjectMode,
    },
}

/// Uma peça do conteúdo — `key` é o id DA FAMÍLIA (camada do Painter, camada do Flip…), opaco aqui.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentItem {
    pub key: u64,
    pub name: String,
    /// Profundidade DENTRO do grupo (grupos de camadas aninham).
    pub depth: u8,
    /// A peça que o módulo tem em mãos agora (a camada activa).
    pub active: bool,
}

/// Um grupo de conteúdo que a família declara para um objecto dela.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentGroup {
    /// A chave i18n do cabeçalho, com `{n}` = quantas peças.
    pub label_key: &'static str,
    pub icon: IconId,
    pub item_icon: IconId,
    /// O modo em que se edita uma peça deste grupo.
    pub mode: ObjectMode,
    pub items: Vec<ContentItem>,
}

/// O conteúdo de cada objecto (por `Entity::to_bits`) — o que as famílias declararam neste quadro.
pub type ContentMap = BTreeMap<u64, Vec<ContentGroup>>;

/// ⭐ **O ícone de uma família de componentes na linha** — `None` = a família não ganha ícone.
///
/// ⚠️ As que ficam de fora dizem o que o objecto É (Identity, Transform, Ordering, Rendering e as
/// de TIPO: Image, Vector, Skeleton) ou já têm o seu selo (Instancing = `PRF`): um ícone que todo
/// objecto mostra não distingue nenhum.
#[must_use]
pub const fn component_icon(category: ComponentCategory) -> Option<IconId> {
    match category {
        ComponentCategory::Animation => Some(IconId::Film),
        ComponentCategory::Anchors => Some(IconId::Pin),
        ComponentCategory::Physics => Some(IconId::Physics),
        ComponentCategory::Logic => Some(IconId::Bolt),
        ComponentCategory::Audio => Some(IconId::Audio),
        ComponentCategory::Camera => Some(IconId::Camera),
        ComponentCategory::Scripting => Some(IconId::Script),
        ComponentCategory::Identity
        | ComponentCategory::Transform
        | ComponentCategory::Ordering
        | ComponentCategory::Rendering
        | ComponentCategory::Image
        | ComponentCategory::Skeleton
        | ComponentCategory::Vector
        | ComponentCategory::Instancing => None,
    }
}

/// As famílias com ícone, uma vez cada, na ordem da paleta ([`ComponentCategory::ALL`]).
#[must_use]
pub fn row_categories(
    present: impl IntoIterator<Item = ComponentCategory>,
) -> Vec<ComponentCategory> {
    let mut seen: Vec<ComponentCategory> = present
        .into_iter()
        .filter(|c| component_icon(*c).is_some())
        .collect();
    seen.sort_unstable();
    seen.dedup();
    ComponentCategory::ALL
        .into_iter()
        .filter(|c| seen.contains(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uma_familia_aparece_uma_vez_e_na_ordem_da_paleta() {
        let got = row_categories([
            ComponentCategory::Audio,
            ComponentCategory::Transform,
            ComponentCategory::Physics,
            ComponentCategory::Audio,
        ]);
        assert_eq!(
            got,
            vec![ComponentCategory::Physics, ComponentCategory::Audio]
        );
    }
}
