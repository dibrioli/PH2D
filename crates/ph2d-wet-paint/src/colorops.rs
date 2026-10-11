//! Color math (port of `colorops.js`, SPEC §14): HSV<->RGB for UI, sRGB<->
//! linear, and Kubelka–Munk single-constant subtractive mixing (opt-in).
//!
//! Engine colors are ALWAYS 0..255 floats and are never quantized to bytes:
//! a wet cell can re-mix thousands of times, and a byte round-trip on every
//! mix drifts washes toward black.
//!
//! The sRGB transfer runs only when the K–M checkbox is ON or the glaze stack
//! renders — never in the default hot path (HR-5) — but when it does run it
//! runs 9-15 times per cell, which made it the whole cost of both
//! EXPERIMENTAL knobs. It lives in [`transfer`], node-tabulated from `libm`
//! and interpolated with IEEE basic operations only: same cross-OS
//! bit-identity, ~11x the speed. Read that module before touching it.

/// h in [0,360), s,v in [0,1] -> (r,g,b) 0..255 floats.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [f64; 3] {
    let c = v * s;
    let hp = (((h % 360.0) + 360.0) % 360.0) / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
    let (mut r, mut g, mut b) = (0.0, 0.0, 0.0);
    if hp < 1.0 {
        r = c;
        g = x;
    } else if hp < 2.0 {
        r = x;
        g = c;
    } else if hp < 3.0 {
        g = c;
        b = x;
    } else if hp < 4.0 {
        g = x;
        b = c;
    } else if hp < 5.0 {
        r = x;
        b = c;
    } else {
        r = c;
        b = x;
    }
    let m = v - c;
    [(r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0]
}

/// r,g,b 0..255 -> (h 0..360, s 0..1, v 0..1).
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> [f64; 3] {
    let rn = r / 255.0;
    let gn = g / 255.0;
    let bn = b / 255.0;
    let max = rn.max(gn).max(bn);
    let min = rn.min(gn).min(bn);
    let d = max - min;
    let mut h = 0.0;
    if d > 0.0 {
        if max == rn {
            h = 60.0 * (((gn - bn) / d) % 6.0);
        } else if max == gn {
            h = 60.0 * ((bn - rn) / d + 2.0);
        } else {
            h = 60.0 * ((rn - gn) / d + 4.0);
        }
    }
    if h < 0.0 {
        h += 360.0;
    }
    [h, if max == 0.0 { 0.0 } else { d / max }, max]
}

pub use ph2d_pigment::transfer;

// The standard sRGB EOTF and its inverse, plus the 0..255-domain doors the
// K–M and glaze sites use. ONE door each: every experimental site goes through
// these, and the tables live behind them so no call site can pick a different
// transfer than its neighbour.
pub use ph2d_pigment::{ks_of_srgb255, linear_to_srgb, srgb_to_linear, srgb255_of_linear};

// ── Kubelka–Munk: a lei MUDOU-SE para a folha [`ph2d_pigment`] (ordem do dono 2026-09-20, «os três
// meios passam a misturar igual»). Ela não mudou uma linha de matemática — mudou de ENDEREÇO, para
// que o Digital e a Aquarela leiam esta e não uma cópia. Este motor delega, logo continua
// byte-idêntico; ⛔ nada de reescrever `ColorMix` aqui, que é a segunda cópia que diverge. ──
pub use ph2d_pigment::{ColorMix, reflectance_of_ks};

/// **A transparência do pigmento que o PRODUTO usa** (W1e, F6), medida no Rebelle 8.3.4: azul sobre
/// vermelho SECO cai a `0,42–0,51` do caminho entre «o azul por cima» e «multiplicar as duas»
/// (régua `docs/Painter/NewFeaturesWetPaint/ferramentas/velatura.py`, tabela no 05 §4b). O motor nasce
/// com `0` (o port, que a impressão digital fixa); o produto liga este valor.
///
/// ⚠️ **0,47 → 1,0 com a Opacity do Rebelle (10/10)**: a tinta diluída e a diferença de pigmento pelo ÂNGULO
/// ([`diferenca_de_pigmento`]). Grelha velatura × filme, água parada (seco γ durante · depois · sangria,
/// fracção do caminho; camadas: o azul escondido à 10.ª molhado; barra camadas ≥ 0,70):
///
/// | g · f | 0,47·0,47 | 0,6·0,35 | 0,6·0,40 | 0,6·0,50 | 0,8·0,40 | **1,0·0,40** |
/// |---|---|---|---|---|---|---|
/// | W1e seco | 0,45·0,41 · +0,015 | 0,37·0,32 · +0,017 | 0,41·0,36 · +0,017 | 0,47·0,43 · +0,014 | 0,41·0,36 · +0,017 | **0,41·0,36 · +0,017** |
/// | W1e molhado γ · sangria | 0,29 ✗ | 0,33 | 0,33 · +0,080 | 0,33 | 0,40 · +0,072 | **0,48 · +0,063** |
/// | camadas | 0,60 ✗ | 0,75 | 0,71 | 0,65 ✗ | 0,71 | **0,71** |
///
/// Com a Live Water e o papel Cold (filme 0,50): g 0,6 molhado γ 0,28 ✗ · 0,8 0,34 · **1,0 0,41**, o resto
/// igual (seco 0,43·0,46 · +0,016, camadas 0,85). (Rebelle seco 0,42–0,49 · 0,44–0,51, molhado 0,62;
/// camadas 0,89 molhado · 0,96 seco.) `1` = a multiplicação inteira, pesada pela diferença de pigmento.
///
/// ⚠️⚠️ **0 desde a lei do Rebelle** ([`crate::assenta::Ecra::Cobre`], dono 10/10: a claridade é o pigmento):
/// quem dá o γ do Rebelle é o TAPAR ([`TINTA_ESCONDE`]); a multiplicação por cima só o afastava (tapar 5,
/// molhada 3, sem papel: velatura 0 → seco 0,44·0,43, molhado 0,81; 1 → 0,44·0,43, molhado 0,88). As
/// tabelas acima são da tinta CHEIA diluída, que saiu.
pub const VELATURA_MEDIDA: f64 = 0.0;

/// **A velatura do filme MOLHADO sobre o assentado, no render** (W1f): o azul ainda em suspensão leva o
/// termo inteiro de uma vez (peso ≈ 1), enquanto ao assentar entra aos bocados — com `VELATURA_MEDIDA`
/// aqui, e a água só a correr depois do Up, o azul sobre vermelho SECO a pintar caía a γ 0,76 (Rebelle
/// 0,42–0,49). Varrido no MESMO processo (`sonda_velatura_do_filme`; seco durante · depois):
///
/// | filme | 0,47 | 0,35 | 0,25 | 0,20 | 0,15 | 0 |
/// |---|---|---|---|---|---|---|
/// | γ (W1f) | 0,76 · 0,62 | 0,67 · 0,54 | 0,59 · 0,47 | ≈ 0,54 · 0,43 | 0,50 · 0,38 | 0,35 · 0,25 |
///
/// ⚠️ **Re-medido com «assentar não muda o que se vê»** ([`crate::assenta`], 10/10): o seco passou a ficar com
/// a cor do ecrã, então esta constante vale também DEPOIS. Seco durante · depois · sangria:
///
/// | filme | 0 | 0,10 | 0,15 | 0,20 | **0,25** | 0,35 |
/// |---|---|---|---|---|---|---|
/// | água parada | 0,39 · 0,20 · +3,8 | 0,49 · 0,33 · +3,5 | 0,53 · 0,39 · +3,3 | 0,58 · 0,45 · +3,1 ✗ | **0,62 · 0,50 · +2,9** | 0,69 · 0,60 · +2,6 |
///
/// (Rebelle 0,42–0,49 · 0,44–0,51 · +1…+2, barra da sangria ≤ 3; o molhado não muda: +13.)
///
/// ⚠️ **0,25 → 0,40 com a Opacity do Rebelle (10/10)** — a grelha no doc de [`VELATURA_MEDIDA`].
/// ⚠️⚠️ **0 desde a lei do Rebelle** (a mesma razão de [`VELATURA_MEDIDA`]; a 0,2 o seco ia a 0,52·0,52).
pub const VELATURA_DO_FILME: f64 = 0.0;

/// **A mesma velatura com a água A CORRER no traço** (W1g, «Live Water»): a água que assenta durante o
/// traço muda a conta, e o `0,20` deixava o azul sangrar `+3,2` sobre o vermelho SECO (barra `≤ 3`,
/// Rebelle `+1…+2`). Varrido no MESMO processo, água ao vivo (seco γ durante · depois · sangria):
///
/// | filme | 0,20 | 0,25 | 0,30 | 0,32 | **0,35** | 0,40 | 0,47 |
/// |---|---|---|---|---|---|---|---|
/// | seco | 0,42 · 0,41 · +3,2 | 0,47 · 0,44 · +3,2 | 0,52 · 0,48 · +3,1 | 0,54 · 0,49 · +3,0 | **0,57 · 0,51 · +2,9** | 0,61 · 0,55 · +2,8 | 0,67 · 0,59 · +3 |
///
/// (molhado: γ durante 0,40–0,44, sangria +27…+28 em todos; Rebelle seco 0,42–0,49 · 0,44–0,51.)
/// Com [`crate::assenta`] (10/10), sem papel: 0,20 · 0,25 · **0,35** → seco 0,44·0,45 · 0,50·0,51 ·
/// **0,60·0,61**, sangria +5,8 · +5,4 · **+4,7** (tecto +5 sem papel); com o Cold a barra inteira continua verde.
/// ⚠️ **0,35 → 0,50 com a Opacity do Rebelle (10/10)**, sem papel (seco γ · sangria; camadas molhado): 0,35
/// 0,33·0,34 · +0,032 ✗ · 0,89 · 0,40 0,36·0,37 · +0,030 ✗ · 0,88 · **0,50 0,43·0,45 · +0,027 · 0,85** ·
/// 0,60 0,54·0,57 · +0,023 · 0,25 ✗ (tecto da sangria sem papel 0,028; camadas ≥ 0,70).
/// ⚠️⚠️ **0 desde a lei do Rebelle**: com a Live Water e sem papel a barra passa inteira a 0 (seco 0,40·0,51,
/// sangria seca +0,018, molhada +0,145).
pub const VELATURA_DO_FILME_VIVA: f64 = 0.0;

/// **Quanto a tinta MOLHADA parece mais forte que a seca** ([`crate::sim::Sim::molhada_escurece`]): o Rebelle
/// 8.3.4 mede um traço Round a ~2 s com carga 111 e seco 54 (`fixtures/rebelle_presets/`), a W1e o azul
/// molhado (133,150,231) contra o seco (204,211,245). Varrido na `sonda_presets_varre` (Round pigmento 90,
/// carga molhada a 2 s → seca; Rebelle 111 → 54, o Drip 110 → 60):
///
/// | fator | 1 | 1,4 | **1,6** | 1,8 | 2 | 3 |
/// |---|---|---|---|---|---|---|
/// | Round | — | 100 → 54 | **109 → 54** | 118 → 54 | 126 → 54 | 159 → 54 |
/// | Drip (pigmento 75) | — | 99 → 60 | **109 → 60** | 117 → 60 | — | — |
///
/// ⚠️ Esta tabela é da 1.ª forma (a força do tom de cima). A lei é agora a COBERTURA do ponto inteiro, com a
/// cor da camada fixa (o papel molhado escurece tudo por igual e só até à cor pura; o tom ao secar desliza
/// 0,06, o Rebelle 0,05): fator 2,2 · 2,8 · **3** · 3,5 → Round 88 · 96 · **~100** · 106 → 55; Drip 85 · 92 ·
/// **~96** · 101 → 60. O Wash fica mais escuro molhado que o do Rebelle (67 → 38 contra 46 → 39): a aguada
/// dele está quase seca aos 2 s, a nossa não.
pub const TINTA_MOLHADA_ESCURECE: f64 = 3.0;

/// **Quanto uma camada tapa a de baixo** ([`crate::assenta::Ecra::Cobre`], `h = 1 − (1 − a)^esconde`, o extra
/// pesado pela diferença de pigmento). Grelha `sonda_velatura_recalibra` (sem papel, água parada, molhada 1,6;
/// W1e seco γ · sangria molhada; camadas molhado · seco; o tom ao secar):
///
/// | esconde | 1 | 2 | 3 | 4 | **5** | 6 | 7 |
/// |---|---|---|---|---|---|---|---|
/// | W1e seco | 0,75·0,76 ✗ | 0,70·0,70 | 0,61·0,58 | 0,59·0,56 | **0,48·0,44** | 0,43·0,39 | 0,39·0,35 |
/// | camadas | 0,84 · 0,86 | 0,86 · 0,87 | 0,87 · 0,88 | — | **0,87 · 0,88** | 0,88 · 0,88 | 0,88 · 0,88 |
///
/// (Rebelle: seco 0,42–0,49 · 0,44–0,51, sangria molhada 0,27 — a nossa 0,26–0,28; camadas 0,89 · 0,96.)
pub const TINTA_ESCONDE: f64 = 5.0;

/// **A VELATURA** — o termo que transforma a junção «por cima» de duas tintas na junção de pigmentos
/// TRANSPARENTES: a cor de cima perde, na fração `glaze`, a luz que a de baixo já absorveu.
///
/// `out` chega com a junção por cima (`mix(baixo, cima, w)`, `w = a_cima / A`, `A = 1 − (1−a_b)(1−a_c)`,
/// que é exactamente `alpha_of_mass(m_b + m_c)`). Sobre o branco, `glaze = 1` dá `O_b · O_c` (a lei de
/// Beer–Lambert: absorbâncias somam-se) e `glaze = 0` não toca em `out` — a aritmética de antes ao bit.
///
/// ⚠️ **O termo é da DIFERENÇA de pigmento** (× [`diferenca_de_pigmento`], o ângulo entre os perfis de absorção): a
/// MESMA tinta não se filtra a si mesma — a cobertura `alpha_of_mass` já a escurece pela massa, e uma
/// mancha de uma cor assenta aos BOCADOS (cada passe de secagem é uma junção). Sem o factor, um traço
/// vermelho sozinho escurecia de `(202, 37, 37)` para `(156, 25, 25)` (sonda `sonda_mistura`, 09/10).
#[inline]
pub fn velatura(
    out: &mut [f64; 3],
    w: f64,
    glaze: f64,
    a_baixo: f64,
    baixo: [f64; 3],
    cima: [f64; 3],
) {
    if glaze <= 0.0 || a_baixo <= 0.0 {
        return;
    }
    let k = w * glaze * a_baixo * diferenca_de_pigmento(baixo, cima);
    for c in 0..3 {
        out[c] = (out[c] - k * cima[c] * (1.0 - baixo[c] / 255.0)).max(0.0);
    }
}

/// **Quão diferentes são os PIGMENTOS de duas tintas** (`0..1`): o seno do ângulo entre os perfis de absorção
/// (`255 − cor`). A diluição muda o comprimento do perfil, não a direcção — a mesma tinta mais clara ou mais
/// escura dá `0` (não se vela), e duas tintas diferentes velam o mesmo a qualquer diluição. ⛔ A distância
/// RGB que esteve aqui encolhia com a diluição: com a Opacity do Rebelle (10/10) o azul sobre o vermelho caía a
/// γ 0,20 (Rebelle 0,42–0,51) porque o claro e o claro ficam «perto». O papel branco (perfil nulo) dá `0`.
#[inline]
pub fn diferenca_de_pigmento(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (pa, pb) = (a.map(|v| 255.0 - v), b.map(|v| 255.0 - v));
    let (na, nb) = (
        pa.iter().map(|v| v * v).sum::<f64>(),
        pb.iter().map(|v| v * v).sum::<f64>(),
    );
    if na < 1e-9 || nb < 1e-9 {
        return 0.0;
    }
    let dot = (0..3).map(|c| pa[c] * pb[c]).sum::<f64>();
    let cos2 = (dot * dot / (na * nb)).min(1.0);
    (1.0 - cos2).sqrt()
}

/// K–M weighted mean of up to 4 engine colors (the advection's incoming-color
/// mean when pigment mixing is ON): mixtures are linear in K/S, so average
/// the corners' K/S ratios with the pull weights.
/// `colors` = flat [r0,g0,b0, r1,g1,b1, ...] (0..255); `inv_total` = 1/Σw.
pub fn km_weighted_mean_color(
    colors: &[f64; 12],
    weights: &[f64; 4],
    count: usize,
    inv_total: f64,
    out: &mut [f64; 3],
) {
    for ch in 0..3 {
        let mut ks = 0.0;
        for k in 0..count {
            // One door per corner per channel: the `/255` rescale rides inside
            // the table's index instead of costing a division each.
            ks += weights[k] * ks_of_srgb255(colors[k * 3 + ch]);
        }
        out[ch] = srgb255_of_linear(reflectance_of_ks(ks * inv_total));
    }
}

/// K–M glaze stack of one pigment layer (linear reflectance r_top, coverage a)
/// over a backdrop (linear reflectance r_bot). Energy-bounded: a=1 -> r_top,
/// a=0 -> r_bot. Guard the denominator against r_top * r_bot ~ 1.
#[inline]
pub fn km_glaze_channel_linear(r_bot: f64, r_top: f64, a: f64) -> f64 {
    let rt = a * r_top;
    let tt = 1.0 - a;
    let den = 1.0 - rt * r_bot;
    let r = rt + (tt * tt * r_bot) / if den < 1e-6 { 1e-6 } else { den };
    r.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trips() {
        let [r, g, b] = hsv_to_rgb(210.0, 0.76, 0.82);
        let [h, s, v] = rgb_to_hsv(r, g, b);
        assert!((h - 210.0).abs() < 0.01 && (s - 0.76).abs() < 0.01 && (v - 0.82).abs() < 0.01);
    }

    #[test]
    fn km_mix_endpoints_are_bit_exact_at_any_colour() {
        // A weight of 0 returns dst and a weight of 1 returns src, to the
        // BIT, for every colour — including BELOW the 1/255 reflectance floor,
        // where the K/S round trip provably cannot recover the input (every
        // colour there shares one K/S). The endpoints do not go through K/S at
        // all, so the floor is irrelevant to them; asserting equality rather
        // than a tolerance is what pins that, and it is what keeps a pass that
        // deposits nothing from tinting paint it did not touch.
        let mut out = [0.0; 3];
        for (d, s) in [
            ([200.0, 90.0, 60.0], [40.0, 120.0, 180.0]),
            ([3.0, 1.0, 0.0], [255.0, 254.0, 250.0]), // both ends, floor included
        ] {
            ColorMix::Km.mix(d[0], d[1], d[2], s[0], s[1], s[2], 0.0, &mut out);
            assert_eq!(out, d, "w=0 must return dst unchanged");
            ColorMix::Km.mix(d[0], d[1], d[2], s[0], s[1], s[2], 1.0, &mut out);
            assert_eq!(out, s, "w=1 must return src exactly");
        }
    }

    #[test]
    fn km_yellow_plus_blue_makes_green() {
        // The signature of subtractive mixing: yellow + blue reads GREEN —
        // the green channel dominates harder than in an additive lerp (the
        // absolute values are darker; subtractive mixing absorbs).
        let mut km = [0.0; 3];
        ColorMix::Km.mix(250.0, 205.0, 40.0, 10.0, 70.0, 150.0, 0.5, &mut km);
        let mut plain = [0.0; 3];
        ColorMix::Plain.mix(250.0, 205.0, 40.0, 10.0, 70.0, 150.0, 0.5, &mut plain);
        assert!(km[1] > km[0] && km[1] > km[2], "K-M mid not green: {km:?}");
        let km_dominance = km[1] / km[0].max(km[2]);
        let plain_dominance = plain[1] / plain[0].max(plain[2]);
        assert!(
            km_dominance > plain_dominance,
            "K-M green dominance {km_dominance:.3} !> plain {plain_dominance:.3}"
        );
    }

    /// A VELATURA: `γ = 0` não toca em nada · a MESMA tinta não se vela · cores opostas a `γ = 1` dão
    /// exactamente a multiplicação sobre o branco (Beer–Lambert: `O_baixo · O_cima`).
    #[test]
    fn a_velatura_e_a_multiplicacao_da_diferenca_de_pigmento() {
        let (a_b, a_c) = (0.6, 0.7);
        let big_a = 1.0 - (1.0 - a_b) * (1.0 - a_c);
        let w = a_c / big_a;
        let (baixo, cima) = ([255.0, 0.0, 0.0], [0.0, 255.0, 255.0]);
        let por_cima = |b: [f64; 3], c: [f64; 3]| {
            let mut o = [0.0; 3];
            ColorMix::Plain.mix(b[0], b[1], b[2], c[0], c[1], c[2], w, &mut o);
            o
        };
        let mut o = por_cima(baixo, cima);
        let antes = o;
        velatura(&mut o, w, 0.0, a_b, baixo, cima);
        assert_eq!(o, antes, "γ = 0 tem de ser a junção de antes ao bit");
        let mut igual = por_cima(cima, cima);
        let antes = igual;
        velatura(&mut igual, w, 1.0, a_b, cima, cima);
        assert_eq!(igual, antes, "a mesma tinta não se vela");
        velatura(&mut o, w, 1.0, a_b, baixo, cima);
        for c in 0..3 {
            let sobre_branco = 1.0 - big_a + big_a * o[c] / 255.0;
            let ob = 1.0 - a_b + a_b * baixo[c] / 255.0;
            let oc = 1.0 - a_c + a_c * cima[c] / 255.0;
            assert!(
                (sobre_branco - ob * oc).abs() < 1e-12,
                "canal {c}: {sobre_branco} contra Beer–Lambert {}",
                ob * oc
            );
        }
    }

    #[test]
    fn glaze_is_energy_bounded() {
        assert!((km_glaze_channel_linear(0.3, 0.8, 1.0) - 0.8).abs() < 1e-12);
        assert!((km_glaze_channel_linear(0.3, 0.8, 0.0) - 0.3).abs() < 1e-12);
    }
}
