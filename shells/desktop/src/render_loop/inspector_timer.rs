//! ⭐⭐⭐ **A secção TIMERS** (TOP-20 #2, W3) — o snapshot que a secção lê e o commit que ela
//! escreve. Irmão do [`crate::render_loop::inspector_anim`], pela mesma razão dele.
//!
//! # ⚠️ A conversão da DURAÇÃO mora aqui, nas duas pontas
//!
//! O motor guarda `u64` de **microssegundos** porque o tique é de passo fixo e o replay tem de o
//! reproduzir; o artista escreve **segundos**. O snapshot divide na saída e o commit multiplica na
//! entrada, **no mesmo ficheiro** — duplicar a conversão faria o número que ele lê e o que o motor
//! aplica divergirem no dia em que uma metade fosse corrigida. É a mesma lei que a §11 escreveu
//! para a velocidade `Q16.16` e a §12 para os pixels.
//!
//! # ⚠️ O snapshot só existe para quem TEM `Timers`
//!
//! É o ADR-0166: *o Inspector mostra o que o objecto TEM, e um componente anexa-se pela paleta*.
//! Devolver `Some` para toda a gente poria uma secção vazia em todo objecto do projecto, e um
//! segundo caminho de anexação ao lado da paleta.
//!
//! # ⚠️ O relógio VIVO não passa por aqui, em nenhum sentido
//!
//! O `TimerRuntime` não é componente registado (é isso que impede um passo de undo por quadro), e
//! nada neste ficheiro lhe toca: a reconciliação do quadro seguinte trata do comprimento e do
//! `autostart` de um slot que acabou de nascer. *Escrever o relógio aqui seria a segunda resposta
//! à pergunta que a [`ph2d_ecs::timer_reconcile`] já responde.*

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{Entity, SimWorld, TIMER_MAX_US, TIMERS_MAX, Timer, Timers, World};
use ph2d_editor::{InspectorTimerInfo, InspectorTimerRow, TimerFieldEdit, Toast};

use crate::render_loop::inspector_ordering::queue_set;

const TIMERS: &str = "ph2d::ecs::Timers";

/// Um segundo em microssegundos — a unidade da conversão, escrita uma vez.
const US_PER_S: f32 = 1_000_000.0;

/// O snapshot da secção, ou `None` quando o objecto não tem `Timers`.
pub(super) fn build_timer_info(
    world: &World,
    entity_bits: u64,
    selected_count: usize,
) -> Option<InspectorTimerInfo> {
    let entity = Entity::from_bits(entity_bits);
    let timers = world.get::<Timers>(entity)?;
    let rows = timers
        .0
        .iter()
        .map(|t| InspectorTimerRow {
            name: t.name.clone(),
            #[allow(clippy::cast_precision_loss)]
            duration_s: t.duration_us as f32 / US_PER_S,
            repeat: t.repeat,
            autostart: t.autostart,
            signal: t.signal.clone(),
        })
        .collect();
    Some(InspectorTimerInfo {
        entity_bits,
        rows,
        selected_count,
    })
}

/// O próximo nome livre — `Timer 1`, `Timer 2`, …
///
/// ⚠️ **Ele conta a partir de um, e salta o que já existe.** Um nome repetido não parte nada (o
/// índice é a identidade), mas uma lista com dois *«Timer 1»* é uma lista que o artista não
/// consegue ler.
fn next_free_name(timers: &Timers) -> String {
    for n in 1..=(TIMERS_MAX + 1) {
        let candidate = format!("Timer {n}");
        if !timers.0.iter().any(|t| t.name == candidate) {
            return candidate;
        }
    }
    String::from("Timer")
}

/// Aplica uma [`TimerFieldEdit`]. Devolve um aviso quando a edição foi **recusada**.
///
/// ⚠️ **Ler-modificar-escrever sobre o componente inteiro**, como a §11: o `Timers` é um vector e
/// o comando do editor põe o componente todo. Reconstruí-lo do snapshot perderia o que outra
/// edição do mesmo quadro escreveu.
pub(super) fn apply_timer_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: &TimerFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Toast> {
    let entity = Entity::from_bits(entity_bits);
    let mut timers = sim.world().get::<Timers>(entity).cloned()?;

    match edit {
        TimerFieldEdit::Add => {
            // ⚠️ **Recusa com aviso**, nunca em silêncio: um `+` que não faz nada lê-se como um
            // botão partido, e o artista carrega nele outra vez.
            if timers.0.len() >= TIMERS_MAX {
                return Some(Toast::warning(format!(
                    "This object already has the maximum of {TIMERS_MAX} timers."
                )));
            }
            let name = next_free_name(&timers);
            // ⚠️ **O `Default` do motor, com o nome por cima** — ele é quem sabe que a duração de
            // partida é UM segundo e não zero (zero é o valor que nunca dispara).
            timers.0.push(Timer {
                name,
                ..Timer::default()
            });
        }
        TimerFieldEdit::Remove(i) => {
            let i = usize::from(*i);
            if i >= timers.0.len() {
                return None;
            }
            timers.0.remove(i);
        }
        TimerFieldEdit::Rename(i, name) => {
            let t = timers.0.get_mut(usize::from(*i))?;
            let trimmed = name.trim();
            // ⚠️ **Um nome vazio é RECUSADO com voz.** A lista escolhe-se por nome; uma linha em
            // branco é uma linha que não se consegue apontar.
            if trimmed.is_empty() {
                return Some(Toast::warning(
                    "A timer needs a name — the list is how you pick one.",
                ));
            }
            t.name = trimmed.to_string();
        }
        TimerFieldEdit::DurationSecs(i, secs) => {
            let t = timers.0.get_mut(usize::from(*i))?;
            // ⚠️ **A saturação é do MOTOR** (`TIMER_MAX_US`), e mora aqui porque é aqui que a
            // unidade muda. Um valor acima do teto entra saturado em vez de dar a volta ao `u64`.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let us = (secs.max(0.0) * US_PER_S).round() as u64;
            t.duration_us = us.min(TIMER_MAX_US);
        }
        TimerFieldEdit::Repeat(i, on) => {
            let t = timers.0.get_mut(usize::from(*i))?;
            t.repeat = *on;
        }
        TimerFieldEdit::Autostart(i, on) => {
            let t = timers.0.get_mut(usize::from(*i))?;
            t.autostart = *on;
        }
        TimerFieldEdit::Signal(i, name) => {
            let t = timers.0.get_mut(usize::from(*i))?;
            // ⚠️ **Vazio é legítimo e não se recusa** — é a lei da §11: um produtor sem nome cumpre
            // o período e cala-se. Quem avisa que ele está mudo é o painel, que o mostra.
            t.signal = name.trim().to_string();
        }
    }
    queue_set(queue, registry, entity_bits, TIMERS, &timers);
    None
}

/// Os gates desta secção — módulo irmão, pelo teto de 600 LOC da shell.
#[cfg(test)]
#[path = "inspector_timer_tests.rs"]
mod tests;
