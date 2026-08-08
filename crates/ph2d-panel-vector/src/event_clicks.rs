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
/// **Os sete widgets da MOLDURA** (plano UI/UX W0) — o pill do 14º modo, os dois chips de recorte
/// e os quatro presets de dispositivo. Um helper, e não sete linhas na cadeia, pelo mesmo motivo
/// que o `filters::is_filter_button`.
///
/// Fora daqui os sete pintam, acendem sob o mouse e **não fazem nada**: o pill nunca troca o modo,
/// os chips nunca alcançam o componente e os presets nunca escrevem W/H.
///
/// ⚠️ **A cadeia abaixo está EXACTAMENTE no teto de 200 LOC** depois desta wave. A próxima adição
/// não cabe: ela tem de extrair uma FAMÍLIA (os catorze pills de modo são a candidata óbvia) —
/// e isso é trabalho de quem for dono da cadeia, não contrabando dentro de outra feature.
/// **As opções dos dois pickers de TOKEN** (plano UI/UX W4). Um helper, e não 162 linhas na
/// cadeia — o mesmo motivo do `is_frame_widget`.
///
/// ⚠️ Os CHIPS não entram: eles são `Dropdown`, e abrir/fechar é do dispatch genérico. Só as
/// opções são `Click` puro que a shell aplica.
fn is_token_option(id: ph2d_a11y::NodeId) -> bool {
    ids::TOKEN_SLOTS
        .iter()
        .any(|s| (0..=s.table.len()).any(|i| id == ids::vector_token_option_id(s.code, i)))
}

fn is_frame_widget(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_MODE_FRAME
        || id == ids::VECTOR_FRAME_CLIP_OFF
        || id == ids::VECTOR_FRAME_CLIP_ON
        || id == ids::VECTOR_FRAME_PANEL_OFF
        || id == ids::VECTOR_FRAME_PANEL_ON
        || ph2d_tool_vector::frames::device_preset(id).is_some()
}

/// **Os chips do AUTO LAYOUT** que a SHELL honra (plano UI/UX W2) — direção, alinhamento e
/// distribuição. Os valores moram no COMPONENTE, então o clique atravessa o barramento.
///
/// ⚠️ **Os dois chips do modo de recuo NÃO estão aqui**: eles só decidem quais campos são
/// pintados, e o documento não sabe nada sobre isso. Mandá-los à shell seria uma porta com nada
/// do outro lado — eles são resolvidos no painel, como o lado e a junção do Offset Path.
///
/// A lista é percorrida a partir do MESMO array que o `populate` regista, menos os dois do modo:
/// uma variante nova entra numa lista só, e o gate de seam clica todas.
fn is_layout_widget(id: ph2d_a11y::NodeId) -> bool {
    id != ids::VECTOR_LAYOUT_PAD_ALL_MODE
        && id != ids::VECTOR_LAYOUT_PAD_EACH_MODE
        && crate::populate::layout::LAYOUT_CHIPS.contains(&id)
}

/// **Os oito chips das ÂNCORAS** (plano UI/UX W3). Todos atravessam o barramento — ao contrário do
/// modo de recuo do layout, aqui não há chip panel-local: cada um escreve o par de âncoras no
/// COMPONENTE, e a régua é capturada do outro lado.
///
/// Percorre o MESMO array que o `populate` regista, pela mesma razão: um chip novo entra numa
/// lista só, e o gate de seam clica todas.
fn is_anchor_widget(id: ph2d_a11y::NodeId) -> bool {
    crate::populate::anchors::ANCHOR_CHIPS.contains(&id)
}

/// **Os treze pills de MODO + os três chips da fonte de largura do lápis.**
///
/// ⚠️ **Extraído da cadeia porque ela bateu no teto de 200 LOC por função** — e o doc-comment do
/// `is_frame_widget` já tinha nomeado esta família como a candidata óbvia quando o momento
/// chegasse. O corte é por ASSUNTO: *que ferramenta está na mão* é uma pergunta, e a cadeia
/// abaixo responde a outra (*este clique é meu ou do shell?*).
///
/// Fora daqui qualquer um deles pinta, ACENDE sob o mouse e o Click morre no painel — nunca vira
/// `ToolPanelEvent`. É o defeito que o smoke do Line/Arc pegou (Enio 2026-07-09) e o que o gate
/// `clicking_connect_pill_reaches_the_tool` vigia.
fn is_mode_pill(id: ph2d_a11y::NodeId) -> bool {
    matches!(
        id,
        x if x == ids::VECTOR_MODE_SELECT
            || x == ids::VECTOR_MODE_NODE
            || x == ids::VECTOR_MODE_PEN
            || x == ids::VECTOR_MODE_PENCIL
            || x == ids::VECTOR_MODE_SHAPE
            || x == ids::VECTOR_MODE_TEXT
            || x == ids::VECTOR_MODE_CONNECT
            || x == ids::VECTOR_MODE_BUILD
            || x == ids::VECTOR_MODE_PICKBLEND
            || x == ids::VECTOR_MODE_FILLET
            || x == ids::VECTOR_MODE_CHAMFER
            || x == ids::VECTOR_MODE_WIDTH
            || x == ids::VECTOR_MODE_CUT
            // A FONTE da largura do lápis (W1d) — três chips exclusivos, do mesmo assunto:
            // com que ferramenta, e como, o traço nasce.
            || x == ids::VECTOR_PENCIL_W_UNIFORM
            || x == ids::VECTOR_PENCIL_W_SPEED
            || x == ids::VECTOR_PENCIL_W_PRESSURE
    )
}

pub(super) fn forwards_plain_click(id: ph2d_a11y::NodeId) -> bool {
    is_mode_pill(id)
        // Os dois botões da seção CUT — executar e descartar a linha de corte.
        || id == ids::VECTOR_CUT_APPLY
        || id == ids::VECTOR_CUT_DISCARD
        // **Resize Box** (W3b) — o override mora no COMPONENTE, então o clique atravessa o
        // barramento. Sem esta linha ele pintaria, acenderia sob o rato e o Click morreria aqui.
        || id == ids::VECTOR_TRANSFORM_RESIZE_BOX
        // **A SIMETRIA de desenho** (W6.3) — o par que arma, os quatro tipos, o par do Fuse e o
        // Apply. Fora daqui eles pintam, ACENDEM sob o mouse e o Click morre no painel: o artista
        // clicaria "On" e nada aconteceria, com o log a dizer `[hero] unhandled event`. Foi
        // exatamente o que aconteceu no primeiro smoke desta wave.
        //
        // ⚠️ Os quatro tipos são percorridos pela MESMA porta que os pinta e que resolve o clique
        // (`SymmetryKind::ALL` → `symmetry_kind_id`): um tipo novo entra aqui sozinho.
        || id == ids::VECTOR_SYM_OFF
        || id == ids::VECTOR_SYM_ON
        || id == ids::VECTOR_SYM_FUSE_OFF
        || id == ids::VECTOR_SYM_FUSE_ON
        || id == ids::VECTOR_SYM_APPLY
        || ph2d_symmetry::SymmetryKind::ALL
            .iter()
            .any(|k| ph2d_tool_vector::params::symmetry_kind_id(*k) == id)
        // **O SELETOR DE CURVA dos estados de UI** (W7). Percorrido pela MESMA porta que o pinta
        // (`ALL` → `vector_easing_*_id`): uma família nova atravessa o barramento sozinha.
        || (0..ids::MAX_EASING_FAMILIES).any(|i| ids::vector_easing_family_id(i) == id)
        || (0..ids::MAX_EASING_MODES).any(|i| ids::vector_easing_mode_id(i) == id)
        || id == ids::VECTOR_TEXT_FONT_PREV
        || id == ids::VECTOR_TEXT_FONT_NEXT
        || id == ids::VECTOR_TEXT_FONT_IMPORT
        || id == ids::VECTOR_TEXT_ALIGN_LEFT
        || id == ids::VECTOR_TEXT_ALIGN_CENTER
        || id == ids::VECTOR_TEXT_ALIGN_RIGHT
        || id == ids::VECTOR_ALIGN_CENTRE
        || id == ids::VECTOR_ALIGN_INNER
        || id == ids::VECTOR_ALIGN_OUTER
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
        // As três da W4. Fora daqui pintam, acendem sob o mouse e o Click morre no painel.
        || id == ids::VECTOR_VERT_AVERAGE
        || id == ids::VECTOR_PATH_JOIN
        || id == ids::VECTOR_PATH_REVERSE
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
        // A BOOLEANA VIVA: os dois chips de modo sao panel-local no VALOR, mas a shell precisa
        // saber que o modo mudou (ela e' quem le' o modo no clique de uma das oito); e o Apply e'
        // um comando de DOCUMENTO. Fora daqui os tres pintariam e estariam MORTOS.
        || is_frame_widget(id)
        || is_layout_widget(id)
        || is_anchor_widget(id)
        // As opções dos pickers de token (plano UI/UX W4). Fora daqui elas pintam, acendem sob
        // o mouse e o Click morre no painel — o artista escolheria um token e nada mudaria.
        || is_token_option(id)
        || id == ids::VECTOR_BOOL_LIVE_OFF
        || id == ids::VECTOR_BOOL_LIVE_ON
        || id == ids::VECTOR_BOOL_APPLY
        || is_prefab_click(id)
        || id == ids::VECTOR_BOOL_MINUS_BACK
        || id == ids::VECTOR_BOOL_TRIM
        || id == ids::VECTOR_BOOL_CROP
        || id == ids::VECTOR_BOOL_MERGE
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
        || id == ids::VECTOR_SNAP_PATH_OFF
        || id == ids::VECTOR_SNAP_PATH_ON
        || id == ids::VECTOR_SNAP_CROSS_OFF
        || id == ids::VECTOR_SNAP_CROSS_ON
        || id == ids::VECTOR_SNAP_GUIDES_OFF
        || id == ids::VECTOR_SNAP_GUIDES_ON
        || id == ids::VECTOR_RULERS_OFF
        || id == ids::VECTOR_RULERS_ON
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

/// **A família do PREFAB e da PELE** — os verbos de componente (W5), os interruptores de peça e os
/// chips de variant (W5b/W5c), e os dois verbos + chips da pele por-widget (W6.2).
///
/// ⚠️ Extraída do [`forwards_plain_click`] quando ele cruzou o teto de 200 LOC, e o corte é por
/// ASSUNTO: tudo aqui é *"esta forma é um componente / veste um widget?"*, e tudo aqui mora no
/// ECS — por isso o clique atravessa o barramento em vez de morrer no painel.
fn is_prefab_click(id: ph2d_a11y::NodeId) -> bool {
    // Os quatro verbos de COMPONENTE (plano UI/UX W5): mestre e instância moram no ECS, então
    // o clique é da SHELL — o painel só mostra que verbos fazem sentido.
    id == ids::VECTOR_COMPONENT_CREATE
    || id == ids::VECTOR_COMPONENT_PLACE
    || id == ids::VECTOR_COMPONENT_DETACH
    || id == ids::VECTOR_COMPONENT_RESET
    || id == ids::VECTOR_COMPONENT_UPDATE_MAIN
    || id == ids::VECTOR_COMPONENT_SWAP
    // **Os interruptores de PEÇA** (W5b) — o override mora no ECS, então o clique atravessa o
    // barramento. A swatch de cor NÃO entra: ela é alvo de picker, e o `register_picker_swatch`
    // é quem trata o clique dela (o precedente da swatch de Fill).
    || (0..ids::MAX_INSTANCE_PIECES).any(|r| ids::vector_instance_piece_show_id(r) == id)
    // **Os chips de VARIANT** (W5c) — escolher uma versão RELIGA a instância a um mestre
    // irmão, e o vínculo mora no ECS: o clique é da shell, pela porta do *Swap Main*.
    || (0..ids::MAX_VARIANT_AXES).any(|a| {
        (0..ids::MAX_VARIANT_VALUES).any(|v| ids::vector_variant_option_id(a, v) == id)
    })
    // **A PELE por-widget** (W6.2) — o componente mora no ECS, então os dois verbos e os chips
    // de tipo atravessam o barramento; o painel só mostra o que faz sentido agora.
    || id == ids::VECTOR_WIDGET_WEAR
    || id == ids::VECTOR_WIDGET_REMOVE
    || id == ids::VECTOR_WIDGET_BIND
    || id == ids::VECTOR_WIDGET_UNBIND
    || (0..ids::MAX_WIDGET_KINDS).any(|i| ids::vector_widget_kind_id(i) == id)
    // **OS ESTADOS de UI** (W7) — a tabela mora no DOCUMENTO (`ProjectState`), então os três
    // verbos atravessam o barramento; o painel só mostra que verbos fazem sentido agora.
    || (0..ids::MAX_STATE_ROLES).any(|i| {
        ids::vector_state_record_id(i) == id
            || ids::vector_state_clear_id(i) == id
            || ids::vector_state_apply_id(i) == id
    })
    // **O MODO DE PREVIEW** (W7r) — quem toma o rato é a shell (só ela tem o picking e o
    // registro de undo), então o interruptor atravessa o barramento como os verbos ao lado.
    || id == ids::VECTOR_STATE_PREVIEW
    // **Mover o widget com TODOS os estados** (W7r): o deslocamento e' aplicado a' TABELA, que
    // mora no documento — entao o toggle atravessa o barramento como os verbos ao lado.
    || id == ids::VECTOR_STATE_MOVE_ALL
}
