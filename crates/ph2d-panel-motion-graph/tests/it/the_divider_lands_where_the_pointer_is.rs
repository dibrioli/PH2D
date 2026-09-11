//! ⭐⭐⭐ **O DIVISOR ATERRA ONDE O PONTEIRO ESTÁ** — a ida-e-volta entre as duas crates.
//!
//! > Enio, 2026-08-31: *«segurar e arrastar o topo do canvas de nós tem um bug, um offset e um
//! > tremor.»*
//!
//! # ⛔⛔ Duas contas para a mesma banda
//!
//! O painel do grafo escrevia a fracção contra `center_viewport + motion_graph` — a soma das duas
//! metades. Isso era a banda até a **timeline docar dentro do split** (W4.T4) e passar a comer o
//! fundo do `motion_graph`:
//!
//! | quem | denominador de `t` |
//! |---|---|
//! | o painel, ao arrastar | `chrome_h − altura_da_timeline` |
//! | o layout, ao aplicar (`top_h = band.h · t`) | `chrome_h` |
//!
//! ⇒ **offset** (o divisor ia mais longe do que o dedo) e **tremor** (a altura da timeline é
//! clampada pela do grafo, logo o denominador dependia do resultado).
//!
//! ⚠️ **Este gate tem de ser de IDA-E-VOLTA e atravessar as duas crates.** Medir só a fórmula
//! confirmaria a fórmula; o que estava errado era ela **não ser a inversa** de quem a aplica.

use ph2d_editor_core::screens::layout::{CenterSplit, ChromeBands, DockSides, HeroLayout};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_motion_graph::split_fraction;

const W: f32 = 1366.0;
const H: f32 = 1024.0;

fn nodes_layout(t: f32, vertical: bool) -> HeroLayout {
    let split = if vertical {
        CenterSplit::Vertical { t }
    } else {
        CenterSplit::Horizontal { t }
    };
    let mut l = HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, W, H),
        false,
        ChromeBands::DEFAULT,
        split,
        DockSides::BOTH,
    );
    // ⚠️ **A docagem é obrigatória neste gate** — é ela que corta o fundo do grafo, e era ela que
    // partia a conta. Um gate sem ela mediria o caso que nunca falhou.
    l.dock_timeline_into_motion();
    l
}

/// Onde a fronteira scene↔grafo está de facto no ecrã.
fn divider_of(l: &HeroLayout, vertical: bool) -> f32 {
    if vertical {
        l.motion_graph.x
    } else {
        l.motion_graph.y
    }
}

/// ⭐⭐⭐ **Agarrar o divisor e largá-lo em `y` põe-no em `y`.**
#[test]
fn dragging_the_divider_puts_it_under_the_pointer() {
    for vertical in [false, true] {
        let start = nodes_layout(CenterSplit::T_DEFAULT, vertical);
        // Controlo: sem a timeline docada este gate mediria o caso que já funcionava.
        assert!(
            start.timeline.h > 1.0
                && (start.motion_graph.y + start.motion_graph.h) < start.viewport.h - 1.0,
            "controlo: a timeline não está docada dentro do split (vertical={vertical})"
        );

        // ⚠️ Os alvos saem da BANDA e ficam dentro da faixa legal do divisor
        // (`T_MIN..T_MAX`): fora dela o `clamp_t` mexe no resultado de propósito, e o gate
        // mediria a cerca em vez da conta. (A 1.ª redacção usava `300 px` fixos — `t = 0,246`
        // no alvo de referência — e acusava a cerca de 4 px de offset.)
        for frac in [0.30_f32, 0.50, 0.70] {
            let band = start.split_band;
            let target = if vertical {
                band.x + band.w * frac
            } else {
                band.y + band.h * frac
            };
            let pointer = if vertical {
                (target, 500.0)
            } else {
                (700.0, target)
            };
            let t = split_fraction(start.split_band, start.motion_graph, pointer);
            let after = nodes_layout(CenterSplit::clamp_t(t), vertical);
            let landed = divider_of(&after, vertical);
            assert!(
                (landed - target).abs() <= 1.0,
                "vertical={vertical}: o dedo largou em {target} e o divisor foi para {landed} \
                 ({} px de offset)",
                landed - target
            );
        }
    }
}

/// ⭐⭐ **E arrastar É ESTÁVEL: repetir o gesto no mesmo sítio não move nada.**
///
/// ⛔ Era este o **tremor**. A altura da timeline é clampada pela altura do grafo; com o
/// denominador a incluí-la, cada quadro do arrasto lia uma banda diferente e o divisor oscilava
/// sem o dedo se mexer. *Uma fracção medida contra uma grandeza que depende dela não converge: ela
/// vibra.*
#[test]
fn holding_the_divider_still_never_makes_it_drift() {
    for vertical in [false, true] {
        let pointer = if vertical {
            (500.0, 500.0)
        } else {
            (700.0, 520.0)
        };
        let mut l = nodes_layout(CenterSplit::T_DEFAULT, vertical);
        let mut seen = Vec::new();
        // Dez quadros com o dedo PARADO — é o que um arrasto real faz.
        for _ in 0..10 {
            let t = CenterSplit::clamp_t(split_fraction(l.split_band, l.motion_graph, pointer));
            l = nodes_layout(t, vertical);
            seen.push(divider_of(&l, vertical));
        }
        let first = seen[0];
        let drift = seen
            .iter()
            .map(|v| (v - first).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            drift <= 0.5,
            "vertical={vertical}: o divisor derivou {drift} px com o dedo parado — {seen:?}"
        );
    }
}
