//! **O arrasto na tira** — mover a chave no tempo, esticar o hold.
//!
//! O gesto que toda tira de animação tem (TVPaint · Callipeg · Harmony · Procreate Dreams):
//! a célula é o desenho no tempo, então pegá-la e levá-la MOVE o desenho, e puxar a borda
//! direita muda por quantos quadros ele fica na tela. Até aqui isso era feito pelos botões
//! `◀`/`▶` e pela caixa **Hold** — o mesmo resultado, num caminho que não é o que a mão faz
//! (`docs/Flip/05 §6`, "follow-ups conscientes").
//!
//! ## MOVER muda o documento UMA vez, no fim; o HOLD estica em TEMPO REAL
//!
//! **Mover** desenha um contorno durante o percurso e escreve no `End`. A razão que morde:
//! o `index` do [`FlipStripHitKind`] é uma posição na lista de células **do frame do
//! Begin** — mover a chave a cada Update reordenaria a lista sob o gesto e o índice
//! passaria a apontar para outra chave (*a coordenada derivada tem de ser lida na mesma
//! referência em que foi semeada*, [[feedback_derived_coordinate_seed_must_match_sample]]).
//!
//! **O hold aplica a cada Update** (Enio, smoke 2026-07-24: *"em vez de fazer um preview ao
//! arrastar e aplicar quando soltar, melhor esticar e achatar em tempo real"*) — e aqui o
//! vivo é SEGURO pelas três razões que o tornavam perigoso no mover:
//!
//! 1. `set_exposure` **não move a chave arrastada nem reordena a lista** (só as seguintes
//!    deslizam de quadro) ⇒ a identidade do Begin sobrevive a cada aplicação;
//! 2. o undo continua **um passo por gesto** sem aprender nada: o `post_frame_undo` do
//!    shell suprime o auto-commit enquanto `held_button` está preso, então as N aplicações
//!    do percurso viram UM diff no soltar;
//! 3. a régua do gesto é **CONGELADA no Begin** (`StripDrag::ruler`) — ver abaixo.
//!
//! ## A régua do gesto é a do Begin, e isso é load-bearing
//!
//! Esticar muda o TOTAL de quadros da tira, e a tira sempre cabe (`ruler.rs`): mais quadros
//! ⇒ menos pixels por quadro ⇒ o MESMO x do ponteiro passa a ler um quadro MAIOR. Com a
//! régua viva isso é realimentação positiva — cada aplicação empurra o alvo mais para a
//! direita e um arrasto de um pixel dispararia a exposição sozinho. Congelada no Begin, o
//! mapeamento pixel→quadro é constante e o gesto é função só da mão. (O preço honesto: se a
//! tira re-escalar no meio, a borda desenhada pode não seguir o dedo pixel a pixel — o
//! gesto vira RELATIVO à escala da pegada, a mesma filosofia do arrasto relativo abaixo.)
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

/// A sessão viva de arrasto (estado de VISTA — o que o painel sabe do gesto).
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
    /// **Hold vivo:** a última exposição JÁ pedida ao documento — o dedup do
    /// emite-quando-muda (pedir por pixel encheria a fila com no-ops). MoveKey não a lê.
    pub(crate) applied: i32,
    /// A régua CONGELADA no Begin — o mapeamento pixel→quadro do GESTO (ver o doc do
    /// módulo: com a régua viva, o hold realimenta a própria escala).
    pub(crate) ruler: StripRuler,
}

impl StripDrag {
    /// O que um arrasto de MOVER pede ao documento no `End` (`None` = o dedo tremeu, o
    /// alvo é onde a chave já está).
    fn move_intent(&self) -> Option<FlipStripIntent> {
        (self.target != self.key).then_some(FlipStripIntent::MoveKey {
            from: self.key,
            to: self.target,
        })
    }
}

/// **Um passo do hold VIVO**: recomputa o alvo com a régua congelada e pede a exposição
/// nova ao documento quando ela muda de inteiro. Chamado pelo Update E pelo End — o
/// ponteiro ainda anda entre o último Move e o soltar.
///
/// A exposição é medida da CHAVE ao ponteiro (a borda direita segue o dedo), não pelo
/// delta: é a largura da célula que o artista está ajustando, e ele a lê na tela.
fn hold_step(d: &mut StripDrag, x: f32) {
    let here = d.ruler.frame_at_x(x);
    d.target = (here - d.key + 1).max(1);
    if d.target != d.applied {
        push_intent(FlipStripIntent::SetHold {
            key: d.key,
            frames: d.target as u32,
        });
        d.applied = d.target;
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
/// os pedidos do arrasto (hold: a cada mudança de alvo; mover: no fim).
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
    // A mesma pergunta, na outra ponta: o índice ainda existe mas a chave dele TROCOU.
    // No hold vivo o snapshot muda por construção (é o gesto aplicando), mas a chave
    // arrastada nunca muda de quadro sob `set_exposure` — se ela trocou, foi outra via.
    if g.phase != GesturePhase::Begin
        && drag.as_ref().is_some_and(|d| d.key != cell.key)
    {
        *drag = None;
        return None;
    }
    match g.phase {
        GesturePhase::Begin => {
            let start = match kind {
                DragKind::MoveKey => cell.key,
                DragKind::Hold => cell.exposure.max(1) as i32,
            };
            *drag = Some(StripDrag {
                kind,
                key: cell.key,
                grab_frame: ruler.frame_at_x(g.x),
                target: start,
                applied: start,
                ruler: *ruler,
            });
            None
        }
        GesturePhase::Update => {
            if let Some(d) = drag.as_mut() {
                match d.kind {
                    DragKind::MoveKey => {
                        let here = d.ruler.frame_at_x(g.x);
                        let (lo, hi) = move_bounds(snap, index);
                        d.target = (d.key + here - d.grab_frame).clamp(lo, hi); // CLAMP-OK: vizinhas
                    }
                    DragKind::Hold => hold_step(d, g.x),
                }
            }
            None
        }
        GesturePhase::End => {
            if let Some(mut d) = drag.take() {
                match d.kind {
                    DragKind::MoveKey => {
                        if let Some(intent) = d.move_intent() {
                            push_intent(intent);
                        }
                    }
                    // O documento já acompanhou o percurso; o End só honra o resto do
                    // caminho entre o último Move e o soltar.
                    DragKind::Hold => hold_step(&mut d, g.x),
                }
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

/// **Onde o contorno do arrasto é desenhado** — só para MOVER (o hold não tem preview: a
/// própria célula estica em tempo real). `None` sem sessão, ou quando ela ainda não pede
/// nada (o dedo tremeu dentro do próprio quadro: um contorno em cima da célula onde ela já
/// está seria ruído).
pub(crate) fn preview_rect(
    state: &crate::state::FlipStripState,
    ruler: &StripRuler,
    snap: &FlipStripSnapshot,
) -> Option<Rect> {
    let d = state.drag?;
    if d.kind != DragKind::MoveKey || d.target == d.key {
        return None;
    }
    // A chave viaja inteira: a largura é a exposição que ela tem hoje.
    let frames = exposure_of(snap, d.key).unwrap_or(1) as i32;
    Some(Rect::new(
        ruler.x_of_frame(d.target),
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
                    ph2d_editor_core::tool::PanelEvent::Click(ph2d_editor_core::ids::flip_cell_id(
                        index,
                    )),
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
    /// (a fila teria um pedido por passo do percurso — e o MOVER reordenaria a lista sob o
    /// próprio índice, a razão de ele NÃO ser vivo).
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

    /// 🔴 **O hold é VIVO** (Enio, smoke 2026-07-24): o pedido sai no UPDATE — a célula
    /// estica na tela enquanto o dedo anda — e o End não duplica nada quando o ponteiro
    /// não andou mais. Dois Updates no mesmo alvo pedem UMA vez (emite-quando-muda).
    /// Mutação que sangra: voltar a aplicar só no End (o 1º drain sai vazio).
    #[test]
    fn dragging_the_hold_edge_stretches_the_exposure_live() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let kind = FlipStripHitKind::HoldEdge { index: 0 };
        let edge = r
            .hold_edge_rect(0, &s)
            .expect("a célula de 4 quadros tem grip");
        let start = edge.x + edge.w * 0.5;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Begin, start));
        // Leva a borda até o meio do quadro 6 ⇒ exposição 7 (quadros 0..=6) — JÁ no Update.
        let x6 = r.x_of_frame(6) + r.ppf * 0.5;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x6));
        assert_eq!(
            crate::state::drain_flip_strip_intents(),
            vec![FlipStripIntent::SetHold { key: 0, frames: 7 }],
            "o hold aplica enquanto o ponteiro anda, não no soltar"
        );
        // Mais um Update no MESMO lugar: alvo igual, nada novo na fila.
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x6));
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, x6));
        assert!(
            crate::state::drain_flip_strip_intents().is_empty(),
            "alvo que não mudou não pede de novo — nem no End"
        );
        assert!(d.is_none(), "o End encerra a sessão");
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
        apply(
            &mut d,
            &r,
            &s,
            gesture(kind, GesturePhase::Update, far_left),
        );
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::End, far_left));
        assert_eq!(
            crate::state::drain_flip_strip_intents(),
            vec![FlipStripIntent::SetHold { key: 4, frames: 1 }]
        );
    }

    /// 🔴 **O mapeamento do gesto é o do Begin.** O hold vivo muda o total de quadros, a
    /// tira re-escala, e uma régua VIVA leria o MESMO x como um quadro maior — realimentação
    /// positiva: a exposição dispararia sozinha. A sessão congela a régua da pegada, então
    /// o mesmo x continua pedindo o mesmo alvo mesmo com a tira já esticada.
    /// Mutação que sangra: `hold_step` ler a régua do frame em vez de `d.ruler`.
    #[test]
    fn the_holds_mapping_is_frozen_at_the_grab() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let mut d = None;
        let kind = FlipStripHitKind::HoldEdge { index: 2 };
        let edge = r.hold_edge_rect(2, &s).expect("grip da última");
        apply(
            &mut d,
            &r,
            &s,
            gesture(kind, GesturePhase::Begin, edge.x + edge.w * 0.5),
        );
        // Estica a última em +2 quadros (4 → 6).
        let x = r.x_of_frame(13) + r.ppf * 0.5;
        apply(&mut d, &r, &s, gesture(kind, GesturePhase::Update, x));
        assert_eq!(
            crate::state::drain_flip_strip_intents(),
            vec![FlipStripIntent::SetHold { key: 8, frames: 6 }]
        );
        // O documento acompanhou (é o hold vivo): o snapshot do frame seguinte tem a
        // exposição nova, e a régua RESOLVIDA dele lê o mesmo x como OUTRO quadro — a
        // fixture tem de conter o fenômeno, senão a mutação não sangra.
        let s2 = FlipStripSnapshot {
            has_layer: true,
            cells: vec![cell(0, 4), cell(4, 4), cell(8, 6)],
            ..Default::default()
        };
        let r2 = StripRuler::resolve(Rect::new(0.0, 0.0, 120.0, 100.0), &s2).expect("régua");
        assert_ne!(
            r.frame_at_x(x),
            r2.frame_at_x(x),
            "pré-condição: a tira re-escalou e a régua viva discorda da congelada"
        );
        // O mesmo x, sob o snapshot novo e a régua nova: o alvo NÃO anda sozinho.
        apply(&mut d, &r2, &s2, gesture(kind, GesturePhase::Update, x));
        apply(&mut d, &r2, &s2, gesture(kind, GesturePhase::End, x));
        assert!(
            crate::state::drain_flip_strip_intents().is_empty(),
            "com a régua congelada, o mesmo x pede o mesmo alvo — sem realimentação"
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
    /// vez de agir sobre a chave que passou a ocupar aquele índice — nas DUAS formas de
    /// ficar obsoleto: o índice não existe mais, ou existe com OUTRA chave dentro.
    #[test]
    fn a_stale_index_drops_the_session_instead_of_moving_the_wrong_key() {
        let (r, s) = fixture();
        let _ = crate::state::drain_flip_strip_intents();
        let session = StripDrag {
            kind: DragKind::MoveKey,
            key: 4,
            grab_frame: 4,
            target: 6,
            applied: 4,
            ruler: r,
        };
        // Índice fora da lista.
        let mut d = Some(session);
        let gone = FlipStripHitKind::Cell { index: 9 };
        assert_eq!(
            apply(&mut d, &r, &s, gesture(gone, GesturePhase::Update, 10.0)),
            None
        );
        assert!(d.is_none());
        // Índice válido, chave trocada (a sessão diz 4; o índice 0 guarda a chave 0).
        let mut d = Some(session);
        let swapped = FlipStripHitKind::Cell { index: 0 };
        assert_eq!(
            apply(&mut d, &r, &s, gesture(swapped, GesturePhase::Update, 10.0)),
            None
        );
        assert!(d.is_none());
        assert!(crate::state::drain_flip_strip_intents().is_empty());
    }
}
