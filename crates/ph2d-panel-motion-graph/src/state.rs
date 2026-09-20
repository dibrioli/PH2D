//! Ephemeral editor state (Motion Nodes M1.E4/E5) — pan / zoom / selection /
//! in-progress drag. Non-undoable (only doc mutations, via `GraphIntent`, are).
//! Owned by the typed panel registry; passed `&mut` into `paint`.

use std::collections::{BTreeMap, BTreeSet};

/// ⭐⭐ **O POPUP e os VERBOS dele** — irmão por RESPONSABILIDADE (o tecto de LOC dos painéis
/// obrigou ao corte, e a pergunta desenhou-o): aqui fica o estado da INTERACÇÃO (a vista, a
/// selecção, o arrasto em curso), ali *o que uma lista flutuante é uma lista DE*.
#[path = "state_menu.rs"]
mod menu;
pub(crate) use menu::{BACKDROP_ACTIONS, Menu, MenuBody, NodeAction};

/// graph-space → screen affine: `screen = panel_origin + pan + graph * zoom`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct ViewState {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }
}

/// The active pointer interaction on the canvas.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Interaction {
    #[default]
    Idle,
    /// Panning the canvas — `last` is the previous pointer position (screen).
    /// Driven by the **middle** button, from anywhere on the surface (over a card,
    /// a wire, a backdrop — the graph moves under the cursor), which is the node-
    /// editor convention (Blender / Nuke / Houdini). The LEFT button is for
    /// selecting, never for panning (Enio, smoke 2026-07-12).
    Pan { last: (f32, f32) },
    /// Rubber-band selection — a left-drag on empty canvas. `anchor` is the press
    /// point and `cur` the live cursor (both SCREEN space, like the canvas's own
    /// rubber band: the band must stay put under the cursor, and the graph cannot
    /// pan mid-drag anyway since panning is on another button). `additive` (Shift
    /// at press) unions with the current selection instead of replacing it.
    BoxSelect {
        anchor: (f32, f32),
        cur: (f32, f32),
        /// Shift held at the start — the band ADDS to the selection instead of replacing it.
        additive: bool,
        /// Ctrl/Cmd held at the start — the band REMOVES the nodes it covers from the selection
        /// (Alt is avoided: KDE steals it for window drags). Wins over `additive` if both are held.
        subtract: bool,
    },
    /// Slicing a knife stroke across the canvas (armed by `K`): every wire the
    /// segment crosses is cut on release, as ONE undo step. Screen space.
    Knife { anchor: (f32, f32), cur: (f32, f32) },
    /// **Arrastar o valor de um param no cartão** (ciclo 1). Guarda o valor de PARTIDA e o x
    /// de partida, e não o último x: somar deltas por quadro acumula o erro de arredondamento
    /// de um param inteiro (arrastar e voltar não devolveria o número onde começou).
    ScrubParam {
        node: u32,
        row: u16,
        /// ⭐ **O NOME do param, capturado na PRESSÃO.** Ele existe para o arrasto ser
        /// PUBLICÁVEL (`snapshot::set_graph_param_scrub`): um gizmo de canvas que queira acender
        /// enquanto a mão mexe num knob precisa de saber em QUAL, e resolver a `row` de fora
        /// obrigaria a segunda cópia do `band_at` — *a coordenada `row` é do pintor e do
        /// hit-test; o nome é do domínio.*
        param: &'static str,
        start_value: f32,
        start_x: f32,
    },
    /// Dragging the selected nodes. `last` is the previous pointer (screen);
    /// each Update pushes an incremental `MoveNodes` delta the shell applies
    /// live (so the node tracks the cursor with no end-jump). `started` gates the
    /// one-undo-step bracket (BeginDrag on the first move, EndDrag on release).
    DragNodes {
        nodes: Vec<u32>,
        last: (f32, f32),
        started: bool,
        /// ⭐ **O deslocamento ACUMULADO do arrasto, em espaço de grafo** — a única coisa que o
        /// painel sabe e a shell não: ela já aplicou cada `MoveNodes` ao vivo, logo não tem mais
        /// onde ler ONDE a carta começou. A troca de dois nós
        /// ([`crate::snapshot::GraphIntent::SwapInChain`]) precisa dele para pôr o alvo no sítio
        /// de onde o arrastado veio.
        moved: (f32, f32),
    },
    /// Dragging a backdrop by its header — it carries the nodes it FRAMES, whose
    /// set is captured once at grab time (`nodes`) rather than re-tested each
    /// frame: a node that drifts to the edge mid-drag must not silently join or
    /// leave the group it is being carried with. Each Update pushes a
    /// `MoveBackdrop` + a companion `MoveNodes` (one undo step for the pair).
    DragBackdrop {
        id: u32,
        nodes: Vec<u32>,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging one of a backdrop's bottom grippers — grows/shrinks it in place.
    /// `left` is the corner grabbed (the opposite edge stays anchored).
    ResizeBackdrop {
        id: u32,
        left: bool,
        last: (f32, f32),
        started: bool,
    },
    /// Dragging a new wire out of an output socket (E6). `cur` is the live
    /// pointer (screen) the ghost wire tracks; `target` is the input socket
    /// currently under the pointer (if any) plus whether it is locally
    /// type-compatible (domain + dim + clock) — drives the ghost color + target
    /// highlight. The drop emits `Connect` for the bridge to validate for real.
    DrawWire {
        from_node: u32,
        from_port: u16,
        cur: (f32, f32),
        target: Option<(u32, u16, bool)>,
        /// **The input this wire was PULLED OFF** (F2, doc 45) — set when the gesture began
        /// on an occupied input socket rather than on an output.
        ///
        /// A grabbed wire-end MOVES; it does not copy. So the drop emits `MoveWireEnd` (one
        /// undo step: unplug the old input, plug the new) instead of `Connect`, and the
        /// painter hides the edge being dragged — a wire that is still drawn into the socket
        /// you are visibly pulling it out of is a wire that has not moved.
        detached: Option<(u32, u16)>,
    },
    /// Dragging a wire BACKWARDS out of an empty input socket, hunting for an output to
    /// feed it (F2, doc 45). `target` is the OUTPUT socket under the pointer (with local
    /// type-compatibility), the mirror of `DrawWire`'s input target.
    ///
    /// Both directions exist because a graph is read in both: sometimes you know what a node
    /// needs and go looking for it, and an editor that only lets you wire forwards makes you
    /// cross the canvas to say so.
    DrawWireBack {
        to_node: u32,
        to_port: u16,
        cur: (f32, f32),
        target: Option<(u32, u16, bool)>,
    },
    /// Dragging the add-menu's scrollbar thumb. `grab` is where INSIDE the thumb the cursor
    /// seized it — without that, the thumb jumps to put its TOP edge under the cursor the moment
    /// you touch it, which is the scrollbar bug everyone has met.
    MenuScroll { grab: f32 },
}

/// The inline rename box: F2 opened it over something, and it is waiting for a name (doc 61).
///
/// The TEXT is not here — it lives in the `WidgetStore`, which owns the buffer, the caret and the
/// selection. This is only what the box is *pointing at* and whether it has claimed the keyboard
/// yet (once, on the frame it opens; re-claiming every frame would fight anything else the artist
/// touches, and re-seeding would stomp what they are typing).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Rename {
    pub target: crate::snapshot::RenameTarget,
    /// What the thing was called when the box opened — the string the field is seeded with.
    pub seed: String,
    pub opened: bool,
}

/// ⭐⭐⭐ **A CAIXA DE DIGITAR UM PARAM, aberta sobre a row do cartão** (report do Enio,
/// 2026-09-05: *«vários nós não permitem clicar no número para usar o teclado para escrever»*).
///
/// Irmã de [`Rename`], e de propósito: as duas são a MESMA coisa — um campo efémero que toma o
/// teclado, vive sobre o sítio onde o valor se lê, e comita por `Enter`. O que se desfaz é o
/// **valor** que ela comita, nunca ela.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ParamEdit {
    /// O nó e a FAIXA (a coordenada do pintor/hit-test) — a row é reencontrada por `band_at`,
    /// nunca guardada, para uma secção dobrada debaixo da caixa não a deixar a apontar a outro
    /// param.
    pub node: u32,
    pub row: u16,
    /// O nome do param — `&'static` do registry, o mesmo que o intent leva.
    pub param: &'static str,
    /// O valor com que a caixa abre: **o que o artista está a ver**, e por isso formatado como a
    /// row o escreve (ver `crate::paint_card_params::param_text`).
    pub seed: String,
    /// A faixa digitável e o passo — do [`crate::snapshot::CardParam`], que os traz do registry.
    pub min: f64,
    pub max: f64,
    pub step: f64,
    /// ⚠️ **A FACE** (`mostrado = guardado × escala`) — a caixa mostra e aceita o número do
    /// artista (`94 px`), e o documento recebe o dele (`0,94`). Guardada aqui e não relida da
    /// row porque a conversão de VOLTA acontece no `commit`, que já não tem a row à mão.
    pub face_scale: f32,
    /// ⭐ **A caixa é de TEXTO** (um nome de coluna, um sinal, uma fórmula) e não de número.
    ///
    /// ⚠️ **Um `bool` e não um enum, porque a pergunta tem duas respostas e nenhuma terceira à
    /// vista** — e porque as duas metades divergem em quatro sítios (abrir, desenhar, comitar, e
    /// a faixa digitável, que só existe num número).
    pub text: bool,
    /// `false` até o quadro em que a caixa toma o teclado (o `settle_focus`), como no [`Rename`].
    pub opened: bool,
}

/// An open popup (E7). Ephemeral — never undoable. The frame (panel, scrollbar,
/// rows) is one widget; WHAT it lists and what a pick MEANS is [`MenuBody`].
/// Retained panel state.
/// **Where a node's postage stamp sits** (doc 86, Feature B) — its own moldura,
/// ABOVE or BELOW the card, chosen by the header toggle. It is a VIEW preference
/// (like `selected`): panel-local, non-undoable, not serialized. `Below` is the
/// default, so the map below only holds the nodes an artist has flipped up.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum PreviewPos {
    Above,
    #[default]
    Below,
}

#[derive(Default)]
pub struct MotionGraphPanelState {
    pub(crate) view: ViewState,
    /// **Per-node preview-frame position** (doc 86) — a node absent from the map
    /// takes the `PreviewPos` default (`Below`). Runtime-only, like `selected`:
    /// it does not travel the graph text format, so it is not persisted (moving
    /// the preview by default would change how every saved graph looks). Keyed by
    /// node id (`NodeId.0`).
    pub(crate) preview_pos: BTreeMap<u32, PreviewPos>,
    /// `false` until the first paint auto-fits the graph (then user-controlled).
    pub(crate) fitted: bool,
    /// A MANUAL fit (the chip or `F`) asked to frame the SELECTION, not the whole
    /// graph. Set only by [`Self::request_fit`] when there is a selection; the
    /// auto-fits (first sight, level change) leave it `false`, so they always frame
    /// everything. Consumed and reset by the paint's fit pass.
    pub(crate) fit_selection: bool,
    /// Selected node ids (`NodeId.0`).
    pub(crate) selected: BTreeSet<u32>,
    /// The selected backdrop, if any. Mutually exclusive with `selected`: the
    /// params panel shows the properties of ONE subject, and a Delete must never
    /// be ambiguous about what it removes.
    pub(crate) selected_backdrop: Option<u32>,
    /// The selected WIRES — each is a wire's unique target input `(to_node, to_port)`, the same
    /// handle a `Disconnect` carries. Mutually exclusive with `selected` and `selected_backdrop`
    /// (one subject at a time), so a left-click on a wire selects it, `Delete` removes it, and the
    /// three selections clear each other. A plain click makes one wire the sole selection; **Shift
    /// toggles a wire into/out of the set** (the node shift-select idiom), so `Delete` can drop a
    /// bundle of edges at once. Left-clicking a wire used to be inert; this is the universal
    /// click-then-Delete idiom the alt-click Disconnect had no visible affordance for.
    pub(crate) selected_wires: BTreeSet<(u32, u16)>,
    /// ⭐⭐⭐ **O que uma largada faria SE o artista soltasse agora** — o realce que o dono pediu
    /// (2026-09-19: *«nós e linhas podem ganhar um destaque de cor ou outro indicativo de que
    /// estão sobrepostos prestes a trocar ou encaixar»*).
    ///
    /// ⚠️ **É resolvido pela MESMA função que decide no largar** (`interact::node_drop::largada`),
    /// e guardado aqui só para o pintor o poder ler. *Um realce derivado de uma segunda leitura
    /// parecida promete uma coisa e entrega outra.*
    pub(crate) largada_viva: Option<crate::interact::node_drop::Largada>,
    /// ⭐⭐⭐ **O ECO da última largada** — lido do canal da shell no início de cada pintura (a
    /// shell já o entrega em INTENSIDADE, ver `ph2d_panel_motion_graph::Piscada`). ⚠️ Vive no
    /// estado só para o pintor não ter de tocar no `thread_local` uma vez por carta.
    pub(crate) piscada_viva: Option<crate::snapshot::Piscada>,
    pub(crate) interaction: Interaction,
    /// ⭐⭐⭐ **O EDITOR RICO ABERTO** (curva, e a seguir gradiente e paleta) — a janela
    /// flutuante que o cartão abre para os controlos que não cabem numa fileira de `22 px`.
    /// Ver [`crate::param_editor`]. Estado de VISTA: morre com o painel, não é intenção nenhuma.
    pub(crate) editor: Option<crate::param_editor::Open>,
    /// Open add-node popup, or `None`. Opened by R-click on empty canvas / `A`;
    /// closed by picking a row, clicking away, or Esc.
    pub(crate) menu: Option<Menu>,
    /// **The open rename box** (doc 61), or `None`. Ephemeral — never undoable; what IS undoable
    /// is the name it commits.
    pub(crate) rename: Option<Rename>,
    /// **A caixa de digitar um número, aberta sobre a row** (report do Enio, 2026-09-05), ou
    /// `None`. Efémera como o [`Rename`] — o que é desfazível é o valor que ela comita.
    pub(crate) param_edit: Option<ParamEdit>,
    /// `P` armed the probe: the NEXT click on a node picks it as the probe target.
    /// Disarmed by the pick, by Esc, or by a second `P` — same three exits as the
    /// knife (a mode you cannot leave is a trap).
    pub(crate) probe_armed: bool,
    /// The node whose output the probe is reading (its readout + sparkline draw
    /// beside the card). `None` = no probe.
    pub(crate) probe: Option<u32>,
    /// `K` armed the knife: the NEXT left-drag on the canvas slices wires instead
    /// of rubber-band selecting. Disarmed by the stroke itself, by Esc, or by a
    /// second `K` — a mode that cannot be left is a trap, so it has three exits.
    pub(crate) knife_armed: bool,
    /// **The level this panel last painted** (doc 57) — mirrored from the snapshot,
    /// never authored here (the SHELL owns the level, so that an undo which dissolves
    /// the group you are standing in can put you back on solid ground). It exists for
    /// one reason: to notice a level CHANGE and re-fit, so entering a group never
    /// lands the artist on a canvas they cannot see.
    pub(crate) level: Option<u32>,
    /// The input a wire's end was just pulled off, held for the ONE frame between
    /// the drop and the shell's answer (doc 45.1).
    ///
    /// The shell applies the panel's intents at the top of the next frame and only
    /// then republishes the snapshot — so the frame in which the drop happens still
    /// paints from a snapshot where the wire is plugged in where it was. Without
    /// this, the released end **snapped back to its old socket for one frame** before
    /// vanishing: a wire the artist had just torn out, drawn back where it no longer
    /// is. Keeping the suppression alive across that boundary means the end is never
    /// drawn anywhere it is not.
    ///
    /// Cleared at the top of the next `interact::apply` — by then the snapshot is the
    /// shell's answer, whatever it was, and it is the truth to paint. (A REFUSED move
    /// therefore shows the original wire again, which is exactly right: it never moved.)
    pub(crate) pending_detach: Option<(u32, u16)>,
}

impl MotionGraphPanelState {
    /// The view (pan/zoom), for the seam gates — see `menu_for_test`.
    pub(crate) fn view_for_test(&self) -> ViewState {
        self.view
    }

    /// The open menu, for the seam gates (the state is private and a paint-driven test has no
    /// other way to see what the artist is looking at).
    pub(crate) fn menu_for_test(&self) -> Option<&Menu> {
        self.menu.as_ref()
    }

    /// **A new level is a new canvas** (doc 57). Adopt the level the shell published;
    /// if it CHANGED, drop the fit so the next paint re-frames.
    ///
    /// The fold moves nothing — a group's members sit exactly where they always sat —
    /// so a view zoomed in on the collapsed card would open the group showing a corner
    /// of it, or empty canvas. Blender and Houdini both re-frame on the way in.
    ///
    /// Returns whether the level changed (the paint does not care; the gate does).
    pub(crate) fn sync_level(&mut self, level: Option<u32>) -> bool {
        if self.level == level {
            return false;
        }
        self.level = level;
        self.fitted = false;
        true
    }

    /// **Request a re-frame** (the Fit chip / `F`) — the draw pass owns the fit math,
    /// this only asks for it. It frames the SELECTION when there is one and the whole
    /// graph otherwise: the universal node-editor `F` (Blender/Nuke/Houdini frame the
    /// selection), and the same "one gesture, two behaviours" the Backdrop chip uses.
    /// ONE door, so the chip and the key can never diverge on the rule.
    pub(crate) fn request_fit(&mut self) {
        self.fitted = false;
        self.fit_selection = !self.selected.is_empty();
    }

    /// Where node `id`'s preview frame sits — **a escolha do artista se ele fez uma, e senão o
    /// lado que a DISPOSIÇÃO escolhe** ([`crate::geom::retratos_em_cima`], cujo resultado desta
    /// tela chega aqui em `em_cima`). A pintura lê-a para pousar a moldura e para acender o
    /// indicador do botão do cabeçalho; é a mesma porta que a virada abaixo escreve.
    ///
    /// ⛔⛔ **O `unwrap_or_default` MORREU, e a morte é a wave inteira:** até 2026-09-20 a
    /// omissão era `Below` para toda gente, e o dono fotografou o cartão do topo de uma coluna
    /// com a moldura no corredor por baixo dele. Hoje a omissão é uma LEI, e o mapa guarda só as
    /// viradas que uma MÃO fez.
    pub(crate) fn preview_position(&self, id: u32, em_cima: &BTreeSet<u32>) -> PreviewPos {
        self.preview_pos
            .get(&id)
            .copied()
            .unwrap_or(if em_cima.contains(&id) {
                PreviewPos::Above
            } else {
                PreviewPos::Below
            })
    }

    /// Flip node `id`'s preview between Above and Below (the header toggle). Stores
    /// the value explicitly (not "remove to reset"), so a flip is a plain toggle
    /// whatever the current state.
    ///
    /// ⚠️⚠️ **Ele vira a partir do que está NA TELA e não a partir de `Below`** — com a omissão
    /// a ser uma lei, um `entry().or_default()` faria o PRIMEIRO clique sobre um cartão que a
    /// lei já pôs em cima ser um **no-op visual**: ele escreveria `Below`, viraria para `Above`,
    /// e nada se mexia. *Um botão cujo primeiro toque não faz nada lê-se como um botão morto.*
    pub(crate) fn toggle_preview_position(&mut self, id: u32, em_cima: &BTreeSet<u32>) {
        let visto = self.preview_position(id, em_cima);
        self.preview_pos.insert(
            id,
            match visto {
                PreviewPos::Below => PreviewPos::Above,
                PreviewPos::Above => PreviewPos::Below,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A omissão é o que a DISPOSIÇÃO escolhe, e o botão do cabeçalho vira a partir dela**
    /// (doc 86 + o report do dono de 2026-09-20).
    ///
    /// ⛔ **A premissa antiga deste gate MORREU e está aqui para se ver:** ele dizia *«the
    /// preview default is Below»* e media `preview_position(7) == Below` sobre um mapa vazio.
    /// Hoje um mapa vazio não responde `Below` — responde o que a lei diz sobre AQUELA tela —, e
    /// é isso que as duas metades abaixo separam: um cartão que a lei deixa em baixo e um que
    /// ela põe em cima.
    ///
    /// ⚠️⚠️ **A segunda metade é a que vale, e ela nasceu de um defeito:** com um
    /// `entry().or_default()` o PRIMEIRO clique sobre um cartão que a lei já pôs em cima
    /// escrevia `Below` e virava para `Above` — *nada se mexia na tela*, e o artista lia um
    /// botão morto. FALSIFICADO por uma virada que parta de `Below` em vez de partir do que está
    /// na tela.
    #[test]
    fn toggling_flips_the_preview_position() {
        let mut st = MotionGraphPanelState::default();
        let ninguem = BTreeSet::new();
        let sete_em_cima = BTreeSet::from([7]);

        assert_eq!(
            st.preview_position(7, &ninguem),
            PreviewPos::Below,
            "sem virada e sem lei, em baixo"
        );
        st.toggle_preview_position(7, &ninguem);
        assert_eq!(st.preview_position(7, &ninguem), PreviewPos::Above);
        st.toggle_preview_position(7, &ninguem);
        assert_eq!(
            st.preview_position(7, &ninguem),
            PreviewPos::Below,
            "and back down"
        );
        assert_eq!(
            st.preview_position(9, &ninguem),
            PreviewPos::Below,
            "an untouched node is unaffected"
        );

        let mut st = MotionGraphPanelState::default();
        assert_eq!(
            st.preview_position(7, &sete_em_cima),
            PreviewPos::Above,
            "sem virada, a omissão é a LEI"
        );
        st.toggle_preview_position(7, &sete_em_cima);
        assert_eq!(
            st.preview_position(7, &sete_em_cima),
            PreviewPos::Below,
            "o PRIMEIRO clique tem de mover a moldura, e a lei perde para a mão"
        );
    }

    #[test]
    fn entering_or_leaving_a_level_re_fits_and_staying_put_does_not() {
        let mut st = MotionGraphPanelState {
            fitted: true,
            ..Default::default()
        };
        assert!(
            !st.sync_level(None),
            "already at the root: nothing happened"
        );
        assert!(st.fitted, "and the artist's pan/zoom is left alone");

        assert!(st.sync_level(Some(3)), "entered a group");
        assert!(!st.fitted, "so the next paint re-frames on its contents");

        st.fitted = true;
        assert!(!st.sync_level(Some(3)), "still in the same room");
        assert!(
            st.fitted,
            "a re-fit on every frame would fight the artist's zoom"
        );

        assert!(st.sync_level(None), "walked the breadcrumb back out");
        assert!(!st.fitted);
    }

    /// **Fit is selection-aware.** With a selection, a manual fit asks to frame it
    /// (the universal `F`); with nothing selected it frames the whole graph. Both
    /// drop `fitted` so the next paint re-frames. FALSIFIED by a `request_fit` that
    /// ignores the selection (always frames all, or always frames "selection").
    #[test]
    fn request_fit_frames_the_selection_only_when_there_is_one() {
        let mut st = MotionGraphPanelState {
            fitted: true,
            ..Default::default()
        };

        st.request_fit();
        assert!(!st.fitted, "an empty-selection fit still re-frames");
        assert!(!st.fit_selection, "nothing selected: frame the whole graph");

        st.selected.insert(7);
        st.fitted = true;
        st.request_fit();
        assert!(!st.fitted);
        assert!(
            st.fit_selection,
            "a selection is present: frame it, not everything"
        );
    }
}
