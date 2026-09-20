//! **O registo dos widgets da secção WEAPON** — a arma do jogador.
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou sete vezes, e a última foi nesta linha em 19/09.*
//!
//! # ⭐⭐⭐ Os dois tectos são de PRODUTO, e dizem-no (§0.0)
//!
//! `WEAPON_MAX_MS` é **um minuto**, e o recurso dele é o PAINEL — não a representação: o acumulador
//! é `u64` de microssegundos e só satura depois de ~584 mil anos. *Uma caixa que aceita um valor que
//! ninguém consegue esperar produz estado inalcançável* (a lei do `TIMER_MAX_US`), e aqui o
//! intervalo é mais curto que o de um relógio de cena **por natureza**, porque quem espera por ele é
//! a mão de quem joga.
//!
//! ⚠️ **O passo é `10 ms` e não `1`**: a cadência de uma arma vive entre `50` e `1 000` ms, e um
//! arrasto de um em um milissegundo atravessaria a faixa útil em cem gestos.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::weapon_edits::{WEAPON_MAX_MS_UI as TECTO, WEAPON_STEP_MS as PASSO};
use ph2d_editor_core::widget::TextInputState;

/// `(id, valor de partida, mínimo, máximo, passo)`.
///
/// ⚠️ **Os valores de partida são os do [`WeaponFire::default()`]**, que são zeros — e aqui isso é
/// o certo: `cooldown_ms = 0` é *sem cadência* e `reload_ms = 0` é *não recarrega*, os dois estados
/// neutros e observáveis. ⛔ Nascer com um número inventado faria o artista herdar um ritmo que ele
/// não escreveu.
const NUMEROS: [(ph2d_a11y::NodeId, f64, f64, f64, f64); 2] = [
    (ids::INSP_WEAPON_COOLDOWN, 0.0, 0.0, TECTO, PASSO),
    (ids::INSP_WEAPON_RELOAD_MS, 0.0, 0.0, TECTO, PASSO),
];

pub(crate) fn populate_weapon(store: &mut WidgetStore) {
    // ⚠️ **Os SEIS nomes são TEXTO**, e vazio = calado — a regra do `SignalOnHit`, palavra por
    // palavra. Registá-los é o que os torna clicáveis; sem isto eles pintam e morrem sob o dedo.
    for id in [
        ids::INSP_WEAPON_ON_SIGNAL,
        ids::INSP_WEAPON_AMMO,
        ids::INSP_WEAPON_RELOAD_ON,
        ids::INSP_WEAPON_ON_FIRE,
        ids::INSP_WEAPON_ON_EMPTY,
        ids::INSP_WEAPON_ON_RELOADED,
        ids::INSP_WEAPON_RESERVE,
    ] {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
    }
    for (id, value, lo, hi, step) in NUMEROS {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(id, lo, hi, step);
    }
}
