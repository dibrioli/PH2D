//! **A lista «Shapes» do cartão Shape de um corpo** (W3-B, `docs/GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md`):
//! VISTA das peças filhas — a linha 0 é a forma do corpo, depois cada peça na ordem da árvore. Clicar
//! seleciona, o «×» apaga a peça, «+ Add shape» nasce uma peça nova. ⛔ Nenhum segundo armazém.

use super::physics_cartao::Tinta;
use super::*;
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};
use ph2d_i18n::tr;

/// O texto de uma linha: o nome e o tipo, ou só o tipo quando a forma não tem nome.
#[must_use]
pub fn rotulo(nome: &str, tag: u8) -> String {
    let tipo = super::physics::SHAPE_LABELS
        .get(usize::from(tag))
        .map_or("", |k| k.tr());
    if nome.is_empty() {
        tipo.to_string()
    } else {
        format!("{nome} \u{b7} {tipo}")
    }
}

/// As linhas (só com duas formas ou mais) e o «+ Add shape». Devolve o `y` de baixo.
pub(super) fn lista(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let mut y = y;
    if info.shapes.len() >= 2 {
        y = t.frase(y, tr("card.insp.phys.shapes"), ColorToken::Text3);
        let gap = ph2d_tokens::control_gap_px();
        let linhas = info.shapes.iter().take(ids::MAX_SHAPE_ROWS).enumerate();
        for (i, (bits, nome, tag)) in linhas {
            let (id, texto) = (ids::INSP_PHYS_SHAPE_ROW[i], rotulo(nome, *tag));
            let mut rect = caixa_do_botao(t.text, t.x, t.w, y, &texto);
            let lado = rect.h;
            rect.w = rect.w.min(t.x + t.w - lado - gap - rect.x);
            let tipo = if *bits == info.entity_bits {
                ButtonKind::Accent
            } else {
                ButtonKind::Default
            };
            let btn = Button::new(id, &texto)
                .kind(tipo)
                .visual(t.store.button_visual(id));
            paint_button(&btn, rect, t.scene, t.text, t.theme);
            t.hit.register(id, rect);
            if i > 0 {
                super::anim::icon_button(
                    t.scene,
                    t.text,
                    t.theme,
                    t.hit,
                    t.store,
                    Rect::new(t.x + t.w - lado, rect.y, lado, lado),
                    ids::INSP_PHYS_SHAPE_DEL[i],
                    IconId::Close,
                    tr("card.insp.phys.remove_part"),
                );
            }
            y = abaixo_do_botao(rect);
        }
    }
    let (r, y) = t.botao(ids::INSP_PHYS_ADD_PART, tr("card.insp.phys.add_part"), y);
    t.hit.register(ids::INSP_PHYS_ADD_PART, r);
    y
}

/// A edição de um clique na lista: «+ Add shape» num corpo, ou o «×» de uma peça (a linha 0 é o corpo).
pub(crate) fn clique(id: NodeId, info: &InspectorPhysicsInfo) -> Option<PhysicsFieldEdit> {
    if id == ids::INSP_PHYS_ADD_PART {
        return info.has_body.then_some(PhysicsFieldEdit::AddPart);
    }
    let i = ids::INSP_PHYS_SHAPE_DEL.iter().position(|&o| o == id)?;
    (i > 0)
        .then(|| info.shapes.get(i))
        .flatten()
        .map(|&(bits, ..)| PhysicsFieldEdit::RemovePart(bits))
}
