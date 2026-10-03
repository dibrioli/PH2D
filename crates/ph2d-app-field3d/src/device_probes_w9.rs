//! ⏱️⭐⭐⭐ **AS SONDAS DA `W9`** — o que MEDE as curas da wave, separado do que AFIRMA.
//!
//! ⚠️ *O [`super`] **afirma** propriedades (com barras, controlos e mutações a matá-las) e isto
//! **mede** (imprime tabelas e não afirma quase nada).* As sondas que viviam neste ficheiro mediam
//! a compilação e a fita do pintor de material do Render traçado, e saíram com ele em 03/10; ficam
//! as da MARCHA, nos irmãos.

use super::*;

/// ⏱️⭐⭐⭐⭐ **O tecto da wave do TORNO** — ver o cabeçalho do [`torno`].
#[path = "device_probes_w9_torno.rs"]
mod torno;

/// ⏱️ **O modo de omissão (o matcap): o arrasto, o campo contra a marcha e o quadro nos dois
/// motores** — ver o cabeçalho do [`omissao`].
#[path = "device_probes_w9_omissao.rs"]
mod omissao;

/// ⏱️⭐⭐⭐⭐ **A grade assada contra a árvore** — ver o cabeçalho do [`grade`].
#[path = "device_probes_w9_grade.rs"]
mod grade;

/// ⏱️⭐⭐⭐⭐ **Onde os passos da marcha acontecem** — ver o cabeçalho do [`perto`].
#[path = "device_probes_w9_perto.rs"]
mod perto;
