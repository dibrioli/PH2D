//! As leis da tinta que COBRE: a molhada de outra cor ([`Knob::WetCover`](crate::tuning::Knob::WetCover), J6 lacuna 7)
//! e a já ASSENTE ([`Knob::DryCover`](crate::tuning::Knob::DryCover), 31/10, a guache opaca).

use crate::grid::Grid;
use crate::opacity::alpha_of_mass;

/// ⭐ **A tinta nova COBRE a molhada de outra cor** ([`crate::tuning::Knob::WetCover`], J6 lacuna 7): `q = cobre ×
/// outra_tinta`; o suspenso da célula `i` perde a fracção `q × peso` (`peso` = a parte da tinta nova na mistura de
/// opacidade logo abaixo: o que ela esconde sai), e devolve `q` — a mistura dá à cor nova `1 − (1 − peso)(1 − q)` (o
/// que fica por baixo não tinge a de cima). ⛔ Medido: só a massa (sem a cor) deixava o cruzamento lilás claro (a 1.ª
/// mistura já púrpura deixa de ser «outra»: resto 0,36 → 0,50); a massa inteira à 1.ª passagem abria o papel (0,77).
pub(super) fn cobre_a_outra(g: &mut Grid, i: usize, cobre: f64, in_a: f64, tip: [f32; 3]) -> f64 {
    let old = f64::from(g.susp[i]);
    let outra = crate::colorops::outra_tinta(g.susp_rgb[i].map(f64::from), tip.map(f64::from));
    let e9 = alpha_of_mass(old) * (1.0 - in_a);
    if outra <= 0.0 || e9 + in_a <= 0.0 {
        return 0.0;
    }
    let q = (cobre * outra).min(1.0);
    g.susp[i] = (old * (1.0 - q * in_a / (e9 + in_a))) as f32;
    q
}

/// A diferença de canal (÷ 255, o maior dos três) a partir da qual o assentado é OUTRA tinta inteira para o
/// [`cobre_o_assentado`]: abaixo, a fracção é linear (a mesma tinta `0`: a ponta suja do próprio traço sobre o que ele
/// já assentou não se apaga). ⚠️ Não é o [`crate::colorops::outra_tinta`]: o seno dos perfis dá `0` ao BRANCO (perfil
/// nulo) e à mesma tinta mais clara — exactamente o claro opaco sobre o escuro que esta lei existe para cobrir.
const DIFERENCA_QUE_COBRE: f64 = 0.25;

/// ⭐ **A guache COBRE a tinta já assente** ([`crate::tuning::Knob::DryCover`], 31/10): `q = cobre × outra` (`outra` =
/// a diferença de canal ÷ [`DIFERENCA_QUE_COBRE`], ≤ 1); o ASSENTADO da célula `i` perde a fracção `q × peso` (`peso` =
/// a parte da tinta nova na mistura de opacidade com o assentado): o que a tinta opaca esconde é a camada de baixo
/// AFASTADA — o papel volta a aparecer por ela e a tinta nova (molhada, na água) fica à vista com a sua cor. O branco
/// a `1` sobre o azul seco fica branco; num papel de cor, o que aparece é o papel sob o branco — nunca mais claro que
/// a tinta. Sem cor nova para pintar a massa sai (ao contrário do [`cobre_a_outra`], não há mistura a tingir).
pub(super) fn cobre_o_assentado(g: &mut Grid, i: usize, cobre: f64, in_a: f64, tip: [f32; 3]) {
    let old = f64::from(g.sett[i]);
    if old <= 0.0 {
        return;
    }
    let s = g.sett_rgb[i];
    let d = (0..3).fold(0.0f64, |m, k| {
        m.max((f64::from(s[k]) - f64::from(tip[k])).abs())
    });
    let outra = (d / (255.0 * DIFERENCA_QUE_COBRE)).min(1.0);
    let e9 = alpha_of_mass(old) * (1.0 - in_a);
    if outra <= 0.0 || e9 + in_a <= 0.0 {
        return;
    }
    let q = (cobre * outra).min(1.0);
    g.sett[i] = (old * (1.0 - q * in_a / (e9 + in_a))) as f32;
}
