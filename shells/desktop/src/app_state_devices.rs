//! **Os dispositivos da `App`** — abrir o comando (gilrs) e o áudio no arranque, e bombear o comando
//! e o `InputState` para o script a cada quadro. Saiu do `main.rs` pelo tecto de LOC dele (as 618
//! linhas de `mod` e aliases da raiz não saem sem mudar o caminho de cada módulo; sai o resto).
//!
//! Corte mecânico, verbatim — com as duas adaptações que o corte obriga: a abertura do comando
//! devolve o `match` em vez de o guardar num `let` (o clippy acusaria `let_and_return`), e os quatro
//! métodos ganham `pub(crate)` (fora da raiz, um método privado deixa de ser visível aos chamadores
//! dele — o `render_loop`, o `input_dispatch` e a ponte vetorial).

use super::*;
use crate::gilrs_adapter;
use crate::input_log::log_input_event;
use ph2d_host::Modifiers;

/// **Abre o comando** e lista os que já estão ligados; sem ele, o app segue sem comando.
pub(crate) fn init_gamepads() -> Option<gilrs::Gilrs> {
    match gilrs::Gilrs::new() {
        Ok(g) => {
            let pads: Vec<String> = g
                .gamepads()
                .map(|(id, pad)| format!("[{:?}] {}", id, pad.name()))
                .collect();
            if pads.is_empty() {
                println!("gilrs: initialized; no gamepads connected yet");
            } else {
                println!("gilrs: detected {} gamepad(s):", pads.len());
                for p in &pads {
                    println!("  {p}");
                }
            }
            Some(g)
        }
        Err(e) => {
            eprintln!("gilrs init failed (continuing without gamepad): {e}");
            None
        }
    }
}

/// **Abre o dispositivo de áudio** (`None` = o app corre em silêncio) e arma as cenas de smoke dele.
pub(crate) fn init_audio() -> Option<ph2d_app_audio::AudioSystem> {
    // Phase 2.1/2.2: open the audio device (None = run silent). As cenas de smoke do áudio são
    // lidas DENTRO da família (`ph2d_app_audio::smoke`, auditoria de arquitectura A1): o `FAMILY`
    // dela declara-as ao registo, e é a crate que as lê.
    let mut audio = ph2d_app_audio::AudioSystem::new();
    if let Some(a) = audio.as_mut() {
        a.stage_armed_smokes();
    }
    audio
}

impl App {
    /// Pump every queued gilrs event into the [`InputState`] and log
    /// the salient ones. Press / release / axis-change all logged at
    /// elapsed-ms timestamps so behavior is auditable from the
    /// terminal without an explicit debug overlay.
    pub(crate) fn pump_gamepad(&mut self) {
        let Some(g) = self.gilrs.as_mut() else {
            return;
        };
        // begin_frame snapshots last-frame held buttons so
        // pressed()/released() return correct edge-trigger values.
        self.input.begin_frame();
        while let Some(gilrs::Event { event, .. }) = g.next_event() {
            match event {
                gilrs::EventType::Connected => {
                    println!("[{:>6}ms] gamepad connected", self.handler.elapsed_ms());
                }
                gilrs::EventType::Disconnected => {
                    println!("[{:>6}ms] gamepad disconnected", self.handler.elapsed_ms());
                }
                _ => {
                    if let Some(translated) = gilrs_adapter::translate(event) {
                        self.input.apply_event(translated);
                        log_input_event(self.handler.elapsed_ms(), &translated);
                    }
                }
            }
        }
    }

    pub(crate) fn convert_modifiers(state: ModifiersState) -> Modifiers {
        Modifiers {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        }
    }

    pub(crate) fn timestamp_ns() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }

    /// Snapshot `InputState` into the ScriptHost's `ph2d.input` table
    /// so Luau can read held buttons / axis values via the canonical
    /// `gamepad.held.<button>` and `gamepad.axis.<axis>` keys.
    /// Cleared and rebuilt every frame — keys absent from this frame
    /// resolve to `nil` on the Luau side (per the M8 ph2d_input
    /// resolves test).
    pub(crate) fn push_input_to_script(&self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let Some(host) = gfx.script.as_ref() else {
            return;
        };
        host.clear_input();
        for button in self.input.gamepad.iter_held() {
            let key = format!("gamepad.held.{}", button.as_lua_key());
            host.provide_input(&key, 1.0);
        }
        for (axis, value) in self.input.gamepad.iter_axes() {
            let key = format!("gamepad.axis.{}", axis.as_lua_key());
            host.provide_input(&key, value as f64);
        }
    }
}
