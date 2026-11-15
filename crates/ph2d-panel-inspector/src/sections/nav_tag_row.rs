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
    opcoes_da_arvore(&ids::INSP_NAV_TAG_OPT, ids::INSP_NAV_TAG_MORE)
}

/// A árvore das tags como opções, uma por id de `opts` (cada lista tem os SEUS ids: o despacho
/// resolve a opção por `position()`). A do Nav, do filtro do Ray Sensor, da Action e da Physics.
///
/// ⛔ **O que não coube é DITO** (U5, auditoria GEF 07): a árvore não tem tecto (`9 344` tags medidas
/// na W1) e os ids têm `opts.len()` — o tecto MEDIDO de um ecrã cheio (a tabela do
/// `a_lista_de_tags_cobre_um_ecra_cheio`, `64`: o recurso é o POPOVER, não o modelo). A sobra vira a
/// última linha, apagada, com o id `mais`: «…and N more». *Um chooser que corta em silêncio ensina que
/// a tag não existe.* Gate: `a_lista_de_tags_nao_corta_calada`.
pub(crate) fn opcoes_da_arvore(
    opts: &[ph2d_a11y::NodeId],
    mais: ph2d_a11y::NodeId,
) -> Vec<DropdownOption<u64>> {
    let arvore = crate::state::current_tag_tree();
    let mut out: Vec<DropdownOption<u64>> = arvore
        .iter()
        .zip(opts.iter())
        .map(|(row, &id)| {
            let recuo = "    ".repeat(row.depth);
            DropdownOption::new(id, row.id, format!("{recuo}{}", row.label))
        })
        .collect();
    let sobra = arvore.len().saturating_sub(opts.len());
    if sobra > 0 {
        let mut linha = DropdownOption::new(
            mais,
            0u64,
            ph2d_i18n::tr_with("panel.inspector.tags.and_n_more", &[("n", &sobra)]),
        );
        linha.disabled = true;
        out.push(linha);
    }
    out
}

#[cfg(test)]
mod tests {
    use ph2d_editor_core::screens::hero::InspectorTagRow;

    /// ⛔ **As quatro listas de tags sem busca nunca cortam caladas** (U5, auditoria GEF 07): com
    /// mais tags na árvore que ids, ou cada tag é alcançável, ou a última linha DIZ quantas ficaram.
    ///
    /// **Mutação que deve sangrar:** tirar a linha «…and N more» de [`super::opcoes_da_arvore`].
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
            let faltam = N - escolhiveis.len();
            if faltam == 0 {
                continue;
            }
            let ultima = opcoes.last().expect("a lista tem linhas");
            assert!(
                ultima.disabled && ultima.label.contains(&faltam.to_string()),
                "a lista de tags do {nome} mostra {} de {N} e corta {faltam} calada \
                 (última linha {:?})",
                escolhiveis.len(),
                ultima.label
            );
            let ids: std::collections::BTreeSet<_> = opcoes.iter().map(|o| o.id).collect();
            assert_eq!(ids.len(), opcoes.len(), "{nome}: dois ids numa lista");
        }
        crate::state::set_current_tag_tree(Vec::new());
    }
}
