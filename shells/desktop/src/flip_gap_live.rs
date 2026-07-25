//! **Os helpers ao vivo do Gap Closure** (doc `06 §8` — a killer feature de UX do GP):
//! em modo Fill, os vãos que o alcance ATUAL fecha aparecem desenhados no canvas, e
//! Ctrl+roda ajusta o alcance sem largar o mouse.
//!
//! **Por que um worker, e por que não foi uma escolha** (`measure_closures.rs`, medido
//! 2026-07-25): o `preview_closures()` custa **5 ms** num quadro típico (60 traços) e
//! **339 ms** num pesado (300 traços) — recompute por frame está refutado, e o síncrono
//! por tique de scroll também. É o MESMO veredito do ajuste ao vivo do Colorize
//! (`flip_colorize_live.rs`), então é o mesmo padrão: **no máximo UM worker em voo, o
//! pedido mais recente coalescido** — o rate-limiter se auto-pace na taxa do próprio
//! cálculo, sem timer.
//!
//! Diferenças deliberadas para o irmão do Colorize:
//! - **display-only**: o resultado nunca toca o documento ⇒ zero interação com o undo
//!   (não há `live_busy`, não há guard de base congelada);
//! - **resultado STALE é descartado, não instalado**: cada job carrega a CHAVE que
//!   computou (fingerprint das linhas + alcance) e só instala se ela ainda é o alvo —
//!   um helper velho na tela é a killer feature mentindo.
//!
//! Os segmentos instalados ficam em coords de **ARTE**; quem projeta por frame é o
//! overlay (`render_loop/flip_gap_overlay.rs`) — barato, e é o que os mantém colados no
//! desenho enquanto o resultado novo não chega.

use ph2d_editor::Job;
use ph2d_flip::FlipDrawing;
use ph2d_flip_fill::{GapHelper, preview_closures};
use ph2d_tool_flip::{FillMode, FlipMode, FlipStyleSnapshot};

/// A chave de um cálculo: (fingerprint do desenho, bits do alcance em doc).
type GapKey = (u64, u32);

/// **A pergunta do modo, feita UMA vez** — o tick, o overlay e a roda perguntam aqui.
/// Fill sim; **Unpaint não** (o Unpaint não roda o solver, então um helper ali seria a
/// tela prometendo um fechamento que o clique não faz).
#[must_use]
pub(crate) fn wants_gap_helpers(active: bool, style: Option<FlipStyleSnapshot>) -> bool {
    active && style.is_some_and(|s| s.mode == FlipMode::Fill && s.fill_mode != FillMode::Unpaint)
}

/// **Um tique da roda em modo Fill** → o track novo (0..1) do slider `FLIP_GAP`, ou
/// `None` se a roda não é do Gap (modo errado). `notches` positivo = aumentar; 1 tique
/// = [`GAP_WHEEL_STEP`] unidades de MUNDO.
///
/// Devolve o TRACK (normalizado) porque os dois consumidores falam track: o valor do
/// widget no store e o `SetValue` que o tool clampa — o mesmo par que o próprio slider
/// emite num arrasto (`panel-flip/event.rs`).
#[must_use]
pub(crate) fn gap_wheel_track(
    active: bool,
    style: Option<FlipStyleSnapshot>,
    notches: f32,
) -> Option<f64> {
    if !wants_gap_helpers(active, style) || notches == 0.0 {
        return None;
    }
    let s = style?;
    let new = f64::from(notches).mul_add(GAP_WHEEL_STEP, s.gap);
    Some(new.clamp(0.0, ph2d_tool_flip::GAP_MAX_WORLD) / ph2d_tool_flip::GAP_MAX_WORLD)
}

/// Unidades de MUNDO de Gap por tique da roda — 0,05 doc (20 tiques no alcance de 0 a 1,0),
/// a mesma ordem de granularidade dos ~40 tiques que o antigo 1 px/tique dava em 0..40.
const GAP_WHEEL_STEP: f64 = 0.05;

/// FNV-1a sobre o CONTEÚDO do desenho — posições, larguras, `closed` e os flags que
/// separam linha de preenchimento. É um SUPERSET do que o `boundaries()` filtra, de
/// propósito: repetir o filtro aqui seria a 2ª cópia de "o que é fronteira" (a que
/// diverge em silêncio); sobre-invalidar só custa um worker a mais, nunca um helper
/// velho.
fn fingerprint(drawing: &FlipDrawing) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    let mut eat = |b: u64| {
        // FNV-1a byte a byte custaria 8× — um fold por palavra basta para invalidação
        // (não é chave criptográfica nem formato de arquivo).
        h ^= b;
        h = h.wrapping_mul(PRIME);
    };
    for s in &drawing.strokes {
        eat(s.len() as u64);
        eat(u64::from(s.closed)
            | (u64::from(s.hide_stroke) << 1)
            | (u64::from(s.fill.is_some()) << 2));
        for p in s.positions() {
            eat((u64::from(p.x.to_bits()) << 32) | u64::from(p.y.to_bits()));
        }
        for w in s.widths() {
            eat(u64::from(w.to_bits()));
        }
    }
    h
}

/// O estado dos helpers: o resultado instalado + o worker em voo.
#[derive(Default)]
pub(crate) struct GapHelpers {
    /// A chave do resultado INSTALADO — o que o overlay está desenhando.
    key: Option<GapKey>,
    /// Os segmentos instalados, em coords de ARTE (o overlay projeta por frame).
    pub(crate) segments: Vec<GapHelper>,
    /// O cálculo em voo (no máximo UM) e a chave que ele computa.
    job: Option<(GapKey, Job<Vec<GapHelper>>)>,
}

impl GapHelpers {
    /// Fora do modo Fill: nada na tela, nada em voo. (Derrubar o `Job` descarta o
    /// resultado; a thread termina sozinha.)
    pub(crate) fn clear(&mut self) {
        self.key = None;
        self.segments.clear();
        self.job = None;
    }

    /// Um passo da máquina: colhe o worker se terminou, e lança outro se o alvo mudou.
    /// Devolve `true` quando INSTALOU segmentos novos (o chamador pede repaint).
    pub(crate) fn drive(&mut self, reach: f32, drawing: &FlipDrawing) -> bool {
        // Alcance nulo: `closures()` seria vazio por definição — instala vazio sem
        // pagar worker nenhum (é o estado do slider em 0, o default).
        let want: GapKey = (fingerprint(drawing), reach.to_bits());
        if reach <= 0.0 {
            let changed = !self.segments.is_empty();
            self.segments.clear();
            self.key = Some(want);
            self.job = None;
            return changed;
        }

        // ── 1. Colhe o pronto — e só instala se ainda é o ALVO (stale = descarte). ──
        let mut installed = false;
        if let Some((jk, job)) = self.job.as_mut()
            && let Some(segments) = job.try_take()
        {
            let jk = *jk;
            self.job = None;
            if jk == want {
                self.segments = segments;
                self.key = Some(jk);
                installed = true;
            }
        }

        // ── 2. O alvo já está na tela, ou alguém está indo buscá-lo? ──
        if self.key == Some(want) || self.job.is_some() {
            return installed;
        }

        // ── 3. Sai um worker (um só; geometria CLONADA — ele nunca vê o FlipDoc). ──
        let lines = crate::flip_fill_dilate::boundaries(drawing);
        self.job = Some((
            want,
            Job::spawn("gap-helpers", move |_| preview_closures(&lines, reach)),
        ));
        installed
    }
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado do ajuste ao vivo do Colorize): mantém os
    /// helpers do Gap Closure sincronizados com o desenho NA TELA e o alcance atual.
    pub(crate) fn flip_gap_helpers_tick(&mut self) {
        if !wants_gap_helpers(self.flip_active, self.flip_style) {
            self.flip_gap.clear();
            return;
        }
        let Some(style) = self.flip_style else {
            self.flip_gap.clear();
            return;
        };
        // **A MESMA régua do clique** (`fill_click`): o Gap é em unidades de MUNDO, a
        // geometria é LOCAL, então só a escala do objeto atravessa (mundo→local) — SEM
        // `px_to_world`. Helper e clique têm de usar a mesma fórmula, senão a tela mostra
        // um vão que o clique não fecha. É a régua zoom-invariante (Enio 2026-07-25).
        let w2l = self.flip_active_world_to_local();
        let obj_scale = w2l.mean_scale() as f32;
        let Some(gfx) = self.gfx.as_ref() else {
            self.flip_gap.clear();
            return;
        };
        let reach = (style.gap as f32) * obj_scale;

        // O desenho NA TELA, read-only — nunca o `flip_autokey` (que CRIA chave; um
        // overlay que autora seria o gesto acontecendo sem ninguém gesticular).
        let Some((oid, lid)) = crate::flip_strip_resolve::target(&gfx.flip, self.flip_active_layer)
        else {
            self.flip_gap.clear();
            return;
        };
        let Some(obj) = gfx.flip.object(oid) else {
            self.flip_gap.clear();
            return;
        };
        let frame = obj.frame_at(&self.playhead);
        let Some(drawing) = obj
            .layer(lid)
            .and_then(|l| l.drawing_at_cycled(frame))
            .and_then(|did| obj.drawing(did))
        else {
            self.flip_gap.clear();
            return;
        };
        if self.flip_gap.drive(reach, drawing) {
            // Sem repaint o resultado espera o próximo input para aparecer.
            self.title_dirty = true;
        }
    }
}

#[cfg(test)]
#[path = "flip_gap_live_tests.rs"]
mod tests;
