//! Per-cell simulation state (port of `grid.js`, SPEC §2) + the canvas-wide
//! actions that are pure field operations (SPEC §12: wet/dry canvas, clear).
//!
//! Layout: the paintable canvas is W x H cells, one cell per pixel, stored in
//! padded arrays with stride S = W + 2 and rows = H + 2 (one border cell on
//! every side). Interior cells are x in [1..W], y in [1..H]; index x + y*S.
//! Brushes may only touch [2..W-1] x [2..H-1] — the outermost interior ring is
//! a drain the boundary pass wipes every frame, so drips run off the sheet
//! instead of pooling at the edge.
//!
//! Two pigment layers is the core model: brushes deposit SUSPENDED pigment
//! (moves with the flow); drying transfers it to SETTLED (stuck to the paper);
//! re-wetting lifts a little back. Two velocity fields is the core trick:
//! gravity accumulates in the PERSISTENT field, while the TRANSIENT flow is
//! rebuilt from it every frame — the absorbency brake only applies during the
//! 1-in-4 rebuild, so a drip keeps its momentum on the other three frames.

use crate::colorops::ColorMix;
use crate::opacity::alpha_of_mass;
use crate::paper::PaperPreset;

pub const DEFAULT_WIDTH: usize = 900;
pub const DEFAULT_HEIGHT: usize = 450;

pub struct Grid {
    pub w: usize,
    pub h: usize,
    /// Stride = w + 2 (one pad column each side).
    pub s: usize,
    pub rows: usize,
    pub cells: usize,
    /// Free water depth, 0..waterCap.
    pub film: Vec<f32>,
    /// Suspended pigment mass (moves with the flow).
    pub susp: Vec<f32>,
    /// Suspended color, 0..255 floats, interleaved [r, g, b] per cell.
    /// (The JS keeps three planes; interleaving is a pure layout change —
    /// same f32 values — that halves the array count the hot passes touch.)
    pub susp_rgb: Vec<[f32; 3]>,
    /// Settled pigment mass (dried onto the paper, does not move).
    pub sett: Vec<f32>,
    /// Settled color, interleaved like `susp_rgb`.
    pub sett_rgb: Vec<[f32; 3]>,
    /// Persistent velocity (gravity lands here).
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    /// Transient flow, rebuilt from the persistent field each frame.
    pub flow_x: Vec<f32>,
    pub flow_y: Vec<f32>,
    /// Wetness byte: "the sheet is damp here".
    pub wet: Vec<u8>,
    /// Paper tooth height 0..1, baked incl. the pad ring.
    pub paper: Vec<f32>,
    /// 0 = skip, 1/2 = solver processes this cell.
    pub active: Vec<u8>,
    /// Per-cell budget for the backrun extension.
    pub bloom: Vec<u8>,
    // Active bounding box (the solver iterates only inside it) + fluid flag.
    pub bx0: i32,
    pub by0: i32,
    pub bx1: i32,
    pub by1: i32,
    pub has_fluid: bool,
    /// **A FAIXA VIVA, por linha** (inclusiva; `lo > hi` = vazia) — a extensão
    /// em x que o solver pode precisar tocar naquela linha.
    ///
    /// A bbox é o CASCO da água, e um casco mente sobre um traço diagonal: um
    /// traço de 50 px de largura cruzando a tela tem casco = tela e água =
    /// 2,4% dele (medido a 4096², `tests/measure_pass_cost.rs`). Todo passe
    /// itera as linhas da bbox, então 97,6% de cada varredura era desperdício,
    /// e o tick pagava **pela CAIXA, não pela poça**.
    ///
    /// A faixa é um **SUPERCONJUNTO DECLARADO**, mantido em três portas — a
    /// [`Grid::expand_bbox`] (por onde TODA água nova entra), o
    /// [`crate::solver::rebuild_active_region`] (que a reaperta a partir do
    /// que observou) e o snapshot de histórico (que a carrega junto, porque
    /// ela É estado do grid). O
    /// invariante que a torna correta é `active ⊆ faixa`, e ele vale **por
    /// construção**: o rebuild escreve `active` só dentro da própria janela
    /// de varredura e depois publica a faixa como a extensão viva DILATADA
    /// por [`SPAN_PAD`], que contém aquelas escritas.
    ///
    /// Isso é o que torna a restrição byte-idêntica: todo passe já faz
    /// early-out por-célula (`active[i] == 0` → `continue`), então pular uma
    /// célula fora da faixa é pular uma célula que não responderia nada.
    pub row_lo: Vec<i32>,
    pub row_hi: Vec<i32>,
    /// Rascunho do rebuild: a extensão VIVA observada nesta passada, antes da
    /// dilatação que a publica. Mora no grid para o rebuild não alocar.
    pub live_lo: Vec<i32>,
    pub live_hi: Vec<i32>,
    /// Ablação da faixa: `false` faz [`Grid::span_x`] devolver a bbox inteira,
    /// isto é, exatamente o que o motor varria antes dela.
    ///
    /// ⚠️ Não é uma segunda implementação — é o MESMO laço com um intervalo
    /// mais largo (a lição do ADR-0120/0124: um caminho lento paralelo é o que
    /// diverge em silêncio). Serve ao gate diferencial (mesma sessão, os dois
    /// modos, fingerprint idêntico) e a bissecar em campo.
    pub spans_enabled: bool,
    // Paper identity (re-baked by paper.rs; part of history snapshots).
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
}

/// Dilatação (em células) que a faixa viva ganha sobre a extensão observada —
/// o análogo por-linha do pad ±5 que a bbox já carrega.
///
/// É folgadíssimo de propósito: `maxVelocity` default é **0,2 célula/frame** e
/// o rebuild roda a cada 2 frames, então a água anda 0,4 célula entre duas
/// republicações da faixa. O pad também é o que cobre a célula que ACABOU de
/// sair do ativo e ainda carrega velocidade (ver o net de debug do `advect`).
pub const SPAN_PAD: i32 = 5;

/// [`Grid::span_x`] em forma livre — os passes que já desestruturam o `Grid`
/// (split-borrow, para o compilador provar os índices do estêncil) não podem
/// chamar o método, e duas cópias da regra divergiriam. Uma implementação,
/// duas formas de chamada.
#[inline]
#[must_use]
pub fn span_x_of(
    row_lo: &[i32],
    row_hi: &[i32],
    enabled: bool,
    bx0: i32,
    bx1: i32,
    y: i32,
) -> (i32, i32) {
    if !enabled {
        return (bx0, bx1);
    }
    let r = y as usize;
    (bx0.max(row_lo[r]), bx1.min(row_hi[r]))
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let s = width + 2;
        let rows = height + 2;
        let n = s * rows;
        Grid {
            w: width,
            h: height,
            s,
            rows,
            cells: n,
            film: vec![0.0; n],
            susp: vec![0.0; n],
            susp_rgb: vec![[0.0; 3]; n],
            sett: vec![0.0; n],
            sett_rgb: vec![[0.0; 3]; n],
            vel_x: vec![0.0; n],
            vel_y: vec![0.0; n],
            flow_x: vec![0.0; n],
            flow_y: vec![0.0; n],
            wet: vec![0; n],
            paper: vec![0.0; n],
            active: vec![0; n],
            bloom: vec![0; n],
            bx0: 0,
            by0: 0,
            bx1: -1,
            by1: -1,
            has_fluid: false,
            row_lo: vec![i32::MAX; rows],
            row_hi: vec![i32::MIN; rows],
            live_lo: vec![i32::MAX; rows],
            live_hi: vec![i32::MIN; rows],
            spans_enabled: true,
            paper_preset: PaperPreset::Cold,
            paper_sheet: 0,
        }
    }

    /// A faixa a varrer na linha `y`, já intersectada com a bbox — a porta
    /// ÚNICA por onde todo passe pergunta "até onde vou nesta linha?".
    ///
    /// A interseção com a bbox é o que torna a restrição segura em qualquer
    /// caso: o resultado nunca contém uma célula que o motor não visitava
    /// antes, então a faixa só pode REMOVER visitas — e as que ela remove têm
    /// `active == 0` (invariante `active ⊆ faixa`).
    #[inline]
    #[must_use]
    pub fn span_x(&self, y: i32) -> (i32, i32) {
        span_x_of(
            &self.row_lo,
            &self.row_hi,
            self.spans_enabled,
            self.bx0,
            self.bx1,
            y,
        )
    }

    /// Declara que as linhas `y0..=y1` podem ter vida em `x0..=x1` — a porta
    /// por onde a água NOVA entra na atenção do solver. Só cresce.
    pub fn note_live(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        if x0 > x1 || y0 > y1 {
            return;
        }
        let y0 = y0.max(0) as usize;
        let y1 = (y1.max(0) as usize).min(self.rows - 1);
        for r in y0..=y1 {
            if x0 < self.row_lo[r] {
                self.row_lo[r] = x0;
            }
            if x1 > self.row_hi[r] {
                self.row_hi[r] = x1;
            }
        }
    }

    /// Esvazia a faixa viva em toda linha (nada a varrer).
    pub fn clear_spans(&mut self) {
        self.row_lo.fill(i32::MAX);
        self.row_hi.fill(i32::MIN);
    }

    /// A JANELA de varredura do rebuild: a faixa viva com margem ±2, clampada
    /// aos índices válidos da linha (inclui o anel de pad). `lo > hi` = vazia.
    ///
    /// A margem espelha o `±2` que o `px0/px1` já dá à bbox, e é o que cobre
    /// as três coisas que o rebuild toca fora da própria faixa: o trio
    /// horizontal do passe 1, as escritas `active[i±1]` dele, e o trio
    /// vertical do passe 2.
    #[inline]
    #[must_use]
    pub fn span_window(&self, y: i32) -> (i32, i32) {
        if !self.spans_enabled {
            return (0, self.s as i32 - 1);
        }
        let r = y as usize;
        if self.row_lo[r] > self.row_hi[r] {
            return (1, 0); // vazia
        }
        (
            (self.row_lo[r] - 2).max(0),
            (self.row_hi[r] + 2).min(self.s as i32 - 1),
        )
    }

    /// Zera o rascunho de extensão viva (topo do rebuild).
    pub fn clear_live(&mut self) {
        self.live_lo.fill(i32::MAX);
        self.live_hi.fill(i32::MIN);
    }

    /// Publica a faixa viva a partir do rascunho: dilatação de [`SPAN_PAD`]
    /// nos DOIS eixos (a linha `y` herda o que as linhas vizinhas observaram,
    /// e o intervalo cresce o mesmo pad em x).
    ///
    /// É a dilatação que sustenta os dois invariantes que o resto do motor
    /// assume: `active ⊆ faixa` (as escritas do rebuild ficam a ≤2 células de
    /// uma célula viva) e "velocidade fora da faixa é zero" (a célula que
    /// acabou de deixar o ativo está a ≤1 célula da água que a deixou).
    pub fn publish_spans_from_live(&mut self) {
        let rows = self.rows as i32;
        let w = self.w as i32;
        for y in 0..rows {
            let mut lo = i32::MAX;
            let mut hi = i32::MIN;
            let d0 = (y - SPAN_PAD).max(0);
            let d1 = (y + SPAN_PAD).min(rows - 1);
            for r in d0..=d1 {
                let r = r as usize;
                if self.live_lo[r] <= self.live_hi[r] {
                    lo = lo.min(self.live_lo[r]);
                    hi = hi.max(self.live_hi[r]);
                }
            }
            let r = y as usize;
            if lo <= hi {
                self.row_lo[r] = (lo - SPAN_PAD).max(1);
                self.row_hi[r] = (hi + SPAN_PAD).min(w);
            } else {
                self.row_lo[r] = i32::MAX;
                self.row_hi[r] = i32::MIN;
            }
        }
    }

    /// Empty the active bbox and drop the fluid flag.
    pub fn empty_bbox(&mut self) {
        self.bx0 = 0;
        self.by0 = 0;
        self.bx1 = -1;
        self.by1 = -1;
        self.has_fluid = false;
        // ⚠️ A faixa NÃO é zerada aqui, e isso é a indução inteira: a água
        // pode ter acabado com VELOCIDADE fóssil espalhada pelo rastro, e é a
        // faixa que lembra onde. Zerá-la aqui quebra o caso base — o próximo
        // traço traria a bbox de volta sobre aquelas células e o `advect`, que
        // hoje as zera, não chegaria mais nelas (a rede de debug pegou isto).
        // Quem zera a faixa é quem zera a velocidade: [`clear_canvas`].
    }

    /// Grow the active bbox to include a rect (canvas cell coords), clamped
    /// to the interior.
    pub fn expand_bbox(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let x0 = x0.max(1);
        let y0 = y0.max(1);
        let x1 = x1.min(self.w as i32);
        let y1 = y1.min(self.h as i32);
        if x0 > x1 || y0 > y1 {
            return;
        }
        if !self.has_fluid || self.bx0 > self.bx1 {
            self.bx0 = x0;
            self.by0 = y0;
            self.bx1 = x1;
            self.by1 = y1;
        } else {
            if x0 < self.bx0 {
                self.bx0 = x0;
            }
            if y0 < self.by0 {
                self.by0 = y0;
            }
            if x1 > self.bx1 {
                self.bx1 = x1;
            }
            if y1 > self.by1 {
                self.by1 = y1;
            }
        }
        self.has_fluid = true;
        // A faixa viva acompanha: esta é a porta por onde a água nova entra.
        self.note_live(x0, y0, x1, y1);
    }
}

/// **A REDE DE DEBUG da faixa viva** — os três invariantes de que a restrição
/// depende, afirmados a cada passo em build de debug (compilada FORA em
/// release, onde as sondas de 4096² rodam).
///
/// Ela existe porque duas das três afirmações são *por construção* e a
/// terceira não é, e um leitor futuro não tem como saber qual é qual olhando
/// um laço. Aqui as três estão escritas, e a suíte inteira do crate —
/// aceitação SPEC §18, product doors, rewet, flood — as exerce sobre física
/// real. É o mesmo padrão de rede que a porta única de escrita de plano do
/// Painter usa: *esquecer é lento, nunca errado*.
///
/// 1. `active ⊆ faixa` — por construção (o rebuild publica a faixa como a
///    extensão viva dilatada por [`SPAN_PAD`], que contém as próprias
///    escritas dele).
/// 2. `{film > 0 ∪ susp > 0} ⊆ faixa` — por construção (a faixa é publicada
///    exatamente desse conjunto; a água nova entra por [`Grid::expand_bbox`]).
/// 3. **fora da faixa, `vel == 0`** — INVARIANTE, não construção. É a única
///    afirmação que sustenta o estreitamento do `advect`, cujo ramo inativo
///    escreve em vez de pular.
#[cfg(debug_assertions)]
pub fn verify_spans(g: &Grid) {
    if !g.spans_enabled {
        return; // faixa == bbox: nada a afirmar
    }
    let s = g.s as i32;
    for y in 0..g.rows as i32 {
        let r = y as usize;
        let (lo, hi) = (g.row_lo[r], g.row_hi[r]);
        let base = r * g.s;
        for x in 0..s {
            let i = base + x as usize;
            let inside = x >= lo && x <= hi;
            if inside {
                continue;
            }
            assert!(
                g.active[i] == 0,
                "faixa viva nao cobre uma celula ATIVA em ({x}, {y}): faixa [{lo}, {hi}]"
            );
            assert!(
                g.film[i] <= 0.0 && g.susp[i] <= 0.0,
                "faixa viva nao cobre agua/pigmento em ({x}, {y}): film {} susp {}, faixa [{lo}, {hi}]",
                g.film[i],
                g.susp[i]
            );
            // O anel de dreno é escrito incondicionalmente pelo
            // `apply_boundaries` e nunca é visitado pelo advect (a bbox para
            // em [1, w] × [1, h]), então ele fica de fora da afirmação 3.
            let ring = x <= 1 || x >= g.w as i32 || y <= 1 || y >= g.h as i32;
            let in_bbox = x >= g.bx0 && x <= g.bx1 && y >= g.by0 && y <= g.by1;
            if in_bbox && !ring {
                assert!(
                    g.vel_x[i] == 0.0 && g.vel_y[i] == 0.0,
                    "velocidade sobrevivente fora da faixa em ({x}, {y}): \
                     ({}, {}) -- o advect nao chegaria mais nela",
                    g.vel_x[i],
                    g.vel_y[i]
                );
            }
        }
    }
}

/// Wetness byte a given paper tooth stamps: valleys (low paper) read wetter.
#[inline]
pub fn wet_byte_from_paper(paper_value: f64) -> u8 {
    let mut v = 2.0 - 2.0 * paper_value;
    if v > 1.0 {
        v = 1.0;
    }
    if v < 0.0 {
        v = 0.0;
    }
    (v * 255.0) as u8
}

// ---------------------------------------------------------------------------
// Canvas-wide actions (SPEC §12)
// ---------------------------------------------------------------------------

/// Wet canvas: raise the wetness byte to the paper-derived value via max over
/// the whole interior. Injects NO water and touches no bbox — the sim stays
/// idle, but subsequent strokes bleed everywhere and show-wet reads damp.
pub fn wet_canvas(g: &mut Grid) {
    let s = g.s;
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let b = wet_byte_from_paper(g.paper[i] as f64);
            if b > g.wet[i] {
                g.wet[i] = b;
            }
            i += 1;
        }
    }
}

/// Dry canvas: one-shot O(area) — settle every cell's suspended mass into the
/// settled layer (opacity-composite color, same as the dry pass), zero water,
/// both velocity fields and wetness, and empty the bbox.
pub fn dry_canvas(g: &mut Grid, mix: ColorMix) {
    let s = g.s;
    let mut out = [0.0f64; 3];
    for y in 1..=g.h {
        let mut i = 1 + y * s;
        for _x in 1..=g.w {
            let dm = g.susp[i] as f64;
            if dm > 0.0 {
                settle_composite(g, i, dm, mix, &mut out);
                g.sett[i] = (g.sett[i] as f64 + dm) as f32;
                g.susp[i] = 0.0;
            }
            g.film[i] = 0.0;
            g.vel_x[i] = 0.0;
            g.vel_y[i] = 0.0;
            g.flow_x[i] = 0.0;
            g.flow_y[i] = 0.0;
            g.wet[i] = 0;
            i += 1;
        }
    }
    g.empty_bbox();
}

/// Opacity-composite `dm` of suspended pigment into the settled layer's color
/// at cell i (SPEC §6.2 step 3). NOT mass-weighted: coverage-weighted, so a
/// thin new glaze barely shifts an already-opaque settled color.
#[inline]
pub fn settle_composite(g: &mut Grid, i: usize, dm: f64, mix: ColorMix, out: &mut [f64; 3]) {
    let a_sett = alpha_of_mass(g.sett[i] as f64);
    let a_in = alpha_of_mass(dm);
    if a_sett > 0.0 {
        let u = a_sett * (1.0 - a_in);
        let w = a_in / (u + a_in);
        let sc = g.sett_rgb[i];
        let uc = g.susp_rgb[i];
        mix.mix(
            sc[0] as f64,
            sc[1] as f64,
            sc[2] as f64,
            uc[0] as f64,
            uc[1] as f64,
            uc[2] as f64,
            w,
            out,
        );
        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    } else {
        g.sett_rgb[i] = g.susp_rgb[i];
    }
}

/// Clear: zero all dynamic state; the paper is untouched.
pub fn clear_canvas(g: &mut Grid) {
    g.film.fill(0.0);
    g.susp.fill(0.0);
    g.susp_rgb.fill([0.0; 3]);
    g.sett.fill(0.0);
    g.sett_rgb.fill([0.0; 3]);
    g.vel_x.fill(0.0);
    g.vel_y.fill(0.0);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.fill(0);
    g.active.fill(0);
    g.bloom.fill(0);
    g.empty_bbox();
    // Aqui a faixa PODE ser zerada: esta é a porta que zerou a velocidade.
    g.clear_spans();
}

// ---------------------------------------------------------------------------
// History snapshots (SPEC §15). The transient flow arrays are scratch rebuilt
// every frame from the persistent field, so they are not captured.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct GridSnapshot {
    pub film: Vec<f32>,
    pub susp: Vec<f32>,
    pub susp_rgb: Vec<[f32; 3]>,
    pub sett: Vec<f32>,
    pub sett_rgb: Vec<[f32; 3]>,
    pub vel_x: Vec<f32>,
    pub vel_y: Vec<f32>,
    pub wet: Vec<u8>,
    pub active: Vec<u8>,
    pub bloom: Vec<u8>,
    pub bx0: i32,
    pub by0: i32,
    pub bx1: i32,
    pub by1: i32,
    pub has_fluid: bool,
    /// A faixa viva ([`Grid::row_lo`]) viaja no snapshot porque ela É estado
    /// do grid, não uma vista dele.
    ///
    /// ⚠️ A alternativa — abrir a faixa inteira no restore e deixar o próximo
    /// rebuild reapertá-la — foi MEDIDA e reprovada: a varredura viva do
    /// rebuild passa a cobrir a folha toda, **0,3 → 17,4 ms a 4096²**, ou
    /// seja um quadro perdido a cada Ctrl+Z do motor. Um snapshot que já
    /// carrega dez planos não fica mais caro por dois vetores de `i32`.
    pub row_lo: Vec<i32>,
    pub row_hi: Vec<i32>,
    pub paper_preset: PaperPreset,
    pub paper_sheet: u32,
}

pub fn snapshot_grid(g: &Grid) -> GridSnapshot {
    GridSnapshot {
        film: g.film.clone(),
        susp: g.susp.clone(),
        susp_rgb: g.susp_rgb.clone(),
        sett: g.sett.clone(),
        sett_rgb: g.sett_rgb.clone(),
        vel_x: g.vel_x.clone(),
        vel_y: g.vel_y.clone(),
        wet: g.wet.clone(),
        active: g.active.clone(),
        bloom: g.bloom.clone(),
        bx0: g.bx0,
        by0: g.by0,
        bx1: g.bx1,
        by1: g.by1,
        has_fluid: g.has_fluid,
        row_lo: g.row_lo.clone(),
        row_hi: g.row_hi.clone(),
        paper_preset: g.paper_preset,
        paper_sheet: g.paper_sheet,
    }
}

/// Restore a snapshot. Returns true when the paper identity changed (the
/// caller re-bakes the sheet).
pub fn restore_grid(g: &mut Grid, s: &GridSnapshot) -> bool {
    g.film.copy_from_slice(&s.film);
    g.susp.copy_from_slice(&s.susp);
    g.susp_rgb.copy_from_slice(&s.susp_rgb);
    g.sett.copy_from_slice(&s.sett);
    g.sett_rgb.copy_from_slice(&s.sett_rgb);
    g.vel_x.copy_from_slice(&s.vel_x);
    g.vel_y.copy_from_slice(&s.vel_y);
    g.flow_x.fill(0.0);
    g.flow_y.fill(0.0);
    g.wet.copy_from_slice(&s.wet);
    g.active.copy_from_slice(&s.active);
    g.bloom.copy_from_slice(&s.bloom);
    g.bx0 = s.bx0;
    g.by0 = s.by0;
    g.bx1 = s.bx1;
    g.by1 = s.by1;
    g.has_fluid = s.has_fluid;
    // A faixa viajou junto: um snapshot descreve o grid INTEIRO, e derivá-la
    // de volta seria uma segunda resposta à pergunta que o rebuild responde.
    g.row_lo.copy_from_slice(&s.row_lo);
    g.row_hi.copy_from_slice(&s.row_hi);
    let paper_changed = g.paper_preset != s.paper_preset || g.paper_sheet != s.paper_sheet;
    g.paper_preset = s.paper_preset;
    g.paper_sheet = s.paper_sheet;
    paper_changed
}
