//! ⭐ **A TAG do modo `Tag` do NAV AGENT** (plano 30, W6) — o chip, a lista e o aviso da tag apagada.
//! Irmão de [`super::nav`] por tecto de LOC, e por ASSUNTO: esta row pergunta à árvore das tags do
//! projecto, que nenhuma outra da secção conhece.
//!
//! ⛔ **A tag APAGADA não alcança ninguém** (a ponte falha FECHADO), e a row di-lo em WARN — a lei
//! do filtro *Only for tag* da §11 (`physics_tag_row`).

use super::*;
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;

/// A row da tag. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn tag_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    tag: u64,
    seccao: Seccao,
) -> f32 {
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        tr("component.field.nav_agent.13"),
        seccao,
    );
    hit_index.register(ids::INSP_NAV_TAG_PICK, linha.control);
    let aberto = matches!(
        store.get(ids::INSP_NAV_TAG_PICK),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let opcoes = nav_tag_options();
    let viva = opcoes.iter().any(|o| o.value == tag);
    let mut dd = Dropdown::new(ids::INSP_NAV_TAG_PICK, "", opcoes)
        .placeholder(tr("panel.inspector.nav.pick_a_tag_u"))
        .open(aberto)
        .visual(store.dropdown_visual(ids::INSP_NAV_TAG_PICK));
    // ⚠️ **A escolha vem do SNAPSHOT e o `open` do store** (a lei do chip da comparação).
    if tag != 0 {
        dd.select(tag);
    }
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    if aberto {
        crate::state_popovers::set_pending_nav_tag_dd(Some(linha.control));
    }
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_campo(
        scene,
        theme,
        linha.dot,
        &[ids::INSP_NAV_TAG_PICK],
        TextKey::new("component.field.nav_agent.13").into(),
    ) {
        hit_index.register(alvo, linha.dot);
    }
    let mut cur_y = linha.fim(ph2d_tokens::row_pitch_px());
    if tag != 0 && !viva {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.that_tag_was_deleted"),
            ColorToken::Warn,
        );
    }
    cur_y
}

/// **As opções: a árvore inteira do projecto**, indentada pela profundidade.
///
/// ⚠️ `pub(crate)` porque o passe diferido e o despacho a re-derivam — a lei do popover.
pub(crate) fn nav_tag_options() -> Vec<DropdownOption<u64>> {
    opcoes_da_arvore(&ids::INSP_NAV_TAG_OPT)
}

/// A árvore das tags como opções, uma por tag e um id de `opts` por posição (cada lista tem os SEUS
/// ids: o despacho resolve a opção pela posição). A do Nav, do filtro do Ray Sensor, da Action e da
/// Physics.
///
/// ⛔ **Sem tecto** (U5, auditoria GEF 07): a árvore não tem tecto (`9 344` tags medidas na W1), os ids
/// nascem pelo índice e o popover rola. Gates: `a_lista_de_tags_nao_corta_calada`, `a_tag_numero_70_*`.
pub(crate) fn opcoes_da_arvore(opts: &ids::OpcoesDeTag) -> Vec<DropdownOption<u64>> {
    let arvore = crate::state::current_tag_tree();
    arvore
        .iter()
        .zip(opts.ids(arvore.len()))
        .map(|(row, id)| {
            let recuo = "    ".repeat(row.depth);
            DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ph2d_editor_core::screens::hero::InspectorTagRow;

    /// ⛔ **As quatro listas de tags sem busca oferecem a árvore INTEIRA** (U5, auditoria GEF 07):
    /// cada tag é uma opção escolhível, com um id próprio.
    ///
    /// **Mutação que deve sangrar:** um tecto em [`super::opcoes_da_arvore`] (`.take(64)`).
    #[test]
    fn a_lista_de_tags_nao_corta_calada() {
        const N: usize = 70;
        crate::state::set_current_tag_tree(
            (1..=N as u64)
                .map(|i| InspectorTagRow {
                    id: i,
                    path: format!("T{i}"),
                    label: format!("T{i}"),
                    depth: 0,
                })
                .collect(),
        );
        let listas = [
            ("nav", super::nav_tag_options()),
            ("ray", crate::sections::ray_filtro::opcoes_da_tag()),
            ("action", crate::sections::actions::tag_options()),
            ("physics", crate::sections::physics_rows::phys_tag_options()),
        ];
        for (nome, opcoes) in listas {
            let escolhiveis: Vec<u64> = opcoes
                .iter()
                .filter(|o| !o.disabled && o.value != 0)
                .map(|o| o.value)
                .collect();
            assert_eq!(escolhiveis, (1..=N as u64).collect::<Vec<_>>(), "{nome}");
            let ids: std::collections::BTreeSet<_> = opcoes.iter().map(|o| o.id).collect();
            assert_eq!(ids.len(), opcoes.len(), "{nome}: dois ids numa lista");
        }
        crate::state::set_current_tag_tree(Vec::new());
    }
}
