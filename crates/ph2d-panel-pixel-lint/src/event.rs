//! Os gestos: a caixa de cada regra alterna-a, o Color limit muda o limite, clicar num grupo ou num
//! aviso escolhe-o (marca + foco).

use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_pixel_lint::{Regra, Regras};

use crate::{PixelLintPanel, canal, ids};

/// O teto do Color limit: um índice de paleta é um `u8` (256 cores).
const LIMITE_MAX: f64 = 256.0;

fn botao(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    store.mark_collapsible_section(ids::SEC_RULES);
    store.mark_collapsible_section(ids::SEC_ISSUES);
    for r in Regra::TODAS {
        botao(store, ids::regra(r));
        botao(store, ids::grupo(r));
        let dica = format!("pixel.lint.rule.{}.tip", ids::nome(r));
        store.set_tooltip(ids::regra(r), ph2d_i18n::tr(&dica));
    }
    store.register(
        ids::COLOR_LIMIT,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: String::new(),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
    store.mark_card_field(
        ids::COLOR_LIMIT,
        CardFieldSpec {
            step: 1.0,
            scrub: 1.0,
            default: None,
        },
    );
    store.set_number_drag_rate(ids::COLOR_LIMIT, 1.0);
    store.set_number_range(ids::COLOR_LIMIT, 1.0, LIMITE_MAX, 1.0);
    store.register(
        ids::LINT_DRAG,
        InteractiveState::BlenderHit {
            parent: PixelLintPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// As linhas dos avisos: a corrida decide quantas são.
pub(crate) fn regista_avisos(store: &mut WidgetStore, v: &canal::LintView) {
    for g in &v.grupos {
        for i in 0..g.avisos.len() {
            botao(store, ids::aviso(g.regra, i));
        }
    }
    if store.focus_id() != Some(ids::COLOR_LIMIT) {
        #[expect(clippy::cast_precision_loss, reason = "≤ 256")]
        store.set_number_value(ids::COLOR_LIMIT, canal::regras().limite_cores as f64);
    }
}

fn alterna(r: &mut Regras, regra: Regra) {
    let b = match regra {
        Regra::Orfao => &mut r.orfao,
        Regra::CantoDuplo => &mut r.canto_duplo,
        Regra::Degrau => &mut r.degrau,
        Regra::Faixa => &mut r.faixa,
        Regra::Almofada => &mut r.almofada,
        Regra::CoresAMais => &mut r.cores_a_mais,
    };
    *b = !*b;
}

pub(crate) fn apply_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> EventOutcome {
    match ev {
        WidgetEvent::ValueChanged(id) if id == ids::COLOR_LIMIT => {
            let Some(n) = host.store().number_value(id).filter(|n| n.is_finite()) else {
                return EventOutcome::Ignored;
            };
            let mut r = canal::regras();
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "1..=256"
            )]
            {
                r.limite_cores = n.clamp(1.0, LIMITE_MAX) as usize; // CLAMP-OK: o campo já prende
            }
            canal::poe_regras(r);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) => {
            if let Some(regra) = Regra::TODAS.into_iter().find(|&r| ids::regra(r) == id) {
                let mut r = canal::regras();
                alterna(&mut r, regra);
                canal::poe_regras(r);
                return EventOutcome::Consumed;
            }
            if let Some(regra) = Regra::TODAS.into_iter().find(|&r| ids::grupo(r) == id) {
                canal::escolhe(regra, 0);
                return EventOutcome::Consumed;
            }
            let v = canal::view();
            for g in &v.grupos {
                if let Some(i) = (0..g.avisos.len()).find(|&i| ids::aviso(g.regra, i) == id) {
                    canal::escolhe(g.regra, i);
                    return EventOutcome::Consumed;
                }
            }
            EventOutcome::Ignored
        }
        _ => EventOutcome::Ignored,
    }
}
