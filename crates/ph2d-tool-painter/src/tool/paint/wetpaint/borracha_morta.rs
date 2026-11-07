//! **A Borracha inteligente na tinta morta** (decisão 4 do dono, [plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)):
//! sem tinta viva (arquivo antigo, salvo sem a opção), a Borracha do Wet imita o que faz na viva: amassa um sulco, alisa o
//! brilho assado na imagem e só tira a tinta aos poucos, onde a pressão é forte. Trabalha nos pixels: nenhuma sessão nasce.
//!
//! A cobertura vem da Borracha normal medida num escudo ([`Self::mede_o_escudo`]), por isso tamanho, dureza, grão e Flow
//! valem como sempre. As constantes medem-se em `sonda_borracha_morta` contra a viva.

use super::*;
use crate::tool::paint::region::dabs_bounds;
use std::sync::Arc;

/// Quanto o sulco alisa a imagem no centro (0..1).
const ALISA: f32 = 0.8;
/// A largura da parede do sulco e do alisar, em fracção do raio da Borracha.
const PAREDE: f32 = 0.25;
/// A luz na parede do sulco, para um gradiente de cobertura de 1 por raio de parede.
const LUZ_DA_PAREDE: f32 = 0.8;
/// A direcção da luz na imagem (a do Óleo Real de fábrica: a parede à esquerda fica à sombra).
const LUZ: [f32; 2] = [0.85, -0.5];
/// O mais que a parede clareia: a parede de sombra pesa mais do que a de luz, como na viva.
const CLAREIA_NO_MAXIMO: f32 = 1.1;
/// O que uma passada de pressão cheia tira, com o Erase Strength de fábrica ([`ERASE_DE_FABRICA`]).
const TIRA: f32 = 0.35;
const ERASE_DE_FABRICA: f32 = 0.4;
/// A pressão (o contacto da ponta com o papel) pesa assim, acima do [`PISO`].
const EXPOENTE_DA_PRESSAO: i32 = 8;
/// O que a pressão mais leve ainda tira, em fracção de uma passada plena (dono 06/11: também chega ao papel). Medido, mão
/// leve (0,35): 0 nunca chega e queima a parede · 0,15 em ~30 passadas · 0,30 em ~20. A pressão plena não muda.
const PISO: f32 = 0.3;

/// O que sai com o contacto `p`: `PISO` é o que a pressão mais leve ainda leva, e a plena é a de sempre.
fn tira_com_o_contacto(p: f32) -> f32 {
    #[cfg(test)]
    let piso = super::borracha_pressao_tests::piso::get().unwrap_or(PISO);
    #[cfg(not(test))]
    let piso = PISO;
    piso + (1.0 - piso) * p.powi(EXPOENTE_DA_PRESSAO)
}

/// Abaixo disto o alisar só deixou um rasto numérico: não custa um ponto de alfa.
const NADA_SAI: f32 = 1e-3;

/// O sulco do traço aberto: a imagem de antes dele e o que a Borracha já cobriu e tirou.
#[derive(Clone)]
pub(crate) struct Sulco {
    pub(super) antes: Arc<Vec<u8>>,
    cobriu: Vec<u8>,
    tirou: Vec<u8>,
    /// Com sessão viva: as células que tinham tinta na grade no início do traço.
    pub(super) com_tinta: Vec<bool>,
    /// Com sessão viva: o plano de pigmento no início do traço, de que sai quanto a tinta viva cobre cada píxel.
    pub(super) pigmento: Vec<u8>,
}

impl Sulco {
    /// O sulco de um traço novo, a partir da imagem `antes` de `n` píxeis.
    pub(super) fn novo(
        antes: Arc<Vec<u8>>,
        n: usize,
        com_tinta: Vec<bool>,
        pigmento: Vec<u8>,
    ) -> Self {
        Self {
            antes,
            cobriu: vec![0; n],
            tirou: vec![0; n],
            com_tinta,
            pigmento,
        }
    }

    /// Só nos testes: o sulco com a cobertura e o que se tirou iguais em todo o lado, sem passar pelos `dabs`.
    #[cfg(test)]
    pub(super) fn com_cobertura(mut self, cobriu: u8, tirou: u8) -> Self {
        self.cobriu.fill(cobriu);
        self.tirou.fill(tirou);
        self
    }

    /// Acumula a cobertura dos `dabs`, medida no escudo `plano`, na caixa.
    pub(super) fn acumula(&mut self, plano: &[u8], dabs: &[Dab], caixa: Caixa, w: usize) {
        let contacto = dabs
            .iter()
            .map(|d| d.coverage * d.radius_px / d.stroke_radius_px.max(1e-3))
            .sum::<f32>()
            / dabs.len() as f32;
        let tira = tira_com_o_contacto(contacto.clamp(0.0, 1.0));
        for y in caixa.y0..caixa.y1 {
            for x in caixa.x0..caixa.x1 {
                let i = y * w + x;
                let c = 255 - plano[i * 4 + 3];
                self.cobriu[i] = self.cobriu[i].max(c);
                self.tirou[i] = self.tirou[i].max((f32::from(c) * tira) as u8);
            }
        }
    }
}

/// A largura da parede para os `dabs`.
pub(super) fn parede_dos_dabs(dabs: &[Dab]) -> usize {
    let raio = dabs.iter().map(|d| d.radius_px).fold(1.0f32, f32::max);
    ((raio * PAREDE).round() as usize).max(1)
}

/// Uma caixa de pixels `[x0, x1) × [y0, y1)`.
#[derive(Clone, Copy)]
pub(super) struct Caixa {
    pub(super) x0: usize,
    pub(super) y0: usize,
    pub(super) x1: usize,
    pub(super) y1: usize,
}

impl Caixa {
    fn w(self) -> usize {
        self.x1 - self.x0
    }

    fn h(self) -> usize {
        self.y1 - self.y0
    }

    fn cresce(self, m: usize, (w, h): (usize, usize)) -> Self {
        Self {
            x0: self.x0.saturating_sub(m),
            y0: self.y0.saturating_sub(m),
            x1: (self.x1 + m).min(w),
            y1: (self.y1 + m).min(h),
        }
    }
}

/// Duas passadas de caixa de raio `r`, em `v` de `w` colunas: o Gauss barato e sem máquina.
fn alisa_plano(v: &mut [f32], w: usize, r: usize) {
    if r == 0 || w == 0 {
        return;
    }
    let h = v.len() / w;
    let mut aux = vec![0.0f32; w.max(h)];
    for _ in 0..2 {
        for linha in v.chunks_exact_mut(w) {
            caixa_1d(linha, 1, w, r, &mut aux);
        }
        for x in 0..w {
            caixa_1d(&mut v[x..], w, h, r, &mut aux);
        }
    }
}

/// A média móvel de `n` amostras de passo `passo`, com a borda repetida.
fn caixa_1d(v: &mut [f32], passo: usize, n: usize, r: usize, aux: &mut [f32]) {
    let lado = (2 * r + 1) as f32;
    let at = |v: &[f32], i: isize| v[(i.clamp(0, n as isize - 1) as usize) * passo];
    let mut soma: f32 = (-(r as isize)..=r as isize).map(|i| at(v, i)).sum();
    for (i, a) in aux.iter_mut().enumerate().take(n) {
        *a = soma / lado;
        soma += at(v, i as isize + r as isize + 1) - at(v, i as isize - r as isize);
    }
    for (i, a) in aux.iter().enumerate().take(n) {
        v[i * passo] = *a;
    }
}

impl PainterTool {
    /// Esta Borracha é a da tinta morta? Traço livre no Wet, sem sessão, fora de macro.
    pub(in crate::tool::paint) fn borracha_morta_assume(&self) -> bool {
        let w = &self.paint.wetpaint;
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && self.paint.eraser
            && self.paint.brush.stroke_method.is_incremental()
            && w.session.is_none()
            && !w.medindo_o_escudo
            && !w.na_miniatura
            && !w.deposit_pass
            && w.fita.is_none()
            && w.ritmo.is_none()
            && !self.undo.marca.ativa()
            && !self.paint.acumulando_no_plano
    }

    /// Os `dabs` da Borracha sobre a tinta morta: acumula a cobertura do traço e refaz a caixa a partir da imagem de antes.
    pub(in crate::tool::paint) fn borracha_na_tinta_morta(&mut self, dabs: &[Dab]) {
        let (w, h) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let Some(r) = dabs_bounds(
            dabs,
            self.source_size.0,
            self.source_size.1,
            self.paint.brush.tip_reach(),
        ) else {
            return;
        };
        if self.canvas_rgba.len() != w * h * 4 {
            return;
        }
        let caixa = Caixa {
            x0: r.x as usize,
            y0: r.y as usize,
            x1: (r.x + r.w) as usize,
            y1: (r.y + r.h) as usize,
        };
        let plano = self.mede_o_escudo(dabs, r);
        let mut sulco = self.paint.wetpaint.sulco.take().unwrap_or_else(|| {
            Sulco::novo(Arc::clone(&self.canvas_rgba), w * h, Vec::new(), Vec::new())
        });
        sulco.acumula(&plano, dabs, caixa, w);
        self.paint.pilha.escudo = plano;
        let parede = parede_dos_dabs(dabs);
        let alfa_preso = self.wet_splat_gates().2.is_some();
        let forca = if alfa_preso {
            0.0
        } else {
            self.forca_do_sulco()
        };
        self.refaz_a_caixa(&sulco, caixa, parede, forca);
        self.paint.wetpaint.sulco = Some(sulco);
    }

    /// O que uma passada de pressão cheia tira, com o Erase Strength e o Flow de agora.
    pub(super) fn forca_do_sulco(&self) -> f32 {
        (self.paint.wetpaint.knobs.erase as f32 / ERASE_DE_FABRICA)
            * self.paint.brush.flow.clamp(0.0, 1.0)
            * TIRA
    }

    fn refaz_a_caixa(&mut self, sulco: &Sulco, caixa: Caixa, parede: usize, forca: f32) {
        let tam = (self.source_size.0 as usize, self.source_size.1 as usize);
        let esc = caixa_de_escrita(caixa, parede, tam);
        let regiao = crate::compositor::Region {
            x: esc.x0 as u32,
            y: esc.y0 as u32,
            w: esc.w() as u32,
            h: esc.h() as u32,
        };
        let tela = crate::tool::paint::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
            Some(regiao),
        );
        escreve_o_sulco(sulco, caixa, tam, parede, forca, tela, &|_| (1.0, 0.0));
        self.declare_wrote(Some(regiao));
        self.mark_dirty(regiao);
    }
}

/// A escrita chega até onde a parede alcança.
pub(super) fn caixa_de_escrita(caixa: Caixa, parede: usize, tam: (usize, usize)) -> Caixa {
    caixa.cresce(2 * 2 * parede + 2, tam)
}

/// **A lei:** refaz os píxeis da caixa de escrita a partir da imagem de antes do traço. `peso(i)` dá, no píxel `i`, o
/// quanto a lei gradual vale e o quanto a cobertura corta o alfa de uma vez: `(0, 0)` não toca nele.
pub(super) fn escreve_o_sulco(
    sulco: &Sulco,
    caixa: Caixa,
    tam: (usize, usize),
    parede: usize,
    forca: f32,
    tela: &mut [u8],
    peso: &(dyn Fn(usize) -> (f32, f32) + Sync),
) {
    let w = tam.0;
    // A leitura vai até onde alisar a escrita alcança.
    let esc = caixa_de_escrita(caixa, parede, tam);
    let lei = caixa_de_escrita(esc, parede, tam);
    let (lw, lh) = (lei.w(), lei.h());
    let plano_de = |fonte: &[u8]| {
        let mut v = Vec::with_capacity(lw * lh);
        for y in lei.y0..lei.y1 {
            v.extend(
                fonte[y * w + lei.x0..y * w + lei.x1]
                    .iter()
                    .map(|b| f32::from(*b) / 255.0),
            );
        }
        alisa_plano(&mut v, lw, parede);
        v
    };
    let cob = plano_de(&sulco.cobriu);
    let tir = plano_de(&sulco.tirou);
    // A imagem alisada, pré-multiplicada: um vizinho transparente não tinge a tinta.
    let mut canais = [
        vec![0.0f32; lw * lh],
        vec![0.0f32; lw * lh],
        vec![0.0f32; lw * lh],
        vec![0.0f32; lw * lh],
    ];
    for y in 0..lh {
        for x in 0..lw {
            let (k, o) = (y * lw + x, ((y + lei.y0) * w + x + lei.x0) * 4);
            let a = f32::from(sulco.antes[o + 3]) / 255.0;
            for (c, canal) in canais.iter_mut().enumerate().take(3) {
                canal[k] = f32::from(sulco.antes[o + c]) * a;
            }
            canais[3][k] = a;
        }
    }
    for canal in &mut canais {
        alisa_plano(canal, lw, parede);
    }
    let ganho = LUZ_DA_PAREDE * parede as f32;
    for y in esc.y0..esc.y1 {
        for x in esc.x0..esc.x1 {
            let (g, corte) = peso(y * w + x);
            if g <= 0.0 && corte <= 0.0 {
                continue;
            }
            let (lx, ly) = (x - lei.x0, y - lei.y0);
            let k = ly * lw + lx;
            let o = (y * w + x) * 4;
            let c = cob[k];
            if c <= 0.0 {
                tela[o..o + 4].copy_from_slice(&sulco.antes[o..o + 4]);
                continue;
            }
            let lado = |d: isize, passo: usize| -> f32 {
                let i = k as isize + d * passo as isize;
                cob.get(i as usize).copied().unwrap_or(c)
            };
            let gx = if lx > 0 && lx + 1 < lw {
                (lado(1, 1) - lado(-1, 1)) * 0.5
            } else {
                0.0
            };
            let gy = if ly > 0 && ly + 1 < lh {
                (lado(1, lw) - lado(-1, lw)) * 0.5
            } else {
                0.0
            };
            let luz = (1.0 - ganho * (gx * LUZ[0] + gy * LUZ[1])).clamp(0.0, CLAREIA_NO_MAXIMO);
            let am = canais[3][k].max(1e-4);
            let mistura = (c * ALISA).min(1.0);
            for n in 0..3 {
                let antes = f32::from(sulco.antes[o + n]);
                let lisa = canais[n][k] / am;
                let novo = (antes + (lisa - antes) * mistura) * luz;
                tela[o + n] = (antes + (novo - antes) * g).round().clamp(0.0, 255.0) as u8;
            }
            // O corte segue a cobertura crua: alisada, passava do anel da Borracha.
            let cortou = f32::from(sulco.cobriu[y * w + x]) / 255.0 * corte;
            let fica = (1.0 - (tir[k] * forca * g).clamp(0.0, 1.0)) * (1.0 - cortou);
            // Conta sobre o alfa inteiro: em f32, 164/255·255 < 164, e ele perdia um ponto onde não se tirou nada.
            if fica < 1.0 - NADA_SAI {
                tela[o + 3] = (f32::from(sulco.antes[o + 3]) * fica).floor() as u8;
            } else {
                tela[o + 3] = sulco.antes[o + 3];
            }
        }
    }
}
