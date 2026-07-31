//! **Quais ids são um `Click` PURO** — irmão do [`super`] pelo teto de 600 LOC do painel.
//!
//! O corte é por RESPONSABILIDADE: aqui mora UMA pergunta ("o painel só encaminha este clique, ou
//! ele significa alguma coisa aqui?"), e a resposta é uma lista que **só cresce** — cada seção
//! nova acrescenta os botões dela. Deixá-la dentro do `apply_event` foi o que estourou primeiro o
//! teto de 200 LOC por função, e mantê-la no `event.rs` foi o que estourou o de 600 do arquivo.
//!
//! ⚠️ Um id **ausente** desta lista pinta, é clicável e está MORTO: o `Click` nunca vira
//! `PanelEvent::Click` no bus, e o drain da shell nunca corre. É o modo de falha que cada seam
//! gate deste painel existe para pegar.

use super::filters;
use super::is_fx_button;
use crate::ids;

/// Ids dos botões cujo `Click` o painel apenas ENCAMINHA (`PanelEvent::Click`) para o
/// shell/tool aplicar (modos, Cap/Join, Vertex, Boolean, Arrange, Fill kind, Align/
/// Distribute, alinhamento de texto…). Extraído de `apply_event` para caber no teto de
/// 200 LOC por função dos painéis — a lista só cresce.
pub(super) fn forwards_plain_click(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_MODE_SELECT
        || id == ids::VECTOR_MODE_NODE
        || id == ids::VECTOR_MODE_PEN
        // **O lápis** (o 11º modo). Fora daqui o chip pinta, ACENDE sob o mouse e o Click
        // morre no painel — nunca vira `ToolPanelEvent`.
        || id == ids::VECTOR_MODE_PENCIL
        // **A FONTE da largura do lápis** (W1d) — três chips exclusivos. Fora daqui eles pintam,
        // acendem sob o mouse e o Click morre no painel: o artista escolheria "Speed" e todo
        // traço continuaria uniforme, sem nada dizer por quê.
        || id == ids::VECTOR_PENCIL_W_UNIFORM
        || id == ids::VECTOR_PENCIL_W_SPEED
        || id == ids::VECTOR_PENCIL_W_PRESSURE
        // O 5º pill: re-arma o gesto de forma. Ausente daqui, o botão pintaria e estaria
        // MORTO — exatamente o bug que o smoke do Line/Arc pegou (Enio 2026-07-09).
        || id == ids::VECTOR_MODE_SHAPE
        || id == ids::VECTOR_MODE_TEXT
        // O 6º pill (Connect). Fora daqui, o botão pinta e está MORTO — é o gate do
        // seam (`clicking_connect_pill_reaches_the_tool`).
        || id == ids::VECTOR_MODE_CONNECT
        // O 7º pill (Build / Shape Builder).
        || id == ids::VECTOR_MODE_BUILD
        // O 8º pill (Pick Shapes / Blend). Fora daqui, o pill pinta e está MORTO.
        || id == ids::VECTOR_MODE_PICKBLEND
        // O 9º e 10º pills (Fillet / Chamfer). Fora daqui, pintam e estão MORTOS.
        || id == ids::VECTOR_MODE_FILLET
        || id == ids::VECTOR_MODE_CHAMFER
        // O 12º modo (Width). Fora daqui, o pill pinta e está MORTO.
        || id == ids::VECTOR_MODE_WIDTH
        || id == ids::VECTOR_TEXT_FONT_PREV
        || id == ids::VECTOR_TEXT_FONT_NEXT
        || id == ids::VECTOR_TEXT_FONT_IMPORT
        || id == ids::VECTOR_TEXT_ALIGN_LEFT
        || id == ids::VECTOR_TEXT_ALIGN_CENTER
        || id == ids::VECTOR_TEXT_ALIGN_RIGHT
        || id == ids::VECTOR_CAP_BUTT
        || id == ids::VECTOR_CAP_ROUND
        || id == ids::VECTOR_CAP_SQUARE
        || id == ids::VECTOR_JOIN_MITER
        || id == ids::VECTOR_JOIN_ROUND
        || id == ids::VECTOR_JOIN_BEVEL
        || id == ids::VECTOR_VERT_CORNER
        || id == ids::VECTOR_VERT_SMOOTH
        || id == ids::VECTOR_VERT_SYMMETRIC
        || id == ids::VECTOR_VERT_DELETE
        || id == ids::VECTOR_VERT_SEL_SUBPATH
        || id == ids::VECTOR_VERT_SEL_SAME
        || id == ids::VECTOR_BLEND_RUN
        || id == ids::VECTOR_BLEND_RESET_SPINE
        || id == ids::VECTOR_BLEND_EXPAND
        || id == ids::VECTOR_BLEND_RELEASE
        || id == ids::VECTOR_MORPH_RUN
        || is_fx_button(id)
        // O botão de SEÇÃO "Apply" (assa a pilha de efeitos). Fora daqui pintaria e estaria
        // MORTO — a shell classifica-o via `fx_bridge_dispatch::classify_click`.
        || id == ids::VECTOR_FX_APPLY
        || id == ids::VECTOR_ENVELOPE_RUN
        || id == ids::VECTOR_ENVELOPE_EXPAND
        || id == ids::VECTOR_ENVELOPE_RELEASE
        || id == ids::VECTOR_ENVELOPE_PERSPECTIVE
        || id == ids::VECTOR_ENVELOPE_MESH
        || id == ids::VECTOR_ENVELOPE_PINS
        || id == ids::VECTOR_ENVELOPE_CLEAR_PINS
        // Text on Path: prender / soltar / o lado. Todos mexem no DOCUMENTO (o componente
        // `VecTextPath` da entidade), então atravessam para a shell como os do envelope.
        || id == ids::VECTOR_TEXTPATH_LINK
        // Pick Path: arma o Picker (a shell captura o texto em foco e espera o clique do guia).
        || id == ids::VECTOR_TEXTPATH_PICK
        || id == ids::VECTOR_TEXTPATH_DETACH
        || id == ids::VECTOR_TEXTPATH_FLIP
        || id == ids::VECTOR_TEXTPATH_FLIP_OFF
        // Pattern on Path: prender / soltar / o lado. Todos mexem no DOCUMENTO (o componente
        // `VecPatternPath` da entidade), então atravessam para a shell como os do texto.
        || id == ids::VECTOR_PATTERNPATH_LINK
        // Pick Path: arma o Picker (a shell captura o motivo selecionado e espera o clique do guia).
        || id == ids::VECTOR_PATTERNPATH_PICK
        || id == ids::VECTOR_PATTERNPATH_DETACH
        || id == ids::VECTOR_PATTERNPATH_FLIP
        || id == ids::VECTOR_PATTERNPATH_FLIP_OFF
        // Contour: criar / materializar / apagar + os dois trios exclusivos. ⚠️ Corner e Side
        // atravessam para a shell (≠ os gêmeos da seção Expand, que são panel-local): lá eles
        // armam o PRÓXIMO offset, aqui retunam um contour que já está na tela, e o que a fileira
        // mostra sai do componente. Guardá-los no painel seria uma segunda cópia do mesmo fato,
        // e ela discordaria assim que o artista selecionasse outra forma.
        || id == ids::VECTOR_CONTOUR_ADD
        || id == ids::VECTOR_CONTOUR_EXPAND
        || id == ids::VECTOR_CONTOUR_REMOVE
        || id == ids::VECTOR_CONTOUR_JOIN_MITER
        || id == ids::VECTOR_CONTOUR_JOIN_ROUND
        || id == ids::VECTOR_CONTOUR_JOIN_BEVEL
        || id == ids::VECTOR_CONTOUR_SIDE_OUTER
        || id == ids::VECTOR_CONTOUR_SIDE_INNER
        || id == ids::VECTOR_CONTOUR_SIDE_BOTH
        // Filters (a pilha de FX raster, plano 24): Add / ✕ / ↑ / ↓ / 👁 e a swatch de cor. O
        // drain da shell os traduz em edições do `VecFilter`. Fora daqui pintariam e estariam
        // MORTOS.
        || filters::is_filter_button(id)
        || (0..ids::MAX_ENVELOPE_PRESETS).any(|i| id == ids::vector_envelope_preset_id(i))

        || id == ids::VECTOR_BOOL_UNION
        || id == ids::VECTOR_BOOL_SUBTRACT
        || id == ids::VECTOR_BOOL_INTERSECT
        || id == ids::VECTOR_BOOL_EXCLUDE
        || id == ids::VECTOR_COMPOUND_MAKE
        || id == ids::VECTOR_COMPOUND_RELEASE
        // Expand: os dois COMANDOS (a junção não vem aqui — é panel-local).
        || id == ids::VECTOR_EXPAND_OFFSET_PATH
        || id == ids::VECTOR_EXPAND_OUTLINE_STROKE
        || id == ids::VECTOR_EXPAND_POWER_STROKE
        // Os perfis nomeados (W2b): o clique escreve os quatro sliders E arma o perfil vivo na
        // seleção, e as duas metades são da SHELL (é ela que tem store e cena). Panel-local, como
        // o Side/Corner, deixaria a forma sem perfil e o botão aceso.
        || (0..ids::MAX_WIDTH_PRESETS).any(|i| id == ids::vector_width_preset_id(i))
        || id == ids::VECTOR_FILL_RULE_NONZERO
        || id == ids::VECTOR_FILL_RULE_EVENODD
        || id == ids::VECTOR_SNAP_OFF
        || id == ids::VECTOR_SNAP_ON
        || id == ids::VECTOR_ARRANGE_DUPLICATE
        || id == ids::VECTOR_ARRANGE_TO_BACK
        || id == ids::VECTOR_ARRANGE_BACKWARD
        || id == ids::VECTOR_ARRANGE_FORWARD
        || id == ids::VECTOR_ARRANGE_TO_FRONT
        || id == ids::VECTOR_ARRANGE_FLIP_H
        || id == ids::VECTOR_ARRANGE_FLIP_V
        || id == ids::VECTOR_ARRANGE_ROTATE_CW
        || id == ids::VECTOR_ARRANGE_ROTATE_CCW
        || id == ids::VECTOR_PATH_SMOOTH
        || id == ids::VECTOR_PATH_SHARPEN
        || id == ids::VECTOR_PATH_SIMPLIFY
        || id == ids::VECTOR_PATH_SUBDIVIDE
        || id == ids::VECTOR_PATH_CLOSE
        || id == ids::VECTOR_FILL_KIND_SOLID
        || id == ids::VECTOR_FILL_KIND_LINEAR
        || id == ids::VECTOR_FILL_KIND_RADIAL
        || id == ids::VECTOR_FILL_KIND_MULTI
        || id == ids::VECTOR_GRAD_ADD_POINT
        || id == ids::VECTOR_GRAD_REMOVE_POINT
        || id == ids::VECTOR_GRAD_ADD_STOP
        || id == ids::VECTOR_GRAD_REMOVE_STOP
        || id == ids::VECTOR_ALIGN_LEFT
        || id == ids::VECTOR_ALIGN_HCENTER
        || id == ids::VECTOR_ALIGN_RIGHT
        || id == ids::VECTOR_ALIGN_TOP
        || id == ids::VECTOR_ALIGN_VCENTER
        || id == ids::VECTOR_ALIGN_BOTTOM
        || id == ids::VECTOR_DISTRIBUTE_H
        || id == ids::VECTOR_DISTRIBUTE_V
        || id == ids::VECTOR_PIVOT_EDIT
        || id == ids::VECTOR_CONVERT_TO_CURVES
        // **Both Ends** (a dupla via). Um `Click` puro — e não um `SetValue` com um booleano
        // — porque o estado é DERIVADO das duas pontas: quem o resolve é a tool, que as
        // possui. Fora daqui o botão pintaria e estaria MORTO.
        || id == ids::VECTOR_MARKER_BOTH
}
