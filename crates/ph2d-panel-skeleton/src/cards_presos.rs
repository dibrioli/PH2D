//! ⭐ **T7 — os cartões do Bones com sujeito** (`docs/CartoesUI/02_especificacao.md` §8): publica o
//! osso em foco como sujeito da lista; um cartão solto PRESO a outro osso constrói-se e pinta-se
//! com o retrato desse osso (`state::com_preso`), com os números dele à parte (T40: a lista mostra a
//! mesma secção para o osso em foco); um alfinete cujo osso deixou de existir solta-se sozinho.

use crate::cards::{CARD_BONE, CARD_IK, CARD_LIMIT, CARD_SMART, cards as do_foco};
use crate::state;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::{CardBody, CardDef};

/// As secções que mostram UM osso (as outras são da ferramenta ou da pele).
pub const COM_SUJEITO: [ph2d_a11y::NodeId; 4] = [CARD_BONE, CARD_LIMIT, CARD_SMART, CARD_IK];

/// Os cartões do painel (osso em foco) e os dos soltos presos (o osso deles).
pub(crate) fn cards(store: &mut WidgetStore) -> (Vec<CardDef<'static>>, Vec<CardDef<'static>>) {
    store.set_card_subject(ids::SKELETON_PANEL, state::current_bone_subject());
    for s in COM_SUJEITO {
        store.mark_card_pinnable(s);
    }
    let out = do_foco();
    let mut presos = Vec::new();
    for (sec, solto) in store.loose_cards_of(ids::SKELETON_PANEL) {
        let Some(pin) = solto.pin else {
            continue;
        };
        let Some(sujeito) = state::com_preso(pin, state::current_bone_subject) else {
            continue; // a shell ainda não publicou este osso: um quadro com o do foco
        };
        let Some((_, nome)) = sujeito else {
            store.unpin_loose_card(sec);
            continue;
        };
        store.set_card_subject_name(pin, nome);
        let Some((def, valores)) = state::com_preso(pin, || {
            let d = do_foco().into_iter().find(|c| c.section == solto.section);
            (d, crate::paint::valores(Some(solto.section)))
        }) else {
            continue;
        };
        store.set_pinned_values(sec, valores);
        let def = def.map(|mut d| {
            d.section = sec; // T54: o def é da INSTÂNCIA
            d.body = d.body.map(|f| -> CardBody<'static> {
                Box::new(move |c, y| state::com_preso(pin, || f(c, y)).unwrap_or(y))
            });
            d
        });
        presos.extend(def);
    }
    (out, presos)
}
