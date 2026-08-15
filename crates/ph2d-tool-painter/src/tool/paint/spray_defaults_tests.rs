//! **A PRIMEIRA nuvem tem de parecer uma nuvem** — o default que o `Count` arma (plano 38 W5).
//!
//! Sem espalhamento, `n` marcas caem no mesmo lugar e o pincel de fábrica (`strength`/`flow` em 1,0)
//! pinta **exatamente** o que uma marca pinta: o slider seria um controle morto. O tool arma o
//! `Jitter → Position` na transição `1 → n`, **só sobre o zero de fábrica** — o molde do
//! `toggle_brush_impasto`: *arma um default, nunca impõe política*.

use crate::tool::PainterTool;
use ph2d_painter_brush::BrushSpec;
use ph2d_painter_brush::stroke::spray::{SPRAY_COUNT_MAX, SPRAY_DEFAULT_SPREAD};

/// A pista `0..1` que pousa exatamente em `n` marcas.
fn track_for(n: u32) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let t = (n - 1) as f32 / (SPRAY_COUNT_MAX - 1) as f32;
    t
}

/// **Pedir a primeira marca extra ARMA o espalhamento.**
#[test]
fn the_first_extra_mark_arms_the_spread() {
    let mut t = PainterTool::default();
    assert_eq!(
        t.brush_settings().jitter,
        BrushSpec::default().jitter,
        "controle: o pincel nasce sem espalhamento"
    );
    t.set_brush_spray_count_norm(track_for(8));
    assert_eq!(t.brush_settings().spray_count, 8);
    assert_eq!(
        t.brush_settings().jitter,
        SPRAY_DEFAULT_SPREAD,
        "a primeira nuvem nasceu empilhada — o Count seria um controle morto"
    );
}

/// **Um espalhamento AUTORADO nunca é sobrescrito** — é o que separa *armar um default* de *impor
/// política*, e é a metade que torna a regra segura.
#[test]
fn a_deliberate_spread_survives_the_count() {
    let mut t = PainterTool::default();
    t.set_brush_jitter_norm(0.2);
    let mine = t.brush_settings().jitter;
    assert!(
        mine > 0.0,
        "controle: o espalhamento autorado tem de existir"
    );
    t.set_brush_spray_count_norm(track_for(8));
    assert_eq!(
        t.brush_settings().jitter,
        mine,
        "o Count pisou num espalhamento que o artista escolheu"
    );
}

/// **Ele arma UMA vez, na transição.** Subir a contagem de novo não re-arma nada — senão zerar o
/// espalhamento de propósito seria desfeito pelo movimento seguinte do mesmo slider.
#[test]
fn the_spread_is_armed_once_at_the_transition() {
    let mut t = PainterTool::default();
    t.set_brush_spray_count_norm(track_for(4));
    assert_eq!(t.brush_settings().jitter, SPRAY_DEFAULT_SPREAD);
    // O artista zera o espalhamento de propósito…
    t.set_brush_jitter_norm(0.0);
    // …e continua a mexer na contagem.
    t.set_brush_spray_count_norm(track_for(16));
    assert_eq!(t.brush_settings().spray_count, 16);
    assert_eq!(
        t.brush_settings().jitter,
        0.0,
        "o Count re-armou por cima de um zero DELIBERADO"
    );
}

/// **Ficar em uma marca não arma nada** — o controle é o que prova que a regra é a TRANSIÇÃO e não
/// *"todo toque no slider"*.
#[test]
fn staying_at_one_mark_arms_nothing() {
    let mut t = PainterTool::default();
    t.set_brush_spray_count_norm(track_for(1));
    assert_eq!(t.brush_settings().spray_count, 1);
    assert_eq!(
        t.brush_settings().jitter,
        BrushSpec::default().jitter,
        "uma marca por ponto não é uma nuvem, e não pode mexer no espalhamento"
    );
}
