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

use super::selection_actions::ReliefClip;
use super::{PainterTool, Region};
use ph2d_painter_brush::material::MaterialBytes;

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
    /// **O CORPO da peça** — os três planos do impasto, `None` quando a tinta copiada era chapada.
    ///
    /// Ele viaja pela MESMA moldura da cor (mesmo `sw`/`sh`, mesma `weights`), e é isso que garante que
    /// pigmento e relevo pousem exatamente nos mesmos texels sob qualquer rotação ou escala.
    pub relief: Option<ReliefClip>,
}

impl PastePatch {
    /// A moldura nascente: a peça pousa exatamente onde foi copiada, sem rotação.
    fn at(rect: Region, rgba: Vec<u8>, relief: Option<ReliefClip>) -> Self {
        Self {
            rgba,
            relief,
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
        (x1 > x0 && y1 > y0).then(|| Region {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        })
    }

    /// **A GEOMETRIA da amostragem — uma porta, dois passageiros.** Devolve os quatro texels da fonte e
    /// os pesos bilineares para o ponto de imagem `p`, ou `None` fora da moldura.
    ///
    /// ⚠️ **A inversa é a TRANSPOSTA porque a base é ortonormal** — `u` é sempre unitário e `v` é a
    /// perpendicular dele —, então a projeção é dois produtos escalares e nada de determinante. É o que
    /// mantém isto livre de caso degenerado: uma escala zero encolhe a peça, nunca a inverte de dentro
    /// para fora.
    ///
    /// ⚠️ **Cor e RELEVO perguntam a ESTA função, nunca cada um à sua.** Uma segunda cópia da projeção
    /// divergiria no primeiro arredondamento, e o que se veria é pigmento e corpo pousando em texels
    /// diferentes sob rotação — as duas metades da mesma tinta discordando sobre onde ela está.
    fn weights(&self, p: [f32; 2]) -> Option<([usize; 4], [f32; 4])> {
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
        let idx = |x: u32, y: u32| (y * self.sw + x) as usize;
        Some((
            [idx(xa, ya), idx(xb, ya), idx(xa, yb), idx(xb, yb)],
            [
                (1.0 - fx) * (1.0 - fy),
                fx * (1.0 - fy),
                (1.0 - fx) * fy,
                fx * fy,
            ],
        ))
    }

    /// Amostra a COR da peça no ponto de imagem `p`; `None` fora dela ou onde ela é transparente.
    fn sample(&self, p: [f32; 2]) -> Option<[u8; 4]> {
        let (ix, wt) = self.weights(p)?;
        let px = |k: usize, c: usize| f32::from(self.rgba[ix[k] * 4 + c]);
        let mut out = [0u8; 4];
        // ⚠️ Interpola em ALFA PREMULTIPLICADO nos canais de cor: em straight, um vizinho transparente
        // arrastaria a cor DELE (tipicamente preto) para dentro da borda — o halo.
        let a: f32 = (0..4).map(|k| px(k, 3) * wt[k]).sum();
        for (c, o) in out.iter_mut().take(3).enumerate() {
            let pm: f32 = (0..4).map(|k| px(k, c) * px(k, 3) * wt[k]).sum();
            let v = if a <= 0.0 { 0.0 } else { pm / a };
            *o = v.round().clamp(0.0, 255.0) as u8;
        }
        out[3] = a.round().clamp(0.0, 255.0) as u8;
        (out[3] > 0).then_some(out)
    }

    /// Amostra o CORPO da peça no ponto `p`: `(altura, cobertura, material)`. `None` sem relevo, fora da
    /// moldura, ou onde a peça não tem tinta nenhuma.
    ///
    /// ⚠️ **A cobertura interpola em bruto, a ALTURA e o MATERIAL ponderados por ela** — o mesmo raciocínio
    /// do alfa premultiplicado acima, e pela mesma razão: num texel de borda o vizinho vazio tem altura 0 e
    /// material 0, e misturar isso na média *afinaria e desbotaria* a tinta que existe em vez de apenas
    /// desvanecê-la. A cobertura já é o canal que conta *quanto há*.
    fn sample_relief(&self, p: [f32; 2]) -> Option<(f32, u8, MaterialBytes)> {
        let r = self.relief.as_ref()?;
        let (ix, wt) = self.weights(p)?;
        let cov: f32 = (0..4).map(|k| f32::from(r.covers[ix[k]]) * wt[k]).sum();
        if cov <= 0.0 {
            return None;
        }
        let hw: f32 = (0..4)
            .map(|k| r.heights[ix[k]] * f32::from(r.covers[ix[k]]) * wt[k])
            .sum();
        let mut mats = [0u8; 7];
        for (c, m) in mats.iter_mut().enumerate() {
            let v: f32 = (0..4)
                .map(|k| f32::from(r.mats[ix[k]][c]) * f32::from(r.covers[ix[k]]) * wt[k])
                .sum();
            *m = (v / cov).round().clamp(0.0, 255.0) as u8;
        }
        Some((hw / cov, cov.round().clamp(0.0, 255.0) as u8, mats))
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
        self.paint.paste_patch = Some(PastePatch::at(clip.rect, clip.rgba, clip.relief));
        self.redraw_paste_patch();
        true
    }

    /// Devolve a tela ao que estava sob a peça (se havia pristino guardado).
    fn restore_paste_pristine(&mut self) {
        let Some(pr) = self.paint.paste_pristine.take() else {
            return;
        };
        self.restore_region(&pr.rect, &pr.rgba);
        self.restore_relief_region(&pr);
        self.mark_dirty(pr.rect);
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
        self.paint.paste_pristine = Some(self.snapshot_paste_pristine(&patch, rect, w));
        self.composite_patch(&patch, rect);
        self.mark_dirty(rect);
    }

    /// **Enter — aplica a peça.** Ela já está desenhada na tela; aplicar é PARAR de guardar o pristino,
    /// e gravar UM passo de undo cujo `before` é a tela sem ela.
    ///
    /// ⚠️ A ordem é carregada: restaura o pristino, tira o snapshot (que é o *antes* honesto), e só
    /// então re-compõe a peça para valer. Commitar sobre a tela já composta gravaria um passo cujo
    /// `before` JÁ tem a peça — e o Ctrl+Z não a tiraria.
    pub fn paste_commit(&mut self) -> bool {
        self.macro_recusa("paste_commit");
        let Some(patch) = self.paint.paste_patch.take() else {
            return false;
        };
        self.restore_paste_pristine();
        let before = self.abre_edicao_estrutural();
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
        self.macro_recusa("paste_cancel");
        if self.paint.paste_patch.take().is_none() {
            return false;
        }
        self.restore_paste_pristine();
        true
    }
}

#[path = "paste_patch_compose.rs"]
mod compose;
#[path = "paste_patch_input.rs"]
mod input;
pub(crate) use compose::PastePristine;

#[cfg(test)]
#[path = "paste_patch_tests.rs"]
mod tests;
