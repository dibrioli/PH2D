//! ⭐ **O Inspector em cartões para TODO tipo de objecto** (`docs/CartoesUI/10`, censo A2): os cartões
//! que dependem do que o objecto É (cópia, propriedades, imagem, visibilidade), o estado vazio e o
//! lugar de cada secção de corpo na lista. Os pintores são os de sempre, os ids também.

use crate::cards_componentes::Presente;
use crate::plano::Tela;
use crate::sections;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef};
use ph2d_editor_core::project::DisplayUnit;
use ph2d_editor_core::screens::hero::{InspectorOrderingInfo, InspectorSpriteInfo};
use ph2d_editor_core::widget::section_cards;
use ph2d_editor_core::{InspectorImageUse, screens::hero::InspectorVisibilitySectionInfo};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_tool_registry::hash_node_id;

pub const CARD_IMAGE: NodeId = hash_node_id("cards.insp.image");
pub const CARD_VISIBILITY: NodeId = hash_node_id("cards.insp.visibility");
pub const CARD_INSTANCE: NodeId = hash_node_id("cards.insp.instance");
pub const CARD_PROPERTIES: NodeId = hash_node_id("cards.insp.properties");
pub const CARD_EMPTY: NodeId = hash_node_id("cards.insp.empty");

const ANTES_DA_ORDEM: [NodeId; 1] = [core_ids::INSP_LIVE_SLICE_SECTION];
const DEPOIS_DA_ORDEM: [NodeId; 2] = [
    core_ids::INSP_LIVE_RENDER_SECTION,
    core_ids::INSP_LIVE_SAMPLING_SECTION,
];
/// Logo abaixo da Image: a animação e o Look (ficha da Sprite: «qual imagem?», «como ela parece?»).
const DEPOIS_DA_IMAGEM: [NodeId; 2] = [
    core_ids::INSP_LIVE_ANIM_SECTION,
    core_ids::INSP_LIVE_COLOR_SECTION,
];

/// A secção Animation de uma sprite tem cartão só no uso Animated, e só na lista de cartões: no
/// painel de antes ela abre dentro da linha Image.
pub(crate) fn animacao_em_cartao(s: &InspectorSpriteInfo) -> bool {
    crate::cards::active() && s.image_use == InspectorImageUse::Animated
}

/// ⭐ Os cartões logo abaixo do objecto, pela ordem de leitura do painel de antes: de quem é cópia, o
/// que declara, a imagem e o uso dela, os controlos de visibilidade.
pub(crate) fn do_topo() -> Vec<CardDef<'static>> {
    let mut out = Vec::new();
    if let Some(info) = crate::state::current_inspector_instance() {
        out.push(sem_titulo(CARD_INSTANCE, move |c, y| {
            com_corpo(c, y, |t, theme, store, x, w, y| {
                sections::instance_cartao::paint_instance(
                    t.scene, t.text, theme, t.hit, store, &info, x, w, y,
                )
            })
        }));
    }
    if let Some(info) = crate::state::current_inspector_properties() {
        out.push(sem_titulo(CARD_PROPERTIES, move |c, y| {
            com_corpo(c, y, |t, theme, store, x, w, y| {
                sections::instance_cartao::paint_properties(
                    t.scene, t.text, theme, t.hit, store, &info, x, w, y,
                )
            })
        }));
    }
    if let Some(info) = crate::state::current_inspector_sprite() {
        out.push(crate::cards_sprite::imagem(info));
    }
    if let Some(info) = crate::state::current_inspector_visibility_section() {
        out.push(visibilidade(info));
    }
    out
}

fn sem_titulo(
    section: NodeId,
    corpo: impl Fn(&mut CardCtx<'_>, f32) -> f32 + 'static,
) -> CardDef<'static> {
    CardDef {
        section,
        title: "",
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(corpo)),
    }
}

/// Um corpo de hoje dentro do cartão, sem a moldura de secção.
pub(crate) fn com_corpo(
    c: &mut CardCtx<'_>,
    y: f32,
    f: impl FnOnce(&mut Tela<'_>, Theme, &WidgetStore, f32, f32, f32) -> f32,
) -> f32 {
    ph2d_editor_core::panel::cards::compacta::fluir(c, |c| {
        let (store, theme, x, w) = (c.store, c.ink.theme, c.x, c.w);
        let (scene, text, hit) = (&mut *c.scene, &mut *c.text_system, &mut *c.hit_index);
        section_cards::with_section_cards(scene, theme, y, |s| {
            f(
                &mut Tela {
                    scene: s,
                    text,
                    hit,
                },
                theme,
                store,
                x,
                w,
                y,
            )
        })
    })
}

/// O tamanho no mundo, na unidade do projecto; a unidade vai no rótulo.
pub(crate) fn tamanho(info: &InspectorSpriteInfo) -> String {
    let unit = crate::state::current_display_unit();
    let ppm = crate::state::current_pixels_per_meter();
    let (w, h) = (
        unit.from_meters(info.world_size[0], ppm),
        unit.from_meters(info.world_size[1], ppm),
    );
    match unit {
        DisplayUnit::Meters => format!("{w:.3} × {h:.3}"),
        DisplayUnit::Pixels => format!("{w:.0} × {h:.0}"),
    }
}

fn visibilidade(info: InspectorVisibilitySectionInfo) -> CardDef<'static> {
    CardDef {
        section: CARD_VISIBILITY,
        title: tr("component.visibility.name"),
        actions: Vec::new(),
        add: None,
        summary: None,
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            com_corpo(c, y, |t, theme, store, x, w, y| {
                sections::paint_visibility_section(
                    t.scene, t.text, theme, t.hit, store, x, w, y, &info,
                )
            })
        })),
    }
}

/// As linhas do Ordering depois do Z index.
pub(crate) fn resto_da_ordem(c: &mut CardCtx<'_>, y: f32, info: &InspectorOrderingInfo) -> f32 {
    let y = com_corpo(c, y, |t, theme, store, x, w, y| {
        sections::ordering::paint_ordering_rest(t.scene, t.text, theme, t.hit, store, x, w, y, info)
    });
    let opcoes = sections::ordering::opcoes_por_revelar(info);
    if opcoes.is_empty() {
        return y;
    }
    let y = y + ph2d_card_stack::metrics::FIELD_ROW_GAP;
    let segs: Vec<ph2d_editor_core::panel::cards::Seg<'_>> = opcoes
        .iter()
        .map(|(id, text)| ph2d_editor_core::panel::cards::Seg {
            id: *id,
            icon: None,
            text,
            on: false,
        })
        .collect();
    c.row(
        &[ph2d_editor_core::panel::cards::Cell::new(
            ph2d_editor_core::panel::cards::Widget::Dropdown {
                id: sections::ordering::DD_OPCOES,
                text: tr("card.insp.order_option"),
                options: &segs,
            },
        )],
        y,
    )
}

/// «A editar 1 de N»: um facto da seleção, por baixo do nome (`paint_cards::paint_selection_card`).
pub(crate) fn frase_da_selecao(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let n = crate::state::current_inspector_selecionados();
    if n <= 1 {
        return y;
    }
    let frase = tr_with("panel.inspector.selection.primary_only", &[("n", &n)]);
    com_corpo(c, y, |t, theme, _, x, w, y| {
        sections::rows::aviso(t.scene, t.text, theme, x, w, y, &frase, ColorToken::Warn)
    })
}

/// A frase do painel sem objecto: a lista não fica muda.
pub(crate) fn vazio(tem_selecao: bool) -> CardDef<'static> {
    let frase = if tem_selecao {
        tr("panel.inspector.panel.no_properties_yet_for_the")
    } else {
        tr("panel.inspector.panel.select_an_entity_in_the")
    };
    sem_titulo(CARD_EMPTY, move |c, y| c.hint(frase, y))
}

/// ⭐ **O lugar de cada secção de corpo na lista**: a Animation e o Look logo abaixo da Image, o 9-Slice antes
/// do Ordering, Advanced e Sampling depois dele; os componentes no fim, pela ordem da paleta.
pub(crate) fn tece<'a>(
    base: Vec<CardDef<'static>>,
    presentes: &'a [Presente],
    cartao: impl Fn(&'a Presente) -> CardDef<'a>,
) -> Vec<CardDef<'a>> {
    let tem = |s: NodeId| base.iter().any(|c| c.section == s);
    let (com_imagem, com_ordem) = (tem(CARD_IMAGE), tem(crate::cards::CARD_ORDERING));
    let mut out: Vec<CardDef<'a>> = Vec::with_capacity(base.len() + presentes.len());
    let poe = |ids: &[NodeId], out: &mut Vec<CardDef<'a>>| {
        out.extend(
            ids.iter()
                .filter_map(|id| presentes.iter().find(|p| p.id == *id))
                .map(&cartao),
        );
    };
    for card in base {
        let s = card.section;
        if s == crate::cards::CARD_ORDERING {
            poe(&ANTES_DA_ORDEM, &mut out);
        }
        out.push(card);
        if s == CARD_IMAGE {
            poe(&DEPOIS_DA_IMAGEM, &mut out);
        } else if s == crate::cards::CARD_ORDERING {
            poe(&DEPOIS_DA_ORDEM, &mut out);
        }
    }
    let ja_posto = |id: NodeId| {
        (com_imagem && DEPOIS_DA_IMAGEM.contains(&id))
            || (com_ordem && (ANTES_DA_ORDEM.contains(&id) || DEPOIS_DA_ORDEM.contains(&id)))
    };
    out.extend(presentes.iter().filter(|p| !ja_posto(p.id)).map(&cartao));
    out
}
