//! A metade `bridge` da família `physics`, vinda de `shells/desktop/src/render_loop/`
//! (W2/L2 Fase B). ⚠️ O prefixo saiu dos nomes: dentro desta crate tudo é a física.

/// ⭐ **O condutor da ponte ECS↔rapier** (ADR-0131 W1), vindo de
/// `shells/desktop/src/physics/bridge.rs` (W2/L2 Fase C).
///
/// ⚠️ **O nome mudou de `bridge` para `bridge::dispatch`** — e foi a única
/// mudança de nome que esta fase fez por colisão de PASTA e não de ficheiro:
/// aqui já existia um módulo `bridge`, que é esta pasta.
pub mod dispatch;
