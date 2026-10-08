//! ⭐ **A cena 11** (`PH2D_OBJECT_MODE_SMOKE=11`, D10 F6 — 08/10): NAVEGAR COM OS DEDOS. Depois da
//! cena 9 (layout *Vetor*, um rectângulo e uma elipse seleccionados, a Caneta na mão), dois dedos
//! SINTÉTICOS pousam no meio da área, andam juntos [`PAN_PX`] (a vista arrasta com eles) e depois
//! afastam-se de [`SPREAD_FROM_PX`] a [`SPREAD_TO_PX`] (a vista aproxima 2× à volta deles) — pelo
//! reconhecedor (`interaction::touch`) e pelas portas de sempre (a shell leva o que ele pede ao
//! botão do MEIO e à RODA). Um ecrã de toque não se simula de fora (o XTest da Xwayland ignora-se,
//! medido no `fotografa_cena.sh`); o que se simula é o CONTACTO, e tudo o que vem depois é o produto.

use ph2d_editor_core::interaction::TouchPhase;
use ph2d_editor_core::screens::hero::HeroScreen;
use std::cell::Cell;

/// Quanto os dois dedos andam juntos (px da janela), na 1.ª metade do gesto.
const PAN_PX: [f32; 2] = [240.0, 60.0]; // LITERAL-PX-OK: o gesto da cena, não medida de UI
/// Meio afastamento dos dedos no início e no fim da pinça (px): dobrar = aproximar 2×.
const SPREAD_FROM_PX: f32 = 40.0; // LITERAL-PX-OK: o gesto da cena
const SPREAD_TO_PX: f32 = 80.0; // LITERAL-PX-OK: o gesto da cena
/// Cada metade do gesto (s).
const HALF_S: f32 = 1.0;
const NS_PER_S: f32 = 1e9;

thread_local! {
    static T0: Cell<u128> = const { Cell::new(0) };
}

fn fingers(mid: [f32; 2], half: f32) -> [[f32; 2]; 2] {
    [[mid[0] - half, mid[1]], [mid[0] + half, mid[1]]]
}

fn feed(hero: &mut HeroScreen, phase: TouchPhase, at: [[f32; 2]; 2], now: u128) {
    let t = hero.store.touch_mut();
    t.feed_synthetic(1, phase, at[0], now);
    t.feed_synthetic(2, phase, at[1], now);
}

/// Um passo da cena 11; devolve o próximo estágio, se mudou. `40` = os dedos pousam · `41` = andam
/// e afastam-se · `9` = feito (os dedos levantaram; o app é do dono).
pub fn step(stage: u8, hero: &mut HeroScreen) -> Option<u8> {
    let r = hero.last_content;
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let c = [r.x + r.w / 2.0, r.y + r.h / 2.0];
    let now = hero.store.clock_ns();
    match stage {
        40 => {
            T0.with(|t| t.set(now));
            feed(hero, TouchPhase::Start, fingers(c, SPREAD_FROM_PX), now);
            Some(41)
        }
        41 => {
            #[allow(clippy::cast_precision_loss)]
            let t = now.saturating_sub(T0.with(Cell::get)) as f32 / NS_PER_S;
            let pan = (t / HALF_S).clamp(0.0, 1.0);
            let pinch = ((t - HALF_S) / HALF_S).clamp(0.0, 1.0);
            let mid = [c[0] + PAN_PX[0] * pan, c[1] + PAN_PX[1] * pan];
            let half = SPREAD_FROM_PX + (SPREAD_TO_PX - SPREAD_FROM_PX) * pinch;
            if t >= 2.0 * HALF_S {
                feed(hero, TouchPhase::End, fingers(mid, half), now);
                return Some(9);
            }
            feed(hero, TouchPhase::Move, fingers(mid, half), now);
            None
        }
        _ => None,
    }
}
