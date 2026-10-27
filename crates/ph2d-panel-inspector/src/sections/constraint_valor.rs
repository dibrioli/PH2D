//! **Os ▾ de propriedade do cartão CONSTRAINTS** (Follow Value, `docs/BlenderConstraints/06_W5_resto.md`
//! §3.1): o Follow (o que o alvo tem) e o Drive (o que o dono tem). A lista vem do mundo e muda com o
//! objecto, por isso é um ▾ com rolagem e não uma fileira de segmentos. Irmão do `constraint.rs`
//! (tecto de LOC); o molde é o ▾ do *Play Clip* (`actions_editor::clip_row`).

use super::*;
use ph2d_editor_core::constraint_edits::AjustePropriedade;
use ph2d_editor_core::property_row::{Seccao, paint_label_row};
use ph2d_editor_core::widget::{Dropdown, DropdownOption, paint_dropdown_chip};
use ph2d_i18n::tr;

/// As opções do ▾, uma por posição na lista do instantâneo. `zip` com os ids: uma lista maior que o
/// tecto perde as excedentes em vez de as pintar sem id.
pub(crate) fn opcoes(p: &AjustePropriedade) -> Vec<DropdownOption<usize>> {
    ids::INSP_CONSTRAINT_PROP_OPCAO
        .iter()
        .zip(&p.opcoes)
        .enumerate()
        .map(|(i, (&id, o))| DropdownOption::new(id, i, o.rotulo.clone()))
        .collect()
}

/// O ▾ `j` da linha `k`: o nome à esquerda, o chip com a escolhida (pelo nome, também fora da lista).
/// ⛔ Não tem losango: escolhe um ENDEREÇO, não um valor que se anime. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn propriedade(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    (hit_index, store): (&mut HitIndex, &WidgetStore),
    (x, w, y): (f32, f32, f32),
    (k, j): (usize, usize),
    p: &AjustePropriedade,
    seccao: Seccao,
) -> f32 {
    let Some(&id) = ids::INSP_CONSTRAINT_PROP.get(j).and_then(|t| t.get(k)) else {
        return y;
    };
    let linha = paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ALTURA_DE_CAMPO,
        tr(p.rotulo),
        seccao,
    );
    let aberto = matches!(
        store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let mut dd = Dropdown::new(id, "", opcoes(p))
        .placeholder(p.atual.clone())
        .open(aberto)
        .visual(store.dropdown_visual(id));
    if let Some(i) = p.opcoes.iter().position(|o| o.prop == p.valor) {
        dd.select(i);
    }
    hit_index.register(id, linha.control);
    paint_dropdown_chip(&dd, linha.control, scene, text_system, theme);
    // Um popover de cada vez: as opções partilham a tabela de ids, e o `set` guarda o último.
    if aberto {
        crate::state_popovers::set_pending_constraint_prop(Some(((k, j), linha.control)));
    }
    ph2d_editor_core::widget::paint_decorator_dot(scene, theme, linha.dot);
    y + ph2d_tokens::row_pitch_px()
}
