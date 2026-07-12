//! O **plano** da barra: a sequência de controles e onde cada um cai.
//!
//! Existe separado do paint por uma razão dura: a tira precisa saber **quantas
//! linhas** a barra vai ocupar *antes* de pintar a superfície (o painel cresce
//! para caber). Medir e pintar por caminhos diferentes é como as duas versões
//! divergem — então há UMA sequência (`items`) e UM algoritmo de quebra
//! (`plan`), e tanto a medida quanto a pintura consomem o mesmo resultado.
//!
//! **A barra QUEBRA, nunca esconde.** A versão anterior descartava em silêncio
//! todo item que não coubesse na linha ("a barra nunca transborda"), e o efeito
//! era pior que transbordar: num viewport de 1280px NOVE dos dezoito controles
//! sumiam — as ops de chave, o hold, o ciclo e o tween inteiro — sem scroll, sem
//! overflow, sem qualquer sinal de que existiam. E como o teste era por-item
//! (`x + w <= right`), uma caixa estreita ENTRAVA depois que um botão largo fora
//! descartado: a barra saía com buracos, fora de ordem. Um controle invisível é
//! um controle que não existe.

use crate::ids;
use crate::state::FlipStripSnapshot;
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{Spacing, TypeToken};

/// Largura de um botão de ícone (quadrado, altura da linha).
pub(crate) const ICON_W: f32 = 28.0; // LITERAL-PX-OK: strip toolbar icon slot
/// Largura de uma caixa numérica pequena da barra.
pub(crate) const NUM_W: f32 = 46.0; // LITERAL-PX-OK: strip toolbar number chip
/// Largura de um toggle de texto (Ghost / Auto / Additive).
pub(crate) const TOGGLE_W: f32 = 62.0; // LITERAL-PX-OK: strip toolbar toggle
/// Largura do chip do ciclo.
pub(crate) const CYCLE_W: f32 = 84.0; // LITERAL-PX-OK: strip cycle dropdown chip
/// Largura média de um glifo em fração do tamanho da fonte — só para RESERVAR o
/// espaço de um rótulo curto da barra (não é métrica de design; a medida real do
/// texto sai do shaper, que aqui seria caro por um "FPS").
const GLYPH_W_RATIO: f32 = 0.62; // LITERAL-PX-OK: estimativa de largura de glifo

/// Um controle da barra (ou um respiro entre grupos).
pub(crate) enum Item {
    Icon(NodeId, IconId),
    Toggle(NodeId, &'static str, bool),
    Number(NodeId, f64),
    Label(&'static str),
    Cycle(u8),
    /// Respiro entre grupos — some no início de uma linha nova.
    Gap,
}

impl Item {
    /// A largura que o item reserva. `None` = não é um controle (respiro).
    fn width(&self) -> Option<f32> {
        Some(match self {
            Item::Icon(..) => ICON_W,
            Item::Toggle(..) => TOGGLE_W,
            Item::Number(..) => NUM_W,
            Item::Label(t) => {
                TypeToken::Sm.px() * GLYPH_W_RATIO * t.len() as f32 + Spacing::Xs.px()
            }
            Item::Cycle(_) => CYCLE_W,
            Item::Gap => return None,
        })
    }
}

/// **A sequência canônica da barra** — transporte · Ghost · autoria · ops de
/// chave · tween · ciclo. É a única lista; medir e pintar leem esta.
pub(crate) fn items(snap: &FlipStripSnapshot) -> Vec<Item> {
    // A exposição da chave ATIVA (o que a caixa "Hold" edita).
    let hold = snap
        .current_key
        .and_then(|k| snap.cells.iter().find(|c| c.key == k))
        .map_or(1, |c| c.exposure);
    let play_icon = if snap.playing {
        IconId::Pause
    } else {
        IconId::Play
    };
    vec![
        // Transporte: ◀ desenho · play/pause · desenho ▶
        Item::Icon(ids::FLIP_PREV_DRAWING, IconId::SkipBack),
        Item::Icon(ids::FLIP_PLAY, play_icon),
        Item::Icon(ids::FLIP_NEXT_DRAWING, IconId::SkipForward),
        Item::Label("FPS"),
        Item::Number(ids::FLIP_FPS_NUM, f64::from(snap.fps)),
        Item::Gap,
        // Ghost Frames: liga/desliga + quantos antes/depois
        Item::Toggle(ids::FLIP_GHOST, "Ghost", snap.ghost),
        Item::Number(ids::FLIP_GHOST_BEFORE_NUM, f64::from(snap.ghost_before)),
        Item::Number(ids::FLIP_GHOST_AFTER_NUM, f64::from(snap.ghost_after)),
        Item::Gap,
        // Autoria: o que nasce ao desenhar depois do hold
        Item::Toggle(ids::FLIP_AUTOKEY, "Auto", snap.autokey),
        Item::Toggle(ids::FLIP_ADDITIVE, "Add.", snap.additive),
        Item::Gap,
        // Ops de chave: + · duplicar · apagar · exposição · mover ±1
        Item::Icon(ids::FLIP_KEY_ADD, IconId::Plus),
        Item::Icon(ids::FLIP_KEY_DUP, IconId::Copy),
        Item::Icon(ids::FLIP_KEY_DELETE, IconId::Trash),
        Item::Label("Hold"),
        Item::Number(ids::FLIP_HOLD_NUM, f64::from(hold)),
        Item::Icon(ids::FLIP_KEY_LEFT, IconId::ChevronLeft),
        Item::Icon(ids::FLIP_KEY_RIGHT, IconId::ChevronRight),
        Item::Gap,
        // Tween: quantos inbetweens + gerar
        Item::Label("Tween"),
        Item::Number(ids::FLIP_TWEEN_NUM, f64::from(snap.tween_count)),
        Item::Toggle(ids::FLIP_TWEEN_ADD, "Add", false),
        Item::Gap,
        // Ciclo (post behavior da camada ativa)
        Item::Cycle(snap.cycle),
    ]
}

/// Onde cada item cai, dado o rect da PRIMEIRA linha. Quebra para a linha
/// seguinte quando o item não cabe — **nunca descarta**. Devolve também quantas
/// linhas a barra ocupou (≥ 1), que é o que faz a tira crescer.
///
/// Um item mais largo que a linha inteira transborda em vez de sumir: melhor um
/// controle cortado (que o usuário vê e pode alcançar redimensionando) do que um
/// controle ausente (que ele conclui que não existe).
pub(crate) fn plan(items: &[Item], first_row: Rect, row_h: f32) -> (Vec<Rect>, u32) {
    let left = first_row.x;
    let right = first_row.x + first_row.w;
    let gap = Spacing::Xs.px();
    let row_gap = Spacing::Xs.px();

    let mut out = Vec::with_capacity(items.len());
    let mut x = left;
    let mut y = first_row.y;
    let mut rows = 1u32;

    for item in items {
        let Some(w) = item.width() else {
            // Respiro: só separa grupos DENTRO de uma linha; no começo de uma
            // linha nova ele não deve empurrar o primeiro controle para dentro.
            if x > left {
                x += Spacing::Md.px();
            }
            out.push(Rect::new(x, y, 0.0, 0.0));
            continue;
        };
        // Quebra: não cabe e a linha já tem algo → próxima linha.
        if x > left && x + w > right {
            x = left;
            y += row_h + row_gap;
            rows += 1;
        }
        out.push(Rect::new(x, y, w, row_h));
        x += w + gap;
    }
    (out, rows)
}

/// Quantas linhas a barra ocupa nesta largura — o que a tira precisa saber
/// ANTES de pintar a superfície (ela cresce para caber a barra inteira).
pub(crate) fn rows(inner: Rect, row_h: f32, snap: &FlipStripSnapshot) -> u32 {
    plan(&items(snap), inner, row_h).1
}

/// A altura total de uma barra de `rows` linhas.
pub(crate) fn bar_height(rows: u32, row_h: f32) -> f32 {
    let n = rows.max(1) as f32;
    n * row_h + (n - 1.0) * Spacing::Xs.px()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> FlipStripSnapshot {
        FlipStripSnapshot {
            has_layer: true,
            fps: 12.0,
            ..Default::default()
        }
    }

    /// **O bug que motivou o módulo:** nenhum controle pode sumir, em nenhuma
    /// largura plausível. Antes, um viewport de 1280 comia nove deles.
    #[test]
    fn no_control_is_ever_dropped_at_any_width() {
        let s = snap();
        let n_controls = items(&s).iter().filter(|i| i.width().is_some()).count();
        for w in [
            320.0f32, 640.0, 800.0, 1024.0, 1280.0, 1440.0, 1920.0, 3440.0,
        ] {
            let (rects, rows) = plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0);
            let placed = rects.iter().filter(|r| r.w > 0.0 && r.h > 0.0).count();
            assert_eq!(
                placed,
                n_controls,
                "largura {w}: {} controles sumiram (a barra escondeu em vez de quebrar)",
                n_controls - placed
            );
            assert!(rows >= 1, "largura {w}: zero linhas");
        }
    }

    /// A barra larga cabe numa linha só (não se paga altura à toa).
    #[test]
    fn a_wide_bar_stays_on_one_row() {
        let (_, rows) = plan(&items(&snap()), Rect::new(0.0, 0.0, 2000.0, 28.0), 28.0);
        assert_eq!(rows, 1);
    }

    /// Estreitar exige mais linhas — e a contagem é monótona (mais largura nunca
    /// pede mais linhas).
    ///
    /// Nota de calibração: a largura aqui é a da BARRA, não a da janela — a tira
    /// desconta o rail, o dock e o padding, então uma janela de 1280px dá uma
    /// barra de ~800. É por isso que o corte real aparece cedo.
    #[test]
    fn narrower_needs_more_rows_monotonically() {
        let s = snap();
        let row_at = |w: f32| plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0).1;
        assert!(row_at(800.0) > 1, "uma barra de 800px tem de quebrar");
        for pair in [(400.0, 640.0), (640.0, 900.0), (900.0, 1400.0)] {
            assert!(
                row_at(pair.0) >= row_at(pair.1),
                "mais estreito ({}) pediu MENOS linhas que {}",
                pair.0,
                pair.1
            );
        }
    }

    /// Nenhum controle é pintado FORA da faixa horizontal (a não ser que ele
    /// sozinho seja mais largo que a linha — aí transborda, mas existe).
    #[test]
    fn controls_stay_inside_the_row_when_they_fit() {
        let s = snap();
        let w = 1280.0;
        let (rects, _) = plan(&items(&s), Rect::new(0.0, 0.0, w, 28.0), 28.0);
        for r in rects.iter().filter(|r| r.w > 0.0) {
            assert!(
                r.x >= 0.0 && r.x + r.w <= w + 0.01,
                "controle fora da faixa: {r:?}"
            );
        }
    }
}
