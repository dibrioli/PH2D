//! ⭐ **A pilha na seleção múltipla** (plano 32,
//! `docs/UI_New_and_Simple/vetor_nova_ui/32_plano_F21_o_misto_completo.md`) — irmão do
//! [`super`]: a ordem da seleção (o primário à frente), o «misto» por camada e as escritas que
//! precisam do primário (o olho, um eixo do deslocamento, a cor pelo seletor).

use ph2d_tool_vector::StyleFields as F;
use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{PaintEntry, PaintKind, Rgba8, VecPathId, VecScene};

/// **A seleção da pilha, com o PRIMÁRIO à frente** — a porta única de *«de quem é a pilha que o
/// painel mostra?»*: o `published` lê o primeiro, e as escritas que comparam com o primário também.
#[must_use]
pub fn ordered(pen: &PenTool) -> Vec<VecPathId> {
    let mut sel = pen.selected_paths().to_vec();
    if let Some(p) = pen.selected()
        && let Some(i) = sel.iter().position(|&id| id == p)
    {
        sel[..=i].rotate_right(1);
    }
    sel
}

/// As pilhas das selecionadas têm ESTRUTURAS diferentes (número de camadas, ou espécie por índice)?
#[must_use]
pub fn layers_differ(scene: &VecScene, sel: &[VecPathId]) -> bool {
    let estrutura = |id: &VecPathId| {
        scene.path(*id).map(|p| {
            p.paints
                .iter()
                .map(|e| matches!(e.kind, PaintKind::Fill(_)))
                .collect::<Vec<_>>()
        })
    };
    let mut todas = sel.iter().filter_map(estrutura);
    let Some(primeira) = todas.next() else {
        return false;
    };
    todas.any(|e| e != primeira)
}

/// Em que campos a camada `i` do primário (`sel[0]`) difere da `i` das outras.
#[must_use]
pub fn row_mixed(
    scene: &VecScene,
    sel: &[VecPathId],
    i: usize,
) -> ph2d_panel_vector::state::RowMixed {
    let camada = |id: &VecPathId| scene.path(*id).and_then(|p| p.paints.get(i));
    let mut m = ph2d_panel_vector::state::RowMixed::default();
    let Some(a) = sel.first().and_then(camada) else {
        return m;
    };
    for b in sel[1..].iter().filter_map(camada) {
        m.color |= a.swatch_color() != b.swatch_color();
        m.width |= a.width() != b.width();
        m.opacity |= a.opacity != b.opacity;
        m.blend |= a.blend != b.blend;
        m.offset |= a.offset != b.offset;
        m.dilate |= a.dilate != b.dilate;
        m.dilate_join |= a.dilate_join != b.dilate_join;
        m.enabled |= a.enabled != b.enabled;
    }
    m
}

/// **O olho da camada `i`**: todas ficam como o OPOSTO do primário. ⛔ Inverter cada uma por si
/// deixava uma seleção mista mista para sempre (a acesa apagava, a apagada acendia).
pub fn toggle(scene: &mut VecScene, sel: &[VecPathId], i: usize) -> bool {
    let Some(alvo) = sel
        .first()
        .and_then(|id| scene.path(*id))
        .and_then(|p| p.paints.get(i))
        .map(|e| !e.enabled)
    else {
        return false;
    };
    super::edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) if e.enabled != alvo => {
            e.enabled = alvo;
            true
        }
        _ => false,
    })
}

/// **Um eixo do deslocamento da camada `i`** — só o eixo que o artista comitou; o outro fica o de
/// CADA forma. ⛔ Ler o outro eixo do primário e escrevê-lo em todas apagava o delas.
pub fn set_offset_axes(
    scene: &mut VecScene,
    sel: &[VecPathId],
    i: usize,
    dx: Option<f64>,
    dy: Option<f64>,
) -> bool {
    super::edit(scene, sel, |p| match p.paints.get_mut(i) {
        Some(e) => {
            let novo = [dx.unwrap_or(e.offset[0]), dy.unwrap_or(e.offset[1])];
            let mudou = novo != e.offset;
            e.offset = novo;
            mudou
        }
        None => false,
    })
}

thread_local! {
    /// O cursor desta escrita no contador de escolhas do seletor partilhado.
    static PICKS_SEEN: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// **A cor que o seletor aberto numa amostra de camada escreve** — a lei do Fill de base (plano 32)
/// levada à pilha: o RGB entra quando o artista ESCOLHE uma cor (ou ela difere da do primário), a
/// alfa só quando difere; cada forma guarda o que o gesto não tocou. Chamar UMA vez por quadro.
///
/// ⛔ Antes a cor do seletor era gravada em todas a CADA quadro em que ele estava aberto: abrir o
/// seletor sobre a camada de uma seleção mista carimbava a cor do primário em todas.
pub fn apply_picker(
    store: &mut ph2d_editor_core::interaction::WidgetStore,
    scene: &mut VecScene,
    sel: &[VecPathId],
) -> bool {
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    let n = store.colour_picks(picker);
    let tocou = PICKS_SEEN.with(|c| c.replace(n)) != n;
    let Some(i) = super::layer_of_picker_target(store) else {
        return false;
    };
    let Some((value, ..)) = store.blender_picker(picker) else {
        return false;
    };
    let Some(prim) = sel
        .first()
        .and_then(|id| scene.path(*id))
        .and_then(|p| p.paints.get(i))
        .map(PaintEntry::swatch_color)
    else {
        return false;
    };
    let mut c = value.rgba;
    if tocou && c[3] == 0 {
        c[3] = u8::MAX;
        store.set_blender_value(
            picker,
            ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
        );
    }
    let nova = Rgba8::new(c[0], c[1], c[2], c[3]);
    let mut campos = F::NONE;
    if tocou || (prim.r, prim.g, prim.b) != (nova.r, nova.g, nova.b) {
        campos |= F::FILL_RGB;
    }
    if prim.a != nova.a {
        campos |= F::FILL_A;
    }
    if campos.is_empty() {
        return false;
    }
    let mut mudou = false;
    for id in sel {
        let Some(cur) = scene
            .path(*id)
            .and_then(|p| p.paints.get(i))
            .map(PaintEntry::swatch_color)
        else {
            continue;
        };
        let m = crate::vector_bridge::pick_merge(cur, nova, campos);
        if m != cur {
            mudou |= super::set_color(scene, &[*id], i, m);
        }
    }
    mudou
}
