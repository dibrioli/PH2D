//! **Os testes do eixo espectral** (`input_dispatch::spectral_axis_tests`) — o corpo do módulo mudou-se VERBATIM
//! (`line/input-dispatch`, 2026-09-13) para um ficheiro, declarado por `#[path]` no índice com o MESMO nome: os
//! testes continuam a chamar-se como se chamavam.

use super::freq_at_y;
use ph2d_app_audio::WaveView;
use ph2d_editor_core::zones::Rect;

fn view() -> WaveView {
    WaveView {
        rect: Rect::new(100.0, 200.0, 400.0, 300.0),
        ruler: Rect::new(100.0, 180.0, 400.0, 20.0),
        frames: 48_000,
    }
}

/// **Frequency runs UP the spectrogram; screen y runs DOWN.**
///
/// This inversion is the whole of the mapping, and getting it backwards is the worst
/// kind of bug this feature can have: the user drags a box around a 5 kHz beep, and the
/// repair confidently rebuilds the *mirror* band instead — removing a sound they wanted
/// and leaving the one they pointed at. Nothing crashes; nothing looks wrong; the beep
/// is still there and something else is gone.
///
/// Red if the `1.0 -` is dropped: the top of the view would report DC.
#[test]
fn the_top_of_the_view_is_the_highest_frequency() {
    let v = view();
    let top = freq_at_y(&v, v.rect.y);
    let bottom = freq_at_y(&v, v.rect.y + v.rect.h);
    assert!(
        top > 0.99,
        "the TOP of the spectrogram should be Nyquist, got {top}"
    );
    assert!(
        bottom < 0.01,
        "the BOTTOM of the spectrogram should be DC, got {bottom}"
    );
    // …and the middle is the middle: a linear axis, which is what the picture draws.
    let mid = freq_at_y(&v, v.rect.y + v.rect.h * 0.5);
    assert!((mid - 0.5).abs() < 0.01, "the axis is not linear: {mid}");
}

/// A drag that leaves the view still names a frequency inside it — the band is clamped,
/// not wrapped. (A wrap would jump the selection to the other end of the spectrum.)
#[test]
fn dragging_out_of_the_view_clamps() {
    let v = view();
    assert_eq!(freq_at_y(&v, v.rect.y - 500.0), 1.0);
    assert_eq!(freq_at_y(&v, v.rect.y + v.rect.h + 500.0), 0.0);
}
