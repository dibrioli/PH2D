//! **A ponte ECS do Flip** — [`entities`] (uma entidade por objecto do `FlipDoc`) e [`transform`]
//! (o afim que leva a arte de um objecto ao mundo, e de volta).
//!
//! ⭐ Nasceu na auditoria de arquitectura de 2026-09-12 (A1), no molde da `ph2d-vec-entities`: a
//! assadura do Motion e a shell precisavam destas leis e, para as ter, dependiam da família Flip
//! inteira. *Uma peça partilhada por duas famílias é uma FOLHA — nunca a casa de uma delas.*
#![forbid(unsafe_code)]

pub mod entities;
pub mod transform;
