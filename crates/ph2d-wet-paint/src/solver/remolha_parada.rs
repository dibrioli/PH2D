//! ⭐ **A água PARADA dissolve a tinta solúvel** (J11, lei do dono 01/11: «a aguada que chega à tinta grossa MISTURA-SE
//! com a da borda»; smoke J10: «não há a mistura dissolvendo e reduzindo o relevo»): a mesma lei do re-molhar do pincel
//! (`trail::transfer::remolha`), agora a cada passo onde a água está — a poça contra a borda, o que escorre por
//! cima. A fracção `Re-wet × Solubility` do assentado do início do traço é o tecto ([`Grid::sett_do_traco`]); a
//! velocidade pesa pela água da célula. O relevo cai com o assentado: o chão da água lê-o ([`super::chao_forca`]).
//! Sem Re-wet, sem o campo do óleo ou sem Solubility, nada corre, ao bit. Medição: `J11_agua_no_oleo.md`.

use crate::grid::Grid;
use crate::sim::Params;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

/// A fracção do tecto que a água cheia dissolve por passo: o tecto chega em ~1 s de água parada (60 quadros × os
/// passos do quadro), nunca de uma vez. MEDIDO na sonda J11 (`diag_j11_agua_no_oleo`).
pub(crate) const TAXA: f64 = 0.02;

/// Ferramenta de MEDIÇÃO: os bits de um `f32` — `0` = [`TAXA`], `> 0` = outra, `< 0` = desligado.
#[doc(hidden)]
pub static REMOLHA_TAXA: AtomicU32 = AtomicU32::new(0);

fn taxa() -> f64 {
    match REMOLHA_TAXA.load(Relaxed) {
        0 => TAXA,
        b => f64::from(f32::from_bits(b)),
    }
}

/// O passe corre nesta grade: Re-wet no pincel, o campo do óleo e a fotografia do início do traço.
pub(crate) fn liga(g: &Grid, p: &Params) -> bool {
    p.k(crate::tuning::Knob::Dissolve) > 0.0
        && taxa() > 0.0
        && !g.oleo.is_empty()
        && g.sett_do_traco.len() == g.cells
}

/// Dissolve, na caixa molhada, `Re-wet·Solubility·TAXA·min(1, f ÷ FILM_CHEIO)` do assentado do início do traço por
/// passo, até ao tecto `Re-wet·Solubility` dele.
pub(crate) fn aplica(g: &mut Grid, p: &Params) {
    let rw = p.k(crate::tuning::Knob::Dissolve);
    let k = taxa();
    let mut out = [0.0f64; 3];
    for y in g.by0.max(1)..=g.by1 {
        for x in g.bx0.max(1)..=g.bx1 {
            let i = x as usize + y as usize * g.s;
            let film = f64::from(g.film[i]);
            if g.active[i] == 0 || film <= 0.0 || g.sett[i] <= 0.0 {
                continue;
            }
            let f =
                (rw * f64::from(crate::grid::oleo_celula::da_celula(&g.oleo, i).soluvel)).min(1.0);
            if f <= 0.0 {
                continue;
            }
            let s0 = f64::from(g.sett_do_traco[i]);
            let sett = f64::from(g.sett[i]);
            let sobra = sett - s0 * (1.0 - f);
            if sobra <= 0.0 {
                continue;
            }
            let wf = (film / super::ladeira_do_oleo::FILM_CHEIO).min(1.0);
            let dm = (s0 * f * k * wf).min(sobra);
            let sett_rgb = g.sett_rgb[i];
            crate::drying::lift_settled(
                dm / sett,
                &mut g.susp[i],
                &mut g.sett[i],
                &mut g.susp_rgb[i],
                sett_rgb,
                p.mix,
                p.glaze,
                crate::assenta::Ecra::Porto,
                &mut out,
            );
        }
    }
}
