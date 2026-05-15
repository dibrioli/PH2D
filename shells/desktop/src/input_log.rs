//! Dev-time logging of [`InputEvent`]s — gamepad button / axis spam.
//!
//! Extracted from [`main`] (Track B4). Pure free fn; called from the
//! main loop on every InputEvent so the dev console shows gamepad
//! state transitions without crossing into the simulation crate.

use ph2d_input::Event as InputEvent;

pub fn log_input_event(elapsed_ms: u128, event: &InputEvent) {
    match event {
        InputEvent::GamepadButtonDown(b) => {
            println!(
                "[{:>6}ms] gamepad button down: {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadButtonUp(b) => {
            println!(
                "[{:>6}ms] gamepad button up:   {}",
                elapsed_ms,
                b.as_lua_key()
            );
        }
        InputEvent::GamepadAxis { axis, value } => {
            // Spam-prone: log only when |value| > 0.25 to skip
            // dead-zone jitter.
            if value.abs() > 0.25 {
                println!(
                    "[{:>6}ms] gamepad axis {} = {:+.2}",
                    elapsed_ms,
                    axis.as_lua_key(),
                    value
                );
            }
        }
        InputEvent::Pencil(_) => {
            // No iPad shell yet; pencil events can't originate here.
        }
    }
}
