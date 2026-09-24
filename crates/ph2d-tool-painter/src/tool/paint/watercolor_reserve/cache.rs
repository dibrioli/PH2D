//! **O CAMPO DA RESERVA PERSISTE ENTRE QUADROS** (ADR-0173, 3.ª ronda — a aquarela da FOTO do dono,
//! 2026-09-23: *«quando usamos tudo que o pincel pode fazer, temos significativa queda de FPS»*).
//!
//! ## O que a medição mostrou
//!
//! Com `Bleed 48` e `Ragged Edge 48` o `pad` da janela do composite é `2·48 + 48 + 2 = 146 px`, e a
//! janela de LEITURA é a de saída alargada por ele OUTRA vez: `~875²` texels por quadro para um traço
//! que avançou `~16 px`. O campo da reserva era refeito sobre ela INTEIRA a cada quadro, e o
//! amostrador da thread principal punha-o em **`30 %`** do quadro — a maior fatia.
//!
//! ## A lei que o torna guardável
//!
//! As somas do campo são INTEIRAS (ver o cabeçalho do pai): o valor num texel é função só dos planos
//! `nível`/`proximidade` a distância de Chebyshev `≤ R` dele (as quatro passagens somam `r₁ + r₂ = R`
//! em cada eixo, e o corte do troço na borda de uma janela só pesa a menos de `r` dela). Logo:
//!
//! 1. um texel a mais de `R` da borda de uma janela tem o MESMO valor em qualquer janela que o
//!    contenha — e o composite só amostra a mais de `reach + ½ ≥ R + ½` da borda da janela de leitura;
//! 2. entre dois quadros, os planos mudam SÓ dentro do rectângulo sujo do quadro (os dabs — o
//!    `smear_level` escreve no disco do dab ACTUAL, e o `splat` idem); o campo muda só no sujo `⊕ R`.
//!
//! ⇒ o campo vive num plano do tamanho do CANVAS, e cada quadro recalcula só **o sujo `⊕ R`** e os
//! LADRILHOS da janela que nunca foram calculados. O resto lê-se do quadro anterior, e é o MESMO
//! `f32`: nenhuma soma mudou de ordem, porque nenhuma soma é de vírgula flutuante.
//!
//! ## A invariante: calculado uma vez, certo para sempre
//!
//! Toda mudança dos planos cai num sujo `D` que o composite desse quadro recalcula (`D ⊕ R`), e
//! `D ⊕ R` cabe sempre na janela (o `pad` dela já é `≥ R`). Logo um texel que alguma vez foi
//! calculado continua CERTO até uma escrita em massa invalidar o plano — e o que falta a uma janela
//! nova é só o que NUNCA foi calculado. ⚠️ **A 1.ª redacção guardava UM rectângulo** (a janela do
//! quadro anterior) e recalculava a diferença; a janela TREME uns pixels de quadro para quadro (o
//! sujo de cada quadro tem outro tamanho), e cada tremor era uma faixa de 1–3 px nos QUATRO lados
//! que custava o avental `⊕ R` inteiro: medido, `~0,2 Mtx` por quadro contra `~0,11` do sujo. Os
//! ladrilhos ([`LADO`]) só entram quando a janela os descobre pela primeira vez.
//!
//! ⚠️ **A amostragem é a do `sample_bilinear` à letra, na janela de LEITURA** — o `clamp`, o `floor` e
//! a fracção em coordenadas LOCAIS; só o endereço do texel soma a origem, e em INTEIROS. Somar a
//! origem ao `sx` em `f32` perderia bits da fracção (o ulp a `4096` é `2⁻¹¹` contra `2⁻¹⁴` a `500`) e
//! os pesos do bilinear mudariam.
//!
//! ⚠️ **Toda escrita EM MASSA nos planos invalida o plano guardado** (`wet_reserve_cache = None`): a
//! criação com o backfill, o zerar do `clear_wet_coverage`, o fim da sessão e os dois resets do
//! fundo. Uma escrita em massa que esquecesse a invalidação daria um campo VELHO — o gate
//! `o_campo_guardado_da_o_byte_do_campo_refeito` corre uma sessão inteira contra o refeito.

use rayon::prelude::*;

use super::{RASCUNHO, Rascunho, ReserveFields, ReserveWindow, campos, lei_antiga};
use crate::compositor::Region;
use crate::tool::paint::watercolor_field::WetStrokeStyle;

/// Um rectângulo do canvas, meio-aberto: `[x0, x1) × [y0, y1)`.
type Rect = (usize, usize, usize, usize);

/// O lado de um ladrilho da invariante (px). Um ladrilho custa o avental `⊕ R` à volta dele quando é
/// calculado sozinho, logo um lado MAIOR que o `R` típico (`14` na foto do dono, `≤ 256`) e pequeno o
/// bastante para a janela não arrastar meio canvas a cada ladrilho novo.
const LADO: usize = 64;

/// O campo da reserva do canvas inteiro, um plano por raio distinto, e QUE ladrilhos dele já foram
/// calculados (ver a invariante no cabeçalho).
pub(in crate::tool::paint) struct ReserveCache {
    fw: usize,
    fh: usize,
    /// Os raios da sessão, ordenados e sem repetição — a MESMA ordem que o [`campos`] devolve.
    radii: Vec<u16>,
    fields: Vec<Vec<f32>>,
    /// Um por ladrilho de [`LADO`]², linha a linha.
    feito: Vec<bool>,
}

impl ReserveFields {
    /// O campo da janela de leitura `win`, lido do plano guardado e recalculado só onde a janela é
    /// nova ou onde os planos mudaram desde o último composite (`changed`, o sujo do quadro).
    pub(in crate::tool::paint) fn build_cached(
        planes: (&[u8], &[u8]),
        win: ReserveWindow,
        table: &[WetStrokeStyle],
        cur: &WetStrokeStyle,
        changed: Option<Region>,
        cache: Option<ReserveCache>,
    ) -> Option<Self> {
        let (fw, n, (rx0, ry0), (rw, rh)) = win;
        let (level, prox) = planes;
        if level.len() != n || prox.len() != n || n == 0 || fw == 0 || rw == 0 || rh == 0 {
            return None;
        }
        if lei_antiga() {
            return Self::build(planes, win, table, cur);
        }
        let fh = n / fw;
        let mut radii: Vec<u16> = table.iter().map(|s| s.reserve_r).collect();
        radii.push(cur.reserve_r);
        radii.sort_unstable();
        radii.dedup();
        let rmax = usize::from(*radii.last()?);
        let mut c = match cache {
            Some(c) if c.fw == fw && c.fh == fh && c.radii == radii => c,
            _ => ReserveCache {
                fw,
                fh,
                fields: radii.iter().map(|_| vec![0.0; n]).collect(),
                radii,
                feito: vec![false; fw.div_ceil(LADO) * fh.div_ceil(LADO)],
            },
        };
        // 1. O sujo do quadro `⊕ R`: onde os planos mudaram, o campo mudou (num plano acabado de
        //    nascer não há nada calculado que envelheça — os ladrilhos em falta cobrem-no).
        if let Some(d) = changed.filter(|_| c.feito.iter().any(|&f| f)) {
            let (dx, dy) = (d.x as usize, d.y as usize);
            let grown = (
                dx.saturating_sub(rmax),
                dy.saturating_sub(rmax),
                (dx + d.w as usize + rmax).min(fw),
                (dy + d.h as usize + rmax).min(fh),
            );
            if grown.0 < grown.2 && grown.1 < grown.3 {
                recompute(&mut c, planes, table, cur, grown, rmax);
            }
        }
        // 2. Os ladrilhos da janela que nunca foram calculados: troços de cada fila, e troços IGUAIS
        //    de filas seguidas juntam-se num rectângulo (o 1.º quadro é a janela inteira, e fila a
        //    fila pagaria o avental `⊕ R` uma vez por fila).
        let tw = fw.div_ceil(LADO);
        let (tx0, tx1) = (rx0 / LADO, (rx0 + rw).div_ceil(LADO));
        let (ty0, ty1) = (ry0 / LADO, (ry0 + rh).div_ceil(LADO));
        let mut trocos: Vec<(usize, usize, usize, usize)> = Vec::new(); // (x0, x1, y0, y1) em ladrilhos
        for ty in ty0..ty1 {
            let mut tx = tx0;
            while tx < tx1 {
                if c.feito[ty * tw + tx] {
                    tx += 1;
                    continue;
                }
                let t0 = tx;
                while tx < tx1 && !c.feito[ty * tw + tx] {
                    c.feito[ty * tw + tx] = true;
                    tx += 1;
                }
                match trocos
                    .iter_mut()
                    .find(|t| t.0 == t0 && t.1 == tx && t.3 == ty)
                {
                    Some(t) => t.3 = ty + 1,
                    None => trocos.push((t0, tx, ty, ty + 1)),
                }
            }
        }
        for (x0, x1, y0, y1) in trocos {
            let u = (
                x0 * LADO,
                y0 * LADO,
                (x1 * LADO).min(fw),
                (y1 * LADO).min(fh),
            );
            recompute(&mut c, planes, table, cur, u, rmax);
        }
        Some(Self {
            by_r: Vec::new(),
            cache: Some(c),
            org: (rx0, ry0),
            rw,
            rh,
            nearest: false,
        })
    }

    /// Devolve o plano guardado ao estado do pincel, para o quadro seguinte.
    pub(in crate::tool::paint) fn devolve(self) -> Option<ReserveCache> {
        self.cache
    }

    /// O `sample_bilinear` da janela de leitura, à letra, lido do plano do canvas.
    #[inline]
    pub(super) fn sample_cached(&self, c: &ReserveCache, r: u16, sx: f32, sy: f32) -> f32 {
        let k = c.radii.iter().position(|&br| br == r).unwrap_or(0);
        let f = &c.fields[k];
        let (w, h) = (self.rw, self.rh);
        let fx = sx.clamp(0.0, (w - 1) as f32);
        let fy = sy.clamp(0.0, (h - 1) as f32);
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(w - 1);
        let y1 = (y0 + 1).min(h - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;
        let at = |x: usize, y: usize| f[(self.org.1 + y) * c.fw + self.org.0 + x];
        let (a, b, cc, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
        let top = a + (b - a) * tx;
        let bot = cc + (d - cc) * tx;
        top + (bot - top) * ty
    }
}

/// Recalcula o campo em `u`, sobre a janela `u ⊕ R` (a distância `> R` de toda borda que não seja
/// a do canvas, logo o valor é o de qualquer janela maior), e copia só `u` para o plano guardado.
fn recompute(
    c: &mut ReserveCache,
    planes: (&[u8], &[u8]),
    table: &[WetStrokeStyle],
    cur: &WetStrokeStyle,
    u: Rect,
    rmax: usize,
) {
    let w = (
        u.0.saturating_sub(rmax),
        u.1.saturating_sub(rmax),
        (u.2 + rmax).min(c.fw),
        (u.3 + rmax).min(c.fh),
    );
    let (ww, wh) = (w.2 - w.0, w.3 - w.1);
    let janela = (c.fw, (w.0, w.1), (ww, wh));
    // ⚠️ `try_borrow_mut`, como no pai: numa espera do rayon esta thread pode roubar outra tarefa
    // que também constrói um campo.
    let by_r = RASCUNHO.with(|r| match r.try_borrow_mut() {
        Ok(mut g) => campos(&mut g, planes, janela, table, cur),
        Err(_) => campos(&mut Rascunho::default(), planes, janela, table, cur),
    });
    debug_assert!(by_r.iter().map(|(r, _)| *r).eq(c.radii.iter().copied()));
    let (fw, uw) = (c.fw, u.2 - u.0);
    for ((_, src), dst) in by_r.iter().zip(c.fields.iter_mut()) {
        dst.par_chunks_mut(fw)
            .skip(u.1)
            .take(u.3 - u.1)
            .enumerate()
            .for_each(|(k, row)| {
                let so = (u.1 + k - w.1) * ww + (u.0 - w.0);
                row[u.0..u.2].copy_from_slice(&src[so..so + uw]);
            });
    }
}
