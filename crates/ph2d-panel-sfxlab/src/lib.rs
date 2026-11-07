//! **Os cartões do SFX Lab** (`docs/SfxLab/04_W3_plano.md`): o painel entra como ABA ao lado do Inspector enquanto a aba
//! do som está ativa, e nunca o esconde. Lê o instantâneo que a família publica e devolve intenções numa fila: quem edita o som é ela.
//! A tabela [`tabela::CAMPOS`] gera o `populate`, o `prepare`, os cartões e o despacho.

#![forbid(unsafe_code)]

mod cards;
mod estado;
pub mod ids;
pub mod medidor;
mod nomes;

pub use estado::{Instantaneo, Intencao, Material, campo_sob_o_mouse, publica, tomar};
pub use nomes::nome_da_camada;
pub use ph2d_sfx_lab::tabela;

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{
    EventOutcome, PaintCtx, Panel, PanelHostInternal, TextKey, seam_reset_button,
};
use ph2d_editor_core::screens::slot::{Slot, SlotSet};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

/// O id do painel, na forma que o censo de `a_layout_never_commands_a_panel_a_bridge_owns` lê.
pub const PANEL_ID: &str = "sfxlab";

#[derive(Clone, Debug, Default)]
pub struct SfxLabPanelState;

pub struct SfxLabPanel;

impl Panel for SfxLabPanel {
    type State = SfxLabPanelState;

    const ID: &'static str = PANEL_ID;
    const NODE_ID: NodeId = ph2d_editor_core::ids::SFX_PANEL;
    const DEFAULT_VISIBLE: bool = false;
    const TITLE: TextKey = TextKey::new("panel.sfxlab.title");
    const ICON: ph2d_editor_core::icons::IconId = ph2d_editor_core::icons::IconId::Bolt;
    const ALLOWED_SLOTS: SlotSet = SlotSet::SIDES;
    const DEFAULT_SLOT: Slot = Slot::RightTop;

    fn paint(_state: &mut SfxLabPanelState, ctx: &mut PaintCtx) {
        if !ctx.host.panel_visible(Self::ID) {
            ctx.host.store_mut().clear_panel_rect(Self::NODE_ID);
            return;
        }
        let rect = ctx.slot;
        ctx.host.store_mut().set_panel_rect(Self::NODE_ID, rect);
        estado::poe_sob_o_mouse(sob_o_mouse(ctx.host.store()));
        let snap = estado::atual();
        cards::prepare(ctx.host.store_mut(), snap.as_deref());
        let cartoes = cards::cards(snap);
        ph2d_editor_core::panel::cards::paint_card_list(
            ctx,
            Self::NODE_ID,
            Self::TITLE.tr(),
            &cartoes,
        );
    }

    fn apply_event(
        _state: &mut SfxLabPanelState,
        host: &mut dyn PanelHostInternal,
        ev: WidgetEvent,
    ) -> EventOutcome {
        EventOutcome::from_bool(despacha(host, ev))
    }

    fn populate(store: &mut WidgetStore) {
        for c in tabela::CAMPOS {
            store.register(
                c.id,
                InteractiveState::NumberInput {
                    state: TextInputState::Normal,
                    value: 0.0,
                    buffer: "0".to_string(),
                    caret: 0,
                    last_committed: 0.0,
                    selection_anchor: None,
                },
            );
            store.set_number_range(c.id, c.faixa.0, c.faixa.1, c.passo);
        }
        let botoes = tabela::OPCOES
            .iter()
            .map(|o| o.id)
            .chain([ids::FORMA_HIT, ids::MATERIAL]);
        for id in botoes {
            registra_botao(store, id);
        }
    }
}

/// Só `Hovered`: um botão focado pelo teclado também acende, mas a mão não está nele.
fn sob_o_mouse(store: &WidgetStore) -> Option<NodeId> {
    let campo = tabela::CAMPOS.iter().map(|c| c.id).find(|id| {
        matches!(
            store.get(*id),
            Some(InteractiveState::NumberInput {
                state: TextInputState::Hovered,
                ..
            })
        )
    });
    campo.or_else(|| {
        tabela::OPCOES.iter().map(|o| o.id).find(|id| {
            matches!(
                store.get(*id),
                Some(InteractiveState::Button {
                    state: ButtonState::Hovered
                })
            )
        })
    })
}

pub(crate) fn registra_botao(store: &mut WidgetStore, id: NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

fn despacha(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    match ev {
        WidgetEvent::ValueChanged(id) if tabela::campo(id).is_some() => {
            if let Some(v) = host.store().number_value(id) {
                estado::pede(Intencao::Valor(id, v));
            }
            true
        }
        WidgetEvent::Click(id) if tabela::opcao(id).is_some() => {
            seam_reset_button(host, id);
            estado::pede(Intencao::Opcao(id));
            true
        }
        WidgetEvent::Click(id) if id == ids::FORMA_HIT => {
            seam_reset_button(host, id);
            estado::pede(Intencao::FormaHit);
            true
        }
        WidgetEvent::Click(id) => match estado::material_de(id) {
            Some(take) => {
                seam_reset_button(host, id);
                estado::pede(Intencao::Material(take));
                true
            }
            None => false,
        },
        _ => false,
    }
}
