//! **O registo dos widgets da secção TWEEN** (suplente #22).
//!
//! ⚠️ **Irmão de [`super::populate`] por CAP de FICHEIRO** — o mesmo corte das irmãs.
//!
//! ⚠️⚠️ **Um id que não passa por aqui é PINTADO, HIT-REGISTADO e MORTO sob o rato**: o despachante
//! decide pelo `is_focusable`, e o ramo `None => false` engole o clique em silêncio. *Esta crate já
//! o pagou oito vezes, e as duas últimas foram nesta linha (as amostras de cor do emissor em 16/09
//! e a secção do raio em 19/09).*
//!
//! # ⭐⭐ A FAIXA dos dois extremos, e porque ela é a MESMA para os oito canais
//!
//! O `de`/`para` de um tween é sempre `[f32; 4]`, e o que cada componente SIGNIFICA depende do
//! canal: uma opacidade e uma cor vivem em `0..1`, uma posição vive em metros, uma rotação em
//! radianos. ⇒ a faixa tem de conter a união delas, e é `−50..50` — a mesma da origem do raio, que
//! é a escala de uma cena deste app.
//!
//! ⛔ **Uma faixa POR CANAL seria uma segunda lei sobre a aridade**, e ela divergiria no dia do
//! nono canal: o painel já deriva do [`ph2d_tween::Canal`] *quantas* componentes contam, e derivar
//! dali também *que faixa cada uma tem* é desenho novo — nomeado e não construído, porque hoje o
//! `clamp` de cada consumidor já é a cerca real (a alfa satura em `0..1` no shader, a escala em
//! nenhuma).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;

/// A faixa dos oito campos numéricos — ver o cabeçalho.
const FAIXA: (f64, f64, f64) = (-50.0, 50.0, 0.05); // LITERAL-PX-OK: a união das unidades dos canais

pub(crate) fn populate_tween(store: &mut WidgetStore) {
    // ⚠️ **Os chips são BOTÕES** — sem registo eles pintam e morrem sob o dedo, que é exactamente o
    // defeito que os treze chips da escultura e as duas amostras do emissor pagaram.
    //
    // ⭐ **A lista sai dos arrays de ids, e eles saem dos `ALL` do motor** — um canal, uma família
    // ou um modo novo entra aqui **sem esta linha mudar**, porque o que se percorre é o array.
    for id in ids::INSP_TWEEN_CANAL
        .iter()
        .chain(ids::INSP_TWEEN_FAMILIA.iter())
        .chain(ids::INSP_TWEEN_MODO.iter())
        .chain(ids::INSP_TWEEN_AO_ACABAR.iter())
        .chain(ids::INSP_TWEEN_ROW.iter())
        .copied()
        .chain([ids::INSP_TWEEN_ADD, ids::INSP_TWEEN_REMOVE])
    {
        store.register(
            id,
            InteractiveState::Button {
                state: Default::default(),
            },
        );
    }
    // ⚠️ **As linhas da LISTA também são botões**, e a razão é a mesma do irmão dos timers: escolher
    // qual tween está aberto é um clique, e um clique precisa de um id focalizável.
    for id in ids::INSP_TWEEN_DE.iter().chain(ids::INSP_TWEEN_PARA.iter()) {
        let value = 0.0;
        store.register(
            *id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value,
                buffer: format_number(value),
                caret: 0,
                last_committed: value,
                selection_anchor: None,
            },
        );
        store.set_number_range(*id, FAIXA.0, FAIXA.1, FAIXA.2);
    }
}
