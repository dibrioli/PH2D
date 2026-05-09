//! Frame-budget bench for [`ph2d_input::InputState::apply_event`].
//!
//! M8 plan: "Frame budget bench rodando em CI pela primeira vez."
//! This bench measures the cost of pumping a representative
//! per-frame input batch (10 button events + 4 axis events + 1 pencil
//! event = 15 events). At a 60 Hz target and 16.7 ms budget, even
//! 1 ms spent in input dispatch would be 6 % of frame — alarm
//! threshold. Realistic measurements should be in single-digit µs.
//!
//! Bench is wired into the CI bench job in `.github/workflows/spike.yml`.

use criterion::{Criterion, criterion_group, criterion_main};
use ph2d_input::{Event, GamepadAxis, GamepadButton, InputState, PencilEvent};
use std::hint::black_box;

fn realistic_frame_batch() -> Vec<Event> {
    vec![
        Event::GamepadButtonDown(GamepadButton::South),
        Event::GamepadButtonUp(GamepadButton::South),
        Event::GamepadButtonDown(GamepadButton::DPadLeft),
        Event::GamepadButtonDown(GamepadButton::DPadRight),
        Event::GamepadButtonUp(GamepadButton::DPadLeft),
        Event::GamepadButtonUp(GamepadButton::DPadRight),
        Event::GamepadButtonDown(GamepadButton::LeftBumper),
        Event::GamepadButtonUp(GamepadButton::LeftBumper),
        Event::GamepadButtonDown(GamepadButton::Start),
        Event::GamepadButtonUp(GamepadButton::Start),
        Event::GamepadAxis {
            axis: GamepadAxis::LeftStickX,
            value: 0.42,
        },
        Event::GamepadAxis {
            axis: GamepadAxis::LeftStickY,
            value: -0.17,
        },
        Event::GamepadAxis {
            axis: GamepadAxis::RightStickX,
            value: 0.0,
        },
        Event::GamepadAxis {
            axis: GamepadAxis::LeftTrigger,
            value: 0.83,
        },
        Event::Pencil(PencilEvent::Squeeze { force: 0.25 }),
    ]
}

fn bench_apply_event(c: &mut Criterion) {
    let batch = realistic_frame_batch();
    c.bench_function("input_state_apply_15_event_frame", |b| {
        let mut state = InputState::new();
        b.iter(|| {
            state.begin_frame();
            for ev in batch.iter().copied() {
                state.apply_event(black_box(ev));
            }
            // Read every button/axis to keep the optimizer honest.
            for btn in state.gamepad.iter_held() {
                black_box(state.gamepad.held(btn));
            }
            for (axis, _v) in state.gamepad.iter_axes() {
                black_box(state.gamepad.axis(axis));
            }
        });
    });
}

criterion_group!(benches, bench_apply_event);
criterion_main!(benches);
