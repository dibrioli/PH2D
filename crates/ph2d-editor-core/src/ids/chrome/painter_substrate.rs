//! **SUBSTRATO** NodeIds — o dente do papel como SUPERFÍCIE, para qualquer meio de pintura.
//!
//! Duas rows no fim da seção **Paper**: quanto o dente sobressai (`RELIEF`) e quão íngremes são as
//! paredes dele (`ROUGHNESS`). Os dois são `NumberInput` de arrasto e viajam como `SetValue`
//! (roteados por [`PAINTER_SUBSTRATE_FIELDS`] + `is_param_field`), o padrão dos irmãos de Grain/Paper.
//!
//! ⚠️ **Por que um arquivo PRÓPRIO, e não o `painter_watercolor.rs` onde o resto da seção Paper mora:**
//! o slot de papel nasceu dentro da aquarela por acidente de história — a
//! [doc 19](../../../../../docs/Painter/19_relevo_do_papel_investigacao.md) mede isso e conclui que
//! *"o papel não é da aquarela, é do SUBSTRATO"*. Mover o slot inteiro é a extração com ADR na frente
//! que aquela doc descreve; o que se pode fazer sem ela é **não acrescentar id novo de substrato ao
//! namespace do meio errado**. Estes dois nascem no lugar certo.

use super::{NodeId, hash_node_id};

/// **Relief** — a amplitude do dente, `0` (desligado, o default) a `1`.
///
/// `SetValue` → `PainterTool::set_substrate_depth`. Ligar sem papel escolhido ARMA um papel (o
/// `PaperCold`), senão o interruptor acende e não mostra nada.
pub const PAINTER_SUBSTRATE_RELIEF: NodeId = hash_node_id("painter_brush.substrate_relief");

/// **Roughness** — a ÍNGREMEZA do dente (`0` satinado, `1` áspero e chapado nos extremos).
///
/// `SetValue` → `PainterTool::set_substrate_roughness`. É o *Contrast* do Corel Painter e o
/// *Roughness* do ArtRage: um ganho em torno do meio-tom do grão, **não** a largura de um realce
/// (essa leitura foi medida em zero texels movidos — o ⛔ em `substrate_relief.rs`).
pub const PAINTER_SUBSTRATE_ROUGHNESS: NodeId = hash_node_id("painter_brush.substrate_roughness");

/// ⚠️ **A terceira row NÃO mora aqui, e a ausência é uma decisão do Enio (2026-08-10).** O relevo do
/// PIGMENTO DEPOSITADO nasceu ao lado destas duas — no Wet Paint as duas metades saem do mesmo
/// checkbox `paper_on` — e o smoke o mudou de casa: *"deveria ter nome mais adequado, deveria ter sido
/// colocado na seção de Shape"*. Ele é `PAINTER_SHAPE_RELIEF`, e o que decide é de quem o número é —
/// o dente é do CANVAS, o relevo do depósito é da SILHUETA que o deposita.
///
/// Os `SetValue` do substrato — uma checagem de pertencimento para o forward do painel
/// (`is_param_field`) e para o roteador do tool.
///
/// ⚠️ **Fora do laço do `populate`, e a ausência foi MEDIDA, não assumida.** A primeira versão os
/// registrava lá junto dos irmãos de Grain/Paper, e a prova de mutação (tirar a linha) **sobreviveu**
/// aos quatro gates de seam: `paint_num_row` já chama `register_if_absent` com um `NumberInput` no
/// primeiro quadro em que a row pinta, então o registro antecipado não decide nada — ele só semeia um
/// buffer vazio que o espelho do quadro seguinte reescreve. Uma linha que nenhuma mutação consegue
/// derrubar é uma linha que o próximo leitor vai tratar como load-bearing; ela saiu.
pub const PAINTER_SUBSTRATE_FIELDS: [NodeId; 2] =
    [PAINTER_SUBSTRATE_RELIEF, PAINTER_SUBSTRATE_ROUGHNESS];
