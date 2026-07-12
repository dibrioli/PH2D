//! ADR-0114 W3 — a **tira de frames** do lado do shell: o estado de autoria + o
//! drain que aplica os eventos do painel no `FlipDoc` e no playhead.
//!
//! O painel (`ph2d-panel-flip-frames`) é stateless: ele pinta um snapshot e emite
//! `ToolPanelEvent`s. Aqui é onde as ações VIRAM documento — e onde mora o punhado
//! de estado que não é do documento (os toggles de autoria e quantos inbetweens o
//! botão de tween vai gerar).
//!
//! Um objeto Flip = uma tira (a camada ATIVA). O playhead é o **global** (o mesmo
//! que a timeline e o resto do engine leem): quando a integração com a timeline
//! chegar (W6), não há relógio para reconciliar — já é o mesmo.

use ph2d_core::Playhead;
use ph2d_flip::{
    CycleMode, DupMode, FlipDoc, FlipObjectId, Frame, Hold, KeyKind, LayerId, TweenOptions,
    TweenRequest,
};

/// O estado de autoria da tira (o que NÃO é documento).
pub(crate) struct FlipStrip {
    /// Desenhar/apagar depois do hold cria uma chave nova (ligado por padrão — é o
    /// que faz o Flip ser um app de animação).
    pub(crate) autokey: bool,
    /// A chave que o DESENHO cria nasce como cópia da anterior (em vez de branca).
    /// A borracha ignora isto: ela SEMPRE duplica ([`ph2d_flip::AutokeyPolicy`]).
    pub(crate) additive: bool,
    /// Quantos inbetweens o botão Tween gera.
    pub(crate) tween_count: u32,
    /// Chaves selecionadas na tira (hoje: a última clicada). O modo `Selected` dos
    /// Ghost Frames lê isto.
    pub(crate) selection: Vec<Frame>,
}

impl Default for FlipStrip {
    fn default() -> Self {
        Self {
            autokey: true,
            additive: false,
            tween_count: 1,
            selection: Vec::new(),
        }
    }
}

impl FlipStrip {
    /// As chaves selecionadas (o render dos fantasmas lê isto no modo `Selected`).
    pub(crate) fn selected_keys(&self) -> &[Frame] {
        &self.selection
    }
}

/// O objeto e a camada que a tira edita: o 1º objeto (igual ao `bake_stroke`) e a
/// camada ativa (fallback: o topo).
fn target(flip: &FlipDoc, active_layer: Option<LayerId>) -> Option<(FlipObjectId, LayerId)> {
    let obj = flip.objects().first()?;
    let lid = active_layer
        .filter(|id| obj.layer(*id).is_some())
        .or_else(|| obj.layers().last().map(|l| l.id))?;
    Some((obj.id, lid))
}

/// Leva o playhead ao quadro `f` (no FPS do objeto) e PAUSA — quem clica numa
/// célula quer ver aquele desenho, não continuar tocando a partir dele.
fn seek(playhead: &mut Playhead, fps: f32, f: Frame) {
    playhead.pause();
    playhead.seek_frame(i64::from(f.max(0)), f64::from(fps));
}

/// Aplica um evento do painel da tira. Devolve `true` se MUDOU O DOCUMENTO (o
/// caller marca a edição; transporte e seleção não são passos de undo).
pub(crate) fn apply_panel_event(
    ev: &ph2d_editor::tool::PanelEvent,
    flip: &mut FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &mut Playhead,
    strip: &mut FlipStrip,
) -> bool {
    use ph2d_editor::ids;
    use ph2d_editor::tool::PanelEvent;

    let Some((oid, lid)) = target(flip, active_layer) else {
        return false;
    };
    let fps = flip.object(oid).map_or(24.0, |o| o.fps);
    let frame = flip.object(oid).map_or(0, |o| o.frame_at(playhead));
    // A chave ATIVA agora — a origem de quase toda op (duplicar, apagar, expor).
    let key = flip
        .object(oid)
        .and_then(|o| o.layer(lid))
        .and_then(|l| l.active_key(frame))
        .filter(|k| {
            flip.object(oid)
                .and_then(|o| o.layer(lid))
                .and_then(|l| l.frames().get(k))
                .is_some_and(|f| f.drawing.is_some())
        });

    match ev {
        // ── Transporte ────────────────────────────────────────────────────────
        PanelEvent::Click(id) if *id == ids::FLIP_PLAY => {
            playhead.toggle_play();
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_PREV_DRAWING || *id == ids::FLIP_NEXT_DRAWING => {
            let next = *id == ids::FLIP_NEXT_DRAWING;
            let Some(layer) = flip.object(oid).and_then(|o| o.layer(lid)) else {
                return false;
            };
            let target = if next {
                layer.next_drawing_key(frame)
            } else {
                layer.prev_drawing_key(frame)
            };
            if let Some(f) = target {
                seek(playhead, fps, f);
                strip.selection = vec![f];
            }
            false
        }
        PanelEvent::SetValue(id, v) if *id == ids::FLIP_FPS_NUM => {
            if let Some(o) = flip.object_mut(oid) {
                o.fps = (*v as f32).clamp(1.0, 120.0);
            }
            true
        }

        // ── Ghost Frames ──────────────────────────────────────────────────────
        PanelEvent::Click(id) if *id == ids::FLIP_GHOST => {
            if let Some(o) = flip.object_mut(oid) {
                o.onion.enabled = !o.onion.enabled;
            }
            true
        }
        PanelEvent::SetValue(id, v)
            if *id == ids::FLIP_GHOST_BEFORE_NUM || *id == ids::FLIP_GHOST_AFTER_NUM =>
        {
            let n = (*v as i64).clamp(0, 8) as u32;
            if let Some(o) = flip.object_mut(oid) {
                if *id == ids::FLIP_GHOST_BEFORE_NUM {
                    o.onion.frames_before = n;
                } else {
                    o.onion.frames_after = n;
                }
            }
            true
        }

        // ── Autoria (flags do shell, não do documento) ────────────────────────
        PanelEvent::Click(id) if *id == ids::FLIP_AUTOKEY => {
            strip.autokey = !strip.autokey;
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_ADDITIVE => {
            strip.additive = !strip.additive;
            false
        }

        // ── Ops de chave ──────────────────────────────────────────────────────
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_ADD || *id == ids::FLIP_KEY_DUP => {
            let dup = *id == ids::FLIP_KEY_DUP;
            // A chave nova entra DEPOIS da exposição da atual (o próximo quadro
            // livre) — que é onde o animador espera o próximo desenho.
            let at = match key {
                Some(k) => {
                    let dur = flip
                        .object(oid)
                        .and_then(|o| o.layer(lid))
                        .map_or(1, |l| l.duration_at(k).max(1));
                    k.saturating_add(dur as i32)
                }
                None => frame,
            };
            let ok = match (dup, key) {
                (true, Some(src)) => flip
                    .object_mut(oid)
                    .is_some_and(|o| o.duplicate_frame(lid, src, at, DupMode::Deep)),
                _ => flip
                    .object_mut(oid)
                    .and_then(|o| o.insert_frame(lid, at, Hold::Implicit, KeyKind::Keyframe))
                    .is_some(),
            };
            if ok {
                seek(playhead, fps, at);
                strip.selection = vec![at];
            }
            ok
        }
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_DELETE => {
            let Some(k) = key else { return false };
            let Some(o) = flip.object_mut(oid) else {
                return false;
            };
            if !o.remove_frame(lid, k) {
                return false;
            }
            o.remove_unused_drawings();
            strip.selection.clear();
            true
        }
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_LEFT || *id == ids::FLIP_KEY_RIGHT => {
            let Some(k) = key else { return false };
            let to = if *id == ids::FLIP_KEY_LEFT {
                k - 1
            } else {
                k + 1
            };
            if to < 0 {
                return false;
            }
            let moved = flip
                .object_mut(oid)
                .is_some_and(|o| o.move_frame(lid, k, to));
            if moved {
                seek(playhead, fps, to);
                strip.selection = vec![to];
            }
            moved
        }
        PanelEvent::SetValue(id, v) if *id == ids::FLIP_HOLD_NUM => {
            let Some(k) = key else { return false };
            let n = (*v as i64).clamp(1, 999) as u32;
            // A mecânica (empurrar as seguintes / mover a sentinela) é do MODELO —
            // é lá que ela é testada.
            flip.object_mut(oid)
                .is_some_and(|o| o.set_exposure(lid, k, n))
        }

        // ── Tween ─────────────────────────────────────────────────────────────
        PanelEvent::SetValue(id, v) if *id == ids::FLIP_TWEEN_NUM => {
            strip.tween_count = (*v as i64).clamp(1, 32) as u32;
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_TWEEN_ADD => {
            let Some(from) = key else { return false };
            let Some(to) = flip
                .object(oid)
                .and_then(|o| o.layer(lid))
                .and_then(|l| l.next_drawing_key(from))
            else {
                return false; // sem a chave seguinte não há entre o quê interpolar
            };
            let made = flip.object_mut(oid).map_or(0, |o| {
                o.tween(TweenRequest {
                    layer: lid,
                    from,
                    to,
                    count: strip.tween_count,
                    options: TweenOptions::default(),
                })
            });
            made > 0
        }

        // ── Ciclo (pre/post behavior da camada) ───────────────────────────────
        PanelEvent::SelectOption(id, val) if *id == ids::FLIP_CYCLE_DD => {
            let Ok(mode) = val.parse::<u8>() else {
                return false;
            };
            let (pre, post) = cycle_pair(mode);
            if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(lid)) {
                l.cycle.pre = pre;
                l.cycle.post = post;
                return true;
            }
            false
        }

        // ── Células: seleciona + leva o playhead até a chave ───────────────────
        PanelEvent::Click(id) => {
            let cells = flip
                .object(oid)
                .and_then(|o| o.layer(lid))
                .map(|l| l.cells())
                .unwrap_or_default();
            for (i, (k, _, _)) in cells.iter().enumerate() {
                if ph2d_editor::ids::flip_cell_id(i) == *id {
                    seek(playhead, fps, *k);
                    strip.selection = vec![*k];
                    return false; // navegar não é edição
                }
            }
            false
        }
        _ => false,
    }
}

impl crate::App {
    /// O FPS do objeto Flip ativo (o relógio em que "um quadro" faz sentido para o
    /// animador). `None` sem objeto.
    pub(crate) fn flip_fps(&self) -> Option<f64> {
        let gfx = self.gfx.as_ref()?;
        gfx.flip.objects().first().map(|o| f64::from(o.fps))
    }

    /// **O flip por DESENHO** (atalho das setas ↑/↓ e dos botões da tira): leva o
    /// playhead à chave anterior/seguinte da camada ativa, PULANDO os holds.
    pub(crate) fn flip_step_drawing(&mut self, next: bool) {
        let active_layer = self.flip_active_layer;
        let Some(gfx) = self.gfx.as_ref() else { return };
        let Some((oid, lid)) = target(&gfx.flip, active_layer) else {
            return;
        };
        let Some(obj) = gfx.flip.object(oid) else {
            return;
        };
        let frame = obj.frame_at(&self.playhead);
        let fps = obj.fps;
        let Some(layer) = obj.layer(lid) else { return };
        let to = if next {
            layer.next_drawing_key(frame)
        } else {
            layer.prev_drawing_key(frame)
        };
        if let Some(f) = to {
            seek(&mut self.playhead, fps, f);
            self.flip_strip.selection = vec![f];
        }
    }
}

/// `CycleMode` do chip → o par (pre, post). "Hold" é o default do sistema (nada
/// antes, o último desenho segura depois); Loop/Ping-Pong valem dos DOIS lados
/// (senão o scrub para trás mostraria vazio no meio de um ciclo).
fn cycle_pair(mode: u8) -> (CycleMode, CycleMode) {
    match mode {
        1 => (CycleMode::None, CycleMode::Hold),
        2 => (CycleMode::Loop, CycleMode::Loop),
        3 => (CycleMode::PingPong, CycleMode::PingPong),
        _ => (CycleMode::None, CycleMode::None),
    }
}
