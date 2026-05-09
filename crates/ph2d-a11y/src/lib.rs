#![forbid(unsafe_code)]
//! ph2d-a11y — AccessKit integration (WCAG 2.2 Level AA per ADR-0023).
//!
//! Empty pending **M12** (per **ADR-0023**): wraps AccessKit's
//! cross-platform `Node` tree → Mac VoiceOver + Win Narrator +
//! iPadOS VoiceOver + Linux AT-SPI (best-effort). Every editor
//! widget exports a node; Single-Touch Companion overlay primitive
//! lives here too (multi-touch fallback for touch devices).
