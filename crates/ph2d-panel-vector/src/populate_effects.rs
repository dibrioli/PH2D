//! O registro dos widgets da seção **EFFECTS** — irmão do [`super`] pelo teto de 600 LOC, e
//! par do `paint_effects` (que os PINTA).
//!
//! Registrar é o que os torna clicáveis: sem `InteractiveState` o Down nunca ativa o widget e
//! o Up nunca emite `Click`, então o botão fica **pintado e morto**.
//!
//! # Registra-se o TETO, sempre
//!
//! A seção é dirigida pela tabela: quantas linhas e quantos parâmetros existem só se sabe no
//! frame, e o store é montado uma vez. Então registra-se o máximo — `MAX_FX_ROWS` linhas ×
//! `MAX_FX_ROW_PARAMS` parâmetros, mais `MAX_FX_KINDS` botões de Add. Registrar de MENOS deixa
//! um controle clicável e morto; registrar de MAIS é inerte, porque quem decide se o clique é
//! possível é a PINTURA (sem hit-rect não há Click). É o padrão dos presets do Envelope.

use super::{button, slider_chip};
use crate::ids;
use ph2d_editor_core::interaction::WidgetStore;

/// O passo dos campos numéricos. As faixas variam (fração, contagem), então o passo é uma
/// fração da faixa — resolvido no `set_number_range` por parâmetro, com este piso.
const STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// O track do slider é NORMALIZADO `0..1` e o valor mostrado é o do documento; a faixa real
/// vem publicada por frame. Aqui o registro usa a identidade, e o `paint` reposiciona o track a
/// cada frame a partir do snapshot.
const IDENTITY_SCALE: f32 = 1.0;
const IDENTITY_OFFSET: f32 = 0.0;

/// Os widgets da seção Effects — o teto de tudo.
pub(super) fn populate_effects(store: &mut WidgetStore) {
    for kind in 0..ids::MAX_FX_KINDS {
        button(store, ids::vector_fx_add_id(kind));
    }
    for row in 0..ids::MAX_FX_ROWS {
        button(store, ids::vector_fx_remove_id(row));
        button(store, ids::vector_fx_up_id(row));
        button(store, ids::vector_fx_down_id(row));
        button(store, ids::vector_fx_hide_id(row));
        for param in 0..ids::MAX_FX_ROW_PARAMS {
            let (slider, num) = (
                ids::vector_fx_param_id(row, param),
                ids::vector_fx_param_num_id(row, param),
            );
            slider_chip(
                store,
                slider,
                num,
                0.0,
                0.0,
                IDENTITY_SCALE,
                IDENTITY_OFFSET,
            );
            // A faixa aqui é a do TRACK (`0..1`); o campo mostra o valor do documento e a
            // faixa dele é republicada pelo paint. Sem `set_number_range` o arrasto do campo
            // escala errado — não é opcional.
            store.set_number_range(num, 0.0, 1.0, STEP);
        }
    }
}
