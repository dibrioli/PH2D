//! **Macros** — gravar os passos do artista e reproduzi-los no tempo da gravação (ADR-0186).
//!
//! Este crate é o núcleo comum e não conhece nenhuma família: guarda trilhas de eventos
//! com tempo ([`MacroDoc`]), lê e escreve o formato versionado ([`codec`]), grava
//! ([`MacroRecorder`]) e despacha por ordem de tempo ([`MacroPlayer`]). A carga de cada evento
//! é bytes codificados pela família dona da trilha (Painter, Motion, …), que também decide a
//! exatidão (a marca de passos da água, o tique da cena).
//!
//! ⛔ Nada aqui lê o relógio de parede: o tempo entra sempre pelo chamador
//! (gate `the_macro_core_never_reads_the_wall_clock`).
//!
//! Plano: `docs/Macros/01_plano_onda1.md`.

pub mod codec;
mod doc;
mod player;
pub mod rec;
mod recordable;
mod recorder;

pub use codec::{CodecError, MAJOR, MINOR};
pub use doc::{Event, MacroDoc, MacroTime, RefRect, Track};
pub use player::MacroPlayer;
pub use recordable::{Door, Recordability, orphans, undecided};
pub use recorder::{MacroRecorder, RecordError, TrackId};
