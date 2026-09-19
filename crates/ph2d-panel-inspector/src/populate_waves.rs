//! **O registo dos widgets das secções que chegam por WAVE** — a fila do TOP-20 e os suplentes.
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO**, e o corte é o certo por
//! responsabilidade: o espelho dele já existia do lado da pintura
//! ([`crate::paint_optional_top20`]), e a fila do TOP-20 acrescenta uma linha por wave. *Antes
//! disto, cada wave engordava o registo das secções de sempre com uma linha que não é delas.*
//!
//! ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.
//!
//! ⚠️⚠️ **E a 1.ª tentativa ENGORDOU o ficheiro** (`601 → 618`): extrair a função sem a mudar de
//! ficheiro paga o doc-comment e não devolve nada. *É a lição que a wave do gatilho já tinha
//! pago — «o corte numa ponta engordou a outra».*

use ph2d_editor_core::interaction::WidgetStore;

/// **Os widgets das secções que chegam por WAVE** — a fila do TOP-20 e os suplentes.
///
/// ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
/// decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
/// o pagou sete vezes*, e é por isso que esta lista existe com um nome em vez de estar diluída
/// no meio do registo das secções de sempre.
pub(crate) fn populate_das_waves(store: &mut WidgetStore) {
    super::populate_factory::populate_factory(store);
    super::populate_projectile::populate_projectile(store);
    super::populate_ray::populate_ray(store);
    super::populate_statemachine::populate_statemachine(store);
    super::populate_particles::populate_particles(store);
    super::populate_hud::populate_hud(store);
    super::populate_sequence::populate_sequence(store);
    super::populate_counter_watch::populate_counter_watch(store);
    super::populate_action_trigger::populate_action_trigger(store);
    super::populate_script::populate_script(store);
    super::populate_topdown::populate_topdown(store);
}
