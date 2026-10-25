//! A forma GRAVADA do [`crate::VecFilter`] até ao `PROJECT_SCHEMA` 195 (sem ids) — só para o degrau
//! `195 → 196` ([`crate::item_id::migra`], F31 §B2). ⛔ Congelada: não acompanhe o tipo vivo.

use serde::Deserialize;

use crate::{FxOp, VecFilter};

#[derive(Deserialize)]
pub(crate) struct VecFilterV195 {
    ops: Vec<FxOp>,
}

impl From<VecFilterV195> for VecFilter {
    fn from(v: VecFilterV195) -> Self {
        Self::from_ops(v.ops)
    }
}
