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

use crate::flip_strip_resolve::{ensure_cycle_span, seek, source_frame, target};
// O intervalo do tween é um resolvedor (mudou-se para o irmão junto com os outros três),
// mas é o único com consumidores FORA daqui — re-exportado para os caller paths ficarem
// intactos, o mesmo que o `inspector_model_physics` fez quando a física se dividiu.
pub(crate) use crate::flip_strip_resolve::current_tween_interval;
use ph2d_core::Playhead;
use ph2d_flip::{
    CycleMode, DupMode, Easing, EasingFamily, EasingMode, FlipDoc, Frame, Hold, Interp, KeyKind,
    LayerId, TweenOptions, TweenRequest,
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
    /// O preset de easing dos inbetweens (índice em `TWEEN_EASE_NAMES` do painel).
    /// O MOTOR sempre soube fazer isto (`TweenOptions::easing`); era a barra que não
    /// oferecia — a dívida que o plano T3.7 declarou como *"carry-over de UI, não de motor"*.
    pub(crate) tween_ease: u8,
    /// Traços que existem em só UMA das duas chaves entram/saem esmaecendo, em vez de
    /// serem cópia estática (`TweenOptions::fade_orphans`).
    pub(crate) tween_fade: bool,
    /// Chaves selecionadas na tira. O modo `Selected` dos Ghost Frames lê isto — e o
    /// **multiframe** (W7) a usa como alvo: com 2+ chaves marcadas, o MESMO gesto de
    /// escultura/balde age em todas.
    pub(crate) selection: Vec<Frame>,
    /// **Light table** (T3.9): chaves fixadas como REFERÊNCIA — elas aparecem como
    /// fantasma além dos vizinhos, em qualquer modo e fora do alcance.
    ///
    /// ⚠️ Estado de SESSÃO, como a `selection` ao lado: pins não viajam no arquivo. Levá-los
    /// ao documento custaria um campo apendado numa struct serializada, e o `FlipDoc` viaja
    /// DENTRO do `ProjectState` sem versão própria — ou seja, um bump de `PROJECT_SCHEMA`
    /// (que RECUSA todo projeto já salvo) numa janela em que outras linhas também bumpam.
    /// Persistir é decisão de produto, nomeada no handoff.
    pub(crate) pinned: Vec<Frame>,
    /// **Shift & Trace** (`docs/Flip/04 §4`): o DESLOCAMENTO de exibição de cada
    /// fantasma, por chave — o papel que desliza no lightbox. Afim no espaço do OBJETO,
    /// composto por cima da pose da chave só no PASSE de fantasmas; o documento nunca o
    /// vê. Estado de SESSÃO como os vizinhos acima (mesma razão, mesmo remap — a porta
    /// `remap_session_*` o carrega quando a chave anda).
    pub(crate) trace: std::collections::BTreeMap<Frame, ph2d_flip::Pose>,
    /// **Falloff temporal** do multiframe (W7): com ele LIGADO, os quadros vizinhos
    /// recebem menos influência que o ativo. Desligado por padrão — o uso comum é
    /// *"aplique esta edição em todos os quadros que marquei"*, e o Blender também o expõe
    /// como um interruptor à parte. **Só pincéis o respeitam**; ops discretas (o balde) usam
    /// sempre `1.0` (`02_referencia §11`).
    pub(crate) falloff: bool,
    /// **A sessão de correção de pares** (a UI que o toggle "Pairs" abre). `Some` = ativa:
    /// o overlay mostra a correspondência do intervalo, o clique do canvas re-pareia, e o
    /// Add commita com o plano corrigido. Estado de autoria (não documento) — corrigir um
    /// par não muda o desenho até o Add. Ver [`crate::flip_tween_correct`].
    pub(crate) tween_correct: Option<crate::flip_tween_correct::TweenCorrect>,
}

/// **Os quatro presets da barra**, na ordem dos rótulos do painel
/// (`Linear · Ease In · Ease Out · Ease In-Out`).
///
/// A FAMÍLIA é fixa em `Quad` de propósito: a barra da tira é um controle rápido, e o
/// picker de família inteiro já existe no menu de curvas da timeline. Oferecer onze
/// famílias num chip de toolbar seria a UI cara no lugar errado.
fn ease_preset(preset: u8) -> Interp {
    let mode = match preset {
        1 => EasingMode::In,
        2 => EasingMode::Out,
        3 => EasingMode::InOut,
        _ => return Interp::Linear,
    };
    Interp::Eased(Easing {
        family: EasingFamily::Quad,
        mode,
    })
}

impl Default for FlipStrip {
    fn default() -> Self {
        Self {
            autokey: true,
            additive: false,
            tween_count: 1,
            tween_ease: 0,
            tween_fade: false,
            selection: Vec::new(),
            pinned: Vec::new(),
            trace: std::collections::BTreeMap::new(),
            falloff: false,
            tween_correct: None,
        }
    }
}

impl FlipStrip {
    /// **As opções do tween montadas a partir da barra** — a porta única entre os dois
    /// controles e o motor. O `Add` lê daqui em vez de montar um `TweenOptions` próprio;
    /// dois lugares montando as mesmas opções é como um deles esquece do knob novo.
    pub(crate) fn tween_options(&self) -> TweenOptions {
        TweenOptions {
            easing: ease_preset(self.tween_ease),
            fade_orphans: self.tween_fade,
            ..TweenOptions::default()
        }
    }

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
        // **Pin** (light table, T3.9) — fixa/desafixa a chave atual como REFERÊNCIA: ela
        // vira fantasma em qualquer modo e fora do alcance. Não é edição de documento
        // (nenhum pixel muda), então NÃO devolve `true`: é estado de sessão, como a
        // seleção — e um passo de undo por "eu quis ver aquele quadro" seria ruído na fila.
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_PIN => {
            if let Some(k) = key {
                strip.toggle_pin(k);
            }
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_KEY_UNLINK => {
            let Some(k) = key else { return false };
            let ok = flip
                .object_mut(oid)
                .is_some_and(|o| o.make_single_user(lid, k));
            if ok {
                // **Desvincular ENCERRA a multisseleção** (só esta chave fica marcada). O
                // sentido do Unlink é "este quadro agora é independente"; se a tira
                // seguisse com 2+ chaves marcadas, o próximo Sculpt seria MULTIFRAME e
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
        PanelEvent::SelectOption(id, val) if *id == ids::FLIP_TWEEN_EASE_DD => {
            strip.tween_ease = val.parse::<u8>().unwrap_or(0).min(3);
            false
        }
        PanelEvent::Click(id) if *id == ids::FLIP_TWEEN_FADE => {
            strip.tween_fade = !strip.tween_fade;
            false
        }
        // **O toggle "Pairs"** (a UI de correção de pares): abre/fecha a sessão de
        // correspondência para o intervalo atual. Aberta, ela intercepta o clique do canvas
        // (re-parear) e o Add commita com o plano corrigido. Sem intervalo válido, não abre —
        // não há entre o quê interpolar, então não há par a corrigir.
        PanelEvent::Click(id) if *id == ids::FLIP_TWEEN_PAIRS => {
            if strip.tween_correct.is_some() {
                strip.tween_correct = None; // fecha
            } else {
                strip.tween_correct =
                    crate::flip_tween_correct::build(flip, active_layer, playhead);
            }
            false // estado de autoria, não documento
        }
        PanelEvent::Click(id) if *id == ids::FLIP_TWEEN_ADD => {
            // Os extremos do tween são **KEYFRAMES** — nunca os breakdowns que ele
            // mesmo gerou. Usar o "próximo desenho" fazia o 2º Add interpolar entre a
            // chave e o inbetween vizinho (lixo entre 0 e 2) em vez de REGENERAR o
            // intervalo 0→8. E parado em cima de um inbetween, o extremo A é a chave
            // anterior — clicar Add de novo regenera, não empilha.
            let Some((_, _, from, to)) = current_tween_interval(flip, active_layer, playhead)
            else {
                return false; // sem os dois extremos não há entre o quê interpolar
            };
            let req = TweenRequest {
                layer: lid,
                from,
                to,
                count: strip.tween_count,
                options: strip.tween_options(),
            };
            // Se a sessão de correção de pares está pinada NESTE intervalo, commita com o
            // plano corrigido; senão o automático de sempre. A porta única de intervalo
            // garante que a comparação bate — um par que o artista corrigiu tem de aparecer
            // no desenho, e a guarda de dimensões do motor recusa um plano obsoleto.
            let corrected = strip
                .tween_correct
                .as_ref()
                .filter(|tc| (tc.layer, tc.from, tc.to) == (lid, from, to))
                .map(|tc| &tc.plan);
            let made = flip.object_mut(oid).map_or(0, |o| match corrected {
                Some(plan) => o.tween_with_plan(req, plan),
                None => o.tween(req),
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

        // ── Shift & Trace: o Reset devolve toda folha deslocada ao lugar ───────
        //
        // Mora AQUI (e não no render_loop) porque esta função já possui o `strip` — e é
        // testável sem janela. Exibição, não documento: devolve `false` (sem undo).
        PanelEvent::Click(id) if *id == ids::FLIP_TRACE_RESET => {
            strip.trace.clear();
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
#[path = "flip_strip_pin_tests.rs"]
mod pin_tests;
#[cfg(test)]
#[path = "flip_strip_tests.rs"]
mod tests;
