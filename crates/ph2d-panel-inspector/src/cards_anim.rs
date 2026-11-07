//! ⭐ **O cartão Animation** da [ficha](../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md): os
//! verbos no cabeçalho, a prévia, a lista, o Playback, Frames/s e Speed numa linha, Cycle, More e Who Picks.
//! A folha e o resto do corte moram no editor. Os ids são os de sempre; `PH2D_CARDS=0` pinta o painel de antes.

use crate::cards_componentes::{Abertas, Presente};
use crate::cards_objeto::com_corpo;
use crate::ids;
use crate::sections;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{
    CardCtx, CardDef, Cell as Celula, HeadAction, Seg, Widget, dropdown,
};
use ph2d_editor_core::screens::hero::{AnimChooser, AnimFieldEdit, InspectorAnimInfo};
use ph2d_editor_core::widget::TextInputState;
use ph2d_i18n::tr;
use std::cell::Cell;

/// O cartão próprio de um objecto com tocador; sem tocador fica o corpo de sempre (o botão que o anexa).
pub(crate) fn proprio<'a>(
    p: &'a Presente,
    actions: Vec<HeadAction>,
    abertas: &'a Cell<Abertas>,
) -> Option<CardDef<'a>> {
    if p.id != core_ids::INSP_LIVE_ANIM_SECTION {
        return None;
    }
    let info = crate::state::current_inspector_anim().filter(|i| i.player_present)?;
    let verbo = |id, icon| HeadAction {
        id,
        icon,
        on: false,
    };
    let mut cabecalho = vec![verbo(ids::INSP_ANIM_ADD, IconId::Plus)];
    if !info.rows.is_empty() {
        cabecalho.push(verbo(ids::INSP_ANIM_REMOVE, IconId::Minus));
    }
    cabecalho.push(verbo(ids::INSP_ANIM_OPEN_EDITOR, IconId::Film));
    // O piso do painel não leva tudo: cede o ↺, depois a contagem. O nome nunca se corta.
    // A barra de rolar, quando há, come o mesmo que a margem da lista (8 px).
    let livre = p.largura - (m::LIST_PAD_X + m::CARD_PAD_X) * 2.0 - m::LIST_PAD_X;
    let sobra = |n: usize| livre - m::HEAD_ICON_BOX * n as f32;
    if p.titulo_w <= sobra(cabecalho.len() + actions.len()) {
        cabecalho.extend(actions);
    }
    let cabe = p.titulo_w <= sobra(cabecalho.len());
    Some(CardDef {
        section: p.id,
        title: if cabe {
            &p.titulo
        } else {
            tr("component.field.sprite_animator.0")
        },
        actions: cabecalho,
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            corpo(c, y, &info, abertas.get())
        })),
    })
}

fn corpo(c: &mut CardCtx<'_>, y: f32, info: &InspectorAnimInfo, a: Abertas) -> f32 {
    let selected = a.anim().min(info.rows.len().saturating_sub(1));
    let mut campo_do_nome = None;
    let y = com_corpo(c, y, |t, theme, store, x, w, y| {
        let y = sections::anim_card::paint_preview(t.scene, x, w, y, info);
        sections::anim_card::paint_list(
            t.scene,
            t.text,
            theme,
            t.hit,
            store,
            x,
            w,
            y,
            info,
            selected,
            // Só no cartão do objecto do renomear: o mesmo id pinta em todo cartão Animation à vista.
            a.renomeando()
                .filter(|(dono, _)| *dono == info.entity_bits)
                .map(|(_, i)| (i, &mut campo_do_nome)),
        )
    });
    // Por cima da linha já registada: o clique no campo é do campo.
    if let Some(r) = campo_do_nome {
        c.text_box(r, ids::INSP_ANIM_NAME, "");
    }
    let y = com_corpo(c, y, |t, theme, store, x, w, y| {
        sections::anim_cartao::resto(t, theme, store, x, w, y, info, selected)
    });
    if info.card.chooser == AnimChooser::Me && !info.rows.is_empty() {
        return comeca_com(c, y, info);
    }
    y
}

/// O menu «Starts with» do «Auto»: as opções são as animações da lista.
fn comeca_com(c: &mut CardCtx<'_>, y: f32, info: &InspectorAnimInfo) -> f32 {
    let linhas: Vec<Seg<'_>> = info
        .rows
        .iter()
        .zip(ids::INSP_ANIM_START_OPT)
        .map(|(row, id)| Seg {
            id,
            icon: None,
            text: &row.name,
            on: row.name == info.card.starts_with,
        })
        .collect();
    c.row(
        &[Celula::new(Widget::Dropdown {
            id: ids::INSP_ANIM_START_DD,
            text: &info.card.starts_with,
            options: &linhas,
        })
        .label(tr("panel.inspector.anim_card.starts_with"))],
        y + m::FIELD_ROW_GAP,
    )
}

/// Uma vez por quadro, antes da pintura.
pub(crate) fn prepara() {
    dropdown::declare(ids::INSP_ANIM_START_DD);
}

/// Abre o campo do nome sobre a linha (o gesto do editor e da árvore de tags).
fn abre_o_nome(store: &mut WidgetStore, nome: &str) {
    store.register(
        ids::INSP_ANIM_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: nome.to_owned(),
            caret: nome.len(),
            selection_anchor: None,
        },
    );
    store.set_focus(Some(ids::INSP_ANIM_NAME));
    store.mark_cancel_on_escape(ids::INSP_ANIM_NAME);
}

/// Dois cliques na linha renomeiam; Enter grava, Esc desiste. `true` = consumido.
pub(crate) fn evento_do_nome(
    panel: &mut crate::state::InspectorState,
    host: &mut dyn PanelHostInternal,
    info: &InspectorAnimInfo,
    ev: WidgetEvent,
) -> bool {
    match ev {
        WidgetEvent::DoubleClick(id) if crate::cards::active() => {
            let Some(i) = ids::INSP_ANIM_ROW.iter().position(|&o| o == id) else {
                return false;
            };
            let Some(row) = info.rows.get(i) else {
                return false;
            };
            panel.anim_selected = i;
            panel.anim_renomeando = Some(crate::state::AnimRenomeando {
                entity_bits: info.entity_bits,
                row: i,
                nome: row.name.clone(),
            });
            abre_o_nome(host.store_mut(), &row.name);
            true
        }
        WidgetEvent::Submit(id) if id == ids::INSP_ANIM_NAME => {
            grava_o_nome(panel, host);
            host.store_mut().set_focus(None);
            true
        }
        WidgetEvent::Cancel(id) if id == ids::INSP_ANIM_NAME => {
            panel.anim_renomeando = None;
            true
        }
        // Enquanto se digita nada se grava: «wa» a meio de «walk» não é uma animação.
        WidgetEvent::TextChanged(id) if id == ids::INSP_ANIM_NAME => {
            panel.anim_renomeando.is_some()
        }
        _ => false,
    }
}

fn grava_o_nome(panel: &mut crate::state::InspectorState, host: &mut dyn PanelHostInternal) {
    let Some(r) = panel.anim_renomeando.take() else {
        return;
    };
    let novo = host.store().text(ids::INSP_ANIM_NAME).unwrap_or("").trim();
    if novo.is_empty() || r.nome == novo {
        return;
    }
    let edit = AnimFieldEdit::Rename(u8::try_from(r.row).unwrap_or(u8::MAX), novo.to_owned());
    crate::event_anim::push(host, r.entity_bits, edit);
}

/// Clicar fora do campo grava, como no editor: o foco saiu e o campo não pode ficar aberto. Vale com a seleção em
/// qualquer objecto, ou sem tocador: quem manda é o dono do renomear.
pub(crate) fn fecha_ao_perder_o_foco(
    panel: &mut crate::state::InspectorState,
    host: &mut dyn PanelHostInternal,
) {
    if panel.anim_renomeando.is_some() && host.store().focus_id() != Some(ids::INSP_ANIM_NAME) {
        grava_o_nome(panel, host);
    }
}
