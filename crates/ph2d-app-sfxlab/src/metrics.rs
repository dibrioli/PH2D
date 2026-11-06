//! As medidas da tela, em px lógicos. As cores vêm dos tokens; aqui só a geometria.

pub(crate) const BAR_H: f32 = 44.0;
pub(crate) const PAD: f32 = 12.0;
pub(crate) const GAP: f32 = 8.0;
pub(crate) const ROW_GAP: f32 = 4.0;
pub(crate) const RADIUS: f32 = 6.0;
pub(crate) const STROKE: f32 = 1.0;

pub(crate) const NAME_PX: f32 = 15.0;
pub(crate) const NAME_W: f32 = 200.0;
pub(crate) const TEXT_PX: f32 = 13.0;
pub(crate) const SMALL_PX: f32 = 11.0;

pub(crate) const BUTTON_H: f32 = 28.0;
pub(crate) const BUTTON_PAD_X: f32 = 12.0;
pub(crate) const CHECK: f32 = 12.0;
pub(crate) const CHECK_RADIUS: f32 = 3.0;

pub(crate) const STACK_W: f32 = 232.0;

pub(crate) const RULER_H: f32 = 24.0;
pub(crate) const RULER_TICK: f32 = 6.0;
pub(crate) const RULER_MINOR: f32 = 3.0;
/// Riscos menores entre dois rótulos.
pub(crate) const RULER_DIVISIONS: u32 = 5;
pub(crate) const CURSOR_W: f32 = 1.5;

pub(crate) const MIX_H: f32 = 72.0;
pub(crate) const ROW_H: f32 = 48.0;
pub(crate) const HANDLE_W: f32 = 14.0;
pub(crate) const HANDLE_LINE: f32 = 1.5;
pub(crate) const HANDLE_STEP: f32 = 4.0;
pub(crate) const TOGGLE: f32 = 24.0;
pub(crate) const WAVE_PAD: f32 = 3.0;

pub(crate) const ADD_H: f32 = 32.0;
pub(crate) const MENU_ROW_H: f32 = 28.0;
pub(crate) const MENU_PAD: f32 = 4.0;

/// Px que a janela dá por linha de roda (o espelho do `on_mouse_wheel` da shell).
pub(crate) const WHEEL_LINE_PX: f32 = 16.0;
