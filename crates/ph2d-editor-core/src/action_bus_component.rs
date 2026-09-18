//! ⭐⭐⭐ **O que uma secção de COMPONENTE DE JOGO pede** — onze formas de uma família só.
//!
//! # ⚠️ Por que elas saíram do [`super`] (2026-09-18)
//!
//! Elas partilham a **forma** (`{ entity_bits, edit }`, letra por letra), o **sujeito** (a entidade
//! PRIMÁRIA — nenhuma delas se espalha sobre a BulkSelect, e as onze dizem-no por escrito), o
//! **dreno** (`shells/desktop/src/render_loop/fase_bus_inspector.rs`, que as empilha uma a uma) e a
//! **origem** (uma secção do Inspector). Escritas como onze variantes soltas, elas eram onze das
//! ~150 do [`super::EditorAction`] — e foram elas que puseram aquele ficheiro a `709/700`.
//!
//! ⭐⭐ **É o MESMO corte que o irmão [`super::hier`] já pagou**, uma família adiante, e o cabeçalho
//! dele escreve a lei: *quando N variantes têm a mesma forma, a forma é que é o dado*. ⛔ E a cura
//! de um tecto é o CORTE, nunca uma entrada nova no `FILE_OVERAGE_OK`.
//!
//! # ⚠️⚠️ A fronteira da família é MEDIDA, não é o calendário
//!
//! O `EditorAction` tem **27** variantes com esta forma exacta. As onze que vivem aqui são as que
//! carregam um vocabulário de `crate::<x>_edits` — o módulo abaixo do `action_bus` que a **catraca
//! do DAG** obriga a existir. As outras dezasseis carregam um `crate::screens::hero::*FieldEdit` e
//! **ficam onde estão**: elas são edições do modelo de SPRITE e de AUTORIA (moldura, fatia, âncora,
//! amostragem, ordem, mistura, física, junta, roda, jogador), e o que as distingue não é a data —
//! é o assunto.
//!
//! ⇒ *uma variante nova cujo vocabulário já viva num `<x>_edits` entra AQUI, e o `action_bus` não
//! cresce mais por isso.*

/// Ver o cabeçalho do módulo. ⚠️ **Uma entrada por SECÇÃO**, e o payload é o vocabulário dela.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentEdit {
    /// **TAGS** (TOP-20 #9) — marcar, desmarcar, ou criar uma tag e marcar.
    Tags(crate::tags_edits::TagsFieldEdit),
    /// **FACTORY / LIFECYCLE** (TOP-20 #11 e #12).
    Factory(crate::factory_edits::FactoryFieldEdit),
    /// **TOP-DOWN PLAYER** (TOP-20 #13).
    TopDown(crate::topdown_edits::TopDownFieldEdit),
    /// **PROJECTILE MOTION** (TOP-20 #14).
    Projectile(crate::projectile_edits::ProjectileFieldEdit),
    /// **STATE MACHINE** (TOP-20 #15).
    StateMachine(crate::statemachine_edits::StateMachineFieldEdit),
    /// **SCRIPT** (TOP-20 #16) — um número declarado pelo `.luau`.
    Script(crate::script_edits::ScriptFieldEdit),
    /// **PARTICLES** (TOP-20 #18).
    Particles(crate::particles_edits::ParticlesFieldEdit),
    /// **HUD** (TOP-20 #20) — canvas, rótulo ou botão.
    Hud(crate::hud_edits::HudFieldEdit),
    /// **SEQUENCE** (o #19) — a cutscene.
    Sequence(crate::sequence_edits::SequenceFieldEdit),
    /// **COUNTER WATCH** (o #17) — a vigia de um contador.
    CounterWatch(crate::counter_watch_edits::CounterWatchFieldEdit),
    /// **GATILHO** (o suplente #24) — a mão de quem joga.
    ActionTrigger(crate::action_trigger_edits::ActionTriggerFieldEdit),
}
