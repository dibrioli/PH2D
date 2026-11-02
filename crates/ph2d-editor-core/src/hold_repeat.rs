//! ⭐ **Segurar repete** — o relógio ÚNICO de quem repete enquanto o botão está premido: o stepper
//! de um número (`interaction::dispatch::tick`) e a seta de uma barra de rolagem
//! (`widget::scroll_list::HoldRepeat`). Folha sem dependências, para que os dois a leiam sem que um
//! dependa do outro (`architecture_the_foundation_modules_form_a_dag`).

/// Initial delay (ns) before continuous-hold repeats start firing.
/// `250 ms` — match macOS Aqua text-field stepper feel.
pub const STEPPER_HOLD_INITIAL_DELAY_NS: u128 = 250_000_000;

/// Repeat interval (ns) once the initial delay elapsed. `30 ms` —
/// ~33 ticks per second, a comfortable fast-but-readable rate.
pub const STEPPER_REPEAT_INTERVAL_NS: u128 = 30_000_000;
