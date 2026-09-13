//! Os ids da seção **Component** do Inspector (ADR-0164 / F5).

// ⛔ **Havia um `INSP_INSTANCE_SECTION` e ele SAIU** (Enio, 2026-08-27): a superfície deixou de ser
// uma seção e passou a ser um CARTÃO no topo, que não tem cabeçalho, nem recolher, nem âncora de
// nota — logo não tem o que registar. *Um id que nada regista é um id que nada resolve.*

/// Quantos valores por eixo. ⚠️ Mesmo teto e mesma razão do irmão do vetor.
pub const MAX_INSTANCE_AXIS_VALUES: usize = 8;
