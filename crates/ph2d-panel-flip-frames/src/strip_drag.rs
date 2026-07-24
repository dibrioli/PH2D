//! **O arrasto na tira** — mover a chave no tempo, esticar o hold.
//!
//! O gesto que toda tira de animação tem (TVPaint · Callipeg · Harmony · Procreate Dreams):
//! a célula é o desenho no tempo, então pegá-la e levá-la MOVE o desenho, e puxar a borda
//! direita muda por quantos quadros ele fica na tela. Até aqui isso era feito pelos botões
//! `◀`/`▶` e pela caixa **Hold** — o mesmo resultado, num caminho que não é o que a mão faz
//! (`docs/Flip/05 §6`, "follow-ups conscientes").
//!
//! ## O documento muda UMA vez, no fim
//!
//! Durante o percurso, o painel desenha para onde a chave VAI; quem escreve é o `End`. Duas
//! razões, e a segunda é a que morde:
//!
//! 1. um gesto = **um passo de undo**, sem precisar ensinar a fila global a reconhecer um
//!    arrasto em curso;
//! 2. o `index` do [`FlipStripHitKind`] é uma posição na lista de células **do frame do
//!    Begin**. Se o documento mudasse a cada Update, a lista se reordenaria sob o gesto e o
//!    índice passaria a apontar para outra chave — o clássico *a coordenada derivada tem de
//!    ser lida na mesma referência em que foi semeada*
//!    ([[feedback_derived_coordinate_seed_must_match_sample]]). Aplicando no fim, a
//!    referência não pode se mover.
//!
//! ## O arrasto é RELATIVO ao ponto de pega
//!
//! O alvo é `chave + (quadro sob o ponteiro − quadro sob o ponteiro no Begin)`, nunca o
//! quadro absoluto sob o cursor: com o absoluto a célula SALTA para debaixo do dedo no
//! primeiro pixel, e pegar uma célula larga pela direita a jogaria vários quadros atrás. É a
//! mesma escolha (pelo mesmo motivo) da alça de duração da timeline.

use crate::ruler::StripRuler;
use crate::state::{FlipStripIntent, FlipStripSnapshot, push_intent};
use ph2d_editor_core::interaction::{FlipStripGesture, FlipStripHitKind, GesturePhase};
use ph2d_editor_core::zones::Rect;

/// Qual dos dois verbos o arrasto está exercendo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum DragKind {
    /// O corpo da célula: a chave anda no tempo.
    MoveKey,
    /// A borda direita: a exposição cresce ou encolhe.
    Hold,
}

/// A sessão viva de arrasto (estado de VISTA — o que o painel desenha antes de o documento
/// saber).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct StripDrag {
    pub(crate) kind: DragKind,
    /// A chave pega (do snapshot do Begin) — a identidade que sobrevive ao percurso; o
    /// índice, não.
    pub(crate) key: i32,
    /// O quadro sob o ponteiro no Begin (a âncora do arrasto relativo).
    pub(crate) grab_frame: i32,
    /// **MoveKey:** o quadro-destino, já clampado. **Hold:** a exposição resultante.
    pub(crate) target: i32,
}

impl StripDrag {
    /// O gesto ainda não pede nada? (o dedo tremeu, mas o alvo é o que já era).
    fn is_noop(&self, snap: &FlipStripSnapshot) -> bool {
        match self.kind {
            DragKind::MoveKey => self.target == self.key,
            DragKind::Hold => exposure_of(snap, self.key).is_some_and(|e| e as i32 == self.target),
        }
    }

    /// O que este arrasto pede ao documento (`None` se não pede nada).
    fn intent(&self, snap: &FlipStripSnapshot) -> Option<FlipStripIntent> {
        if self.is_noop(snap) {
            return None;
        }
        Some(match self.kind {
            DragKind::MoveKey => FlipStripIntent::MoveKey {
                from: self.key,
                to: self.target,
            },
            DragKind::Hold => FlipStripIntent::SetHold {
                key: self.key,
                frames: self.target.max(1) as u32,
            },
        })
    }
}

/// A exposição da chave `key` no snapshot.
fn exposure_of(snap: &FlipStripSnapshot, key: i32) -> Option<u32> {
    snap.cells
        .iter()
        .find(|c| c.key == key)
        .map(|c| c.exposure.max(1))
}

/// **Até onde a chave `i` pode ir** — `(mínimo, máximo)` em quadros.
///
/// Uma chave não pode passar por cima da vizinha: `move_frame` RECUSA um destino ocupado
/// (devolve `false`), e um gesto que às vezes não faz nada é pior que um gesto que para —
/// o artista não descobre a regra, descobre a intermitência. Então o arrasto **encosta** na
/// vizinha, como o trim de uma strip encosta na próxima.
///
/// Os dois extremos são abertos de propósito: a primeira chave para em **0** (o tempo do
/// objeto começa ali, e o que estivesse antes seria invisível na tira, cuja escala sai do
/// vão) e a última **não tem teto** — arrastá-la para a direita é como se estende uma cena.
fn move_bounds(snap: &FlipStripSnapshot, i: usize) -> (i32, i32) {
    let prev = i.checked_sub(1).and_then(|p| snap.cells.get(p));
    let next = snap.cells.get(i + 1);
    let lo = prev.map_or(0, |c| c.key + 1);
    let hi = next.map_or(i32::MAX, |c| c.key - 1);
    (lo, hi.max(lo))
}

/// Alimenta a sessão com um gesto. Função **pura** sobre o estado do painel: devolve o
/// índice da célula TOCADA (um toque, que segue saindo por `PanelEvent::Click`) e enfileira
/// o pedido do arrasto quando ele termina.
pub(crate) fn apply(
    drag: &mut Option<StripDrag>,
    ruler: &StripRuler,
    snap: &FlipStripSnapshot,
    g: FlipStripGesture,
) -> Option<usize> {
    let (index, kind) = match g.kind {
        FlipStripHitKind::Cell { index } => (index as usize, DragKind::MoveKey),
        FlipStripHitKind::HoldEdge { index } => (index as usize, DragKind::Hold),
    };
    let Some(cell) = snap.cells.get(index) else {
        // A tira mudou embaixo do gesto (o documento foi editado por outra via). Largar é
        // mais honesto que agir sobre a chave errada.
        *drag = None;
        return None;
    };
    match g.phase {
        GesturePhase::Begin => {
            *drag = Some(StripDrag {
                kind,
                key: cell.key,
                grab_frame: ruler.frame_at_x(g.x),
                target: match kind {
                    DragKind::MoveKey => cell.key,
                    DragKind::Hold => cell.exposure.max(1) as i32,
                },
            });
            None
        }
        GesturePhase::Update => {
            if let Some(d) = drag.as_mut() {
                let here = ruler.frame_at_x(g.x);
                match d.kind {
                    DragKind::MoveKey => {
                        let (lo, hi) = move_bounds(snap, index);
                        d.target = (d.key + here - d.grab_frame).clamp(lo, hi); // CLAMP-OK: vizinhas
                    }
                    // A exposição é medida da CHAVE ao ponteiro (a borda direita segue o
                    // dedo), não pelo delta: é a largura da célula que o artista está
                    // ajustando, e ele a lê na tela enquanto arrasta.
                    DragKind::Hold => d.target = (here - d.key + 1).max(1),
                }
            }
            None
        }
        GesturePhase::End => {
            if let Some(d) = drag.take()
                && let Some(intent) = d.intent(snap)
            {
                push_intent(intent);
            }
            None
        }
        // Um toque: a sessão morre sem pedir nada, e o índice volta para o chamador emitir
        // o `PanelEvent::Click` de sempre — selecionar uma chave não mudou de rota.
        GesturePhase::Click | GesturePhase::DoubleClick => {
            *drag = None;
            Some(index)
        }
    }
}

/// **Onde o preview do arrasto é desenhado** — `None` sem sessão, ou quando ela ainda não
/// pede nada (o dedo tremeu dentro do próprio quadro: desenhar um contorno em cima da
/// célula onde ela já está seria ruído).
pub(crate) fn preview_rect(
    state: &crate::state::FlipStripState,
    ruler: &StripRuler,
    snap: &FlipStripSnapshot,
) -> Option<Rect> {
    let d = state.drag?;
    if d.is_noop(snap) {
        return None;
    }
    let (start, frames) = match d.kind {
        // A chave viaja inteira: a largura é a exposição que ela tem hoje.
        DragKind::MoveKey => (d.target, exposure_of(snap, d.key).unwrap_or(1) as i32),
        // A chave fica: o que muda é o quanto ela mede.
        DragKind::Hold => (d.key, d.target.max(1)),
    };
    Some(Rect::new(
        ruler.x_of_frame(start),
        ruler.cells_top,
        (frames as f32 * ruler.ppf - 1.0).max(crate::ruler::MIN_CELL_W),
        ruler.cell_h,
    ))
}

/// Drena os gestos da tira deste frame e os aplica à sessão.
///
/// Roda no PAINT (é lá que a geometria existe), antes de pintar. O **toque** volta a sair
/// por `PanelEvent::Click(flip_cell_id(i))` — exatamente o evento que o `apply_event`
/// empurrava quando a célula era um botão, então o shell não distingue as duas eras e a
/// multi-seleção com modificador (que o shell lê do seu próprio estado) segue intacta.
pub(crate) fn process(
    state: &mut crate::state::FlipStripState,
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    area: Rect,
    snap: &FlipStripSnapshot,
) {
    let gestures: Vec<FlipStripGesture> =
        ctx.host.store_mut().drain_flip_strip_gestures().collect();
    if gestures.is_empty() {
        return;
    }
    let Some(ruler) = StripRuler::resolve(area, snap) else {
        state.drag = None;
        return;
    };
    for g in gestures {
        if let Some(index) = apply(&mut state.drag, &ruler, snap, g) {
            ctx.host
                .bus_mut()
                .push(ph2d_editor_core::action_bus::EditorAction::ToolPanelEvent(
                    ph2d_editor_core::tool::PanelEvent::Click(
                        ph2d_editor_core::ids::flip_cell_id(index),
                    ),
                ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::FlipCell;
    use ph2d_editor_core::interaction::GestureMods;
    use ph2d_host::PointerButton;

    fn cell(key: i32, exposure: u32) -> FlipCell {
        FlipCell {
            key,
            exposure,
            breakdown: false,
            instanced: false,
            selected: false,
            pinned: false,
            weight: 1.0,
        }
    }

    /// Chaves em 0/4/8, cada uma expondo 4 — e uma faixa estreita o bastante para o teto de
    /// px-por-quadro não mascarar a escala.
    fn fixture() -> (StripRuler, FlipStripSnapshot) {
        let snap = FlipStripSnapshot {
            has_layer: true,
            cells: vec![cell(0, 4), cell(4, 4), cell(8, 4)],
            ..Default::default()
        };
        let ruler = StripRuler::resolve(Rect::new(0.0, 0.0, 120.0, 100.0), &snap).expect("régua");
        (ruler, snap)
    }

    fn gesture(kind: FlipStripHitKind, phase: GesturePhase, x: f32) -> FlipStripGesture {
        FlipStripGesture {
            surface: ph2d_a11y::NodeId(1),
            kind,
            phase,
            x,
            y: 0.0,
            button: PointerButton::Primary,
            mods: GestureMods::default(),
        }
    }

    /// Arrasta a célula `i` do meio dela até o quadro `to_frame`, e devolve o que ficou na
    /// fila de pedidos.
    fn drag_cell(i: u16, to_frame: i32) -> Vec<FlipStripIntent> {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents(); // a fila é thread_local
        let mut d = None;
        let body = r.cell_rect(i as usize, &s).unwrap();
        let start = body.x + body.w * 0.5;
        let kind = FlipStripHitKind::Cell { index: i };
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, start));
        // O ponteiro anda o mesmo tanto que o alvo tem de andar.
        let delta = (to_frame - s.cells[i as usize].key) as f32 * r.ppf;
        apply(
            &mut d,
            &r,
            &s,
            gesture(kind, GesturePhase::Update, start + delta),
        );
        apply(
            &mut d,
            &r,
            &s,
            gesture(kind, GesturePhase::End, start + delta),
        );
        crate::state::drain_flip_strip_intents()
    }

    /// 🔴 O gesto central: pegar a chave do meio e largá-la um quadro adiante pede
    /// exatamente `move_frame(4 → 5)`. Mutação que sangra: aplicar no Update em vez do End
    /// (a fila teria um pedido por passo do percurso, e o undo, um passo por pixel).
    #[test]
    fn dragging_a_cell_asks_to_move_its_key() {
        assert_eq!(
            drag_cell(1, 5),
            vec![FlipStripIntent::MoveKey { from: 4, to: 5 }]
        );
    }

    /// 🔴 **O arrasto é relativo ao ponto de pega** — pegar a célula pelo MEIO e não mover o
    /// ponteiro não move a chave. Com alvo absoluto (`target = frame sob o cursor`) a chave
    /// saltaria para debaixo do dedo no primeiro Update: aqui, dois quadros adiante.
    #[test]
    fn grabbing_a_wide_cell_off_centre_does_not_teleport_it() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let body = r.cell_rect(1, &s).unwrap();
        let grab = body.x + body.w * 0.75; // bem à direita do começo da célula
        let kind = FlipStripHitKind::Cell { index: 1 };
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, grab));
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, grab));
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, grab));
        assert!(
            crate::state::drain_flip_strip_intents().is_empty(),
            "sem percurso não há pedido — a chave não pode saltar para o dedo"
        );
    }

    /// 🔴 **A chave encosta na vizinha, não passa por cima dela.** `move_frame` RECUSA um
    /// destino ocupado, e um gesto que às vezes não faz nada ensina intermitência.
    #[test]
    fn a_key_stops_against_its_neighbour_instead_of_being_refused() {
        // A do meio (4) puxada para 99: a vizinha começa em 8 ⇒ para em 7.
        assert_eq!(
            drag_cell(1, 99),
            vec![FlipStripIntent::MoveKey { from: 4, to: 7 }]
        );
        // E para trás: a anterior começa em 0 ⇒ para em 1.
        assert_eq!(
            drag_cell(1, -99),
            vec![FlipStripIntent::MoveKey { from: 4, to: 1 }]
        );
    }

    /// A PRIMEIRA chave para em 0 (o tempo começa ali) e a ÚLTIMA não tem teto — arrastá-la
    /// para a direita é como uma cena se estende.
    #[test]
    fn the_first_key_stops_at_zero_and_the_last_one_has_no_ceiling() {
        assert!(
            drag_cell(0, -5).is_empty(),
            "a primeira já está em 0: puxar para trás não pede nada"
        );
        assert_eq!(
            drag_cell(2, 20),
            vec![FlipStripIntent::MoveKey { from: 8, to: 20 }]
        );
    }

    /// 🔴 A borda direita estica a EXPOSIÇÃO, medida da chave ao ponteiro — e o pedido é o
    /// `set_exposure`, que EMPURRA as seguintes (a semântica da tira, já testada no modelo).
    #[test]
    fn dragging_the_hold_edge_asks_for_the_new_exposure() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let kind = FlipStripHitKind::HoldEdge { index: 0 };
        let edge = r.hold_edge_rect(0, &s).expect("a célula de 4 quadros tem grip");
        let start = edge.x + edge.w * 0.5;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, start));
        // Leva a borda até o meio do quadro 6 ⇒ exposição 7 (quadros 0..=6).
        let x6 = r.x_of_frame(6) + r.ppf * 0.5;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x6));
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, x6));
        assert_eq!(
            crate::state::drain_flip_strip_intents(),
            vec![FlipStripIntent::SetHold { key: 0, frames: 7 }]
        );
    }

    /// A exposição nunca chega a zero: arrastar a borda para trás da própria chave a deixa
    /// em 1 quadro (um desenho que fica zero quadro na tela não é um desenho, é um delete —
    /// e delete é outro botão).
    #[test]
    fn the_hold_never_shrinks_below_one_frame() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let kind = FlipStripHitKind::HoldEdge { index: 1 };
        let edge = r.hold_edge_rect(1, &s).unwrap();
        apply(
            &mut d,
            &r,
            &s,
            gesture(kind, GesturePhase::Begin, edge.x + edge.w * 0.5),
        );
        let far_left = r.x_of_frame(-20);
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, far_left));
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, far_left));
        assert_eq!(
            crate::state::drain_flip_strip_intents(),
            vec![FlipStripIntent::SetHold { key: 4, frames: 1 }]
        );
    }

    /// 🔴 **Um toque não pede nada e devolve a célula** — é o `PanelEvent::Click` de sempre
    /// (selecionar a chave, com o modificador que o shell lê). Se o toque começasse a pedir
    /// um `MoveKey{from: k, to: k}`, todo clique viraria uma edição no-op na fila de undo.
    #[test]
    fn a_tap_asks_for_nothing_and_reports_the_cell() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let kind = FlipStripHitKind::Cell { index: 2 };
        let x = r.cell_rect(2, &s).unwrap().x + 1.0;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, x));
        let tapped = apply(&mut d, &r, &s, gesture(kind, GesturePhase::Click, x));
        assert_eq!(tapped, Some(2));
        assert!(crate::state::drain_flip_strip_intents().is_empty());
        assert!(d.is_none(), "o toque encerra a sessão");
    }

    /// A tira mudou embaixo do gesto (outra via editou o documento): a sessão é largada em
    /// vez de agir sobre a chave que passou a ocupar aquele índice.
    #[test]
    fn a_stale_index_drops_the_session_instead_of_moving_the_wrong_key() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = Some(StripDrag {
            kind: DragKind::MoveKey,
            key: 4,
            grab_frame: 4,
            target: 6,
        });
        let kind = FlipStripHitKind::Cell { index: 9 }; // não existe
        assert_eq!(
            apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, 10.0)),
            None
        );
        assert!(d.is_none());
        assert!(crate::state::drain_flip_strip_intents().is_empty());
    }
}
