//! **O clique, a PREVIEW, o FLIP e os gizmos de nó** — ramos do `on_mouse_input` ([`super`]): a preview da UI
//! desenhada (dona do clique), os Up que fecham traços e arrastos em curso, a costura da coluna e as guias, e os
//! press do Flip (Draw, Pairs, Fill, Colorize, Trace, Erase, Reshape, Edit) e dos gizmos de pose, de selecção,
//! de warp e de field. Os corpos MUDARAM-SE verbatim (`line/input-dispatch`, 2026-09-13) e correm no sítio da
//! chamada, pela mesma ordem: todos os Up antes de todos os press.
//!
//! ⚠️ Cada ramo devolve `true` onde o handler fazia `return;`, e o índice devolve ao receber `true`.

use super::*;

impl crate::App {
    /// Os press do Flip no canvas (Draw, Pairs, Fill, Colorize, Trace, Erase, Reshape, Edit) e dos gizmos de pose,
    /// de selecção, de warp (só no canvas) e de field.
    pub(super) fn ramo_flip_premidos(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        menu_open_before: bool,
        on_canvas: bool,
    ) -> bool {
        if self.flip_wants_canvas()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Tween v2 — correção de pares: enquanto a sessão Pairs está aberta, o clique do
        // canvas RE-PAREIA (sobrepõe o modo atual — Draw/Erase/etc — porque é um sub-modo do
        // fluxo de tween, não um modo de desenho). Uma chamada faz tudo (não é arrasto).
        if self.flip_wants_tween_pairs()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_tween_pairs_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip bucket (W4): um CLIQUE no modo Fill preenche a região sob o cursor.
        // Não é um gesto de arrasto — uma chamada faz tudo.
        if self.flip_wants_fill()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_fill_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // ADR-0114 C2: a pen-DOWN no modo Colorize começa um RABISCO (arrasto), como o Draw.
        if self.flip_wants_colorize()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_colorize_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Shift & Trace: a pen-DOWN no modo Trace pega o fantasma sob o cursor (Ctrl =
        // girar) e CONSOME mesmo errando — o Trace é dono do canvas (cair no gizmo
        // moveria o objeto no meio do posicionamento da referência).
        if self.flip_wants_trace()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_trace_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip eraser (T2.9): a pen-DOWN in Erase mode on the canvas begins an
        // erase gesture (Select falls through to gizmo/pick, like Draw).
        if self.flip_wants_erase()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_erase_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip sculpt (W5): a pen-DOWN no modo Reshape começa um gesto de escultura
        // (Select cai no gizmo/pick, como o Draw e a borracha).
        if self.flip_wants_reshape()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_reshape_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip W7.5: um handle do gizmo de POSE sob o cursor abre o arrasto de pose
        // (rotate/scale da instância). Vem ANTES do arm de canvas do Edit — um handle
        // registrado no hit-index torna `on_canvas` falso, então sem este arm o clique
        // cairia no caminho genérico de gizmo (que escreve o `Transform` do OBJETO).
        // O método só consome quando o hit é `GizmoTarget::FlipPose`.
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_pose_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip §4.A: um handle do gizmo de SELEÇÃO sob o cursor abre o arrasto de
        // seleção (rotate/scale assado nos pontos). Como o da pose, vem ANTES do arm de
        // canvas do Edit — um handle no hit-index torna `on_canvas` falso. O método só
        // consome quando o hit é `GizmoTarget::FlipSelection` (mutuamente exclusivo com
        // o da pose, então nunca disputam o mesmo clique).
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.flip_selection_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Motion Nodes: um handle do gizmo de FIELD sob o cursor abre o arrasto (rotate/
        // scale/translate escrito nos params do NÓ). Como os do Flip, vem ANTES do caminho
        // genérico de gizmo — um handle no hit-index o alcançaria e escreveria um
        // `Transform`. O método só consome quando o hit é `GizmoTarget::MotionField`, que
        // só existe com a tool Motion ativa + um field espacial selecionado no grafo.
        // Motion Nodes: uma ALÇA do gizmo dos deformadores de quadrilátero (Corner Pin +
        // Bezier Warp) sob o cursor abre o arrasto, que escreve os params do NÓ. Vem antes
        // do gizmo de field e do genérico pela mesma razão que aquele: uma alça alcançada
        // pelo caminho genérico escreveria um `Transform` de entidade. O método só consome
        // quando há retrato publicado — ou seja com a tool Motion activa e um dos dois nós
        // seleccionado.
        //
        // ⚠️ **E ele exige `on_canvas`, ao contrário dos irmãos** — Enio, 2026-08-23:
        // *"se colocar transform antes, não é possível conectar transform em Bezier
        // Warp"*. Os gizmos acima consomem pelo HIT-INDEX (`GizmoTarget::…`), que já
        // sabe das regiões; este faz o seu próprio hit-test em coordenadas de MUNDO, e
        // sem guarda ele convertia um clique **no painel do grafo** para o mundo, calhava
        // de cair sobre uma alça, e ENGOLIA o gesto de ligar um fio. *Um consumidor que
        // decide sozinho tem de saber sozinho onde ele vale.*
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && on_canvas
            && self.warp_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.field_gizmo_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        // Flip Edit Mode (W6): um CLIQUE no modo Edit seleciona o TRAÇO sob o cursor
        // (Shift alterna; no vazio, desmarca). Consome mesmo errando o traço — no Edit o
        // gizmo de objeto não manda, senão o arrasto seguinte moveria o objeto inteiro.
        if self.flip_wants_edit()
            && kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && on_canvas
            && !menu_open_before
            && self.flip_edit_canvas_down(self.last_pointer.0, self.last_pointer.1)
        {
            return true;
        }
        false
    }

    /// A preview (dona do clique sobre canvas ou gizmo), e os Up que fecham o que está em curso — traço e borracha do
    /// Flip, costura da coluna, guias, Reshape, Edit, Colorize, Trace, gizmos de pose/selecção/warp/field.
    pub(super) fn ramo_preview_e_fechos(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        evt: PointerEvent,
        menu_open_before: bool,
    ) -> bool {
        // **O MODO DE PREVIEW** (plano UI/UX W7r) — a UI desenhada a responder ao rato.
        //
        // ⚠️ Vem antes de TODA ferramenta, e não só das do Vector: enquanto ele corre não existe
        // pincel, traço do Flip, seleção, gizmo nem caneta — o clique é da interface que o artista
        // desenhou. É a mesma doutrina dos picks armados (*um modo em curso é dono do clique*),
        // uma família adiante: aqueles são modais sobre o Vector, este é modal sobre o editor.
        //
        // ⚠️ **A guarda é `over_canvas_or_gizmo`, e o `on_canvas` estava ERRADO — reportado pelo
        // Enio (2026-08-07: *"em preview permite que tanto o pai como o filho fossem
        // selecionados"*).** O `on_canvas` exige o `hit_index` **VAZIO**, e o gizmo registra as
        // alças (e o interior de translação) NELE — o doc do `over_canvas_or_gizmo` diz isto
        // literalmente, e eu escolhi o outro justificando-o num comentário. Entrar na preview
        // **exige o hospedeiro SELECIONADO** (a seção States só existe assim), logo o gizmo está
        // sempre lá, logo a guarda **nunca disparava na configuração em que a feature roda** — o
        // clique caía no picking e selecionava o filho. E ficava ERRÁTICO quando um estado movia
        // a forma para longe: fora da caixa do gizmo o `hit_index` volta a estar vazio e a guarda
        // acordava, então o mesmo gesto funcionava ou não conforme ONDE a forma estava.
        //
        // O `over_canvas_or_gizmo` aceita o gizmo por cima e **continua a barrar painel** — é
        // isso que mantém o próprio botão *Preview* clicável, que é a porta de saída visível.
        if self.ui_preview.is_on()
            && self.over_canvas_or_gizmo(evt.x, evt.y)
            && !menu_open_before
            && mapped_button == ph2d_host::PointerButton::Primary
            && matches!(kind, PointerKind::Down | PointerKind::Up)
        {
            self.ui_preview_point(evt.x, evt.y, kind == PointerKind::Down);
            return true;
        }
        // ADR-0114 W2: desenho do Flip. O pen-UP sempre encerra um traço em curso
        // (consome), mesmo que o modo tenha mudado no meio. O pen-DOWN começa um
        // traço só no modo Draw, em canvas vazio — em Select cai no gizmo/pick.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_state.draw.is_active()
            && self.flip_canvas_up()
        {
            return true;
        }
        // Flip eraser (T2.9): the pen-UP ends an erase gesture (+ Soft cleanup).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_state.erasing
            && self.flip_erase_canvas_up()
        {
            return true;
        }
        // **AS GUIAS** (plano 25 §9, a W6.2). O gesto da régua vem ANTES de toda ferramenta,
        // e não por prioridade inventada: a faixa da régua está VISÍVEL com qualquer ferramenta
        // na mão, então um press nela que caísse no picking/gizmo moveria um objeto em vez de
        // puxar uma guia — chrome desenhado e morto sob o mouse, que é o defeito que esta
        // codebase varre a cada wave.
        //
        // ⚠️ Uma vez começado, o arrasto é DONO do ponteiro até o Up (o padrão do `joint_draw`).
        // O passo do MEIO — o Move que leva a guia — mora no `on_cursor_moved`, que é o
        // handler que o winit usa para movimento; **este só recebe Down e Up**.
        //
        // ⚠️ E o Up **não é gateado no Primary**, pelo mesmo motivo que abre o `on_mouse_input`
        // com o release da mão: um arrasto que sobrevive ao release fica colado no cursor para
        // sempre, e um botão secundário não é um modificador de gesto.
        // ⭐ **A BORDA DA COLUNA redimensiona** (Enio, 2026-08-30). Vem antes de tudo pelo mesmo
        // motivo que o gesto da guia: a costura vive DENTRO da coluna, por cima do corpo do
        // painel — sem a precedência, o painel come o press e a borda fica inerte.
        if kind == PointerKind::Up && self.dock_seam_up() {
            return true;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.dock_seam_down(evt.x, evt.y)
        {
            return true;
        }
        if kind == PointerKind::Up && self.guide_pointer_up(evt.x, evt.y) {
            return true;
        }
        if kind == PointerKind::Down
            && mapped_button == ph2d_host::PointerButton::Primary
            && !menu_open_before
            && self.guide_pointer_down(evt.x, evt.y)
        {
            return true;
        }
        // Flip sculpt (W5): o pen-UP encerra o gesto (a máscara congelada morre com
        // ele; o passo de undo sai do diff pós-frame).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_reshape_canvas_up()
        {
            return true;
        }
        // Flip Edit Mode (W6.1): o pen-UP fecha o marquee (aplicando a seleção) ou o
        // move. Como os outros UPs, ele vem ANTES dos DOWNs e não depende do modo atual
        // (o gesto pode ter começado antes de uma troca de modo).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_edit_canvas_up()
        {
            return true;
        }
        // ADR-0114 C2: o pen-UP fecha um RABISCO do Colorize e o acumula no buffer (as
        // regiões só nascem no Apply). Como os outros UPs, vem antes dos DOWNs.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_colorize_canvas_up()
        {
            return true;
        }
        // Shift & Trace: o pen-UP fecha o arrasto de trace (o deslocamento já está
        // aplicado — é exibição, não documento). Como os outros UPs, vem antes dos DOWNs.
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_trace_canvas_up()
        {
            return true;
        }
        // Flip W7.5: o pen-UP fecha um arrasto do gizmo de pose (o passo de undo sai
        // do diff pós-frame, como os outros gestos).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_pose_gizmo_up()
        {
            return true;
        }
        // Flip §4.A: o pen-UP fecha um arrasto do gizmo de seleção (idem — undo pós-frame).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && self.flip_selection_gizmo_up()
        {
            return true;
        }
        // Motion Nodes: o pen-UP fecha um arrasto do gizmo de field e commita o passo de
        // undo (um arrasto = um passo, como um drag de nó).
        if kind == PointerKind::Up
            && mapped_button == ph2d_host::PointerButton::Primary
            && (self.warp_gizmo_up() || self.field_gizmo_up())
        {
            return true;
        }
        false
    }
}
