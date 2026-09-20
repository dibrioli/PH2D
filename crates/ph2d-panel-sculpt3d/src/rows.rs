//! **A tabela de knobs — a lista que todo o resto do painel percorre.**
//!
//! Um controle contínuo aparece em quatro lugares: ele é pintado, é registrado
//! (senão o clique é descartado em silêncio), vira um valor quando o artista o
//! arrasta, e é varrido pelo teste de costura. Quatro listas escritas à mão
//! derivam, e a deriva é MUDA — uma row pintada e não registrada está morta sob
//! o mouse.
//!
//! Então há UMA lista. `paint`, `populate`, `event` e o `tests/seam.rs` percorrem
//! [`ROWS`]; acrescentar um knob é acrescentar uma linha, e ele nasce pintado,
//! registrado, vivo e varrido.

use ph2d_sculpt3d::Verb;

use crate::state::UiLevel;

/// O teto do raio que o SLIDER oferece, em pixels de tela — e a pista é CÚBICA ([`Row::curva`]).
///
/// ⚠️ **Ele não é o teto do produto:** quem aperta é a DIAGONAL DA VISTA (`radius_ceiling_px`, na
/// família: 2 203 px a 1920×1080, 2 779 px a 2560×1080), e este tem de a alcançar — `5000` passa a
/// diagonal de um ecrã 4K (4 406 px) e o digitável do alvo (5 000 px de raio). Numa janela menor o valor
/// **volta** ao encostar no teto real, porque o retrato que o painel pinta é o número JÁ CLAMPADO
/// que o dab usa. Uma pista que anda além do teto é honesta; uma que mostra um número que o pincel
/// não usa não é. ⛔ Era `200`, com o teto em `1/8` da altura, e o dono não chegava à peça (16/09).
const RADIUS_TRACK_MAX_PX: f32 = 5000.0; // LITERAL-PX-OK: extensao da PISTA, nao metrica de design (o teto real e a diagonal da vista)

#[path = "rows_types.rs"]
mod types;

/// Os knobs da LEITURA da forma — ver o doc do módulo.
#[path = "rows_shading.rs"]
mod shading;

/// Os knobs da RESOLUÇÃO da malha — ver o doc do módulo.
#[path = "rows_topology.rs"]
mod topology;
pub use topology::TOPOLOGY;

/// Os cinco números do pincel de TECIDO — ver o doc do módulo.
#[path = "rows_cloth.rs"]
mod cloth;
/// Os cinco números do PINCEL DE PLANO — ver [`plano`].
/// **OS DOIS KNOBS DO HC** — ver [`hc`]. Irmão pelo mesmo corte de ASSUNTO dos
/// vizinhos, e ele foi FORÇADO pelo teto de LOC deste ficheiro quando o pincel
/// afiado entrou: a tabela cresce uma linha por pincel e a prosa de cada knob
/// cresce um parágrafo, logo o que sai são os GRUPOS que já se lêem sozinhos.
#[path = "rows_hc.rs"]
mod hc;

#[path = "rows_plano.rs"]
mod plano;
/// Os três números do pincel de POSE — ver [`pose`].
#[path = "rows_pose.rs"]
mod pose;

/// O número do pincel de CONTORNO — ver [`boundary`].
#[path = "rows_boundary.rs"]
mod boundary;

/// ⭐ Os quatro números do FILTRO de tecido, e a *Quality* do pincel — ver o doc
/// do módulo. ⚠️ **Irmão do [`cloth`] por SUJEITO**: os números do filtro não são
/// os do pincel, e a pergunta de visibilidade deles é outra.
#[path = "rows_cloth_filter.rs"]
mod cloth_filter;

/// Os dois números do EXTRACT — ver o doc do módulo.
#[path = "rows_extract.rs"]
mod extract;

/// As perguntas que o PADRÃO faz — ver o doc do módulo.
#[path = "rows_alpha.rs"]
mod alpha;

/// **QUAIS SEÇÕES existem e que CABEÇALHOS elas têm** — ver o doc do módulo.
///
/// ⚠️ Nenhum caminho de chamador muda: `rows::SECTIONS`, `rows::rows()`,
/// `rows::row_for()` e `rows::section_headers()` continuam onde estavam.
#[path = "rows_sections.rs"]
mod sections;
pub use sections::{BUTTON_SECTIONS, SECTIONS, row_for, rows, section_headers};

pub use types::{Place, Row, Section};

/// O teto da pista de **Extract Smooth**, em passadas.
///
/// ⚠️ **OITO, e o número é MEDIDO** (`ph2d-mesh/tests/it/measure_extract.rs`): o
/// relaxamento da costura **CONVERGE**, e o que ele compra por passada cai
/// rápido. Numa costura serrilhada — a que uma mão pintada deixa — a rugosidade
/// da beira vai de **0,09369 a 0,05117 em oito passadas (−45%)**, e da oitava em
/// diante cada uma compra **0,4%**. Uma pista mais longa seria uma faixa onde
/// arrastar não faz nada, que é o controle morto que esta casa varre a cada wave.
const MAX_EXTRACT_SMOOTH: f32 = 8.0; // LITERAL-PX-OK: contagem de passadas MEDIDA, nao metrica de design

/// **AS PERGUNTAS QUE UMA FILEIRA FAZ** — ver [`show`]; o corte foi forçado
/// pelo tecto de LOC e o cabeçalho dele diz porquê.
#[path = "rows_show.rs"]
mod show;
pub(super) use show::{always, penteia, shapes_the_distance, suaviza_o_traco, tem_raio};

/// **A TABELA DO PINCEL** — ver [`brush`]. Irmã (`#[path]`) das outras seis, e
/// o corte foi forçado pelo tecto de LOC deste painel.
#[path = "rows_brush.rs"]
mod brush;
pub(crate) use brush::BRUSH;
