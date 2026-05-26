#![forbid(unsafe_code)]

//! Painter cascada W0 — homestead único dos arch-gates de contratos
//! (ADR-0043..0053 ratificados 2026-05-26).
//!
//! Este crate **não exporta código de runtime**. Sua razão de existir é
//! hospedar os arch-gates textuais em `tests/architecture_painter_contract_surface.rs`,
//! que auditam o source dos crates filhos do Painter conforme eles nascem em W1+.
//!
//! ## Por que crate isolado?
//!
//! - **Zero deps de runtime.** Auditoria textual via `std` + `walkdir`; sem importar
//!   tipos dos crates filhos. Permite o gate ativar **antes** dos crates filhos
//!   existirem (em W0/T0.8) e continuar válido conforme eles nascem (W1+).
//! - **Homestead único.** ADRs 0043..0053 espelharam findings de auditoria
//!   adversarial (2026-05-26) — o gate está num só lugar canônico, não espalhado.
//! - **Não-bloqueante para fan-out.** Crates filhos (`ph2d-tool-painter`,
//!   `ph2d-painter-brush`, etc.) podem nascer/morrer/migrar sem tocar este crate.
//!
//! Vide [ADR-0043 §2.4](../../../docs/architecture/decisions/0043-painter-contract.md).
