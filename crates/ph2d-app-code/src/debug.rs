//! ⭐ **O depurador na aba** (plano 13 Onda D, decisão 11 do dono): os pontos de parada de cada
//! script (clique na coluna das marcas, F9), o cartão «Stopped» com as variáveis, a pilha e o
//! Continue / Next line, e os valores ao vivo SÓ do script da aba aberta, só no Play. O motor é o
//! `ScriptHost` ([`ph2d_script`]); aqui só se pede e se mostra.
//!
//! Teclas: F9 põe/tira o ponto na linha do cursor; F10 = Next line; F8 com o jogo parado nesta aba =
//! Continue (o F8/F10 do Chrome DevTools; o F5 é o Play da shell). Fora de uma paragem o F8 continua
//! a ser «Next Problem».

use crate::commands::{CMD_CONTINUE, CMD_NEXT_LINE};
use crate::doc::{CodeDoc, map_byte};
use crate::docs::CodeDocs;
use ph2d_a11y::NodeId;
use ph2d_code_buffer::TextEdit;
use ph2d_ecs::{Entity, World};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, paint_text_title, resolve};
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{Toast, ToastQueue};
use ph2d_i18n::{tr, tr_with};
use ph2d_panel_code::{Breakpoint, DebugMarks, Ghost};
use ph2d_script::{DebugStop, LineLive, ProjectScripts, ScriptHost, ScriptId, StopKind};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// O espelho do Pause on Error do `ScriptHost` (o rótulo do menu lê-o; o host é a verdade).
pub(crate) static PAUSE_ON_ERROR: AtomicBool = AtomicBool::new(true);

/// As variáveis que o cartão mostra antes do «+N more» (a altura do cartão; o resto rola na pilha
/// do Luau, não no ecrã).
const VARS_SHOWN: usize = 8;
/// Os caracteres de um valor ao vivo no fim da linha (legibilidade; o resto é «…»).
const GHOST_CHARS: usize = 60;

/// O que a aba pede ao jogo parado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Ask {
    Continue,
    NextLine,
}

/// O cartão de baixo do código: parado (cabeçalho com a fila e os botões, variáveis em grelha,
/// pilha) ou de quem são os valores ao vivo.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DebugCard {
    pub title: String,
    /// «2 more waiting», o chip do cabeçalho.
    pub waiting: Option<String>,
    /// A mensagem do erro (Pause on Error).
    pub error: Option<String>,
    /// `(nome, valor)`; o «+N more» é um nome sem valor.
    pub vars: Vec<(String, String)>,
    /// A pilha já escrita (`update · 14`).
    pub stack: Option<String>,
    /// `false` = só o rótulo dos valores ao vivo.
    pub stopped: bool,
    /// O Next line (escondido num erro).
    pub can_step: bool,
}

/// O depurador de UM documento.
#[derive(Debug, Default)]
pub(crate) struct DocDebug {
    /// O byte do início da linha de cada ponto de parada (segue as edições).
    marks: Vec<usize>,
    /// As linhas (1-based) que o host recebeu; `None` = ainda não falou com ele.
    sent: Option<Vec<u32>>,
    pub(crate) armed: Vec<u32>,
    pub(crate) stopped: Option<usize>,
    pub(crate) ghosts: Vec<Ghost>,
    pub(crate) card: Option<DebugCard>,
    /// Os botões que o último quadro pintou, e o cartão.
    pub(crate) painted: Vec<(Ask, Rect)>,
    pub(crate) card_rect: Option<Rect>,
    pub(crate) ask: Option<Ask>,
    marks_view: Vec<Breakpoint>,
}

/// O depurador da família.
#[derive(Debug, Default)]
pub(crate) struct DebugState {
    pub(crate) ask: Option<Ask>,
    pub(crate) toggle_pause: bool,
    pub(crate) remove_all: bool,
    /// A paragem já mostrada (só uma nova abre a aba e rola).
    last_stop: Option<(Entity, ScriptId, u32, bool)>,
    /// A corrida do Play em que o aviso das paragens bloqueadas já saiu.
    blocked_told: Option<u64>,
}

impl DocDebug {
    pub(crate) fn edited(&mut self, edits: &[TextEdit]) {
        for b in &mut self.marks {
            for e in edits {
                *b = map_byte(*b, e);
            }
        }
    }

    /// A marca para a pintura.
    pub(crate) fn view(&self) -> DebugMarks<'_> {
        DebugMarks {
            breakpoints: &self.marks_view,
            stopped: self.stopped,
            ghosts: &self.ghosts,
        }
    }
}

impl CodeDoc {
    /// As linhas (0-based) com ponto de parada, por ordem e sem repetidas.
    #[must_use]
    pub fn breakpoint_lines(&self) -> Vec<usize> {
        let t = self.buffer.text();
        let mut v: Vec<usize> = self
            .debug
            .marks
            .iter()
            .map(|&b| t.line_of_byte(b.min(t.len())))
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Põe/tira o ponto de parada da linha `line` (0-based).
    pub fn toggle_breakpoint(&mut self, line: usize) {
        let t = self.buffer.text();
        let line = line.min(t.line_count().saturating_sub(1));
        let before = self.debug.marks.len();
        self.debug
            .marks
            .retain(|&b| t.line_of_byte(b.min(t.len())) != line);
        if self.debug.marks.len() == before {
            self.debug.marks.push(t.byte_of_line(line));
        }
        self.refresh_marks();
    }

    /// F9: o ponto na linha do cursor.
    pub fn toggle_breakpoint_at_caret(&mut self) {
        let t = self.buffer.text();
        let line = t.line_of_byte(self.buffer.selection().primary().head);
        self.toggle_breakpoint(line);
    }

    fn set_breakpoint_lines(&mut self, lines: &[u32]) {
        let t = self.buffer.text();
        self.debug.marks = lines
            .iter()
            .filter(|&&n| n >= 1 && (n as usize) <= t.line_count())
            .map(|&n| t.byte_of_line(n as usize - 1))
            .collect();
        self.refresh_marks();
    }

    /// Recalcula o que a calha pinta (oco = o motor não armou nessa linha).
    pub(crate) fn refresh_marks(&mut self) {
        let lines = self.breakpoint_lines();
        let armed = &self.debug.armed;
        let known = self.debug.sent.is_some() && !armed.is_empty();
        self.debug.marks_view = lines
            .into_iter()
            .map(|line| Breakpoint {
                line,
                armed: !known || armed.contains(&(line as u32 + 1)),
            })
            .collect();
    }

    /// Um clique no cartão: o botão pede; o resto do cartão engole o clique. `true` = tomado.
    pub(crate) fn debug_press(&mut self, x: f32, y: f32) -> bool {
        if !self.debug.card_rect.is_some_and(|r| r.contains(x, y)) {
            return false;
        }
        if let Some(&(ask, _)) = self.debug.painted.iter().find(|(_, r)| r.contains(x, y)) {
            self.debug.ask = Some(ask);
        }
        true
    }

    /// ⚠️ SONDA (o condutor das fotos): a coluna das marcas na linha da 1.ª `needle`.
    pub(crate) fn mark_rect_of(&self, needle: &str) -> Option<Rect> {
        let at = self.buffer.text().to_string().find(needle)?;
        let c = self.view.caret_rect(&self.buffer, at)?;
        let m = self.view.metrics();
        let x = self.view.text_rect().x - m.gutter_w;
        Some(Rect::new(x, c.y, m.mark_w, c.h))
    }

    /// F8 com este script parado: Continue.
    pub(crate) fn is_stopped_here(&self) -> bool {
        self.debug.card.as_ref().is_some_and(|c| c.stopped)
    }

    /// A altura do cartão de baixo do código em `w` de largura (0 = nenhum).
    pub(crate) fn debug_card_h(&self, w: f32) -> f32 {
        self.debug.card.as_ref().map_or(0.0, |c| {
            let rows = 1
                + usize::from(c.error.is_some())
                + self.var_grid(c, w).rows
                + usize::from(c.stack.is_some());
            rows as f32 * ph2d_tokens::row_pitch_px() + Spacing::Sm.px() * 2.0
        })
    }

    /// A grelha das variáveis: colunas iguais na largura natural (o nome e o valor mais compridos,
    /// na mono do código), tantas quantas cabem em `w`, com o vão `Spacing::Xl` — compacta, à
    /// esquerda.
    fn var_grid(&self, c: &DebugCard, w: f32) -> VarGrid {
        let adv = self.view.metrics().advance;
        let chars = |f: fn(&(String, String)) -> &String| {
            c.vars
                .iter()
                .map(|v| f(v).chars().count())
                .max()
                .unwrap_or(0) as f32
                * adv
        };
        let inner = (w - Spacing::Sm.px() * 4.0).max(0.0);
        let name_w = chars(|v| &v.0);
        let col_w = (name_w + Spacing::Sm.px() + chars(|v| &v.1)).min(inner);
        let gap = Spacing::Xl.px();
        let cols = (((inner + gap) / (col_w + gap)).floor() as usize).clamp(1, c.vars.len().max(1));
        VarGrid {
            cols,
            rows: c.vars.len().div_ceil(cols),
            name_w,
            col_w,
            gap,
        }
    }

    /// Pinta o cartão em `r` (dentro do seu molde, [`crate::cards::open`]).
    pub(crate) fn paint_debug_card(
        &mut self,
        r: Rect,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: &Theme,
    ) {
        self.debug.painted.clear();
        self.debug.card_rect = None;
        let Some(c) = self.debug.card.clone() else {
            return;
        };
        self.debug.card_rect = Some(r);
        let pad = Spacing::Sm.px();
        let rh = ph2d_tokens::row_pitch_px();
        let font = TypeToken::Sm.px();
        let ink = |t: ColorToken| resolve(t, *theme);
        let (x, w) = (r.x + pad * 2.0, (r.w - pad * 4.0).max(0.0));
        let right = x + w;
        let mut y = r.y + pad;
        let ty = |y: f32| y + (rh - font) * 0.5;
        if !c.stopped {
            // Os valores ao vivo: só o rótulo, no tom apagado dos valores.
            paint_text(
                ts,
                scene,
                &c.title,
                x,
                ty(y),
                font,
                w,
                ink(ColorToken::Text3),
            );
            return;
        }
        // O cabeçalho: título · chip da fila … [Continue] F8  [Next line] F10, à direita.
        let bh = rh - ph2d_tokens::control_gap_px();
        let mut buttons = vec![(
            Ask::Continue,
            CMD_CONTINUE,
            tr("code.debug.continue"),
            tr("code.debug.continue_key"),
        )];
        if c.can_step {
            buttons.push((
                Ask::NextLine,
                CMD_NEXT_LINE,
                tr("code.debug.next_line"),
                tr("code.debug.next_line_key"),
            ));
        }
        let gap = Spacing::Md.px();
        let sized: Vec<_> = buttons
            .into_iter()
            .map(|(ask, id, label, key)| {
                let bw = ts.layout(label, font, f32::INFINITY).width() + pad * 4.0;
                let kw = ts.layout(key, font, f32::INFINITY).width() + Spacing::Xs.px();
                (ask, id, label, key, bw, kw)
            })
            .collect();
        let total: f32 = sized.iter().map(|b| b.4 + pad + b.5 + gap).sum::<f32>() - gap;
        let mut bx = (right - total).max(x);
        let buttons_x = bx;
        let by = y + (rh - bh) * 0.5;
        for (i, (ask, id, label, key, bw, kw)) in sized.into_iter().enumerate() {
            let br = Rect::new(bx, by, bw, bh);
            let b = Button::new(id, label);
            let b = if i == 0 { b.accent() } else { b };
            paint_button(&b, br, scene, ts, *theme);
            self.debug.painted.push((ask, br));
            bx += bw + pad;
            paint_text(ts, scene, key, bx, ty(y), font, kw, ink(ColorToken::Text3));
            bx += kw + gap;
        }
        let room = (buttons_x - gap - x).max(0.0);
        let chip_w = c.waiting.as_ref().map_or(0.0, |s| {
            ts.layout(s, font, f32::INFINITY).width() + pad * 2.0
        });
        let title_room = (room - chip_w - gap).max(0.0);
        let title_w =
            (ts.layout(&c.title, font, f32::INFINITY).width() + Spacing::Xs.px()).min(title_room);
        let title_ink = if c.error.is_some() {
            ColorToken::Danger
        } else {
            ColorToken::Text1
        };
        paint_text_title(
            ts,
            scene,
            &c.title,
            x,
            ty(y),
            font,
            title_room,
            ink(title_ink),
        );
        if let Some(wait) = &c.waiting {
            let cr = Rect::new(x + title_w + gap, by, chip_w.min(room), bh);
            fill_rounded_rect(
                scene,
                cr,
                Radius::Full.px().min(bh * 0.5),
                ink(ColorToken::Bg3),
            );
            paint_text(
                ts,
                scene,
                wait,
                cr.x + pad,
                ty(y),
                font,
                cr.w,
                ink(ColorToken::Text2),
            );
        }
        y += rh;
        if let Some(e) = &c.error {
            paint_text(ts, scene, e, x, ty(y), font, w, ink(ColorToken::Danger));
            y += rh;
        }
        // As variáveis: nome apagado, valor na mono do código.
        let g = self.var_grid(&c, r.w);
        for (i, (name, value)) in c.vars.iter().enumerate() {
            let cx = x + (i % g.cols) as f32 * (g.col_w + g.gap);
            let cy = y + (i / g.cols) as f32 * rh;
            let cell_right = (cx + g.col_w).min(right);
            let name_ink = if value.is_empty() {
                ph2d_panel_code::Ink::Comment
            } else {
                ph2d_panel_code::Ink::Hint
            };
            self.view.paint_code_value(
                ts,
                scene,
                *theme,
                name,
                name_ink,
                (cx, cy, rh),
                (cx + g.name_w).min(cell_right),
            );
            if !value.is_empty() {
                let vx = cx + g.name_w + pad;
                self.view.paint_code_value(
                    ts,
                    scene,
                    *theme,
                    value,
                    ph2d_panel_code::Ink::Plain,
                    (vx, cy, rh),
                    cell_right,
                );
            }
        }
        y += g.rows as f32 * rh;
        if let Some(stack) = &c.stack {
            let label = tr("code.debug.stack");
            let lw = ts.layout(label, font, f32::INFINITY).width();
            paint_text(ts, scene, label, x, ty(y), font, w, ink(ColorToken::Text3));
            let vx = x + lw.max(g.name_w) + pad;
            paint_text(
                ts,
                scene,
                stack,
                vx,
                ty(y),
                font,
                (right - vx).max(0.0),
                ink(ColorToken::Text2),
            );
        }
    }
}

/// A grelha das variáveis do cartão parado.
struct VarGrid {
    cols: usize,
    rows: usize,
    /// A coluna dos nomes dentro de cada célula.
    name_w: f32,
    col_w: f32,
    gap: f32,
}

/// «x = 412, step = 1  ×3  printed 2×» de uma linha.
pub(crate) fn ghost_text(live: &LineLive) -> String {
    let mut parts: Vec<String> = Vec::new();
    let vals: Vec<String> = live
        .values
        .iter()
        .map(|v| format!("{} = {}", v.name, v.value))
        .collect();
    if !vals.is_empty() {
        parts.push(vals.join(", "));
    }
    if live.hits > 1 {
        parts.push(tr_with("code.debug.hits", &[("count", &live.hits)]));
    }
    if live.prints > 0 {
        parts.push(tr_with("code.debug.printed", &[("count", &live.prints)]));
    }
    let s = parts.join("  ");
    if s.chars().count() > GHOST_CHARS {
        let mut cut: String = s.chars().take(GHOST_CHARS).collect();
        cut.push_str(tr("code.fold.ellipsis"));
        cut
    } else {
        s
    }
}

/// Os valores ao vivo de um script, por linha (0-based).
pub(crate) fn ghosts(live: &BTreeMap<u32, LineLive>) -> Vec<Ghost> {
    live.iter()
        .filter(|(n, _)| **n >= 1)
        .map(|(n, l)| Ghost {
            line: *n as usize - 1,
            text: ghost_text(l),
        })
        .filter(|g| !g.text.is_empty())
        .collect()
}

/// O cartão «Stopped» de uma paragem (`who` = o nome do objecto).
pub(crate) fn stop_card(stop: &DebugStop, who: &str) -> DebugCard {
    let line = stop.frame.as_ref().map_or(0, |f| f.line);
    let (title, error) = match &stop.why {
        StopKind::Error(m) => (
            tr_with(
                "code.debug.stopped_error",
                &[("line", &line), ("who", &who)],
            ),
            Some(m.clone()),
        ),
        StopKind::Breakpoint | StopKind::Step => (
            tr_with("code.debug.stopped", &[("line", &line), ("who", &who)]),
            None,
        ),
    };
    let waiting =
        (stop.queued > 0).then(|| tr_with("code.debug.waiting", &[("count", &stop.queued)]));
    let mut vars = Vec::new();
    let mut stack = None;
    if let Some(f) = &stop.frame {
        // Com os campos do `self` à mostra, o «self = table» não diz nada.
        let all: Vec<(String, String)> = f
            .locals
            .iter()
            .filter(|v| f.this.is_empty() || v.name != "self")
            .map(|v| (v.name.clone(), v.value.clone()))
            .chain(
                f.this
                    .iter()
                    .map(|v| (format!("self.{}", v.name), v.value.clone())),
            )
            .collect();
        let more = all.len().saturating_sub(VARS_SHOWN);
        vars.extend(all.into_iter().take(VARS_SHOWN));
        if more > 0 {
            vars.push((
                tr_with("code.debug.more_vars", &[("count", &more)]),
                String::new(),
            ));
        }
        if !f.stack.is_empty() {
            let calls: Vec<String> = f
                .stack
                .iter()
                .map(|(name, n)| {
                    let name = if name.is_empty() {
                        tr("code.debug.top_level")
                    } else {
                        name.as_str()
                    };
                    tr_with("code.debug.frame", &[("function", &name), ("line", n)])
                })
                .collect();
            stack = Some(calls.join(tr("code.debug.stack_sep")));
        }
    }
    DebugCard {
        title,
        waiting,
        error,
        vars,
        stack,
        stopped: true,
        can_step: !matches!(stop.why, StopKind::Error(_)),
    }
}

/// ⭐ **Um quadro do depurador**: os pontos vão ao host, os pedidos também; a paragem e os valores
/// ao vivo voltam para as abas. `watched` = o script da aba de código activa; `selected` = o
/// objecto seleccionado na Hierarchy.
pub(crate) fn frame(
    docs: &mut CodeDocs,
    h: &mut ScriptHost,
    world: &World,
    (watched, selected): (Option<ScriptId>, Option<Entity>),
    toasts: &mut ToastQueue,
) {
    if std::mem::take(&mut docs.debug.toggle_pause) {
        h.set_pause_on_error(!h.pause_on_error());
    }
    PAUSE_ON_ERROR.store(h.pause_on_error(), Ordering::Relaxed);
    if std::mem::take(&mut docs.debug.remove_all) {
        if let Some(lib) = world.get_resource::<ProjectScripts>() {
            for (id, _) in lib.iter() {
                h.set_breakpoints(id, Vec::new());
            }
        }
        for d in &mut docs.docs {
            d.debug.marks.clear();
            d.debug.sent = Some(Vec::new());
        }
    }
    for d in &mut docs.docs {
        match &d.debug.sent {
            // Uma aba que abre lembra-se dos pontos que o host guarda.
            // (os da aba, se já os tem, mandam).
            None => {
                let mine: Vec<u32> = d
                    .breakpoint_lines()
                    .into_iter()
                    .map(|l| l as u32 + 1)
                    .collect();
                if mine.is_empty() {
                    let lines = h.breakpoints(d.script).to_vec();
                    d.set_breakpoint_lines(&lines);
                    d.debug.sent = Some(lines);
                } else {
                    h.set_breakpoints(d.script, mine.clone());
                    d.debug.sent = Some(mine);
                }
            }
            Some(sent) => {
                let lines: Vec<u32> = d
                    .breakpoint_lines()
                    .into_iter()
                    .map(|l| l as u32 + 1)
                    .collect();
                if &lines != sent {
                    h.set_breakpoints(d.script, lines.clone());
                    d.debug.sent = Some(lines);
                }
            }
        }
        d.debug.armed = h.armed_lines(d.script).to_vec();
        d.refresh_marks();
    }
    h.set_live_watch(watched, selected);
    let stop = h.debug_stop();
    let live_who = h.live_object().map(|e| crate::frame::object_name(world, e));
    for d in &mut docs.docs {
        let here = stop.as_ref().filter(|s| s.script == d.script);
        d.debug.stopped = here
            .and_then(|s| s.frame.as_ref())
            .map(|f| (f.line as usize).saturating_sub(1));
        d.debug.ghosts = if Some(d.script) == watched {
            ghosts(&h.live_values(d.script))
        } else {
            Vec::new()
        };
        d.debug.card = match here {
            Some(s) => Some(stop_card(s, &crate::frame::object_name(world, s.entity))),
            None if !d.debug.ghosts.is_empty() => live_who.as_ref().map(|who| DebugCard {
                title: tr_with("code.debug.live", &[("who", who)]),
                ..DebugCard::default()
            }),
            None => None,
        };
        if let Some(a) = d.debug.ask.take() {
            docs.debug.ask = Some(a);
        }
    }
    match docs.debug.ask.take() {
        Some(Ask::Continue) => h.request_continue(),
        Some(Ask::NextLine) => h.request_next_line(),
        None => {}
    }
    follow_stop(docs, world, stop.as_ref());
    let blocked = h.take_blocked_stops();
    let run = h.play_run();
    if blocked > 0 && docs.debug.blocked_told != Some(run) {
        docs.debug.blocked_told = Some(run);
        toasts.push(Toast::warning(tr("code.debug.blocked")));
    }
}

/// Uma paragem NOVA abre (ou activa) a aba do script e leva o cursor à linha.
fn follow_stop(docs: &mut CodeDocs, world: &World, stop: Option<&DebugStop>) {
    let key = stop.map(|s| {
        (
            s.entity,
            s.script,
            s.frame.as_ref().map_or(0, |f| f.line),
            matches!(s.why, StopKind::Error(_)),
        )
    });
    if key == docs.debug.last_stop {
        return;
    }
    docs.debug.last_stop = key;
    let Some(s) = stop else { return };
    if !docs.docs.iter().any(|d| d.script == s.script) {
        let Some(lib) = world.get_resource::<ProjectScripts>() else {
            return;
        };
        let Some(src) = lib.get(s.script) else { return };
        let (name, text) = (src.name.clone(), src.text.clone());
        docs.open(s.script, &name, &text);
    }
    docs.focus = Some(s.script.0);
    let line = s.frame.as_ref().map_or(1, |f| f.line as usize);
    if let Some(d) = docs.docs.iter_mut().find(|d| d.script == s.script) {
        ph2d_code_buffer::goto_line(&mut d.buffer, line);
        d.reveal();
    }
}

impl CodeDocs {
    /// Os comandos do depurador que valem para a família. `true` = era um deles.
    pub(crate) fn debug_command(&mut self, cmd: NodeId) -> bool {
        use crate::commands::{CMD_PAUSE_ON_ERROR, CMD_REMOVE_BREAKPOINTS};
        match cmd {
            CMD_CONTINUE => self.debug.ask = Some(Ask::Continue),
            CMD_NEXT_LINE => self.debug.ask = Some(Ask::NextLine),
            CMD_PAUSE_ON_ERROR => self.debug.toggle_pause = true,
            CMD_REMOVE_BREAKPOINTS => self.debug.remove_all = true,
            _ => return false,
        }
        true
    }
}

/// ⚠️ SONDAS (os testes e o condutor das fotos).
impl CodeDocs {
    /// As linhas (0-based) com ponto de parada na aba `id`.
    #[must_use]
    pub fn breakpoints_of(&self, id: ScriptId) -> Vec<usize> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(CodeDoc::breakpoint_lines)
            .unwrap_or_default()
    }

    /// O que a calha pinta: `(linha, armado)`.
    #[must_use]
    pub fn breakpoint_marks_of(&self, id: ScriptId) -> Vec<(usize, bool)> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| {
                d.debug
                    .marks_view
                    .iter()
                    .map(|b| (b.line, b.armed))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// O cartão do depurador da aba `id`: o título, a mensagem do erro, as linhas e os botões
    /// pintados (`(rótulo, rect)`, 0 = Continue).
    #[must_use]
    pub fn debug_card_of(&self, id: ScriptId) -> Option<(String, Option<String>, Vec<String>)> {
        let c = self
            .docs
            .iter()
            .find(|d| d.script == id)?
            .debug
            .card
            .clone()?;
        let mut rows: Vec<String> = c.waiting.into_iter().collect();
        rows.extend(c.vars.iter().map(|(n, v)| {
            if v.is_empty() {
                n.clone()
            } else {
                format!("{n} = {v}")
            }
        }));
        rows.extend(c.stack.map(|s| format!("{}  {s}", tr("code.debug.stack"))));
        Some((c.title, c.error, rows))
    }

    /// Os botões do cartão pintados no último quadro (0 = Continue, 1 = Next line).
    #[must_use]
    pub fn debug_buttons_of(&self, id: ScriptId) -> Vec<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| d.debug.painted.iter().map(|b| b.1).collect())
            .unwrap_or_default()
    }

    /// A linha parada (0-based) da aba `id`.
    #[must_use]
    pub fn stopped_line_of(&self, id: ScriptId) -> Option<usize> {
        self.docs.iter().find(|d| d.script == id)?.debug.stopped
    }

    /// Os valores ao vivo da aba `id`: `(linha 0-based, texto)`.
    #[must_use]
    pub fn ghosts_of(&self, id: ScriptId) -> Vec<(usize, String)> {
        self.docs
            .iter()
            .find(|d| d.script == id)
            .map(|d| {
                d.debug
                    .ghosts
                    .iter()
                    .map(|g| (g.line, g.text.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// A coluna das marcas na linha da 1.ª `needle` da aba `id` (pintada).
    #[must_use]
    pub fn mark_rect(&self, id: ScriptId, needle: &str) -> Option<Rect> {
        self.docs
            .iter()
            .find(|d| d.script == id)?
            .mark_rect_of(needle)
    }
}
