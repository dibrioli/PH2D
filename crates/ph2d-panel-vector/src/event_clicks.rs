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
/// ⚠️ **A cadeia abaixo vive ENCOSTADA no teto de 200 LOC**, e já o estourou duas vezes: a cura é
/// sempre extrair uma FAMÍLIA (os pills de modo foram a primeira; a booleana, em 2026-08-27, foi a
/// segunda — ver [`is_boolean_click`]). ⛔ Nunca uma entrada no `FN_OVERAGE_OK`.
///
/// ⚠️⚠️ E o gate mora em `ph2d-editor-core/tests/`: **um fechamento por `cargo test -p
/// ph2d-panel-vector` NÃO o alcança**, então a wave que acrescenta um botão aqui só o vê no gate
/// batched — que é onde ele foi de facto apanhado das duas vezes.
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
            || x == ids::VECTOR_MODE_TRIM
            || x == ids::VECTOR_MODE_BUCKET
            || x == ids::VECTOR_PATH_WELD
            // A FONTE da largura do lápis (W1d) — três chips exclusivos, do mesmo assunto:
            // com que ferramenta, e como, o traço nasce.
            || x == ids::VECTOR_PENCIL_W_UNIFORM
            || x == ids::VECTOR_PENCIL_W_SPEED
            || x == ids::VECTOR_PENCIL_W_PRESSURE
    )
}

/// ⭐ **A seção MORPH STATES** (plano 32 W4/W8) — o botão que faz o conjunto, e a acção que dispara
/// cada transição. As duas mexem no MUNDO, então o clique atravessa o barramento e é a shell que
/// age; o painel só mostra.
///
/// ⚠️ **Percorridas pela MESMA porta que as pinta** (`MAX_MORPH_STATES` → `morph_arrow_*`): uma
/// forma a mais entra aqui sozinha. ⛔ Sem isto elas pintariam, acenderiam sob o rato e o Click
/// **morreria no painel** — o defeito exacto que a décima lista deste modo custou uma wave atrás.
///
/// ⚠️ **Função própria e não mais um `||` na irmã**: o `forwards_plain_click` bateu no teto de 200
/// LOC no dia em que este bloco lá entrou, e o corte por assunto já era o certo.
fn is_morph_states_control(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_MORPH_STATES_MAKE
        || id == ids::VECTOR_MORPH_PREVIEW
        || id == ids::VECTOR_MORPH_DISSOLVE
        || (0..ids::MAX_MORPH_STATES)
            .any(|r| id == ids::morph_shape_play_id(r) || id == ids::morph_shape_disconnect_id(r))
        || (0..ids::MAX_MORPH_STATES).any(|r| {
            (0..ids::MAX_MORPH_ACTIONS).any(|a| id == ids::morph_shape_key_option_id(r, a))
        })
}

pub(super) fn forwards_plain_click(id: ph2d_a11y::NodeId) -> bool {
    is_mode_pill(id)
        // Os dois botões da seção CUT — executar e descartar a linha de corte.
        || id == ids::VECTOR_CUT_APPLY
        || id == ids::VECTOR_CUT_DISCARD
        || is_morph_states_control(id)
        // **A FORMA do marquee** (`Box | Lasso`) — a tool é a dona do valor pegajoso, então o
        // clique atravessa o barramento como o dos pills de modo.
        || id == ids::VECTOR_MARQUEE_BOX
        || id == ids::VECTOR_MARQUEE_LASSO
        // **Resize Box** (W3b) — o override mora no COMPONENTE, então o clique atravessa o
        // barramento. Sem esta linha ele pintaria, acenderia sob o rato e o Click morreria aqui.
        || id == ids::VECTOR_TRANSFORM_RESIZE_BOX
        // ⭐ **Stroke** (plano 34) — a caixa mexe no DOCUMENTO (`path.stroke`), então o clique é da
        // shell. Fora daqui ela pintaria, acenderia sob o rato e o Click morreria no painel.
        || id == ids::VECTOR_STROKE_PRESENT
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
        || id == ids::VECTOR_TEXT_WRAP_AUTO
        || id == ids::VECTOR_TEXT_WRAP_FIXED
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
        || is_boolean_click(id)
        || is_frame_widget(id)
        || is_layout_widget(id)
        || is_anchor_widget(id)
        // As opções dos pickers de token (plano UI/UX W4). Fora daqui elas pintam, acendem sob
        // o mouse e o Click morre no painel — o artista escolheria um token e nada mudaria.
        || is_token_option(id)
        || is_prefab_click(id)
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
        || id == ids::VECTOR_FILL_KIND_PATTERN
        // ⭐ **A TINTA DO TRAÇO** (plano 35, wave D) — ela mexe no DOCUMENTO (`path.stroke.paint`),
        // então o clique é da shell, como o da fileira do preenchimento logo acima.
        || id == ids::VECTOR_STROKE_KIND_SOLID
        || id == ids::VECTOR_STROKE_KIND_PATTERN
        || id == ids::VECTOR_STROKE_KIND_BRUSH
        // ⭐ A secção BRUSH (plano 36, W4): o picker da arte e o `Flip`.
        || id == ids::VECTOR_BRUSH_PICK_SHAPE
        || id == ids::VECTOR_BRUSH_FLIP
        // ⭐⭐ **AS DUAS secções PATTERN** (plano 35, wave F) — a do preenchimento e a do traço.
        //
        // ⚠️ **Percorridas pela MESMA lista que as PINTA** (`TexPatKnob::ALL` × os slots): um knob
        // novo atravessa o barramento sozinho, nas duas secções. ⛔ Uma allowlist escrita à mão
        // aqui seria a terceira cópia da lista de controlos — e a que envelhece primeiro.
        || crate::paint_sections::texture_pattern::texpat_knob_of(id).is_some()
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
    // As linhas do picker de ÍCONE. ⚠️ A varredura é sobre o CATÁLOGO + 1 (o *Drawing*), e não
    // sobre um `MAX_*`: o id é hash de runtime, então não há teto de tabela a decidir quantos
    // glifos são alcançáveis — é a razão de o picker ser um dropdown e não uma segmentada.
    || (0..=ph2d_editor_core::icons::IconId::all().len())
        .any(|i| ids::vector_widget_icon_option_id(i) == id)
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
    // **A MOLA** (W7m): ela troca o motor da transicao, e o motor mora na tabela do DOCUMENTO —
    // entao o checkbox atravessa o barramento como os verbos ao lado.
    || id == ids::VECTOR_STATE_SPRING
    // ⭐ **A TABELA SINAL → PAPEL** (item 4 do estudo dos conteineres): a ligacao mora no
    // DOCUMENTO (`HostStates.on_signal`), entao os tres gestos — escolher o papel, apagar a
    // linha, acrescentar uma — atravessam o barramento como os verbos ao lado. O NOME nao passa
    // por aqui: ele e' um COMMIT de texto, e vai por `SelectOption` (o unico variante do
    // `PanelEvent` que carrega uma string).
    || id == ids::VECTOR_STATE_SIGNAL_ADD
    || (0..ids::MAX_SIGNAL_BINDINGS).any(|i| {
        ids::vector_state_signal_remove_id(i) == id
            || (0..ids::MAX_STATE_ROLES).any(|r| ids::vector_state_signal_role_id(i, r) == id)
    })
}

/// **A BOOLEANA, num predicado só** — os dois chips de modo, o Apply, os oito verbos de receita, os
/// quatro verbos de FORMA e o par do compound path.
///
/// ⚠️ Extraído do [`forwards_plain_click`] pelo teto de 200 LOC por função, e o corte é o mesmo dos
/// irmãos ([`is_frame_widget`], [`is_layout_widget`], [`is_anchor_widget`]): **um assunto**. Os oito
/// da receita e os quatro da FORMA convivem na mesma secção e fazem coisas diferentes sobre a mesma
/// selecção — os dois conjuntos têm ids próprios de propósito.
///
/// ⚠️ **Todos atravessam o barramento.** Os chips de modo são panel-local no VALOR, mas a shell
/// precisa saber que o modo mudou (é ela quem o lê no clique de um dos oito), e o Apply é um comando
/// de DOCUMENTO. Fora daqui eles pintariam, acenderiam sob o rato e o `Click` morreria no painel —
/// o report que o Enio já deu duas vezes sobre esta mesma fileira.
fn is_boolean_click(id: ph2d_a11y::NodeId) -> bool {
    id == ids::VECTOR_BOOL_UNION
        || id == ids::VECTOR_BOOL_LIVE_OFF
        || id == ids::VECTOR_BOOL_LIVE_ON
        || id == ids::VECTOR_BOOL_APPLY
        || id == ids::VECTOR_BOOL_MINUS_BACK
        || id == ids::VECTOR_BOOL_TRIM
        || id == ids::VECTOR_BOOL_CROP
        || id == ids::VECTOR_BOOL_MERGE
        || id == ids::VECTOR_BOOL_SUBTRACT
        || id == ids::VECTOR_BOOL_INTERSECT
        || id == ids::VECTOR_BOOL_EXCLUDE
        || id == ids::VECTOR_BOOL_SHAPE_UNION
        || id == ids::VECTOR_BOOL_SHAPE_SUBTRACT
        || id == ids::VECTOR_BOOL_SHAPE_INTERSECT
        || id == ids::VECTOR_BOOL_SHAPE_EXCLUDE
        || id == ids::VECTOR_COMPOUND_MAKE
        || id == ids::VECTOR_COMPOUND_RELEASE
}

/// ⭐⭐⭐ **Uma linha do popover de MISTURA do objecto foi escolhida** — fecha o chip e encaminha o
/// MODO (estudo 42 item 2).
///
/// ⚠️ **O que viaja no bus é o CÓDIGO do modo, não a linha do popover.** A lista é derivada da
/// tradução para o Vello, e mandar o índice obrigaria a shell a reconstruí-la para o traduzir —
/// uma segunda cópia da mesma lista é como os dois lados passam a discordar sobre o que a linha 7
/// significa. É o mesmo desenho do picker de PONTA (ele manda `marker.as_u8()`).
///
/// ⚠️ E o chip fecha AQUI: o light-dismiss não dispara num clique DENTRO do popover, e sem isto a
/// lista fica aberta por cima da secção depois de escolher (a cicatriz que o dropdown de fonte já
/// carrega).
pub(crate) fn pick_object_blend(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> bool {
    let Some(i) = crate::state::blend_option_index(id) else {
        return false;
    };
    let Some(mode) = ph2d_vec_render::blend::offered().nth(i) else {
        return false; // slot de id vazio: o espaço é fixo e a lista oferecida é menor
    };
    if let Some(ph2d_editor_core::interaction::InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(ids::VECTOR_OBJ_BLEND)
    {
        *open = false;
        *selected_index = Some(i);
    }
    host.bus_mut()
        .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
            ph2d_editor_core::tool::PanelEvent::SetValue(
                ids::VECTOR_OBJ_BLEND,
                f64::from(mode.to_u8()),
            ),
        ));
    true
}

/// ⭐⭐⭐ **Uma linha do popover de MISTURA de uma CAMADA** (estudo 42 item 4) — espelho exacto do
/// [`pick_object_blend`], com o espaço de ids próprio.
///
/// ⛔ **Sem este braço a lista da camada é um CONTROLO MORTO**: ela pinta, regista os hit-rects e
/// consome o clique, e o valor não chega a consumidor nenhum. Foi o `clippy` que o apanhou — a
/// função de resolução ficava *never used* —, e é exactamente a espécie que o `CLAUDE.md` §5.0
/// chama de *dreno de UM BRAÇO SÓ*.
pub(crate) fn pick_layer_blend(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> bool {
    let Some(i) = crate::state::paint_blend_option_index(id) else {
        return false;
    };
    let Some(mode) = ph2d_vec_render::blend::offered().nth(i) else {
        return false; // slot de id vazio: o espaço é fixo e a lista oferecida é menor
    };
    if let Some(ph2d_editor_core::interaction::InteractiveState::Dropdown {
        open,
        selected_index,
        ..
    }) = host.store_mut().get_mut(ids::VECTOR_PAINT_BLEND)
    {
        *open = false;
        *selected_index = Some(i);
    }
    host.bus_mut()
        .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
            ph2d_editor_core::tool::PanelEvent::SetValue(
                ids::VECTOR_PAINT_BLEND,
                f64::from(mode.to_u8()),
            ),
        ));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **OS QUATRO CHIPS DO VERBO POR FORMA ATRAVESSAM O BARRAMENTO.**
    ///
    /// ⚠️ Um id que este predicado não encaminha **pinta, acende sob o mouse, e o Click morre
    /// aqui** — o artista clica e nada acontece, com o log a dizer `[hero] unhandled event`. O
    /// comentário da SIMETRIA acima conta que foi exactamente assim que ela falhou o primeiro
    /// smoke dela.
    ///
    /// Este gate nasceu da caça de 2026-08-22: os quatro chips shiparam com o modelo, o cozimento
    /// e a triagem gateados, e **zero** gates no caminho do clique. A causa acabou por ser outra
    /// (o sujeito tinha de ser o primário), mas a metade que faltava era esta — e ela é barata.
    #[test]
    fn the_per_shape_boolean_chips_cross_the_bus() {
        for id in [
            ids::VECTOR_BOOL_SHAPE_UNION,
            ids::VECTOR_BOOL_SHAPE_SUBTRACT,
            ids::VECTOR_BOOL_SHAPE_INTERSECT,
            ids::VECTOR_BOOL_SHAPE_EXCLUDE,
        ] {
            assert!(
                forwards_plain_click(id),
                "um chip do verbo por forma nao atravessa: o Click morreria no painel"
            );
        }
    }
}
