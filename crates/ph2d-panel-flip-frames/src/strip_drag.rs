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
//!
//! ## A SELEÇÃO viaja junta (o follow-up nomeado da wave)
//!
//! Pegar uma célula **marcada** (multiframe, W7) move a seleção INTEIRA pelo mesmo delta —
//! o idioma de todo dope-sheet (Blender/TVPaint): marcou N quadros, o gesto age nos N.
//! Pegar uma célula NÃO marcada segue movendo só ela (a seleção não é tocada). Três fatos
//! carregam o desenho:
//!
//! 1. **O limite é dos vizinhos NÃO marcados**: o grupo anda rígido (os espaçamentos
//!    internos não mudam), então marcada não colide com marcada — quem para o grupo é a
//!    primeira chave não marcada que alguma marcada alcançaria, e o piso `0` da primeira.
//!    O delta permitido é a interseção dos limites por-chave (`selection_delta_bounds`).
//! 2. **A ordem de emissão é quem garante que todo `move_frame` pousa**: duas marcadas
//!    adjacentes movidas `+1` colidem se a da esquerda anda primeiro (o destino ainda está
//!    ocupado pela irmã) — para a direita move-se da DIREITA para a esquerda; para a
//!    esquerda, o inverso. Com os limites do item 1, essa ordem torna a recusa do
//!    `move_frame` inalcançável.
//! 3. **Uma célula marcada sozinha é o gesto de sempre**: os limites contra vizinhos não
//!    marcados degeneram nos limites por-índice, e a emissão é um pedido só — o caso comum
//!    (clique seleciona a célula, depois arrasta) não muda um byte de comportamento.

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
    /// **A célula pega estava MARCADA no Begin** ⇒ o arrasto move a seleção inteira
    /// (só MoveKey; o hold segue por-célula). O conjunto em si é relido do snapshot —
    /// ele não muda sob o ponteiro capturado, e guardá-lo aqui seria a 2ª cópia.
    pub(crate) group: bool,
}

impl StripDrag {
    /// O que um arrasto de MOVER pede ao documento no `End` — nada quando o dedo tremeu
    /// (o alvo é onde a chave já está). No **grupo** (célula marcada), um pedido por chave
    /// marcada, na ordem que garante que todo `move_frame` pousa: movendo para a DIREITA,
    /// da direita para a esquerda (a vizinha marcada já saiu do destino quando a irmã
    /// chega); para a esquerda, o inverso.
    fn push_move_intents(&self, snap: &FlipStripSnapshot) {
        let delta = self.target - self.key;
        if delta == 0 {
            return;
        }
        if !self.group {
            push_intent(FlipStripIntent::MoveKey {
                from: self.key,
                to: self.target,
            });
            return;
        }
        let keys = snap.cells.iter().filter(|c| c.selected).map(|c| c.key);
        let emit = |k: i32| {
            push_intent(FlipStripIntent::MoveKey {
                from: k,
                to: k + delta,
            });
        };
        if delta > 0 {
            keys.rev().for_each(emit);
        } else {
            keys.for_each(emit);
        }
    }
}

/// **Quanto a SELEÇÃO inteira pode andar** — `(delta mínimo, delta máximo)`.
///
/// O grupo move rígido, então marcada nunca colide com marcada; o limite de cada chave
/// marcada é o vizinho **não marcado** mais próximo de cada lado (ele fica parado), mais o
/// piso `0` à esquerda (o mesmo da chave única). O delta permitido é a interseção: a chave
/// mais apertada trava o grupo inteiro — encostar e parar, como no gesto de uma célula.
fn selection_delta_bounds(snap: &FlipStripSnapshot) -> (i32, i32) {
    let mut dmin = i32::MIN;
    let mut dmax = i32::MAX;
    for (i, c) in snap.cells.iter().enumerate() {
        if !c.selected {
            continue;
        }
        let lo = snap.cells[..i]
            .iter()
            .rev()
            .find(|n| !n.selected)
            .map_or(0, |n| n.key + 1);
        dmin = dmin.max(lo - c.key);
        if let Some(n) = snap.cells[i + 1..].iter().find(|n| !n.selected) {
            dmax = dmax.min(n.key - 1 - c.key);
        }
    }
    // Chave ordenada e vizinho não marcado ficam sempre do lado certo (dmin ≤ 0 ≤ dmax);
    // o guard só existe para um snapshot malformado não virar pânico de `clamp`.
    (dmin, dmax.max(dmin))
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
    // E um arrasto de GRUPO cuja célula pega deixou de estar marcada é a mesma coisa: a
    // sessão descreve um conjunto que não existe mais.
    if g.phase != GesturePhase::Begin
        && drag
            .as_ref()
            .is_some_and(|d| d.key != cell.key || (d.group && !cell.selected))
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
                group: kind == DragKind::MoveKey && cell.selected,
            });
            None
        }
        GesturePhase::Update => {
            if let Some(d) = drag.as_mut() {
                match d.kind {
                    DragKind::MoveKey => {
                        let here = d.ruler.frame_at_x(g.x);
                        let want = here - d.grab_frame;
                        // No grupo o clamp é no DELTA (os limites vêm em delta, e um
                        // `key + i32::MAX` estouraria); na célula única, no alvo.
                        d.target = if d.group {
                            let (dlo, dhi) = selection_delta_bounds(snap);
                            d.key + want.clamp(dlo, dhi) // CLAMP-OK: vizinhas
                        } else {
                            let (lo, hi) = move_bounds(snap, index);
                            (d.key + want).clamp(lo, hi) // CLAMP-OK: vizinhas
                        };
                    }
                    DragKind::Hold => hold_step(d, g.x),
                }
            }
            None
        }
        GesturePhase::End => {
            if let Some(mut d) = drag.take() {
                match d.kind {
                    DragKind::MoveKey => d.push_move_intents(snap),
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

/// **Onde os contornos do arrasto são desenhados** — só para MOVER (o hold não tem
/// preview: a própria célula estica em tempo real). Vazio sem sessão, ou quando ela ainda
/// não pede nada (o dedo tremeu dentro do próprio quadro: um contorno em cima da célula
/// onde ela já está seria ruído). Num arrasto de GRUPO, um contorno por célula marcada —
/// cada uma no seu destino, com a SUA largura: o gesto mostra tudo o que vai mudar.
pub(crate) fn preview_rects(
    state: &crate::state::FlipStripState,
    ruler: &StripRuler,
    snap: &FlipStripSnapshot,
) -> Vec<Rect> {
    let Some(d) = state.drag else {
        return Vec::new();
    };
    if d.kind != DragKind::MoveKey || d.target == d.key {
        return Vec::new();
    }
    let ghost = |frame: i32, exposure: u32| {
        Rect::new(
            ruler.x_of_frame(frame),
            ruler.cells_top,
            (exposure.max(1) as f32 * ruler.ppf - 1.0).max(crate::ruler::MIN_CELL_W),
            ruler.cell_h,
        )
    };
    if d.group {
        let delta = d.target - d.key;
        snap.cells
            .iter()
            .filter(|c| c.selected)
            .map(|c| ghost(c.key + delta, c.exposure))
            .collect()
    } else {
        // A chave viaja inteira: a largura é a exposição que ela tem hoje.
        vec![ghost(d.target, exposure_of(snap, d.key).unwrap_or(1))]
    }
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
#[path = "strip_drag_tests.rs"]
mod tests;
