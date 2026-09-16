//! Layout constants used by every painter in this crate. Ported
//! verbatim from `ph2d_editor_core::grid_snap::panel`.
//!
//! Wave 10 / Etapa 5.1: ROW_H/pad()/row_gap()/LABEL_FONT_SIZE now flow
//! from `ph2d_tokens` (no literal pixels). ⭐ E a coluna do rótulo saiu daqui
//! em 2026-09-14: ela é a porta `property_label_col_w`, perguntada no sítio da
//! pintura com a largura REAL da linha.

use ph2d_tokens::{ROW_H_PX, Spacing};

pub(crate) const ROW_H: f32 = ROW_H_PX;
pub(crate) fn pad() -> f32 {
    Spacing::Lg.px()
}
/// ⛔ **Era a QUINTA cópia da mesma resposta, e a única escondida atrás de uma função** — por
/// isso o censo por `grep` de 2026-09-06 a subestimou. Ela devolvia `Sm` (6 px) enquanto o resto
/// do app avançava 4, e é o que fazia este painel respirar diferente dos vizinhos. Delega.
pub(crate) fn row_gap() -> f32 {
    ph2d_tokens::control_gap_px()
}
// ⛔⛔ **O `LABEL_FONT_SIZE` MORREU em 2026-09-15.** Ele era `TypeToken::Base` — **um px maior**
// que o `Sm` que toda linha de propriedade do app escreve —, e o último sítio a lê-lo era o nome do
// interruptor *Show grid*, ao lado de linhas de número em `Sm`. *Um corpo de letra próprio de um
// painel é a forma mais discreta de ele respirar diferente dos vizinhos*, e este painel já pagou a
// mesma lição no `row_gap` (ver acima). ⇒ quem escreve um nome de linha chama a porta
// (`widget::paint_property_label`), que sabe a fonte.
