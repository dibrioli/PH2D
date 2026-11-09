//! ⭐ **O quadro do depurador** (plano 14, decisão do dono 13): o modelo ([`DebugCard`], a partir do
//! [`ph2d_script::DebugStop`]), o «mudou desde o passo anterior» ([`StepMemory`]), a aparência
//! ([`CardStyle`]) e as linhas da planilha ([`sheet_rows`]). Pinta-o o `ph2d-panel-code::debug_card`.

mod live;
mod model;
mod sheet;
mod style;

pub use live::{LIVE_WINDOW_S, LiveRow, live_rows};
pub use model::*;
pub use ph2d_script::{LineLive, ValueKind, Var};
pub use sheet::{columns, sheet_rows, type_text, value_ink};
pub use style::CardStyle;
