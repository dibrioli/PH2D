//! ⭐ **A amostra dos valores ao vivo** (plano 13 Onda D + plano 14 Q3): a 2,5 Hz ([`ghosts_due`]) a
//! mesma leitura dá o texto no fim de cada linha ([`Ghost`]) e as linhas da tabela «Live values»
//! com o traço dos últimos [`LIVE_WINDOW_S`] segundos.

use crate::debug_marks::{Ghost, ghosts_due};
use ph2d_code_debugcard::{LIVE_WINDOW_S, LineLive, LiveRow, live_rows};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::Duration;
use ph2d_ui_table::Trails;
use std::collections::BTreeMap;

/// Os caracteres de um valor ao vivo no fim da linha (legibilidade; o resto é «…»).
const GHOST_CHARS: usize = 60;

/// As amostras que cabem na janela do traço (4 s a 2,5 Hz = 10).
#[must_use]
pub fn window_samples() -> usize {
    (LIVE_WINDOW_S as f32 * 1000.0 / Duration::Slow.ms()).round() as usize // LITERAL-PX-OK: ms por segundo
}

/// O ritmo da amostra escrito («2.5×/s», a faixa do Live).
#[must_use]
pub fn rate_text() -> String {
    let rate = 1000.0 / Duration::Slow.ms(); // LITERAL-PX-OK: ms por segundo
    tr_with("code.debug.live_rate", &[("rate", &rate)])
}

/// ⭐ Os valores ao vivo de um script: o que se pinta no fim das linhas e na tabela.
#[derive(Clone, Debug)]
pub struct LiveSampler {
    pub ghosts: Vec<Ghost>,
    pub rows: Vec<LiveRow>,
    /// Quando se amostrou (ns do relógio da aba).
    sampled: Option<u128>,
    trails: Trails,
}

impl Default for LiveSampler {
    fn default() -> Self {
        Self {
            ghosts: Vec::new(),
            rows: Vec::new(),
            sampled: None,
            trails: Trails::new(window_samples()),
        }
    }
}

impl LiveSampler {
    /// É hora de amostrar? Sem nada mostrado, já.
    #[must_use]
    pub fn due(&self, now_ns: u128) -> bool {
        self.ghosts.is_empty() || ghosts_due(self.sampled, now_ns)
    }

    /// Uma amostra de `live` (o que o motor relata por linha, 1-based).
    pub fn take(&mut self, now_ns: u128, live: &BTreeMap<u32, LineLive>) {
        self.ghosts = live
            .iter()
            .filter(|(n, _)| **n >= 1)
            .map(|(n, l)| Ghost {
                line: *n as usize - 1,
                text: ghost_text(l),
            })
            .filter(|g| !g.text.is_empty())
            .collect();
        self.rows = live_rows(live, &mut self.trails);
        self.sampled = Some(now_ns);
    }
}

/// «x = 412, step = 1  ×3  printed 2×» de uma linha.
#[must_use]
pub fn ghost_text(live: &LineLive) -> String {
    let mut parts: Vec<String> = Vec::new();
    let vals: Vec<String> = live
        .values
        .iter()
        .map(|v| format!("{} = {}", v.name, v.value))
        .collect();
    if !vals.is_empty() {
        parts.push(vals.join(", "));
    }
    if live.hits > 1 {
        parts.push(tr_with("code.debug.hits", &[("count", &live.hits)]));
    }
    if live.prints > 0 {
        parts.push(tr_with("code.debug.printed", &[("count", &live.prints)]));
    }
    let s = parts.join("  ");
    if s.chars().count() > GHOST_CHARS {
        let mut cut: String = s.chars().take(GHOST_CHARS).collect();
        cut.push_str(tr("code.fold.ellipsis"));
        cut
    } else {
        s
    }
}
