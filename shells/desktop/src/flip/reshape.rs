//! ADR-0114 W5 — **a escultura de traço** do lado do shell: a fronteira entre o
//! gesto (px de tela, botão, Ctrl) e o solver (`ph2d-flip-reshape`, espaço local).
//!
//! Três decisões moram nesta fronteira:
//!
//! 1. **O desenho-alvo vem do autokey por-tool com política `Modify`** — esculpir é
//!    MODIFICAR o que está na tela. No rabo de um hold, a chave nova nasce como
//!    duplicata do desenho visível, nunca em branco (esculpir um quadro vazio e
//!    invisível seria o mesmo desastre da borracha, `docs/Flip/05 §4`).
//! 2. **O raio e o delta descem para o espaço LOCAL do objeto** (ADR-0111): a
//!    geometria de um objeto já movido pelo gizmo é local, então o cursor, o quanto
//!    ele andou e o raio do pincel recuam pela escala do objeto. Num objeto na
//!    identidade (o comum) isso é um no-op.
//! 3. **A sessão nasce no pen-down e morre no pen-up.** É ela que carrega a máscara
//!    congelada (e, no Grab, os pesos) — sem isso o gesto recrutaria geometria nova
//!    enquanto anda e o resultado dependeria do caminho do mouse.

use ph2d_core::Vec2;
use ph2d_flip::LayerId;
use ph2d_flip_reshape::{InputSample, ReshapeKind, ReshapeParams, Session};
use ph2d_tool_flip::FlipStyleSnapshot;
use ph2d_vec_scene::Xform;

/// Um desenho que este gesto esculpe: a sessão DELE (máscara congelada + pesos do Grab)
/// e o falloff temporal DELE.
pub(crate) struct ReshapeTarget {
    pub(crate) did: ph2d_flip::DrawingId,
    pub(crate) session: Session,
    /// `1.0` no quadro ativo; menor nos vizinhos (W7 — `flip_multiframe`).
    pub(crate) falloff: f32,
}

// **Por que a POSE da chave (W7.2) NÃO é compensada aqui** — a pergunta que o próximo
// leitor vai fazer, e a resposta não é óbvia (eu escrevi a compensação primeiro, e um
// gate a derrubou).
//
// O cursor chega convertido pela pose do quadro ATIVO. Um alvo noutra chave, deslocada,
// tem a arte em OUTRO lugar da tela — então "o mesmo ponto do MUNDO" cairia no vazio para
// ele, e o multiframe silenciosamente só editaria o quadro ativo sempre que as poses
// diferissem.
//
// O multiframe é ancorado na **ARTE**, não no mundo: as mesmas coordenadas na geometria
// de cada desenho são a mesma parte do personagem. É o que o animador quer dizer com
// *"conserta o cotovelo em todos os quadros que marquei"* — mesmo que os quadros estejam
// espalhados pela tela (que é justamente o que uma pose faz, e o motivo de ela existir:
// um ciclo que ANDA). Por isso a mesma `InputSample` vai para todos os alvos.
//
// Gate: `multiframe_is_art_anchored_not_world_anchored`.

/// O gesto de escultura em curso (o que o `App` guarda entre o down e o up).
pub(crate) struct FlipReshape {
    /// **Um alvo por DESENHO** (W7 — multiframe): com chaves selecionadas na tira, o MESMO
    /// gesto esculpe N quadros. Cada um tem a sua `Session` — a máscara é congelada no down
    /// **por desenho** (os traços de um quadro não são os do outro), e o Grab congela os
    /// pesos dele. Sem seleção múltipla a lista tem UM elemento e nada muda.
    targets: Vec<ReshapeTarget>,
    /// O cursor da amostra anterior, em espaço LOCAL (o delta sai daqui).
    last_local: Vec2,
    /// O objeto que o gesto está esculpindo — resolvido UMA vez, no down. (Se o alvo
    /// pudesse mudar no meio, um scrub de playhead partiria a escultura em dois.)
    oid: ph2d_flip::FlipObjectId,
}

/// O vocabulário da tool → o do solver. As duas listas são espelhos (o mesmo
/// precedente do balde: a tool não depende do solver, o shell traduz).
fn kind_of(k: ph2d_tool_flip::ReshapeKind) -> ReshapeKind {
    match k {
        ph2d_tool_flip::ReshapeKind::Smooth => ReshapeKind::Smooth,
        ph2d_tool_flip::ReshapeKind::Push => ReshapeKind::Push,
        ph2d_tool_flip::ReshapeKind::Grab => ReshapeKind::Grab,
        ph2d_tool_flip::ReshapeKind::Pinch => ReshapeKind::Pinch,
        ph2d_tool_flip::ReshapeKind::Twist => ReshapeKind::Twist,
        ph2d_tool_flip::ReshapeKind::Thickness => ReshapeKind::Thickness,
        ph2d_tool_flip::ReshapeKind::Strength => ReshapeKind::Strength,
        ph2d_tool_flip::ReshapeKind::Randomize => ReshapeKind::Randomize,
    }
}

/// Os parâmetros do pincel, na unidade do solver (espaço local do objeto).
///
/// - **raio** = metade do Size (o Size é o DIÂMETRO do pincel, como na borracha);
/// - **força** = o Strength do painel (o `opacity` da tool — a borracha faz igual);
/// - **invert** = Ctrl (só morde nos pincéis com direção; ver `ReshapeKind`);
/// - **`frame_falloff`** — o peso temporal do quadro (W7). Ele sai daqui em `1.0` e é
///   SOBRESCRITO por alvo em `reshape_sample` (cada quadro do multiframe tem o seu). Os
///   oito pincéis já o respeitam: ele entra no funil único `influence()` do solver.
pub(crate) fn params_from(
    style: &FlipStyleSnapshot,
    px_to_world: f32,
    w2l: &Xform,
    invert: bool,
) -> ReshapeParams {
    let obj_scale = w2l.mean_scale() as f32;
    // O `px_to_local` FICA — mas só para o DELTA do arrasto: um gesto de N px na tela tem
    // de mover a arte junto com o cursor, e isso é conversão de tela por definição. O
    // RAIO é outra grandeza (§4.C.6): ele é a porção de ARTE que o pincel abrange.
    let px_to_local = px_to_world * obj_scale;
    ReshapeParams {
        kind: kind_of(style.reshape),
        // **Raio fixo no MUNDO** (§4.C.6, porta única `size_to_world`), recuado à escala
        // do objeto: o pincel de escultura esculpe sempre a mesma porção de ARTE. O Size é
        // o MESMO número do pincel de desenho, então ele não pode ser mundo lá e tela cá.
        radius: ph2d_tool_flip::size_to_world(style.width_px) * 0.5 * obj_scale,
        strength: style.opacity,
        invert,
        frame_falloff: 1.0,
        px_to_local,
    }
}

/// **Começa a escultura no documento** — a função livre (o método do `App` só junta
/// câmera, modificadores e estilo e chama esta).
///
/// Resolve o desenho-alvo pelo **autokey por-tool com política `Modify`**, congela a
/// máscara e aplica a 1ª amostra. `None` = a camada está travada, ou não há desenho
/// neste quadro com o AutoKey desligado (o chamador vira isso num toast; uma
/// ferramenta que consome o clique e não faz nada parece quebrada).
pub(crate) fn reshape_begin(
    flip: &mut ph2d_flip::FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<LayerId>,
    strip: &mut crate::flip::strip::FlipStrip,
    p: &ReshapeParams,
    s: &InputSample,
    falloff_on: bool,
) -> Option<(ph2d_flip::FlipObjectId, Vec<ReshapeTarget>)> {
    let (oid, lid, did) = crate::flip::autokey::target_drawing(
        flip,
        playhead,
        active_layer,
        strip,
        crate::flip::autokey::FlipEdit::Modify,
    )?;
    // **O alvo multiframe é resolvido ANTES do gesto** (`02_referencia §11`): daqui para
    // baixo o Sculpt não sabe que multiframe existe — ele só itera `(drawing, falloff)`.
    // O quadro ATIVO entra sempre, com influência cheia; os selecionados entram
    // deduplicados por desenho (um desenho instanciado por duas chaves é UM alvo, senão o
    // pincel o esculpiria em dobro).
    let frame = flip.object(oid).map_or(0, |o| o.frame_at(playhead));
    let mf = ph2d_app_flip::multiframe::targets(
        flip,
        oid,
        lid,
        playhead,
        strip.selected_keys(),
        (did, frame),
        falloff_on,
    );
    let mut targets = Vec::with_capacity(mf.len());
    for t in mf {
        let Some(drawing) = flip.object_mut(oid).and_then(|o| o.drawing_mut(t.did)) else {
            continue;
        };
        // A máscara é congelada POR DESENHO: os traços de um quadro não são os do outro
        // (e o auto-masking pela seleção — W6 — vale em cada um por si).
        let mut session = Session::begin(&drawing.strokes, p, s);
        let mut pf = *p;
        pf.frame_falloff = t.falloff;
        session.apply(&mut drawing.strokes, &pf, s);
        targets.push(ReshapeTarget {
            did: t.did,
            session,
            falloff: t.falloff,
        });
    }
    (!targets.is_empty()).then_some((oid, targets))
}

/// Uma amostra do gesto nos desenhos que ele já resolveu (nunca re-resolve o alvo: um
/// scrub de playhead no meio do gesto partiria a escultura em dois desenhos).
pub(crate) fn reshape_sample(
    flip: &mut ph2d_flip::FlipDoc,
    oid: ph2d_flip::FlipObjectId,
    targets: &mut [ReshapeTarget],
    p: &ReshapeParams,
    s: &InputSample,
) -> bool {
    let mut changed = false;
    for t in targets.iter_mut() {
        let Some(drawing) = flip.object_mut(oid).and_then(|o| o.drawing_mut(t.did)) else {
            continue;
        };
        // O falloff temporal DESTE quadro entra no funil único do solver (`influence()`),
        // e por isso os oito pincéis o respeitam sem saber que multiframe existe.
        let mut pf = *p;
        pf.frame_falloff = t.falloff;
        changed |= t.session.apply(&mut drawing.strokes, &pf, s);
    }
    changed
}

#[cfg(test)]
#[path = "reshape_tests.rs"]
mod tests;

use crate::flip::ctx::FlipFrame;
use crate::flip::state::FlipState;

/// A tool Flip quer o canvas para ESCULPIR agora? (ativa + modo Reshape.)
#[must_use]
pub(crate) fn wants(state: &FlipState) -> bool {
    state.active
        && matches!(
            state.style.map(|s| s.mode),
            Some(ph2d_tool_flip::FlipMode::Reshape)
        )
}

/// O cursor (tela) → o espaço local do objeto Flip ativo + a escala da câmera.
///
/// ⚠️ O `w2l` chega PRONTO — derivá-lo é do `transform`, e quem chama já o tem.
fn local_at(f: &FlipFrame<'_>, w2l: &Xform, x: f32, y: f32) -> (Vec2, f32) {
    let world = f.to_world(x, y);
    let local = w2l.apply([f64::from(world[0]), f64::from(world[1])]);
    (
        Vec2::new(local[0] as f32, local[1] as f32),
        f.px_to_world(),
    )
}

/// Pen-down: resolve o desenho-alvo, congela a máscara e aplica a 1ª amostra.
///
/// ⚠️ Devolve `(consumido, avisou)` — o `title_dirty` é da shell.
pub(crate) fn canvas_down(
    state: &mut FlipState,
    f: &mut FlipFrame<'_>,
    toasts: &mut ph2d_editor::ToastQueue,
    w2l: &Xform,
    invert: bool,
    x: f32,
    y: f32,
) -> (bool, bool) {
    if !wants(state) {
        return (false, false);
    }
    let Some(style) = state.style else {
        return (false, false);
    };
    let (local, px_to_world) = local_at(f, w2l, x, y);
    let p = params_from(&style, px_to_world, w2l, invert);

    let active_layer = state.active_layer;
    let falloff_on = state.strip.falloff;
    let s = InputSample {
        pos: local,
        delta: Vec2::ZERO, // no pen-down o cursor ainda não andou
        pressure: 1.0,     // sem caneta real: pressão cheia (igual ao desenho)
    };
    let Some((oid, targets)) = reshape_begin(
        f.flip,
        f.playhead,
        active_layer,
        &mut state.strip,
        &p,
        &s,
        falloff_on,
    ) else {
        // Camada travada, ou sem chave com o AutoKey desligado. Uma ferramenta que consome o
        // clique e não faz NADA parece quebrada — ela tem de DIZER.
        toasts.push(ph2d_editor::Toast::warning(
            "Sculpt: the layer is locked, or has no drawing on this frame",
        ));
        return (true, true);
    };
    state.reshape = Some(FlipReshape {
        targets,
        last_local: local,
        oid,
    });
    (true, false)
}

/// Move: uma amostra por movimento (a **dose é por amostra** — mover devagar aplica mais, que
/// é como o pincel do GP se comporta). `true` enquanto o gesto está vivo.
pub(crate) fn canvas_move(
    state: &mut FlipState,
    f: &mut FlipFrame<'_>,
    w2l: &Xform,
    invert: bool,
    x: f32,
    y: f32,
) -> bool {
    if state.reshape.is_none() {
        return false;
    }
    let Some(style) = state.style else {
        return true;
    };
    let (local, px_to_world) = local_at(f, w2l, x, y);
    let p = params_from(&style, px_to_world, w2l, invert);

    let Some(g) = state.reshape.as_mut() else {
        return true;
    };
    let s = InputSample {
        pos: local,
        delta: local - g.last_local,
        pressure: 1.0,
    };
    g.last_local = local;
    let oid = g.oid;
    reshape_sample(f.flip, oid, &mut g.targets, &p, &s);
    true
}

/// Pen-up: encerra o gesto (a máscara congelada morre com ele). O passo de undo sai de graça
/// — o `post_frame_undo` registra o diff quando o botão é solto.
pub(crate) fn canvas_up(state: &mut FlipState) -> bool {
    state.reshape.take().is_some()
}
