//! A metade `inspector` da família `physics`, vinda de `shells/desktop/src/render_loop/`
//! (W2/L2 Fase B). ⚠️ O prefixo saiu dos nomes: dentro desta crate tudo é a física.

/// A §11 — o corpo rígido, o colisor, as zonas. Vinda de
/// `shells/desktop/src/physics/inspector_body.rs` (W2/L2 Fase C): ela é a irmã
/// da [`player`] por assunto, e morava noutra pasta só por ordem de chegada.
pub mod body;
pub mod player;
