#![forbid(unsafe_code)]
//! `ph2d-pigment` — **como duas tintas se misturam**, e nada mais.
//!
//! Esta folha existe por ordem do dono (2026-09-20): *«trocar as duas para a lei do Wet Paint — os
//! três meios passam a misturar igual»*. A lei **não é nova**: ela é o Kubelka–Munk de constante
//! única que o motor de fluido ([`ph2d-wet-paint`]) já shipava, MOVIDO para aqui sem uma linha de
//! matemática mudada, para que o Digital e a Aquarela leiam a MESMA e não uma cópia.
//!
//! ⚠️ **Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.** O que este repo já
//! pagou meia dúzia de vezes é a segunda cópia que diverge quando alguém afina a primeira; por isso
//! o `ph2d-wet-paint` **delega** aqui em vez de guardar a sua, e as três casas partilham as tabelas.
//!
//! ## A lei
//!
//! Uma reflectância `R` mapeia para `K/S = (1−R)² / 2R`; misturas são **LINEARES em K/S**, logo
//! converte-se, interpola-se e inverte-se (`R = 1 + KS − √(KS² + 2·KS)`). É isto que faz azul com
//! amarelo dar VERDE: em luz eles somam para cinzento, em pigmento eles subtraem.
//!
//! ⚠️ **A metade cara é a transferência sRGB, e ela é uma TABELA** ([`transfer`]) — a mesma que
//! torna o resultado bit-idêntico entre sistemas operativos, que é a propriedade que a lei do porte
//! exige de um transcendental.

pub mod transfer;

pub use transfer::{
    ks_of_srgb255, ks_of_srgb255_exact, linear_to_srgb, linear_to_srgb_exact, srgb_to_linear,
    srgb_to_linear_exact, srgb255_of_linear, srgb255_to_linear,
};

/// A metade INVERSA do K–M (K/S → reflectância). Fica fora da tabela de propósito: é um `sqrt` e
/// não há nada para tabular.
#[inline]
#[must_use]
pub fn reflectance_of_ks(ks: f64) -> f64 {
    1.0 + ks - (ks * ks + 2.0 * ks).sqrt()
}

/// How the engine blends two pigment colors. `Plain` is the default (the
/// composites of SPEC §6/§10/§11); `Km` is the "pigment mixing (K–M)"
/// checkbox. An enum, not a fn pointer: the hot loops match on
/// it once per call and the compiler sees through both arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMix {
    Plain,
    Km,
}

impl ColorMix {
    /// Mix dst -> src by weight w, both 0..255 float channels, into `out`.
    ///
    /// ⚠️ **Isenção NOMEADA (`9` argumentos contra o tecto de `7`), e ela é a assinatura que os
    /// laços quentes do motor de fluido já chamam** — empacotá-los num tipo custaria a cópia por
    /// célula que esta forma existe para não pagar. A crate de origem tolerava-a por um
    /// `allow(clippy::too_many_arguments)` de CRATE INTEIRA; aqui a licença é deste método só.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn mix(
        self,
        dr: f64,
        dg: f64,
        db: f64,
        sr: f64,
        sg: f64,
        sb: f64,
        w: f64,
        out: &mut [f64; 3],
    ) {
        match self {
            ColorMix::Plain => {
                out[0] = dr + (sr - dr) * w;
                out[1] = dg + (sg - dg) * w;
                out[2] = db + (sb - db) * w;
            }
            ColorMix::Km => {
                // A mix that moves nothing must CHANGE nothing, to the bit.
                // Mathematically w=0 is dst and w=1 is src, so this is not a
                // shortcut past the model, it is the model's own answer — and
                // taking it matters because a weight of exactly 0 does occur
                // (a settle or a lift whose incoming mass rounds to no
                // coverage) and it recurs on the SAME cell every pass. The
                // tabulated round trip is accurate but is not an identity, so
                // without this a still wash would be nudged forever by passes
                // that deposit nothing (see `transfer`'s fixed-point note).
                if w <= 0.0 {
                    *out = [dr, dg, db];
                    return;
                }
                if w >= 1.0 {
                    *out = [sr, sg, sb];
                    return;
                }
                // Otherwise: into K/S through the door, lerp (mixtures are
                // LINEAR in K/S — that is the whole of the model), and back
                // out through the inverse. The `/255` rescale and the floor
                // ride inside the door's table index.
                let iw = 1.0 - w;
                let ks_r = iw * ks_of_srgb255(dr) + w * ks_of_srgb255(sr);
                let ks_g = iw * ks_of_srgb255(dg) + w * ks_of_srgb255(sg);
                let ks_b = iw * ks_of_srgb255(db) + w * ks_of_srgb255(sb);
                out[0] = srgb255_of_linear(reflectance_of_ks(ks_r));
                out[1] = srgb255_of_linear(reflectance_of_ks(ks_g));
                out[2] = srgb255_of_linear(reflectance_of_ks(ks_b));
            }
        }
    }
}

/// A porta do PINCEL: duas cores em bytes sRGB misturadas como tinta, `t = 0` → `a`, `t = 1` → `b`.
///
/// ⚠️ **Cores IGUAIS devolvem a entrada, ao BIT, e essa metade é a que torna a troca de lei
/// segura:** a ida-e-volta pela tabela é exacta ao dígito mas **não é uma identidade**, e no
/// depósito da aquarela um traço de UMA cor volta a misturar o mesmo texel a cada dab — sem este
/// atalho ele derivaria um byte de cada vez ao longo do traço. Com ele, **todo traço de uma cor só
/// sai byte-idêntico** ao que saía antes desta lei existir, que é o corpus inteiro de gates.
#[inline]
#[must_use]
pub fn mix_srgb8(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    if a == b || t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let mut out = [0.0f64; 3];
    ColorMix::Km.mix(
        f64::from(a[0]),
        f64::from(a[1]),
        f64::from(a[2]),
        f64::from(b[0]),
        f64::from(b[1]),
        f64::from(b[2]),
        f64::from(t),
        &mut out,
    );
    [
        (out[0] + 0.5).clamp(0.0, 255.0) as u8,
        (out[1] + 0.5).clamp(0.0, 255.0) as u8,
        (out[2] + 0.5).clamp(0.0, 255.0) as u8,
    ]
}

/// A mesma porta em floats `[0, 1]` — a forma que o motor de pincel e a lavagem falam.
///
/// ⛔⛔ **Ela NÃO passa por bytes, e a 1.ª redacção passava.** A tentação é reusar a
/// [`mix_srgb8`] com um arredondamento de cada lado; medido, isso **quantiza a saída a `1/255`** e
/// apaga diferenças que o composite da lavagem carrega em vírgula flutuante — o gate
/// `watercolor_soak_deepens_and_widens_the_dissolve_while_parked` passou a ler *«2 s de demora
/// deram o MESMO pixel que a passagem rápida»*, com a lei certa e a precisão perdida. ⭐ E não é
/// preciso: a porta do K/S é uma **tabela interpolada**, logo aceita `0..255` fraccionário — a
/// escala entra sem arredondar e a volta sai sem arredondar.
#[inline]
#[must_use]
pub fn mix_unit(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    if a == b || t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let mut out = [0.0f64; 3];
    ColorMix::Km.mix(
        f64::from(a[0].clamp(0.0, 1.0)) * 255.0,
        f64::from(a[1].clamp(0.0, 1.0)) * 255.0,
        f64::from(a[2].clamp(0.0, 1.0)) * 255.0,
        f64::from(b[0].clamp(0.0, 1.0)) * 255.0,
        f64::from(b[1].clamp(0.0, 1.0)) * 255.0,
        f64::from(b[2].clamp(0.0, 1.0)) * 255.0,
        f64::from(t),
        &mut out,
    );
    [
        (out[0] / 255.0) as f32,
        (out[1] / 255.0) as f32,
        (out[2] / 255.0) as f32,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A LEI: azul com amarelo dá VERDE, não o cinzento que a mistura de LUZ dá. É o caso-bandeira
    /// do modelo, e é o que o dono pediu para os três meios.
    #[test]
    fn azul_com_amarelo_da_verde_e_nao_cinzento() {
        let azul = [26u8, 64, 217];
        let amarelo = [250u8, 230, 26];
        let m = mix_srgb8(azul, amarelo, 0.5);
        assert!(
            m[1] > m[0] && m[1] > m[2],
            "o verde tem de dominar a mistura, e leu {m:?}"
        );
        // O controlo: a média LINEAR dos mesmos dois é um cinzento sem dominante.
        let cinza = [
            (u16::from(azul[0]) + u16::from(amarelo[0])) / 2,
            (u16::from(azul[1]) + u16::from(amarelo[1])) / 2,
            (u16::from(azul[2]) + u16::from(amarelo[2])) / 2,
        ];
        assert!(
            cinza[1] < cinza[0] + 20 && cinza[1] > cinza[0].saturating_sub(20),
            "o controlo tem de ser um cinzento (sem dominante verde), e leu {cinza:?}"
        );
    }

    /// ⚠️ A metade que torna a troca de lei SEGURA: duas cores iguais devolvem a entrada AO BIT, em
    /// todo peso. Sem ela um traço de uma cor deriva um byte por dab.
    #[test]
    fn cores_iguais_devolvem_a_entrada_ao_bit() {
        for c in [[0u8, 0, 0], [26, 64, 217], [128, 128, 128], [255, 255, 255]] {
            for t in [0.0f32, 0.1, 0.5, 0.9, 1.0] {
                assert_eq!(mix_srgb8(c, c, t), c, "cor {c:?} peso {t}");
            }
        }
    }

    /// As pontas são a entrada exacta — a lei é um caminho entre duas tintas, não uma terceira.
    #[test]
    fn as_pontas_sao_exactas() {
        let a = [26u8, 64, 217];
        let b = [250u8, 230, 26];
        assert_eq!(mix_srgb8(a, b, 0.0), a);
        assert_eq!(mix_srgb8(a, b, 1.0), b);
    }
}
