//! ⭐ **Os valores ao vivo** (plano 14 Q3): uma linha por variável que uma linha do script mudou;
//! o traço e o «mudou» vêm dos anéis ([`Trails`]). As colunas pinta-as o `ph2d-panel-code`.

use crate::VarRow;
use ph2d_script::{LineLive, ValueKind};
use ph2d_ui_table::Trails;
use std::collections::BTreeMap;

/// A janela do traço, em segundos (a coluna «Last 4 s»).
pub const LIVE_WINDOW_S: u32 = 4;

/// Uma variável ao vivo: a linha (1-based) que a mudou e o traço (só números).
#[derive(Clone, Debug, PartialEq)]
pub struct LiveRow {
    pub line: u32,
    /// A chave é `linha:nome`.
    pub var: VarRow,
    pub trail: Option<Vec<f32>>,
}

/// ⭐ Uma amostra dos valores ao vivo: entra nos anéis e sai em linhas.
pub fn live_rows(live: &BTreeMap<u32, LineLive>, trails: &mut Trails) -> Vec<LiveRow> {
    let vars = live
        .iter()
        .flat_map(|(n, l)| l.values.iter().map(|v| (*n, v)));
    let mut rows: Vec<LiveRow> = vars
        .map(|(line, v)| LiveRow {
            line,
            var: VarRow {
                key: format!("{line}:{}", v.name),
                name: v.name.clone(),
                depth: 0,
                value: v.value.clone(),
                kind: v.kind,
                field_count: v.field_count,
                expandable: false,
                old: None,
            },
            trail: None,
        })
        .collect();
    trails.sample(rows.iter().map(|r| {
        let n = (r.var.kind == ValueKind::Number).then(|| r.var.value.parse().ok());
        (r.var.key.clone(), r.var.value.clone(), n.flatten())
    }));
    for r in &mut rows {
        r.var.old = trails
            .old(&r.var.key)
            .filter(|_| r.var.kind != ValueKind::Table)
            .map(str::to_owned);
        r.trail = trails.samples(&r.var.key);
    }
    rows
}
