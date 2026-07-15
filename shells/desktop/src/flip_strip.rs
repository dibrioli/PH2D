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
    /// Chaves selecionadas na tira. O modo `Selected` dos Ghost Frames lê isto — e o
    /// **multiframe** (W7) a usa como alvo: com 2+ chaves marcadas, o MESMO gesto de
    /// escultura/balde age em todas.
    pub(crate) selection: Vec<Frame>,
    /// **Falloff temporal** do multiframe (W7): com ele LIGADO, os quadros vizinhos
    /// recebem menos influência que o ativo. Desligado por padrão — o uso comum é
    /// *"aplique esta edição em todos os quadros que marquei"*, e o Blender também o expõe
    /// como um interruptor à parte. **Só pincéis o respeitam**; ops discretas (o balde) usam
    /// sempre `1.0` (`02_referencia §11`).
    pub(crate) falloff: bool,
}

impl Default for FlipStrip {
    fn default() -> Self {
        Self {
            autokey: true,
            additive: false,
            tween_count: 1,
            selection: Vec::new(),
            falloff: false,
        }
    }
}

impl FlipStrip {
    /// As chaves selecionadas (os fantasmas leem isto no modo `Selected`; o **multiframe**
    /// as usa como alvo — `flip_multiframe::targets`).
    pub(crate) fn selected_keys(&self) -> &[Frame] {
        &self.selection
    }
}

/// Alterna a chave `k` na seleção (Shift/Ctrl+clique na célula).
fn toggle_key(sel: &mut Vec<Frame>, k: Frame) {
    if let Some(i) = sel.iter().position(|x| *x == k) {
        sel.remove(i);
    } else {
        sel.push(k);
        sel.sort_unstable();
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

/// O **quadro-fonte** da camada agora: sob um ciclo, o quadro do vão que está sendo
/// exibido. Toda op de chave age NELE (a célula que se vê é a que se edita).
fn source_frame(flip: &FlipDoc, oid: FlipObjectId, lid: LayerId, playhead: &Playhead) -> Frame {
    let Some(obj) = flip.object(oid) else {
        return 0;
    };
    let raw = obj.frame_at(playhead);
    obj.layer(lid).map_or(raw, |l| l.source_frame(raw))
}

/// **Ligar um ciclo dá uma exposição REAL à última célula.**
///
/// O hold implícito da última chave é infinito — sem sentinela, o vão fecha em
/// `última + 1` e ela expõe UM quadro. Num Loop isso é um piscar: as outras células
/// seguram 8 quadros e a última passa num frame. Então, ao ligar Loop/Ping-Pong,
/// materializamos a exposição da última chave **igual à da anterior** (o ritmo que o
/// animador já estabeleceu; `1` se não há anterior).
///
/// Não é mágica escondida: a exposição vira uma sentinela VISÍVEL (a célula alarga na
/// tira) e editável (a caixa **Hold**). Idempotente — se a última chave já tem
/// exposição fixa, nada muda.
fn ensure_cycle_span(flip: &mut FlipDoc, oid: FlipObjectId, lid: LayerId) -> bool {
    let Some(layer) = flip.object(oid).and_then(|o| o.layer(lid)) else {
        return false;
    };
    let cells = layer.cells();
    let Some(&(last, _, _)) = cells.last() else {
        return false;
    };
    if layer.duration_at(last) != 0 {
        return false; // já tem sentinela: a exposição é explícita, respeita
    }
    let prev = cells
        .len()
        .checked_sub(2)
        .and_then(|i| cells.get(i))
        .map_or(1, |&(_, _, e)| e);
    flip.object_mut(oid)
        .is_some_and(|o| o.set_exposure(lid, last, prev))
}

/// Aplica um evento do painel da tira. Devolve `true` se MUDOU O DOCUMENTO (o
/// caller marca a edição; transporte e seleção não são passos de undo).
pub(crate) fn apply_panel_event(
    ev: &ph2d_editor::tool::PanelEvent,
    flip: &mut FlipDoc,
    active_layer: Option<LayerId>,
    playhead: &mut Playhead,
    strip: &mut FlipStrip,
    add: bool,
) -> bool {
    use ph2d_editor::ids;
    use ph2d_editor::tool::PanelEvent;

    let Some((oid, lid)) = target(flip, active_layer) else {
        return false;
    };
    let fps = flip.object(oid).map_or(24.0, |o| o.fps);
    // O quadro-FONTE (o do ciclo): sob um Loop, a célula que se vê na 2ª volta é a do
    // vão, e é NELA que as ops de chave agem (duplicar/apagar/expor o que está na tela).
    let frame = source_frame(flip, oid, lid, playhead);
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
        // **A régua de scrub** (W7.3): o painel já resolveu o QUADRO (o inverso do handle,
        // `scrub_frame`); aqui só se leva o playhead até lá. **NÃO tocamos a seleção** — é o
        // ponto INTEIRO da régua: mover o playhead entre os quadros marcados (p/ ver o
        // falloff, re-ancorar) sem desmontar o multiframe. Clicar numa CÉLULA é que
        // seleciona; a régua só scrubba (smoke do Enio, 2026-07-14). Transporte, não edição.
        PanelEvent::SetValue(id, v) if *id == ids::FLIP_SCRUB => {
            seek(playhead, fps, *v as Frame);
            false
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
        PanelEvent::Click(id) if *id == ids::FLIP_FALLOFF => {
            strip.falloff = !strip.falloff;
            false // política de autoria, não documento
        }
        PanelEvent::Click(id) if *id == ids::FLIP_ADDITIVE => {
            strip.additive = !strip.additive;
            false
        }

        // ── Ops de chave ──────────────────────────────────────────────────────
        PanelEvent::Click(id)
            if *id == ids::FLIP_KEY_ADD
                || *id == ids::FLIP_KEY_DUP
                || *id == ids::FLIP_KEY_INSTANCE =>
        {
            // O que a chave nova carrega:
            //   `None`            → BRANCA (Key Add).
            //   `Deep`            → uma CÓPIA da arte (Key Dup): desenho novo, independente.
            //   `Instance`        → o MESMO desenho (`users += 1`, o *linked duplicate*):
            //                       editar uma chave edita as DUAS. É como um ciclo reusa
            //                       arte — e é o que acende o pontinho na célula e faz o
            //                       multiframe deduplicar (`flip_multiframe::targets`).
            let mode = match id {
                i if *i == ids::FLIP_KEY_DUP => Some(DupMode::Deep),
                i if *i == ids::FLIP_KEY_INSTANCE => Some(DupMode::Instance),
                _ => None,
            };
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
            // **Se `at` cai numa CHAVE REAL, abre espaço** (ripple do bloco contíguo). Sem
            // isto, criar quadro só funcionava na ÚLTIMA chave — em qualquer outra o alvo
            // `chave + duração` colidia com a próxima e o insert falhava mudo (smoke do
            // Enio, 2026-07-14). Uma sentinela em `at` NÃO precisa de fenda: o insert a
            // sobrescreve.
            let on_real_key = flip
                .object(oid)
                .and_then(|o| o.layer(lid))
                .and_then(|l| l.frames().get(&at))
                .is_some_and(|f| f.drawing.is_some());
            if on_real_key {
                flip.object_mut(oid).map(|o| o.open_gap_at(lid, at));
            }
            let ok = match (mode, key) {
                // Sem chave de origem não há o que duplicar: cai na chave branca.
                (Some(m), Some(src)) => flip
                    .object_mut(oid)
                    .is_some_and(|o| o.duplicate_frame(lid, src, at, m)),
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
        // **Unlink** — a chave larga a arte compartilhada e ganha uma cópia SÓ dela
        // (`make_single_user`). É a volta da instância: sem ela, compartilhar arte seria
        // irreversível (só apagando a chave e redesenhando). No-op honesto quando o
        // desenho já é exclusivo — não há vínculo a quebrar.
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_UNLINK => {
            let Some(k) = key else { return false };
            let ok = flip
                .object_mut(oid)
                .is_some_and(|o| o.make_single_user(lid, k));
            if ok {
                // **Desvincular ENCERRA a multisseleção** (só esta chave fica marcada). O
                // sentido do Unlink é "este quadro agora é independente"; se a tira
                // continuasse com 2+ chaves marcadas, o próximo Sculpt seria MULTIFRAME e
                // editaria os dois de novo — e como a cópia nasce idêntica à origem, o
                // resultado idêntico PARECE que o vínculo voltou (smoke do Enio,
                // 2026-07-14). Uma chave só ⇒ multiframe desligado (`selection.len() < 2`).
                strip.selection = vec![k];
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
            // Os extremos do tween são **KEYFRAMES** — nunca os breakdowns que ele
            // mesmo gerou. Usar o "próximo desenho" fazia o 2º Add interpolar entre a
            // chave e o inbetween vizinho (lixo entre 0 e 2) em vez de REGENERAR o
            // intervalo 0→8. E parado em cima de um inbetween, o extremo A é a chave
            // anterior — clicar Add de novo regenera, não empilha.
            let Some(layer) = flip.object(oid).and_then(|o| o.layer(lid)) else {
                return false;
            };
            let (Some(from), Some(to)) = (
                layer.keyframe_at_or_before(frame),
                layer.next_keyframe_key(frame),
            ) else {
                return false; // sem os dois extremos não há entre o quê interpolar
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
            // Loop/Ping-Pong repetem o VÃO — e o vão só tem fim quando a última chave
            // tem exposição real. Sem isso ela expõe 1 quadro e o ciclo "pisca" no fim.
            if matches!(post, CycleMode::Loop | CycleMode::PingPong) {
                ensure_cycle_span(flip, oid, lid);
            }
            if let Some(l) = flip.object_mut(oid).and_then(|o| o.layer_mut(lid)) {
                l.cycle.pre = pre;
                l.cycle.post = post;
                return true;
            }
            false
        }

        // ── Células: seleciona + leva o playhead até a chave ───────────────────
        //
        // **Com modificador (Shift/Ctrl), ALTERNA a chave na seleção e NÃO move o
        // playhead** (W7 — multiframe). Mover o playhead junto seria destrutivo: o quadro
        // ATIVO é a âncora do falloff temporal e é o único que recebe influência cheia —
        // montar a seleção arrastaria a âncora a cada clique, e o gesto sairia pesando os
        // quadros errados.
        //
        // O modificador vem do SHELL (o `add`), não do evento: o `WidgetEvent::Click` não
        // carrega modificadores e o `PanelEvent` está congelado (ADR-0040). Nenhum contrato
        // é tocado.
        PanelEvent::Click(id) => {
            let cells = flip
                .object(oid)
                .and_then(|o| o.layer(lid))
                .map(|l| l.cells())
                .unwrap_or_default();
            for (i, (k, _, _)) in cells.iter().enumerate() {
                if ph2d_editor::ids::flip_cell_id(i) == *id {
                    if add {
                        toggle_key(&mut strip.selection, *k);
                    } else {
                        seek(playhead, fps, *k);
                        strip.selection = vec![*k];
                    }
                    return false; // navegar/selecionar não é edição
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
        let fps = obj.fps;
        let Some(layer) = obj.layer(lid) else { return };
        // No quadro-FONTE: sob um Loop, navegar a partir da 2ª volta tem de andar
        // dentro do vão (no quadro cru não haveria vizinho nenhum).
        let frame = layer.source_frame(obj.frame_at(&self.playhead));
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

#[cfg(test)]
#[path = "flip_strip_tests.rs"]
mod tests;
