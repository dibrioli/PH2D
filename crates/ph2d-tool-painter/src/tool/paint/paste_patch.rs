//! **O Paste FLUTUA antes de pousar** (Enio, 2026-08-07: *"no caso do Paste (colar) e retalho deve
//! aparecer com um segundo gizmo de sprite para dar a oportunidade de transformar antes de aplicar.
//! Pode aplicar com o enter."*).
//!
//! Antes disto o Paste compositava na hora, no retângulo de origem: o artista colava e, se quisesse
//! mover, tinha de desfazer. Agora ele **arma uma peça flutuante** — os pixels do clipboard sobre uma
//! moldura orientada (centro, eixo `u`, meias-extensões) — com o MESMO gizmo de transformação de
//! sprite que as formas de seleção usam. **Enter aplica, Esc cancela.**
//!
//! ## Duas coisas decidem o desenho
//!
//! 1. **A dança é a que o canvas já faz** ([`super::stamp_preview`]): *guarda o pristino → desenha →
//!    restaura no quadro seguinte*. A peça é re-desenhada a cada transformação sobre a região salva,
//!    então arrastá-la não deixa rastro e **cancelar não precisa de undo** — nada foi commitado.
//!    ⚠️ O `paste_pristine` mora ao lado do patch e morre com ele: um pristino órfão é um retângulo de
//!    arte velha esperando ser restaurado por cima de tinta nova.
//!
//! 2. **A peça é REAMOSTRADA da fonte, sempre.** Cada quadro compõe a partir do `rgba` original pelo
//!    afim inverso — nunca do resultado do quadro anterior. É a mesma lei que esta linha pagou quatro
//!    vezes no relevo: *reamostrar repetidamente o RESULTADO é um produto sobre a lista de gestos*, e
//!    girar 90° em quatro passos de 22,5° deixaria a peça borrada. Aqui o que se compõe é a MOLDURA
//!    (dois números e um eixo); a imagem é reamostrada uma vez, do original.

use super::selection_gizmo::SelectionGizmoView;
use super::{PainterTool, Region};

/// A peça colada, flutuando sobre a tela até o Enter.
#[derive(Clone, Debug)]
pub(crate) struct PastePatch {
    /// Os pixels do clipboard (straight RGBA), `sw`×`sh`. A FONTE — nunca reescrita.
    pub rgba: Vec<u8>,
    pub sw: u32,
    pub sh: u32,
    /// Centro em px de imagem.
    pub center: [f32; 2],
    /// Eixo unitário local +x (a rotação).
    pub u: [f32; 2],
    /// Meias-extensões ao longo de `u` e da perpendicular (a escala).
    pub hx: f32,
    pub hy: f32,
}

impl PastePatch {
    /// A moldura nascente: a peça pousa exatamente onde foi copiada, sem rotação.
    fn at(rect: Region, rgba: Vec<u8>) -> Self {
        Self {
            rgba,
            sw: rect.w,
            sh: rect.h,
            center: [
                rect.x as f32 + rect.w as f32 * 0.5,
                rect.y as f32 + rect.h as f32 * 0.5,
            ],
            u: [1.0, 0.0],
            hx: rect.w as f32 * 0.5,
            hy: rect.h as f32 * 0.5,
        }
    }

    /// Os quatro cantos da moldura, em px de imagem (ordem: `--`, `+-`, `++`, `-+`).
    #[must_use]
    pub fn corners(&self) -> [[f32; 2]; 4] {
        let (u, v) = (self.u, [-self.u[1], self.u[0]]);
        let (c, hx, hy) = (self.center, self.hx, self.hy);
        let p = |sx: f32, sy: f32| {
            [
                c[0] + u[0] * hx * sx + v[0] * hy * sy,
                c[1] + u[1] * hx * sx + v[1] * hy * sy,
            ]
        };
        [p(-1.0, -1.0), p(1.0, -1.0), p(1.0, 1.0), p(-1.0, 1.0)]
    }

    /// O retângulo de imagem que a peça pode tocar, clampado à tela (`None` quando ela está toda fora).
    fn footprint(&self, w: u32, h: u32) -> Option<Region> {
        let c = self.corners();
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for p in c {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
        // Uma célula de folga em cada lado: a amostragem bilinear lê o vizinho.
        let x0 = (x0.floor() as i64 - 1).max(0) as u32;
        let y0 = (y0.floor() as i64 - 1).max(0) as u32;
        let x1 = ((x1.ceil() as i64 + 1).max(0) as u32).min(w);
        let y1 = ((y1.ceil() as i64 + 1).max(0) as u32).min(h);
        (x1 > x0 && y1 > y0).then_some(Region {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        })
    }

    /// Amostra a peça no ponto de imagem `p`, bilinear; `None` fora dela.
    ///
    /// ⚠️ **A inversa é a TRANSPOSTA porque a base é ortonormal** — `u` é sempre unitário e `v` é a
    /// perpendicular dele —, então a projeção é dois produtos escalares e nada de determinante. É o que
    /// mantém isto livre de caso degenerado: uma escala zero encolhe a peça, nunca a inverte de dentro
    /// para fora.
    fn sample(&self, p: [f32; 2]) -> Option<[u8; 4]> {
        if self.hx <= 0.0 || self.hy <= 0.0 {
            return None;
        }
        let (u, v) = (self.u, [-self.u[1], self.u[0]]);
        let d = [p[0] - self.center[0], p[1] - self.center[1]];
        let lx = d[0] * u[0] + d[1] * u[1]; // ∈ [-hx, hx]
        let ly = d[0] * v[0] + d[1] * v[1]; // ∈ [-hy, hy]
        // Moldura → texel da fonte (centro de pixel).
        let sx = (lx / self.hx * 0.5 + 0.5) * self.sw as f32 - 0.5;
        let sy = (ly / self.hy * 0.5 + 0.5) * self.sh as f32 - 0.5;
        if sx < -0.5 || sy < -0.5 || sx > self.sw as f32 - 0.5 || sy > self.sh as f32 - 0.5 {
            return None;
        }
        let (x0, y0) = (sx.floor(), sy.floor());
        let (fx, fy) = (sx - x0, sy - y0);
        let cl = |a: f32, n: u32| (a.max(0.0) as u32).min(n.saturating_sub(1));
        let (xa, xb) = (cl(x0, self.sw), cl(x0 + 1.0, self.sw));
        let (ya, yb) = (cl(y0, self.sh), cl(y0 + 1.0, self.sh));
        let at = |x: u32, y: u32| {
            let i = ((y * self.sw + x) * 4) as usize;
            [
                f32::from(self.rgba[i]),
                f32::from(self.rgba[i + 1]),
                f32::from(self.rgba[i + 2]),
                f32::from(self.rgba[i + 3]),
            ]
        };
        let (p00, p10, p01, p11) = (at(xa, ya), at(xb, ya), at(xa, yb), at(xb, yb));
        let mut out = [0u8; 4];
        for c in 0..4 {
            // ⚠️ Interpola em ALFA PREMULTIPLICADO nos canais de cor: em straight, um vizinho
            // transparente arrastaria a cor DELE (tipicamente preto) para dentro da borda — o halo.
            let w00 = (1.0 - fx) * (1.0 - fy);
            let w10 = fx * (1.0 - fy);
            let w01 = (1.0 - fx) * fy;
            let w11 = fx * fy;
            let v = if c == 3 {
                p00[3] * w00 + p10[3] * w10 + p01[3] * w01 + p11[3] * w11
            } else {
                let pm = p00[c] * p00[3] * w00
                    + p10[c] * p10[3] * w10
                    + p01[c] * p01[3] * w01
                    + p11[c] * p11[3] * w11;
                let a = p00[3] * w00 + p10[3] * w10 + p01[3] * w01 + p11[3] * w11;
                if a <= 0.0 { 0.0 } else { pm / a }
            };
            out[c] = v.round().clamp(0.0, 255.0) as u8;
        }
        (out[3] > 0).then_some(out)
    }
}

impl PainterTool {
    /// `true` enquanto uma peça colada flutua — a pergunta que o shell faz para desenhar o gizmo, e a
    /// que o roteador de ponteiro faz para dar a ela a primeira palavra.
    #[must_use]
    pub fn paste_patch_live(&self) -> bool {
        self.paint.paste_patch.is_some()
    }

    /// Arma a peça flutuante a partir do clipboard. `false` com o clipboard vazio.
    pub(super) fn arm_paste_patch(&mut self) -> bool {
        let Some(clip) = self.paint.selection_clipboard.clone() else {
            return false;
        };
        if clip.rect.w == 0 || clip.rect.h == 0 {
            return false;
        }
        // Uma peça viva é substituída pela nova — colar duas vezes cola duas vezes, e a primeira já
        // teria de ter sido aplicada ou cancelada para virar tinta.
        self.restore_paste_pristine();
        self.paint.paste_patch = Some(PastePatch::at(clip.rect, clip.rgba));
        self.redraw_paste_patch();
        true
    }

    /// Devolve a tela ao que estava sob a peça (se havia pristino guardado).
    fn restore_paste_pristine(&mut self) {
        if let Some((rect, pixels)) = self.paint.paste_pristine.take() {
            self.restore_region(&rect, &pixels);
            self.mark_dirty(rect);
        }
    }

    /// A dança de um quadro: restaura o anterior, guarda o novo pristino, compõe a peça.
    pub(super) fn redraw_paste_patch(&mut self) {
        self.restore_paste_pristine();
        let Some(patch) = self.paint.paste_patch.clone() else {
            return;
        };
        let (w, h) = self.source_size;
        if w == 0 || h == 0 || self.canvas_rgba.len() != (w as usize) * (h as usize) * 4 {
            return;
        }
        let Some(rect) = patch.footprint(w, h) else {
            return;
        };
        self.paint.paste_pristine = Some((
            rect,
            super::region::region_pixels(&self.canvas_rgba, rect, w),
        ));
        self.composite_patch(&patch, rect);
        self.mark_dirty(rect);
    }

    /// Compõe a peça sobre a tela dentro de `rect` (source-over pelo alfa da peça).
    fn composite_patch(&mut self, patch: &PastePatch, rect: Region) {
        let w = self.source_size.0 as usize;
        let buf = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(rect),
        );
        for y in rect.y..rect.y + rect.h {
            for x in rect.x..rect.x + rect.w {
                let Some(s) = patch.sample([x as f32 + 0.5, y as f32 + 0.5]) else {
                    continue;
                };
                let a = f32::from(s[3]) / 255.0;
                let d = (y as usize * w + x as usize) * 4;
                for c in 0..3 {
                    let src = f32::from(s[c]);
                    let dst = f32::from(buf[d + c]);
                    buf[d + c] = (src * a + dst * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
                }
                let da = f32::from(buf[d + 3]) / 255.0;
                buf[d + 3] = ((a + da * (1.0 - a)) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    /// **Enter — aplica a peça.** Ela já está desenhada na tela; aplicar é PARAR de guardar o pristino,
    /// e gravar UM passo de undo cujo `before` é a tela sem ela.
    ///
    /// ⚠️ A ordem é carregada: restaura o pristino, tira o snapshot (que é o *antes* honesto), e só
    /// então re-compõe a peça para valer. Commitar sobre a tela já composta gravaria um passo cujo
    /// `before` JÁ tem a peça — e o Ctrl+Z não a tiraria.
    pub fn paste_commit(&mut self) -> bool {
        let Some(patch) = self.paint.paste_patch.take() else {
            return false;
        };
        self.restore_paste_pristine();
        let before = self.snapshot_model();
        let (w, h) = self.source_size;
        if let Some(rect) = patch.footprint(w, h) {
            self.composite_patch(&patch, rect);
            self.mark_dirty(rect);
        }
        self.commit_structural_edit(before);
        true
    }

    /// **Esc — descarta a peça.** Nada foi commitado, então não há undo a gravar: basta devolver o
    /// pristino.
    pub fn paste_cancel(&mut self) -> bool {
        if self.paint.paste_patch.take().is_none() {
            return false;
        }
        self.restore_paste_pristine();
        true
    }

    /// O gizmo da peça, na MESMA forma que o shell já desenha para as formas de seleção.
    ///
    /// ⚠️ **`op = 4`** — fora da faixa `0..=3` das operações booleanas, e é o que faz o shell desenhar
    /// um glifo próprio no quadrado central em vez de herdar o `n` do New: a peça não é uma operação
    /// de conjunto, e um `n` ali diria que ela é.
    #[must_use]
    pub fn paste_patch_gizmo(&self) -> Option<SelectionGizmoView> {
        let patch = self.paint.paste_patch.as_ref()?;
        let tol = self.paint.shape_grab_tol_px;
        let c = patch.corners();
        let (u, v) = (patch.u, [-patch.u[1], patch.u[0]]);
        let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
        let handles = [
            c[0],
            c[1],
            c[2],
            c[3],
            mid(c[1], c[2]),
            mid(c[0], c[1]),
            mid(c[3], c[0]),
            mid(c[2], c[3]),
        ];
        let _ = (u, v);
        Some(SelectionGizmoView {
            outline: vec![c[0], c[1], c[2], c[3], c[0]],
            box_corners: c,
            scale_handles: handles,
            center: patch.center,
            diamond: None,
            scale_tol: tol,
            rotate_tol: tol,
            accent: 0,
            op: 4,
            edit_curve: None,
        })
    }
}

// ── O gesto ─────────────────────────────────────────────────────────────────────────────────────────
// Alças, no MESMO vocabulário do gizmo de seleção: `0..=3` quinas, `4..=7` meios de aresta (R,T,L,B),
// `8` rotacionar, `9` mover.
const H_ROTATE: u8 = 8;
const H_MOVE: u8 = 9;
/// O anel de rotação alcança este tanto de raios-de-alça além de um quadrado de escala — o MESMO
/// número do gizmo de seleção, para o gesto ler igual nos dois.
const ROTATE_BAND: f32 = 2.6;

impl PainterTool {
    /// Ponteiro sobre a peça flutuante. `true` quando ela consumiu o evento — enquanto ela vive, ela
    /// tem a primeira palavra sobre o canvas.
    ///
    /// ⚠️ **Um Down FORA de toda alça e fora da peça NÃO a aplica nem a cancela** — ele é consumido e
    /// não faz nada. Aplicar num clique perdido transformaria um toque acidental em tinta, e cancelar
    /// jogaria fora o trabalho de posicionar; as duas saídas são teclas, e é o que as torna deliberadas.
    pub(super) fn paste_patch_pointer(
        &mut self,
        ev: ph2d_editor_core::tool::CanvasPointer,
    ) -> bool {
        use ph2d_editor_core::tool::PointerPhase;
        let Some(patch) = self.paint.paste_patch.clone() else {
            return false;
        };
        match ev.phase {
            PointerPhase::Down => {
                let tol = self.paint.shape_grab_tol_px;
                if let Some(h) = patch_handle_at(&patch, ev.pos, tol) {
                    self.paint.paste_grab = Some((h, ev.pos, patch));
                }
                true
            }
            PointerPhase::Move => {
                if let Some((h, start, initial)) = self.paint.paste_grab.clone() {
                    let next = transformed(&initial, h, start, ev.pos);
                    self.paint.paste_patch = Some(next);
                    self.redraw_paste_patch();
                }
                true
            }
            PointerPhase::Up => {
                self.paint.paste_grab = None;
                true
            }
            // Hover: a peça não age, mas CONSOME — enquanto ela flutua o canvas é dela, e deixar o
            // hover passar acenderia a mira de outra ferramenta por baixo dela.
            PointerPhase::Hover => true,
        }
    }
}

/// Qual alça está sob `p` — quinas/arestas primeiro, depois o anel de rotação, depois o centro, e por
/// fim o CORPO da peça (que move igual ao centro: arrastar a coisa é o gesto que ninguém procura numa
/// alça).
fn patch_handle_at(patch: &PastePatch, p: [f32; 2], tol: f32) -> Option<u8> {
    let c = patch.corners();
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5];
    let handles = [
        c[0],
        c[1],
        c[2],
        c[3],
        mid(c[1], c[2]),
        mid(c[0], c[1]),
        mid(c[3], c[0]),
        mid(c[2], c[3]),
    ];
    let near = |a: [f32; 2], r: f32| {
        let d = [p[0] - a[0], p[1] - a[1]];
        d[0] * d[0] + d[1] * d[1] <= r * r
    };
    // ⚠️ **As alças ESPECÍFICAS primeiro, o centro depois** — a mesma precedência da cadeia do Delete
    // (o alvo mais específico vence). Perguntar pelo centro antes tem um modo de falha medido: numa
    // peça menor que o dobro da tolerância o quadrado central COBRE as quinas, e escalar vira
    // impossível — o gate mediu a quina oposta andando os 16 px inteiros do arrasto, isto é, a peça
    // toda se movendo. Mover continua alcançável pelo CORPO, que é o último degrau.
    for (i, hp) in handles.iter().enumerate() {
        if near(*hp, tol) {
            return Some(i as u8);
        }
    }
    if near(patch.center, tol) {
        return Some(H_MOVE);
    }
    for hp in &handles {
        if near(*hp, tol * ROTATE_BAND) {
            return Some(H_ROTATE);
        }
    }
    patch.sample(p).map(|_| H_MOVE)
}

/// A peça transformada por um arrasto da alça `h`, computada SEMPRE da `initial` pristina — nunca do
/// quadro anterior. É o que mantém um arrasto longo livre de deriva.
fn transformed(initial: &PastePatch, h: u8, start: [f32; 2], now: [f32; 2]) -> PastePatch {
    let mut out = initial.clone();
    let d = [now[0] - start[0], now[1] - start[1]];
    if h == H_MOVE {
        out.center = [initial.center[0] + d[0], initial.center[1] + d[1]];
        return out;
    }
    if h == H_ROTATE {
        // Rotação pelo ÂNGULO VARRIDO em torno do centro, transcendental-free: o seno e o cosseno
        // saem do produto escalar e do produto vetorial dos dois raios normalizados.
        let a = [start[0] - initial.center[0], start[1] - initial.center[1]];
        let b = [now[0] - initial.center[0], now[1] - initial.center[1]];
        let (la, lb) = (
            (a[0] * a[0] + a[1] * a[1]).sqrt(),
            (b[0] * b[0] + b[1] * b[1]).sqrt(),
        );
        if la <= f32::EPSILON || lb <= f32::EPSILON {
            return out;
        }
        let (ax, ay) = (a[0] / la, a[1] / la);
        let (bx, by) = (b[0] / lb, b[1] / lb);
        let (cos, sin) = (ax * bx + ay * by, ax * by - ay * bx);
        let u = initial.u;
        out.u = [u[0] * cos - u[1] * sin, u[0] * sin + u[1] * cos];
        return out;
    }
    // Escala: a alça arrastada anda, a OPOSTA fica parada — que é o que faz uma quina puxar a caixa
    // em vez de a inflar em torno do centro.
    let (u, v) = (initial.u, [-initial.u[1], initial.u[0]]);
    let lx = d[0] * u[0] + d[1] * u[1];
    let ly = d[0] * v[0] + d[1] * v[1];
    // Sinais por alça: quinas mexem nos dois eixos, meios de aresta só no seu.
    let (sx, sy): (f32, f32) = match h {
        0 => (-1.0, -1.0),
        1 => (1.0, -1.0),
        2 => (1.0, 1.0),
        3 => (-1.0, 1.0),
        4 => (1.0, 0.0),  // meio da aresta direita
        5 => (0.0, -1.0), // topo
        6 => (-1.0, 0.0), // esquerda
        _ => (0.0, 1.0),  // base
    };
    const MIN_HALF: f32 = 0.5;
    let hx = (initial.hx + sx * lx * 0.5).max(MIN_HALF);
    let hy = (initial.hy + sy * ly * 0.5).max(MIN_HALF);
    // O centro acompanha metade do crescimento, na direção da alça: é isso que prega o lado oposto.
    let gx = (hx - initial.hx) * sx;
    let gy = (hy - initial.hy) * sy;
    out.hx = hx;
    out.hy = hy;
    out.center = [
        initial.center[0] + u[0] * gx + v[0] * gy,
        initial.center[1] + u[1] * gx + v[1] * gy,
    ];
    out
}

#[cfg(test)]
#[path = "paste_patch_tests.rs"]
mod tests;
