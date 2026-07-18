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
use crate::{ids, state};
use ph2d_editor_core::interaction::WidgetStore;

/// O passo dos campos numéricos. As faixas variam (fração, contagem), então o passo é uma
/// fração da faixa — resolvido no `set_number_range` por parâmetro, com este piso.
const STEP: f64 = 0.01; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Em quantos passos o arrasto atravessa a faixa inteira. É granularidade de gesto — não há
/// token de design para "quantos degraus tem um scrub".
const SCRUB_STEPS: f64 = 100.0; // LITERAL-PX-OK: contagem de passos do arrasto, não medida

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
            // A faixa aqui é a do TRACK (`0..1`); a REAL é republicada por frame pelo
            // `seed_effect_ranges` — só ele sabe que efeito caiu nesta linha. Sem
            // `set_number_range` o arrasto do campo escala errado: não é opcional.
            store.set_number_range(num, 0.0, 1.0, STEP);
        }
    }
}

/// **Republica a faixa REAL de cada parâmetro visível**, por frame.
///
/// # O bug que isto fecha (Enio, 2026-07-18)
///
/// > *"o número que aparece ao arrastar a caixa numérica é de 0 a 1, sendo que os números reais
/// > (quando solta o mouse) são outros"*
///
/// O slider guarda um track normalizado `0..1` e o chip está LIGADO a ele. Registados na
/// identidade, os dois diziam a mesma coisa — e essa coisa era o track. O valor do documento só
/// aparecia no fim, quando o snapshot voltava com ele: **o chip mostrava um número durante o
/// gesto e outro depois**, que é a definição de um readout em que não se pode confiar.
///
/// O canal certo já existia: `link_slider_number_mapped` regista a projeção afim
/// `display = track · escala + origem`. Escala e origem são a faixa do efeito.
///
/// **Por que aqui e não no `populate`:** que efeito ocupa a linha `k` só se sabe no frame — o
/// artista adiciona, remove e reordena. O `populate` corre uma vez. Este é o mesmo padrão do
/// `set_number_range` dos eixos de variação de fonte, cuja faixa também vem do documento.
///
/// **Por que não converter no painel:** ele teria de guardar a faixa, e passaria a haver duas
/// cópias dela — a do motor e a dele. Divergiam no primeiro efeito com faixa diferente, que é
/// exatamente o que o Zig Zag trouxe (`Size` vai a 100, `Ridges` a 128).
pub(crate) fn seed_effect_ranges(store: &mut WidgetStore) {
    for (row, fx) in state::stack().iter().enumerate().take(ids::MAX_FX_ROWS) {
        for (param, p) in fx.params.iter().enumerate().take(ids::MAX_FX_ROW_PARAMS) {
            let (slider, num) = (
                ids::vector_fx_param_id(row, param),
                ids::vector_fx_param_num_id(row, param),
            );
            // Uma caixinha é pintada como BOTÃO — não tem chip para mapear, e mapeá-la
            // registaria uma projeção para um widget que ninguém desenha.
            if p.toggle {
                continue;
            }
            #[allow(clippy::cast_possible_truncation)]
            let (scale, offset) = ((p.max - p.min) as f32, p.min as f32);
            // Faixa degenerada: a projeção seria uma divisão por zero no caminho inverso.
            if scale.abs() <= f32::EPSILON {
                store.link_slider_number(slider, num);
            } else if p.integer {
                store.link_slider_number_mapped_integer(slider, num, scale, offset);
            } else {
                store.link_slider_number_mapped(slider, num, scale, offset);
            }
            // O passo do arrasto é uma fração da faixa — num parâmetro de CONTAGEM ele é 1,
            // senão as setinhas do chip empurrariam um valor que o motor arredonda de volta.
            let step = if p.integer {
                1.0
            } else {
                ((p.max - p.min) / SCRUB_STEPS).max(STEP)
            };
            store.set_number_range(num, p.min, p.max, step);
        }
    }
}
