//! ⭐ **T36/T44/T61 — os cartões soltos do Inspector presos a um objecto**
//! (`docs/CartoesUI/01_decisoes_do_dono.md`): os cartões do objecto ([`DO_OBJECTO`]) e TODO cartão de
//! componente, por uma lei.
//! A shell publica o retrato de cada preso ([`crate::retrato`]); aqui o cartão preso constrói-se e
//! pinta-se com esse retrato, as caixas dele são as que a semente da seleção escreveria com ele
//! ([`ensaia`]), e o gesto que cai nele corre com ele instalado ([`com_gesto`]) — os pedidos levam o
//! `entity_bits` do preso.

use crate::cards::{
    CARD_APPEARANCE, CARD_FILL, CARD_MATERIAL, CARD_OBJECT, CARD_ORDERING, CARD_STROKE,
    CARD_TRANSFORM, CARD_TRANSFORM_IMAGE,
};
use crate::cards_componentes::{Abertas, Presente};
use crate::paint_frame::LiveSnapshots;
use crate::retrato::{Retrato, com_preso};
use crate::state::{self, InspectorState};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardBody, CardDef};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

/// Os cartões do objecto que se prendem: a escrita de cada um vai ao preso (`docs/CartoesUI/09`).
const DO_OBJECTO: [NodeId; 8] = [
    CARD_OBJECT,
    CARD_TRANSFORM,
    CARD_TRANSFORM_IMAGE,
    CARD_APPEARANCE,
    CARD_ORDERING,
    CARD_MATERIAL,
    CARD_FILL,
    CARD_STROKE,
];

/// Um solto preso deste quadro: a instância, o objecto e o que o corpo do cartão empresta.
pub(crate) struct Preso {
    k: NodeId,
    pin: u64,
    secao: NodeId,
    componente: Option<(Presente, LiveSnapshots)>,
}

/// A chave do que já foi medido: o retrato do preso, o da seleção e o seletor de unidade.
type Chave = (Retrato, Retrato, (bool, u32));
/// O retrato do preso, o seletor de unidade e o componente presente com que o cartão se montou.
type Montado = (Retrato, u32, Option<Presente>);

thread_local! {
    /// O ensaio e a montagem de cada solto preso só se refazem quando a chave muda (medido no bloco
    /// 10, cena 7, perfil smoke: ~1 ms e ~0,6 ms por quadro e por cartão sem isto).
    static MEDIDOS: RefCell<BTreeMap<NodeId, Chave>> = RefCell::default();
    static MONTADOS: RefCell<BTreeMap<NodeId, Montado>> = RefCell::default();
    /// O objecto do cartão preso cujo corpo abriu uma lista suspensa neste quadro.
    static SUJEITO_DA_LISTA: Cell<Option<u64>> = const { Cell::new(None) };
}

/// O corpo de um cartão preso: com o retrato do objecto; se ele abriu uma lista, ela é desse objecto.
fn corpo_do_preso<R>(pin: u64, y: R, f: impl FnOnce() -> R) -> R {
    use ph2d_editor_core::widget::card_ink::troca_popover;
    let antes = troca_popover(None);
    let fim =
        ph2d_editor_core::widget::key_mark::com_marcas_de(pin, || com_preso(pin, f)).unwrap_or(y);
    match troca_popover(None) {
        Some(aberta) => {
            SUJEITO_DA_LISTA.with(|s| s.set(Some(pin)));
            let _ = troca_popover(Some(aberta));
        }
        None => {
            let _ = troca_popover(antes);
        }
    }
    fim
}

/// O objecto da lista aberta deste quadro (`None` = a seleção); lê-se uma vez, no passe das listas.
pub(crate) fn toma_sujeito_da_lista() -> Option<u64> {
    SUJEITO_DA_LISTA.with(Cell::take)
}

/// As caixas de cada preso: a semente da seleção corrida com o retrato dele, num ensaio (o store,
/// o foco e o barramento voltam como estavam); fica só o que difere da seleção.
pub(crate) fn ensaia(inspector_state: &InspectorState, host: &mut dyn PanelHostInternal) {
    let selecao = Retrato::captura();
    let unidade = crate::unidade_mundo::assinatura(host.project());
    for (k, solto) in host.store().loose_cards_of(ids::INSP_PANEL) {
        let Some(r) = solto.pin.and_then(crate::retrato::de) else {
            continue;
        };
        let chave = (r, selecao.clone(), unidade);
        let feito = host.store().pinned_states(k).is_some()
            && MEDIDOS.with(|m| m.borrow().get(&k) == Some(&chave));
        if feito {
            continue;
        }
        let pin = solto.pin.unwrap_or_default();
        let medido = com_preso(pin, || {
            let n = host.bus().len();
            let ensaio = host.store_mut().abre_ensaio();
            crate::sync::semeia_tudo(&mut inspector_state.clone(), host, true, false);
            crate::cards::semeia_a_opacidade(host.store_mut());
            crate::cards::semeia_as_linhas(host.store_mut());
            crate::cards::semeia_as_amostras(host.store_mut());
            let e = host.store_mut().fecha_ensaio(ensaio);
            host.bus_mut().truncate(n);
            e
        });
        if let Some(e) = medido {
            host.store_mut().set_pinned_states(k, e);
            MEDIDOS.with(|m| m.borrow_mut().insert(k, chave));
        }
    }
}

/// Os soltos presos e o que cada cartão precisa (o retrato do objecto à volta).
pub(crate) fn monta(
    store: &mut WidgetStore,
    text: &mut TextSystem,
    theme: Theme,
    w: f32,
    abertas: Abertas,
    presentes: &[Presente],
) -> Vec<Preso> {
    for c in DO_OBJECTO {
        store.mark_card_pinnable(c);
    }
    for p in presentes {
        store.mark_card_pinnable(p.id);
    }
    let mut out = Vec::new();
    for (k, solto) in store.loose_cards_of(ids::INSP_PANEL) {
        let Some(pin) = solto.pin else { continue };
        let secao = solto.section;
        // `None` = ainda não publicado (ou o objecto saiu): a chave durável espera por ele.
        let Some((nome, componente)) = com_preso(pin, || {
            let nome = state::current_inspector_name().map(|n| n.name);
            if DO_OBJECTO.contains(&secao) {
                crate::cards::dica_do_cadeado(store);
                return (nome, Some(None));
            }
            let snaps = LiveSnapshots::fetch();
            let r = crate::retrato::de(pin).unwrap_or_default();
            let guardado = MONTADOS.with(|m| {
                m.borrow()
                    .get(&k)
                    .filter(|(g, gw, _)| *g == r && *gw == w.to_bits())
                    .map(|(_, _, p)| p.clone())
            });
            let p = guardado.unwrap_or_else(|| {
                let p = crate::cards_componentes::presentes(
                    store,
                    text,
                    theme,
                    w,
                    abertas,
                    &snaps,
                    Some(secao),
                )
                .pop();
                MONTADOS.with(|m| m.borrow_mut().insert(k, (r, w.to_bits(), p.clone())));
                p
            });
            (nome, p.map(|p| Some((p, snaps))))
        }) else {
            continue;
        };
        let Some(componente) = componente else {
            continue; // o preso não tem (já) este componente
        };
        store.set_card_subject_name(pin, nome.unwrap_or_default());
        out.push(Preso {
            k,
            pin,
            secao,
            componente,
        });
    }
    out
}

/// Os cartões dos soltos presos: o def de cada um com o retrato do objecto à volta do corpo.
pub(crate) fn cards<'a>(
    presos: &'a [Preso],
    store: &WidgetStore,
    abertas: &'a Cell<Abertas>,
) -> Vec<CardDef<'a>> {
    let mut out = Vec::new();
    for p in presos {
        let pin = p.pin;
        let def = com_preso(pin, || match &p.componente {
            Some((presente, snaps)) => {
                Some(crate::cards_componentes::cartao(presente, abertas, snaps))
            }
            None => ph2d_editor_core::panel::cards::estilos::declarando(pin, p.k, || {
                crate::cards::cards(store)
                    .into_iter()
                    .find(|c| c.section == p.secao)
            }),
        });
        let Some(Some(mut d)) = def else { continue };
        d.section = p.k; // T54: o def é da INSTÂNCIA (`cards_solto::pinta_soltos` acha-o por ela)
        d.body = d
            .body
            .map(|f| -> CardBody<'a> { Box::new(move |c, y| corpo_do_preso(pin, y, || f(c, y))) });
        out.push(d);
    }
    out
}
